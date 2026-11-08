//! `freemapper-courseplan mcp`: the MCP server on stdio, so an AI assistant can set courses for a
//! user. The tools are thin wrappers over the library: each call loads the project, applies one
//! edit, saves on success and answers with compact JSON. Tool, parameter and field names stay
//! English; tool descriptions, parameter descriptions, the server instructions and the error
//! texts come from the catalog in the language chosen at start (ruling R1).

use std::path::PathBuf;
use std::sync::Arc;

use freemapper_courseplan::course::{control_numbers, course_length_m};
use freemapper_courseplan::edit::{self, LegStart};
use freemapper_courseplan::init::init_project;
use freemapper_courseplan::map::{FeatureGeometry, MapIndex, MapKind, map_kind};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{
    Class, Control, ControlCode, ControlDescription, ControlKind, Course, CourseKind, Event,
    MapPoint, NumberPlacement, ProjectFile, SpecialObject,
};
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde_json::{Map, Value, json};

use crate::edit_cmd::{self, Failure};

mod params;
use params::*;

/// Most features `map_features_near` returns, and the default.
const LIMIT_DEFAULT: usize = 20;
const LIMIT_MAX: usize = 200;

// ---------------------------------------------------------------------------------------------
// Results

type Answer = Result<Value, String>;

/// A tool result with one text block.
fn success(mut value: Value) -> Result<CallToolResult, McpError> {
    if let Value::Object(map) = &mut value {
        map.insert("ok".to_owned(), Value::Bool(true));
    }
    Ok(CallToolResult::success(vec![ContentBlock::text(
        value.to_string(),
    )]))
}

/// A tool error the assistant can relay to the user.
fn failure(message: String) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![ContentBlock::text(message)]))
}

fn reply(answer: Answer) -> Result<CallToolResult, McpError> {
    match answer {
        Ok(value) => success(value),
        Err(message) => failure(message),
    }
}

fn pt(p: &MapPoint) -> Value {
    json!({ "x": p.x, "y": p.y })
}

fn pts(points: &[MapPoint]) -> Value {
    Value::Array(points.iter().map(pt).collect())
}

fn placement(number: &Option<NumberPlacement>) -> Value {
    match number {
        Some(n) => json!({ "direction": n.direction, "gap_mm": n.gap_mm }),
        None => Value::Null,
    }
}

fn round(x: f64, digits: i32) -> f64 {
    let f = 10f64.powi(digits);
    (x * f).round() / f
}

fn map_kind_name(kind: MapKind) -> &'static str {
    match kind {
        MapKind::Sprint => "sprint",
        MapKind::Schoolyard => "schoolyard",
        MapKind::Forest => "forest",
    }
}

fn control_json(c: &Control) -> Value {
    json!({
        "code": c.code.0,
        "kind": match c.kind {
            ControlKind::Control => "control",
            ControlKind::CrossingPoint => "crossing_point",
        },
        "map_mm": pt(&c.map_mm),
        "description": serde_json::to_value(&c.description).unwrap_or(Value::Null),
        "number": placement(&c.number),
    })
}

fn special_json(s: &SpecialObject) -> Value {
    let (kind, geometry) = match s {
        SpecialObject::Start { map_mm, .. } => ("start", ("at", pt(map_mm))),
        SpecialObject::Finish { map_mm, .. } => ("finish", ("at", pt(map_mm))),
        SpecialObject::FirstAid { map_mm, .. } => ("first_aid", ("at", pt(map_mm))),
        SpecialObject::Water { map_mm, .. } => ("water", ("at", pt(map_mm))),
        SpecialObject::MarkedRoute { points, .. } => ("marked_route", ("points", pts(points))),
        SpecialObject::ForbiddenRoute { points, .. } => {
            ("forbidden_route", ("points", pts(points)))
        }
        SpecialObject::OutOfBoundsArea { polygon, .. } => {
            ("out_of_bounds", ("points", pts(polygon)))
        }
        SpecialObject::DangerousArea { polygon, .. } => {
            ("dangerous_area", ("points", pts(polygon)))
        }
    };
    let mut out = Map::new();
    out.insert("id".to_owned(), json!(s.id()));
    out.insert("kind".to_owned(), json!(kind));
    out.insert(geometry.0.to_owned(), geometry.1);
    out.insert("courses".to_owned(), json!(s.courses()));
    Value::Object(out)
}

/// One course for `event_info` and `list_courses`. Only individual courses are measured.
fn course_json(event: &Event, course: &Course, map_scale: u32, lang: &Lang) -> Answer {
    let kind = match course.kind {
        CourseKind::Individual { .. } => "individual",
        CourseKind::Team { .. } => "team",
        CourseKind::Relay { .. } => "relay",
        CourseKind::Score { .. } => "score",
    };
    let mut out = json!({
        "name": course.name,
        "kind": kind,
        "start": course.start,
        "finish": course.finish,
        "classes": course.classes,
        "climb_m": course.climb_m,
        "print_scale": course.planned_print_scale,
        "controls": [],
        "control_count": 0,
        "numbers_set": 0,
        "length_km": Value::Null,
    });
    if let CourseKind::Individual { controls } = &course.kind {
        let length = course_length_m(event, course, map_scale).map_err(|e| e.localize(lang))?;
        out["controls"] = json!(controls.iter().map(|c| &c.code.0).collect::<Vec<_>>());
        out["control_count"] = json!(control_numbers(event, course).iter().flatten().count());
        out["numbers_set"] = json!(controls.iter().filter(|c| c.number.is_some()).count());
        out["length_km"] = json!(round(length / 1000.0, 3));
    }
    Ok(out)
}

/// The paths: absolute, or `~/…` inside the home folder. Relative paths are refused because the
/// server's working folder is unknown to the assistant.
fn resolve(lang: &Lang, path: &str) -> Result<PathBuf, String> {
    let path = path.trim();
    let resolved = if path == "~" || path.starts_with("~/") || path.starts_with("~\\") {
        let home = dirs::home_dir().ok_or_else(|| text(lang, "mcp-error-no-home", &[]))?;
        home.join(path[1..].trim_start_matches(['/', '\\']))
    } else {
        PathBuf::from(path)
    };
    if !resolved.is_absolute() {
        return Err(text(
            lang,
            "mcp-error-relative-path",
            &[("path", FluentValue::from(path.to_owned()))],
        ));
    }
    Ok(resolved)
}

fn flag(value: Option<bool>) -> bool {
    value.unwrap_or(false)
}

// ---------------------------------------------------------------------------------------------
// The server

#[derive(Clone)]
pub struct CoursePlanServer {
    tool_router: ToolRouter<Self>,
    lang: Lang,
}

#[tool_router]
impl CoursePlanServer {
    pub fn new(lang: Lang) -> Self {
        let mut tool_router = Self::tool_router();
        // `#[tool(description = ...)]` takes a static string, so the macro holds an English
        // placeholder and the catalog text replaces it here, for every tool and parameter.
        for (name, route) in &mut tool_router.map {
            let id = name.replace('_', "-");
            route.attr.description = Some(text(&lang, &format!("mcp-tool-{id}"), &[]).into());
            let mut schema = Value::Object((*route.attr.input_schema).clone());
            localize_schema(&mut schema, &lang, &id);
            if let Value::Object(map) = schema {
                route.attr.input_schema = Arc::new(map);
            }
        }
        Self { tool_router, lang }
    }

    fn load(&self, project: &str) -> Result<ProjectFile, String> {
        let path = resolve(&self.lang, project)?;
        edit_cmd::load(&self.lang, &path).map_err(Failure::into_message)
    }

    /// Loads the project, applies `change`, and saves when it succeeded.
    fn edit<T>(
        &self,
        project: &str,
        change: impl FnOnce(&mut ProjectFile) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut file = self.load(project)?;
        let out = change(&mut file)?;
        edit_cmd::save(&self.lang, &mut file).map_err(Failure::into_message)?;
        Ok(out)
    }

    fn refused(&self, e: edit::EditError) -> String {
        e.localize(&self.lang)
    }

    fn extent(&self, file: &ProjectFile) -> Result<Option<(MapPoint, MapPoint)>, String> {
        edit_cmd::map_extent(&self.lang, file).map_err(Failure::into_message)
    }

    #[tool(description = "Summary of the event")]
    async fn event_info(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let doc = edit_cmd::read_map(&self.lang, &file).map_err(Failure::into_message)?;
            let scale = doc.georef.scale_denominator;
            let extent = MapIndex::from_document(&doc).extent();
            let event = &file.event;
            let courses = event
                .courses
                .iter()
                .map(|c| course_json(event, c, scale, &self.lang))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({
                "title": event.event.title,
                "map": event.sources.map.display().to_string(),
                "map_kind": map_kind_name(map_kind(&doc)),
                "scale": scale,
                "extent": extent_json(extent),
                "counts": {
                    "controls": event.controls.len(),
                    "specials": event.specials.len(),
                    "classes": event.classes.len(),
                    "courses": event.courses.len(),
                },
                "courses": courses,
            }))
        })())
    }

    #[tool(description = "List the controls")]
    async fn list_controls(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.load(&req.project).map(|file| {
            json!({ "controls": file.event.controls.iter().map(control_json).collect::<Vec<_>>() })
        }))
    }

    #[tool(description = "List the courses")]
    async fn list_courses(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let doc = edit_cmd::read_map(&self.lang, &file).map_err(Failure::into_message)?;
            let scale = doc.georef.scale_denominator;
            let courses = file
                .event
                .courses
                .iter()
                .map(|c| course_json(&file.event, c, scale, &self.lang))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({ "courses": courses }))
        })())
    }

    #[tool(description = "List the special objects")]
    async fn list_specials(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.load(&req.project).map(|file| {
            json!({ "specials": file.event.specials.iter().map(special_json).collect::<Vec<_>>() })
        }))
    }

    #[tool(description = "List the classes")]
    async fn list_classes(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.load(&req.project).map(|file| {
            let classes: Vec<Value> = file
                .event
                .classes
                .iter()
                .map(|c| {
                    json!({
                        "name": c.name,
                        "rule_category": c.rule_category,
                        "expected_starters": c.expected_starters,
                    })
                })
                .collect();
            json!({ "classes": classes })
        }))
    }

    #[tool(description = "Add a control")]
    async fn add_control(
        &self,
        Parameters(req): Parameters<AddControl>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let extent = self.extent(file)?;
            let kind = if flag(req.crossing_point) {
                ControlKind::CrossingPoint
            } else {
                ControlKind::Control
            };
            let code = edit::add_control(
                &mut file.event,
                extent,
                req.at.map_point(),
                req.code.map(ControlCode),
                kind,
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "code": code.0 }))
        }))
    }

    #[tool(description = "Move a control")]
    async fn move_control(
        &self,
        Parameters(req): Parameters<MoveControl>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let extent = self.extent(file)?;
            edit::move_control(
                &mut file.event,
                extent,
                &ControlCode(req.code.clone()),
                req.at.map_point(),
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "code": req.code }))
        }))
    }

    #[tool(description = "Remove a control")]
    async fn remove_control(
        &self,
        Parameters(req): Parameters<ControlRef>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let left = edit::remove_control(&mut file.event, &ControlCode(req.code.clone()))
                .map_err(|e| self.refused(e))?;
            Ok(json!({ "code": req.code, "left_courses": left }))
        }))
    }

    #[tool(description = "Describe a control")]
    async fn describe_control(
        &self,
        Parameters(req): Parameters<DescribeControl>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let key = ControlCode(req.code.clone());
            // An unknown code is reported by the edit operation below.
            let current = file
                .event
                .controls
                .iter()
                .find(|c| c.code == key)
                .map(|c| c.description.clone());
            let mut description = if flag(req.clear) {
                ControlDescription::default()
            } else {
                current.unwrap_or_default()
            };
            for (slot, new) in [
                (&mut description.c, req.c),
                (&mut description.d, req.d),
                (&mut description.e, req.e),
                (&mut description.f, req.f),
                (&mut description.g, req.g),
                (&mut description.h, req.h),
            ] {
                if new.is_some() {
                    *slot = new;
                }
            }
            edit::describe_control(&mut file.event, &key, description.clone())
                .map_err(|e| self.refused(e))?;
            Ok(json!({
                "code": req.code,
                "description": serde_json::to_value(&description).unwrap_or(Value::Null),
            }))
        }))
    }

    #[tool(description = "Set the number placement of a control")]
    async fn set_control_number(
        &self,
        Parameters(req): Parameters<SetControlNumber>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let number = match (req.direction, req.gap_mm, flag(req.clear)) {
                (Some(direction), Some(gap_mm), false) => {
                    Some(NumberPlacement { direction, gap_mm })
                }
                (None, None, true) => None,
                _ => return Err(text(&self.lang, "mcp-error-number-args", &[])),
            };
            edit::set_central_number(
                &mut file.event,
                &ControlCode(req.code.clone()),
                number.clone(),
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "code": req.code, "number": placement(&number) }))
        }))
    }

    #[tool(description = "Add a special object")]
    async fn add_special(
        &self,
        Parameters(req): Parameters<AddSpecial>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let kind = req.kind;
            let wrong = |id: &str| {
                text(
                    &self.lang,
                    id,
                    &[(
                        "kind",
                        FluentValue::from(text(&self.lang, kind.word_id(), &[])),
                    )],
                )
            };
            let points = req.points.unwrap_or_default();
            let courses = req.courses.unwrap_or_default();
            if kind.is_point() && (req.at.is_none() || !points.is_empty()) {
                return Err(wrong("mcp-error-kind-needs-at"));
            }
            if !kind.is_point() && (req.at.is_some() || points.is_empty()) {
                return Err(wrong("mcp-error-kind-needs-points"));
            }
            if matches!(kind, SpecialKindArg::Start | SpecialKindArg::Finish) && !courses.is_empty()
            {
                return Err(wrong("mcp-error-no-courses-for-kind"));
            }
            let id = req.id.clone();
            let at = || {
                req.at
                    .map(Point::map_point)
                    .unwrap_or(MapPoint { x: 0.0, y: 0.0 })
            };
            let points = map_points(points);
            let special = match kind {
                SpecialKindArg::Start => SpecialObject::Start { id, map_mm: at() },
                SpecialKindArg::Finish => SpecialObject::Finish { id, map_mm: at() },
                SpecialKindArg::FirstAid => SpecialObject::FirstAid {
                    id,
                    map_mm: at(),
                    courses,
                },
                SpecialKindArg::Water => SpecialObject::Water {
                    id,
                    map_mm: at(),
                    courses,
                },
                SpecialKindArg::MarkedRoute => SpecialObject::MarkedRoute {
                    id,
                    points,
                    courses,
                },
                SpecialKindArg::ForbiddenRoute => SpecialObject::ForbiddenRoute {
                    id,
                    points,
                    courses,
                },
                SpecialKindArg::OutOfBounds => SpecialObject::OutOfBoundsArea {
                    id,
                    polygon: points,
                    courses,
                },
                SpecialKindArg::DangerousArea => SpecialObject::DangerousArea {
                    id,
                    polygon: points,
                    courses,
                },
            };
            let extent = self.extent(file)?;
            edit::add_special(&mut file.event, extent, special).map_err(|e| self.refused(e))?;
            Ok(json!({ "id": req.id }))
        }))
    }

    #[tool(description = "Remove a special object")]
    async fn remove_special(
        &self,
        Parameters(req): Parameters<RemoveSpecial>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::remove_special(&mut file.event, &req.id).map_err(|e| self.refused(e))?;
            Ok(json!({ "id": req.id }))
        }))
    }

    #[tool(description = "Add a class")]
    async fn add_class(
        &self,
        Parameters(req): Parameters<AddClass>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let class = Class {
                name: req.name.clone(),
                rule_category: req.rule_category,
                expected_starters: req.expected_starters,
            };
            edit::add_class(&mut file.event, class).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name }))
        }))
    }

    #[tool(description = "Remove a class")]
    async fn remove_class(
        &self,
        Parameters(req): Parameters<RemoveClass>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::remove_class(&mut file.event, &req.name).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name }))
        }))
    }

    #[tool(description = "Add an individual course")]
    async fn add_course(
        &self,
        Parameters(req): Parameters<AddCourse>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let controls = edit_cmd::codes(req.controls.unwrap_or_default())
                .into_iter()
                .map(edit_cmd::course_control)
                .collect();
            let course = Course {
                name: req.name.clone(),
                classes: edit_cmd::codes(req.classes.unwrap_or_default()),
                start: req.start,
                finish: req.finish,
                start_bends: vec![],
                start_taped: false,
                print_area: None,
                climb_m: req.climb_m,
                planned_print_scale: req.print_scale,
                kind: CourseKind::Individual { controls },
            };
            edit::add_course(&mut file.event, course).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name }))
        }))
    }

    #[tool(description = "Remove a course")]
    async fn remove_course(
        &self,
        Parameters(req): Parameters<CourseRef>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::remove_course(&mut file.event, &req.name).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name }))
        }))
    }

    #[tool(description = "Replace the controls of a course")]
    async fn set_course_controls(
        &self,
        Parameters(req): Parameters<SetCourseControls>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let codes = edit_cmd::codes(req.codes.iter().map(|c| c.trim().to_owned()));
            let sequence: Vec<ControlCode> = codes.iter().cloned().map(ControlCode).collect();
            edit::set_course_controls(&mut file.event, &req.name, &sequence)
                .map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name, "controls": codes }))
        }))
    }

    #[tool(description = "Set the bends and the marked-route flag of one leg")]
    async fn set_leg(
        &self,
        Parameters(req): Parameters<SetLeg>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            // The assistant counts from 1 as on the map; the library counts from 0.
            let bad_index = |index: u32| {
                self.refused(edit::EditError::BadIndex {
                    course: req.name.clone(),
                    index: index as usize,
                })
            };
            let leg = match (flag(req.from_start), req.after) {
                (true, None) => LegStart::Start,
                (false, Some(0)) => return Err(bad_index(0)),
                (false, Some(n)) => LegStart::After(n as usize - 1),
                _ => return Err(text(&self.lang, "mcp-error-leg-target", &[])),
            };
            let bends = map_points(req.bends.unwrap_or_default());
            let taped = flag(req.taped);
            let count = bends.len();
            edit::set_leg(&mut file.event, &req.name, leg, bends, taped).map_err(|e| match e {
                edit::EditError::BadIndex { .. } => bad_index(req.after.unwrap_or(0)),
                other => self.refused(other),
            })?;
            Ok(json!({ "name": req.name, "bends": count, "taped": taped }))
        }))
    }

    #[tool(description = "Change classes, climb or print scale of a course")]
    async fn set_course_details(
        &self,
        Parameters(req): Parameters<SetCourseDetails>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let climb = match (req.climb_m, flag(req.no_climb)) {
                (Some(c), _) => Some(Some(c)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            let scale = match (req.print_scale, flag(req.no_print_scale)) {
                (Some(s), _) => Some(Some(s)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            let classes = req.classes.map(edit_cmd::codes);
            if classes.is_none() && climb.is_none() && scale.is_none() {
                return Err(text(&self.lang, "mcp-error-no-change", &[]));
            }
            edit::set_course_details(&mut file.event, &req.name, classes, climb, scale)
                .map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name }))
        }))
    }

    #[tool(description = "Place the control numbers of a course")]
    async fn place_numbers(
        &self,
        Parameters(req): Parameters<PlaceNumbers>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let doc = edit_cmd::read_map(&self.lang, file).map_err(Failure::into_message)?;
            let placed = freemapper_courseplan::numbers::place_numbers(
                &mut file.event,
                &req.name,
                doc.georef.scale_denominator,
                map_kind(&doc),
                flag(req.overwrite),
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name, "placed": placed }))
        }))
    }

    #[tool(description = "Map objects near a position")]
    async fn map_features_near(
        &self,
        Parameters(req): Parameters<FeaturesNear>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let doc = edit_cmd::read_map(&self.lang, &file).map_err(Failure::into_message)?;
            let index = MapIndex::from_document(&doc);
            let limit = req
                .limit
                .map_or(LIMIT_DEFAULT, |l| l as usize)
                .clamp(1, LIMIT_MAX);
            let at = MapPoint { x: req.x, y: req.y };
            let near = index.features_near(at, req.radius_mm);
            let total = near.len();
            let features: Vec<Value> = near
                .into_iter()
                .take(limit)
                .map(|(f, distance)| {
                    json!({
                        "symbol_code": f.symbol_code,
                        "kind": match f.geometry {
                            FeatureGeometry::Point(_) => "point",
                            FeatureGeometry::Line(_) => "line",
                            FeatureGeometry::Area(_) => "area",
                        },
                        "distance_mm": round(distance, 2),
                        "part": f.part,
                    })
                })
                .collect();
            Ok(json!({ "features": features, "total": total }))
        })())
    }

    #[tool(description = "Extent of the map")]
    async fn map_extent(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let doc = edit_cmd::read_map(&self.lang, &file).map_err(Failure::into_message)?;
            Ok(json!({
                "extent": extent_json(MapIndex::from_document(&doc).extent()),
                "scale": doc.georef.scale_denominator,
                "map_kind": map_kind_name(map_kind(&doc)),
            }))
        })())
    }

    #[tool(description = "Start an event from an OOM map")]
    async fn init_event(
        &self,
        Parameters(req): Parameters<InitEvent>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let map = resolve(&self.lang, &req.map)?;
            let project = match &req.project {
                Some(p) => resolve(&self.lang, p)?,
                None => crate::default_project_path(&map),
            };
            let created = init_project(&map, &project, req.title.clone())
                .map_err(|e| e.localize(&self.lang))?;
            Ok(json!({ "project": created.path.display().to_string() }))
        })())
    }
}

fn extent_json(extent: Option<(MapPoint, MapPoint)>) -> Value {
    match extent {
        Some((min, max)) => json!({ "min": pt(&min), "max": pt(&max) }),
        None => Value::Null,
    }
}

/// Sets the catalog description on every property of a tool's input schema, including the
/// properties of types it refers to (`Point`). A parameter has its own text under
/// `mcp-param-<tool>-<name>`, else the shared one under `mcp-param-<name>`.
fn localize_schema(schema: &mut Value, lang: &Lang, tool: &str) {
    match schema {
        Value::Object(map) => {
            if let Some(Value::Object(properties)) = map.get_mut("properties") {
                for (name, property) in properties.iter_mut() {
                    let name = name.replace('_', "-");
                    let own = text(lang, &format!("mcp-param-{tool}-{name}"), &[]);
                    let shared = text(lang, &format!("mcp-param-{name}"), &[]);
                    let description = if !own.starts_with("mcp-param-") {
                        own
                    } else if !shared.starts_with("mcp-param-") {
                        shared
                    } else {
                        continue;
                    };
                    if let Value::Object(property) = property {
                        property.insert("description".to_owned(), Value::String(description));
                    }
                }
            }
            map.values_mut()
                .for_each(|v| localize_schema(v, lang, tool));
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|v| localize_schema(v, lang, tool)),
        _ => {}
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for CoursePlanServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "freemapper-courseplan",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(text(&self.lang, "mcp-instructions", &[]))
    }
}

/// Serves MCP on stdin and stdout until the client disconnects.
pub fn run(lang: Lang) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async {
        let service = CoursePlanServer::new(lang)
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| e.to_string())?;
        service.waiting().await.map_err(|e| e.to_string())?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan::text::available;

    #[test]
    fn paths_must_be_absolute_or_home() {
        let lang = Lang::english();
        let refused = resolve(&lang, "projects/x.toml").unwrap_err();
        assert!(refused.contains("absolute path"), "{refused}");
        let home = dirs::home_dir().unwrap();
        assert_eq!(resolve(&lang, "~/maps/x").unwrap(), home.join("maps/x"));
        assert_eq!(resolve(&lang, "~\\maps").unwrap(), home.join("maps"));
        let absolute = std::env::temp_dir().join("x");
        assert_eq!(
            resolve(&lang, absolute.to_str().unwrap()).unwrap(),
            absolute
        );
    }

    /// Every tool and every parameter, nested ones included, has a catalog text in every
    /// language, and no English placeholder is left.
    #[test]
    fn every_tool_and_parameter_has_a_catalog_text() {
        fn check(value: &Value, path: &str, seen: &mut usize) {
            match value {
                Value::Object(map) => {
                    if let Some(Value::Object(properties)) = map.get("properties") {
                        for (name, property) in properties {
                            let description = property["description"].as_str().unwrap_or_default();
                            assert!(
                                description.len() > 8 && !description.starts_with("mcp-"),
                                "{path}.{name}: {description:?}"
                            );
                            *seen += 1;
                        }
                    }
                    map.iter()
                        .for_each(|(k, v)| check(v, &format!("{path}/{k}"), seen));
                }
                Value::Array(items) => items.iter().for_each(|v| check(v, path, seen)),
                _ => {}
            }
        }
        for lang in available() {
            let server = CoursePlanServer::new(lang.clone());
            let tools = server.tool_router.list_all();
            assert_eq!(tools.len(), 23);
            let mut seen = 0;
            for tool in &tools {
                let id = format!("mcp-tool-{}", tool.name.replace('_', "-"));
                let expected = text(&lang, &id, &[]);
                assert_ne!(expected, id, "{}: {id} is missing", lang.code());
                assert_eq!(tool.description.as_deref(), Some(expected.as_str()));
                let schema = Value::Object((*tool.input_schema).clone());
                check(
                    &schema,
                    &format!("{}/{}", lang.code(), tool.name),
                    &mut seen,
                );
            }
            assert!(seen > 60, "only {seen} parameters");
        }
    }
}
