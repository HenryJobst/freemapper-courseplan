//! `export`: writes the courses for other programs. IOF XML 3.0 is the first format. The same
//! function serves the CLI and the MCP tool.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use freemapper_courseplan::iof_xml::{IofXmlError, SkippedCourse, export_iof_xml};
use freemapper_courseplan::map::map_kind;
use freemapper_courseplan::omap::OmapError;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{ProjectError, ProjectFile, write_atomic};

use crate::edit_cmd::{Failure, map_load_error};

/// What an export wrote.
#[derive(Debug)]
pub(crate) struct ExportReport {
    pub(crate) courses: usize,
    pub(crate) skipped: Vec<SkippedCourse>,
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

/// Writes the individual courses of `project` to `target` as IOF XML 3.0. Nothing is written when
/// a course cannot be measured; an existing `target` is replaced atomically.
pub(crate) fn export_iof_xml_file(
    project: &Path,
    target: &Path,
    created: &str,
) -> Result<ExportReport, ExportError> {
    let file = ProjectFile::load(project).map_err(ExportError::Project)?;
    let map_path = file.map_path();
    let is_oom = map_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| ["omap", "xmap"].iter().any(|x| e.eq_ignore_ascii_case(x)));
    if !is_oom {
        return Err(ExportError::NotMap(map_path));
    }
    let doc = read_map_file(&map_path).map_err(|e| ExportError::Map(map_path.clone(), e))?;
    let exported = export_iof_xml(
        &file.event,
        doc.georef.scale_denominator,
        map_kind(&doc),
        created,
    )
    .map_err(ExportError::Course)?;
    write_atomic(target, &exported.xml).map_err(ExportError::Project)?;
    Ok(ExportReport {
        courses: file.event.courses.len() - exported.skipped.len(),
        skipped: exported.skipped,
    })
}

/// The `export` command.
pub(crate) fn export(lang: &Lang, project: &Path, iof_xml: Option<PathBuf>) -> Result<(), Failure> {
    let Some(target) = iof_xml else {
        return Err(Failure::Usage(text(
            lang,
            "export-iof-xml-nothing-to-do",
            &[],
        )));
    };
    let report = export_iof_xml_file(project, &target, &now_iso()).map_err(|e| {
        Failure::Error(e.message(
            lang,
            |e| e.localize(lang),
            |p, e| map_load_error(lang, p, e),
        ))
    })?;
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
                        FluentValue::from(text(lang, &format!("word-kind-{}", skipped.kind), &[])),
                    ),
                ],
            )
        );
    }
    println!(
        "{}",
        text(
            lang,
            "export-iof-xml-written",
            &[
                ("count", FluentValue::from(report.courses)),
                ("path", FluentValue::from(target.display().to_string())),
            ],
        )
    );
    Ok(())
}

/// The current UTC date and time as `YYYY-MM-DDTHH:MM:SS`, for `createTime`. The standard library
/// has no time zones, so this is UTC and not local time.
pub(crate) fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    iso_from_unix(secs)
}

/// `YYYY-MM-DDTHH:MM:SS` for seconds since 1970-01-01 00:00:00 UTC.
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
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}",
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
        assert_eq!(iso_from_unix(0), "1970-01-01T00:00:00");
        assert_eq!(iso_from_unix(951_782_400), "2000-02-29T00:00:00");
        assert_eq!(iso_from_unix(1_000_000_000), "2001-09-09T01:46:40");
        assert_eq!(iso_from_unix(1_762_689_045), "2025-11-09T11:50:45");
        assert_eq!(iso_from_unix(4_102_444_799), "2099-12-31T23:59:59");
    }
}
