//! `export`: writes the courses for other programs, as IOF XML 3.0 or as a Purple Pen event (`.ppen`).
//! The same functions serve the CLI and the MCP tool.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use freemapper_courseplan::init::link_from_project;
use freemapper_courseplan::iof_xml::{IofXmlError, MapGrid, SkippedCourse, export_iof_xml};
use freemapper_courseplan::map::map_kind_for_file;
use freemapper_courseplan::omap::OmapError;
use freemapper_courseplan::omap::model::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::ppen_export::{
    Deletions, ExportNote, PpenExportError, Target, export_ppen_as, store_ids,
};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, ppen_export_error, text};
use freemapper_courseplan_model::{ProjectError, ProjectFile, write_atomic};

use crate::edit_cmd::{Failure, map_load_error};
use crate::import_cmd::{ReadFileError, read_course_file};

/// What an export wrote.
#[derive(Debug)]
pub(crate) struct ExportReport {
    pub(crate) courses: usize,
    pub(crate) skipped: Vec<SkippedCourse>,
    /// The controls carry a geographic `Position`; false when the map has no usable georeferencing.
    pub(crate) geo_positions: bool,
}

/// Why an export failed. The caller turns it into text, because the MCP server words project and
/// map errors differently from the CLI.
#[derive(Debug)]
pub(crate) enum ExportError {
    /// The project file cannot be read, or the target cannot be written.
    Project(ProjectError),
    /// The project's map is not an OOM file.
    NotMap(PathBuf),
    Map(PathBuf, OmapError),
    Course(IofXmlError),
    /// The export to the `.ppen` at the path failed; nothing was written.
    Ppen(PathBuf, Box<PpenExportError>),
    /// The existing Purple Pen event to patch cannot be read.
    ReadTarget(PathBuf, ReadFileError),
    /// The target is the project file or the map, which the export would overwrite.
    Overwrites(PathBuf),
    /// The `.ppen` at the path was written, but the project file with its Purple Pen ids (and the
    /// link to it) could not be saved.
    IdsNotStored(PathBuf, ProjectError),
}

impl ExportError {
    /// The text of the error; `project` and `map` word the errors of the project file and the map.
    pub(crate) fn message(
        self,
        lang: &Lang,
        project: impl FnOnce(ProjectError) -> String,
        map: impl FnOnce(&Path, OmapError) -> String,
    ) -> String {
        match self {
            ExportError::Project(e) => project(e),
            ExportError::NotMap(path) => text(
                lang,
                "mcp-error-not-map",
                &[("path", FluentValue::from(path.display().to_string()))],
            ),
            ExportError::Map(path, e) => map(&path, e),
            ExportError::Overwrites(path) => text(
                lang,
                "error-export-overwrites-input",
                &[("path", FluentValue::from(path.display().to_string()))],
            ),
            ExportError::Ppen(path, e) => ppen_export_error(&e, lang, &path.display().to_string()),
            ExportError::ReadTarget(path, e) => e.message(lang, &path),
            ExportError::IdsNotStored(path, e) => text(
                lang,
                "error-export-ppen-ids-not-stored",
                &[
                    ("path", FluentValue::from(path.display().to_string())),
                    ("source", FluentValue::from(project(e))),
                ],
            ),
            ExportError::Course(IofXmlError::Course { course, source }) => text(
                lang,
                "error-export-iof-xml-course",
                &[
                    ("course", FluentValue::from(course)),
                    ("source", FluentValue::from(source.localize(lang))),
                ],
            ),
        }
    }
}

/// The project, loaded, and its map, read, for an export to `target`. Refuses a target that is the
/// project file or the map, which the export would overwrite.
pub(crate) fn open_inputs(
    project: &Path,
    target: &Path,
) -> Result<(ProjectFile, MapDocument), ExportError> {
    let file = ProjectFile::load(project).map_err(ExportError::Project)?;
    let map_path = file.map_path();
    if same_file(target, project) || same_file(target, &map_path) {
        return Err(ExportError::Overwrites(target.to_path_buf()));
    }
    let is_oom = map_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ["omap", "xmap"].iter().any(|x| e.eq_ignore_ascii_case(x)));
    if !is_oom {
        return Err(ExportError::NotMap(map_path));
    }
    let doc = read_map_file(&map_path).map_err(|e| ExportError::Map(map_path.clone(), e))?;
    Ok((file, doc))
}

/// Writes the individual courses of `project` to `target` as IOF XML 3.0. Nothing is written when
/// a course cannot be measured; an existing `target` is replaced atomically.
pub(crate) fn export_iof_xml_file(
    project: &Path,
    target: &Path,
    created: &str,
) -> Result<ExportReport, ExportError> {
    let (file, doc) = open_inputs(project, target)?;
    let grid = MapGrid::of(&doc.georef).ok();
    let exported = export_iof_xml(
        &file.event,
        doc.georef.scale_denominator,
        map_kind_for_file(&doc, &file.map_path()),
        grid.as_ref(),
        created,
    )
    .map_err(ExportError::Course)?;
    write_atomic(target, &exported.xml).map_err(ExportError::Project)?;
    Ok(ExportReport {
        courses: file.event.courses.len() - exported.skipped.len(),
        skipped: exported.skipped,
        geo_positions: exported.geo_positions,
    })
}

/// What a `.ppen` export wrote.
#[derive(Debug)]
pub(crate) struct PpenReport {
    pub(crate) courses: usize,
    /// What the export left out, changed or kept.
    pub(crate) notes: Vec<ExportNote>,
    /// The target was an existing file, patched in place.
    pub(crate) patched: bool,
    /// The number of objects of the project that got a Purple Pen id for the first time. Only the
    /// linked file gives them.
    pub(crate) ids_stored: usize,
    /// The export made the target the project's linked file (`sources.ppen`).
    pub(crate) linked_now: bool,
}

impl PpenReport {
    /// The team, relay and score courses the export left out, as name and kind.
    pub(crate) fn skipped(&self) -> Vec<(&str, &str)> {
        self.notes
            .iter()
            .filter_map(|n| match n {
                ExportNote::CourseSkipped { course, kind } => Some((course.as_str(), *kind)),
                _ => None,
            })
            .collect()
    }
}

/// Writes the individual courses of `project` to the Purple Pen event `target`.
///
/// An existing `target` is the base: the export changes what the project changed and leaves the rest of
/// the file as it is, unless `new` asks for a file from scratch. A missing `target` is created.
///
/// The project's Purple Pen ids count only for its linked file, the one `sources.ppen` names. For a
/// project without `sources.ppen`, a new file (`target` did not exist, or `new` was given) becomes the
/// linked file *(controller's ruling)*: `sources.ppen` names it, relative to the project file. For the
/// linked file the ids the export assigned are stored in the project. Any other target, an existing file
/// or a new one while the project is linked elsewhere, gets no ids, and nothing is written back: the link
/// stays as it is.
///
/// The `.ppen` is written first, atomically, then the project (also atomically, and only when something
/// changed). Nothing is written when the export fails; when only the project save fails, the error says
/// that the `.ppen` was written.
pub(crate) fn export_ppen_file(
    project: &Path,
    target: &Path,
    new: bool,
    lang: &Lang,
) -> Result<PpenReport, ExportError> {
    let (mut file, doc) = open_inputs(project, target)?;
    let folder = project.parent().unwrap_or_else(|| Path::new(""));
    let linked_file = file
        .event
        .sources
        .ppen
        .as_ref()
        .is_some_and(|link| same_file(target, &folder.join(link)));
    let fresh = new || !target.exists();
    let base = if fresh {
        None
    } else {
        Some(
            read_course_file(target)
                .map_err(|e| ExportError::ReadTarget(target.to_path_buf(), e))?,
        )
    };
    // A new file links a project that has no link yet; one linked elsewhere keeps its link, and the
    // new file is treated like any other foreign target.
    let has_link = file.event.sources.ppen.is_some();
    let linked = linked_file || (fresh && !has_link);
    let map_path = link_from_project(target, &file.map_path())
        .map_or_else(
            || file.map_path().display().to_string(),
            |p| p.display().to_string(),
        )
        .replace('\\', "/");
    let lang_code = lang.code();
    let exported = export_ppen_as(
        &file.event,
        &doc,
        map_kind_for_file(&doc, &file.map_path()),
        Target {
            base: base.as_deref(),
            linked,
            map_path,
            lang: &lang_code,
            delete: Deletions::default(),
        },
    )
    .map_err(|e| ExportError::Ppen(target.to_path_buf(), Box::new(e)))?;
    write_atomic(target, &exported.text).map_err(ExportError::Project)?;
    let mut ids_stored = 0;
    let linked_now = linked && !linked_file;
    if linked {
        let mut event = file.event.clone();
        ids_stored = store_ids(&mut event, &exported.ids);
        if linked_now {
            event.sources.ppen =
                Some(link_from_project(project, target).unwrap_or_else(|| target.to_path_buf()));
        }
        if event != file.event {
            file.event = event;
            file.save()
                .map_err(|e| ExportError::IdsNotStored(target.to_path_buf(), e))?;
        }
    }
    Ok(PpenReport {
        courses: exported.courses,
        notes: exported.notes,
        patched: base.is_some(),
        ids_stored,
        linked_now,
    })
}

/// Whether two paths name the same file. A path that does not exist yet is resolved through its
/// parent directory; paths that cannot be resolved at all count as different.
fn same_file(a: &Path, b: &Path) -> bool {
    fn resolve(path: &Path) -> Option<PathBuf> {
        if let Ok(p) = path.canonicalize() {
            return Some(p);
        }
        let name = path.file_name()?;
        let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
        let parent = parent.unwrap_or_else(|| Path::new("."));
        Some(parent.canonicalize().ok()?.join(name))
    }
    matches!((resolve(a), resolve(b)), (Some(x), Some(y)) if x == y)
}

/// The `export` command.
pub(crate) fn export(
    lang: &Lang,
    project: &Path,
    iof_xml: Option<PathBuf>,
    ppen: Option<PathBuf>,
    new: bool,
) -> Result<(), Failure> {
    if iof_xml.is_none() && ppen.is_none() {
        return Err(Failure::Usage(text(lang, "export-nothing-to-do", &[])));
    }
    let failed = |e: ExportError| {
        Failure::Error(e.message(
            lang,
            |e| e.localize(lang),
            |p, e| map_load_error(lang, p, e),
        ))
    };
    let path_arg = |p: &Path| FluentValue::from(p.display().to_string());
    // The IOF XML is written first and would replace the Purple Pen file the second step reads.
    if let (Some(xml), Some(ppen)) = (&iof_xml, &ppen)
        && same_file(xml, ppen)
    {
        return Err(Failure::Error(text(
            lang,
            "error-export-same-target",
            &[("path", path_arg(ppen))],
        )));
    }
    if let Some(target) = iof_xml {
        let report = export_iof_xml_file(project, &target, &now_iso()).map_err(failed)?;
        for skipped in &report.skipped {
            println!(
                "{}",
                text(
                    lang,
                    "export-iof-xml-skipped",
                    &[
                        ("course", FluentValue::from(skipped.name.clone())),
                        (
                            "kind",
                            FluentValue::from(text(
                                lang,
                                &format!("word-kind-{}", skipped.kind),
                                &[]
                            )),
                        ),
                    ],
                )
            );
        }
        if !report.geo_positions {
            println!("{}", text(lang, "export-iof-xml-no-geo", &[]));
        }
        println!(
            "{}",
            text(
                lang,
                "export-iof-xml-written",
                &[
                    ("count", FluentValue::from(report.courses)),
                    ("path", path_arg(&target)),
                ],
            )
        );
    }
    if let Some(target) = ppen {
        let report = export_ppen_file(project, &target, new, lang).map_err(failed)?;
        let id = if report.patched {
            "export-ppen-patched"
        } else {
            "export-ppen-written"
        };
        println!(
            "{}",
            text(
                lang,
                id,
                &[
                    ("count", FluentValue::from(report.courses)),
                    ("path", path_arg(&target)),
                ],
            )
        );
        for note in &report.notes {
            println!(
                "{}",
                text(
                    lang,
                    "export-ppen-note-line",
                    &[("note", FluentValue::from(note.localize(lang)))],
                )
            );
        }
        if report.linked_now {
            println!(
                "{}",
                text(lang, "export-ppen-linked", &[("path", path_arg(&target))])
            );
        }
        if report.ids_stored > 0 {
            println!(
                "{}",
                text(
                    lang,
                    "export-ppen-ids-stored",
                    &[("count", FluentValue::from(report.ids_stored))],
                )
            );
        }
    }
    Ok(())
}

/// The current UTC date and time as `YYYY-MM-DDTHH:MM:SSZ`, for `createTime`. The standard library
/// has no time zones, so this is UTC and the `Z` says so.
pub(crate) fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    iso_from_unix(secs)
}

/// `YYYY-MM-DDTHH:MM:SSZ` for seconds since 1970-01-01 00:00:00 UTC.
fn iso_from_unix(secs: u64) -> String {
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let rest = secs % 86_400;
    // Civil date from days since 1970-01-01 (proleptic Gregorian calendar).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    use super::iso_from_unix;

    #[test]
    fn formats_known_instants() {
        assert_eq!(iso_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_from_unix(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso_from_unix(1_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(iso_from_unix(1_762_689_045), "2025-11-09T11:50:45Z");
        assert_eq!(iso_from_unix(4_102_444_799), "2099-12-31T23:59:59Z");
    }
}
