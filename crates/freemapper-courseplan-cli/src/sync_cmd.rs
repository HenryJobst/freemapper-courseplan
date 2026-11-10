//! `sync`: merges the project with its linked Purple Pen file (ADR 0040). The same functions serve the CLI
//! and, later, the MCP tools.

use std::path::{Path, PathBuf};

use freemapper_courseplan::map::map_kind_for_file;
use freemapper_courseplan::omap::OmapError;
use freemapper_courseplan::sync::base::{
    BaseState, base_path, base_state, fnv1a64, strip_absolute_path,
};
use freemapper_courseplan::sync::decide::Take;
use freemapper_courseplan::sync::merge3::Pick;
use freemapper_courseplan::sync::{SyncError, SyncInput, SyncPlan, SyncReport, sync_ppen};
use freemapper_courseplan::text::{FluentValue, Lang, Localize, sync_error, sync_value, text};
use freemapper_courseplan_model::{
    ProjectError, ProjectFile, Side, SyncDecision, SyncState, write_atomic,
};

use crate::edit_cmd::{Failure, map_load_error};
use crate::export_cmd::{ExportError, open_inputs};
use crate::import_cmd::{ReadFileError, read_course_file};

/// What a sync did, or with `dry_run` would do.
#[derive(Debug)]
pub(crate) struct SyncOutcome {
    pub(crate) report: SyncReport,
    /// The sync changed files: the project and the base copy, and the `.ppen` when `ppen_changed`.
    pub(crate) written: bool,
    pub(crate) ppen_changed: bool,
    pub(crate) base: BaseState,
    pub(crate) dry_run: bool,
    /// The number of `KEY=SIDE` decisions now stored in the project because conflicts remain.
    pub(crate) decisions: usize,
    /// The linked file, for the messages.
    pub(crate) ppen: PathBuf,
    pub(crate) project: PathBuf,
}

/// Why a sync failed. The caller turns it into text.
#[derive(Debug)]
pub(crate) enum SyncCmdError {
    /// The project has no `sources.ppen`.
    NoLink(PathBuf),
    /// The file `sources.ppen` names does not exist.
    LinkMissing(PathBuf),
    Read(PathBuf, ReadFileError),
    Project(ProjectError),
    NotMap(PathBuf),
    Map(PathBuf, OmapError),
    /// The link points at the project file or its map.
    Overwrites(PathBuf),
    Sync(PathBuf, SyncError),
    /// A `--take KEY=SIDE` names no open conflict.
    TakeKey(String),
    /// An error variant that cannot occur here; the word says which.
    Internal(&'static str),
    /// The linked file changed between the first read and the write (Purple Pen saved meanwhile).
    ChangedMeanwhile(PathBuf),
    /// The first write succeeded and a later one failed; `sync` finishes the job.
    Partial {
        written: Vec<PathBuf>,
        source: ProjectError,
    },
}

impl SyncCmdError {
    pub(crate) fn message(self, lang: &Lang) -> String {
        self.message_with(lang, &|e| e.localize(lang), &|path, e| {
            map_load_error(lang, path, e)
        })
    }

    /// The text of the error; `project` and `map` word the errors of the project file and the map
    /// (the MCP server redacts them).
    pub(crate) fn message_with(
        self,
        lang: &Lang,
        project: &dyn Fn(ProjectError) -> String,
        map: &dyn Fn(&Path, OmapError) -> String,
    ) -> String {
        let path_arg = |p: &Path| FluentValue::from(p.display().to_string());
        match self {
            SyncCmdError::NoLink(path) => {
                text(lang, "error-sync-no-link", &[("path", path_arg(&path))])
            }
            SyncCmdError::LinkMissing(path) => text(
                lang,
                "error-sync-link-missing",
                &[("path", path_arg(&path))],
            ),
            SyncCmdError::Read(path, e) => e.message(lang, &path),
            SyncCmdError::Project(e) => project(e),
            SyncCmdError::NotMap(path) => {
                text(lang, "error-import-not-map", &[("path", path_arg(&path))])
            }
            SyncCmdError::Map(path, e) => map(&path, e),
            SyncCmdError::Overwrites(path) => text(
                lang,
                "error-export-overwrites-input",
                &[("path", path_arg(&path))],
            ),
            SyncCmdError::Sync(path, e) => sync_error(&e, lang, &path.display().to_string()),
            SyncCmdError::TakeKey(key) => text(
                lang,
                "error-sync-take-unknown-key",
                &[("key", FluentValue::from(key))],
            ),
            SyncCmdError::Internal(what) => text(
                lang,
                "error-sync-internal",
                &[("what", FluentValue::from(what))],
            ),
            SyncCmdError::ChangedMeanwhile(path) => text(
                lang,
                "error-sync-changed-meanwhile",
                &[("path", path_arg(&path))],
            ),
            SyncCmdError::Partial { written, source } => text(
                lang,
                "error-sync-partial",
                &[
                    (
                        "written",
                        FluentValue::from(
                            written
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                    ),
                    ("source", FluentValue::from(project(source))),
                ],
            ),
        }
    }
}

impl From<ExportError> for SyncCmdError {
    fn from(e: ExportError) -> Self {
        match e {
            ExportError::Project(e) => SyncCmdError::Project(e),
            ExportError::NotMap(path) => SyncCmdError::NotMap(path),
            ExportError::Map(path, e) => SyncCmdError::Map(path, e),
            ExportError::Overwrites(path) => SyncCmdError::Overwrites(path),
            // `open_inputs` returns none of these; each still maps to a clear error rather than a panic.
            ExportError::Course(_) => SyncCmdError::Internal("course"),
            ExportError::Ppen(..) => SyncCmdError::Internal("ppen"),
            ExportError::ReadTarget(path, e) => SyncCmdError::Read(path, e),
            ExportError::IdsNotStored(_, e) => SyncCmdError::Project(e),
        }
    }
}

/// The test hook of debug builds: rewrites the linked file between the first read and the
/// re-read, as Purple Pen saving at that moment would. It does not exist in release builds.
#[cfg(debug_assertions)]
fn test_touch(ppen: &Path) {
    use std::io::Write;
    if std::env::var_os("FREEMAPPER_SYNC_TEST_TOUCH").is_some()
        && let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(ppen)
    {
        let _ = file.write_all(b"\n");
    }
}

#[cfg(not(debug_assertions))]
fn test_touch(_ppen: &Path) {}

/// The base copy, or `None` when the file does not exist.
fn read_base(path: &Path) -> Result<Option<Vec<u8>>, SyncCmdError> {
    match read_course_file(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(ReadFileError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(SyncCmdError::Read(path.to_path_buf(), e)),
    }
}

/// Syncs `project` with its linked Purple Pen file.
///
/// Order of the writes (D13): the `.ppen`, then the project with the hash of the new base copy and no
/// stored decisions, then the base copy. Just before the first write the `.ppen` is read again; if it
/// changed, nothing is written. A failure after the first write is a [`SyncCmdError::Partial`]; running
/// `sync` again finishes the job, because identical changes on both sides are no conflict and a base copy
/// that lags behind is repaired.
///
/// Open conflicts write nothing, except the `KEY=SIDE` decisions in the project file. With `dry_run`
/// nothing is written at all.
pub(crate) fn sync_file(
    project: &Path,
    dry_run: bool,
    take: &[Take],
) -> Result<SyncOutcome, SyncCmdError> {
    sync_inner(project, dry_run, take, false)
}

/// Stores the decision for one open conflict in the project file, with the fingerprint of the conflict as
/// it is now, and nothing else. The key must name a conflict of the sync as it would run without any
/// stored decision; a decision stored before for the key is replaced.
pub(crate) fn store_decision(project: &Path, key: &str, side: Side) -> Result<(), SyncCmdError> {
    sync_inner(project, false, &[Take::One(key.to_owned(), side)], true).map(drop)
}

fn sync_inner(
    project: &Path,
    dry_run: bool,
    take: &[Take],
    store_only: bool,
) -> Result<SyncOutcome, SyncCmdError> {
    let first = ProjectFile::load(project).map_err(SyncCmdError::Project)?;
    let Some(link) = first.event.sources.ppen.clone() else {
        return Err(SyncCmdError::NoLink(project.to_path_buf()));
    };
    let folder = project.parent().unwrap_or_else(|| Path::new(""));
    let ppen_path = folder.join(&link);
    if !ppen_path.exists() {
        return Err(SyncCmdError::LinkMissing(ppen_path));
    }
    let (mut file, doc) = open_inputs(project, &ppen_path)?;
    let theirs =
        read_course_file(&ppen_path).map_err(|e| SyncCmdError::Read(ppen_path.clone(), e))?;
    let base_file = base_path(project);
    let stored_base = read_base(&base_file)?;
    let state = base_state(
        file.event.sync.as_ref(),
        &link,
        stored_base.as_deref(),
        &theirs,
    );
    let (base, base_note) = state.resolve(stored_base.as_deref(), &theirs);
    let kind = map_kind_for_file(&doc, &file.map_path());
    // The event as it is without any stored decision.
    let bare = {
        let mut bare = file.event.clone();
        if let Some(sync) = &mut bare.sync {
            sync.decisions.clear();
        }
        bare
    };
    let run = |event, take: &[Take]| {
        sync_ppen(SyncInput {
            project: event,
            base: base.as_deref(),
            theirs: &theirs,
            map: &doc,
            kind,
            take,
            base_note: base_note.clone(),
        })
        .map_err(|e| SyncCmdError::Sync(ppen_path.clone(), e))
    };
    let plan = if store_only {
        // Only the decisions are wanted: take nothing, so the plan is always the conflict branch.
        match run(&bare, &[])? {
            plan @ SyncPlan::Conflicts(_) => plan,
            SyncPlan::Ready { .. } => {
                let key = take.iter().find_map(|t| match t {
                    Take::One(key, _) => Some(key.clone()),
                    Take::All(_) => None,
                });
                return Err(SyncCmdError::TakeKey(key.unwrap_or_default()));
            }
        }
    } else {
        run(&file.event, take)?
    };

    let outcome = |report, written, ppen_changed, decisions| SyncOutcome {
        report,
        written,
        ppen_changed,
        base: state,
        dry_run,
        decisions,
        ppen: ppen_path.clone(),
        project: project.to_path_buf(),
    };

    match plan {
        SyncPlan::Conflicts(report) => {
            let keyed: Vec<&Take> = take.iter().filter(|t| matches!(t, Take::One(..))).collect();
            if dry_run || keyed.is_empty() {
                return Ok(outcome(report, false, false, 0));
            }
            // The conflicts as they are without any stored decision give each key its fingerprint.
            let all = match run(&bare, &[])? {
                SyncPlan::Conflicts(all) => all.conflicts,
                SyncPlan::Ready { .. } => Vec::new(),
            };
            let old = file
                .event
                .sync
                .as_ref()
                .map_or(&[][..], |s| s.decisions.as_slice());
            // A stored decision stays while its conflict has the same values.
            let mut decisions: Vec<SyncDecision> = old
                .iter()
                .filter(|d| {
                    all.iter()
                        .any(|c| c.key == d.key && c.fingerprint == d.fingerprint)
                })
                .cloned()
                .collect();
            for t in keyed {
                let Take::One(key, side) = t else { continue };
                let Some(conflict) = all.iter().find(|c| &c.key == key) else {
                    return Err(SyncCmdError::TakeKey(key.clone()));
                };
                decisions.retain(|d| &d.key != key);
                decisions.push(SyncDecision {
                    key: key.clone(),
                    take: *side,
                    fingerprint: conflict.fingerprint.clone(),
                });
            }
            let count = decisions.len();
            let mut state = file.event.sync.clone().unwrap_or_default();
            state.decisions = decisions;
            if file.event.sync.as_ref() != Some(&state) {
                file.event.sync = Some(state);
                file.save().map_err(SyncCmdError::Project)?;
            }
            Ok(outcome(report, false, false, count))
        }
        SyncPlan::Ready {
            project: mut event,
            ppen,
            base: base_text,
            report,
        } => {
            if dry_run {
                return Ok(outcome(report, false, ppen.is_some(), 0));
            }
            // The base copy keeps no `absolute-path`, and its hash covers the stored bytes (D1).
            let stored = strip_absolute_path(base_text.as_bytes());
            let stored = String::from_utf8(stored)
                .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
            event.sync = Some(SyncState {
                ppen: Some(link),
                base_fnv1a64: Some(fnv1a64(stored.as_bytes())),
                decisions: Vec::new(),
            });

            test_touch(&ppen_path);
            let again = read_course_file(&ppen_path)
                .map_err(|e| SyncCmdError::Read(ppen_path.clone(), e))?;
            if again != theirs {
                return Err(SyncCmdError::ChangedMeanwhile(ppen_path));
            }

            let mut written = Vec::new();
            let partial = |written: &[PathBuf], source| {
                if written.is_empty() {
                    SyncCmdError::Project(source)
                } else {
                    SyncCmdError::Partial {
                        written: written.to_vec(),
                        source,
                    }
                }
            };
            let ppen_changed = ppen.is_some();
            if let Some(text) = &ppen {
                write_atomic(&ppen_path, text).map_err(SyncCmdError::Project)?;
                written.push(ppen_path.clone());
            }
            file.event = event;
            file.save().map_err(|e| partial(&written, e))?;
            written.push(project.to_path_buf());
            write_atomic(&base_file, &stored).map_err(|e| partial(&written, e))?;
            Ok(outcome(report, true, ppen_changed, 0))
        }
    }
}

fn side_word(side: Pick) -> &'static str {
    match side {
        Pick::Ours => "ours",
        Pick::Theirs => "theirs",
        _ => "both",
    }
}

/// The result as JSON. Keys, values of changes and conflicts stay English; the notes are text in `lang`.
pub(crate) fn sync_json(o: &SyncOutcome, lang: &Lang) -> serde_json::Value {
    let base = match o.base {
        BaseState::Use => "use",
        BaseState::Repair => "repair",
        BaseState::First => "first",
        BaseState::Ignore(_) => "ignore",
    };
    serde_json::json!({
        "written": o.written,
        "dry_run": o.dry_run,
        "ppen_changed": o.ppen_changed,
        "base": base,
        "changes": o.report.changes.iter().map(|c| serde_json::json!({
            "key": c.key,
            "side": side_word(c.side),
            "value": c.value,
        })).collect::<Vec<_>>(),
        "conflicts": o.report.conflicts.iter().map(|c| serde_json::json!({
            "key": c.key,
            "base": c.base,
            "ours": c.ours,
            "theirs": c.theirs,
        })).collect::<Vec<_>>(),
        "notes": o.report.notes.iter().map(|n| n.localize(lang)).collect::<Vec<_>>(),
        "stale": o.report.stale,
    })
}

/// Prints the outcome as text.
fn print_outcome(o: &SyncOutcome, lang: &Lang) {
    let path = FluentValue::from(o.ppen.display().to_string());
    let line = |id: &str, args: &[(&str, FluentValue)]| println!("{}", text(lang, id, args));
    let count = |side: Pick| {
        o.report
            .changes
            .iter()
            .filter(|c| side_word(c.side) == side_word(side))
            .count()
    };
    for note in &o.report.notes {
        line(
            "export-ppen-note-line",
            &[("note", FluentValue::from(note.localize(lang)))],
        );
    }
    for key in &o.report.stale {
        line(
            "sync-stale-decision",
            &[("key", FluentValue::from(key.clone()))],
        );
    }
    if o.report.conflicts.is_empty() && o.report.changes.is_empty() {
        line("sync-nothing-to-do", &[("path", path)]);
    } else {
        line(
            "sync-summary",
            &[
                ("path", path.clone()),
                ("ours", FluentValue::from(count(Pick::Ours))),
                ("theirs", FluentValue::from(count(Pick::Theirs))),
                ("both", FluentValue::from(count(Pick::Both))),
            ],
        );
        for change in &o.report.changes {
            line(
                "sync-change-line",
                &[
                    ("side", FluentValue::from(side_word(change.side))),
                    ("key", FluentValue::from(change.key.clone())),
                    ("value", FluentValue::from(sync_value(lang, &change.value))),
                ],
            );
        }
    }
    for c in &o.report.conflicts {
        line(
            "sync-conflict-line",
            &[
                ("key", FluentValue::from(c.key.clone())),
                ("base", FluentValue::from(sync_value(lang, &c.base))),
                ("ours", FluentValue::from(sync_value(lang, &c.ours))),
                ("theirs", FluentValue::from(sync_value(lang, &c.theirs))),
            ],
        );
    }
    if !o.report.conflicts.is_empty() {
        line(
            "sync-conflicts-remain",
            &[("count", FluentValue::from(o.report.conflicts.len()))],
        );
    }
    if o.decisions > 0 {
        line(
            "sync-decisions-stored",
            &[
                ("count", FluentValue::from(o.decisions)),
                ("path", FluentValue::from(o.project.display().to_string())),
            ],
        );
    }
    if o.dry_run {
        line("sync-dry-run", &[]);
    }
}

/// The `sync` command. Exit code 1 when conflicts remain.
pub(crate) fn sync(
    lang: &Lang,
    project: &Path,
    dry_run: bool,
    take: &[Take],
    json: bool,
) -> Result<(), Failure> {
    let outcome = sync_file(project, dry_run, take).map_err(|e| Failure::Error(e.message(lang)))?;
    if json {
        let value = sync_json(&outcome, lang);
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else {
        print_outcome(&outcome, lang);
    }
    if outcome.report.conflicts.is_empty() {
        Ok(())
    } else {
        Err(Failure::Findings)
    }
}
