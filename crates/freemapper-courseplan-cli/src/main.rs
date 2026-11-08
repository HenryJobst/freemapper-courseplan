//! The `freemapper-courseplan` binary. Commands come with the milestones in the design
//! (`docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::error::{ContextKind, ContextValue, ErrorKind};
use clap::{Arg, ArgMatches, CommandFactory, FromArgMatches, Parser, Subcommand};
use freemapper_courseplan::init::init_project;
use freemapper_courseplan::map::{MapKind, map_kind};
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::text::{
    FluentValue, Lang, Localize, available, detect_lang, parse_lang_arg, text,
};
use freemapper_courseplan_model::{CourseKind, ProjectFile};

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
    },
    /// Print the title, map, controls, courses and classes of a project file
    Info {
        /// The project file
        project: PathBuf,
    },
    /// Show the help of a command
    Help {
        /// The command
        command: Option<String>,
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
        "{{about-with-newline}}\n{} {{usage}}\n\n{{all-args}}",
        t("cli-heading-usage")
    );
    // Clap wants 'static headings; the program builds its command once, so leaking is harmless.
    let leak = |id: &str| -> &'static str { Box::leak(t(id).into_boxed_str()) };
    // Clap adds the colon after a heading itself.
    let options = leak("cli-heading-options").trim_end_matches(':');
    let arguments = leak("cli-heading-arguments").trim_end_matches(':');
    let commands = leak("cli-heading-commands").trim_end_matches(':');
    let style = |cmd: clap::Command| {
        cmd.help_template(template.clone())
            .disable_help_subcommand(true)
            .subcommand_help_heading(commands)
            .mut_args(|a| {
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
        })
        .mut_subcommand("info", |s| {
            s.about(t("cli-help-info"))
                .mut_arg("project", |a| with_help(a, "cli-arg-info-project"))
        })
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
    let mut root = style(root);
    let names: Vec<String> = root
        .get_subcommands()
        .map(|c| c.get_name().to_owned())
        .collect();
    for name in names {
        root = root.mut_subcommand(name, style);
    }
    root
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
        ErrorKind::InvalidValue => (
            "cli-parse-invalid-value",
            vec![("arg", arg()), ("value", value())],
        ),
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
        } => init(&lang, &map, project, title),
        Command::Info { project } => info(&lang, &project),
        Command::Help { command: name } => {
            command.build();
            let target = name
                .as_deref()
                .and_then(|n| command.find_subcommand_mut(n))
                .map(|c| c.print_help());
            let _ = match target {
                Some(result) => result,
                None => command.print_help(),
            };
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            print_error(&lang, &message);
            ExitCode::from(1)
        }
    }
}

fn init(
    lang: &Lang,
    map: &Path,
    project: Option<PathBuf>,
    title: Option<String>,
) -> Result<(), String> {
    let project = project.unwrap_or_else(|| default_project_path(map));
    let created = init_project(map, &project, title).map_err(|e| e.localize(lang))?;
    println!("{}", created.path.display());
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
    let number = |id: &str, n: usize| line(id, &[("count", FluentValue::from(n.to_string()))]);
    value("cli-info-title", event.event.title.clone());
    value("cli-info-map", event.sources.map.display().to_string());
    value("cli-info-map-kind", text(lang, kind, &[]));
    number("cli-info-controls", event.controls.len());
    let n = |n: usize| FluentValue::from(n.to_string());
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

    #[test]
    fn ids_used_in_the_cli_exist() {
        let code = include_str!("main.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();
        let ids: Vec<&str> = code
            .split('"')
            .skip(1)
            .step_by(2)
            .filter(|lit| {
                ["error-", "cli-", "word-"]
                    .iter()
                    .any(|p| lit.starts_with(p))
                    && lit.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            })
            .collect();
        assert!(ids.len() > 25, "scanner found only {}", ids.len());
        for lang in available() {
            for id in &ids {
                let out = text(&lang, id, &[]);
                assert_ne!(&out, id, "{}: {id} is missing", lang.code());
                assert!(!out.contains(['\u{2068}', '\u{2069}']), "{id}");
            }
        }
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
