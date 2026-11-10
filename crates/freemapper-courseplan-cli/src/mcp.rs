//! `freemapper-courseplan mcp`: the MCP server on stdio, so an AI assistant can set courses for a
//! user. The tools are thin wrappers over the library: each call loads the project, applies one
//! edit, saves on success and answers with compact JSON. Tool, parameter and field names stay
//! English; tool descriptions, parameter descriptions, the server instructions and the error
//! texts come from the catalog in the language chosen at start (ruling R1).

use std::path::PathBuf;
use std::sync::Arc;

use freemapper_courseplan::check::{
    ArgValue, Args, CheckReport, Finding, Location, check_event_as, source_text,
};
use freemapper_courseplan::course::{control_numbers, course_length_m};
use freemapper_courseplan::edit::{self, Accepted, LegStart};
use freemapper_courseplan::init::{InitError, init_project};
use freemapper_courseplan::map::{FeatureGeometry, MapIndex, map_kind_for_file};
use freemapper_courseplan::numbers::{self, MAX_GAP_MM};
use freemapper_courseplan::omap::OmapError;
use freemapper_courseplan::omap::model::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::rules::{
    PROPOSAL_DOC, RuleSet, SourceRef, load_rule_set, print_scale_of,
};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{
    AcceptedFinding, Class, Control, ControlCode, ControlDescription, ControlKind, Course,
    CourseKind, Event, LegRef, MapPoint, NumberPlacement, ProjectError, ProjectFile, SpecialObject,
};
use rmcp::handler::server::{router::tool::ToolRouter, wrapper::Parameters};
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData as McpError, ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use serde_json::{Map, Value, json};

use crate::edit_cmd;

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

/// The legs that lost bends and taped flag, as `[{course, after}]`; `after` counts from 1, and 0
/// is the leg from the start.
fn cleared_json(cleared: &[edit::ClearedLeg]) -> Value {
    Value::Array(
        cleared
            .iter()
            .map(|leg| json!({ "course": leg.course, "after": leg.after }))
            .collect(),
    )
}

fn round(x: f64, digits: i32) -> f64 {
    let f = 10f64.powi(digits);
    (x * f).round() / f
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

/// A location as a tagged object: `{"kind": "event"}`, `{"kind": "course", "course"}`,
/// `{"kind": "control", "code"}`, `{"kind": "controls", "codes"}`,
/// `{"kind": "leg", "course", "after", "from", "to"}` (after is the position the leg leaves,
/// from 1; 0 is the leg from the start; from and to are the ids of its ends) or
/// `{"kind": "legs", "course", "afters", "legs"}` for two or more legs of one course, `legs`
/// holding after, from and to of each.
fn location_json(location: &Location) -> Value {
    match location {
        Location::Event => json!({ "kind": "event" }),
        Location::Course(name) => json!({ "kind": "course", "course": name }),
        Location::Control { code } => json!({ "kind": "control", "code": code.0 }),
        Location::Controls { codes } => json!({
            "kind": "controls",
            "codes": codes.iter().map(|c| &c.0).collect::<Vec<_>>(),
        }),
        Location::Leg { course, leg } => json!({
            "kind": "leg",
            "course": course,
            "after": leg.after,
            "from": leg.from.id(),
            "to": leg.to.id(),
        }),
        Location::Legs { course, legs } => json!({
            "kind": "legs",
            "course": course,
            "afters": legs.iter().map(|l| l.after).collect::<Vec<_>>(),
            "legs": legs
                .iter()
                .map(|l| json!({ "after": l.after, "from": l.from.id(), "to": l.to.id() }))
                .collect::<Vec<_>>(),
        }),
    }
}

fn args_json(args: &Args) -> Value {
    let map: Map<String, Value> = args
        .iter()
        .map(|(name, value)| {
            let value = match value {
                ArgValue::Text(s) => json!(s),
                ArgValue::Number(x) | ArgValue::Decimals(x, _) => json!(round(*x, 3)),
                ArgValue::Int(n) => json!(n),
                ArgValue::List(id, items) => json!(
                    items
                        .iter()
                        .map(|args| json!({ "message_id": id, "args": args_json(args) }))
                        .collect::<Vec<_>>()
                ),
            };
            ((*name).to_owned(), value)
        })
        .collect();
    Value::Object(map)
}

/// The document and section a rule cites, with the document's title and link from its profile.
/// A value this program proposes (doc `claude`) gets the localized title "proposal of
/// freemapper-courseplan" (ruling R27); doc and section stay as machine fields.
fn source_json(rules: &RuleSet, profile: &str, source: &SourceRef, lang: &Lang) -> Value {
    let doc = rules.sources.get(profile).and_then(|s| s.get(&source.doc));
    let title = if source.doc == PROPOSAL_DOC {
        Some(source_text(lang, source))
    } else {
        doc.map(|d| d.title.clone())
    };
    json!({
        "doc": source.doc,
        "section": source.section,
        "title": title,
        "url": doc.and_then(|d| d.url.as_deref()),
    })
}

/// The legs of an accepted finding with their stored ends.
fn legs_json(legs: &[LegRef]) -> Value {
    json!(
        legs.iter()
            .map(|l| json!({ "course": l.course, "after": l.after, "from": l.from, "to": l.to }))
            .collect::<Vec<_>>()
    )
}

/// An accepted finding as stored: rule, course, controls, legs with their ends, note.
fn accepted_json(accepted: &AcceptedFinding) -> Value {
    json!({
        "rule": accepted.rule,
        "course": accepted.course,
        "controls": accepted.controls.iter().map(|c| &c.0).collect::<Vec<_>>(),
        "legs": legs_json(&accepted.legs),
        "note": accepted.note,
    })
}

fn finding_json(rules: &RuleSet, finding: &Finding, lang: &Lang) -> Value {
    let note = rules
        .rule(&finding.rule)
        .and_then(|r| r.rule.note.as_ref())
        .map(|n| n.get(&lang.code()).to_owned());
    json!({
        "rule": finding.rule,
        "profile": finding.profile,
        "severity": finding.severity.as_str(),
        "location": location_json(&finding.location),
        "message_id": finding.message_id,
        "args": args_json(&finding.args),
        "source": source_json(rules, &finding.profile, &finding.source, lang),
        "rule_note": note,
        "text": finding.text(lang),
    })
}

/// The result of `check`: summary, findings, accepted findings with their notes and skipped
/// rules, each with machine fields and the text in the server's language.
fn report_json(rules: &RuleSet, event: &Event, report: &CheckReport, lang: &Lang) -> Value {
    use freemapper_courseplan::rules::Severity;
    let accepted: Vec<Value> = report
        .accepted
        .iter()
        .map(|(finding, note)| {
            let mut value = finding_json(rules, finding, lang);
            value["note"] = json!(note);
            value
        })
        .collect();
    let unmatched: Vec<Value> = report
        .unmatched
        .iter()
        .map(|a| {
            let location = Location::of_accepted(event, a);
            let mut value = accepted_json(a);
            value["location"] = location_json(&location);
            value["text"] = json!(location.text_in(lang, event));
            value
        })
        .collect();
    let skipped: Vec<Value> = report
        .skipped
        .iter()
        .map(|s| {
            json!({
                "rule": s.rule,
                "profile": s.profile,
                "location": location_json(&s.location),
                "reason_id": s.reason_id,
                "args": args_json(&s.args),
                "text": s.text(lang),
            })
        })
        .collect();
    json!({
        "rule_profile": rules.base,
        "rule_overlays": rules.overlays,
        "summary": {
            "errors": report.count(Severity::Error),
            "warnings": report.count(Severity::Warning),
            "hints": report.count(Severity::Hint),
            "accepted": report.accepted.len(),
            "unmatched": report.unmatched.len(),
            "skipped": report.skipped.len(),
        },
        "findings": report
            .findings
            .iter()
            .map(|f| finding_json(rules, f, lang))
            .collect::<Vec<_>>(),
        "accepted": accepted,
        "unmatched": unmatched,
        "skipped": skipped,
    })
}

/// The accepted finding a tool call names.
fn named_finding(
    rule: Name,
    controls: Option<Vec<Code>>,
    legs: Option<Vec<LegArg>>,
    course: Option<Name>,
    note: String,
) -> AcceptedFinding {
    AcceptedFinding {
        rule: rule.0,
        controls: code_list(controls.unwrap_or_default())
            .into_iter()
            .map(ControlCode)
            .collect(),
        course: course.map(|c| c.0).filter(|c| !c.is_empty()),
        legs: legs
            .unwrap_or_default()
            .into_iter()
            .map(|l| LegRef::at(l.course.0, l.after))
            .collect(),
        note,
    }
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

/// Whether `path` ends in one of `extensions`, ignoring case. The check is lexical: a symlink
/// named `*.toml` is not resolved. The second layer is that errors never quote file content.
fn has_extension(path: &std::path::Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| extensions.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// A path the server opens: besides being absolute, it must carry one of `extensions`, so a
/// misled assistant cannot point the server at other files (keys, settings) and read them
/// through an error text (ruling R9).
fn resolve_kind(
    lang: &Lang,
    path: &str,
    extensions: &[&str],
    refusal: &str,
) -> Result<PathBuf, String> {
    let resolved = resolve(lang, path)?;
    if has_extension(&resolved, extensions) {
        Ok(resolved)
    } else {
        Err(text(
            lang,
            refusal,
            &[("path", FluentValue::from(path.trim().to_owned()))],
        ))
    }
}

fn resolve_project(lang: &Lang, path: &str) -> Result<PathBuf, String> {
    resolve_kind(lang, path, &["toml"], "mcp-error-not-toml")
}

fn resolve_xml(lang: &Lang, path: &str) -> Result<PathBuf, String> {
    resolve_kind(lang, path, &["xml"], "mcp-error-not-xml")
}

fn resolve_ppen(lang: &Lang, path: &str) -> Result<PathBuf, String> {
    resolve_kind(lang, path, &["ppen"], "mcp-error-not-ppen")
}

fn resolve_map(lang: &Lang, path: &str) -> Result<PathBuf, String> {
    resolve_kind(lang, path, &["omap", "xmap"], "mcp-error-not-map")
}

/// A project error as text for the assistant. Parser and schema errors can quote the file (a
/// source line, a value, a field name), so only the position stays, with a fixed reason from
/// the catalog. Read errors keep the OS text.
fn project_error(lang: &Lang, error: ProjectError) -> String {
    let (path, message) = match error {
        ProjectError::Parse { path, message } => (path, message),
        // The detail lists keys, codes and names from the file: only for a courseplan project.
        ProjectError::Invalid { ref path, .. } if !is_courseplan_project(path) => {
            let path = FluentValue::from(path.display().to_string());
            return text(lang, "mcp-error-project-parse-nopos", &[("path", path)]);
        }
        other => return other.localize(lang),
    };
    let path = FluentValue::from(path.display().to_string());
    match error_position(&message) {
        Some((line, column)) => text(
            lang,
            "mcp-error-project-parse",
            &[
                ("path", path),
                ("line", FluentValue::from(line)),
                ("column", FluentValue::from(column)),
            ],
        ),
        None => text(lang, "mcp-error-project-parse-nopos", &[("path", path)]),
    }
}

/// Whether the file has a top-level integer `format` and an `[event]` table.
fn is_courseplan_project(path: &std::path::Path) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| t.parse::<toml::Table>().ok())
        .is_some_and(|t| {
            t.get("format").is_some_and(toml::Value::is_integer)
                && t.get("event").is_some_and(toml::Value::is_table)
        })
}

/// An init error as text for the assistant, with the same redaction as the other tools.
fn init_error(lang: &Lang, error: InitError) -> String {
    match error {
        InitError::Map { path, source } => map_error(lang, &path, source),
        InitError::Project(e) => project_error(lang, e),
        other => other.localize(lang),
    }
}

/// "line N, column M" of a toml_edit error text.
fn error_position(message: &str) -> Option<(u64, u64)> {
    let rest = &message[message.find("line ")? + 5..];
    let (line, rest) = rest.split_once(", column ")?;
    let column: String = rest.chars().take_while(char::is_ascii_digit).collect();
    Some((line.parse().ok()?, column.parse().ok()?))
}

/// A map error as text for the assistant: read errors keep the OS text, format errors get a
/// fixed reason, because XML errors can quote the file.
fn map_error(lang: &Lang, path: &std::path::Path, error: OmapError) -> String {
    /// The fixed refusal text of `omap/read.rs` for the old binary format. It reveals only that
    /// the file starts with `OMAP`.
    const BINARY_FORMAT_MESSAGE: &str = "This file is in the old binary format of OpenOrienteering Mapper, which this program cannot read. Open it in Mapper and save it again as a current .omap or .xmap file.";
    let source = match &error {
        OmapError::Io(_) | OmapError::MissingElement(_) => error.localize(lang),
        OmapError::Unsupported(m) if m == BINARY_FORMAT_MESSAGE => error.localize(lang),
        _ => text(
            lang,
            "mcp-error-map-invalid",
            &[("path", FluentValue::from(path.display().to_string()))],
        ),
    };
    text(
        lang,
        "error-init-map",
        &[
            ("path", FluentValue::from(path.display().to_string())),
            ("source", FluentValue::from(source)),
        ],
    )
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
        let path = resolve_project(&self.lang, project)?;
        ProjectFile::load(&path).map_err(|e| project_error(&self.lang, e))
    }

    /// The project's map, if it is an OOM file.
    fn read_map(&self, file: &ProjectFile) -> Result<MapDocument, String> {
        let path = file.map_path();
        if !has_extension(&path, &["omap", "xmap"]) {
            return Err(text(
                &self.lang,
                "mcp-error-not-map",
                &[("path", FluentValue::from(path.display().to_string()))],
            ));
        }
        read_map_file(&path).map_err(|e| map_error(&self.lang, &path, e))
    }

    /// Loads the project, applies `change`, and saves when it succeeded.
    ///
    /// Single flight: the handlers contain no `.await` and the server runs on a current-thread
    /// runtime, so one call finishes before the next starts and load, apply and save are atomic
    /// within this process. Two processes editing the same file can still race.
    fn edit<T>(
        &self,
        project: &str,
        change: impl FnOnce(&mut ProjectFile) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut file = self.load(project)?;
        let out = change(&mut file)?;
        file.save().map_err(|e| project_error(&self.lang, e))?;
        Ok(out)
    }

    fn refused(&self, e: edit::EditError) -> String {
        e.localize(&self.lang)
    }

    /// The rule set the event names.
    fn rule_set(&self, event: &Event) -> Result<RuleSet, String> {
        load_rule_set(&event.event.rule_profile, &event.event.rule_overlays)
            .map_err(|e| e.localize(&self.lang))
    }

    /// A number placement from `direction` and `gap_mm`, or none with `clear`. The direction is
    /// brought into [0, 360); the gap must be more than 0 and at most [`MAX_GAP_MM`] print mm.
    fn number_arg(
        &self,
        direction: Option<f64>,
        gap_mm: Option<f64>,
        clear: Option<bool>,
    ) -> Result<Option<NumberPlacement>, String> {
        match (direction, gap_mm, flag(clear)) {
            (Some(direction), Some(gap_mm), false) => {
                if !numbers::gap_in_range(gap_mm) {
                    return Err(text(
                        &self.lang,
                        "mcp-error-bad-gap",
                        &[
                            ("value", FluentValue::from(gap_mm.to_string())),
                            ("max", FluentValue::from(MAX_GAP_MM.to_string())),
                        ],
                    ));
                }
                Ok(Some(NumberPlacement {
                    direction: numbers::normalize_direction(direction),
                    gap_mm,
                }))
            }
            (None, None, true) => Ok(None),
            _ => Err(text(&self.lang, "mcp-error-number-args", &[])),
        }
    }

    fn extent(&self, file: &ProjectFile) -> Result<Option<(MapPoint, MapPoint)>, String> {
        Ok(MapIndex::from_document(&self.read_map(file)?).extent())
    }

    #[tool(description = "Summary of the event")]
    async fn event_info(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let doc = self.read_map(&file)?;
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
                "map_kind": map_kind_for_file(&doc, &file.map_path()).as_str(),
                "scale": scale,
                "rule_profile": event.event.rule_profile,
                "rule_overlays": event.event.rule_overlays,
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
            let doc = self.read_map(&file)?;
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
                req.code.map(|c| ControlCode(c.0)),
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
                &ControlCode(req.code.0.clone()),
                req.at.map_point(),
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "code": req.code.0 }))
        }))
    }

    #[tool(description = "Remove a control")]
    async fn remove_control(
        &self,
        Parameters(req): Parameters<ControlRef>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let removed = edit::remove_control(&mut file.event, &ControlCode(req.code.0.clone()))
                .map_err(|e| self.refused(e))?;
            Ok(json!({
                "code": req.code.0,
                "left_courses": removed.courses,
                "cleared_legs": cleared_json(&removed.cleared),
            }))
        }))
    }

    #[tool(description = "Describe a control")]
    async fn describe_control(
        &self,
        Parameters(req): Parameters<DescribeControl>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let key = ControlCode(req.code.0.clone());
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
                "code": req.code.0,
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
            let number = self.number_arg(req.direction, req.gap_mm, req.clear)?;
            edit::set_central_number(
                &mut file.event,
                &ControlCode(req.code.0.clone()),
                number.clone(),
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "code": req.code.0, "number": placement(&number) }))
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
            let courses = name_list(req.courses.unwrap_or_default());
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
            let id = req.id.0.clone();
            let at = || {
                req.at
                    .map(Point::map_point)
                    .unwrap_or(MapPoint { x: 0.0, y: 0.0 })
            };
            let points = map_points(points);
            let special = match kind {
                SpecialKindArg::Start => SpecialObject::Start {
                    id,
                    ppen_id: None,
                    map_mm: at(),
                },
                SpecialKindArg::Finish => SpecialObject::Finish {
                    id,
                    ppen_id: None,
                    map_mm: at(),
                },
                SpecialKindArg::FirstAid => SpecialObject::FirstAid {
                    id,
                    ppen_id: None,
                    map_mm: at(),
                    courses,
                },
                SpecialKindArg::Water => SpecialObject::Water {
                    id,
                    ppen_id: None,
                    map_mm: at(),
                    courses,
                },
                SpecialKindArg::MarkedRoute => SpecialObject::MarkedRoute {
                    id,
                    ppen_id: None,
                    points,
                    courses,
                },
                SpecialKindArg::ForbiddenRoute => SpecialObject::ForbiddenRoute {
                    id,
                    ppen_id: None,
                    points,
                    courses,
                },
                SpecialKindArg::OutOfBounds => SpecialObject::OutOfBoundsArea {
                    id,
                    ppen_id: None,
                    polygon: points,
                    courses,
                },
                SpecialKindArg::DangerousArea => SpecialObject::DangerousArea {
                    id,
                    ppen_id: None,
                    polygon: points,
                    courses,
                },
            };
            let extent = self.extent(file)?;
            edit::add_special(&mut file.event, extent, special).map_err(|e| self.refused(e))?;
            Ok(json!({ "id": req.id.0 }))
        }))
    }

    #[tool(description = "Remove a special object")]
    async fn remove_special(
        &self,
        Parameters(req): Parameters<RemoveSpecial>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::remove_special(&mut file.event, &req.id.0).map_err(|e| self.refused(e))?;
            Ok(json!({ "id": req.id.0 }))
        }))
    }

    #[tool(description = "Set the courses of a special object")]
    async fn set_special_courses(
        &self,
        Parameters(req): Parameters<SetSpecialCourses>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let courses = match (req.courses, flag(req.all)) {
                (Some(list), false) => {
                    let names = name_list(list);
                    if names.is_empty() {
                        return Err(text(&self.lang, "mcp-error-special-courses-args", &[]));
                    }
                    names
                }
                (None, true) => Vec::new(),
                _ => return Err(text(&self.lang, "mcp-error-special-courses-args", &[])),
            };
            edit::set_special_courses(&mut file.event, &req.id.0, courses.clone())
                .map_err(|e| self.refused(e))?;
            Ok(json!({ "id": req.id.0, "courses": courses }))
        }))
    }

    #[tool(description = "Add a class")]
    async fn add_class(
        &self,
        Parameters(req): Parameters<AddClass>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let class = Class {
                name: req.name.0.clone(),
                rule_category: req.rule_category,
                expected_starters: req.expected_starters,
            };
            edit::add_class(&mut file.event, class).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name.0 }))
        }))
    }

    #[tool(description = "Change the rule category or expected starters of a class")]
    async fn set_class(
        &self,
        Parameters(req): Parameters<SetClass>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let conflict = |arg: &str, other: &str| {
                text(
                    &self.lang,
                    "cli-parse-argument-conflict",
                    &[
                        ("arg", FluentValue::from(arg.to_owned())),
                        ("other", FluentValue::from(other.to_owned())),
                    ],
                )
            };
            if req.rule_category.is_some() && flag(req.no_rule_category) {
                return Err(conflict("rule_category", "no_rule_category"));
            }
            if req.expected_starters.is_some() && flag(req.no_expected_starters) {
                return Err(conflict("expected_starters", "no_expected_starters"));
            }
            let category = match (req.rule_category, flag(req.no_rule_category)) {
                (Some(c), _) => Some(Some(c)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            let starters = match (req.expected_starters, flag(req.no_expected_starters)) {
                (Some(n), _) => Some(Some(n)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            if category.is_none() && starters.is_none() {
                return Err(text(&self.lang, "mcp-error-no-class-change", &[]));
            }
            edit::set_class(&mut file.event, &req.name.0, category, starters)
                .map_err(|e| self.refused(e))?;
            let class = file
                .event
                .classes
                .iter()
                .find(|c| c.name == req.name.0)
                .expect("just changed");
            Ok(json!({
                "name": class.name,
                "rule_category": class.rule_category,
                "expected_starters": class.expected_starters,
            }))
        }))
    }

    #[tool(description = "Remove a class")]
    async fn remove_class(
        &self,
        Parameters(req): Parameters<RemoveClass>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::remove_class(&mut file.event, &req.name.0).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name.0 }))
        }))
    }

    #[tool(description = "Add an individual course")]
    async fn add_course(
        &self,
        Parameters(req): Parameters<AddCourse>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let controls = code_list(req.controls.unwrap_or_default())
                .into_iter()
                .map(edit_cmd::course_control)
                .collect();
            let course = Course {
                ppen_id: None,
                name: req.name.0.clone(),
                classes: name_list(req.classes.unwrap_or_default()),
                start: req.start.0,
                finish: req.finish.0,
                start_bends: vec![],
                start_taped: false,
                print_area: None,
                climb_m: req.climb_m,
                planned_print_scale: req.print_scale,
                kind: CourseKind::Individual { controls },
            };
            edit::add_course(&mut file.event, course).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name.0 }))
        }))
    }

    #[tool(description = "Remove a course")]
    async fn remove_course(
        &self,
        Parameters(req): Parameters<CourseRef>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::remove_course(&mut file.event, &req.name.0).map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name.0 }))
        }))
    }

    #[tool(description = "Replace the controls of a course")]
    async fn set_course_controls(
        &self,
        Parameters(req): Parameters<SetCourseControls>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let codes = code_list(req.codes);
            let sequence: Vec<ControlCode> = codes.iter().cloned().map(ControlCode).collect();
            let cleared = edit::set_course_controls(&mut file.event, &req.name.0, &sequence)
                .map_err(|e| self.refused(e))?;
            Ok(json!({
                "name": req.name.0,
                "controls": codes,
                "cleared_legs": cleared_json(&cleared),
            }))
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
                    course: req.name.0.clone(),
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
            edit::set_leg(&mut file.event, &req.name.0, leg, bends, taped).map_err(
                |e| match e {
                    edit::EditError::BadIndex { .. } => bad_index(req.after.unwrap_or(0)),
                    other => self.refused(other),
                },
            )?;
            Ok(json!({ "name": req.name.0, "bends": count, "taped": taped }))
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
            let classes = req.classes.map(name_list);
            if classes.is_none() && climb.is_none() && scale.is_none() {
                return Err(text(&self.lang, "mcp-error-no-change", &[]));
            }
            edit::set_course_details(&mut file.event, &req.name.0, classes, climb, scale)
                .map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name.0 }))
        }))
    }

    #[tool(description = "Place the control numbers of a course")]
    async fn place_numbers(
        &self,
        Parameters(req): Parameters<PlaceNumbers>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let doc = self.read_map(file)?;
            // The course must exist first; an unknown name keeps the not-found error of the edit.
            let gaps = match file.event.courses.iter().find(|c| c.name == req.name.0) {
                Some(course) => {
                    let rules = self.rule_set(&file.event)?;
                    numbers::gap_range(
                        &rules,
                        map_kind_for_file(&doc, &file.map_path()),
                        file.event.event.discipline,
                        print_scale_of(course, doc.georef.scale_denominator),
                    )
                }
                None => numbers::GapRange::DEFAULT,
            };
            let kind = map_kind_for_file(&doc, &file.map_path());
            let placed = numbers::place_numbers(
                &mut file.event,
                &req.name.0,
                &MapIndex::from_document(&doc),
                doc.georef.scale_denominator,
                kind,
                gaps,
                flag(req.overwrite),
            )
            .map_err(|e| self.refused(e))?;
            Ok(json!({ "name": req.name.0, "placed": placed }))
        }))
    }

    #[tool(description = "Set the number placement of a control in a course")]
    async fn set_course_number(
        &self,
        Parameters(req): Parameters<SetCourseNumber>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let number = self.number_arg(req.direction, req.gap_mm, req.clear)?;
            // The assistant counts from 1 as on the map; the library counts from 0.
            let bad_index = || {
                self.refused(edit::EditError::BadIndex {
                    course: req.name.0.clone(),
                    index: req.after as usize,
                })
            };
            if req.after == 0 {
                return Err(bad_index());
            }
            edit::set_course_number(
                &mut file.event,
                &req.name.0,
                req.after as usize - 1,
                number.clone(),
            )
            .map_err(|e| match e {
                edit::EditError::BadIndex { .. } => bad_index(),
                other => self.refused(other),
            })?;
            Ok(json!({ "name": req.name.0, "after": req.after, "number": placement(&number) }))
        }))
    }

    #[tool(description = "Map objects near a position")]
    async fn map_features_near(
        &self,
        Parameters(req): Parameters<FeaturesNear>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let doc = self.read_map(&file)?;
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
            let doc = self.read_map(&file)?;
            Ok(json!({
                "extent": extent_json(MapIndex::from_document(&doc).extent()),
                "scale": doc.georef.scale_denominator,
                "map_kind": map_kind_for_file(&doc, &file.map_path()).as_str(),
            }))
        })())
    }

    #[tool(description = "Check the courses")]
    async fn check(
        &self,
        Parameters(req): Parameters<ProjectOnly>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let file = self.load(&req.project)?;
            let rules = self.rule_set(&file.event)?;
            let doc = self.read_map(&file)?;
            let report = check_event_as(
                &file.event,
                &doc,
                map_kind_for_file(&doc, &file.map_path()),
                &rules,
            );
            Ok(report_json(&rules, &file.event, &report, &self.lang))
        })())
    }

    #[tool(description = "Export the courses as IOF XML 3.0")]
    async fn export_iof_xml(
        &self,
        Parameters(req): Parameters<ExportIofXml>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let project = resolve_project(&self.lang, &req.project)?;
            let target = resolve_xml(&self.lang, &req.path)?;
            let report = crate::export_cmd::export_iof_xml_file(
                &project,
                &target,
                &crate::export_cmd::now_iso(),
            )
            .map_err(|e| {
                e.message(
                    &self.lang,
                    |e| project_error(&self.lang, e),
                    |p, e| map_error(&self.lang, p, e),
                )
            })?;
            let skipped: Vec<Value> = report
                .skipped
                .iter()
                .map(|s| json!({ "name": s.name, "kind": s.kind }))
                .collect();
            Ok(json!({
                "path": target.display().to_string(),
                "courses": report.courses,
                "skipped": skipped,
            }))
        })())
    }

    #[tool(description = "Export the courses to a Purple Pen event")]
    async fn export_ppen(
        &self,
        Parameters(req): Parameters<ExportPpen>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let project = resolve_project(&self.lang, &req.project)?;
            let target = resolve_ppen(&self.lang, &req.path)?;
            let report =
                crate::export_cmd::export_ppen_file(&project, &target, flag(req.new), &self.lang)
                    .map_err(|e| {
                    e.message(
                        &self.lang,
                        |e| project_error(&self.lang, e),
                        |p, e| map_error(&self.lang, p, e),
                    )
                })?;
            let skipped: Vec<Value> = report
                .skipped()
                .into_iter()
                .map(|(name, kind)| json!({ "name": name, "kind": kind }))
                .collect();
            let notes: Vec<String> = report
                .notes
                .iter()
                .map(|n| n.localize(&self.lang))
                .collect();
            Ok(json!({
                "path": target.display().to_string(),
                "courses": report.courses,
                "skipped": skipped,
                "notes": notes,
                "patched": report.patched,
                "ids_stored": report.ids_stored,
            }))
        })())
    }

    #[tool(description = "Import a Purple Pen event")]
    async fn import_ppen(
        &self,
        Parameters(req): Parameters<ImportPpen>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let project = resolve_project(&self.lang, &req.project)?;
            let ppen = resolve_ppen(&self.lang, &req.path)?;
            let report = crate::import_cmd::import_ppen_file(&project, &ppen, flag(req.replace))
                .map_err(|e| {
                    e.message(
                        &self.lang,
                        |e| project_error(&self.lang, e),
                        |p, e| map_error(&self.lang, p, e),
                    )
                })?;
            let notes: Vec<String> = report
                .notes
                .iter()
                .map(|n| n.localize(&self.lang))
                .collect();
            Ok(json!({
                "controls": report.controls,
                "courses": report.courses,
                "notes": notes,
            }))
        })())
    }

    #[tool(description = "Accept a finding")]
    async fn accept_finding(
        &self,
        Parameters(req): Parameters<AcceptFinding>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let rules = self.rule_set(&file.event)?;
            let finding = named_finding(req.rule, req.controls, req.legs, req.course, req.note);
            // The stored entry carries the leg ends `accept_finding` took from the course.
            let (done, stored) = edit::accept_finding(&mut file.event, &rules, finding)
                .map_err(|e| self.refused(e))?;
            let mut value = accepted_json(&stored);
            value["replaced"] = json!(done == Accepted::NoteReplaced);
            Ok(value)
        }))
    }

    #[tool(description = "Remove an accepted finding")]
    async fn remove_accepted_finding(
        &self,
        Parameters(req): Parameters<RemoveAcceptedFinding>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let finding =
                named_finding(req.rule, req.controls, req.legs, req.course, String::new());
            let removed = edit::remove_accepted_finding(&mut file.event, &finding.key())
                .map_err(|e| self.refused(e))?;
            Ok(json!({
                "rule": finding.rule,
                "removed": removed.iter().map(accepted_json).collect::<Vec<_>>(),
            }))
        }))
    }

    #[tool(description = "Set the rule profile")]
    async fn set_rule_profile(
        &self,
        Parameters(req): Parameters<SetRuleProfile>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            let overlays = name_list(req.overlays.unwrap_or_default());
            let set = edit::set_rule_profile(&mut file.event, &req.base.0, &overlays)
                .map_err(|e| self.refused(e))?;
            Ok(json!({
                "rule_profile": set.base,
                "rule_overlays": set.overlays,
                "rules": set.rules.len(),
            }))
        }))
    }

    #[tool(description = "Set the start window")]
    async fn set_start_window(
        &self,
        Parameters(req): Parameters<SetStartWindow>,
    ) -> Result<CallToolResult, McpError> {
        reply(self.edit(&req.project, |file| {
            edit::set_start_window(&mut file.event, req.minutes).map_err(|e| self.refused(e))?;
            Ok(json!({ "start_window_min": req.minutes }))
        }))
    }

    #[tool(description = "Start an event from an OOM map")]
    async fn init_event(
        &self,
        Parameters(req): Parameters<InitEvent>,
    ) -> Result<CallToolResult, McpError> {
        reply((|| {
            let map = resolve_map(&self.lang, &req.map)?;
            let project = match &req.project {
                Some(p) => resolve_project(&self.lang, p)?,
                None => crate::default_project_path(&map),
            };
            let created = init_project(&map, &project, req.title.clone())
                .map_err(|e| init_error(&self.lang, e))?;
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

/// Serves MCP on stdin and stdout until the client disconnects. The current-thread runtime is
/// part of the single-flight rule described at `CoursePlanServer::edit`.
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
    use freemapper_courseplan::check::{LegEnd, LegSpan};
    use freemapper_courseplan::text::available;

    #[test]
    fn a_finding_at_two_legs_names_both_positions() {
        let span = |after, from: LegEnd, to: &str| LegSpan {
            after,
            from,
            to: LegEnd::Control(to.to_owned()),
        };
        let legs = Location::Legs {
            course: "A".to_owned(),
            legs: vec![
                span(0, LegEnd::Start("S1".to_owned()), "31"),
                span(4, LegEnd::Control("34".to_owned()), "35"),
            ],
        };
        assert_eq!(
            location_json(&legs),
            json!({
                "kind": "legs",
                "course": "A",
                "afters": [0, 4],
                "legs": [
                    { "after": 0, "from": "S1", "to": "31" },
                    { "after": 4, "from": "34", "to": "35" },
                ],
            })
        );
    }

    #[test]
    fn check_json_has_machine_fields_and_localized_texts() {
        use freemapper_courseplan::check::{SKIP_CLIMB_MISSING, SKIP_PRINT_SCALE, Skipped};
        use freemapper_courseplan::rules::Severity;
        let rules = load_rule_set("wkb-2026", &[]).unwrap();
        let finding = Finding {
            rule: "wkb2026.distance.min".to_owned(),
            profile: "wkb-2026".to_owned(),
            severity: Severity::Error,
            location: Location::Leg {
                course: "A".to_owned(),
                leg: LegSpan {
                    after: 3,
                    from: LegEnd::Control("33".to_owned()),
                    to: LegEnd::Control("34".to_owned()),
                },
            },
            message_id: SKIP_PRINT_SCALE,
            args: vec![
                ("scale", ArgValue::Int(2500)),
                ("metres", ArgValue::Number(12.3456)),
                ("name", ArgValue::Text("H21".to_owned())),
            ],
            source: SourceRef {
                doc: "WKB26".to_owned(),
                section: "B 7.2.1".to_owned(),
            },
        };
        let accepted = Finding {
            location: Location::Controls {
                codes: vec![ControlCode("31".to_owned()), ControlCode("32".to_owned())],
            },
            ..finding.clone()
        };
        let report = CheckReport {
            findings: vec![finding],
            accepted: vec![(accepted, "on purpose".to_owned())],
            unmatched: vec![AcceptedFinding {
                rule: "wkb2026.leg.acute_angle".to_owned(),
                controls: vec![],
                course: None,
                legs: vec![LegRef {
                    course: "Z".to_owned(),
                    after: 1,
                    from: "31".to_owned(),
                    to: "32".to_owned(),
                }],
                note: "gone".to_owned(),
            }],
            skipped: vec![Skipped {
                rule: Some("wkb2026.code.range".to_owned()),
                profile: Some("wkb-2026".to_owned()),
                location: Location::Event,
                reason_id: SKIP_CLIMB_MISSING,
                args: vec![],
            }],
        };
        let de = available().into_iter().find(|l| l.code() == "de").unwrap();
        let event: Event =
            toml::from_str("format = 1\n[event]\ntitle = \"T\"\n[sources]\nmap = \"m.omap\"\n")
                .unwrap();
        let json = report_json(&rules, &event, &report, &de);
        assert_eq!(json["rule_profile"], "wkb-2026");
        assert_eq!(json["rule_overlays"], json!([]));
        assert_eq!(
            json["summary"],
            json!({
                "errors": 1,
                "warnings": 0,
                "hints": 0,
                "accepted": 1,
                "unmatched": 1,
                "skipped": 1
            })
        );
        let f = &json["findings"][0];
        assert_eq!(f["rule"], "wkb2026.distance.min");
        assert_eq!(f["profile"], "wkb-2026");
        assert_eq!(f["severity"], "error");
        assert_eq!(
            f["location"],
            json!({ "kind": "leg", "course": "A", "after": 3, "from": "33", "to": "34" })
        );
        assert_eq!(f["message_id"], "check-skip-print-scale");
        assert_eq!(
            f["args"],
            json!({ "scale": 2500, "metres": 12.346, "name": "H21" })
        );
        assert_eq!(f["source"]["doc"], "WKB26");
        assert_eq!(f["source"]["section"], "B 7.2.1");
        assert!(
            f["source"]["title"].as_str().unwrap().contains("Wettkampf"),
            "{f}"
        );
        assert_eq!(
            f["text"],
            "Keine Prüfregel dieser Art für den Druckmaßstab 1:2500"
        );
        let a = &json["accepted"][0];
        assert_eq!(
            a["location"],
            json!({ "kind": "controls", "codes": ["31", "32"] })
        );
        assert_eq!(a["note"], "on purpose");
        let s = &json["skipped"][0];
        assert_eq!(s["rule"], "wkb2026.code.range");
        assert_eq!(s["location"], json!({ "kind": "event" }));
        assert_eq!(s["reason_id"], "check-skip-climb-missing");
        assert_eq!(s["text"], "Für die Bahn ist kein Anstieg angegeben");
        let u = &json["unmatched"][0];
        assert_eq!(u["rule"], "wkb2026.leg.acute_angle");
        assert_eq!(
            u["legs"],
            json!([{ "course": "Z", "after": 1, "from": "31", "to": "32" }])
        );
        assert_eq!(u["note"], "gone");
        assert_eq!(u["text"], "Bahn Z, Strecke 31–32");
    }

    #[test]
    fn a_proposed_value_gets_a_localized_source_title() {
        let rules = load_rule_set("wkb-2026", &[]).unwrap();
        let de = available().into_iter().find(|l| l.code() == "de").unwrap();
        let source = SourceRef {
            doc: "claude".to_owned(),
            section: "M2b plan, Task 5".to_owned(),
        };
        let json = source_json(&rules, "wkb-2026", &source, &de);
        assert_eq!(json["doc"], "claude");
        assert_eq!(json["section"], "M2b plan, Task 5");
        assert_eq!(json["title"], "Vorschlag von freemapper-courseplan");
    }

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
            assert_eq!(tools.len(), 34);
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
