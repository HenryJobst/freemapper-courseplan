//! The `freemapper-courseplan` binary. Commands come with the milestones in the design
//! (`docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`).

mod check_cmd;
mod edit_cmd;
mod export_cmd;
mod import_cmd;
mod mcp;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use check_cmd::{EventCmd, FindingCmd};
use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, ArgMatches, CommandFactory, FromArgMatches, Parser, Subcommand};
use edit_cmd::{ClassCmd, ControlCmd, CourseCmd, Failure, SpecialCmd};
use freemapper_courseplan::course::{control_numbers, course_length_m};
use freemapper_courseplan::init::init_project;
use freemapper_courseplan::map::{MapKind, map_kind};
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::text::{
    FluentValue, Lang, Localize, available, detect_lang, parse_lang_arg, text,
};
use freemapper_courseplan_model::{ControlKind, CourseKind, ProjectFile};

// The help texts come from the Fluent catalog, not from doc comments: `localized_command` sets
// `about` and `help` on the command that `Cli::command()` derives, once the language is known.
#[derive(Parser)]
#[command(name = "freemapper-courseplan", version)]
struct Cli {
    /// Language of messages and help (de or en); see `localized_command`.
    #[arg(long, global = true, value_name = "LANG")]
    lang: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start an event from a map and write its project file
    Init {
        /// The OOM map
        map: PathBuf,
        /// Where to write the project file
        #[arg(long, value_name = "FILE")]
        project: Option<PathBuf>,
        /// Event title
        #[arg(long)]
        title: Option<String>,
        /// Start from a Purple Pen event
        #[arg(long, value_name = "FILE")]
        ppen: Option<PathBuf>,
    },
    /// Print the title, map, controls, courses and classes of a project file
    Info {
        /// The project file
        project: PathBuf,
    },
    /// Take over a Purple Pen event
    Import {
        /// The project file
        project: PathBuf,
        /// The Purple Pen event
        #[arg(long, value_name = "FILE")]
        ppen: PathBuf,
        /// Replace what the project has
        #[arg(long)]
        replace: bool,
    },
    /// Write the courses to a file for other programs
    Export {
        /// The project file
        project: PathBuf,
        /// Write the individual courses as IOF XML 3.0 CourseData
        #[arg(long, value_name = "FILE")]
        iof_xml: Option<PathBuf>,
    },
    /// Add, move, describe, number and remove controls
    Control {
        #[command(subcommand)]
        action: ControlCmd,
    },
    /// Add and remove special objects
    Special {
        #[command(subcommand)]
        action: SpecialCmd,
    },
    /// Add and remove classes
    Class {
        #[command(subcommand)]
        action: ClassCmd,
    },
    /// Add, change, number and remove courses
    Course {
        #[command(subcommand)]
        action: CourseCmd,
    },
    /// Check the courses against the rule profile
    Check {
        /// The project file
        project: PathBuf,
        /// Also list accepted findings and skipped rules
        #[arg(long)]
        all: bool,
    },
    /// Accept findings as intended exceptions, or remove such an acceptance
    Finding {
        #[command(subcommand)]
        action: FindingCmd,
    },
    /// Set event-wide settings: the rule profile
    Event {
        #[command(subcommand)]
        action: EventCmd,
    },
    /// Serve course setting to AI assistants over MCP on stdin and stdout
    Mcp,
    /// Show the help of a command
    Help {
        /// The command, with its subcommands: `help control add`
        command: Vec<String>,
    },
}

/// The `--lang` value in the arguments: `--lang de` or `--lang=de`, the last one wins. The scan
/// runs before clap parses, because the help text itself needs the language.
fn lang_from_args(args: &[OsString]) -> Option<String> {
    let mut found = None;
    let mut iter = args.iter().skip(1).map(|a| a.to_string_lossy());
    while let Some(arg) = iter.next() {
        if arg == "--" {
            break;
        }
        if arg == "--lang" {
            found = iter.next().map(|v| v.into_owned());
        } else if let Some(value) = arg.strip_prefix("--lang=") {
            found = Some(value.to_owned());
        }
    }
    found
}

/// The command from the derive with its help texts taken from the catalog in `lang`.
///
/// Clap's own texts are replaced too: the headings come from the catalog through `help_template`,
/// `subcommand_help_heading` and `Arg::help_heading`, and the automatic `help` subcommand is
/// disabled in favour of the derived `Help` variant. Remaining English: the placeholders in the
/// usage line (`[OPTIONS]`, `<COMMAND>`, `<MAP>`), and the text of rare parse errors
/// (see `parse_error_message`).
fn localized_command(lang: &Lang) -> clap::Command {
    let t = |id: &str| text(lang, id, &[]);
    let with_help = |arg: Arg, id: &str| arg.help(text(lang, id, &[]));
    let template = format!(
        "{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}{{after-help}}",
        t("cli-heading-usage")
    );
    // Clap wants 'static headings; the program builds its command once, so leaking is harmless.
    let leak = |id: &str| -> &'static str { Box::leak(t(id).into_boxed_str()) };
    // Clap adds the colon after a heading itself.
    let options = leak("cli-heading-options").trim_end_matches(':');
    let arguments = leak("cli-heading-arguments").trim_end_matches(':');
    let commands = leak("cli-heading-commands").trim_end_matches(':');
    let possible = t("cli-help-possible-values");
    let style = |cmd: clap::Command| {
        // Clap sorts the help by display order and then by name, so the arguments get their
        // declaration order here; global arguments (--lang, --help) come after a command's own.
        let mut order = 0;
        cmd.help_template(template.clone())
            .disable_help_subcommand(true)
            .subcommand_help_heading(commands)
            .mut_args(|a| {
                order += 1;
                let offset = if a.is_global_set() { 1000 } else { 0 };
                let a = a.display_order(offset + order);
                // Clap's "[possible values: …]" is English; the catalog's words replace it.
                let takes_values = a.get_action().takes_values();
                let values: Vec<String> = a
                    .get_possible_values()
                    .iter()
                    .filter(|_| takes_values)
                    .filter(|v| !v.is_hide_set())
                    .map(|v| v.get_name().to_owned())
                    .collect();
                let a = if values.is_empty() {
                    a
                } else {
                    let help = a.get_help().map(ToString::to_string).unwrap_or_default();
                    let text = format!("{help} [{possible}: {}]", values.join(", "));
                    a.hide_possible_values(true).help(text)
                };
                if a.is_positional() {
                    a.help_heading(arguments)
                } else {
                    a.help_heading(options)
                }
            })
    };
    let help_flag = Arg::new("help")
        .short('h')
        .long("help")
        .action(clap::ArgAction::Help)
        .global(true)
        .help(t("cli-arg-help"));
    let root = Cli::command()
        .about(t("cli-about"))
        .mut_arg("lang", |a| with_help(a, "cli-arg-lang"))
        .mut_subcommand("init", |s| {
            s.about(t("cli-help-init"))
                .mut_arg("map", |a| with_help(a, "cli-arg-init-map"))
                .mut_arg("project", |a| with_help(a, "cli-arg-init-project"))
                .mut_arg("title", |a| with_help(a, "cli-arg-init-title"))
                .mut_arg("ppen", |a| with_help(a, "cli-arg-init-ppen"))
        })
        .mut_subcommand("import", |s| {
            s.about(t("cli-help-import"))
                .mut_arg("project", |a| with_help(a, "cli-arg-import-project"))
                .mut_arg("ppen", |a| with_help(a, "cli-arg-import-ppen"))
                .mut_arg("replace", |a| with_help(a, "cli-arg-import-replace"))
        })
        .mut_subcommand("info", |s| {
            s.about(t("cli-help-info"))
                .mut_arg("project", |a| with_help(a, "cli-arg-info-project"))
        })
        .mut_subcommand("export", |s| {
            s.about(t("cli-help-export"))
                .mut_arg("project", |a| with_help(a, "cli-arg-export-project"))
                .mut_arg("iof_xml", |a| with_help(a, "cli-arg-export-iof-xml"))
        })
        .mut_subcommand("check", |s| {
            s.about(t("cli-help-check"))
                .after_help(t("cli-help-check-exit-codes"))
                .mut_arg("project", |a| with_help(a, "cli-arg-info-project"))
                .mut_arg("all", |a| with_help(a, "cli-arg-check-all"))
        })
        .mut_subcommand("mcp", |s| s.about(t("cli-help-mcp")))
        .mut_subcommand("help", |s| {
            s.about(t("cli-help-help"))
                .mut_arg("command", |a| with_help(a, "cli-arg-help-command"))
        })
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(help_flag)
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(t("cli-arg-version")),
        );
    let root = edit_cmd::localize_groups(root, lang);
    style_tree(root, &style)
}

/// Applies `style` to the command and, below it, to every subcommand.
fn style_tree(cmd: clap::Command, style: &dyn Fn(clap::Command) -> clap::Command) -> clap::Command {
    let mut cmd = style(cmd);
    let names: Vec<String> = cmd
        .get_subcommands()
        .map(|c| c.get_name().to_owned())
        .collect();
    for name in names {
        cmd = cmd.mut_subcommand(name, |s| style_tree(s, style));
    }
    cmd
}

/// Text of a context value (an argument name, a value, a list of them).
fn context(error: &clap::Error, kind: ContextKind) -> String {
    match error.get(kind) {
        Some(ContextValue::String(s)) => s.clone(),
        Some(ContextValue::Strings(list)) => list.join(", "),
        Some(ContextValue::StyledStr(s)) => s.to_string(),
        Some(ContextValue::StyledStrs(list)) => list
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        Some(ContextValue::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// The catalog text for the common parse errors; `None` for the rare kinds, which keep clap's
/// English text.
fn parse_error_message(lang: &Lang, error: &clap::Error) -> Option<String> {
    let arg = || FluentValue::from(context(error, ContextKind::InvalidArg));
    let value = || FluentValue::from(context(error, ContextKind::InvalidValue));
    let (id, args): (&str, Vec<(&str, FluentValue)>) = match error.kind() {
        ErrorKind::UnknownArgument => ("cli-parse-unknown-argument", vec![("arg", arg())]),
        ErrorKind::MissingRequiredArgument => ("cli-parse-missing-required", vec![("args", arg())]),
        ErrorKind::InvalidValue => match context(error, ContextKind::ValidValue) {
            values if values.is_empty() => (
                "cli-parse-invalid-value",
                vec![("arg", arg()), ("value", value())],
            ),
            values => (
                "cli-parse-invalid-value-choices",
                vec![
                    ("arg", arg()),
                    ("value", value()),
                    ("values", FluentValue::from(values)),
                ],
            ),
        },
        ErrorKind::InvalidSubcommand => {
            // Clap stores the unknown word under its own key.
            let word = match context(error, ContextKind::InvalidSubcommand) {
                word if word.is_empty() => context(error, ContextKind::InvalidArg),
                word => word,
            };
            (
                "cli-parse-invalid-subcommand",
                vec![("value", FluentValue::from(word))],
            )
        }
        ErrorKind::MissingSubcommand | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            ("cli-parse-missing-subcommand", vec![])
        }
        ErrorKind::TooManyValues => (
            "cli-parse-too-many-values",
            vec![("arg", arg()), ("value", value())],
        ),
        ErrorKind::ArgumentConflict => (
            "cli-parse-argument-conflict",
            vec![
                ("arg", arg()),
                (
                    "other",
                    FluentValue::from(context(error, ContextKind::PriorArg)),
                ),
            ],
        ),
        _ => return None,
    };
    Some(text(lang, id, &args))
}

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn print_error(lang: &Lang, message: &str) {
    eprintln!("{}: {message}", text(lang, "cli-error-prefix", &[]));
}

/// Prints help and version as clap does; shows other errors from the catalog where it has a
/// text, else with clap's own message. Usage errors exit with 2.
fn report_parse_error(lang: &Lang, error: &clap::Error) -> ExitCode {
    if !error.use_stderr() {
        let _ = error.print();
        return ExitCode::SUCCESS;
    }
    match parse_error_message(lang, error) {
        Some(message) => {
            print_error(lang, &message);
            eprintln!("\n{}", text(lang, "cli-parse-hint", &[]));
        }
        None => {
            let _ = error.print();
        }
    }
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().collect();
    let arg = lang_from_args(&args);
    if let Some(value) = &arg
        && parse_lang_arg(value).is_err()
    {
        // The argument is unusable, so the message speaks the language of the environment.
        let lang = detect_lang(None, &env_var);
        let codes: Vec<String> = available().iter().map(Lang::code).collect();
        let message = text(
            &lang,
            "cli-error-unknown-lang",
            &[
                ("value", FluentValue::from(value.clone())),
                ("available", FluentValue::from(codes.join(", "))),
            ],
        );
        print_error(&lang, &message);
        return ExitCode::from(2);
    }
    let lang = detect_lang(arg.as_deref(), &env_var);
    let mut command = localized_command(&lang);
    let matches: ArgMatches = match command.clone().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => return report_parse_error(&lang, &error),
    };
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => return report_parse_error(&lang, &error),
    };
    let result = match cli.command {
        Command::Init {
            map,
            project,
            title,
            ppen,
        } => init(&lang, &map, project, title, ppen),
        Command::Import {
            project,
            ppen,
            replace,
        } => import_cmd::import(&lang, &project, &ppen, replace),
        Command::Info { project } => info(&lang, &project).map_err(Failure::Error),
        Command::Control { action } => edit_cmd::control(&lang, action),
        Command::Special { action } => edit_cmd::special(&lang, action),
        Command::Class { action } => edit_cmd::class(&lang, action),
        Command::Course { action } => edit_cmd::course(&lang, action),
        Command::Export { project, iof_xml } => export_cmd::export(&lang, &project, iof_xml),
        Command::Check { project, all } => check_cmd::check(&lang, project, all),
        Command::Finding { action } => check_cmd::finding(&lang, action),
        Command::Event { action } => check_cmd::event(&lang, action),
        Command::Mcp => mcp::run(lang.clone()).map_err(Failure::Error),
        Command::Help { command: path } => {
            command.build();
            let mut found: &clap::Command = &command;
            let unknown = path.iter().find(|name| match found.find_subcommand(name) {
                Some(sub) => {
                    found = sub;
                    false
                }
                None => true,
            });
            match unknown {
                Some(name) => Err(Failure::Usage(text(
                    &lang,
                    "cli-parse-invalid-subcommand",
                    &[("value", FluentValue::from(name.clone()))],
                ))),
                None => {
                    let mut target = &mut command;
                    for name in &path {
                        target = target.find_subcommand_mut(name).expect("found above");
                    }
                    let _ = target.print_help();
                    Ok(())
                }
            }
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Error(message)) => {
            print_error(&lang, &message);
            ExitCode::from(1)
        }
        Err(Failure::Usage(message)) => {
            print_error(&lang, &message);
            ExitCode::from(2)
        }
        Err(Failure::Findings) => ExitCode::from(1),
    }
}

fn init(
    lang: &Lang,
    map: &Path,
    project: Option<PathBuf>,
    title: Option<String>,
    ppen: Option<PathBuf>,
) -> Result<(), Failure> {
    let project = project.unwrap_or_else(|| default_project_path(map));
    let Some(ppen) = ppen else {
        let created =
            init_project(map, &project, title).map_err(|e| Failure::Error(e.localize(lang)))?;
        println!("{}", created.path.display());
        return Ok(());
    };
    let (created, report) = import_cmd::init_from_ppen(map, &project, title, &ppen)
        .map_err(|e| import_cmd::failure(lang, e))?;
    println!("{}", created.path.display());
    import_cmd::print_report(lang, &report, &ppen);
    Ok(())
}

/// `<map folder>/<map stem>.courseplan.toml`
fn default_project_path(map: &Path) -> PathBuf {
    let stem = map.file_stem().unwrap_or_default().to_string_lossy();
    map.with_file_name(format!("{stem}.courseplan.toml"))
}

fn info(lang: &Lang, path: &Path) -> Result<(), String> {
    let project = ProjectFile::load(path).map_err(|e| e.localize(lang))?;
    let map_path = project.map_path();
    let doc = read_map_file(&map_path).map_err(|e| {
        text(
            lang,
            "error-init-map",
            &[
                ("path", FluentValue::from(map_path.display().to_string())),
                ("source", FluentValue::from(e.localize(lang))),
            ],
        )
    })?;
    let kind = match map_kind(&doc) {
        MapKind::Sprint => "word-map-sprint",
        MapKind::Schoolyard => "word-map-schoolyard",
        MapKind::Forest => "word-map-forest",
    };
    let event = &project.event;
    let count = |f: fn(&CourseKind) -> bool| event.courses.iter().filter(|c| f(&c.kind)).count();
    let line = |id: &str, args: &[(&str, FluentValue)]| println!("{}", text(lang, id, args));
    let value = |id: &str, v: String| line(id, &[("value", FluentValue::from(v))]);
    let number = |id: &str, n: usize| line(id, &[("count", FluentValue::from(n))]);
    value("cli-info-title", event.event.title.clone());
    value("cli-info-map", event.sources.map.display().to_string());
    value("cli-info-map-kind", text(lang, kind, &[]));
    let profile = FluentValue::from(event.event.rule_profile.clone());
    if event.event.rule_overlays.is_empty() {
        line("cli-info-rule-profile", &[("value", profile)]);
    } else {
        let list = FluentValue::from(event.event.rule_overlays.join(", "));
        let count = FluentValue::from(event.event.rule_overlays.len());
        line(
            "cli-info-rule-profile-overlays",
            &[("value", profile), ("list", list), ("count", count)],
        );
    }
    // Crossing points get no number, so they are not counted as controls, as in the course lines.
    let crossings = event
        .controls
        .iter()
        .filter(|c| c.kind == ControlKind::CrossingPoint)
        .count();
    line(
        "cli-info-controls",
        &[
            ("count", FluentValue::from(event.controls.len() - crossings)),
            ("crossings", FluentValue::from(crossings)),
        ],
    );
    let n = |n: usize| FluentValue::from(n);
    line(
        "cli-info-courses",
        &[
            ("total", n(event.courses.len())),
            (
                "individual",
                n(count(|k| matches!(k, CourseKind::Individual { .. }))),
            ),
            ("team", n(count(|k| matches!(k, CourseKind::Team { .. })))),
            ("relay", n(count(|k| matches!(k, CourseKind::Relay { .. })))),
            ("score", n(count(|k| matches!(k, CourseKind::Score { .. })))),
        ],
    );
    number("cli-info-classes", event.classes.len());
    let scale = doc.georef.scale_denominator;
    for course in &event.courses {
        let name = || FluentValue::from(course.name.clone());
        if !matches!(course.kind, CourseKind::Individual { .. }) {
            let kind = match course.kind {
                CourseKind::Individual { .. } => "word-kind-individual",
                CourseKind::Team { .. } => "word-kind-team",
                CourseKind::Relay { .. } => "word-kind-relay",
                CourseKind::Score { .. } => "word-kind-score",
            };
            line(
                "cli-info-course-unmeasured",
                &[
                    ("name", name()),
                    ("kind", FluentValue::from(text(lang, kind, &[]))),
                ],
            );
            continue;
        }
        let metres = course_length_m(event, course, scale).map_err(|e| e.localize(lang))?;
        // Fluent's NUMBER() does not localize separators in fluent-bundle 0.16. So the program
        // formats with one decimal and swaps the separator from the catalog; input keeps the dot.
        let km =
            format!("{:.1}", metres / 1000.0).replace('.', &text(lang, "word-decimal-point", &[]));
        let controls = control_numbers(event, course).iter().flatten().count();
        let climb = course
            .climb_m
            .map_or_else(|| "\u{2013}".to_owned(), |m| m.to_string());
        line(
            "cli-info-course",
            &[
                ("name", name()),
                ("controls", n(controls)),
                ("km", FluentValue::from(km)),
                ("climb", FluentValue::from(climb)),
            ],
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<OsString> {
        list.iter().map(OsString::from).collect()
    }

    #[test]
    fn finds_lang_before_parsing() {
        assert_eq!(
            lang_from_args(&args(&["p", "--lang", "de", "init"])),
            Some("de".into())
        );
        assert_eq!(
            lang_from_args(&args(&["p", "init", "--lang=en"])),
            Some("en".into())
        );
        assert_eq!(
            lang_from_args(&args(&["p", "--lang", "de", "--lang", "en"])),
            Some("en".into())
        );
        assert_eq!(
            lang_from_args(&args(&["p", "init", "--", "--lang", "de"])),
            None
        );
        assert_eq!(lang_from_args(&args(&["p", "info"])), None);
        assert_eq!(lang_from_args(&args(&["p", "--lang"])), None);
    }

    #[test]
    fn localized_commands_are_valid_in_every_language() {
        for lang in available() {
            localized_command(&lang).debug_assert();
        }
    }

    /// The CLI source files that the id scanner reads, by path below `src/`.
    const SCANNED_SOURCES: &[(&str, &str)] = &[
        ("main.rs", include_str!("main.rs")),
        ("edit_cmd.rs", include_str!("edit_cmd.rs")),
        ("check_cmd.rs", include_str!("check_cmd.rs")),
        ("export_cmd.rs", include_str!("export_cmd.rs")),
        ("import_cmd.rs", include_str!("import_cmd.rs")),
        ("mcp.rs", include_str!("mcp.rs")),
        ("mcp/params.rs", include_str!("mcp/params.rs")),
    ];

    #[test]
    fn every_cli_module_is_scanned_for_message_ids() {
        fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let rel = path.strip_prefix(root).unwrap();
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        walk(&root, &root, &mut files);
        let missing: Vec<&String> = files
            .iter()
            .filter(|f| {
                let name = f.rsplit('/').next().unwrap_or(f);
                !(name == "tests.rs" || name.ends_with("_tests.rs"))
                    && !SCANNED_SOURCES.iter().any(|(n, _)| n == f)
            })
            .collect();
        assert!(
            missing.is_empty(),
            "add to SCANNED_SOURCES in main.rs: {missing:?}"
        );
    }

    #[test]
    fn ids_used_in_the_cli_exist() {
        let ids: Vec<&str> = SCANNED_SOURCES
            .iter()
            .filter_map(|(_, s)| s.split("#[cfg(test)]").next())
            .flat_map(|code| code.split('"').skip(1).step_by(2))
            .filter(|lit| {
                ["error-", "cli-", "word-", "mcp-", "check-"]
                    .iter()
                    .any(|p| lit.starts_with(p))
                    && lit.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                    && !lit.ends_with('-')
            })
            .collect();
        assert!(
            ids.contains(&"cli-check-summary"),
            "the scanner sees check_cmd.rs"
        );
        assert!(ids.len() > 25, "scanner found only {}", ids.len());
        for lang in available() {
            for id in &ids {
                let out = text(&lang, id, &[]);
                assert_ne!(&out, id, "{}: {id} is missing", lang.code());
                assert!(!out.contains(['\u{2068}', '\u{2069}']), "{id}");
            }
        }
    }

    /// Every command and argument of the edit groups has a catalog text in every language.
    #[test]
    fn edit_commands_have_help_texts() {
        fn walk(cmd: &clap::Command, path: &str, seen: &mut usize) {
            for sub in cmd.get_subcommands() {
                let path = format!("{path} {}", sub.get_name());
                let about = sub.get_about().map(ToString::to_string).unwrap_or_default();
                assert!(
                    !about.is_empty() && !about.starts_with("cli-"),
                    "{path}: {about}"
                );
                for arg in sub.get_arguments() {
                    let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
                    assert!(
                        !help.is_empty() && !help.starts_with("cli-"),
                        "{path} {}: {help}",
                        arg.get_id()
                    );
                    *seen += 1;
                }
                walk(sub, &path, seen);
            }
        }
        for lang in available() {
            let command = localized_command(&lang);
            let mut seen = 0;
            for group in edit_cmd::GROUPS {
                walk(
                    &clap::Command::new("x")
                        .subcommand(command.find_subcommand(group).unwrap().clone()),
                    "",
                    &mut seen,
                );
            }
            assert!(seen > 30, "only {seen} arguments");
        }
    }

    /// A line needs two points, so the CLI never prints the singular; the catalog still has it.
    #[test]
    fn point_count_has_a_singular() {
        let de = available().into_iter().find(|l| l.code() == "de").unwrap();
        let args = [
            ("kind", FluentValue::from("Pflichtstrecke")),
            ("id", FluentValue::from("M1")),
            ("count", FluentValue::from(1)),
        ];
        assert_eq!(
            text(&de, "cli-edit-special-added-points", &args),
            "Pflichtstrecke M1 mit 1 Punkt angelegt"
        );
        assert_eq!(
            text(&Lang::english(), "cli-edit-special-added-points", &args),
            "added Pflichtstrecke M1 with 1 point"
        );
    }

    #[test]
    fn parse_errors_keep_their_values() {
        let lang = available().into_iter().find(|l| l.code() == "de").unwrap();
        type Case<'a> = (ErrorKind, Vec<(ContextKind, &'a str)>, Vec<&'a str>);
        let cases: Vec<Case> = vec![
            (
                ErrorKind::UnknownArgument,
                vec![(ContextKind::InvalidArg, "--bogus")],
                vec!["--bogus"],
            ),
            (
                ErrorKind::MissingRequiredArgument,
                vec![(ContextKind::InvalidArg, "<MAP>")],
                vec!["<MAP>"],
            ),
            (
                ErrorKind::InvalidValue,
                vec![
                    (ContextKind::InvalidArg, "--lang"),
                    (ContextKind::InvalidValue, "xx"),
                ],
                vec!["--lang", "xx"],
            ),
            (
                ErrorKind::InvalidSubcommand,
                vec![(ContextKind::InvalidSubcommand, "frob")],
                vec!["frob"],
            ),
            (
                ErrorKind::InvalidValue,
                vec![
                    (ContextKind::InvalidArg, "--kind"),
                    (ContextKind::InvalidValue, "wasser"),
                    (ContextKind::ValidValue, "start, water"),
                ],
                vec!["--kind", "wasser", "Mögliche Werte: start, water"],
            ),
            (ErrorKind::MissingSubcommand, vec![], vec![]),
            (
                ErrorKind::TooManyValues,
                vec![
                    (ContextKind::InvalidArg, "<MAP>"),
                    (ContextKind::InvalidValue, "extra"),
                ],
                vec!["<MAP>", "extra"],
            ),
            (
                ErrorKind::ArgumentConflict,
                vec![
                    (ContextKind::InvalidArg, "--a"),
                    (ContextKind::PriorArg, "--b"),
                ],
                vec!["--a", "--b"],
            ),
        ];
        for (kind, context, expected) in cases {
            let mut error = clap::Error::new(kind);
            for (key, value) in context {
                error.insert(key, ContextValue::String(value.to_owned()));
            }
            let message = parse_error_message(&lang, &error).expect("mapped kind");
            for value in expected {
                assert!(message.contains(value), "{kind:?}: {message}");
            }
            assert!(!message.contains("unexpected"), "{kind:?}: {message}");
        }
    }
}
