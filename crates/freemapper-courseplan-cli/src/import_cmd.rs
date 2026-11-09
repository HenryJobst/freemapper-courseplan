//! `import --ppen` and `init --ppen`: take over a Purple Pen event. The same functions serve the
//! CLI and the MCP tool.

use std::path::{Path, PathBuf};

use freemapper_courseplan::init::{InitError, create_project, link_from_project, new_event};
use freemapper_courseplan::omap::OmapError;
use freemapper_courseplan::omap::model::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::ppen_import::{ImportNote, PpenImportError, import_ppen};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{Event, ProjectError, ProjectFile};

use crate::edit_cmd::{Failure, map_load_error};

/// What an import brought in, with the notes on what it left out or changed.
#[derive(Debug)]
pub(crate) struct ImportReport {
    pub(crate) controls: usize,
    pub(crate) courses: usize,
    pub(crate) notes: Vec<ImportNote>,
}

/// Why an import failed. The caller turns it into text, because the MCP server words project and
/// map errors differently from the CLI.
#[derive(Debug)]
pub(crate) enum ImportError {
    /// The project file cannot be read, or written, or the result is not valid.
    Project(ProjectError),
    /// The project's map is not an OOM file.
    NotMap(PathBuf),
    Map(PathBuf, OmapError),
    /// The Purple Pen file cannot be read from disk.
    ReadPpen(PathBuf, std::io::Error),
    Ppen(PpenImportError),
    /// The project has content and `replace` was not given.
    NeedsReplace(PathBuf),
    Init(InitError),
}

impl ImportError {
    /// The text of the error; `project` and `map` word the errors of the project file and the map.
    pub(crate) fn message(
        self,
        lang: &Lang,
        project: impl FnOnce(ProjectError) -> String,
        map: impl FnOnce(&Path, OmapError) -> String,
    ) -> String {
        let path_arg = |p: &Path| FluentValue::from(p.display().to_string());
        match self {
            ImportError::Project(e) => project(e),
            ImportError::NotMap(path) => {
                text(lang, "error-import-not-map", &[("path", path_arg(&path))])
            }
            ImportError::Map(path, e) => map(&path, e),
            ImportError::ReadPpen(path, e) => text(
                lang,
                "error-import-ppen-read",
                &[
                    ("path", path_arg(&path)),
                    ("source", FluentValue::from(e.to_string())),
                ],
            ),
            ImportError::Ppen(e) => e.localize(lang),
            ImportError::NeedsReplace(path) => text(
                lang,
                "import-ppen-needs-replace",
                &[("path", path_arg(&path))],
            ),
            ImportError::Init(e) => e.localize(lang),
        }
    }
}

/// The `sources.ppen` link: `ppen` relative to the project's folder. The fallback, `ppen` as given (relative
/// to the working folder), is reached only when `ppen` cannot be resolved; the caller has just read it, so
/// this is a safeguard.
fn link_for(project: &Path, ppen: &Path) -> PathBuf {
    link_from_project(project, ppen).unwrap_or_else(|| ppen.to_path_buf())
}

fn read_oom(map_path: &Path) -> Result<MapDocument, ImportError> {
    let is_oom = map_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ["omap", "xmap"].iter().any(|x| e.eq_ignore_ascii_case(x)));
    if !is_oom {
        return Err(ImportError::NotMap(map_path.to_path_buf()));
    }
    read_map_file(map_path).map_err(|e| ImportError::Map(map_path.to_path_buf(), e))
}

fn read_ppen_file(ppen: &Path) -> Result<Vec<u8>, ImportError> {
    std::fs::read(ppen).map_err(|e| ImportError::ReadPpen(ppen.to_path_buf(), e))
}

/// Puts the content of the Purple Pen event `bytes` into `event`: controls, special objects and courses,
/// and the link `sources.ppen`. Classes and accepted findings stay. The caller validates the event.
fn bring_in(
    event: &mut Event,
    doc: &MapDocument,
    bytes: &[u8],
    link: PathBuf,
    replace: bool,
    project: &Path,
) -> Result<ImportReport, ImportError> {
    let has_content =
        !event.controls.is_empty() || !event.courses.is_empty() || !event.specials.is_empty();
    if has_content && !replace {
        return Err(ImportError::NeedsReplace(project.to_path_buf()));
    }
    let import = import_ppen(bytes, doc).map_err(ImportError::Ppen)?;
    let report = ImportReport {
        controls: import.controls.len(),
        courses: import.courses.len(),
        notes: import.notes,
    };
    event.controls = import.controls;
    event.specials = import.specials;
    event.courses = import.courses;
    event.sources.ppen = Some(link);
    Ok(report)
}

/// Takes over the Purple Pen event `ppen` into the event of `project` and saves the project file.
/// The file is saved only when the result passes the model's validation; otherwise it stays as it was.
pub(crate) fn import_ppen_file(
    project: &Path,
    ppen: &Path,
    replace: bool,
) -> Result<ImportReport, ImportError> {
    let mut file = ProjectFile::load(project).map_err(ImportError::Project)?;
    let doc = read_oom(&file.map_path())?;
    let bytes = read_ppen_file(ppen)?;
    let link = link_for(project, ppen);
    let report = bring_in(&mut file.event, &doc, &bytes, link, replace, project)?;
    file.save().map_err(ImportError::Project)?;
    Ok(report)
}

/// Starts an event from `map` and the Purple Pen event `ppen`, and writes the project file. Nothing is
/// written when a step fails or the result is invalid.
pub(crate) fn init_from_ppen(
    map: &Path,
    project: &Path,
    title: Option<String>,
    ppen: &Path,
) -> Result<(ProjectFile, ImportReport), ImportError> {
    let mut event = new_event(map, project, title).map_err(ImportError::Init)?;
    let doc = read_oom(map)?;
    let bytes = read_ppen_file(ppen)?;
    let link = link_for(project, ppen);
    let report = bring_in(&mut event, &doc, &bytes, link, false, project)?;
    let file = create_project(project, event).map_err(ImportError::Init)?;
    Ok((file, report))
}

/// Prints the done line and the notes of an import.
pub(crate) fn print_report(lang: &Lang, report: &ImportReport, ppen: &Path) {
    println!(
        "{}",
        text(
            lang,
            "import-ppen-done",
            &[
                ("controls", FluentValue::from(report.controls)),
                ("courses", FluentValue::from(report.courses)),
                ("path", FluentValue::from(ppen.display().to_string())),
            ],
        )
    );
    for note in &report.notes {
        println!(
            "{}",
            text(
                lang,
                "import-ppen-note-line",
                &[("note", FluentValue::from(note.localize(lang)))],
            )
        );
    }
}

/// The error of an import as the CLI words it.
pub(crate) fn failure(lang: &Lang, error: ImportError) -> Failure {
    Failure::Error(error.message(
        lang,
        |e| e.localize(lang),
        |p, e| map_load_error(lang, p, e),
    ))
}

/// The `import` command.
pub(crate) fn import(
    lang: &Lang,
    project: &Path,
    ppen: &Path,
    replace: bool,
) -> Result<(), Failure> {
    let report = import_ppen_file(project, ppen, replace).map_err(|e| failure(lang, e))?;
    print_report(lang, &report, ppen);
    Ok(())
}
