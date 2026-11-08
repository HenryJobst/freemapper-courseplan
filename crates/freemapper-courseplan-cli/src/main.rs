//! The `freemapper-courseplan` binary. Commands come with the milestones in the design
//! (`docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
fn localized_command(lang: &Lang) -> clap::Command {
    let t = |id: &str| text(lang, id, &[]);
    let with_help = |arg: Arg, id: &str| arg.help(text(lang, id, &[]));
    Cli::command()
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
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .action(clap::ArgAction::Help)
                .global(true)
                .help(t("cli-arg-help")),
        )
        .arg(
            Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(t("cli-arg-version")),
        )
}

fn env_var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

fn print_error(lang: &Lang, message: &str) {
    eprintln!("{}: {message}", text(lang, "cli-error-prefix", &[]));
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
    let matches: ArgMatches = localized_command(&lang).get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    let result = match cli.command {
        Command::Init {
            map,
            project,
            title,
        } => init(&lang, &map, project, title),
        Command::Info { project } => info(&lang, &project),
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
}
