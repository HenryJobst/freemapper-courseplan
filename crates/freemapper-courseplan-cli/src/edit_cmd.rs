//! The edit commands: `control`, `special`, `class` and `course`. Each loads the project, applies one edit
//! from `freemapper_courseplan::edit`, saves, and prints one confirmation line from the catalog.
//! The map is read only where the operation checks a point against its extent.

use std::path::{Path, PathBuf};

use clap::{ArgGroup, Subcommand, ValueEnum};
use freemapper_courseplan::edit::{self, LegStart};
use freemapper_courseplan::map::{MapIndex, map_kind};
use freemapper_courseplan::numbers::{self, MAX_GAP_MM};
use freemapper_courseplan::omap::model::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{
    Class, ControlCode, ControlDescription, ControlKind, Course, CourseControl, CourseKind,
    MapPoint, NumberPlacement, ProjectFile, SpecialObject,
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
            conflicts_with = "point"
        )]
        at: Option<String>,
        #[arg(
            long,
            value_name = "X,Y",
            num_args = 1,
            action = clap::ArgAction::Append,
            allow_hyphen_values = true,
        )]
        point: Vec<String>,
        #[arg(long, value_name = "A,B", value_delimiter = ',')]
        courses: Vec<String>,
    },
    /// Remove a special object
    Remove { project: PathBuf, id: String },
    /// Set the courses a special object belongs to
    #[command(group(
        ArgGroup::new("target").required(true).args(["courses", "all"])
    ))]
    Courses {
        project: PathBuf,
        id: String,
        #[arg(value_name = "A,B")]
        courses: Option<String>,
        #[arg(long)]
        all: bool,
    },
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

#[derive(Subcommand)]
pub enum CourseCmd {
    /// Add an individual course
    Add {
        project: PathBuf,
        name: String,
        #[arg(long, value_name = "ID")]
        start: String,
        #[arg(long, value_name = "ID")]
        finish: String,
        #[arg(long, value_name = "31,32", value_delimiter = ',')]
        controls: Vec<String>,
        #[arg(long, value_name = "A,B", value_delimiter = ',')]
        classes: Vec<String>,
        #[arg(long, value_name = "M")]
        climb: Option<String>,
        #[arg(long, value_name = "N")]
        print_scale: Option<String>,
    },
    /// Replace the controls of a course
    Controls {
        project: PathBuf,
        name: String,
        /// Codes in running order, separated by commas
        codes: String,
    },
    /// Remove a course
    Remove { project: PathBuf, name: String },
    /// Set the bends and the marked-route flag of one leg
    #[command(group(
        ArgGroup::new("leg").required(true).args(["from_start", "after"])
    ))]
    Leg {
        project: PathBuf,
        name: String,
        #[arg(long)]
        from_start: bool,
        #[arg(long, value_name = "N")]
        after: Option<String>,
        #[arg(
            long,
            value_name = "X,Y",
            num_args = 1,
            action = clap::ArgAction::Append,
            allow_hyphen_values = true,
        )]
        bend: Vec<String>,
        #[arg(long)]
        taped: bool,
    },
    /// Change classes, climb or print scale of a course
    #[command(group(
        ArgGroup::new("change")
            .required(true)
            .multiple(true)
            .args(["classes", "climb", "no_climb", "print_scale", "no_print_scale"])
    ))]
    Set {
        project: PathBuf,
        name: String,
        #[arg(long, value_name = "A,B", value_delimiter = ',')]
        classes: Option<Vec<String>>,
        #[arg(long, value_name = "M", conflicts_with = "no_climb")]
        climb: Option<String>,
        #[arg(long)]
        no_climb: bool,
        #[arg(long, value_name = "N", conflicts_with = "no_print_scale")]
        print_scale: Option<String>,
        #[arg(long)]
        no_print_scale: bool,
    },
    /// Set or clear the number placement of one control of a course
    #[command(group(
        ArgGroup::new("mode").required(true).args(["direction", "clear"])
    ))]
    Number {
        project: PathBuf,
        name: String,
        #[arg(long, value_name = "N")]
        after: String,
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
    /// Place the control numbers of a course
    Numbers {
        project: PathBuf,
        name: String,
        #[arg(long)]
        overwrite: bool,
    },
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

/// Arguments that hold codes, ids or names, or lists of them. Their values lose surrounding white
/// space, each list item on its own, so `--controls "31, 32"` means 31 and 32.
const TRIMMED: [&str; 8] = [
    "code", "id", "name", "start", "finish", "controls", "classes", "courses",
];

fn trimmed(value: &str) -> Result<String, std::convert::Infallible> {
    Ok(value.trim().to_owned())
}

/// Sets the catalog texts on the four command groups and everything below them, and the trimming
/// value parser on the arguments in [`TRIMMED`].
pub fn localize_groups(root: clap::Command, lang: &Lang) -> clap::Command {
    let mut root = root;
    for group in ["control", "special", "class", "course"] {
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
                            let a = if TRIMMED.contains(&a.get_id().as_str()) {
                                a.value_parser(trimmed)
                            } else {
                                a
                            };
                            let id = match a.get_id().as_str() {
                                "project" => "cli-arg-info-project".to_owned(),
                                "code" if path != "control-add" => "cli-arg-edit-code".to_owned(),
                                "name" if group == "course" => "cli-arg-edit-course".to_owned(),
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

/// A number placement from `--direction` and `--gap`: the direction brought into [0, 360), the gap
/// more than 0 and at most [`MAX_GAP_MM`] print mm.
fn parse_placement(lang: &Lang, direction: &str, gap: &str) -> Result<NumberPlacement, Failure> {
    let direction = numbers::normalize_direction(parse_number(lang, direction)?);
    let gap_mm = parse_number(lang, gap)?;
    if !numbers::gap_in_range(gap_mm) {
        return Err(Failure::Usage(text(
            lang,
            "cli-edit-bad-gap",
            &[("value", v(gap)), ("max", v(fmt(MAX_GAP_MM)))],
        )));
    }
    Ok(NumberPlacement { direction, gap_mm })
}

fn parse_count(lang: &Lang, value: &str) -> Result<u32, Failure> {
    value
        .parse::<u32>()
        .map_err(|_| Failure::Usage(text(lang, "cli-edit-bad-count", &[("value", v(value))])))
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

pub fn load(lang: &Lang, path: &Path) -> Result<ProjectFile, Failure> {
    ProjectFile::load(path).map_err(|e| Failure::Error(e.localize(lang)))
}

pub fn save(lang: &Lang, project: &mut ProjectFile) -> Outcome {
    project.save().map_err(|e| Failure::Error(e.localize(lang)))
}

/// The project's map.
pub fn read_map(lang: &Lang, project: &ProjectFile) -> Result<MapDocument, Failure> {
    let map_path = project.map_path();
    read_map_file(&map_path).map_err(|e| {
        Failure::Error(text(
            lang,
            "error-init-map",
            &[
                ("path", v(map_path.display().to_string())),
                ("source", v(e.localize(lang))),
            ],
        ))
    })
}

/// The extent of the project's map, for checking points against it.
pub fn map_extent(
    lang: &Lang,
    project: &ProjectFile,
) -> Result<Option<(MapPoint, MapPoint)>, Failure> {
    Ok(MapIndex::from_document(&read_map(lang, project)?).extent())
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
            let id = if crossing_point {
                "cli-edit-crossing-point-added"
            } else {
                "cli-edit-control-added"
            };
            say(lang, id, &[("code", v(code.0)), ("x", v(x)), ("y", v(y))]);
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
            let removed = edit::remove_control(&mut file.event, &ControlCode(code.clone()))
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            if removed.courses.is_empty() {
                say(lang, "cli-edit-control-removed", &[("code", v(code))]);
            } else {
                let courses = removed.courses.join(", ");
                say(
                    lang,
                    "cli-edit-control-removed-from",
                    &[("code", v(code)), ("courses", v(courses))],
                );
            }
            say_cleared(lang, &removed.cleared);
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
                (Some(direction), Some(gap)) if !clear => {
                    Some(parse_placement(lang, &direction, &gap)?)
                }
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
            point: points,
            courses,
        } => {
            let courses = codes(courses);
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
                args.push(("count", FluentValue::from(count)));
                say(lang, "cli-edit-special-added-points", &args);
            }
        }
        SpecialCmd::Remove { project, id } => {
            let mut file = load(lang, &project)?;
            edit::remove_special(&mut file.event, &id).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-special-removed", &[("id", v(id))]);
        }
        SpecialCmd::Courses {
            project,
            id,
            courses,
            all,
        } => {
            let courses = match courses {
                Some(list) if !all => {
                    let names = split_list(&list);
                    if names.is_empty() {
                        return Err(Failure::Usage(text(lang, "cli-edit-no-course-names", &[])));
                    }
                    names
                }
                _ => Vec::new(),
            };
            let mut file = load(lang, &project)?;
            edit::set_special_courses(&mut file.event, &id, courses.clone())
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            if courses.is_empty() {
                say(lang, "cli-edit-special-all-courses", &[("id", v(id))]);
            } else {
                say(
                    lang,
                    "cli-edit-special-courses",
                    &[("id", v(id)), ("courses", v(courses.join(", ")))],
                );
            }
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

pub fn course_control(code: String) -> CourseControl {
    CourseControl {
        code: ControlCode(code),
        number: None,
        bends: vec![],
        taped: false,
    }
}

/// The items of a comma-separated list, each without surrounding white space; empty items are
/// left out.
pub fn split_list(list: &str) -> Vec<String> {
    codes(list.split(',').map(str::to_owned))
}

/// Codes or names from a list, each without surrounding white space; empty items are left out.
pub fn codes(list: impl IntoIterator<Item = String>) -> Vec<String> {
    list.into_iter()
        .map(|c| c.trim().to_owned())
        .filter(|c| !c.is_empty())
        .collect()
}

pub fn course(lang: &Lang, cmd: CourseCmd) -> Outcome {
    match cmd {
        CourseCmd::Add {
            project,
            name,
            start,
            finish,
            controls,
            classes,
            climb,
            print_scale,
        } => {
            let climb_m = climb.map(|c| parse_count(lang, &c)).transpose()?;
            let planned_print_scale = print_scale.map(|c| parse_count(lang, &c)).transpose()?;
            let controls: Vec<CourseControl> =
                codes(controls).into_iter().map(course_control).collect();
            let mut file = load(lang, &project)?;
            let course = Course {
                name: name.clone(),
                classes: codes(classes),
                start,
                finish,
                start_bends: vec![],
                start_taped: false,
                print_area: None,
                climb_m,
                planned_print_scale,
                kind: CourseKind::Individual { controls },
            };
            edit::add_course(&mut file.event, course).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-course-added", &[("name", v(name))]);
        }
        CourseCmd::Controls {
            project,
            name,
            codes: list,
        } => {
            let list = codes(list.split(',').map(|c| c.trim().to_owned()));
            let mut file = load(lang, &project)?;
            let sequence: Vec<ControlCode> = list.iter().cloned().map(ControlCode).collect();
            let cleared = edit::set_course_controls(&mut file.event, &name, &sequence)
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            let shown = if list.is_empty() {
                text(lang, "word-empty", &[])
            } else {
                list.join(", ")
            };
            say(
                lang,
                "cli-edit-course-controls",
                &[("name", v(name)), ("codes", v(shown))],
            );
            say_cleared(lang, &cleared);
        }
        CourseCmd::Remove { project, name } => {
            let mut file = load(lang, &project)?;
            edit::remove_course(&mut file.event, &name).map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-course-removed", &[("name", v(name))]);
        }
        CourseCmd::Leg {
            project,
            name,
            from_start,
            after,
            bend,
            taped,
        } => {
            // The command line counts from 1 as on the map; the library counts from 0.
            let shown = after.as_deref().map(|a| parse_count(lang, a)).transpose()?;
            let bad_index = |index: u32| {
                refused(
                    lang,
                    edit::EditError::BadIndex {
                        course: name.clone(),
                        index: index as usize,
                    },
                )
            };
            let leg = match shown {
                _ if from_start => LegStart::Start,
                Some(0) | None => return Err(bad_index(0)),
                Some(n) => LegStart::After(n as usize - 1),
            };
            let bends = bend
                .iter()
                .map(|b| parse_point(lang, b))
                .collect::<Result<Vec<_>, _>>()?;
            let count = bends.len();
            let mut file = load(lang, &project)?;
            edit::set_leg(&mut file.event, &name, leg, bends, taped).map_err(|e| match e {
                edit::EditError::BadIndex { .. } => bad_index(shown.unwrap_or(0)),
                other => refused(lang, other),
            })?;
            save(lang, &mut file)?;
            let taped = text(
                lang,
                if taped {
                    "word-taped"
                } else {
                    "word-not-taped"
                },
                &[],
            );
            let mut args = vec![
                ("name", v(name)),
                ("bends", v(count.to_string())),
                ("taped", v(taped)),
            ];
            match shown {
                Some(index) if !from_start => {
                    args.push(("index", v(index.to_string())));
                    say(lang, "cli-edit-course-leg-after", &args);
                }
                _ => say(lang, "cli-edit-course-leg-start", &args),
            }
        }
        CourseCmd::Set {
            project,
            name,
            classes,
            climb,
            no_climb,
            print_scale,
            no_print_scale,
        } => {
            let climb = match (climb, no_climb) {
                (Some(c), _) => Some(Some(parse_count(lang, &c)?)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            let scale = match (print_scale, no_print_scale) {
                (Some(c), _) => Some(Some(parse_count(lang, &c)?)),
                (None, true) => Some(None),
                (None, false) => None,
            };
            let classes = classes.map(codes);
            let mut file = load(lang, &project)?;
            edit::set_course_details(&mut file.event, &name, classes, climb, scale)
                .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            say(lang, "cli-edit-course-set", &[("name", v(name))]);
        }
        CourseCmd::Number {
            project,
            name,
            after,
            direction,
            gap,
            clear,
        } => {
            // The command line counts from 1 as on the map; the library counts from 0.
            let shown = parse_count(lang, &after)?;
            let placement = match (direction, gap) {
                (Some(direction), Some(gap)) if !clear => {
                    Some(parse_placement(lang, &direction, &gap)?)
                }
                _ => None,
            };
            let bad_index = || {
                refused(
                    lang,
                    edit::EditError::BadIndex {
                        course: name.clone(),
                        index: shown as usize,
                    },
                )
            };
            if shown == 0 {
                return Err(bad_index());
            }
            let mut file = load(lang, &project)?;
            edit::set_course_number(
                &mut file.event,
                &name,
                shown as usize - 1,
                placement.clone(),
            )
            .map_err(|e| match e {
                edit::EditError::BadIndex { .. } => bad_index(),
                other => refused(lang, other),
            })?;
            save(lang, &mut file)?;
            let index = ("index", v(shown.to_string()));
            match placement {
                Some(p) => say(
                    lang,
                    "cli-edit-course-number-set",
                    &[
                        ("name", v(name)),
                        index,
                        ("direction", v(fmt(p.direction))),
                        ("gap", v(fmt(p.gap_mm))),
                    ],
                ),
                None => say(
                    lang,
                    "cli-edit-course-number-cleared",
                    &[("name", v(name)), index],
                ),
            }
        }
        CourseCmd::Numbers {
            project,
            name,
            overwrite,
        } => {
            let mut file = load(lang, &project)?;
            let doc = read_map(lang, &file)?;
            let placed = freemapper_courseplan::numbers::place_numbers(
                &mut file.event,
                &name,
                doc.georef.scale_denominator,
                map_kind(&doc),
                overwrite,
            )
            .map_err(|e| refused(lang, e))?;
            save(lang, &mut file)?;
            // Nothing placed although --overwrite would place course numbers again: say so.
            let replaceable = file
                .event
                .courses
                .iter()
                .find(|c| c.name == name)
                .map_or(0, |c| match &c.kind {
                    CourseKind::Individual { controls } => controls
                        .iter()
                        .filter(|cc| cc.number.is_some())
                        .filter(|cc| {
                            !file
                                .event
                                .controls
                                .iter()
                                .any(|c| c.code == cc.code && c.number.is_some())
                        })
                        .count(),
                    _ => 0,
                });
            if placed == 0 && !overwrite && replaceable > 0 {
                say(lang, "cli-edit-course-numbers-none", &[("name", v(name))]);
            } else {
                say(
                    lang,
                    "cli-edit-course-numbers",
                    &[("count", FluentValue::from(placed)), ("name", v(name))],
                );
            }
        }
    }
    Ok(())
}

/// One line per leg that lost its bends and taped flag (ruling R12).
fn say_cleared(lang: &Lang, cleared: &[edit::ClearedLeg]) {
    for leg in cleared {
        let name = ("name", v(leg.course.clone()));
        match leg.after {
            0 => say(lang, "cli-edit-start-leg-cleared", &[name]),
            after => say(
                lang,
                "cli-edit-leg-cleared",
                &[name, ("index", v(after.to_string()))],
            ),
        }
    }
}

fn say(lang: &Lang, id: &str, args: &[(&str, FluentValue)]) {
    println!("{}", text(lang, id, args));
}
