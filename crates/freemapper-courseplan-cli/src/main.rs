//! The `freemapper-courseplan` binary. Commands come with the milestones in the design
//! (`docs/superpowers/specs/2026-11-08-freemapper-courseplan-design.md`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use freemapper_courseplan::init::init_project;
use freemapper_courseplan::map::{MapKind, map_kind};
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan_model::{CourseKind, ProjectFile};

#[derive(Parser)]
#[command(
    name = "freemapper-courseplan",
    version,
    about = "Set orienteering courses on an OOM map"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start an event from a map and write its project file
    Init {
        /// The OOM map (sprint or schoolyard)
        map: PathBuf,
        /// Where to write the project file [default: <map folder>/<map name>.courseplan.toml]
        #[arg(long, value_name = "FILE")]
        project: Option<PathBuf>,
        /// Event title [default: `title` in freemapper.toml next to the map, else the map name]
        #[arg(long)]
        title: Option<String>,
    },
    /// Print the title, map, controls, courses and classes of a project file
    Info {
        /// The project file
        project: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init {
            map,
            project,
            title,
        } => init(&map, project, title),
        Command::Info { project } => info(&project),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
    }
}

fn init(map: &Path, project: Option<PathBuf>, title: Option<String>) -> Result<(), String> {
    let project = project.unwrap_or_else(|| default_project_path(map));
    let created = init_project(map, &project, title).map_err(|e| e.to_string())?;
    println!("{}", created.path.display());
    Ok(())
}

/// `<map folder>/<map stem>.courseplan.toml`
fn default_project_path(map: &Path) -> PathBuf {
    let stem = map.file_stem().unwrap_or_default().to_string_lossy();
    map.with_file_name(format!("{stem}.courseplan.toml"))
}

fn info(path: &Path) -> Result<(), String> {
    let project = ProjectFile::load(path).map_err(|e| e.to_string())?;
    let map_path = project.map_path();
    let doc = read_map_file(&map_path)
        .map_err(|e| format!("cannot read map {}: {e}", map_path.display()))?;
    let kind = match map_kind(&doc) {
        MapKind::Sprint => "sprint",
        MapKind::Schoolyard => "schoolyard",
        MapKind::Forest => "forest",
    };
    let event = &project.event;
    let count = |f: fn(&CourseKind) -> bool| event.courses.iter().filter(|c| f(&c.kind)).count();
    println!("title: {}", event.event.title);
    println!("map: {}", event.sources.map.display());
    println!("map kind: {kind}");
    println!("controls: {}", event.controls.len());
    println!(
        "courses: {} (individual {}, team {}, relay {}, score {})",
        event.courses.len(),
        count(|k| matches!(k, CourseKind::Individual { .. })),
        count(|k| matches!(k, CourseKind::Team { .. })),
        count(|k| matches!(k, CourseKind::Relay { .. })),
        count(|k| matches!(k, CourseKind::Score { .. })),
    );
    println!("classes: {}", event.classes.len());
    Ok(())
}
