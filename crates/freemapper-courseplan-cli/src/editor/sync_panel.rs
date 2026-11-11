//! Sync with the linked Purple Pen file from the page (ADR 0054).
//!
//! The page names no file: the sync uses the project's `sources.ppen`, as the CLI does, with the link checks
//! of the MCP sync. Conflict choices go through the operation `resolve_sync_conflict`; this route only runs
//! the sync, as a dry run or for real.

use axum::http::StatusCode;
use freemapper_courseplan::sync::SyncError;
use freemapper_courseplan::sync::decide::Take;
use freemapper_courseplan_model::{ProjectError, Side};
use serde::Deserialize;
use serde_json::json;

use super::EditorState;
use super::api::{Reply, conflict, msg, reload, tidy};
use super::exchange::{current_for_write, fail, linked_ppen};
use crate::sync_cmd::{SyncCmdError, sync_file, sync_json};

/// The body of `POST api/sync`. `take` applies to every open conflict of this call.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyncRequest {
    dry_run: bool,
    take: Option<TakeSide>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum TakeSide {
    Ours,
    Theirs,
}

/// `POST api/sync`. A real sync names the revision the page saw and clears the undo history when it wrote.
pub(crate) fn sync(st: &mut EditorState, revision: Option<&str>, body: &[u8]) -> Reply {
    let lang = st.lang.clone();
    let Ok(request) = serde_json::from_slice::<SyncRequest>(body) else {
        return fail(StatusCode::BAD_REQUEST, &lang, "editor-sync-bad-request");
    };
    if request.dry_run {
        reload(st);
        if let Err(e) = &st.loaded.file {
            let e = e.clone();
            return conflict(st, e);
        }
    } else if let Err(reply) = current_for_write(st, revision) {
        return reply;
    }
    let target = match linked_ppen(st, "editor-sync-no-link") {
        Ok(target) => target,
        Err(reply) => return reply,
    };
    let take: Vec<Take> = request
        .take
        .map(|side| match side {
            TakeSide::Ours => Take::All(Side::Ours),
            TakeSide::Theirs => Take::All(Side::Theirs),
        })
        .into_iter()
        .collect();
    let result = sync_file(&st.project, request.dry_run, &take);
    // A real sync may have written the project: read it again whatever the result was.
    reload(st);
    match result {
        Ok(outcome) => {
            if outcome.written {
                st.history.clear();
            }
            let mut answer = sync_json(&outcome, &lang);
            answer["revision"] = json!(st.loaded.revision.0);
            (StatusCode::OK, answer)
        }
        Err(e) => failure(st, e, &target),
    }
}

/// The status and text of a failed sync. Parser errors and the lists of an invalid merge quote the Purple Pen
/// file, so they get fixed texts; every path shrinks to its file name.
fn failure(st: &EditorState, error: SyncCmdError, target: &std::path::Path) -> Reply {
    let lang = &st.lang;
    match &error {
        SyncCmdError::Project(ProjectError::ChangedMeanwhile { .. })
        | SyncCmdError::Partial {
            source: ProjectError::ChangedMeanwhile { .. },
            ..
        } => return conflict(st, msg(lang, "editor-op-stale")),
        SyncCmdError::Project(ProjectError::Read { .. } | ProjectError::Write { .. }) => {
            return fail(StatusCode::INTERNAL_SERVER_ERROR, lang, "editor-io-failed");
        }
        SyncCmdError::Sync(_, inner) => match inner.as_ref() {
            SyncError::Theirs(_) | SyncError::Base(_) | SyncError::Export(_) => {
                return fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    lang,
                    "editor-sync-ppen-invalid",
                );
            }
            SyncError::Invalid(_) => {
                return fail(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    lang,
                    "editor-sync-invalid",
                );
            }
            _ => {}
        },
        _ => {}
    }
    let changed_meanwhile = matches!(error, SyncCmdError::ChangedMeanwhile(_));
    let partial = matches!(error, SyncCmdError::Partial { .. });
    let text = tidy(&error.message(lang), &[&st.project, target]);
    if changed_meanwhile || partial {
        return (
            StatusCode::CONFLICT,
            json!({ "error": text, "revision": st.loaded.revision.0 }),
        );
    }
    (StatusCode::UNPROCESSABLE_ENTITY, json!({ "error": text }))
}
