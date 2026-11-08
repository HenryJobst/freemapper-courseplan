//! The edit commands: `control`, `special` and `class`. Each loads the project, applies one edit
//! from `freemapper_courseplan::edit`, saves, and prints one confirmation line from the catalog.
//! The map is read only where the operation checks a point against its extent.

use std::path::{Path, PathBuf};

use clap::{ArgGroup, Subcommand, ValueEnum};
use freemapper_courseplan::edit;
use freemapper_courseplan::map::MapIndex;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{
    Class, ControlCode, ControlDescription, ControlKind, MapPoint, NumberPlacement, ProjectFile,
    SpecialObject,
};

/// Why a command failed: a wrong call (exit code 2) or a refused operation (exit code 1).
pub enum Failure {
    Usage(String),
    Error(String),
}

type Outcome = Result<(), Failure>;

// The help texts come from the catalog (see `localize_groups`), not from doc comments.
#[derive(Subcommand)]
pub enum ControlCmd {
    /// Add a control
    Add {
        project: PathBuf,
        #[arg(long, value_name = "X,Y", allow_hyphen_values = true)]
        at: String,
        #[arg(long)]
        code: Option<String>,
        #[arg(long)]
        crossing_point: bool,
    },
    /// Move a control
    Move {
        project: PathBuf,
        code: String,
        #[arg(long, value_name = "X,Y", allow_hyphen_values = true)]
        at: String,
    },
    /// Remove a control
    Remove { project: PathBuf, code: String },
    /// Set the description symbols of a control
    #[command(group(
        ArgGroup::new("change")
            .required(true)
            .multiple(true)
            .args(["c", "d", "e", "f", "g", "h", "clear"])
    ))]
    Describe {
        project: PathBuf,
        code: String,
        #[arg(long, value_name = "ID")]
        c: Option<String>,
        #[arg(long, value_name = "ID")]
        d: Option<String>,
        #[arg(long, value_name = "ID")]
        e: Option<String>,
        #[arg(long, value_name = "ID")]
        f: Option<String>,
        #[arg(long, value_name = "ID")]
        g: Option<String>,
        #[arg(long, value_name = "ID")]
        h: Option<String>,
        #[arg(long)]
        clear: bool,
    },
    /// Set or clear the number placement of a control
    #[command(group(
        ArgGroup::new("mode").required(true).args(["direction", "clear"])
    ))]
    Number {
        project: PathBuf,
        code: String,
        #[arg(long, value_name = "DEG", requires = "gap", allow_hyphen_values = true)]
        direction: Option<String>,
        #[arg(
            long,
            value_name = "MM",
            requires = "direction",
            allow_hyphen_values = true
        )]
        gap: Option<String>,
        #[arg(long, conflicts_with_all = ["direction", "gap"])]
        clear: bool,
    },
}

#[derive(Subcommand)]
pub enum SpecialCmd {
    /// Add a special object
    Add {
        project: PathBuf,
        #[arg(long, value_enum)]
        kind: SpecialKind,
        #[arg(long)]
        id: String,
        #[arg(
            long,
            value_name = "X,Y",
            allow_hyphen_values = true,
            conflicts_with = "points"
        )]
        at: Option<String>,
        #[arg(
            long,
            value_name = "X,Y",
            num_args = 1..,
            allow_hyphen_values = true,
        )]
        points: Vec<String>,
        #[arg(long, value_name = "A,B", value_delimiter = ',')]
        courses: Vec<String>,
    },
    /// Remove a special object
    Remove { project: PathBuf, id: String },
}

#[derive(Subcommand)]
pub enum ClassCmd {
    /// Add a class
    Add {
        project: PathBuf,
        name: String,
        #[arg(long, value_name = "CAT")]
        rule_category: Option<String>,
        #[arg(long, value_name = "N")]
        starters: Option<String>,
    },
    /// Remove a class
    Remove { project: PathBuf, name: String },
}

/// The kinds of `special add`, named as on the command line.
#[derive(Clone, Copy, ValueEnum)]
pub enum SpecialKind {
    Start,
    Finish,
    MarkedRoute,
    OutOfBounds,
    ForbiddenRoute,
    DangerousArea,
    FirstAid,
    Water,
}

impl SpecialKind {
    fn word_id(self) -> &'static str {
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
}

/// Sets the catalog texts on the three command groups and everything below them.
pub fn localize_groups(root: clap::Command, lang: &Lang) -> clap::Command {
    let mut root = root;
    for group in ["control", "special", "class"] {
        root = root.mut_subcommand(group, |g| {
            let subs: Vec<String> = g
                .get_subcommands()
                .map(|s| s.get_name().to_owned())
                .collect();
            let mut g = g.about(text(lang, &format!("cli-help-{group}"), &[]));
            for sub in subs {
                let path = format!("{group}-{sub}");
                g = g.mut_subcommand(&sub, |s| {
                    s.about(text(lang, &format!("cli-help-{path}"), &[]))
                        .mut_args(|a| {
                            let id = match a.get_id().as_str() {
                                "project" => "cli-arg-info-project".to_owned(),
                                "code" => "cli-arg-edit-code".to_owned(),
                                name => format!("cli-arg-{path}-{}", name.replace('_', "-")),
                            };
                            a.help(text(lang, &id, &[]))
                        })
                });
            }
            g
        });
    }
    root
}

fn v(value: impl Into<String>) -> FluentValue<'static> {
    FluentValue::from(value.into())
}

/// A number without trailing zeros: `12.5`, `-30`, `0`.
fn fmt(x: f64) -> String {
    if x == 0.0 {
        "0".to_owned()
    } else {
        x.to_string()
    }
}

fn parse_number(lang: &Lang, value: &str) -> Result<f64, Failure> {
    match value.parse::<f64>() {
        Ok(x) if x.is_finite() => Ok(x),
        _ => Err(Failure::Usage(text(
            lang,
            "cli-edit-bad-number",
            &[("value", v(value))],
        ))),
    }
}

/// `X,Y` in map millimetres with a dot as decimal separator.
fn parse_point(lang: &Lang, value: &str) -> Result<MapPoint, Failure> {
    let parsed = value.split_once(',').and_then(|(x, y)| {
        let x = x.parse::<f64>().ok().filter(|x| x.is_finite())?;
        let y = y.parse::<f64>().ok().filter(|y| y.is_finite())?;
        Some(MapPoint { x, y })
    });
    parsed.ok_or_else(|| Failure::Usage(text(lang, "cli-edit-bad-point", &[("value", v(value))])))
}

fn load(lang: &Lang, path: &Path) -> Result<ProjectFile, Failure> {
    ProjectFile::load(path).map_err(|e| Failure::Error(e.localize(lang)))
}

fn save(lang: &Lang, project: &mut ProjectFile) -> Outcome {
    project.save().map_err(|e| Failure::Error(e.localize(lang)))
}

/// The extent of the project's map, for checking points against it.
fn map_extent(lang: &Lang, project: &ProjectFile) -> Result<Option<(MapPoint, MapPoint)>, Failure> {
    let map_path = project.map_path();
    let doc = read_map_file(&map_path).map_err(|e| {
        Failure::Error(text(
            lang,
            "error-init-map",
            &[
                ("path", v(map_path.display().to_string())),
                ("source", v(e.localize(lang))),
            ],
        ))
    })?;
    Ok(MapIndex::from_document(&doc).extent())
}

fn refused(lang: &Lang, e: edit::EditError) -> Failure {
    Failure::Error(e.localize(lang))
}

pub fn control(lang: &Lang, cmd: ControlCmd) -> Outcome {
    match cmd {
        ControlCmd::Add {
            project,
            at,
            code,
            crossing_point,
        } => {
            let at = parse_point(lang, &at)?;
            let mut file = load(lang, &project)?;
            let extent = map_extent(lang, &file)?;
            let kind = if crossing_point {
                ControlKind::CrossingPoint
            } else {
                ControlKind::Control
            };
            let (x, y) = (fmt(at.x), fmt(at.y));
            let code = edit::add_control(&mut file.event, extent, at, code.map(ControlCode), kind)
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(
                lang,
                "cli-edit-control-added",
                &[("code", v(code.0)), ("x", v(x)), ("y", v(y))],
            );
        }
        ControlCmd::Move { project, code, at } => {
            let at = parse_point(lang, &at)?;
            let mut file = load(lang, &project)?;
            let extent = map_extent(lang, &file)?;
            let (x, y) = (fmt(at.x), fmt(at.y));
            edit::move_control(&mut file.event, extent, &ControlCode(code.clone()), at)
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(
                lang,
                "cli-edit-control-moved",
                &[("code", v(code)), ("x", v(x)), ("y", v(y))],
            );
        }
        ControlCmd::Remove { project, code } => {
            let mut file = load(lang, &project)?;
            let left = edit::remove_control(&mut file.event, &ControlCode(code.clone()))
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            if left.is_empty() {
                say(lang, "cli-edit-control-removed", &[("code", v(code))]);
            } else {
                let courses = left.join(", ");
                say(
                    lang,
                    "cli-edit-control-removed-from",
                    &[("code", v(code)), ("courses", v(courses))],
                );
            }
        }
        ControlCmd::Describe {
            project,
            code,
            c,
            d,
            e,
            f,
            g,
            h,
            clear,
        } => {
            let mut file = load(lang, &project)?;
            let key = ControlCode(code.clone());
            let current = file
                .event
                .controls
                .iter()
                .find(|x| x.code == key)
                .map(|x| x.description.clone());
            // An unknown code is reported by the edit operation below.
            let mut description = if clear {
                ControlDescription::default()
            } else {
                current.unwrap_or_default()
            };
            for (slot, new) in [
                (&mut description.c, c),
                (&mut description.d, d),
                (&mut description.e, e),
                (&mut description.f, f),
                (&mut description.g, g),
                (&mut description.h, h),
            ] {
                if new.is_some() {
                    *slot = new;
                }
            }
            edit::describe_control(&mut file.event, &key, description.clone())
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            let columns: Vec<String> = [
                ("C", &description.c),
                ("D", &description.d),
                ("E", &description.e),
                ("F", &description.f),
                ("G", &description.g),
                ("H", &description.h),
            ]
            .into_iter()
            .filter_map(|(name, id)| id.as_ref().map(|id| format!("{name} {id}")))
            .collect();
            let columns = if columns.is_empty() {
                text(lang, "word-empty", &[])
            } else {
                columns.join(", ")
            };
            say(
                lang,
                "cli-edit-control-described",
                &[("code", v(code)), ("columns", v(columns))],
            );
        }
        ControlCmd::Number {
            project,
            code,
            direction,
            gap,
            clear,
        } => {
            let placement = match (direction, gap) {
                (Some(direction), Some(gap)) if !clear => Some(NumberPlacement {
                    direction: parse_number(lang, &direction)?,
                    gap_mm: parse_number(lang, &gap)?,
                }),
                _ => None,
            };
            let mut file = load(lang, &project)?;
            edit::set_central_number(
                &mut file.event,
                &ControlCode(code.clone()),
                placement.clone(),
            )
            .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            match placement {
                Some(p) => say(
                    lang,
                    "cli-edit-number-set",
                    &[
                        ("code", v(code)),
                        ("direction", v(fmt(p.direction))),
                        ("gap", v(fmt(p.gap_mm))),
                    ],
                ),
                None => say(lang, "cli-edit-number-cleared", &[("code", v(code))]),
            }
        }
    }
    Ok(())
}

pub fn special(lang: &Lang, cmd: SpecialCmd) -> Outcome {
    match cmd {
        SpecialCmd::Add {
            project,
            kind,
            id,
            at,
            points,
            courses,
        } => {
            let kind_word = text(lang, kind.word_id(), &[]);
            let wrong =
                |id: &str| Failure::Usage(text(lang, id, &[("kind", v(kind_word.clone()))]));
            let is_point = matches!(
                kind,
                SpecialKind::Start
                    | SpecialKind::Finish
                    | SpecialKind::FirstAid
                    | SpecialKind::Water
            );
            if is_point && (at.is_none() || !points.is_empty()) {
                return Err(wrong("cli-edit-kind-needs-at"));
            }
            if !is_point && (at.is_some() || points.is_empty()) {
                return Err(wrong("cli-edit-kind-needs-points"));
            }
            if matches!(kind, SpecialKind::Start | SpecialKind::Finish) && !courses.is_empty() {
                return Err(wrong("cli-edit-no-courses-for-kind"));
            }
            let at = at.map(|a| parse_point(lang, &a)).transpose()?;
            let points = points
                .iter()
                .map(|p| parse_point(lang, p))
                .collect::<Result<Vec<_>, _>>()?;
            let shown = match &at {
                Some(p) => (fmt(p.x), fmt(p.y)),
                None => (String::new(), String::new()),
            };
            let count = points.len();
            let id_copy = id.clone();
            let map_mm = || at.clone().unwrap_or(MapPoint { x: 0.0, y: 0.0 });
            let special = match kind {
                SpecialKind::Start => SpecialObject::Start {
                    id,
                    map_mm: map_mm(),
                },
                SpecialKind::Finish => SpecialObject::Finish {
                    id,
                    map_mm: map_mm(),
                },
                SpecialKind::FirstAid => SpecialObject::FirstAid {
                    id,
                    map_mm: map_mm(),
                    courses,
                },
                SpecialKind::Water => SpecialObject::Water {
                    id,
                    map_mm: map_mm(),
                    courses,
                },
                SpecialKind::MarkedRoute => SpecialObject::MarkedRoute {
                    id,
                    points,
                    courses,
                },
                SpecialKind::ForbiddenRoute => SpecialObject::ForbiddenRoute {
                    id,
                    points,
                    courses,
                },
                SpecialKind::OutOfBounds => SpecialObject::OutOfBoundsArea {
                    id,
                    polygon: points,
                    courses,
                },
                SpecialKind::DangerousArea => SpecialObject::DangerousArea {
                    id,
                    polygon: points,
                    courses,
                },
            };
            let mut file = load(lang, &project)?;
            let extent = map_extent(lang, &file)?;
            edit::add_special(&mut file.event, extent, special).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            let common = [("kind", v(kind_word)), ("id", v(id_copy))];
            if is_point {
                let mut args = common.to_vec();
                args.push(("x", v(shown.0)));
                args.push(("y", v(shown.1)));
                say(lang, "cli-edit-special-added-at", &args);
            } else {
                let mut args = common.to_vec();
                args.push(("count", v(count.to_string())));
                say(lang, "cli-edit-special-added-points", &args);
            }
        }
        SpecialCmd::Remove { project, id } => {
            let mut file = load(lang, &project)?;
            edit::remove_special(&mut file.event, &id).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-special-removed", &[("id", v(id))]);
        }
    }
    Ok(())
}

pub fn class(lang: &Lang, cmd: ClassCmd) -> Outcome {
    match cmd {
        ClassCmd::Add {
            project,
            name,
            rule_category,
            starters,
        } => {
            let expected_starters = starters
                .map(|s| {
                    s.parse::<u32>().map_err(|_| {
                        Failure::Usage(text(lang, "cli-edit-bad-count", &[("value", v(s.clone()))]))
                    })
                })
                .transpose()?;
            let mut file = load(lang, &project)?;
            let class = Class {
                name: name.clone(),
                rule_category,
                expected_starters,
            };
            edit::add_class(&mut file.event, class).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-class-added", &[("name", v(name))]);
        }
        ClassCmd::Remove { project, name } => {
            let mut file = load(lang, &project)?;
            edit::remove_class(&mut file.event, &name).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-class-removed", &[("name", v(name))]);
        }
    }
    Ok(())
}

fn say(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) {
    println!("{}", text(lang, id, args));
}
