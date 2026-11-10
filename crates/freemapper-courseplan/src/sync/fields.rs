//! Fields of the matched objects and their classification by the three-way rule (D3 to D6, D10, D11).
//!
//! Every object of a view is cut into fields (D4). Each field has a value per side, `None` when the
//! object or the field does not exist there. [`classify`] runs the pick rule on every field. When an
//! object exists on one side only, the whole object is one outcome (`FieldKey::Object`), so a deletion
//! against an edit is one conflict on the object and never a pile of field conflicts.
//!
//! Identities (`usize`) are indices into the [`Matching`]: a control is its index in `controls`; a special
//! object, a start or a finish among them, is `controls.len()` plus its index in `specials`; a course is
//! its index in `courses`.
//!
//! Data only courseplan holds (classes, starter counts, accepted findings, `print_area`, team, relay and
//! score courses) never enters a field, so the merge cannot touch it (D10).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use freemapper_courseplan_model::{
    Control, ControlKind, Course, CourseKind, Event, MapPoint, SpecialObject,
};

use super::matching::{Matching, Triple};
use super::merge3::{Merged3, Pick, diff3, pick};
use super::view::Views;
use crate::ppen_import::PpenImport;

/// What kind of object an identity names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectKind {
    Control,
    Special,
    Course,
}

/// A field of a control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ControlField {
    Code,
    Kind,
    Position,
    /// One description column, `'c'` to `'h'`. Each column is merged on its own.
    Column(char),
}

/// A field of a course.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CourseField {
    Name,
    /// Identity sequence, start and finish included.
    Stops,
    Climb,
    PrintScale,
    /// The shown number placement of the `visit`-th (1-based) stop at `control`.
    Number {
        control: usize,
        visit: u32,
    },
}

/// Names one field of one matched object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FieldKey {
    Control(usize, ControlField),
    /// The shape of a special object, a start or finish included.
    Special(usize),
    Course(usize, CourseField),
    /// The leg between two stop identities, event-wide.
    Leg(usize, usize),
    /// A whole object that exists on one side only, or the existence of one that does not.
    Object(ObjectKind, usize),
}

/// The value of a field on one side. Points are micrometres; a number is hundredths of a degree and of a
/// millimetre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    Code(String),
    Kind(ControlKind),
    Point(i64, i64),
    Column(Option<String>),
    /// Points and the identities of the courses the object is limited to.
    Shape(Vec<(i64, i64)>, Vec<usize>),
    Name(String),
    Stops(Vec<usize>),
    /// Bends and whether the leg is a marked route.
    Leg(Vec<(i64, i64)>, bool),
    Number(Option<(i64, i64)>),
    /// Climb in metres or print scale.
    Scale(Option<u32>),
    /// All field values of a whole object, for outcomes on `FieldKey::Object`.
    Object(Vec<FieldValue>),
}

/// The classification of one field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldOutcome {
    pub key: FieldKey,
    pub pick: Pick,
    pub base: Option<FieldValue>,
    pub ours: Option<FieldValue>,
    pub theirs: Option<FieldValue>,
    /// Set for `Stops` when both sides changed them and diff3 merged them cleanly.
    pub merged: Option<FieldValue>,
}

/// One view with the identity of each of its objects.
struct Side<'a> {
    imp: &'a PpenImport,
    controls: Vec<Option<usize>>,
    specials: Vec<Option<usize>>,
    courses: Vec<Option<usize>>,
    by_code: HashMap<&'a str, usize>,
    special_by_id: HashMap<&'a str, usize>,
    course_by_name: HashMap<&'a str, usize>,
    special_offset: usize,
}

fn index_ids(
    triples: &[Triple],
    len: usize,
    get: fn(&Triple) -> Option<usize>,
) -> Vec<Option<usize>> {
    let mut out = vec![None; len];
    for (identity, t) in triples.iter().enumerate() {
        if let Some(i) = get(t).filter(|&i| i < len) {
            out[i] = Some(identity);
        }
    }
    out
}

impl<'a> Side<'a> {
    fn new(imp: &'a PpenImport, m: &Matching, get: fn(&Triple) -> Option<usize>) -> Self {
        Side {
            imp,
            controls: index_ids(&m.controls, imp.controls.len(), get),
            specials: index_ids(&m.specials, imp.specials.len(), get),
            courses: index_ids(&m.courses, imp.courses.len(), get),
            by_code: imp
                .controls
                .iter()
                .enumerate()
                .map(|(i, c)| (c.code.0.as_str(), i))
                .collect(),
            special_by_id: imp
                .specials
                .iter()
                .enumerate()
                .map(|(i, s)| (s.id(), i))
                .collect(),
            course_by_name: imp
                .courses
                .iter()
                .enumerate()
                .map(|(i, c)| (c.name.as_str(), i))
                .collect(),
            special_offset: m.controls.len(),
        }
    }

    fn special_identity(&self, id: &str) -> Option<usize> {
        let i = *self.special_by_id.get(id)?;
        Some(self.special_offset + self.specials[i]?)
    }
}

fn point(p: &MapPoint) -> (i64, i64) {
    ((p.x * 1000.0).round() as i64, (p.y * 1000.0).round() as i64)
}

fn points(ps: &[MapPoint]) -> Vec<(i64, i64)> {
    ps.iter().map(point).collect()
}

fn special_points(s: &SpecialObject) -> Vec<(i64, i64)> {
    match s {
        SpecialObject::Start { map_mm, .. }
        | SpecialObject::Finish { map_mm, .. }
        | SpecialObject::FirstAid { map_mm, .. }
        | SpecialObject::Water { map_mm, .. } => vec![point(map_mm)],
        SpecialObject::MarkedRoute { points: ps, .. }
        | SpecialObject::ForbiddenRoute { points: ps, .. } => points(ps),
        SpecialObject::OutOfBoundsArea { polygon, .. }
        | SpecialObject::DangerousArea { polygon, .. } => points(polygon),
    }
}

fn control_fields(out: &mut BTreeMap<FieldKey, FieldValue>, identity: usize, c: &Control) {
    let key = |f| FieldKey::Control(identity, f);
    out.insert(key(ControlField::Code), FieldValue::Code(c.code.0.clone()));
    out.insert(key(ControlField::Kind), FieldValue::Kind(c.kind));
    let (x, y) = point(&c.map_mm);
    out.insert(key(ControlField::Position), FieldValue::Point(x, y));
    let d = &c.description;
    for (letter, column) in [
        ('c', &d.c),
        ('d', &d.d),
        ('e', &d.e),
        ('f', &d.f),
        ('g', &d.g),
        ('h', &d.h),
    ] {
        out.insert(
            key(ControlField::Column(letter)),
            FieldValue::Column(column.clone()),
        );
    }
}

fn course_fields(
    out: &mut BTreeMap<FieldKey, FieldValue>,
    side: &Side<'_>,
    identity: usize,
    c: &Course,
) {
    let key = |f| FieldKey::Course(identity, f);
    out.insert(key(CourseField::Name), FieldValue::Name(c.name.clone()));
    out.insert(key(CourseField::Climb), FieldValue::Scale(c.climb_m));
    out.insert(
        key(CourseField::PrintScale),
        FieldValue::Scale(c.planned_print_scale),
    );
    let CourseKind::Individual { controls } = &c.kind else {
        return;
    };
    let start = side.special_identity(&c.start);
    let finish = side.special_identity(&c.finish);
    // The stops in order, each with the leg that leaves it.
    let mut chain: Vec<Option<usize>> = vec![start];
    let mut legs: Vec<(Vec<(i64, i64)>, bool)> = vec![(points(&c.start_bends), c.start_taped)];
    let mut visits: HashMap<usize, u32> = HashMap::new();
    for cc in controls {
        let found = side.by_code.get(cc.code.0.as_str()).copied();
        let id = found.and_then(|i| side.controls[i]);
        chain.push(id);
        legs.push((points(&cc.bends), cc.taped));
        if let (Some(i), Some(id)) = (found, id) {
            let visit = visits.entry(id).or_insert(0);
            *visit += 1;
            let shown = cc.number.as_ref().or(side.imp.controls[i].number.as_ref());
            out.insert(
                key(CourseField::Number {
                    control: id,
                    visit: *visit,
                }),
                FieldValue::Number(shown.map(|n| {
                    (
                        (n.direction * 100.0).round() as i64,
                        (n.gap_mm * 100.0).round() as i64,
                    )
                })),
            );
        }
    }
    chain.push(finish);
    out.insert(
        key(CourseField::Stops),
        FieldValue::Stops(chain.iter().flatten().copied().collect()),
    );
    for (k, (bends, taped)) in legs.into_iter().enumerate() {
        if let (Some(from), Some(to)) = (chain[k], chain[k + 1]) {
            out.entry(FieldKey::Leg(from, to))
                .or_insert(FieldValue::Leg(bends, taped));
        }
    }
}

/// All fields of one view.
fn collect(side: &Side<'_>) -> BTreeMap<FieldKey, FieldValue> {
    let mut out = BTreeMap::new();
    for (i, c) in side.imp.controls.iter().enumerate() {
        if let Some(identity) = side.controls[i] {
            control_fields(&mut out, identity, c);
        }
    }
    for (i, s) in side.imp.specials.iter().enumerate() {
        if let Some(identity) = side.specials[i] {
            let mut courses: Vec<usize> = s
                .courses()
                .iter()
                .filter_map(|n| side.courses[*side.course_by_name.get(n.as_str())?])
                .collect();
            courses.sort_unstable();
            courses.dedup();
            out.insert(
                FieldKey::Special(identity),
                FieldValue::Shape(special_points(s), courses),
            );
        }
    }
    for (i, c) in side.imp.courses.iter().enumerate() {
        if let Some(identity) = side.courses[i] {
            course_fields(&mut out, side, identity, c);
        }
    }
    out
}

/// The object a field belongs to; legs belong to none.
fn owner(key: &FieldKey) -> Option<(ObjectKind, usize)> {
    match *key {
        FieldKey::Control(i, _) => Some((ObjectKind::Control, i)),
        FieldKey::Special(i) => Some((ObjectKind::Special, i)),
        FieldKey::Course(i, _) => Some((ObjectKind::Course, i)),
        FieldKey::Leg(..) | FieldKey::Object(..) => None,
    }
}

type Fields = BTreeMap<FieldKey, FieldValue>;

fn field_outcome(key: FieldKey, base: &Fields, ours: &Fields, theirs: &Fields) -> FieldOutcome {
    let (b, o, t) = (base.get(&key), ours.get(&key), theirs.get(&key));
    let mut p = pick(b, o, t);
    let mut merged = None;
    if let (
        Pick::Conflict,
        FieldKey::Course(_, CourseField::Stops),
        Some(FieldValue::Stops(bs)),
        Some(FieldValue::Stops(os)),
        Some(FieldValue::Stops(ts)),
    ) = (p, key, b, o, t)
        && let Merged3::Clean(list) = diff3(bs, os, ts)
    {
        merged = Some(FieldValue::Stops(list));
        p = Pick::Both;
    }
    FieldOutcome {
        key,
        pick: p,
        base: b.cloned(),
        ours: o.cloned(),
        theirs: t.cloned(),
        merged,
    }
}

/// All field values of an object on one side; `keys` are the object's fields on any side.
fn whole(fields: &Fields, keys: &[FieldKey]) -> FieldValue {
    FieldValue::Object(keys.iter().filter_map(|k| fields.get(k).cloned()).collect())
}

/// Classifies every field of every matched object, then the legs.
///
/// An object that exists on only one of ours and theirs is one outcome on `FieldKey::Object`; if it was
/// deleted on one side and changed on the other, that outcome is a conflict and its fields are dropped.
/// `project` is read for D9's rule only: Purple Pen deleting a course that has classes is a conflict.
pub fn classify(v: &Views, m: &Matching, project: &Event) -> Vec<FieldOutcome> {
    let sides = (
        Side::new(&v.base, m, |t| t.base),
        Side::new(&v.ours, m, |t| t.ours),
        Side::new(&v.theirs, m, |t| t.theirs),
    );
    let (base, ours, theirs) = (collect(&sides.0), collect(&sides.1), collect(&sides.2));
    let mut groups: BTreeMap<(ObjectKind, usize), Vec<FieldKey>> = BTreeMap::new();
    let mut legs = Vec::new();
    let all: BTreeSet<FieldKey> = base
        .keys()
        .chain(ours.keys())
        .chain(theirs.keys())
        .copied()
        .collect();
    for key in all {
        match owner(&key) {
            Some(object) => groups.entry(object).or_default().push(key),
            None => legs.push(key),
        }
    }
    let mut out = Vec::new();
    for (kind, triples) in [
        (ObjectKind::Control, &m.controls),
        (ObjectKind::Special, &m.specials),
        (ObjectKind::Course, &m.courses),
    ] {
        for (identity, t) in triples.iter().enumerate() {
            let object = (kind, identity);
            if t.ours.is_some() && t.theirs.is_some() {
                for &key in groups.get(&object).into_iter().flatten() {
                    out.push(field_outcome(key, &base, &ours, &theirs));
                }
                continue;
            }
            let keys = groups.get(&object).map_or(&[][..], Vec::as_slice);
            let value = |present: bool, fields: &Fields| present.then(|| whole(fields, keys));
            let (b, o, th) = (
                value(t.base.is_some(), &base),
                value(t.ours.is_some(), &ours),
                value(t.theirs.is_some(), &theirs),
            );
            let mut p = pick(b.as_ref(), o.as_ref(), th.as_ref());
            if kind == ObjectKind::Course
                && p == Pick::Theirs
                && th.is_none()
                && let Some(oi) = t.ours
            {
                let name = &v.ours.courses[oi].name;
                if project
                    .courses
                    .iter()
                    .any(|c| &c.name == name && !c.classes.is_empty())
                {
                    p = Pick::Conflict;
                }
            }
            out.push(FieldOutcome {
                key: FieldKey::Object(kind, identity),
                pick: p,
                base: b,
                ours: o,
                theirs: th,
                merged: None,
            });
        }
    }
    for key in legs {
        let mut o = field_outcome(key, &base, &ours, &theirs);
        // D6: a leg that one side removed through its stops no longer exists; the other side's bends for
        // it are dropped (`apply` notes it), so the removal wins instead of asking.
        if o.pick == Pick::Conflict && o.ours.is_some() != o.theirs.is_some() {
            o.pick = if o.ours.is_none() {
                Pick::Ours
            } else {
                Pick::Theirs
            };
        }
        out.push(o);
    }
    out
}

#[cfg(test)]
mod tests {
    use freemapper_courseplan_model::{
        ControlCode, ControlDescription, CourseControl, EventMeta, NumberPlacement, Sources,
    };

    use super::*;
    use crate::sync::matching::match_sides;

    fn ctl(code: &str, x: f64) -> Control {
        Control {
            code: ControlCode(code.to_owned()),
            kind: ControlKind::Control,
            map_mm: MapPoint { x, y: 0.0 },
            description: ControlDescription::default(),
            number: None,
            ppen_id: code.parse().ok(),
        }
    }

    fn cc(code: &str) -> CourseControl {
        CourseControl {
            code: ControlCode(code.to_owned()),
            number: None,
            bends: vec![],
            taped: false,
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
                controls: codes.iter().map(|c| cc(c)).collect(),
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

    fn import(controls: Vec<Control>, courses: Vec<Course>) -> PpenImport {
        PpenImport {
            controls,
            specials: ends(),
            courses,
            notes: vec![],
        }
    }

    fn views(base: PpenImport, ours: PpenImport, theirs: PpenImport) -> Views {
        Views {
            base,
            ours,
            theirs,
            ours_ids: Default::default(),
            skipped: vec![],
        }
    }

    fn project(controls: Vec<Control>, courses: Vec<Course>) -> Event {
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
            controls,
            specials: ends(),
            courses,
            accepted_findings: vec![],
            sync: None,
        }
    }

    fn run(v: &Views, p: &Event) -> (Matching, Vec<FieldOutcome>) {
        let m = match_sides(v, true);
        let out = classify(v, &m, p);
        (m, out)
    }

    fn empty_project() -> Event {
        project(vec![], vec![])
    }

    /// Identity of the control with `code`, or of the special object `S` or `F`.
    fn ident(v: &Views, m: &Matching, name: &str) -> usize {
        if name == "S" || name == "F" {
            let at = m
                .specials
                .iter()
                .position(|t| {
                    [(&v.base, t.base), (&v.ours, t.ours), (&v.theirs, t.theirs)]
                        .iter()
                        .any(|(imp, i)| i.is_some_and(|i| imp.specials[i].id() == name))
                })
                .unwrap();
            return m.controls.len() + at;
        }
        m.controls
            .iter()
            .position(|t| {
                [(&v.base, t.base), (&v.ours, t.ours), (&v.theirs, t.theirs)]
                    .iter()
                    .any(|(imp, i)| i.is_some_and(|i| imp.controls[i].code.0 == name))
            })
            .unwrap()
    }

    fn course_ident(v: &Views, m: &Matching, name: &str) -> usize {
        m.courses
            .iter()
            .position(|t| {
                [(&v.base, t.base), (&v.ours, t.ours), (&v.theirs, t.theirs)]
                    .iter()
                    .any(|(imp, i)| i.is_some_and(|i| imp.courses[i].name == name))
            })
            .unwrap()
    }

    fn of(out: &[FieldOutcome], key: FieldKey) -> &FieldOutcome {
        out.iter().find(|o| o.key == key).unwrap()
    }

    fn base3() -> Vec<Control> {
        vec![ctl("31", 10.0), ctl("34", 20.0)]
    }

    #[test]
    fn one_side_changes_are_taken() {
        let mut ours = base3();
        ours[0].map_mm.x = 11.0;
        let mut theirs = base3();
        theirs[1].map_mm.x = 21.0;
        let v = views(
            import(base3(), vec![]),
            import(ours, vec![]),
            import(theirs, vec![]),
        );
        let (m, out) = run(&v, &empty_project());
        let pos = |c: &str| FieldKey::Control(ident(&v, &m, c), ControlField::Position);
        assert_eq!(of(&out, pos("34")).pick, Pick::Theirs);
        assert_eq!(of(&out, pos("31")).pick, Pick::Ours);
    }

    #[test]
    fn same_change_both_sides_is_both() {
        let mut moved = base3();
        moved[0].map_mm.x = 12.0;
        let v = views(
            import(base3(), vec![]),
            import(moved.clone(), vec![]),
            import(moved, vec![]),
        );
        let (m, out) = run(&v, &empty_project());
        let key = FieldKey::Control(ident(&v, &m, "31"), ControlField::Position);
        assert_eq!(of(&out, key).pick, Pick::Both);
    }

    #[test]
    fn both_moved_differently_is_a_conflict() {
        let mut ours = base3();
        ours[0].map_mm.x = 11.0;
        let mut theirs = base3();
        theirs[0].map_mm.x = 13.0;
        let v = views(
            import(base3(), vec![]),
            import(ours, vec![]),
            import(theirs, vec![]),
        );
        let (m, out) = run(&v, &empty_project());
        let o = of(
            &out,
            FieldKey::Control(ident(&v, &m, "31"), ControlField::Position),
        );
        assert_eq!(o.pick, Pick::Conflict);
        assert_eq!(o.base, Some(FieldValue::Point(10_000, 0)));
        assert_eq!(o.ours, Some(FieldValue::Point(11_000, 0)));
        assert_eq!(o.theirs, Some(FieldValue::Point(13_000, 0)));
    }

    #[test]
    fn columns_merge_separately() {
        let mut ours = base3();
        ours[0].description.g = Some("5.1".to_owned());
        let mut theirs = base3();
        theirs[0].description.d = Some("1.8".to_owned());
        let v = views(
            import(base3(), vec![]),
            import(ours, vec![]),
            import(theirs, vec![]),
        );
        let (m, out) = run(&v, &empty_project());
        let id = ident(&v, &m, "31");
        let col = |c| FieldKey::Control(id, ControlField::Column(c));
        assert_eq!(of(&out, col('g')).pick, Pick::Ours);
        assert_eq!(of(&out, col('d')).pick, Pick::Theirs);
        assert_eq!(of(&out, col('h')).pick, Pick::Unchanged);
    }

    #[test]
    fn unrepresentable_symbol_survives_ppen_column_edit() {
        // The project holds a 2024-only symbol in column e. Purple Pen cannot hold it, so base and ours
        // lack it in the view, and the symbol is not touched by an edit of column d in the file.
        let mut held = ctl("31", 10.0);
        held.description.e = Some("2024-only".to_owned());
        let mut theirs = base3();
        theirs[0].description.d = Some("1.8".to_owned());
        let v = views(
            import(base3(), vec![]),
            import(base3(), vec![]),
            import(theirs, vec![]),
        );
        let (m, out) = run(&v, &project(vec![held], vec![]));
        let id = ident(&v, &m, "31");
        let col = |c| FieldKey::Control(id, ControlField::Column(c));
        assert_eq!(of(&out, col('e')).pick, Pick::Unchanged);
        assert_eq!(of(&out, col('d')).pick, Pick::Theirs);
    }

    #[test]
    fn shown_number_moves_per_stop() {
        let courses = || vec![course("A", 1, &["31", "34"]), course("B", 2, &["31"])];
        let mut theirs_courses = courses();
        if let CourseKind::Individual { controls } = &mut theirs_courses[0].kind {
            controls[0].number = Some(NumberPlacement {
                direction: 45.0,
                gap_mm: 1.5,
            });
        }
        let v = views(
            import(base3(), courses()),
            import(base3(), courses()),
            import(base3(), theirs_courses),
        );
        let (m, out) = run(&v, &empty_project());
        let c31 = ident(&v, &m, "31");
        let number = |course: &str| {
            FieldKey::Course(
                course_ident(&v, &m, course),
                CourseField::Number {
                    control: c31,
                    visit: 1,
                },
            )
        };
        let a = of(&out, number("A"));
        assert_eq!(a.pick, Pick::Theirs);
        assert_eq!(a.theirs, Some(FieldValue::Number(Some((4500, 150)))));
        assert_eq!(of(&out, number("B")).pick, Pick::Unchanged);
    }

    #[test]
    fn stops_both_changed_without_overlap_merge() {
        let mut base = base3();
        base.push(ctl("32", 30.0));
        let mut ours_controls = base.clone();
        ours_controls.push(ctl("40", 40.0));
        let mut theirs_controls = base.clone();
        theirs_controls.push(ctl("41", 50.0));
        let v = views(
            import(base, vec![course("A", 1, &["31", "34", "32"])]),
            import(
                ours_controls,
                vec![course("A", 1, &["31", "40", "34", "32"])],
            ),
            import(
                theirs_controls,
                vec![course("A", 1, &["31", "34", "32", "41"])],
            ),
        );
        let (m, out) = run(&v, &empty_project());
        let o = of(
            &out,
            FieldKey::Course(course_ident(&v, &m, "A"), CourseField::Stops),
        );
        assert_eq!(o.pick, Pick::Both);
        let expect: Vec<usize> = ["S", "31", "40", "34", "32", "41", "F"]
            .iter()
            .map(|n| ident(&v, &m, n))
            .collect();
        assert_eq!(o.merged, Some(FieldValue::Stops(expect)));
    }

    fn with35() -> Vec<Control> {
        vec![ctl("34", 20.0), ctl("35", 25.0)]
    }

    fn only_object(out: &[FieldOutcome]) -> &FieldOutcome {
        let objects: Vec<_> = out
            .iter()
            .filter(|o| matches!(o.key, FieldKey::Object(ObjectKind::Control, _)))
            .collect();
        assert_eq!(objects.len(), 1);
        objects[0]
    }

    #[test]
    fn deleted_in_ppen_edited_here_is_a_conflict() {
        let mut ours = with35();
        ours[1].map_mm.x = 26.0;
        let v = views(
            import(with35(), vec![]),
            import(ours, vec![]),
            import(vec![ctl("34", 20.0)], vec![]),
        );
        let (m, out) = run(&v, &empty_project());
        let id = ident(&v, &m, "35");
        let o = only_object(&out);
        assert_eq!(o.key, FieldKey::Object(ObjectKind::Control, id));
        assert_eq!(o.pick, Pick::Conflict);
        assert!(o.ours.is_some() && o.theirs.is_none());
        assert!(
            !out.iter()
                .any(|o| matches!(o.key, FieldKey::Control(i, _) if i == id))
        );
    }

    #[test]
    fn deleted_here_edited_in_ppen_is_a_conflict() {
        let mut theirs = with35();
        theirs[1].map_mm.x = 26.0;
        let v = views(
            import(with35(), vec![]),
            import(vec![ctl("34", 20.0)], vec![]),
            import(theirs, vec![]),
        );
        let (m, out) = run(&v, &empty_project());
        let id = ident(&v, &m, "35");
        let o = only_object(&out);
        assert_eq!(o.key, FieldKey::Object(ObjectKind::Control, id));
        assert_eq!(o.pick, Pick::Conflict);
        assert!(o.ours.is_none() && o.theirs.is_some());
        assert!(
            !out.iter()
                .any(|o| matches!(o.key, FieldKey::Control(i, _) if i == id))
        );
    }

    #[test]
    fn deleted_both_sides_is_both() {
        let v = views(
            import(with35(), vec![]),
            import(vec![ctl("34", 20.0)], vec![]),
            import(vec![ctl("34", 20.0)], vec![]),
        );
        let (_, out) = run(&v, &empty_project());
        assert_eq!(only_object(&out).pick, Pick::Both);
    }

    #[test]
    fn ppen_deleted_course_with_classes_is_a_conflict() {
        let a = || course("A", 1, &["34"]);
        let v = views(
            import(base3(), vec![a()]),
            import(base3(), vec![a()]),
            import(base3(), vec![]),
        );
        let mut with_classes = a();
        with_classes.classes = vec!["Offen".to_owned()];
        let (m, out) = run(&v, &project(vec![], vec![with_classes]));
        let key = FieldKey::Object(ObjectKind::Course, course_ident(&v, &m, "A"));
        assert_eq!(of(&out, key).pick, Pick::Conflict);
        let (_, out) = run(&v, &project(vec![], vec![a()]));
        assert_eq!(of(&out, key).pick, Pick::Theirs);
    }

    #[test]
    fn leg_is_event_wide() {
        let courses = || vec![course("A", 1, &["31", "34"]), course("B", 2, &["31", "34"])];
        let mut theirs = courses();
        for c in &mut theirs {
            if let CourseKind::Individual { controls } = &mut c.kind {
                controls[0].bends = vec![MapPoint { x: 15.0, y: 3.0 }];
            }
        }
        let v = views(
            import(base3(), courses()),
            import(base3(), courses()),
            import(base3(), theirs),
        );
        let (m, out) = run(&v, &empty_project());
        let (a, b) = (ident(&v, &m, "31"), ident(&v, &m, "34"));
        let legs: Vec<_> = out
            .iter()
            .filter(|o| o.key == FieldKey::Leg(a, b))
            .collect();
        assert_eq!(legs.len(), 1);
        assert_eq!(legs[0].pick, Pick::Theirs);
        assert_eq!(
            legs[0].theirs,
            Some(FieldValue::Leg(vec![(15_000, 3_000)], false))
        );
    }

    /// D6: a leg one side removed through its stops, bent on the other side, is dropped (with a note in
    /// `apply`), not a question.
    #[test]
    fn leg_removed_on_one_side_and_bent_on_the_other_is_not_a_conflict() {
        let mut bent = vec![course("A", 1, &["31", "34"])];
        if let CourseKind::Individual { controls } = &mut bent[0].kind {
            controls[0].bends = vec![MapPoint { x: 15.0, y: 3.0 }];
        }
        let v = views(
            import(base3(), vec![course("A", 1, &["31", "34"])]),
            import(base3(), vec![course("A", 1, &["34"])]),
            import(base3(), bent),
        );
        let (m, out) = run(&v, &empty_project());
        let (a, b) = (ident(&v, &m, "31"), ident(&v, &m, "34"));
        assert_eq!(of(&out, FieldKey::Leg(a, b)).pick, Pick::Ours);
        // And the other way round.
        let mut bent = vec![course("A", 1, &["31", "34"])];
        if let CourseKind::Individual { controls } = &mut bent[0].kind {
            controls[0].bends = vec![MapPoint { x: 15.0, y: 3.0 }];
        }
        let v = views(
            import(base3(), vec![course("A", 1, &["31", "34"])]),
            import(base3(), bent),
            import(base3(), vec![course("A", 1, &["34"])]),
        );
        let (m, out) = run(&v, &empty_project());
        let (a, b) = (ident(&v, &m, "31"), ident(&v, &m, "34"));
        assert_eq!(of(&out, FieldKey::Leg(a, b)).pick, Pick::Theirs);
    }
}
