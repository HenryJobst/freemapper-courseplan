//! The editor's state: the project file as the server last read it (parsed once per revision), its
//! revision, and the JSON the page reads (`api/state`, `api/revision`, `api/overlay`) (ADR 0046).

use std::path::{Path, PathBuf};

use freemapper_courseplan::course::{control_numbers, course_length_m};
use freemapper_courseplan::map::map_kind_for_file;
use freemapper_courseplan::sync::base::fnv1a64;
use freemapper_courseplan::text::{FluentValue, Lang, Localize, text};
use freemapper_courseplan_model::{ProjectError, ProjectFile};
use serde_json::{Value, json};

use super::map_view::{FileStamp, MapEntry};
use super::overlay::r3;

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
pub(crate) fn state_json(loaded: &Loaded, map: Result<&MapEntry, &str>) -> Value {
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
            "kind": kind.as_str(),
            "extent": extent,
        },
        "courses": courses,
        "undo": { "undo": Value::Null, "redo": Value::Null },
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
fn decode(raw: &str) -> Option<String> {
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
