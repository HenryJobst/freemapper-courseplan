//! The parameters of the MCP tools. Their descriptions come from the catalog
//! (`localize_schema` in the parent module), not from doc comments.

use freemapper_courseplan_model::MapPoint;
use schemars::JsonSchema;
use serde::Deserialize;

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
pub(super) struct AddControl {
    pub(super) project: String,
    pub(super) at: Point,
    pub(super) code: Option<String>,
    pub(super) crossing_point: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct MoveControl {
    pub(super) project: String,
    pub(super) code: String,
    pub(super) at: Point,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ControlRef {
    pub(super) project: String,
    pub(super) code: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DescribeControl {
    pub(super) project: String,
    pub(super) code: String,
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
    pub(super) code: String,
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
    pub(super) id: String,
    pub(super) at: Option<Point>,
    pub(super) points: Option<Vec<Point>>,
    pub(super) courses: Option<Vec<String>>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RemoveSpecial {
    pub(super) project: String,
    pub(super) id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetSpecialCourses {
    pub(super) project: String,
    pub(super) id: String,
    pub(super) courses: Option<Vec<String>>,
    pub(super) all: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddClass {
    pub(super) project: String,
    pub(super) name: String,
    pub(super) rule_category: Option<String>,
    pub(super) expected_starters: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RemoveClass {
    pub(super) project: String,
    pub(super) name: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddCourse {
    pub(super) project: String,
    pub(super) name: String,
    pub(super) start: String,
    pub(super) finish: String,
    pub(super) controls: Option<Vec<String>>,
    pub(super) classes: Option<Vec<String>>,
    pub(super) climb_m: Option<u32>,
    pub(super) print_scale: Option<u32>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CourseRef {
    pub(super) project: String,
    pub(super) name: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetCourseControls {
    pub(super) project: String,
    pub(super) name: String,
    pub(super) codes: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetLeg {
    pub(super) project: String,
    pub(super) name: String,
    pub(super) from_start: Option<bool>,
    pub(super) after: Option<u32>,
    pub(super) bends: Option<Vec<Point>>,
    pub(super) taped: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetCourseDetails {
    pub(super) project: String,
    pub(super) name: String,
    pub(super) classes: Option<Vec<String>>,
    pub(super) climb_m: Option<u32>,
    pub(super) no_climb: Option<bool>,
    pub(super) print_scale: Option<u32>,
    pub(super) no_print_scale: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct PlaceNumbers {
    pub(super) project: String,
    pub(super) name: String,
    pub(super) overwrite: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct SetCourseNumber {
    pub(super) project: String,
    pub(super) name: String,
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
