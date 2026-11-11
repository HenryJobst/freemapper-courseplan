//! Conflict keys, their display values, and the sides picked for them (D9, D15).

use std::collections::HashMap;
use std::str::FromStr;

use freemapper_courseplan_model::{ControlKind, Side, SyncDecision};

use super::base::fnv1a64;
use super::fields::{ControlField, CourseField, FieldKey, FieldOutcome, FieldValue, ObjectKind};
use super::matching::{Matching, Triple};
use super::merge3::Pick;
use super::view::Views;

/// One conflict as shown to the user and stored by key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncConflict {
    pub key: String,
    pub base: String,
    pub ours: String,
    pub theirs: String,
    pub fingerprint: String,
}

/// Which side to take: one for every open conflict, or for the conflict with the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Take {
    All(Side),
    One(String, Side),
}

/// A `--take` value that is neither a side nor `KEY=side`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("expected `ours`, `theirs`, `KEY=ours` or `KEY=theirs`, got `{0}`")]
pub struct TakeParseError(pub String);

fn side(s: &str) -> Option<Side> {
    match s {
        "ours" => Some(Side::Ours),
        "theirs" => Some(Side::Theirs),
        _ => None,
    }
}

impl FromStr for Take {
    type Err = TakeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || TakeParseError(s.to_owned());
        match s.split_once('=') {
            None => side(s).map(Take::All).ok_or_else(err),
            Some((key, value)) if !key.is_empty() => side(value)
                .map(|side| Take::One(key.to_owned(), side))
                .ok_or_else(err),
            Some(_) => Err(err()),
        }
    }
}

/// A take that contradicts another: two different bare sides (`None`), or two different sides for one key
/// (`Some(key)`). The first such case, if any.
pub fn contradicting_take(takes: &[Take]) -> Option<Option<String>> {
    for (i, a) in takes.iter().enumerate() {
        for b in &takes[i + 1..] {
            match (a, b) {
                (Take::All(x), Take::All(y)) if x != y => return Some(None),
                (Take::One(k, x), Take::One(l, y)) if k == l && x != y => {
                    return Some(Some(k.clone()));
                }
                _ => {}
            }
        }
    }
    None
}

/// What the stored decisions and the takes make of the open conflicts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Resolution {
    /// The side picked per conflict key.
    pub sides: HashMap<String, Side>,
    /// Keys whose stored decision no longer fits the conflict's values. Such a decision is ignored.
    pub stale: Vec<String>,
    /// Keys without a side, in conflict order.
    pub open: Vec<String>,
}

/// Percent-encodes `/`, `%`, `=`, `#` and `-` so a name fits into one key segment: `#` marks the visit of a
/// number key and `-` joins the two ends of a leg key.
pub(crate) fn escape(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            '/' => out.push_str("%2F"),
            '%' => out.push_str("%25"),
            '=' => out.push_str("%3D"),
            '#' => out.push_str("%23"),
            '-' => out.push_str("%2D"),
            c => out.push(c),
        }
    }
    out
}

/// A name of an object: the project's, else the base's, else theirs (M5). The project's name is the one the
/// user reads in the project file: a start is `S1` there, `STA1` in Purple Pen.
fn pick_name<T>(
    t: &Triple,
    v: &Views,
    get: impl Fn(&crate::ppen_import::PpenImport) -> &[T],
    name: impl Fn(&T) -> String,
    ours_names: &[Option<String>],
) -> String {
    let from = |imp: &crate::ppen_import::PpenImport, i: Option<usize>| {
        i.and_then(|i| get(imp).get(i)).map(&name)
    };
    t.ours
        .and_then(|i| ours_names.get(i).cloned().flatten())
        .or_else(|| from(&v.ours, t.ours))
        .or_else(|| from(&v.base, t.base))
        .or_else(|| from(&v.theirs, t.theirs))
        .unwrap_or_default()
}

fn control_name(id: usize, v: &Views, m: &Matching) -> String {
    m.controls
        .get(id)
        .map(|t| {
            pick_name(
                t,
                v,
                |i| &i.controls,
                |c| c.code.0.clone(),
                &v.ours_names.controls,
            )
        })
        .unwrap_or_default()
}

fn special_name(index: usize, v: &Views, m: &Matching) -> String {
    m.specials
        .get(index)
        .map(|t| {
            pick_name(
                t,
                v,
                |i| &i.specials,
                |s| s.id().to_owned(),
                &v.ours_names.specials,
            )
        })
        .unwrap_or_default()
}

fn course_name(id: usize, v: &Views, m: &Matching) -> String {
    m.courses
        .get(id)
        .map(|t| {
            pick_name(
                t,
                v,
                |i| &i.courses,
                |c| c.name.clone(),
                &v.ours_names.courses,
            )
        })
        .unwrap_or_default()
}

/// The name of a stop identity: a control, or a special object after the controls.
fn stop_name(id: usize, v: &Views, m: &Matching) -> String {
    if id < m.controls.len() {
        control_name(id, v, m)
    } else {
        special_name(id - m.controls.len(), v, m)
    }
}

/// The stable key of the conflict an outcome stands for.
pub fn conflict_key(o: &FieldOutcome, v: &Views, m: &Matching) -> String {
    let control = |id: usize| escape(&control_name(id, v, m));
    let course = |id: usize| escape(&course_name(id, v, m));
    match o.key {
        FieldKey::Control(id, field) => {
            let name = control(id);
            match field {
                ControlField::Code => format!("control/{name}/code"),
                ControlField::Kind => format!("control/{name}/kind"),
                ControlField::Position => format!("control/{name}/position"),
                ControlField::Column(c) => format!("control/{name}/column/{c}"),
            }
        }
        // The identity of a special object's own field is its index among the special objects.
        FieldKey::Special(id) => format!("special/{}", escape(&special_name(id, v, m))),
        FieldKey::Course(id, field) => {
            let name = course(id);
            match field {
                CourseField::Name => format!("course/{name}/name"),
                CourseField::Stops => format!("course/{name}/stops"),
                CourseField::Climb => format!("course/{name}/climb"),
                CourseField::PrintScale => format!("course/{name}/print_scale"),
                CourseField::Number { control: c, visit } if visit > 1 => {
                    format!("course/{name}/number/{}#{visit}", control(c))
                }
                CourseField::Number { control: c, .. } => {
                    format!("course/{name}/number/{}", control(c))
                }
            }
        }
        FieldKey::Leg(a, b) => format!(
            "leg/{}-{}",
            escape(&stop_name(a, v, m)),
            escape(&stop_name(b, v, m))
        ),
        FieldKey::Object(ObjectKind::Control, id) => format!("control/{}", control(id)),
        FieldKey::Object(ObjectKind::Special, id) => {
            format!("special/{}", escape(&special_name(id, v, m)))
        }
        FieldKey::Object(ObjectKind::Course, id) => format!("course/{}", course(id)),
    }
}

/// Fingerprint of a conflict: FNV-1a 64 of the rendered ours and theirs values.
pub fn fingerprint(ours: &str, theirs: &str) -> String {
    let mut bytes = ours.as_bytes().to_vec();
    bytes.push(0);
    bytes.extend_from_slice(theirs.as_bytes());
    fnv1a64(&bytes)
}

fn mm(um: i64) -> String {
    format!("{:.3}", um as f64 / 1000.0)
}

fn pts(points: &[(i64, i64)]) -> String {
    points
        .iter()
        .map(|&(x, y)| format!("{}, {}", mm(x), mm(y)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn scale(s: Option<u32>) -> String {
    s.map_or_else(|| "none".to_owned(), |s| s.to_string())
}

/// The kind of a control as a word. `as_str` is the English text and the key a front end translates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindWord {
    Control,
    CrossingPoint,
    Start,
    Finish,
}

impl KindWord {
    pub fn of(kind: ControlKind) -> Self {
        match kind {
            ControlKind::Control => KindWord::Control,
            ControlKind::CrossingPoint => KindWord::CrossingPoint,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            KindWord::Control => "control",
            KindWord::CrossingPoint => "crossing point",
            KindWord::Start => "start",
            KindWord::Finish => "finish",
        }
    }
}

fn render_value(value: &FieldValue, v: &Views, m: &Matching) -> String {
    match value {
        // A name that reads like one of the words a front end translates is quoted, so it stays a name.
        FieldValue::Code(s) | FieldValue::Name(s) => {
            if crate::text::is_sync_word(s) {
                format!("\"{s}\"")
            } else {
                s.clone()
            }
        }
        FieldValue::Kind(k) => KindWord::of(*k).as_str().to_owned(),
        FieldValue::Point(x, y) => format!("{}, {} mm", mm(*x), mm(*y)),
        FieldValue::Column(c) => c.clone().unwrap_or_else(|| "empty".to_owned()),
        FieldValue::Shape(points, courses) => {
            let mut s = format!("{} mm", pts(points));
            if !courses.is_empty() {
                let names: Vec<_> = courses.iter().map(|&c| course_name(c, v, m)).collect();
                s.push_str(&format!(" on {}", names.join(" ")));
            }
            s
        }
        FieldValue::Stops(ids) => ids
            .iter()
            .map(|&id| stop_name(id, v, m))
            .collect::<Vec<_>>()
            .join(" "),
        FieldValue::Leg(bends, taped) => {
            let mut s = if bends.is_empty() {
                "straight".to_owned()
            } else {
                format!("bends {} mm", pts(bends))
            };
            if *taped {
                s.push_str(", taped");
            }
            s
        }
        FieldValue::Number(None) => "none".to_owned(),
        FieldValue::Number(Some((direction, gap))) => format!(
            "{:.2} deg, {:.2} mm",
            *direction as f64 / 100.0,
            *gap as f64 / 100.0
        ),
        FieldValue::Scale(s) => scale(*s),
        FieldValue::Object(parts) => parts
            .iter()
            .map(|p| render_value(p, v, m))
            .collect::<Vec<_>>()
            .join("; "),
    }
}

/// A value for display: `"-50.100, -3.600 mm"`, `"S1 31 32 F1"`, `"deleted"` for an absent one.
pub fn render(value: Option<&FieldValue>, views: &Views, m: &Matching) -> String {
    value.map_or_else(
        || "deleted".to_owned(),
        |value| render_value(value, views, m),
    )
}

/// The outcomes that are conflicts, with both values rendered.
pub fn conflicts(outcomes: &[FieldOutcome], v: &Views, m: &Matching) -> Vec<SyncConflict> {
    outcomes
        .iter()
        .filter(|o| o.pick == Pick::Conflict)
        .map(|o| {
            let ours = render(o.ours.as_ref(), v, m);
            let theirs = render(o.theirs.as_ref(), v, m);
            SyncConflict {
                key: conflict_key(o, v, m),
                // A value the base did not have is no deletion: there was none.
                base: o
                    .base
                    .as_ref()
                    .map_or_else(|| "none".to_owned(), |b| render(Some(b), v, m)),
                fingerprint: fingerprint(&ours, &theirs),
                ours,
                theirs,
            }
        })
        .collect()
}

/// Picks a side per conflict. An explicit `KEY=side` beats a stored decision, which beats a bare side.
/// A stored decision counts only while its fingerprint fits; one for a key that is no conflict any
/// more is dropped without a word, because that conflict is gone.
pub fn resolve(conflicts: &[SyncConflict], stored: &[SyncDecision], take: &[Take]) -> Resolution {
    let mut out = Resolution::default();
    for c in conflicts {
        let keyed = take.iter().rev().find_map(|t| match t {
            Take::One(key, side) if *key == c.key => Some(*side),
            _ => None,
        });
        let decision = stored.iter().rev().find(|d| d.key == c.key);
        let valid = decision.filter(|d| d.fingerprint == c.fingerprint);
        if decision.is_some() && valid.is_none() {
            out.stale.push(c.key.clone());
        }
        let bare = take.iter().rev().find_map(|t| match t {
            Take::All(side) => Some(*side),
            _ => None,
        });
        match keyed.or(valid.map(|d| d.take)).or(bare) {
            Some(side) => {
                out.sides.insert(c.key.clone(), side);
            }
            None => out.open.push(c.key.clone()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_words_are_readable() {
        assert_eq!(KindWord::of(ControlKind::Control).as_str(), "control");
        assert_eq!(
            KindWord::of(ControlKind::CrossingPoint).as_str(),
            "crossing point"
        );
        assert_eq!(KindWord::Start.as_str(), "start");
        assert_eq!(KindWord::Finish.as_str(), "finish");
    }

    fn conflict(key: &str) -> SyncConflict {
        SyncConflict {
            key: key.to_owned(),
            base: "b".to_owned(),
            ours: "o".to_owned(),
            theirs: "t".to_owned(),
            fingerprint: fingerprint("o", "t"),
        }
    }

    fn decision(key: &str, take: Side, fp: &str) -> SyncDecision {
        SyncDecision {
            key: key.to_owned(),
            take,
            fingerprint: fp.to_owned(),
        }
    }

    #[test]
    fn bare_take_applies_to_all_open() {
        let cs = [conflict("a"), conflict("b")];
        let stored = [decision("a", Side::Ours, &cs[0].fingerprint)];
        let r = resolve(&cs, &stored, &[Take::All(Side::Theirs)]);
        assert_eq!(r.sides["a"], Side::Ours, "stored beats a bare take");
        assert_eq!(r.sides["b"], Side::Theirs);
        assert!(r.open.is_empty() && r.stale.is_empty());
    }

    #[test]
    fn key_take_beats_stored_decision() {
        let cs = [conflict("a"), conflict("b")];
        let stored = [decision("a", Side::Ours, &cs[0].fingerprint)];
        let r = resolve(&cs, &stored, &[Take::One("a".to_owned(), Side::Theirs)]);
        assert_eq!(r.sides["a"], Side::Theirs);
        assert_eq!(r.open, vec!["b".to_owned()]);
    }

    #[test]
    fn stale_fingerprint_is_ignored() {
        let cs = [conflict("a")];
        let stored = [
            decision("a", Side::Ours, "0000000000000000"),
            decision("gone", Side::Ours, "0000000000000000"),
        ];
        let r = resolve(&cs, &stored, &[]);
        assert_eq!(r.stale, vec!["a".to_owned()]);
        assert_eq!(r.open, vec!["a".to_owned()]);
        assert!(r.sides.is_empty());
    }

    #[test]
    fn contradicting_takes_are_found() {
        let one = |k: &str, s| Take::One(k.to_owned(), s);
        assert_eq!(contradicting_take(&[]), None);
        assert_eq!(
            contradicting_take(&[Take::All(Side::Ours), Take::All(Side::Ours)]),
            None
        );
        assert_eq!(
            contradicting_take(&[Take::All(Side::Ours), Take::All(Side::Theirs)]),
            Some(None)
        );
        assert_eq!(
            contradicting_take(&[
                one("a", Side::Ours),
                Take::All(Side::Theirs),
                one("a", Side::Theirs)
            ]),
            Some(Some("a".to_owned()))
        );
        assert_eq!(
            contradicting_take(&[one("a", Side::Ours), one("b", Side::Theirs)]),
            None
        );
    }

    #[test]
    fn take_parses() {
        assert_eq!("ours".parse(), Ok(Take::All(Side::Ours)));
        assert_eq!(
            "control/34/position=theirs".parse(),
            Ok(Take::One("control/34/position".to_owned(), Side::Theirs))
        );
        assert!("mine".parse::<Take>().is_err());
        assert!("a=mine".parse::<Take>().is_err());
        assert!("=ours".parse::<Take>().is_err());
    }

    #[test]
    fn escapes_reserved_characters() {
        assert_eq!(escape("A/B"), "A%2FB");
        assert_eq!(escape("50%=x"), "50%25%3Dx");
        // `#` marks the visit of a number key and `-` joins the ends of a leg key.
        assert_eq!(escape("31#2"), "31%232");
        assert_eq!(escape("a-b"), "a%2Db");
    }

    #[test]
    fn keys_escape_slashes() {
        use freemapper_courseplan_model::{Course, CourseKind};

        use crate::ppen_import::PpenImport;
        let course = Course {
            name: "A/B".to_owned(),
            classes: vec![],
            start: "S".to_owned(),
            finish: "F".to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: None,
            ppen_id: None,
            kind: CourseKind::Individual { controls: vec![] },
        };
        let import = PpenImport {
            controls: vec![],
            specials: vec![],
            courses: vec![course],
            notes: vec![],
        };
        let v = Views {
            base: import.clone(),
            ours: import.clone(),
            theirs: import,
            ours_ids: Default::default(),
            ours_names: Default::default(),
            skipped: vec![],
        };
        let m = Matching {
            courses: vec![Triple {
                base: Some(0),
                ours: Some(0),
                theirs: Some(0),
            }],
            ..Default::default()
        };
        let o = FieldOutcome {
            key: FieldKey::Course(0, CourseField::Stops),
            pick: Pick::Conflict,
            base: None,
            ours: None,
            theirs: Some(FieldValue::Stops(vec![])),
            merged: None,
        };
        assert_eq!(conflict_key(&o, &v, &m), "course/A%2FB/stops");
        let cs = conflicts(&[o], &v, &m);
        assert_eq!(cs[0].ours, "deleted");
    }

    #[test]
    fn names_that_look_like_words_are_quoted_and_a_missing_base_is_none() {
        let empty = crate::ppen_import::PpenImport {
            controls: vec![],
            specials: vec![],
            courses: vec![],
            notes: vec![],
        };
        let v = Views {
            base: empty.clone(),
            ours: empty.clone(),
            theirs: empty,
            ours_ids: Default::default(),
            ours_names: Default::default(),
            skipped: vec![],
        };
        let m = Matching::default();
        let name = FieldValue::Name("none".to_owned());
        assert_eq!(render(Some(&name), &v, &m), "\"none\"");
        assert_eq!(render(Some(&FieldValue::Name("A".to_owned())), &v, &m), "A");
        assert_eq!(render(None, &v, &m), "deleted");
        let o = FieldOutcome {
            key: FieldKey::Course(0, CourseField::Climb),
            pick: Pick::Conflict,
            base: None,
            ours: Some(FieldValue::Scale(Some(1))),
            theirs: Some(FieldValue::Scale(Some(2))),
            merged: None,
        };
        let m = Matching {
            courses: vec![Triple {
                base: None,
                ours: None,
                theirs: None,
            }],
            ..Default::default()
        };
        assert_eq!(conflicts(&[o], &v, &m)[0].base, "none");
    }

    #[test]
    fn fingerprint_depends_on_both_values() {
        assert_ne!(fingerprint("a", "b"), fingerprint("b", "a"));
        assert_ne!(fingerprint("ab", ""), fingerprint("a", "b"));
        assert_eq!(fingerprint("a", "b").len(), 16);
    }
}
