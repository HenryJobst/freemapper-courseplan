//! `list controls|courses|specials|classes` and `map extent|near`: the read-only MCP tools on the
//! command line (ADR 0059). Each command runs the tool's operation through `CoursePlanServer::call_json`,
//! so the answer is the one the MCP tool gives. `--json` prints it unchanged (with `"ok": true`, as the
//! MCP result has it); without it the same value becomes a text table.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use freemapper_courseplan::text::{FluentValue, Lang, text};
use serde_json::{Value, json};

use crate::edit_cmd::{Failure, load, parse_count, parse_point};
use crate::mcp::{CoursePlanServer, OpError};

type Outcome = Result<(), Failure>;

// The help texts come from the catalog (see `edit_cmd::localize_groups`), not from doc comments.
#[derive(Subcommand)]
pub enum ListCmd {
    /// List the controls
    Controls {
        project: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// List the courses
    Courses {
        project: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// List the special objects
    Specials {
        project: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// List the classes
    Classes {
        project: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum MapCmd {
    /// Show the extent, scale and kind of the map
    Extent {
        project: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// List the map objects near a point
    Near {
        project: PathBuf,
        #[arg(long, value_name = "X,Y", allow_hyphen_values = true)]
        at: String,
        #[arg(long, value_name = "MM", allow_hyphen_values = true)]
        radius: String,
        #[arg(long, value_name = "N")]
        limit: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

pub fn list(lang: &Lang, cmd: ListCmd) -> Outcome {
    let (project, json, tool) = match cmd {
        ListCmd::Controls { project, json } => (project, json, "list_controls"),
        ListCmd::Courses { project, json } => (project, json, "list_courses"),
        ListCmd::Specials { project, json } => (project, json, "list_specials"),
        ListCmd::Classes { project, json } => (project, json, "list_classes"),
    };
    let value = run_tool(lang, &project, tool, json!({}))?;
    if json {
        return print_json(&value);
    }
    let (headers, rows) = match tool {
        "list_controls" => controls_table(lang, &value),
        "list_courses" => courses_table(lang, &value),
        "list_specials" => specials_table(lang, &value),
        _ => classes_table(lang, &value),
    };
    print_table(lang, &headers, rows);
    Ok(())
}

pub fn map(lang: &Lang, cmd: MapCmd) -> Outcome {
    match cmd {
        MapCmd::Extent { project, json } => {
            let value = run_tool(lang, &project, "map_extent", json!({}))?;
            if json {
                return print_json(&value);
            }
            print_extent(lang, &value);
        }
        MapCmd::Near {
            project,
            at,
            radius,
            limit,
            json,
        } => {
            let at = parse_point(lang, &at)?;
            let radius = match radius.parse::<f64>() {
                Ok(r) if r.is_finite() && r > 0.0 => r,
                _ => {
                    return Err(Failure::Usage(text(
                        lang,
                        "cli-map-bad-radius",
                        &[("value", FluentValue::from(radius))],
                    )));
                }
            };
            let mut args = json!({ "x": at.x, "y": at.y, "radius_mm": radius });
            if let Some(limit) = limit {
                args["limit"] = json!(parse_count(lang, &limit)?);
            }
            let value = run_tool(lang, &project, "map_features_near", args)?;
            if json {
                return print_json(&value);
            }
            print_near(lang, &value);
        }
    }
    Ok(())
}

/// Runs one MCP tool on the project and returns its JSON, with `"ok": true` as the MCP result has it.
/// The project is loaded once first, so a missing or broken file is reported as every other command
/// reports it; the MCP tool's own message is redacted for the assistant.
pub(crate) fn run_tool(
    lang: &Lang,
    project: &Path,
    tool: &str,
    args: Value,
) -> Result<Value, Failure> {
    load(lang, project)?;
    let absolute = std::path::absolute(project)
        .map_err(|e| Failure::Error(format!("{}: {e}", project.display())))?;
    let server = CoursePlanServer::new(lang.clone());
    match server.call_json(&absolute, tool, args) {
        Ok(Value::Object(mut map)) => {
            map.insert("ok".to_owned(), Value::Bool(true));
            Ok(Value::Object(map))
        }
        Ok(other) => Ok(other),
        Err(OpError::Refused(message)) | Err(OpError::BadArgs(message)) => {
            Err(Failure::Error(message))
        }
        Err(OpError::Io) => Err(Failure::Error(text(lang, "cli-query-io-failed", &[]))),
        Err(OpError::UnknownTool) => Err(Failure::Error(tool.to_owned())),
    }
}

pub(crate) fn print_json(value: &Value) -> Outcome {
    println!(
        "{}",
        serde_json::to_string_pretty(value).unwrap_or_default()
    );
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Text

fn s(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn point(value: &Value) -> String {
    format!("{},{}", s(&value["x"]), s(&value["y"]))
}

fn list_of(value: &Value) -> String {
    value
        .as_array()
        .map(|items| items.iter().map(s).collect::<Vec<_>>().join(", "))
        .unwrap_or_default()
}

fn items<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map_or(&[], Vec::as_slice)
}

fn word(lang: &Lang, id: &str) -> String {
    text(lang, id, &[])
}

type Table = (Vec<String>, Vec<Vec<String>>);

fn headers(lang: &Lang, ids: &[&str]) -> Vec<String> {
    ids.iter()
        .map(|id| word(lang, &format!("cli-col-{id}")))
        .collect()
}

fn controls_table(lang: &Lang, value: &Value) -> Table {
    let rows = items(value, "controls")
        .iter()
        .map(|c| {
            let kind = match c["kind"].as_str() {
                Some("crossing_point") => "word-sync-kind-crossing-point",
                _ => "word-sync-kind-control",
            };
            let number = match &c["number"] {
                Value::Null => String::new(),
                n => format!("{} / {}", s(&n["direction"]), s(&n["gap_mm"])),
            };
            vec![s(&c["code"]), word(lang, kind), point(&c["map_mm"]), number]
        })
        .collect();
    (headers(lang, &["code", "kind", "position", "number"]), rows)
}

fn courses_table(lang: &Lang, value: &Value) -> Table {
    let rows = items(value, "courses")
        .iter()
        .map(|c| {
            let kind = word(lang, &format!("word-kind-{}", s(&c["kind"])));
            let km = match &c["length_km"] {
                Value::Null => String::new(),
                km => s(km),
            };
            vec![
                s(&c["name"]),
                kind,
                s(&c["start"]),
                s(&c["finish"]),
                s(&c["control_count"]),
                km,
                s(&c["climb_m"]),
                list_of(&c["classes"]),
            ]
        })
        .collect();
    let ids = [
        "name", "kind", "start", "finish", "controls", "length", "climb", "classes",
    ];
    (headers(lang, &ids), rows)
}

fn specials_table(lang: &Lang, value: &Value) -> Table {
    let rows = items(value, "specials")
        .iter()
        .map(|sp| {
            let kind = s(&sp["kind"]).replace('_', "-");
            let place = if sp["at"].is_object() {
                point(&sp["at"])
            } else {
                text(
                    lang,
                    "cli-list-points",
                    &[("count", FluentValue::from(items(sp, "points").len()))],
                )
            };
            vec![
                s(&sp["id"]),
                word(lang, &format!("word-special-{kind}")),
                place,
                list_of(&sp["courses"]),
            ]
        })
        .collect();
    (headers(lang, &["id", "kind", "position", "courses"]), rows)
}

fn classes_table(lang: &Lang, value: &Value) -> Table {
    let rows = items(value, "classes")
        .iter()
        .map(|c| {
            vec![
                s(&c["name"]),
                s(&c["rule_category"]),
                s(&c["expected_starters"]),
            ]
        })
        .collect();
    (headers(lang, &["name", "category", "starters"]), rows)
}

/// Columns padded to their widest cell; the last column is not padded.
fn print_table(lang: &Lang, header: &[String], rows: Vec<Vec<String>>) {
    if rows.is_empty() {
        println!("{}", word(lang, "cli-list-empty"));
        return;
    }
    let mut widths: Vec<usize> = header.iter().map(|h| h.chars().count()).collect();
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }
    let line = |cells: &[String]| {
        let mut out = String::new();
        for (i, cell) in cells.iter().enumerate() {
            if i + 1 == cells.len() {
                out.push_str(cell);
            } else {
                let pad = widths[i] - cell.chars().count();
                out.push_str(cell);
                out.push_str(&" ".repeat(pad + 2));
            }
        }
        out.trim_end().to_owned()
    };
    println!("{}", line(header));
    for row in &rows {
        println!("{}", line(row));
    }
}

fn print_extent(lang: &Lang, value: &Value) {
    let kind = match value["map_kind"].as_str() {
        Some("sprint") => "word-map-sprint",
        Some("schoolyard") => "word-map-schoolyard",
        _ => "word-map-forest",
    };
    let extent = &value["extent"];
    if extent.is_null() {
        println!("{}", word(lang, "cli-map-extent-none"));
    } else {
        println!(
            "{}",
            text(
                lang,
                "cli-map-extent",
                &[
                    ("min", FluentValue::from(point(&extent["min"]))),
                    ("max", FluentValue::from(point(&extent["max"]))),
                ],
            )
        );
    }
    println!(
        "{}",
        text(
            lang,
            "cli-map-scale",
            &[("scale", FluentValue::from(s(&value["scale"])))]
        )
    );
    println!(
        "{}",
        text(
            lang,
            "cli-info-map-kind",
            &[("value", FluentValue::from(word(lang, kind)))]
        )
    );
}

fn print_near(lang: &Lang, value: &Value) {
    let features = items(value, "features");
    let rows: Vec<Vec<String>> = features
        .iter()
        .map(|f| {
            vec![
                s(&f["distance_mm"]),
                s(&f["symbol_code"]),
                word(lang, &format!("cli-map-kind-{}", s(&f["kind"]))),
                s(&f["part"]),
            ]
        })
        .collect();
    print_table(
        lang,
        &headers(lang, &["distance", "symbol", "geometry", "part"]),
        rows,
    );
    if !features.is_empty() {
        println!(
            "{}",
            text(
                lang,
                "cli-map-near-total",
                &[
                    ("count", FluentValue::from(features.len())),
                    ("total", FluentValue::from(s(&value["total"]))),
                ],
            )
        );
    }
}
