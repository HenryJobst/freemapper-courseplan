//! The overlay of a patch: the base file's own values wherever the project still equals what the import
//! made of them.
//!
//! The import rounds every point to the map's µm, a number offset to a placement of 0.01° and 0.01 mm, and
//! the climb to whole metres, and it leaves out what the model cannot hold. Mapping the project back
//! therefore never gives the base file's values exactly, and the core writer, which compares exactly, would
//! rewrite every object. So the base file is imported onto the same map as the project, and each project
//! object is compared with the import's object it pairs with: by Purple Pen id when the base is the
//! project's linked file, else by code, name or special object id. Where the two are equal, the base file's
//! object goes into the event as the reader read it, and the writer keeps its bytes. Where they differ, the
//! object is the project's, and the fields the project does not hold, or holds as it was imported, come
//! from the paired base object:
//!
//! - Controls: the position if it did not move, the description if it did not change, else column F text
//!   (normal controls only). Starts and finishes keep their description.
//! - Courses: secondary title, course length and family; the print scale and `CourseObjRatio`, and with
//!   them each number offset whose placement and sequence number did not change, when the planned print
//!   scale is the imported one; the climb if it is the imported one; per stop its map exchange and map
//!   flip. A leg whose bends and tape did not change is the base file's, with its gaps (and partial
//!   flagging, which the writer keeps for unchanged bends) and with the map exchange controls the import
//!   turned into bends. A leg whose bends did not change but whose tape did keeps its gaps.
//! - Circle gaps belong to a control and a scale in Purple Pen, shared by every course: each stop gets the
//!   base file's gaps of its control at the scale its course uses them at. They are angles around the
//!   circle, so they stay when the control moves *(Claude's call)*.
//! - Special objects: the base file's when unchanged.
//!
//! Map exchange controls that a course of the event visits are taken from the base file, since the project
//! has none.

use std::collections::{HashMap, HashSet};

use freemapper_course::{
    Control as PControl, ControlKind as PKind, Course as PCourse, Description, ItemScaling, Leg,
    RawEvent, RawPoint, Special, Stop,
};
use freemapper_courseplan_model::{
    Control, ControlCode, ControlKind, Course, CourseKind, Event, MapPoint, NumberPlacement,
    SpecialObject,
};

use freemapper_course::model::MAX_TOTAL_POINTS;

use super::{PpenExportError, PrintScale, ToRaw, raw_course};
use crate::map::MapKind;
use crate::ppen_import::PpenImport;

/// The print scale Purple Pen assumes for a course without one (its `CourseView` default).
const DEFAULT_PRINT_SCALE: u32 = 15000;

/// The scale whose circle gaps a course of print scale `print` uses in `base` (Purple Pen's
/// `CourseView.CircleGapScale`, as the reader keys them): the print scale for item scaling none, else the
/// map scale or 1:15000.
fn gap_key(base: &RawEvent, print: Option<u32>) -> u32 {
    match base.source.item_scaling.unwrap_or_default() {
        ItemScaling::None => print.unwrap_or(DEFAULT_PRINT_SCALE),
        ItemScaling::RelativeToMap => base.source.map_scale.unwrap_or(0),
        ItemScaling::RelativeTo15000 => DEFAULT_PRINT_SCALE,
    }
}

/// Where an object of the written event comes from.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Origin {
    /// A control of the project.
    Control(ControlCode),
    /// A start or finish of the project, by its id.
    Special(String),
    /// The base file only (a map exchange control a course visits).
    Base,
}

/// The project's event on the base file, with the origin of each control, course and special object.
pub(super) struct Overlaid {
    pub event: RawEvent,
    pub controls: Vec<Origin>,
    /// Project course names.
    pub courses: Vec<String>,
    /// Project special object ids.
    pub specials: Vec<String>,
}

/// What the overlay needs of the export beside the objects.
pub(super) struct Frame {
    pub kind: MapKind,
    pub linked: bool,
}

/// The origins of the objects `to_raw` mapped, before any overlay: starts and finishes, then the
/// project's controls; the written individual courses; the written special objects.
pub(super) fn origins(project: &Event, mapped: &ToRaw) -> (Vec<Origin>, Vec<String>, Vec<String>) {
    let mut controls: Vec<Origin> = project
        .specials
        .iter()
        .filter(|s| is_start_or_finish(s))
        .map(|s| Origin::Special(s.id().to_owned()))
        .collect();
    controls.extend(
        project
            .controls
            .iter()
            .map(|c| Origin::Control(c.code.clone())),
    );
    let courses = mapped
        .event
        .courses
        .iter()
        .map(|c| c.name.clone())
        .collect();
    let specials = mapped
        .specials_from
        .iter()
        .map(|&i| project.specials[i].id().to_owned())
        .collect();
    (controls, courses, specials)
}

fn is_start_or_finish(s: &SpecialObject) -> bool {
    matches!(
        s,
        SpecialObject::Start { .. } | SpecialObject::Finish { .. }
    )
}

/// First index per key.
fn index<'a, K: std::hash::Hash + Eq>(
    keys: impl Iterator<Item = Option<K>> + 'a,
) -> HashMap<K, usize> {
    let mut map = HashMap::new();
    for (i, k) in keys.enumerate() {
        if let Some(k) = k {
            map.entry(k).or_insert(i);
        }
    }
    map
}

/// The base file as the reader read it and as the import made it, indexed.
struct Sides<'a> {
    base: &'a RawEvent,
    import: &'a PpenImport,
    linked: bool,
    base_control_by_id: HashMap<u32, usize>,
    base_control_by_code: HashMap<&'a str, usize>,
    base_course_by_id: HashMap<u32, usize>,
    base_course_by_name: HashMap<&'a str, usize>,
    base_special_by_id: HashMap<u32, usize>,
    import_control_by_id: HashMap<u32, usize>,
    import_control_by_code: HashMap<&'a str, usize>,
    /// Starts and finishes have the ids of controls, the others those of special objects: keyed by
    /// whether the object is a start or finish, and the id.
    import_special_by_id: HashMap<(bool, u32), usize>,
    import_special_by_name: HashMap<&'a str, usize>,
    import_course_by_id: HashMap<u32, usize>,
    import_course_by_name: HashMap<&'a str, usize>,
}

impl<'a> Sides<'a> {
    fn new(base: &'a RawEvent, import: &'a PpenImport, linked: bool) -> Sides<'a> {
        Sides {
            base,
            import,
            linked,
            base_control_by_id: index(base.controls.iter().map(|c| c.id)),
            base_control_by_code: index(base.controls.iter().map(|c| Some(c.code.as_str()))),
            base_course_by_id: index(base.courses.iter().map(|c| c.id)),
            base_course_by_name: index(base.courses.iter().map(|c| Some(c.name.as_str()))),
            base_special_by_id: index(base.specials.iter().map(Special::id)),
            import_control_by_id: index(import.controls.iter().map(|c| c.ppen_id)),
            import_control_by_code: index(import.controls.iter().map(|c| Some(c.code.0.as_str()))),
            import_special_by_id: index(
                import
                    .specials
                    .iter()
                    .map(|s| s.ppen_id().map(|id| (is_start_or_finish(s), id))),
            ),
            import_special_by_name: index(import.specials.iter().map(|s| Some(s.id()))),
            import_course_by_id: index(import.courses.iter().map(|c| c.ppen_id)),
            import_course_by_name: index(import.courses.iter().map(|c| Some(c.name.as_str()))),
        }
    }

    /// The import's object a project object pairs with: by Purple Pen id for the linked file, else (or
    /// when the id is not there) by `key`.
    fn pair(
        &self,
        ppen_id: Option<u32>,
        by_id: &HashMap<u32, usize>,
        key: &str,
        by_key: &HashMap<&str, usize>,
    ) -> Option<usize> {
        ppen_id
            .filter(|_| self.linked)
            .and_then(|id| by_id.get(&id))
            .or_else(|| by_key.get(key))
            .copied()
    }

    fn import_control(&self, c: &Control) -> Option<&'a Control> {
        let i = self.pair(
            c.ppen_id,
            &self.import_control_by_id,
            &c.code.0,
            &self.import_control_by_code,
        )?;
        Some(&self.import.controls[i])
    }

    fn import_special(&self, s: &SpecialObject) -> Option<usize> {
        let i = s
            .ppen_id()
            .filter(|_| self.linked)
            .and_then(|id| self.import_special_by_id.get(&(is_start_or_finish(s), id)))
            .or_else(|| self.import_special_by_name.get(s.id()))
            .copied()?;
        let same_kind =
            std::mem::discriminant(s) == std::mem::discriminant(&self.import.specials[i]);
        same_kind.then_some(i)
    }

    fn import_course(&self, c: &Course) -> Option<&'a Course> {
        let i = self.pair(
            c.ppen_id,
            &self.import_course_by_id,
            &c.name,
            &self.import_course_by_name,
        )?;
        Some(&self.import.courses[i])
    }

    /// The base control an imported control, start or finish came from.
    fn base_control(&self, ppen_id: Option<u32>, code: &str) -> Option<&'a PControl<RawPoint>> {
        let i = match ppen_id {
            Some(id) => self.base_control_by_id.get(&id),
            None => self.base_control_by_code.get(code),
        }?;
        Some(&self.base.controls[*i])
    }

    fn base_course(&self, imported: &Course) -> Option<usize> {
        match imported.ppen_id {
            Some(id) => self.base_course_by_id.get(&id),
            None => self.base_course_by_name.get(imported.name.as_str()),
        }
        .copied()
    }

    fn base_kind(&self, code: &str) -> Option<PKind> {
        self.base_control_by_code
            .get(code)
            .map(|&i| self.base.controls[i].kind)
    }
}

/// One leg of an imported course in the base file: the bends and tape the import gave it, and the base
/// file's legs and the map exchange stops between them.
struct Segment<'a> {
    bends: &'a [MapPoint],
    taped: bool,
    legs: &'a [Leg<RawPoint>],
    inner: &'a [Stop],
}

/// The legs of an imported course, by pair of stops, as the base course `b` has them; empty when the
/// two do not line up.
fn segments<'a>(
    sides: &Sides<'a>,
    imported: &'a Course,
    b: &'a PCourse<RawPoint>,
) -> Vec<(String, String, Segment<'a>)> {
    let CourseKind::Individual { controls: ccs } = &imported.kind else {
        return Vec::new();
    };
    let at: Vec<usize> = (0..b.stops.len())
        .filter(|&i| sides.base_kind(&b.stops[i].control) != Some(PKind::MapExchange))
        .collect();
    let codes: Vec<&str> = std::iter::once(imported.start.as_str())
        .chain(ccs.iter().map(|c| c.code.0.as_str()))
        .chain(std::iter::once(imported.finish.as_str()))
        .collect();
    let lines_up = at.len() == codes.len()
        && at
            .iter()
            .zip(&codes)
            .all(|(&i, c)| b.stops[i].control == *c)
        && b.legs.len() + 1 >= b.stops.len();
    if !lines_up {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(codes.len().saturating_sub(1));
    for k in 0..codes.len() - 1 {
        let (bends, taped) = match k {
            0 => (imported.start_bends.as_slice(), imported.start_taped),
            _ => (ccs[k - 1].bends.as_slice(), ccs[k - 1].taped),
        };
        let (from, to) = (at[k], at[k + 1]);
        out.push((
            codes[k].to_owned(),
            codes[k + 1].to_owned(),
            Segment {
                bends,
                taped,
                legs: &b.legs[from..to],
                inner: &b.stops[from + 1..to],
            },
        ));
    }
    out
}

/// The number placement each control of a course shows, with its sequence number; `None` for a
/// crossing point (which has no number) and for a control `controls` does not know.
fn shown_numbers(
    course: &Course,
    controls: &HashMap<&str, &Control>,
) -> Vec<Option<(Option<NumberPlacement>, u32)>> {
    let CourseKind::Individual { controls: ccs } = &course.kind else {
        return Vec::new();
    };
    let mut n = 0;
    ccs.iter()
        .map(|cc| match controls.get(cc.code.0.as_str()) {
            Some(c) if c.kind == ControlKind::Control => {
                n += 1;
                Some((cc.number.clone().or(c.number.clone()), n))
            }
            _ => None,
        })
        .collect()
}

/// The project course equals the imported one in everything the file holds: stops, bends, tape, climb,
/// print scale and the number each stop shows.
fn same_course(
    p: &Course,
    i: &Course,
    p_controls: &HashMap<&str, &Control>,
    i_controls: &HashMap<&str, &Control>,
) -> bool {
    let (CourseKind::Individual { controls: a }, CourseKind::Individual { controls: b }) =
        (&p.kind, &i.kind)
    else {
        return false;
    };
    p.name == i.name
        && p.start == i.start
        && p.finish == i.finish
        && p.start_bends == i.start_bends
        && p.start_taped == i.start_taped
        && p.climb_m == i.climb_m
        && p.planned_print_scale == i.planned_print_scale
        && a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.code == y.code && x.bends == y.bends && x.taped == y.taped)
        && shown_numbers(p, p_controls) == shown_numbers(i, i_controls)
}

/// The geometry and course list of two special objects of the same kind are equal; ids aside.
fn same_special(a: &SpecialObject, b: &SpecialObject) -> bool {
    use SpecialObject as S;
    let shape = match (a, b) {
        (S::ForbiddenRoute { points: p, .. }, S::ForbiddenRoute { points: q, .. })
        | (S::MarkedRoute { points: p, .. }, S::MarkedRoute { points: q, .. })
        | (S::OutOfBoundsArea { polygon: p, .. }, S::OutOfBoundsArea { polygon: q, .. })
        | (S::DangerousArea { polygon: p, .. }, S::DangerousArea { polygon: q, .. }) => p == q,
        (S::FirstAid { map_mm: p, .. }, S::FirstAid { map_mm: q, .. })
        | (S::Water { map_mm: p, .. }, S::Water { map_mm: q, .. })
        | (S::Start { map_mm: p, .. }, S::Start { map_mm: q, .. })
        | (S::Finish { map_mm: p, .. }, S::Finish { map_mm: q, .. }) => p == q,
        _ => false,
    };
    shape && a.courses() == b.courses()
}

/// Puts the base file's values into `mapped`, the project as `to_raw` mapped it in the base file's
/// standard and item scaling. See the module for the rules.
///
/// # Errors
/// [`PpenExportError::CircleGapLimit`] when the stops would copy more circle gaps than
/// `MAX_TOTAL_POINTS`.
pub(super) fn overlay(
    mapped: ToRaw,
    project: &Event,
    base: &RawEvent,
    import: &PpenImport,
    frame: &Frame,
) -> Result<Overlaid, PpenExportError> {
    let sides = Sides::new(base, import, frame.linked);
    let (mut origins, course_names, special_ids) = origins(project, &mapped);
    let mut event = mapped.event;

    // Controls, starts and finishes.
    let starts_finishes: Vec<&SpecialObject> = project
        .specials
        .iter()
        .filter(|s| is_start_or_finish(s))
        .collect();
    for (k, out) in event.controls.iter_mut().enumerate() {
        if let Some(s) = starts_finishes.get(k) {
            let Some(i) = sides.import_special(s) else {
                continue;
            };
            let imported = &import.specials[i];
            let Some(b) = sides.base_control(imported.ppen_id(), imported.id()) else {
                continue;
            };
            if let (Some(p_at), Some(i_at)) = (start_at(s), start_at(imported)) {
                *out = start_or_finish(out, b, s.id() == imported.id(), p_at, i_at);
            }
            continue;
        }
        let c = &project.controls[k - starts_finishes.len()];
        let Some(imported) = sides.import_control(c) else {
            continue;
        };
        let Some(b) = sides.base_control(imported.ppen_id, &imported.code.0) else {
            continue;
        };
        *out = control(out, b, c, imported);
    }

    // Courses.
    let p_controls: HashMap<&str, &Control> = project
        .controls
        .iter()
        .map(|c| (c.code.0.as_str(), c))
        .collect();
    let i_controls: HashMap<&str, &Control> = import
        .controls
        .iter()
        .map(|c| (c.code.0.as_str(), c))
        .collect();
    let mut circle_gaps: HashMap<(&str, u32), &[(f64, f64)]> = HashMap::new();
    for c in &base.courses {
        let scale = gap_key(base, c.print_scale_hint);
        for s in &c.stops {
            circle_gaps
                .entry((s.control.as_str(), scale))
                .or_insert(&s.circle_gaps);
        }
    }
    // The legs of every imported course without a map exchange between their stops: Purple Pen keeps
    // one leg per pair of controls.
    let mut direct: HashMap<(String, String), Segment<'_>> = HashMap::new();
    for imported in &import.courses {
        let Some(bi) = sides.base_course(imported) else {
            continue;
        };
        for (from, to, seg) in segments(&sides, imported, &base.courses[bi]) {
            if seg.inner.is_empty() {
                direct.entry((from, to)).or_insert(seg);
            }
        }
    }
    let by_name: HashMap<&str, &Course> = project
        .courses
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();
    for (ci, out) in event.courses.iter_mut().enumerate() {
        let p = by_name[course_names[ci].as_str()];
        let Some(imported) = sides.import_course(p) else {
            continue;
        };
        let Some(bi) = sides.base_course(imported) else {
            continue;
        };
        let b = &base.courses[bi];
        if same_course(p, imported, &p_controls, &i_controls) {
            *out = b.clone();
            continue;
        }
        // A course can visit a pair of controls more than once, through a map exchange on one visit
        // and straight on another: every visit is kept, and each project leg picks the one it equals.
        let mut own: HashMap<(String, String), Vec<Segment<'_>>> = HashMap::new();
        for (f, t, s) in segments(&sides, imported, b) {
            own.entry((f, t)).or_default().push(s);
        }
        let course = Changed {
            p,
            imported,
            b,
            p_controls: &p_controls,
            i_controls: &i_controls,
            own: &own,
            direct: &direct,
            frame,
        };
        *out = course.write(out);
    }
    // Every stop at a control gets its gaps, so the copies are counted against the reader's budget
    // before they are made: the reader charged the base file's stops, not the project's visits.
    let mut copied = 0usize;
    for out in &mut event.courses {
        let scale = gap_key(base, out.print_scale_hint);
        for s in &mut out.stops {
            if let Some(g) = circle_gaps.get(&(s.control.as_str(), scale)) {
                copied = copied.saturating_add(g.len());
                if copied > MAX_TOTAL_POINTS {
                    return Err(PpenExportError::CircleGapLimit {
                        limit: MAX_TOTAL_POINTS,
                    });
                }
                s.circle_gaps = g.to_vec();
            }
        }
    }

    // Special objects. The reader gives an area of a .ppen one ring, which the import makes one
    // project object.
    let project_special: HashMap<&str, usize> = project
        .specials
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id(), i))
        .collect();
    for (r, out) in event.specials.iter_mut().enumerate() {
        let s = &project.specials[project_special[special_ids[r].as_str()]];
        let Some(i) = sides.import_special(s) else {
            continue;
        };
        let imported = &import.specials[i];
        let Some(&bi) = imported
            .ppen_id()
            .and_then(|id| sides.base_special_by_id.get(&id))
        else {
            continue;
        };
        let b = &base.specials[bi];
        let one_ring = match b {
            Special::OutOfBounds { rings, .. } | Special::Dangerous { rings, .. } => {
                rings.len() == 1
            }
            _ => true,
        };
        if one_ring && same_special(s, imported) {
            *out = b.clone();
        }
    }

    // Map exchange controls (and any other base control) the courses visit but the project lacks.
    let mut present: HashSet<String> = event.controls.iter().map(|c| c.code.clone()).collect();
    let mut missing = Vec::new();
    for c in &event.courses {
        for s in &c.stops {
            if !present.contains(&s.control)
                && let Some(&i) = sides.base_control_by_code.get(s.control.as_str())
            {
                present.insert(s.control.clone());
                missing.push(base.controls[i].clone());
            }
        }
    }
    origins.extend(missing.iter().map(|_| Origin::Base));
    event.controls.extend(missing);

    Ok(Overlaid {
        event,
        controls: origins,
        courses: course_names,
        specials: special_ids,
    })
}

/// A start or finish: the base file's when unchanged, else the base control with the project's code and
/// position (it keeps its description).
fn start_or_finish(
    out: &PControl<RawPoint>,
    b: &PControl<RawPoint>,
    same_code: bool,
    p_at: &MapPoint,
    i_at: &MapPoint,
) -> PControl<RawPoint> {
    let mut c = b.clone();
    if !same_code {
        c.code = out.code.clone();
    }
    c.position = position(out.position, b.position, p_at, i_at);
    c
}

fn start_at(s: &SpecialObject) -> Option<&MapPoint> {
    match s {
        SpecialObject::Start { map_mm, .. } | SpecialObject::Finish { map_mm, .. } => Some(map_mm),
        _ => None,
    }
}

/// The position to write: per coordinate the base file's where the project's equals the import's, so
/// that a control moved along one axis keeps the other coordinate's bytes.
fn position(
    out: Option<RawPoint>,
    base: Option<RawPoint>,
    p: &MapPoint,
    imported: &MapPoint,
) -> Option<RawPoint> {
    match (out, base) {
        (Some(o), Some(b)) => Some(RawPoint {
            x_mm: if p.x == imported.x { b.x_mm } else { o.x_mm },
            y_mm: if p.y == imported.y { b.y_mm } else { o.y_mm },
        }),
        (out, _) => out,
    }
}

/// A control: the base file's when unchanged, else the base control with what changed.
fn control(
    out: &PControl<RawPoint>,
    b: &PControl<RawPoint>,
    p: &Control,
    imported: &Control,
) -> PControl<RawPoint> {
    let mut c = b.clone();
    if p.code != imported.code {
        c.code = out.code.clone();
    }
    c.position = position(out.position, b.position, &p.map_mm, &imported.map_mm);
    if p.kind != imported.kind {
        c.kind = out.kind;
    }
    if p.description != imported.description || p.kind != imported.kind {
        let text = b
            .description
            .as_ref()
            .and_then(|d| d.column_f_text.clone())
            .filter(|_| out.kind == PKind::Control);
        c.description = match (out.description.clone(), text) {
            (Some(mut d), text) => {
                d.column_f_text = text;
                Some(d)
            }
            (None, Some(text)) => Some(Description {
                columns: Default::default(),
                standard: b
                    .description
                    .as_ref()
                    .map(|d| d.standard.clone())
                    .unwrap_or_default(),
                column_f_text: Some(text),
            }),
            (None, None) => None,
        };
    }
    c
}

/// A course that differs from the import, with the base file's values for what did not change.
struct Changed<'a, 'b> {
    p: &'a Course,
    imported: &'a Course,
    b: &'a PCourse<RawPoint>,
    p_controls: &'b HashMap<&'a str, &'a Control>,
    i_controls: &'b HashMap<&'a str, &'a Control>,
    /// The legs of the imported course by pair of stops, every visit in course order.
    own: &'b HashMap<(String, String), Vec<Segment<'a>>>,
    direct: &'b HashMap<(String, String), Segment<'a>>,
    frame: &'b Frame,
}

impl Changed<'_, '_> {
    fn write(&self, mapped: &PCourse<RawPoint>) -> PCourse<RawPoint> {
        let (p, imported, b) = (self.p, self.imported, self.b);
        // The print scale as imported: the base file's, with its ratio.
        let base_print = PrintScale {
            hint: b.print_scale_hint,
            ratio: b.course_obj_ratio.unwrap_or(1.0),
        };
        let same_print = p.planned_print_scale == imported.planned_print_scale;
        let mapped = if same_print {
            raw_course(self.p_controls, p, self.frame.kind, base_print, mapped.id)
        } else {
            mapped.clone()
        };

        let CourseKind::Individual { controls: ccs } = &p.kind else {
            return mapped;
        };
        let codes: Vec<&str> = mapped.stops.iter().map(|s| s.control.as_str()).collect();
        let p_numbers = shown_numbers(p, self.p_controls);
        let i_numbers = shown_numbers(imported, self.i_controls);
        let i_codes: Vec<&str> = match &imported.kind {
            CourseKind::Individual { controls } => {
                controls.iter().map(|c| c.code.0.as_str()).collect()
            }
            _ => Vec::new(),
        };

        // Stops: per visit of a control, the base stop of the same visit.
        let mut seen: HashMap<&str, usize> = HashMap::new();
        let mut stops = Vec::with_capacity(mapped.stops.len());
        for (j, stop) in mapped.stops.iter().enumerate() {
            let visit = seen.entry(stop.control.as_str()).or_insert(0);
            *visit += 1;
            let mut s = stop.clone();
            let base_stop = b
                .stops
                .iter()
                .filter(|x| x.control == stop.control)
                .nth(*visit - 1);
            if let Some(bs) = base_stop {
                s.kind = bs.kind;
                s.map_flip = bs.map_flip;
                s.number = bs.number;
                // A control of the course (not the start or finish): keep the base offset when the
                // number shows as imported, at the same sequence number and print scale.
                if same_print && j > 0 && j <= ccs.len() {
                    let mine = p_numbers.get(j - 1).cloned().flatten();
                    let theirs = i_codes
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| **c == stop.control)
                        .nth(*visit - 1)
                        .and_then(|(i, _)| i_numbers.get(i).cloned().flatten());
                    if mine.is_some() && mine == theirs {
                        s.number_text_at = bs.number_text_at;
                    }
                }
            }
            stops.push(s);
        }

        // Legs: the base file's where bends and tape are as imported.
        let mut out_stops = Vec::with_capacity(stops.len());
        let mut legs = Vec::with_capacity(mapped.legs.len());
        let mut stops = stops.into_iter();
        if let Some(first) = stops.next() {
            out_stops.push(first);
        }
        for (k, stop) in stops.enumerate() {
            let (bends, taped) = match k {
                0 => (p.start_bends.as_slice(), p.start_taped),
                _ => (ccs[k - 1].bends.as_slice(), ccs[k - 1].taped),
            };
            let key = (codes[k].to_owned(), codes[k + 1].to_owned());
            let mapped_leg = &mapped.legs[k];
            // The course's own visits of the pair, else the leg another course draws between them.
            let candidates: Vec<&Segment<'_>> = match self.own.get(&key) {
                Some(visits) if !visits.is_empty() => visits.iter().collect(),
                _ => self.direct.get(&key).into_iter().collect(),
            };
            let exact = candidates
                .iter()
                .find(|seg| seg.bends == bends && seg.taped == taped);
            let retaped = candidates
                .iter()
                .find(|seg| seg.bends == bends && seg.inner.is_empty());
            match (exact, retaped) {
                (Some(seg), _) => {
                    for (i, leg) in seg.legs.iter().enumerate() {
                        legs.push(leg.clone());
                        if let Some(inner) = seg.inner.get(i) {
                            out_stops.push(inner.clone());
                        }
                    }
                }
                (None, Some(seg)) => {
                    let mut leg = seg.legs[0].clone();
                    leg.marked_route = mapped_leg.marked_route;
                    legs.push(leg);
                }
                (None, None) => legs.push(mapped_leg.clone()),
            }
            out_stops.push(stop);
        }

        PCourse {
            id: b.id,
            name: mapped.name.clone(),
            family: b.family.clone(),
            kind: b.kind,
            legs,
            stops: out_stops,
            climb_m: if p.climb_m == imported.climb_m {
                b.climb_m
            } else {
                mapped.climb_m
            },
            length_m: b.length_m,
            print_scale_hint: mapped.print_scale_hint,
            course_obj_ratio: mapped.course_obj_ratio,
            secondary_title: b.secondary_title.clone(),
        }
    }
}
