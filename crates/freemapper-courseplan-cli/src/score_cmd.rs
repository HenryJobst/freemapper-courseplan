//! `score controls|value|link|settings|course` and `course add --kind score`: the score editing tools on the
//! command line (ADR 0067). Like the read commands (ADR 0059), each runs the tool's operation through
//! `CoursePlanServer::call_json`, so `--json` prints what the MCP tool answers; without it the same value
//! becomes one line from the catalog.

use std::path::PathBuf;

use clap::{ArgGroup, Subcommand};
use freemapper_courseplan::text::{FluentValue, Lang, text};
use serde_json::{Value, json};

use crate::edit_cmd::{Failure, parse_count};
use crate::query_cmd::{print_json, run_tool};

type Outcome = Result<(), Failure>;

// The help texts come from the catalog (see `edit_cmd::localize_groups`), not from doc comments.
#[derive(Subcommand)]
pub enum ScoreCmd {
    /// Replace the controls of a score course
    Controls {
        project: PathBuf,
        name: String,
        codes: String,
        #[arg(long)]
        json: bool,
    },
    /// Set the value of a control
    Value {
        project: PathBuf,
        name: String,
        code: String,
        value: String,
        #[arg(long)]
        json: bool,
    },
    /// Add or remove mandatory links
    Link {
        #[command(subcommand)]
        action: LinkCmd,
    },
    /// Change the score settings
    #[command(group(
        ArgGroup::new("change")
            .required(true)
            .multiple(true)
            .args(["value_max", "good_pace", "very_good_pace", "mix_hard", "seed"])
    ))]
    Settings {
        project: PathBuf,
        #[arg(long, value_name = "N")]
        value_max: Option<String>,
        #[arg(long, value_name = "S")]
        good_pace: Option<String>,
        #[arg(long, value_name = "S")]
        very_good_pace: Option<String>,
        #[arg(long, value_name = "PERCENT")]
        mix_hard: Option<String>,
        #[arg(long, value_name = "N")]
        seed: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Change a score course
    Course {
        #[command(subcommand)]
        action: ScoreCourseCmd,
    },
}

#[derive(Subcommand)]
pub enum LinkCmd {
    /// Add a link
    Add {
        project: PathBuf,
        name: String,
        a: String,
        b: String,
        #[arg(long)]
        json: bool,
    },
    /// Remove a link
    Remove {
        project: PathBuf,
        name: String,
        a: String,
        b: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum ScoreCourseCmd {
    /// Change the time limit or the penalty
    #[command(group(
        ArgGroup::new("change")
            .required(true)
            .multiple(true)
            .args(["time_limit", "penalty", "no_penalty"])
    ))]
    Set {
        project: PathBuf,
        name: String,
        #[arg(long, value_name = "MIN")]
        time_limit: Option<String>,
        #[arg(long, value_name = "N", conflicts_with = "no_penalty")]
        penalty: Option<String>,
        #[arg(long)]
        no_penalty: bool,
        #[arg(long)]
        json: bool,
    },
}

pub fn score(lang: &Lang, cmd: ScoreCmd) -> Outcome {
    match cmd {
        ScoreCmd::Controls {
            project,
            name,
            codes,
            json,
        } => {
            let controls = parse_controls(lang, &codes)?;
            let value = run_tool(
                lang,
                &project,
                "set_score_controls",
                json!({ "name": name, "controls": controls }),
            )?;
            if json {
                return print_json(&value);
            }
            let list = items(&value, "controls");
            let total: u64 = list.iter().filter_map(|c| c["value"].as_u64()).sum();
            let shown = list
                .iter()
                .map(|c| format!("{}={}", s(&c["code"]), s(&c["value"])))
                .collect::<Vec<_>>()
                .join(", ");
            say(
                lang,
                "cli-score-controls",
                &[
                    ("name", s(&value["name"])),
                    ("count", list.len().to_string()),
                    ("total", total.to_string()),
                    ("list", shown),
                ],
            );
            Ok(())
        }
        ScoreCmd::Value {
            project,
            name,
            code,
            value,
            json,
        } => {
            let number = parse_count(lang, &value)?;
            let answer = run_tool(
                lang,
                &project,
                "set_control_value",
                json!({ "name": name, "code": code, "value": number }),
            )?;
            if json {
                return print_json(&answer);
            }
            say(
                lang,
                "cli-score-value",
                &[
                    ("name", s(&answer["name"])),
                    ("code", s(&answer["code"])),
                    ("value", s(&answer["value"])),
                ],
            );
            Ok(())
        }
        ScoreCmd::Link { action } => link(lang, action),
        ScoreCmd::Settings {
            project,
            value_max,
            good_pace,
            very_good_pace,
            mix_hard,
            seed,
            json,
        } => {
            let mut args = json!({});
            if let Some(v) = value_max {
                args["value_max"] = json!(clamp_u8(parse_count(lang, &v)?));
            }
            if let Some(v) = good_pace {
                args["good_pace_s_per_km"] = json!(parse_pace(lang, &v)?);
            }
            if let Some(v) = very_good_pace {
                args["very_good_pace_s_per_km"] = json!(parse_pace(lang, &v)?);
            }
            if let Some(v) = mix_hard {
                args["mix_hard_percent"] = json!(clamp_u8(parse_count(lang, &v)?));
            }
            if let Some(v) = seed {
                args["seed"] = json!(parse_count(lang, &v)?);
            }
            let value = run_tool(lang, &project, "set_score_settings", args)?;
            if json {
                return print_json(&value);
            }
            say(
                lang,
                "cli-score-settings",
                &[
                    ("value_max", s(&value["value_max"])),
                    ("good", s(&value["good_pace_s_per_km"])),
                    ("very_good", s(&value["very_good_pace_s_per_km"])),
                    ("mix_hard", s(&value["mix_hard_percent"])),
                    ("seed", s(&value["seed"])),
                ],
            );
            Ok(())
        }
        ScoreCmd::Course {
            action:
                ScoreCourseCmd::Set {
                    project,
                    name,
                    time_limit,
                    penalty,
                    no_penalty,
                    json,
                },
        } => {
            let mut args = json!({ "name": name });
            if let Some(v) = time_limit {
                args["time_limit_min"] = json!(parse_count(lang, &v)?);
            }
            if let Some(v) = penalty {
                args["penalty_per_min"] = json!(parse_count(lang, &v)?);
            }
            if no_penalty {
                args["no_penalty"] = json!(true);
            }
            let value = run_tool(lang, &project, "set_score_course_details", args)?;
            if json {
                return print_json(&value);
            }
            say(
                lang,
                "cli-score-course-set",
                &[
                    ("name", s(&value["name"])),
                    ("limit", s(&value["time_limit_min"])),
                    ("penalty", s(&value["penalty_per_min"])),
                    ("default", default_flag(&value)),
                ],
            );
            Ok(())
        }
    }
}

/// `yes` when the answer's penalty is the default, for the select in the catalog text.
fn default_flag(value: &Value) -> String {
    if value["penalty_is_default"] == true {
        "yes"
    } else {
        "no"
    }
    .to_owned()
}

fn link(lang: &Lang, cmd: LinkCmd) -> Outcome {
    let (project, name, a, b, json, tool, id) = match cmd {
        LinkCmd::Add {
            project,
            name,
            a,
            b,
            json,
        } => (
            project,
            name,
            a,
            b,
            json,
            "add_score_link",
            "cli-score-link-added",
        ),
        LinkCmd::Remove {
            project,
            name,
            a,
            b,
            json,
        } => (
            project,
            name,
            a,
            b,
            json,
            "remove_score_link",
            "cli-score-link-removed",
        ),
    };
    let value = run_tool(
        lang,
        &project,
        tool,
        json!({ "name": name, "a": a, "b": b }),
    )?;
    if json {
        return print_json(&value);
    }
    say(
        lang,
        id,
        &[
            ("name", s(&value["name"])),
            ("a", s(&value["a"])),
            ("b", s(&value["b"])),
        ],
    );
    Ok(())
}

/// The arguments of `course add --kind score`.
pub struct NewScoreCourse {
    pub project: PathBuf,
    pub name: String,
    pub start: String,
    pub finish: String,
    pub time_limit: u32,
    pub penalty: Option<u32>,
}

/// `course add --kind score`: the tool `add_score_course`.
pub fn add_course(lang: &Lang, new: NewScoreCourse, json: bool) -> Outcome {
    let NewScoreCourse {
        project,
        name,
        start,
        finish,
        time_limit,
        penalty,
    } = new;
    let mut args = json!({
        "name": name,
        "start": start,
        "finish": finish,
        "time_limit_min": time_limit,
    });
    if let Some(p) = penalty {
        args["penalty_per_min"] = json!(p);
    }
    let value = run_tool(lang, &project, "add_score_course", args)?;
    if json {
        return print_json(&value);
    }
    say(
        lang,
        "cli-score-course-added",
        &[
            ("name", s(&value["name"])),
            ("limit", s(&value["time_limit_min"])),
            ("penalty", s(&value["penalty_per_min"])),
            ("default", default_flag(&value)),
        ],
    );
    Ok(())
}

/// `31,42=2` as `[{code, value?}]`.
fn parse_controls(lang: &Lang, list: &str) -> Result<Vec<Value>, Failure> {
    let bad = |item: &str| {
        Failure::Usage(text(
            lang,
            "cli-score-bad-control",
            &[("value", FluentValue::from(item.to_owned()))],
        ))
    };
    let mut out = Vec::new();
    for item in list.split(',').map(str::trim).filter(|i| !i.is_empty()) {
        match item.split_once('=') {
            None => out.push(json!({ "code": item })),
            Some((code, value)) => {
                let code = code.trim();
                let value = value.trim().parse::<u32>().map_err(|_| bad(item))?;
                if code.is_empty() {
                    return Err(bad(item));
                }
                out.push(json!({ "code": code, "value": value }));
            }
        }
    }
    Ok(out)
}

/// A pace in seconds per kilometre: `250` or `4:10`.
fn parse_pace(lang: &Lang, value: &str) -> Result<u32, Failure> {
    let parsed = match value.split_once(':') {
        Some((m, sec)) => m
            .trim()
            .parse::<u32>()
            .ok()
            .zip(sec.trim().parse::<u32>().ok().filter(|s| *s < 60))
            .map(|(m, sec)| m * 60 + sec),
        None => value.trim().parse::<u32>().ok(),
    };
    parsed.filter(|p| *p > 0).ok_or_else(|| {
        Failure::Usage(text(
            lang,
            "cli-score-bad-pace",
            &[("value", FluentValue::from(value.to_owned()))],
        ))
    })
}

fn clamp_u8(n: u32) -> u8 {
    u8::try_from(n).unwrap_or(u8::MAX)
}

fn s(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn items<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value[key].as_array().map_or(&[], Vec::as_slice)
}

fn say(lang: &Lang, id: &str, args: &[(&str, String)]) {
    let args: Vec<(&str, FluentValue)> = args
        .iter()
        .map(|(k, v)| (*k, FluentValue::from(v.clone())))
        .collect();
    println!("{}", text(lang, id, &args));
}
