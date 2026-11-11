//! The editor's state: the project file as the server last read it (parsed once per revision), its
//! revision, and the JSON the page reads (`api/state`, `api/revision`, `api/overlay`) (ADR 0046).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::http::StatusCode;
use freemapper_courseplan::course::{control_numbers, course_length_m};
use freemapper_courseplan::map::map_kind_for_file;
use freemapper_courseplan::sync::base::fnv1a64;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{ProjectError, ProjectFile};
use serde_json::{Value, json};

use super::EditorState;
use super::history::HistoryError;
use super::map_view::{FileStamp, MapEntry};
use super::overlay::r3;
use crate::mcp::{EDITOR_TOOLS, OpError};

/// The fingerprint of the project file (FNV-1a 64 over its bytes, 16 hex digits).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Revision(pub String);

impl Revision {
    pub(crate) fn of(bytes: &[u8]) -> Revision {
        Revision(fnv1a64(bytes))
    }

    /// The revision of a file that cannot be read.
    fn missing() -> Revision {
        Revision("missing".to_owned())
    }
}

/// Largest project file the editor opens. Reading, hashing and parsing a bigger one would freeze the page.
pub(crate) const MAX_PROJECT_BYTES: u64 = 16 * 1024 * 1024;
/// Longest error text sent to the page, in characters.
const MAX_MESSAGE_CHARS: usize = 500;

/// Makes an error text fit for the page: each of `paths` shrinks to its file name, and the text is cut at
/// [`MAX_MESSAGE_CHARS`] characters.
pub(crate) fn tidy(message: &str, paths: &[&Path]) -> String {
    let mut out = message.to_owned();
    for path in paths {
        let full = path.display().to_string();
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && !full.is_empty()
        {
            out = out.replace(&full, name);
        }
    }
    if out.chars().count() > MAX_MESSAGE_CHARS {
        out = out.chars().take(MAX_MESSAGE_CHARS).collect();
        out.push('…');
    }
    out
}

/// The file name of `path` for messages.
pub(crate) fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// The project file at one revision: parsed, or the localized reason why it is not.
pub(crate) struct Loaded {
    pub revision: Revision,
    pub file: Result<ProjectFile, String>,
}

/// The bytes of the project file, or why there are none.
enum Source {
    Bytes(Vec<u8>),
    TooLarge,
    Unreadable(std::io::Error),
}

impl Source {
    /// Reads the file once, at most [`MAX_PROJECT_BYTES`] + 1 bytes.
    fn of(path: &Path) -> Source {
        use std::io::Read as _;
        let read = || -> std::io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(MAX_PROJECT_BYTES + 1)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        };
        match read() {
            Ok(bytes) if bytes.len() as u64 > MAX_PROJECT_BYTES => Source::TooLarge,
            Ok(bytes) => Source::Bytes(bytes),
            Err(e) => Source::Unreadable(e),
        }
    }

    fn revision(&self) -> Revision {
        match self {
            Source::Bytes(bytes) => Revision::of(bytes),
            Source::TooLarge => Revision("too-large".to_owned()),
            Source::Unreadable(_) => Revision::missing(),
        }
    }
}

impl Loaded {
    /// Reads the file at `path`: the bytes once, hashed and parsed as they are.
    pub(crate) fn read(path: &Path, lang: &Lang) -> Loaded {
        let source = Source::of(path);
        let revision = source.revision();
        Self::from_source(path, lang, source, revision)
    }

    /// Reads the file and returns the new state, or `None` when its bytes hash to the revision of `prev`.
    pub(crate) fn changed_since(prev: &Loaded, path: &Path, lang: &Lang) -> Option<Loaded> {
        let source = Source::of(path);
        let revision = source.revision();
        (revision != prev.revision).then(|| Self::from_source(path, lang, source, revision))
    }

    fn from_source(path: &Path, lang: &Lang, source: Source, revision: Revision) -> Loaded {
        let file = match source {
            Source::TooLarge => Err(text(
                lang,
                "error-editor-project-too-large",
                &[
                    ("file", FluentValue::from(file_name(path))),
                    (
                        "limit",
                        FluentValue::from(MAX_PROJECT_BYTES / (1024 * 1024)),
                    ),
                ],
            )),
            Source::Unreadable(source) => Err(ProjectError::Read {
                path: path.to_owned(),
                source,
            }
            .localize(lang)),
            Source::Bytes(bytes) => String::from_utf8(bytes)
                .map_err(|e| ProjectError::Read {
                    path: path.to_owned(),
                    source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                })
                .and_then(|text| ProjectFile::from_text(path, &text))
                .map_err(|e| e.localize(lang)),
        };
        Loaded {
            revision,
            file: file.map_err(|message| tidy(&message, &[path])),
        }
    }
}

/// Replaces the object `{"$__toml_private_datetime": "..."}` that a TOML date becomes in JSON by its text.
fn plain_dates(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(text)) = map.get("$__toml_private_datetime")
                && map.len() == 1
            {
                *value = Value::String(text.clone());
                return;
            }
            map.values_mut().for_each(plain_dates);
        }
        Value::Array(items) => items.iter_mut().for_each(plain_dates),
        _ => {}
    }
}

/// `GET api/state`. A broken project file or map gives only `revision` and `error`.
pub(crate) fn state_json(
    loaded: &Loaded,
    map: Result<&MapEntry, &str>,
    undo: (Option<&str>, Option<&str>),
) -> Value {
    let broken = |error: &str| json!({ "revision": loaded.revision.0, "error": error });
    let file = match &loaded.file {
        Ok(file) => file,
        Err(e) => return broken(e),
    };
    let entry = match map {
        Ok(entry) => entry,
        Err(e) => return broken(e),
    };
    let scale = entry.doc.georef.scale_denominator;
    let kind = map_kind_for_file(&entry.doc, &file.map_path());
    let event = &file.event;
    let courses: Vec<Value> = event
        .courses
        .iter()
        .map(|course| {
            json!({
                "name": course.name,
                "length_m": course_length_m(event, course, scale).ok().map(r3),
                "numbers": control_numbers(event, course),
            })
        })
        .collect();
    let extent = entry
        .index
        .extent()
        .map(|(a, b)| json!([[r3(a.x), r3(a.y)], [r3(b.x), r3(b.y)]]));
    let ppen = event
        .sources
        .ppen
        .as_ref()
        .map(|p| file.path.parent().unwrap_or(Path::new("")).join(p));
    let mut event_json = serde_json::to_value(event).unwrap_or(Value::Null);
    plain_dates(&mut event_json);
    json!({
        "revision": loaded.revision.0,
        "event": event_json,
        "map": {
            "path": file.map_path().display().to_string(),
            "scale": scale,
            "kind": kind.name(),
            "extent": extent,
        },
        "courses": courses,
        "undo": { "undo": undo.0, "redo": undo.1 },
        "link": { "ppen": ppen.map(|p| p.display().to_string()) },
        "error": Value::Null,
    })
}

/// `GET api/revision`: the project revision and the stamps of the map and the linked `.ppen`.
pub(crate) fn revision_json(loaded: &Loaded) -> Value {
    let stamp = |path: Option<PathBuf>| {
        path.and_then(|p| FileStamp::of(&p).ok())
            .map_or_else(String::new, |s| s.token())
    };
    let (map, ppen) = match &loaded.file {
        Ok(file) => (
            stamp(Some(file.map_path())),
            stamp(
                file.event
                    .sources
                    .ppen
                    .as_ref()
                    .map(|p| file.path.parent().unwrap_or(Path::new("")).join(p)),
            ),
        ),
        Err(_) => (String::new(), String::new()),
    };
    json!({ "project": loaded.revision.0, "map": map, "ppen": ppen })
}

/// The value of a query parameter, percent-decoded (`+` is a space).
pub(super) fn decode(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                let hex = raw.get(i + 1..i + 3)?;
                out.push(u8::from_str_radix(hex, 16).ok()?);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// The `course` parameter of `api/overlay`: `Ok(None)` without it. An unknown or repeated parameter is an
/// error.
pub(crate) fn overlay_query(raw: Option<&str>) -> Result<Option<String>, ()> {
    let mut course = None;
    for pair in raw.unwrap_or("").split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').ok_or(())?;
        if key != "course" || course.is_some() {
            return Err(());
        }
        course = Some(decode(value).ok_or(())?);
    }
    Ok(course)
}

/// Largest coordinate the page may send, in map mm.
const MAX_COORDINATE: f64 = 1.0e7;

/// The query of `api/number-at`: `course`, `index` (from 1, the overlay's `numbers[].index`), `x` and `y` (map
/// mm), each exactly once, finite and in range.
pub(crate) fn number_at_query(raw: Option<&str>) -> Option<(String, usize, f64, f64)> {
    let (mut course, mut index, mut x, mut y) = (None, None, None, None);
    for pair in raw.unwrap_or("").split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=')?;
        let fresh = match key {
            "course" => course.replace(decode(value)?).is_none(),
            "index" => index.replace(value.parse::<usize>().ok()?).is_none(),
            "x" => x.replace(finite(value)?).is_none(),
            "y" => y.replace(finite(value)?).is_none(),
            _ => false,
        };
        if !fresh {
            return None;
        }
    }
    let (course, index, x, y) = (course?, index?, x?, y?);
    (index >= 1 && x.abs() <= MAX_COORDINATE && y.abs() <= MAX_COORDINATE)
        .then_some((course, index, x, y))
}

fn finite(value: &str) -> Option<f64> {
    value.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Smallest gap the page may store: the 0.01 mm the answer rounds to.
const MIN_GAP_MM: f64 = 0.01;

/// Rounds to 0.01 without float noise; no negative zero.
fn r2(v: f64) -> f64 {
    ((v * 100.0).round() / 100.0) + 0.0
}

/// The direction and gap of the number of control `index` (from 1, as in the overlay) of `course` when its box
/// is centred on `drop`. `None` when the course or the number does not exist.
pub(crate) fn number_at(
    event: &freemapper_courseplan_model::Event,
    course: &str,
    index: usize,
    drop: &freemapper_courseplan_model::MapPoint,
    kind: freemapper_courseplan::map::MapKind,
    map_scale: u32,
) -> Option<Value> {
    use freemapper_courseplan_model::CourseKind;
    let course = event.courses.iter().find(|c| c.name == course)?;
    let CourseKind::Individual { controls } = &course.kind else {
        return None;
    };
    let number = control_numbers(event, course)
        .get(index.checked_sub(1)?)
        .copied()
        .flatten()?;
    let code = &controls.get(index - 1)?.code;
    let control = event.controls.iter().find(|c| &c.code == code)?;
    let found = freemapper_courseplan::numbers::placement_from_centre(
        &control.map_mm,
        drop,
        number,
        kind,
        map_scale,
    );
    let direction = found.direction.rem_euclid(360.0);
    let direction = r2(direction);
    let direction = if direction >= 360.0 { 0.0 } else { direction };
    // A box over the circle has a negative gap and a far drop a huge one; the operations store a gap of more
    // than 0 and at most MAX_GAP_MM only, and 0.004 rounds to 0. The answer is kept inside that range and
    // says so, so the page can tell the user instead of showing a refusal.
    let gap = r2(found.gap_mm);
    let kept = gap.clamp(MIN_GAP_MM, freemapper_courseplan::numbers::MAX_GAP_MM);
    Some(json!({ "direction": direction, "gap_mm": kept, "clamped": kept != gap }))
}

/// Status and JSON body of an operation or history answer.
pub(crate) type Reply = (StatusCode, Value);

/// The text of a catalog message without arguments.
pub(super) fn msg(lang: &Lang, id: &str) -> String {
    text(lang, id, &[])
}

/// The project text of a source, when it is UTF-8.
fn text_of(source: &Source) -> Option<String> {
    match source {
        Source::Bytes(bytes) => std::str::from_utf8(bytes).ok().map(str::to_owned),
        _ => None,
    }
}

/// Reads the project file again into the state and returns its text.
pub(super) fn reload(st: &mut EditorState) -> Option<String> {
    // The stamp comes first: a write after the read then shows up as a changed stamp.
    st.stamp = FileStamp::of(&st.project).ok();
    let source = Source::of(&st.project);
    let revision = source.revision();
    let text = text_of(&source);
    st.loaded = Arc::new(Loaded::from_source(&st.project, &st.lang, source, revision));
    text
}

pub(super) fn conflict(st: &EditorState, error: String) -> Reply {
    (
        StatusCode::CONFLICT,
        json!({ "error": error, "revision": st.loaded.revision.0 }),
    )
}

/// Projects that get a write between the first read and the tool (tests only).
#[cfg(test)]
pub(super) static RACE: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

#[cfg(test)]
fn race_hook(path: &Path) {
    use std::io::Write as _;
    if RACE
        .lock()
        .is_ok_and(|paths| paths.iter().any(|p| p == path))
        && let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(path)
    {
        let _ = file.write_all(b"\n# raced\n");
    }
}

/// Takes the injected field `project` out of the list of expected fields in a serde error text.
pub(crate) fn without_project_field(message: &str) -> String {
    message
        .replace("`project`, ", "")
        .replace("`project` or ", "")
        .replace(", `project`", "")
}

/// One editor operation (`POST api/op/{tool}`), under the state lock: read the file, compare the revision
/// the page saw, run the MCP tool, read the file again and record the step (ADR 0043).
pub(crate) fn run_op(
    st: &mut EditorState,
    tool: &str,
    revision: Option<&str>,
    args: Option<Value>,
) -> Reply {
    let lang = st.lang.clone();
    let fail = |status, id: &str| (status, json!({ "error": msg(&lang, id) }));
    if !EDITOR_TOOLS.contains(&tool) {
        return fail(StatusCode::NOT_FOUND, "editor-op-unknown");
    }
    let Some(revision) = revision else {
        return fail(StatusCode::BAD_REQUEST, "editor-op-no-revision");
    };
    let Some(args @ Value::Object(_)) = args else {
        return fail(StatusCode::BAD_REQUEST, "editor-op-bad-body");
    };
    let before = reload(st);
    if let Err(e) = &st.loaded.file {
        let e = e.clone();
        return conflict(st, e);
    }
    let seen = st.loaded.revision.clone();
    if seen.0 != revision {
        return conflict(st, msg(&lang, "editor-op-stale"));
    }
    // The tool reads the file again. If another program wrote in between, `before` is not what the tool
    // will start from, and a step recorded from it would let undo drop that write.
    #[cfg(test)]
    race_hook(&st.project);
    if std::fs::read(&st.project).ok().as_deref() != before.as_deref().map(str::as_bytes) {
        reload(st);
        return conflict(st, msg(&lang, "editor-op-stale"));
    }
    match st.server.call_json(&st.project, tool, args) {
        Ok(result) => {
            let after = reload(st);
            if let (Some(before), Some(after)) = (before, after)
                && before != after
            {
                let label = msg(&lang, &format!("editor-op-{}", tool.replace('_', "-")));
                st.history.record(label, before, after);
            }
            (
                StatusCode::OK,
                json!({ "revision": st.loaded.revision.0, "result": result }),
            )
        }
        Err(OpError::UnknownTool) => fail(StatusCode::NOT_FOUND, "editor-op-unknown"),
        Err(OpError::BadArgs(message)) => (
            StatusCode::BAD_REQUEST,
            json!({ "error": tidy(&without_project_field(&message), &[]) }),
        ),
        Err(OpError::Io) => {
            reload(st);
            fail(StatusCode::INTERNAL_SERVER_ERROR, "editor-io-failed")
        }
        Err(OpError::Refused(message)) => {
            reload(st);
            if st.loaded.revision == seen {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    json!({ "error": tidy(&message, &[&st.project]) }),
                )
            } else {
                // Someone wrote the file while the operation ran, and the save guard refused.
                conflict(st, msg(&lang, "editor-op-stale"))
            }
        }
    }
}

/// `POST api/undo` and `POST api/redo`. Like an operation, they name the revision the page saw: a second
/// tab that shows an older revision must not revert what the first tab did.
pub(crate) fn run_history(st: &mut EditorState, redo: bool, revision: Option<&str>) -> Reply {
    let lang = st.lang.clone();
    let Some(revision) = revision else {
        return (
            StatusCode::BAD_REQUEST,
            json!({ "error": msg(&lang, "editor-op-no-revision") }),
        );
    };
    reload(st);
    if st.loaded.revision.0 != revision {
        return conflict(st, msg(&lang, "editor-op-stale"));
    }
    let result = if redo {
        st.history.redo(&st.project)
    } else {
        st.history.undo(&st.project)
    };
    reload(st);
    match result {
        Ok(_) => (StatusCode::OK, json!({ "revision": st.loaded.revision.0 })),
        Err(HistoryError::Nothing) => {
            let id = if redo {
                "editor-redo-nothing"
            } else {
                "editor-undo-nothing"
            };
            conflict(st, msg(&lang, id))
        }
        Err(HistoryError::Changed) => conflict(st, msg(&lang, "editor-undo-changed")),
        Err(HistoryError::Write(reason)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": tidy(&format!("{}: {reason}", msg(&lang, "editor-undo-failed")), &[&st.project]) }),
        ),
    }
}
