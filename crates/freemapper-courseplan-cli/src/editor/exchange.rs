//! Export and import from the menu (ADR 0053): IOF XML and Purple Pen files, down- and uploaded.
//!
//! The page never names a file. The server writes only the project's own linked `.ppen`, by the rules of the
//! CLI export; everything else is made in memory and handed over, or read from the request body.

use std::path::{Path, PathBuf};

use axum::http::StatusCode;
use freemapper_courseplan::ppen_export::ExportNote;
use freemapper_courseplan::text::{Lang, Localize};
use freemapper_courseplan_model::ProjectError;
use serde_json::{Value, json};

use super::EditorState;
use super::api::{Reply, conflict, msg, reload, tidy};
use crate::edit_cmd::map_load_error;
use crate::export_cmd::{
    ExportError, ExportReport, PpenReport, export_ppen_file, iof_xml_text, now_iso,
    ppen_download_text,
};
use crate::import_cmd::{ImportError, import_ppen_bytes};

/// The most characters of the project's file name that reach a download's name.
const MAX_STEM_CHARS: usize = 80;

/// The name part of a download: the project file's name without `.courseplan.toml` or `.toml`, every
/// character outside `[A-Za-z0-9._-]` replaced by `_`, and never starting with a dot.
pub(crate) fn download_stem(project: &Path) -> String {
    let name = project
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = name
        .strip_suffix(".courseplan.toml")
        .or_else(|| name.strip_suffix(".toml"))
        .unwrap_or(&name);
    let mut out: String = stem
        .chars()
        .take(MAX_STEM_CHARS)
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.starts_with('.') {
        out.replace_range(..1, "_");
    }
    if out.is_empty() {
        "event".to_owned()
    } else {
        out
    }
}

pub(super) fn fail(status: StatusCode, lang: &Lang, id: &str) -> Reply {
    (status, json!({ "error": msg(lang, id) }))
}

/// The status and text of a failed export. The text names files by their names only.
fn export_failure(st: &EditorState, error: ExportError, paths: &[&Path]) -> Reply {
    let lang = &st.lang;
    match &error {
        ExportError::IdsNotStored(_, ProjectError::ChangedMeanwhile { .. })
        | ExportError::Project(ProjectError::ChangedMeanwhile { .. }) => {
            return conflict(st, msg(lang, "editor-op-stale"));
        }
        ExportError::Project(ProjectError::Read { .. } | ProjectError::Write { .. }) => {
            return fail(StatusCode::INTERNAL_SERVER_ERROR, lang, "editor-io-failed");
        }
        _ => {}
    }
    let text = error.message(
        lang,
        |e| e.localize(lang),
        |p, e| map_load_error(lang, p, e),
    );
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        json!({ "error": tidy(&text, paths) }),
    )
}

/// The skipped courses and notes of an export, as the page shows them.
fn ppen_report_json(report: &PpenReport, lang: &Lang) -> Value {
    let skipped: Vec<Value> = report
        .skipped()
        .into_iter()
        .map(|(name, kind)| json!({ "name": name, "kind": kind }))
        .collect();
    let notes: Vec<String> = report
        .notes
        .iter()
        .map(|n: &ExportNote| n.localize(lang))
        .collect();
    json!({
        "courses": report.courses,
        "skipped": skipped,
        "notes": notes,
        "patched": report.patched,
        "ids_stored": report.ids_stored,
        "linked_now": report.linked_now,
    })
}

/// What `GET api/export/iof-xml` hands over: the text, the report and the file name.
pub(crate) struct XmlDownload {
    pub xml: String,
    pub report: ExportReport,
    pub file_name: String,
}

/// `GET api/export/iof-xml`: made in memory from the project file as it is on disk.
pub(crate) fn iof_xml(project: &Path, lang: &Lang) -> Result<XmlDownload, Reply> {
    let st_like = |error: ExportError| {
        let text = error.message(
            lang,
            |e| e.localize(lang),
            |p, e| map_load_error(lang, p, e),
        );
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": tidy(&text, &[project]) }),
        )
    };
    let (xml, report) = iof_xml_text(project, &now_iso()).map_err(st_like)?;
    Ok(XmlDownload {
        xml,
        report,
        file_name: format!("{}.xml", download_stem(project)),
    })
}

/// Reads the project again and checks that the page saw its current revision.
pub(super) fn current_for_write(st: &mut EditorState, revision: Option<&str>) -> Result<(), Reply> {
    let lang = st.lang.clone();
    let Some(revision) = revision else {
        return Err(fail(
            StatusCode::BAD_REQUEST,
            &lang,
            "editor-op-no-revision",
        ));
    };
    reload(st);
    if let Err(e) = &st.loaded.file {
        let e = e.clone();
        return Err(conflict(st, e));
    }
    if st.loaded.revision.0 != revision {
        return Err(conflict(st, msg(&lang, "editor-op-stale")));
    }
    Ok(())
}

/// The linked `.ppen` of the project: it must end in `.ppen` and must not be a symbolic link (the rules of
/// the MCP sync).
pub(super) fn linked_ppen(st: &EditorState, no_link: &str) -> Result<PathBuf, Reply> {
    let lang = &st.lang;
    let Ok(file) = &st.loaded.file else {
        return Err(fail(
            StatusCode::INTERNAL_SERVER_ERROR,
            lang,
            "editor-io-failed",
        ));
    };
    let Some(link) = &file.event.sources.ppen else {
        return Err(fail(StatusCode::UNPROCESSABLE_ENTITY, lang, no_link));
    };
    let target = st.project.parent().unwrap_or(Path::new("")).join(link);
    let is_ppen = target
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ppen"));
    let symlink = std::fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink());
    if !is_ppen || symlink {
        return Err(fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            lang,
            "editor-export-link-refused",
        ));
    }
    Ok(target)
}

/// `POST api/export/ppen`. `linked` writes the project's linked file by the CLI rules, clears the undo history
/// and needs the current revision; otherwise a new file is made in memory and handed over inside the answer.
pub(crate) fn export_ppen(st: &mut EditorState, to: &str, revision: Option<&str>) -> Reply {
    match to {
        "linked" => export_linked(st, revision),
        "download" => {
            let name = format!("{}.ppen", download_stem(&st.project));
            match ppen_download_text(&st.project, &st.lang, &name) {
                Ok((text, report)) => {
                    let mut answer = ppen_report_json(&report, &st.lang);
                    answer["file"] = json!(name);
                    answer["text"] = json!(text);
                    (StatusCode::OK, answer)
                }
                Err(e) => export_failure(st, e, &[&st.project]),
            }
        }
        _ => fail(
            StatusCode::BAD_REQUEST,
            &st.lang,
            "editor-export-bad-request",
        ),
    }
}

fn export_linked(st: &mut EditorState, revision: Option<&str>) -> Reply {
    if let Err(reply) = current_for_write(st, revision) {
        return reply;
    }
    let target = match linked_ppen(st, "editor-export-no-link") {
        Ok(target) => target,
        Err(reply) => return reply,
    };
    let result = export_ppen_file(&st.project, &target, false, &st.lang);
    // The export may have changed the project file (ids, link): read it again whatever the result was.
    reload(st);
    match result {
        Ok(report) => {
            st.history.clear();
            let mut answer = ppen_report_json(&report, &st.lang);
            answer["revision"] = json!(st.loaded.revision.0);
            (StatusCode::OK, answer)
        }
        Err(e) => export_failure(st, e, &[&st.project, &target]),
    }
}

/// `POST api/import/ppen`: takes over an uploaded Purple Pen file. `sources.ppen` stays as it is. The undo
/// history is cleared.
pub(crate) fn import_ppen(
    st: &mut EditorState,
    revision: Option<&str>,
    replace: bool,
    bytes: &[u8],
) -> Reply {
    if let Err(reply) = current_for_write(st, revision) {
        return reply;
    }
    let lang = st.lang.clone();
    let result = import_ppen_bytes(&st.project, bytes, replace);
    reload(st);
    let unprocessable = |id: &str| fail(StatusCode::UNPROCESSABLE_ENTITY, &lang, id);
    match result {
        Ok(report) => {
            st.history.clear();
            let notes: Vec<String> = report.notes.iter().map(|n| n.localize(&lang)).collect();
            (
                StatusCode::OK,
                json!({
                    "revision": st.loaded.revision.0,
                    "controls": report.controls,
                    "courses": report.courses,
                    "notes": notes,
                }),
            )
        }
        Err(ImportError::NeedsReplace(_)) => unprocessable("editor-import-needs-replace"),
        // The reader's own text can quote the file; the page gets a fixed one.
        Err(ImportError::Ppen(freemapper_courseplan::ppen_import::PpenImportError::Read(_))) => {
            unprocessable("editor-import-invalid")
        }
        Err(ImportError::Project(ProjectError::ChangedMeanwhile { .. })) => {
            conflict(st, msg(&lang, "editor-op-stale"))
        }
        Err(ImportError::Project(ProjectError::Read { .. } | ProjectError::Write { .. })) => {
            fail(StatusCode::INTERNAL_SERVER_ERROR, &lang, "editor-io-failed")
        }
        Err(other) => {
            let text = other.message(
                &lang,
                |e| e.localize(&lang),
                |p, e| map_load_error(&lang, p, e),
            );
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({ "error": tidy(&text, &[&st.project]) }),
            )
        }
    }
}

/// The query of `api/import/ppen`: `replace=true|false`, at most once, nothing else.
pub(crate) fn import_query(raw: Option<&str>) -> Option<bool> {
    let mut replace = None;
    for pair in raw.unwrap_or("").split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=')?;
        let value = match value {
            "true" => true,
            "false" => false,
            _ => return None,
        };
        if key != "replace" || replace.replace(value).is_some() {
            return None;
        }
    }
    Some(replace.unwrap_or(false))
}
