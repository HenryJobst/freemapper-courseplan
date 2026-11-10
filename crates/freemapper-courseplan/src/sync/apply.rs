//! Applies the resolved merge to the project (D6, D8, D10, D11).
//!
//! The result starts as a clone of the project. Where the Purple Pen side wins, its value is written into
//! the project, with every identity translated to the project's names. Renames follow into accepted
//! findings, leg references and the course lists of special objects. Data only courseplad holds never
//! enters a field, so it stays as it is.

use std::collections::{HashMap, HashSet};

use freemapper_courseplan_model::{
    Control, ControlCode, Course, CourseControl, CourseKind, Event, MapPoint, NumberPlacement,
    Side, SpecialObject, validate,
};

use super::SyncError;
use super::fields::{ControlField, CourseField, FieldKey, FieldOutcome, FieldValue, ObjectKind};
use super::matching::Matching;
use super::merge3::Pick;
use super::view::Views;
use crate::edit::{rename_control, rename_course};
use crate::ppen_export::{Deletions, ExportNote};

/// Something the sync did that the user should hear about.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncNote {
    /// The project deleted the control, but a course still visits it, so it stays.
    DeletionKept { control: String, course: String },
    /// Bends or a marked-route flag changed on a leg that no course runs any more.
    LegChangeDropped { from: String, to: String },
    /// A new object from Purple Pen got another name, because the project already uses its own.
    NewNameTaken { from: String, to: String },
    /// The stored base copy was not used.
    BaseIgnored { why: &'static str },
    /// The first sync of this project with this file: there is no base copy yet, so every difference
    /// counts as a conflict and nothing is deleted. Not a problem.
    FirstSync,
    /// The stored base copy was repaired.
    BaseRepaired,
    /// An object the project holds that Purple Pen's normal courses cannot.
    NotWritten(ExportNote),
}

/// The merged event and what the writer must delete from the Purple Pen file.
#[derive(Debug, Clone)]
pub struct Applied {
    pub event: Event,
    /// Purple Pen's ids (in the linked file) of the objects the project deleted.
    pub delete: Deletions,
    pub notes: Vec<SyncNote>,
    /// The project's final name of each matched object, by index into the `Matching` vectors
    /// (`None`: the project has no such object). The sync uses it to give each object theirs' id.
    pub names: AppliedNames,
}

/// Final project names per `Matching` triple.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppliedNames {
    pub controls: Vec<Option<String>>,
    pub specials: Vec<Option<String>>,
    pub courses: Vec<Option<String>>,
}

fn mm(um: i64) -> f64 {
    um as f64 / 1000.0
}

fn map_points(points: &[(i64, i64)]) -> Vec<MapPoint> {
    points
        .iter()
        .map(|&(x, y)| MapPoint { x: mm(x), y: mm(y) })
        .collect()
}

fn is_end(s: &SpecialObject) -> bool {
    matches!(
        s,
        SpecialObject::Start { .. } | SpecialObject::Finish { .. }
    )
}

fn set_id(s: &mut SpecialObject, new: &str) {
    match s {
        SpecialObject::Start { id, .. }
        | SpecialObject::Finish { id, .. }
        | SpecialObject::MarkedRoute { id, .. }
        | SpecialObject::OutOfBoundsArea { id, .. }
        | SpecialObject::ForbiddenRoute { id, .. }
        | SpecialObject::DangerousArea { id, .. }
        | SpecialObject::FirstAid { id, .. }
        | SpecialObject::Water { id, .. } => new.clone_into(id),
    }
}

fn courses_mut(s: &mut SpecialObject) -> Option<&mut Vec<String>> {
    match s {
        SpecialObject::Start { .. } | SpecialObject::Finish { .. } => None,
        SpecialObject::MarkedRoute { courses, .. }
        | SpecialObject::OutOfBoundsArea { courses, .. }
        | SpecialObject::ForbiddenRoute { courses, .. }
        | SpecialObject::DangerousArea { courses, .. }
        | SpecialObject::FirstAid { courses, .. }
        | SpecialObject::Water { courses, .. } => Some(courses),
    }
}

fn set_points(s: &mut SpecialObject, points: &[(i64, i64)]) {
    let p = map_points(points);
    match s {
        SpecialObject::Start { map_mm, .. }
        | SpecialObject::Finish { map_mm, .. }
        | SpecialObject::FirstAid { map_mm, .. }
        | SpecialObject::Water { map_mm, .. } => {
            if let Some(first) = p.first() {
                *map_mm = first.clone();
            }
        }
        SpecialObject::MarkedRoute { points, .. }
        | SpecialObject::ForbiddenRoute { points, .. } => {
            *points = p;
        }
        SpecialObject::OutOfBoundsArea { polygon, .. }
        | SpecialObject::DangerousArea { polygon, .. } => *polygon = p,
    }
}

/// `name`, or the first free `name` with a counted tail: `CROSS1` gives `CROSS2`.
fn free_name(name: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_owned();
    }
    let prefix = name.trim_end_matches(|c: char| c.is_ascii_digit());
    let start = name[prefix.len()..]
        .parse::<u64>()
        .unwrap_or(0)
        .saturating_add(1);
    (start..)
        .map(|n| format!("{prefix}{n}"))
        .find(|candidate| !taken(candidate))
        .unwrap_or_else(|| format!("{name}x"))
}

fn index_of(
    len: usize,
    triples: &[super::matching::Triple],
    get: fn(&super::matching::Triple) -> Option<usize>,
) -> Vec<Option<usize>> {
    let mut out = vec![None; len];
    for (identity, t) in triples.iter().enumerate() {
        if let Some(i) = get(t).filter(|&i| i < len) {
            out[i] = Some(identity);
        }
    }
    out
}

struct Ctx<'a> {
    v: &'a Views,
    m: &'a Matching,
    ev: Event,
    /// The project's current name per identity; `None` when the project has no such object.
    ctl: Vec<Option<String>>,
    sp: Vec<Option<String>>,
    co: Vec<Option<String>>,
    /// Identity of each object of theirs.
    t_ctl: Vec<Option<usize>>,
    t_sp: Vec<Option<usize>>,
    t_co: Vec<Option<usize>>,
    notes: Vec<SyncNote>,
    /// Identities of controls and specials that were deleted by the project and put back.
    restored_ctl: HashSet<usize>,
    restored_sp: HashSet<usize>,
    added_courses: HashSet<String>,
    added_specials: Vec<(usize, String)>,
    /// Codes of controls and special objects in the project, and the position of each control. Built after
    /// the renames (`index_names`) and kept up to date by the additions, so adding n objects stays linear.
    taken_codes: HashSet<String>,
    control_pos: HashMap<String, usize>,
    /// The first control of theirs per code.
    theirs_by_code: HashMap<String, usize>,
}

impl<'a> Ctx<'a> {
    fn new(project: &Event, v: &'a Views, m: &'a Matching) -> Self {
        let names = super::view::project_names(project, &v.ours, &v.ours_ids);
        let by_triple = |triples: &[super::matching::Triple], names: &[Option<String>]| {
            triples
                .iter()
                .map(|t| t.ours.and_then(|oi| names.get(oi).cloned().flatten()))
                .collect::<Vec<_>>()
        };
        let ctl = by_triple(&m.controls, &names.controls);
        let sp = by_triple(&m.specials, &names.specials);
        let co = by_triple(&m.courses, &names.courses);
        Ctx {
            v,
            m,
            ev: project.clone(),
            ctl,
            sp,
            co,
            t_ctl: index_of(v.theirs.controls.len(), &m.controls, |t| t.theirs),
            t_sp: index_of(v.theirs.specials.len(), &m.specials, |t| t.theirs),
            t_co: index_of(v.theirs.courses.len(), &m.courses, |t| t.theirs),
            notes: Vec::new(),
            restored_ctl: HashSet::new(),
            restored_sp: HashSet::new(),
            added_courses: HashSet::new(),
            added_specials: Vec::new(),
            taken_codes: HashSet::new(),
            control_pos: HashMap::new(),
            theirs_by_code: {
                let mut by_code = HashMap::new();
                for (i, c) in v.theirs.controls.iter().enumerate() {
                    by_code.entry(c.code.0.clone()).or_insert(i);
                }
                by_code
            },
        }
    }

    /// Builds the name indexes of the additions; the renames are done by then.
    fn index_names(&mut self) {
        self.taken_codes = self
            .ev
            .controls
            .iter()
            .map(|c| c.code.0.clone())
            .chain(self.ev.specials.iter().map(|s| s.id().to_owned()))
            .collect();
        self.control_pos.clear();
        for (i, c) in self.ev.controls.iter().enumerate() {
            self.control_pos.entry(c.code.0.clone()).or_insert(i);
        }
    }

    fn code_taken(&self, name: &str) -> bool {
        self.taken_codes.contains(name)
    }

    fn course_idx(&self, name: &str) -> Option<usize> {
        self.ev.courses.iter().position(|c| c.name == name)
    }

    fn control_idx(&self, name: &str) -> Option<usize> {
        self.ev.controls.iter().position(|c| c.code.0 == name)
    }

    /// The theirs object of a control identity, else the base's.
    fn source_control(&self, id: usize) -> Option<&'a Control> {
        let t = self.m.controls.get(id)?;
        t.theirs
            .and_then(|i| self.v.theirs.controls.get(i))
            .or_else(|| t.base.and_then(|i| self.v.base.controls.get(i)))
    }

    fn source_special(&self, idx: usize) -> Option<&'a SpecialObject> {
        let t = self.m.specials.get(idx)?;
        t.theirs
            .and_then(|i| self.v.theirs.specials.get(i))
            .or_else(|| t.base.and_then(|i| self.v.base.specials.get(i)))
    }

    /// Adds a copy of a control of the Purple Pen side to the project, under a free name.
    fn add_control(&mut self, id: usize, mut c: Control) -> String {
        let from = c.code.0.clone();
        let name = free_name(&from, |n| self.code_taken(n));
        if name != from {
            self.notes.push(SyncNote::NewNameTaken {
                from,
                to: name.clone(),
            });
        }
        c.code = ControlCode(name.clone());
        self.taken_codes.insert(name.clone());
        self.control_pos
            .entry(name.clone())
            .or_insert(self.ev.controls.len());
        self.ev.controls.push(c);
        self.ctl[id] = Some(name.clone());
        name
    }

    fn add_special(&mut self, idx: usize, mut s: SpecialObject) -> String {
        let from = s.id().to_owned();
        let name = free_name(&from, |n| self.code_taken(n));
        if name != from {
            self.notes.push(SyncNote::NewNameTaken {
                from,
                to: name.clone(),
            });
        }
        set_id(&mut s, &name);
        self.taken_codes.insert(name.clone());
        if !is_end(&s) {
            self.added_specials.push((idx, name.clone()));
        }
        self.ev.specials.push(s);
        self.sp[idx] = Some(name.clone());
        name
    }

    /// The project's code for a control identity. A control the project deleted, but a course of the
    /// merge visits, comes back with the note `DeletionKept` (D11).
    fn need_control(&mut self, id: usize, course: &str) -> Option<String> {
        if let Some(name) = &self.ctl[id] {
            return Some(name.clone());
        }
        let c = self.source_control(id)?.clone();
        let name = self.add_control(id, c);
        self.restored_ctl.insert(id);
        self.notes.push(SyncNote::DeletionKept {
            control: name.clone(),
            course: course.to_owned(),
        });
        Some(name)
    }

    fn need_special(&mut self, idx: usize, course: &str) -> Option<String> {
        if let Some(name) = &self.sp[idx] {
            return Some(name.clone());
        }
        let s = self.source_special(idx)?.clone();
        let name = self.add_special(idx, s);
        self.restored_sp.insert(idx);
        self.notes.push(SyncNote::DeletionKept {
            control: name.clone(),
            course: course.to_owned(),
        });
        Some(name)
    }

    /// Renames in two steps through names nobody uses, so two codes can swap in one sync.
    fn rename_controls(&mut self, list: &[(usize, String)]) {
        let temps: Vec<String> = (0..list.len()).map(|n| format!("\u{1}sync{n}")).collect();
        for ((id, _), temp) in list.iter().zip(&temps) {
            if let Some(from) = self.ctl[*id].clone() {
                rename_control(&mut self.ev, &ControlCode(from), &ControlCode(temp.clone()));
            }
        }
        for ((id, to), temp) in list.iter().zip(&temps) {
            if self.ctl[*id].is_some() {
                rename_control(
                    &mut self.ev,
                    &ControlCode(temp.clone()),
                    &ControlCode(to.clone()),
                );
                self.ctl[*id] = Some(to.clone());
            }
        }
    }

    fn rename_courses(&mut self, list: &[(usize, String)]) {
        let temps: Vec<String> = (0..list.len()).map(|n| format!("\u{1}sync{n}")).collect();
        for ((id, _), temp) in list.iter().zip(&temps) {
            if let Some(from) = self.co[*id].clone() {
                rename_course(&mut self.ev, &from, temp);
            }
        }
        for ((id, to), temp) in list.iter().zip(&temps) {
            if self.co[*id].is_some() {
                rename_course(&mut self.ev, temp, to);
                self.co[*id] = Some(to.clone());
            }
        }
    }

    /// Name of a stop identity: a control, or a special object after the controls.
    fn stop_name(&self, id: usize) -> Option<String> {
        if id < self.m.controls.len() {
            self.ctl[id].clone()
        } else {
            self.sp.get(id - self.m.controls.len())?.clone()
        }
    }

    /// Whether the special object of this index is a start (`Some(true)`), a finish or another kind.
    fn end_kind(&self, idx: usize) -> Option<bool> {
        let t = self.m.specials.get(idx)?;
        let s = t
            .ours
            .and_then(|i| self.v.ours.specials.get(i))
            .or_else(|| t.theirs.and_then(|i| self.v.theirs.specials.get(i)))
            .or_else(|| t.base.and_then(|i| self.v.base.specials.get(i)))?;
        match s {
            SpecialObject::Start { .. } => Some(true),
            SpecialObject::Finish { .. } => Some(false),
            _ => None,
        }
    }

    /// Replaces the stops of an individual course; the legs that still run between the same stops keep their
    /// bends, and so does the n-th visit of a code its number.
    fn set_stops(&mut self, name: &str, ids: &[usize]) {
        let Some(ci) = self.course_idx(name) else {
            return;
        };
        let mut start = None;
        let mut finish = None;
        let mut codes: Vec<String> = Vec::new();
        for &id in ids {
            if id < self.m.controls.len() {
                if let Some(code) = self.need_control(id, name) {
                    codes.push(code);
                }
            } else {
                let idx = id - self.m.controls.len();
                match self.end_kind(idx) {
                    Some(true) => start = self.need_special(idx, name),
                    Some(false) => finish = self.need_special(idx, name),
                    None => {}
                }
            }
        }
        let course = &mut self.ev.courses[ci];
        let CourseKind::Individual { controls } = &mut course.kind else {
            return;
        };
        let old_start = course.start.clone();
        let old_finish = course.finish.clone();
        let mut old_legs: HashMap<(String, String), (Vec<MapPoint>, bool)> = HashMap::new();
        let mut old_numbers: HashMap<(String, u32), Option<NumberPlacement>> = HashMap::new();
        let mut visits: HashMap<&str, u32> = HashMap::new();
        for (k, cc) in controls.iter().enumerate() {
            let from = if k == 0 {
                &old_start
            } else {
                &controls[k - 1].code.0
            };
            let _ = from;
            let to = controls.get(k + 1).map_or(&old_finish, |n| &n.code.0);
            old_legs.insert(
                (cc.code.0.clone(), to.clone()),
                (cc.bends.clone(), cc.taped),
            );
            let visit = visits.entry(cc.code.0.as_str()).or_insert(0);
            *visit += 1;
            old_numbers.insert((cc.code.0.clone(), *visit), cc.number.clone());
        }
        let first = codes.first().cloned().unwrap_or_else(|| old_finish.clone());
        let old_first = controls
            .first()
            .map_or_else(|| old_finish.clone(), |c| c.code.0.clone());
        let (start_bends, start_taped) = if old_first == first {
            (course.start_bends.clone(), course.start_taped)
        } else {
            (Vec::new(), false)
        };
        let new_finish = finish.unwrap_or(old_finish);
        let mut seen: HashMap<String, u32> = HashMap::new();
        let mut out = Vec::new();
        for (k, code) in codes.iter().enumerate() {
            let visit = seen.entry(code.clone()).or_insert(0);
            *visit += 1;
            let next = codes.get(k + 1).unwrap_or(&new_finish);
            let (bends, taped) = old_legs
                .get(&(code.clone(), next.clone()))
                .cloned()
                .unwrap_or_default();
            out.push(CourseControl {
                code: ControlCode(code.clone()),
                number: old_numbers.get(&(code.clone(), *visit)).cloned().flatten(),
                bends,
                taped,
            });
        }
        *controls = out;
        course.start_bends = start_bends;
        course.start_taped = start_taped;
        if let Some(s) = start {
            course.start = s;
        }
        course.finish = new_finish;
    }

    /// The model has one central placement per control and cannot say "automatic in this course" while the
    /// control has one. When Purple Pen shows a stop automatically, the central placement moves onto every
    /// other visit of the control that has none, and the control keeps none (I1). `except` is the course
    /// index and the 1-based visit that stays without one.
    fn demote_central(&mut self, code: &str, except: Option<(usize, usize)>) {
        let Some(&i) = self.control_pos.get(code) else {
            return;
        };
        let Some(central) = self.ev.controls.get_mut(i).and_then(|c| c.number.take()) else {
            return;
        };
        // Every course kind that holds per-stop placements: Individual, Team (mandatory stops) and Relay
        // (common and fork parts). Score stops have none.
        for (ci, course) in self.ev.courses.iter_mut().enumerate() {
            let lists: Vec<&mut Vec<CourseControl>> = match &mut course.kind {
                CourseKind::Individual { controls } => vec![controls],
                CourseKind::Team { mandatory, .. } => vec![mandatory],
                CourseKind::Relay { parts, .. } => parts
                    .iter_mut()
                    .flat_map(|p| match p {
                        freemapper_courseplan_model::RelayPart::Common { controls } => {
                            vec![controls]
                        }
                        freemapper_courseplan_model::RelayPart::Fork { variants } => {
                            variants.iter_mut().collect()
                        }
                    })
                    .collect(),
                CourseKind::Score { .. } => continue,
            };
            let mut visit = 0;
            for cc in lists
                .into_iter()
                .flat_map(|l| l.iter_mut())
                .filter(|cc| cc.code.0 == code)
            {
                visit += 1;
                if cc.number.is_none() && except != Some((ci, visit)) {
                    cc.number = Some(central.clone());
                }
            }
        }
    }

    /// Adds a course of the Purple Pen side, without classes. Its stops are translated to the project's
    /// names.
    fn add_course(&mut self, id: usize, theirs: &Course) {
        let from = theirs.name.clone();
        let name = free_name(&from, |n| self.ev.courses.iter().any(|c| c.name == n));
        if name != from {
            self.notes.push(SyncNote::NewNameTaken {
                from,
                to: name.clone(),
            });
        }
        let mut course = theirs.clone();
        course.name.clone_from(&name);
        course.classes = Vec::new();
        course.print_area = None;
        let t_special = |ctx: &Self, id: &str| {
            ctx.v
                .theirs
                .specials
                .iter()
                .position(|s| s.id() == id)
                .and_then(|i| ctx.t_sp[i])
        };
        if let Some(idx) = t_special(self, &theirs.start)
            && let Some(n) = self.need_special(idx, &name)
        {
            course.start = n;
        }
        if let Some(idx) = t_special(self, &theirs.finish)
            && let Some(n) = self.need_special(idx, &name)
        {
            course.finish = n;
        }
        if let CourseKind::Individual { controls } = &mut course.kind {
            // What each stop shows in Purple Pen: its own placement, else the control's.
            let mut out = Vec::new();
            for cc in controls.iter() {
                let ti = self.theirs_by_code.get(&cc.code.0).copied();
                let Some(cid) = ti.and_then(|i| self.t_ctl[i]) else {
                    continue;
                };
                let Some(code) = self.need_control(cid, &name) else {
                    continue;
                };
                let mut cc = cc.clone();
                let shown = cc
                    .number
                    .clone()
                    .or_else(|| ti.and_then(|i| self.v.theirs.controls[i].number.clone()));
                cc.code = ControlCode(code.clone());
                // I1: a stop Purple Pen places automatically must not pick up another course's placement.
                if shown.is_none() {
                    self.demote_central(&code, None);
                }
                cc.number = shown;
                out.push(cc);
            }
            // A placement equal to the control's central one is stored as "central".
            for cc in &mut out {
                let central = self
                    .control_pos
                    .get(&cc.code.0)
                    .and_then(|&i| self.ev.controls[i].number.as_ref());
                if central.is_some() && cc.number.as_ref() == central {
                    cc.number = None;
                }
            }
            *controls = out;
        }
        self.ev.courses.push(course);
        self.co[id] = Some(name.clone());
        self.added_courses.insert(name);
    }

    /// Takes a course list of the Purple Pen side: its identities as the project's names. Entries for
    /// courses the Purple Pen view never held stay.
    fn course_list(&self, ids: &[usize], old: &[String]) -> Vec<String> {
        let viewed: HashSet<&str> = self
            .m
            .courses
            .iter()
            .enumerate()
            .filter(|(_, t)| t.ours.is_some())
            .filter_map(|(i, _)| self.co[i].as_deref())
            .collect();
        let mut out: Vec<String> = ids.iter().filter_map(|&i| self.co[i].clone()).collect();
        out.extend(old.iter().filter(|n| !viewed.contains(n.as_str())).cloned());
        out.dedup();
        out
    }

    fn visited_by(&self, code: &str) -> Option<String> {
        let uses = |cs: &[CourseControl]| cs.iter().any(|c| c.code.0 == code);
        self.ev
            .courses
            .iter()
            .find(|c| match &c.kind {
                CourseKind::Individual { controls } => uses(controls),
                CourseKind::Team {
                    mandatory, free, ..
                } => uses(mandatory) || free.iter().any(|f| f.0 == code),
                CourseKind::Relay { parts, .. } => parts.iter().any(|p| match p {
                    freemapper_courseplan_model::RelayPart::Common { controls } => uses(controls),
                    freemapper_courseplan_model::RelayPart::Fork { variants } => {
                        variants.iter().any(|v| uses(v))
                    }
                }),
                CourseKind::Score { controls, .. } => controls.iter().any(|c| c.code.0 == code),
            })
            .map(|c| c.name.clone())
    }

    /// The chain of an individual course as names: start, controls, finish.
    fn chain(course: &Course) -> Vec<String> {
        let mut out = vec![course.start.clone()];
        if let CourseKind::Individual { controls } = &course.kind {
            out.extend(controls.iter().map(|c| c.code.0.clone()));
        }
        out.push(course.finish.clone());
        out
    }

    /// For every leg `(from, to)` the individual courses that run it: `(course index, position k in the
    /// chain)`. Built once per apply; the stops do not change while the legs are drawn.
    fn leg_index(&self) -> LegIndex {
        let mut index: LegIndex = HashMap::new();
        for (ci, course) in self.ev.courses.iter().enumerate() {
            if !matches!(course.kind, CourseKind::Individual { .. }) {
                continue;
            }
            let chain = Self::chain(course);
            for (k, pair) in chain.windows(2).enumerate() {
                index
                    .entry((pair[0].clone(), pair[1].clone()))
                    .or_default()
                    .push((ci, k));
            }
        }
        index
    }

    /// Draws a leg on every course that runs between the two stops; `only_added` limits it to the courses
    /// this sync added.
    fn set_leg(
        &mut self,
        index: &LegIndex,
        (a, b): (String, String),
        bends: &[(i64, i64)],
        taped: bool,
        only_added: bool,
    ) {
        let bends = map_points(bends);
        for &(ci, k) in index.get(&(a, b)).into_iter().flatten() {
            let course = &mut self.ev.courses[ci];
            if only_added && !self.added_courses.contains(&course.name) {
                continue;
            }
            let CourseKind::Individual { controls } = &mut course.kind else {
                continue;
            };
            if k == 0 {
                course.start_bends.clone_from(&bends);
                course.start_taped = taped;
            } else {
                controls[k - 1].bends.clone_from(&bends);
                controls[k - 1].taped = taped;
            }
        }
    }
}

/// Legs to the courses and chain positions that run them.
type LegIndex = HashMap<(String, String), Vec<(usize, usize)>>;

/// The value of the winning side of an outcome.
fn winner(o: &FieldOutcome, take: bool) -> Option<&FieldValue> {
    if take {
        o.theirs.as_ref()
    } else {
        o.ours.as_ref()
    }
}

/// Applies the resolved merge to the project.
///
/// `sides` is the side picked per conflict key (see `decide::resolve`); a conflict without a side keeps the
/// project's value. The result passes `validate`.
///
/// # Errors
/// [`SyncError::Invalid`] when the merged event is not consistent.
pub fn apply(
    project: &Event,
    v: &Views,
    m: &Matching,
    outcomes: &[FieldOutcome],
    sides: &HashMap<String, Side>,
) -> Result<Applied, SyncError> {
    let mut cx = Ctx::new(project, v, m);
    let take: Vec<bool> = outcomes
        .iter()
        .map(|o| match o.pick {
            Pick::Theirs => true,
            Pick::Conflict => {
                sides.get(&super::decide::conflict_key(o, v, m)) == Some(&Side::Theirs)
            }
            _ => false,
        })
        .collect();

    // Renames first, all at once.
    let mut code_renames = Vec::new();
    let mut course_renames = Vec::new();
    for (o, &t) in outcomes.iter().zip(&take) {
        if !t {
            continue;
        }
        match (o.key, &o.theirs) {
            (FieldKey::Control(id, ControlField::Code), Some(FieldValue::Code(to)))
                if cx.ctl[id].as_ref().is_some_and(|c| c != to) =>
            {
                code_renames.push((id, to.clone()));
            }
            (FieldKey::Course(id, CourseField::Name), Some(FieldValue::Name(to)))
                if cx.co[id].as_ref().is_some_and(|c| c != to) =>
            {
                course_renames.push((id, to.clone()));
            }
            _ => {}
        }
    }
    cx.rename_controls(&code_renames);
    cx.rename_courses(&course_renames);
    cx.index_names();

    // Objects new in Purple Pen, or put back by a conflict: controls and special objects, then courses.
    for (o, &t) in outcomes.iter().zip(&take) {
        let FieldKey::Object(kind, id) = o.key else {
            continue;
        };
        if !t {
            continue;
        }
        let triple = match kind {
            ObjectKind::Control => m.controls[id],
            ObjectKind::Special => m.specials[id],
            ObjectKind::Course => m.courses[id],
        };
        let (Some(ti), None) = (triple.theirs, triple.ours) else {
            continue;
        };
        match kind {
            ObjectKind::Control => {
                cx.add_control(id, v.theirs.controls[ti].clone());
            }
            ObjectKind::Special => {
                cx.add_special(id, v.theirs.specials[ti].clone());
            }
            ObjectKind::Course => {}
        }
    }
    for (o, &t) in outcomes.iter().zip(&take) {
        if let (FieldKey::Object(ObjectKind::Course, id), true) = (o.key, t)
            && let (Some(ti), None) = (m.courses[id].theirs, m.courses[id].ours)
        {
            let theirs = v.theirs.courses[ti].clone();
            cx.add_course(id, &theirs);
        }
    }
    // The course lists of new special objects.
    for (idx, name) in cx.added_specials.clone() {
        let Some(ti) = m.specials[idx].theirs else {
            continue;
        };
        let ids: Vec<usize> = v.theirs.specials[ti]
            .courses()
            .iter()
            .filter_map(|n| v.theirs.courses.iter().position(|c| &c.name == n))
            .filter_map(|i| cx.t_co[i])
            .collect();
        let list = cx.course_list(&ids, &[]);
        if list.is_empty() && !v.theirs.specials[ti].courses().is_empty() {
            cx.ev.specials.retain(|s| s.id() != name);
            continue;
        }
        if let Some(s) = cx.ev.specials.iter_mut().find(|s| s.id() == name)
            && let Some(l) = courses_mut(s)
        {
            *l = list;
        }
    }

    // Stops, then what hangs on them.
    for (o, &t) in outcomes.iter().zip(&take) {
        let FieldKey::Course(cid, CourseField::Stops) = o.key else {
            continue;
        };
        let list = match (&o.merged, winner(o, t)) {
            (Some(FieldValue::Stops(l)), _) => l.clone(),
            (None, Some(FieldValue::Stops(l))) if t => l.clone(),
            _ => continue,
        };
        if let Some(name) = cx.co[cid].clone() {
            cx.set_stops(&name, &list);
        }
    }
    for (o, &t) in outcomes.iter().zip(&take) {
        if !t {
            continue;
        }
        match (o.key, &o.theirs) {
            (FieldKey::Control(id, field), theirs) => {
                let Some(ci) = cx.ctl[id].as_deref().and_then(|n| cx.control_idx(n)) else {
                    continue;
                };
                let c = &mut cx.ev.controls[ci];
                match (field, theirs) {
                    (ControlField::Kind, Some(FieldValue::Kind(k))) => c.kind = *k,
                    (ControlField::Position, Some(FieldValue::Point(x, y))) => {
                        c.map_mm = MapPoint {
                            x: mm(*x),
                            y: mm(*y),
                        };
                    }
                    (ControlField::Column(l), Some(FieldValue::Column(text))) => {
                        let d = &mut c.description;
                        let slot = match l {
                            'c' => &mut d.c,
                            'd' => &mut d.d,
                            'e' => &mut d.e,
                            'f' => &mut d.f,
                            'g' => &mut d.g,
                            _ => &mut d.h,
                        };
                        slot.clone_from(text);
                    }
                    _ => {}
                }
            }
            (FieldKey::Special(id), Some(FieldValue::Shape(points, courses))) => {
                let Some(name) = cx.sp.get(id).cloned().flatten() else {
                    continue;
                };
                let old: Vec<String> = cx
                    .ev
                    .specials
                    .iter()
                    .find(|s| s.id() == name)
                    .map(|s| s.courses().to_vec())
                    .unwrap_or_default();
                let list = cx.course_list(courses, &old);
                if let Some(s) = cx.ev.specials.iter_mut().find(|s| s.id() == name) {
                    set_points(s, points);
                    if let Some(l) = courses_mut(s) {
                        *l = list;
                    }
                }
            }
            (FieldKey::Course(id, field), theirs) => {
                let Some(ci) = cx.co[id].as_deref().and_then(|n| cx.course_idx(n)) else {
                    continue;
                };
                match (field, theirs) {
                    (CourseField::Climb, Some(FieldValue::Scale(s))) => {
                        cx.ev.courses[ci].climb_m = *s;
                    }
                    (CourseField::PrintScale, Some(FieldValue::Scale(s))) => {
                        cx.ev.courses[ci].planned_print_scale = *s;
                    }
                    (
                        CourseField::Number { control, visit },
                        Some(FieldValue::Number(placement)),
                    ) => {
                        let Some(code) = cx.ctl[control].clone() else {
                            continue;
                        };
                        // I1: automatic in Purple Pen here, so no central placement may reach this visit.
                        if placement.is_none() {
                            cx.demote_central(&code, Some((ci, visit as usize)));
                        }
                        let central = cx
                            .control_pos
                            .get(&code)
                            .and_then(|&i| cx.ev.controls[i].number.clone());
                        let value = placement.and_then(|(d, g)| {
                            let n = NumberPlacement {
                                direction: d as f64 / 100.0,
                                gap_mm: g as f64 / 100.0,
                            };
                            let same = central.as_ref().is_some_and(|c| {
                                (c.direction * 100.0).round() as i64 == d
                                    && (c.gap_mm * 100.0).round() as i64 == g
                            });
                            (!same).then_some(n)
                        });
                        if let CourseKind::Individual { controls } = &mut cx.ev.courses[ci].kind
                            && let Some(cc) = controls
                                .iter_mut()
                                .filter(|c| c.code.0 == code)
                                .nth(visit as usize - 1)
                        {
                            cc.number = value;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    let leg_index = cx.leg_index();
    for (o, &t) in outcomes.iter().zip(&take) {
        let FieldKey::Leg(a, b) = o.key else {
            continue;
        };
        let (Some(a), Some(b)) = (cx.stop_name(a), cx.stop_name(b)) else {
            continue;
        };
        if let Some(FieldValue::Leg(bends, taped)) = winner(o, t) {
            // Purple Pen's value wins on every drawing; the project's own value only reaches new courses.
            cx.set_leg(&leg_index, (a, b), bends, *taped, !t);
        }
    }

    // Deletions by Purple Pen.
    let mut gone_courses = Vec::new();
    for (o, &t) in outcomes.iter().zip(&take) {
        let FieldKey::Object(kind, id) = o.key else {
            continue;
        };
        let triple = match kind {
            ObjectKind::Control => m.controls[id],
            ObjectKind::Special => m.specials[id],
            ObjectKind::Course => m.courses[id],
        };
        if !t || triple.theirs.is_some() || triple.ours.is_none() {
            continue;
        }
        match kind {
            ObjectKind::Control => {
                let Some(name) = cx.ctl[id].clone() else {
                    continue;
                };
                if let Some(course) = cx.visited_by(&name) {
                    cx.notes.push(SyncNote::DeletionKept {
                        control: name,
                        course,
                    });
                } else {
                    cx.ev.controls.retain(|c| c.code.0 != name);
                    cx.ctl[id] = None;
                }
            }
            ObjectKind::Special => {
                if let Some(name) = cx.sp[id].clone()
                    && !cx
                        .ev
                        .courses
                        .iter()
                        .any(|c| c.start == name || c.finish == name)
                {
                    cx.ev.specials.retain(|s| s.id() != name);
                    cx.sp[id] = None;
                }
            }
            ObjectKind::Course => {
                if let Some(name) = cx.co[id].clone() {
                    gone_courses.push(name);
                    cx.co[id] = None;
                }
            }
        }
    }
    for name in &gone_courses {
        cx.ev.courses.retain(|c| &c.name != name);
        cx.ev.specials.retain_mut(|s| {
            let Some(list) = courses_mut(s) else {
                return true;
            };
            let had = !list.is_empty();
            list.retain(|c| c != name);
            !(had && list.is_empty())
        });
    }

    // Deletions by the project go to the Purple Pen file.
    let mut delete = Deletions::default();
    for (o, &t) in outcomes.iter().zip(&take) {
        let FieldKey::Object(kind, id) = o.key else {
            continue;
        };
        let triple = match kind {
            ObjectKind::Control => m.controls[id],
            ObjectKind::Special => m.specials[id],
            ObjectKind::Course => m.courses[id],
        };
        let (Some(ti), None) = (triple.theirs, triple.ours) else {
            continue;
        };
        if t {
            continue;
        }
        match kind {
            ObjectKind::Control => {
                if !cx.restored_ctl.contains(&id)
                    && let Some(pid) = v.theirs.controls[ti].ppen_id
                {
                    delete.controls.push(pid);
                }
            }
            ObjectKind::Special => {
                if !cx.restored_sp.contains(&id)
                    && let Some(pid) = v.theirs.specials[ti].ppen_id()
                {
                    if is_end(&v.theirs.specials[ti]) {
                        delete.controls.push(pid);
                    } else {
                        delete.specials.push(pid);
                    }
                }
            }
            ObjectKind::Course => {
                if let Some(pid) = v.theirs.courses[ti].ppen_id {
                    delete.courses.push(pid);
                }
            }
        }
    }

    // Bends that belonged to a leg no course runs any more (D6).
    let mut pairs: HashSet<(String, String)> = HashSet::new();
    for course in &cx.ev.courses {
        if matches!(course.kind, CourseKind::Individual { .. }) {
            let chain = Ctx::chain(course);
            pairs.extend(chain.windows(2).map(|w| (w[0].clone(), w[1].clone())));
        }
    }
    for (o, &t) in outcomes.iter().zip(&take) {
        let FieldKey::Leg(a, b) = o.key else {
            continue;
        };
        // A leg the winner removed counts with the other side's value: that one is dropped with the note.
        let value = winner(o, t).or(o.ours.as_ref()).or(o.theirs.as_ref());
        let changed = value.is_some_and(|w| {
            Some(w) != o.base.as_ref()
                && !matches!(w, FieldValue::Leg(bends, false) if bends.is_empty())
        });
        if !changed {
            continue;
        }
        if let (Some(na), Some(nb)) = (cx.stop_name(a), cx.stop_name(b))
            && !pairs.contains(&(na.clone(), nb.clone()))
        {
            cx.notes
                .push(SyncNote::LegChangeDropped { from: na, to: nb });
        }
    }

    for n in &v.skipped {
        cx.notes.push(SyncNote::NotWritten(n.clone()));
    }
    let errors = validate(&cx.ev);
    if !errors.is_empty() {
        return Err(SyncError::Invalid(errors));
    }
    Ok(Applied {
        event: cx.ev,
        delete,
        notes: cx.notes,
        names: AppliedNames {
            controls: cx.ctl,
            specials: cx.sp,
            courses: cx.co,
        },
    })
}

#[cfg(test)]
mod tests {
    use freemapper_courseplan_model::{
        AcceptedFinding, ControlDescription, ControlKind, EventMeta, LegRef, Sources,
    };

    use super::*;
    use crate::ppen_export::ExportIds;
    use crate::ppen_import::PpenImport;
    use crate::sync::decide::conflict_key;
    use crate::sync::fields::classify;
    use crate::sync::matching::match_sides;

    fn ctl(code: &str, id: u32, x: f64) -> Control {
        Control {
            code: ControlCode(code.to_owned()),
            kind: ControlKind::Control,
            map_mm: MapPoint { x, y: 0.0 },
            description: ControlDescription::default(),
            number: None,
            ppen_id: Some(id),
        }
    }

    fn cross(code: &str, id: u32, x: f64) -> Control {
        Control {
            kind: ControlKind::CrossingPoint,
            ..ctl(code, id, x)
        }
    }

    fn course(name: &str, id: u32, codes: &[&str]) -> Course {
        Course {
            name: name.to_owned(),
            classes: vec![],
            start: "S".to_owned(),
            finish: "F".to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: None,
            ppen_id: Some(id),
            kind: CourseKind::Individual {
                controls: codes
                    .iter()
                    .map(|c| CourseControl {
                        code: ControlCode((*c).to_owned()),
                        number: None,
                        bends: vec![],
                        taped: false,
                    })
                    .collect(),
            },
        }
    }

    fn ends() -> Vec<SpecialObject> {
        vec![
            SpecialObject::Start {
                id: "S".to_owned(),
                ppen_id: Some(100),
                map_mm: MapPoint { x: -5.0, y: 0.0 },
            },
            SpecialObject::Finish {
                id: "F".to_owned(),
                ppen_id: Some(101),
                map_mm: MapPoint { x: 99.0, y: 0.0 },
            },
        ]
    }

    fn base_controls() -> Vec<Control> {
        vec![
            ctl("31", 31, 10.0),
            ctl("34", 34, 20.0),
            ctl("35", 35, 30.0),
            ctl("36", 36, 40.0),
        ]
    }

    fn import(controls: Vec<Control>, courses: Vec<Course>) -> PpenImport {
        PpenImport {
            controls,
            specials: ends(),
            courses,
            notes: vec![],
        }
    }

    fn base() -> PpenImport {
        import(
            base_controls(),
            vec![course("A", 1, &["31", "34", "35", "36"])],
        )
    }

    fn project(imp: &PpenImport) -> Event {
        Event {
            format: 1,
            event: EventMeta {
                title: "t".to_owned(),
                date: None,
                discipline: Default::default(),
                rule_profile: "wkb-2026".to_owned(),
                rule_overlays: vec![],
                start_window_min: None,
            },
            sources: Sources {
                map: "m.omap".into(),
                ppen: None,
            },
            classes: vec![],
            controls: imp.controls.clone(),
            specials: imp.specials.clone(),
            courses: imp.courses.clone(),
            accepted_findings: vec![],
            sync: None,
        }
    }

    struct Run {
        v: Views,
        m: Matching,
        outcomes: Vec<FieldOutcome>,
        project: Event,
    }

    fn setup(project: Event, theirs: PpenImport) -> Run {
        let ours = PpenImport {
            controls: project.controls.clone(),
            specials: project.specials.clone(),
            courses: project
                .courses
                .iter()
                .filter(|c| matches!(c.kind, CourseKind::Individual { .. }))
                .cloned()
                .collect(),
            notes: vec![],
        };
        let ids = ExportIds {
            controls: ours
                .controls
                .iter()
                .filter_map(|c| Some((c.code.clone(), c.ppen_id?)))
                .collect(),
            courses: ours
                .courses
                .iter()
                .filter_map(|c| Some((c.name.clone(), c.ppen_id?)))
                .collect(),
            specials: ours
                .specials
                .iter()
                .filter_map(|s| Some((s.id().to_owned(), s.ppen_id()?)))
                .collect(),
        };
        let v = Views {
            base: base(),
            ours,
            theirs,
            ours_ids: ids,
            ours_names: Default::default(),
            skipped: vec![],
        };
        let m = match_sides(&v, true);
        let outcomes = classify(&v, &m, &project);
        Run {
            v,
            m,
            outcomes,
            project,
        }
    }

    fn run(r: &Run, sides: &HashMap<String, Side>) -> Applied {
        apply(&r.project, &r.v, &r.m, &r.outcomes, sides).unwrap()
    }

    fn moved(mut imp: PpenImport, code: &str, x: f64) -> PpenImport {
        imp.controls
            .iter_mut()
            .find(|c| c.code.0 == code)
            .unwrap()
            .map_mm
            .x = x;
        imp
    }

    fn at(e: &Event, code: &str) -> f64 {
        e.controls
            .iter()
            .find(|c| c.code.0 == code)
            .unwrap()
            .map_mm
            .x
    }

    fn none() -> HashMap<String, Side> {
        HashMap::new()
    }

    #[test]
    fn theirs_position_lands_ours_stays() {
        let p = project(&moved(base(), "31", 11.0));
        let r = setup(p, moved(base(), "34", 22.0));
        let a = run(&r, &none());
        assert!((at(&a.event, "34") - 22.0).abs() < 1e-9);
        assert!((at(&a.event, "31") - 11.0).abs() < 1e-9);
    }

    #[test]
    fn ppen_recode_follows_into_findings() {
        let mut p = project(&base());
        p.accepted_findings = vec![AcceptedFinding {
            rule: "r".to_owned(),
            controls: vec![ControlCode("36".to_owned())],
            course: None,
            legs: vec![LegRef {
                course: "A".to_owned(),
                after: 4,
                from: "36".to_owned(),
                to: "F".to_owned(),
            }],
            note: "n".to_owned(),
        }];
        let mut theirs = base();
        theirs.controls[3].code = ControlCode("46".to_owned());
        if let CourseKind::Individual { controls } = &mut theirs.courses[0].kind {
            controls[3].code = ControlCode("46".to_owned());
        }
        let a = run(&setup(p, theirs), &none());
        assert_eq!(a.event.accepted_findings[0].legs[0].from, "46");
        assert_eq!(a.event.accepted_findings[0].controls[0].0, "46");
        let CourseKind::Individual { controls } = &a.event.courses[0].kind else {
            unreachable!()
        };
        assert_eq!(controls[3].code.0, "46");
    }

    #[test]
    fn ppen_course_rename_keeps_classes() {
        let mut p = project(&base());
        p.courses[0].classes = vec!["Offen".to_owned()];
        let mut theirs = base();
        theirs.courses[0].name = "Lang".to_owned();
        let a = run(&setup(p, theirs), &none());
        assert_eq!(a.event.courses[0].name, "Lang");
        assert_eq!(a.event.courses[0].classes, vec!["Offen".to_owned()]);
    }

    #[test]
    fn new_ppen_course_is_added_without_classes() {
        let mut theirs = base();
        theirs.courses.push(course("B", 2, &["31", "34"]));
        let a = run(&setup(project(&base()), theirs), &none());
        let b = a.event.courses.iter().find(|c| c.name == "B").unwrap();
        assert!(b.classes.is_empty());
        assert_eq!(b.ppen_id, Some(2));
    }

    #[test]
    fn new_ppen_crossing_point_gets_a_free_name() {
        let mut p = project(&base());
        p.controls.push(cross("CROSS1", 77, 50.0));
        let mut theirs = base();
        theirs.controls.push(cross("CROSS1", 80, 60.0));
        let a = run(&setup(p, theirs), &none());
        let c = a
            .event
            .controls
            .iter()
            .find(|c| c.code.0 == "CROSS2")
            .unwrap();
        assert!((c.map_mm.x - 60.0).abs() < 1e-9);
        assert!(a.notes.contains(&SyncNote::NewNameTaken {
            from: "CROSS1".to_owned(),
            to: "CROSS2".to_owned()
        }));
    }

    fn without_35() -> Event {
        let mut p = project(&base());
        p.controls.retain(|c| c.code.0 != "35");
        if let CourseKind::Individual { controls } = &mut p.courses[0].kind {
            controls.retain(|c| c.code.0 != "35");
        }
        p
    }

    #[test]
    fn ours_deletion_goes_to_deletions() {
        let a = run(&setup(without_35(), base()), &none());
        assert_eq!(a.delete.controls, vec![35]);
        assert!(a.event.controls.iter().all(|c| c.code.0 != "35"));
    }

    #[test]
    fn used_control_is_not_deleted() {
        let mut theirs = base();
        theirs.courses.push(course("B", 2, &["31", "35"]));
        let a = run(&setup(without_35(), theirs), &none());
        assert!(a.event.controls.iter().any(|c| c.code.0 == "35"));
        assert!(a.delete.controls.is_empty());
        assert!(a.notes.contains(&SyncNote::DeletionKept {
            control: "35".to_owned(),
            course: "B".to_owned()
        }));
    }

    #[test]
    fn team_course_survives() {
        let mut p = project(&base());
        let mut team = course("T", 9, &[]);
        team.ppen_id = None;
        team.kind = CourseKind::Team {
            team_size: 2,
            mandatory: vec![CourseControl {
                code: ControlCode("31".to_owned()),
                number: None,
                bends: vec![],
                taped: false,
            }],
            free: vec![ControlCode("34".to_owned())],
        };
        p.courses.push(team.clone());
        let a = run(&setup(p, moved(base(), "36", 41.0)), &none());
        assert_eq!(a.event.courses.iter().find(|c| c.name == "T"), Some(&team));
    }

    fn conflict_run(side: Option<Side>) -> Applied {
        let p = project(&moved(base(), "34", 21.0));
        let r = setup(p, moved(base(), "34", 22.0));
        let o = r
            .outcomes
            .iter()
            .find(|o| o.pick == Pick::Conflict)
            .unwrap();
        let mut sides = HashMap::new();
        if let Some(s) = side {
            sides.insert(conflict_key(o, &r.v, &r.m), s);
        }
        run(&r, &sides)
    }

    #[test]
    fn taken_ours_on_conflict_keeps_project_value() {
        let a = conflict_run(Some(Side::Ours));
        assert!((at(&a.event, "34") - 21.0).abs() < 1e-9);
    }

    #[test]
    fn taken_theirs_on_conflict_sets_theirs_value() {
        let a = conflict_run(Some(Side::Theirs));
        assert!((at(&a.event, "34") - 22.0).abs() < 1e-9);
    }
}
