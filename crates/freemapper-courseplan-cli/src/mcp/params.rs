//! The parameters of the MCP tools. Their descriptions come from the catalog
//! (`localize_schema` in the parent module), not from doc comments.

use std::borrow::Cow;
use std::fmt;

use freemapper_courseplan_model::MapPoint;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

/// A name or id from the assistant, without surrounding white space.
#[derive(Clone)]
pub(super) struct Name(pub(super) String);

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Name(String::deserialize(deserializer)?.trim().to_owned()))
    }
}

impl JsonSchema for Name {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        "Name".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        String::json_schema(generator)
    }
}

/// The names of a list; empty ones are left out.
pub(super) fn name_list(list: Vec<Name>) -> Vec<String> {
    list.into_iter()
        .map(|n| n.0)
        .filter(|n| !n.is_empty())
        .collect()
}

/// A control code: a JSON string or a whole number, as in the project file, without surrounding
/// white space.
#[derive(Clone)]
pub(super) struct Code(pub(super) String);

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CodeVisitor;

        impl Visitor<'_> for CodeVisitor {
            type Value = Code;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a control code as string or whole number")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Code, E> {
                Ok(Code(v.to_string()))
            }

            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Code, E> {
                Ok(Code(v.to_string()))
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Code, E> {
                Ok(Code(v.trim().to_owned()))
            }
        }

        deserializer.deserialize_any(CodeVisitor)
    }
}

impl JsonSchema for Code {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        "ControlCode".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({ "type": ["string", "integer"] })
    }
}

/// The codes of a list; empty ones are left out.
pub(super) fn code_list(list: Vec<Code>) -> Vec<String> {
    list.into_iter()
        .map(|c| c.0)
        .filter(|c| !c.is_empty())
        .collect()
}

/// A position on the map in millimetres at map scale, y pointing down.
#[derive(Deserialize, JsonSchema, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub(super) struct Point {
    pub(super) x: f64,
    pub(super) y: f64,
}

impl Point {
    pub(super) fn map_point(self) -> MapPoint {
        MapPoint {
            x: self.x,
            y: self.y,
        }
    }
}

pub(super) fn map_points(points: Vec<Point>) -> Vec<MapPoint> {
    points.into_iter().map(Point::map_point).collect()
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ProjectOnly {
    pub(super) project: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ExportIofXml {
    pub(super) project: String,
    pub(super) path: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ExportPpen {
    pub(super) project: String,
    pub(super) path: String,
    pub(super) new: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ImportPpen {
    pub(super) project: String,
    pub(super) path: String,
    pub(super) replace: Option<bool>,
}

/// The side of a sync conflict to keep.
#[derive(Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub(super) enum SyncSide {
    Ours,
    Theirs,
}

impl From<SyncSide> for freemapper_courseplan_model::Side {
    fn from(side: SyncSide) -> Self {
        match side {
            SyncSide::Ours => Self::Ours,
            SyncSide::Theirs => Self::Theirs,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SyncPpen {
    pub(super) project: String,
    pub(super) dry_run: Option<bool>,
    pub(super) take: Option<SyncSide>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ResolveSyncConflict {
    pub(super) project: String,
    pub(super) key: String,
    pub(super) take: SyncSide,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddControl {
    pub(super) project: String,
    pub(super) at: Point,
    pub(super) code: Option<Code>,
    pub(super) crossing_point: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct MoveControl {
    pub(super) project: String,
    pub(super) code: Code,
    pub(super) at: Point,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ControlRef {
    pub(super) project: String,
    pub(super) code: Code,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetControlCode {
    pub(super) project: String,
    pub(super) from: Code,
    pub(super) to: Code,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RenameCourse {
    pub(super) project: String,
    pub(super) from: Name,
    pub(super) to: Name,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DescribeControl {
    pub(super) project: String,
    pub(super) code: Code,
    pub(super) c: Option<String>,
    pub(super) d: Option<String>,
    pub(super) e: Option<String>,
    pub(super) f: Option<String>,
    pub(super) g: Option<String>,
    pub(super) h: Option<String>,
    pub(super) clear: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetControlNumber {
    pub(super) project: String,
    pub(super) code: Code,
    pub(super) direction: Option<f64>,
    pub(super) gap_mm: Option<f64>,
    pub(super) clear: Option<bool>,
}

/// The kinds of special objects, named as in the project file.
#[derive(Deserialize, JsonSchema, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub(super) enum SpecialKindArg {
    Start,
    Finish,
    MarkedRoute,
    OutOfBounds,
    ForbiddenRoute,
    DangerousArea,
    FirstAid,
    Water,
}

impl SpecialKindArg {
    pub(super) fn word_id(self) -> &'static str {
        match self {
            Self::Start => "word-special-start",
            Self::Finish => "word-special-finish",
            Self::MarkedRoute => "word-special-marked-route",
            Self::OutOfBounds => "word-special-out-of-bounds",
            Self::ForbiddenRoute => "word-special-forbidden-route",
            Self::DangerousArea => "word-special-dangerous-area",
            Self::FirstAid => "word-special-first-aid",
            Self::Water => "word-special-water",
        }
    }

    pub(super) fn is_point(self) -> bool {
        matches!(
            self,
            Self::Start | Self::Finish | Self::FirstAid | Self::Water
        )
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddSpecial {
    pub(super) project: String,
    pub(super) kind: SpecialKindArg,
    pub(super) id: Name,
    pub(super) at: Option<Point>,
    pub(super) points: Option<Vec<Point>>,
    pub(super) courses: Option<Vec<Name>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RemoveSpecial {
    pub(super) project: String,
    pub(super) id: Name,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetSpecialCourses {
    pub(super) project: String,
    pub(super) id: Name,
    pub(super) courses: Option<Vec<Name>>,
    pub(super) all: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddClass {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) rule_category: Option<String>,
    pub(super) expected_starters: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetClass {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) rule_category: Option<String>,
    pub(super) no_rule_category: Option<bool>,
    pub(super) expected_starters: Option<u32>,
    pub(super) no_expected_starters: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RemoveClass {
    pub(super) project: String,
    pub(super) name: Name,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddCourse {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) start: Name,
    pub(super) finish: Name,
    pub(super) controls: Option<Vec<Code>>,
    pub(super) classes: Option<Vec<Name>>,
    pub(super) climb_m: Option<u32>,
    pub(super) print_scale: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CourseRef {
    pub(super) project: String,
    pub(super) name: Name,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetCourseControls {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) codes: Vec<Code>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetLeg {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) from_start: Option<bool>,
    pub(super) after: Option<u32>,
    pub(super) bends: Option<Vec<Point>>,
    pub(super) taped: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetCourseDetails {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) classes: Option<Vec<Name>>,
    pub(super) climb_m: Option<u32>,
    pub(super) no_climb: Option<bool>,
    pub(super) print_scale: Option<u32>,
    pub(super) no_print_scale: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct PlaceNumbers {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) overwrite: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetCourseNumber {
    pub(super) project: String,
    pub(super) name: Name,
    pub(super) after: u32,
    pub(super) direction: Option<f64>,
    pub(super) gap_mm: Option<f64>,
    pub(super) clear: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct FeaturesNear {
    pub(super) project: String,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) radius_mm: f64,
    pub(super) limit: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct InitEvent {
    pub(super) map: String,
    pub(super) project: Option<String>,
    pub(super) title: Option<String>,
}

/// A leg of a course: the leg after position `after`, counted from 1; 0 is the leg from the start.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct LegArg {
    pub(super) course: Name,
    pub(super) after: u32,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AcceptFinding {
    pub(super) project: String,
    pub(super) rule: Name,
    pub(super) controls: Option<Vec<Code>>,
    pub(super) legs: Option<Vec<LegArg>>,
    pub(super) course: Option<Name>,
    pub(super) note: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RemoveAcceptedFinding {
    pub(super) project: String,
    pub(super) rule: Name,
    pub(super) controls: Option<Vec<Code>>,
    pub(super) legs: Option<Vec<LegArg>>,
    pub(super) course: Option<Name>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetRuleProfile {
    pub(super) project: String,
    pub(super) base: Name,
    pub(super) overlays: Option<Vec<Name>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetStartWindow {
    pub(super) project: String,
    pub(super) minutes: Option<u32>,
    pub(super) no_start_window: Option<bool>,
}
