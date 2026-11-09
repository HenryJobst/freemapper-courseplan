//! Serde domain types of an event: controls, special objects, courses of all kinds, classes.
//!
//! Positions are [`MapPoint`]s: millimetres at map scale, y down. Terms follow `docs/glossary.md`.

use std::fmt;
use std::path::PathBuf;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Version of the project file format written into `format`.
pub const FORMAT_VERSION: u32 = 1;

/// A point on the map in millimetres at map scale, y down. Written as `[x, y]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "[f64; 2]", into = "[f64; 2]")]
pub struct MapPoint {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for MapPoint {
    fn from([x, y]: [f64; 2]) -> Self {
        Self { x, y }
    }
}

impl From<MapPoint> for [f64; 2] {
    fn from(p: MapPoint) -> Self {
        [p.x, p.y]
    }
}

/// The code of a control as printed on the map and the control flag, for example `31` or `S1`.
///
/// Read from a TOML integer or string. Written as an integer when the text is all ASCII digits
/// without a leading zero, otherwise as a string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ControlCode(pub String);

impl fmt::Display for ControlCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for ControlCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = &self.0;
        let plain_number = !s.is_empty()
            && s.bytes().all(|b| b.is_ascii_digit())
            && (!s.starts_with('0') || s == "0");
        match s.parse::<i64>() {
            Ok(n) if plain_number => serializer.serialize_i64(n),
            _ => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for ControlCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CodeVisitor;

        impl Visitor<'_> for CodeVisitor {
            type Value = ControlCode;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a control code as integer or string")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(ControlCode(v.to_string()))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(ControlCode(v.to_string()))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(ControlCode(v.to_owned()))
            }
        }

        deserializer.deserialize_any(CodeVisitor)
    }
}

/// Where the control number is drawn relative to the control circle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberPlacement {
    /// Degrees, 0 = east, counter-clockwise.
    pub direction: f64,
    /// Shortest distance in print millimetres between the outer edge of the circle line and the
    /// number box, which sits centred on the direction ray (agreed with freemapper-print, ADR 0030).
    pub gap_mm: f64,
}

/// Control description: IOF 2024 symbol ids per column C to H, for example `"1.8"`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlDescription {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub d: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub f: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub g: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<String>,
}

/// What a [`Control`] is. A crossing point is listed in the sequence of a course but gets no number
/// (IOF XML `CrossingPoint`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlKind {
    #[default]
    Control,
    CrossingPoint,
}

impl ControlKind {
    fn is_control(&self) -> bool {
        *self == ControlKind::Control
    }
}

/// A control site with code and description, shared by all courses that use it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Control {
    pub code: ControlCode,
    #[serde(default, skip_serializing_if = "ControlKind::is_control")]
    pub kind: ControlKind,
    pub map_mm: MapPoint,
    #[serde(default)]
    pub description: ControlDescription,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<NumberPlacement>,
}

/// A course object that is not a control. `courses` names the courses the object belongs to; empty
/// means all courses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpecialObject {
    Start {
        id: String,
        map_mm: MapPoint,
    },
    Finish {
        id: String,
        map_mm: MapPoint,
    },
    MarkedRoute {
        id: String,
        points: Vec<MapPoint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        courses: Vec<String>,
    },
    OutOfBoundsArea {
        id: String,
        polygon: Vec<MapPoint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        courses: Vec<String>,
    },
    ForbiddenRoute {
        id: String,
        points: Vec<MapPoint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        courses: Vec<String>,
    },
    DangerousArea {
        id: String,
        polygon: Vec<MapPoint>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        courses: Vec<String>,
    },
    FirstAid {
        id: String,
        map_mm: MapPoint,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        courses: Vec<String>,
    },
    Water {
        id: String,
        map_mm: MapPoint,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        courses: Vec<String>,
    },
}

impl SpecialObject {
    pub fn id(&self) -> &str {
        match self {
            Self::Start { id, .. }
            | Self::Finish { id, .. }
            | Self::MarkedRoute { id, .. }
            | Self::OutOfBoundsArea { id, .. }
            | Self::ForbiddenRoute { id, .. }
            | Self::DangerousArea { id, .. }
            | Self::FirstAid { id, .. }
            | Self::Water { id, .. } => id,
        }
    }

    /// The courses the object is limited to; empty for all courses, and always empty for start and finish.
    pub fn courses(&self) -> &[String] {
        match self {
            Self::Start { .. } | Self::Finish { .. } => &[],
            Self::MarkedRoute { courses, .. }
            | Self::OutOfBoundsArea { courses, .. }
            | Self::ForbiddenRoute { courses, .. }
            | Self::DangerousArea { courses, .. }
            | Self::FirstAid { courses, .. }
            | Self::Water { courses, .. } => courses,
        }
    }
}

/// A control as used in one course, with its own number placement and the bends of the leg
/// that leaves it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CourseControl {
    pub code: ControlCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number: Option<NumberPlacement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bends: Vec<MapPoint>,
    /// The leg leaving this control is a marked route.
    #[serde(default, skip_serializing_if = "is_false")]
    pub taped: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// A control of a score course with its value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreControl {
    pub code: ControlCode,
    pub value: u32,
}

/// A part of a relay course: shared by all runners or split into fork variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelayPart {
    Common { controls: Vec<CourseControl> },
    Fork { variants: Vec<Vec<CourseControl>> },
}

/// The kind of a course and the data that depends on it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CourseKind {
    Individual {
        controls: Vec<CourseControl>,
    },
    Team {
        team_size: u8,
        mandatory: Vec<CourseControl>,
        free: Vec<ControlCode>,
    },
    Relay {
        legs: u8,
        parts: Vec<RelayPart>,
    },
    Score {
        time_limit_min: u32,
        penalty_per_min: u32,
        controls: Vec<ScoreControl>,
    },
}

/// The part of the map a course sheet shows: corners in map millimetres, y down.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrintArea {
    pub min: MapPoint,
    pub max: MapPoint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<String>,
    /// Id of the start special object.
    pub start: String,
    /// Id of the finish special object.
    pub finish: String,
    /// Bends of the leg from the start to the first control.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub start_bends: Vec<MapPoint>,
    /// The leg from the start to the first control is a marked route.
    #[serde(default, skip_serializing_if = "is_false")]
    pub start_taped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print_area: Option<PrintArea>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub climb_m: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned_print_scale: Option<u32>,
    #[serde(flatten)]
    pub kind: CourseKind,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Discipline {
    Sprint,
    Middle,
    Long,
    Ultralong,
    #[default]
    Training,
}

impl Discipline {
    /// Every discipline, in declaration order.
    pub const ALL: [Discipline; 5] = [
        Discipline::Sprint,
        Discipline::Middle,
        Discipline::Long,
        Discipline::Ultralong,
        Discipline::Training,
    ];

    /// The name used in project and profile files.
    pub fn as_str(self) -> &'static str {
        match self {
            Discipline::Sprint => "sprint",
            Discipline::Middle => "middle",
            Discipline::Long => "long",
            Discipline::Ultralong => "ultralong",
            Discipline::Training => "training",
        }
    }
}

/// A start class, optionally tied to a rule category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_starters: Option<u32>,
}

/// A finding the course setter marked as an intended exception.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptedFinding {
    pub rule: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<ControlCode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub course: Option<String>,
    /// Legs the finding sits on; part of its key together with `rule` and `controls`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub legs: Vec<LegRef>,
    pub note: String,
}

/// What identifies an accepted finding: rule id, course, and the controls and legs it names.
/// Controls and legs are deduplicated and sorted, so their order in the file does not matter.
/// `course: None` and `Some` are different keys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FindingKey {
    pub rule: String,
    pub course: Option<String>,
    pub controls: Vec<ControlCode>,
    pub legs: Vec<LegRef>,
}

impl AcceptedFinding {
    /// The key a check finding is matched against. Controls whose code is all digits come first,
    /// by numeric value; other codes follow in lexical order.
    pub fn key(&self) -> FindingKey {
        let mut controls = self.controls.clone();
        controls.sort_by(|a, b| control_order(a).cmp(&control_order(b)));
        controls.dedup();
        let mut legs = self.legs.clone();
        legs.sort();
        legs.dedup();
        FindingKey {
            rule: self.rule.clone(),
            course: self.course.clone(),
            controls,
            legs,
        }
    }
}

impl FindingKey {
    /// The key with the ends of its legs left out, so legs compare by course and position only.
    /// `finding remove` names a leg this way, also after its ends changed.
    pub fn by_position(&self) -> FindingKey {
        let mut legs: Vec<LegRef> = self
            .legs
            .iter()
            .map(|l| LegRef::at(l.course.clone(), l.after))
            .collect();
        legs.sort();
        legs.dedup();
        FindingKey {
            legs,
            ..self.clone()
        }
    }
}

/// Sort key of a control code: numeric codes (by value, then spelling) before the rest.
fn control_order(code: &ControlCode) -> (bool, usize, &str, &str) {
    let text = code.0.as_str();
    if !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
        let digits = text.trim_start_matches('0');
        (false, digits.len(), digits, text)
    } else {
        (true, 0, text, text)
    }
}

/// A leg of a course: the leg that leaves position `after` of the course's stop list (1-based,
/// as in the CLI, crossing points included; 0 is the leg from the start), with the ids of the
/// stops at its ends: control codes, or the ids of the start and the finish (ruling R24 of plan
/// M2b). The ends are part of the key, so an accepted finding on a leg comes back once the leg
/// runs between other stops. A file without them reads them as empty; such a key matches no
/// finding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LegRef {
    pub course: String,
    pub after: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub from: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub to: String,
}

impl LegRef {
    /// The leg by course and position only, with empty ends.
    pub fn at(course: impl Into<String>, after: u32) -> LegRef {
        LegRef {
            course: course.into(),
            after,
            from: String::new(),
            to: String::new(),
        }
    }
}

/// The rule profile of an event whose file names none.
pub(crate) const DEFAULT_RULE_PROFILE: &str = "wkb-2026";

fn default_rule_profile() -> String {
    DEFAULT_RULE_PROFILE.to_owned()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMeta {
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Local date, written as an unquoted TOML date.
    pub date: Option<toml::value::Datetime>,
    #[serde(default)]
    pub discipline: Discipline,
    #[serde(default = "default_rule_profile")]
    pub rule_profile: String,
    /// Overlay profiles applied on top of `rule_profile`, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rule_overlays: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_window_min: Option<u32>,
}

/// Input files of the event. Paths are relative to the project file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sources {
    pub map: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ppen: Option<PathBuf>,
}

/// A whole event as stored in the project file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub format: u32,
    pub event: EventMeta,
    pub sources: Sources,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<Class>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub controls: Vec<Control>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub specials: Vec<SpecialObject>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub courses: Vec<Course>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_findings: Vec<AcceptedFinding>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(s: &str) -> ControlCode {
        ControlCode(s.to_owned())
    }

    fn cc(s: &str) -> CourseControl {
        CourseControl {
            code: code(s),
            number: None,
            bends: vec![],
            taped: false,
        }
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Holder {
        code: ControlCode,
    }

    #[test]
    fn control_code_reads_integer_and_string() {
        let a: Holder = toml::from_str("code = 31").unwrap();
        let b: Holder = toml::from_str("code = \"S31\"").unwrap();
        assert_eq!(a.code, code("31"));
        assert_eq!(b.code, code("S31"));
        assert_eq!(a.code.to_string(), "31");

        let ser = |s: &str| toml::to_string(&Holder { code: code(s) }).unwrap();
        assert_eq!(ser("31").trim(), "code = 31");
        assert_eq!(ser("031").trim(), "code = \"031\"");
        assert_eq!(ser("S31").trim(), "code = \"S31\"");
    }

    #[test]
    fn every_course_kind_round_trips() {
        let course = |name: &str, kind| Course {
            name: name.to_owned(),
            classes: vec!["H21".to_owned()],
            start: "S1".to_owned(),
            finish: "F1".to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: Some(35),
            planned_print_scale: Some(4000),
            kind,
        };
        let event = Event {
            format: FORMAT_VERSION,
            event: EventMeta {
                title: "Test".to_owned(),
                date: Some("2026-11-14".parse().unwrap()),
                discipline: Discipline::Sprint,
                rule_profile: "wkb-2026".to_owned(),
                rule_overlays: vec!["nor-bb-2021".to_owned()],
                start_window_min: Some(90),
            },
            sources: Sources {
                map: "maps/a.omap".into(),
                ppen: Some("a.ppen".into()),
            },
            classes: vec![Class {
                name: "H21".to_owned(),
                rule_category: Some("H21".to_owned()),
                expected_starters: Some(12),
            }],
            controls: vec![Control {
                code: code("31"),
                kind: ControlKind::Control,
                map_mm: MapPoint { x: 12.4, y: -33.1 },
                description: ControlDescription {
                    d: Some("1.8".to_owned()),
                    g: Some("11.2".to_owned()),
                    ..Default::default()
                },
                number: Some(NumberPlacement {
                    direction: 45.0,
                    gap_mm: 1.0,
                }),
            }],
            specials: vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    map_mm: MapPoint { x: 0.0, y: 0.0 },
                },
                SpecialObject::MarkedRoute {
                    id: "m1".to_owned(),
                    points: vec![MapPoint { x: 1.0, y: 2.0 }, MapPoint { x: 3.0, y: 4.0 }],
                    courses: vec![],
                },
                SpecialObject::OutOfBoundsArea {
                    id: "o1".to_owned(),
                    polygon: vec![MapPoint { x: 1.0, y: 2.0 }; 3],
                    courses: vec![],
                },
                SpecialObject::Water {
                    id: "w1".to_owned(),
                    map_mm: MapPoint { x: 5.0, y: 5.0 },
                    courses: vec!["A".to_owned()],
                },
            ],
            courses: vec![
                course(
                    "A",
                    CourseKind::Individual {
                        controls: vec![CourseControl {
                            bends: vec![MapPoint { x: 20.0, y: -30.0 }],
                            taped: true,
                            ..cc("31")
                        }],
                    },
                ),
                course(
                    "T",
                    CourseKind::Team {
                        team_size: 3,
                        mandatory: vec![cc("31"), cc("32"), cc("33")],
                        free: vec![code("40"), code("41")],
                    },
                ),
                course(
                    "R",
                    CourseKind::Relay {
                        legs: 3,
                        parts: vec![
                            RelayPart::Common {
                                controls: vec![cc("31")],
                            },
                            RelayPart::Fork {
                                variants: vec![vec![cc("32")], vec![cc("33"), cc("34")]],
                            },
                        ],
                    },
                ),
                course(
                    "S",
                    CourseKind::Score {
                        time_limit_min: 60,
                        penalty_per_min: 30,
                        controls: vec![
                            ScoreControl {
                                code: code("31"),
                                value: 10,
                            },
                            ScoreControl {
                                code: code("32"),
                                value: 40,
                            },
                        ],
                    },
                ),
            ],
            accepted_findings: vec![AcceptedFinding {
                rule: "wkb26.b7.2.1.min_distance".to_owned(),
                controls: vec![code("31"), code("44")],
                course: None,
                legs: vec![LegRef {
                    course: "A".to_owned(),
                    after: 3,
                    from: "33".to_owned(),
                    to: "34".to_owned(),
                }],
                note: "Checked on site.".to_owned(),
            }],
        };
        let text = toml::to_string(&event).unwrap();
        let back: Event = toml::from_str(&text).unwrap();
        assert_eq!(back, event, "{text}");
    }

    #[test]
    fn finding_key_sorts_and_deduplicates() {
        let code = |s: &str| ControlCode(s.to_owned());
        let leg = |c: &str, after| LegRef::at(c, after);
        let finding =
            |controls: Vec<ControlCode>, legs: Vec<LegRef>, course: Option<&str>| AcceptedFinding {
                rule: "r".to_owned(),
                controls,
                course: course.map(str::to_owned),
                legs,
                note: "n".to_owned(),
            };
        let key = finding(
            vec![
                code("100"),
                code("S1"),
                code("31"),
                code("100"),
                code("9"),
                code("A"),
            ],
            vec![leg("B", 1), leg("A", 3), leg("A", 0), leg("A", 3)],
            None,
        )
        .key();
        assert_eq!(
            key.controls,
            vec![code("9"), code("31"), code("100"), code("A"), code("S1")]
        );
        assert_eq!(key.legs, vec![leg("A", 0), leg("A", 3), leg("B", 1)]);
        let shuffled = finding(
            vec![code("A"), code("9"), code("S1"), code("100"), code("31")],
            vec![leg("A", 3), leg("B", 1), leg("A", 0)],
            None,
        );
        assert_eq!(shuffled.key(), key);
        let with_course = finding(shuffled.controls.clone(), shuffled.legs.clone(), Some("A"));
        assert_ne!(with_course.key(), key);
    }

    #[test]
    fn leg_ends_are_part_of_the_key_but_not_of_the_position() {
        let leg = |from: &str, to: &str| LegRef {
            course: "A".to_owned(),
            after: 6,
            from: from.to_owned(),
            to: to.to_owned(),
        };
        let accepted = |legs: Vec<LegRef>| AcceptedFinding {
            rule: "r".to_owned(),
            controls: vec![],
            course: None,
            legs,
            note: "n".to_owned(),
        };
        let old = accepted(vec![leg("36", "37")]).key();
        let moved = accepted(vec![leg("36", "38")]).key();
        assert_ne!(old, moved);
        assert_eq!(old.by_position(), moved.by_position());
        assert_eq!(old.by_position().legs, vec![LegRef::at("A", 6)]);
    }

    #[test]
    fn event_date_is_unquoted_toml_date() {
        let read = |text: &str| -> EventMeta { toml::from_str(text).unwrap() };
        let meta = read("title = \"T\"\ndate = 2026-11-14\n");
        assert_eq!(meta.date, Some("2026-11-14".parse().unwrap()));
        let text = toml::to_string(&meta).unwrap();
        assert!(text.contains("date = 2026-11-14\n"), "{text}");
        let none = read("title = \"T\"\n");
        assert_eq!(none.date, None);
        assert!(!toml::to_string(&none).unwrap().contains("date"));
    }

    #[test]
    fn special_object_kinds_are_spelt_in_snake_case() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Specials {
            specials: Vec<SpecialObject>,
        }
        let p = || MapPoint { x: 1.0, y: 2.0 };
        let specials = Specials {
            specials: vec![
                SpecialObject::Start {
                    id: "S1".to_owned(),
                    map_mm: p(),
                },
                SpecialObject::Finish {
                    id: "F1".to_owned(),
                    map_mm: p(),
                },
                SpecialObject::MarkedRoute {
                    id: "m1".to_owned(),
                    points: vec![p()],
                    courses: vec![],
                },
                SpecialObject::OutOfBoundsArea {
                    id: "o1".to_owned(),
                    polygon: vec![p()],
                    courses: vec![],
                },
                SpecialObject::ForbiddenRoute {
                    id: "f1".to_owned(),
                    points: vec![p()],
                    courses: vec![],
                },
                SpecialObject::DangerousArea {
                    id: "d1".to_owned(),
                    polygon: vec![p()],
                    courses: vec![],
                },
                SpecialObject::FirstAid {
                    id: "a1".to_owned(),
                    map_mm: p(),
                    courses: vec![],
                },
                SpecialObject::Water {
                    id: "w1".to_owned(),
                    map_mm: p(),
                    courses: vec![],
                },
            ],
        };
        let text = toml::to_string(&specials).unwrap();
        let kinds: Vec<&str> = text
            .lines()
            .filter_map(|l| l.strip_prefix("kind = "))
            .collect();
        assert_eq!(
            kinds,
            [
                "\"start\"",
                "\"finish\"",
                "\"marked_route\"",
                "\"out_of_bounds_area\"",
                "\"forbidden_route\"",
                "\"dangerous_area\"",
                "\"first_aid\"",
                "\"water\"",
            ],
            "{text}"
        );
        let back: Specials = toml::from_str(&text).unwrap();
        assert_eq!(back, specials);
    }

    #[test]
    fn omitted_rule_profile_and_discipline_take_their_defaults() {
        let meta: EventMeta = toml::from_str("title = \"T\"\n").unwrap();
        assert_eq!(meta.rule_profile, "wkb-2026");
        assert_eq!(meta.discipline, Discipline::Training);
        assert_eq!(
            toml::to_string(&meta).unwrap(),
            "title = \"T\"\ndiscipline = \"training\"\nrule_profile = \"wkb-2026\"\n"
        );
        let spelt = |text: &str| toml::from_str::<EventMeta>(text).unwrap().discipline;
        assert_eq!(
            spelt("title = \"T\"\ndiscipline = \"sprint\"\n"),
            Discipline::Sprint
        );
        assert_eq!(
            spelt("title = \"T\"\ndiscipline = \"middle\"\n"),
            Discipline::Middle
        );
        assert_eq!(
            spelt("title = \"T\"\ndiscipline = \"long\"\n"),
            Discipline::Long
        );
        assert_eq!(
            spelt("title = \"T\"\ndiscipline = \"ultralong\"\n"),
            Discipline::Ultralong
        );
    }

    #[test]
    fn individual_course_toml_shape() {
        let text = r#"
name = "A"
kind = "individual"
classes = ["H21", "D21"]
start = "S1"
finish = "F1"
climb_m = 35
controls = [
  { code = 31, number = { direction = 270, gap_mm = 1.0 } },
  { code = 32, bends = [[20.0, -30.0]] },
]
"#;
        let got: Course = toml::from_str(text).unwrap();
        let want = Course {
            name: "A".to_owned(),
            classes: vec!["H21".to_owned(), "D21".to_owned()],
            start: "S1".to_owned(),
            finish: "F1".to_owned(),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: Some(35),
            planned_print_scale: None,
            kind: CourseKind::Individual {
                controls: vec![
                    CourseControl {
                        code: code("31"),
                        number: Some(NumberPlacement {
                            direction: 270.0,
                            gap_mm: 1.0,
                        }),
                        bends: vec![],
                        taped: false,
                    },
                    CourseControl {
                        code: code("32"),
                        number: None,
                        bends: vec![MapPoint { x: 20.0, y: -30.0 }],
                        taped: false,
                    },
                ],
            },
        };
        assert_eq!(got, want);
    }
}
