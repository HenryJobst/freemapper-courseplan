//! The editor's state: the project file as the server last read it (parsed once per revision), its
//! revision, and the JSON the page reads (`api/state`, `api/revision`, `api/overlay`) (ADR 0046).

use std::path::{Path, PathBuf};

use freemapper_courseplan::course::{control_numbers, course_length_m};
use freemapper_courseplan::map::map_kind_for_file;
use freemapper_courseplan::sync::base::fnv1a64;
use freemapper_courseplan::text::{Lang, Localize};
use freemapper_courseplan_model::ProjectFile;
use serde_json::{Value, json};

use super::map_view::{FileStamp, MapCache};
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

/// The project file at one revision: parsed, or the localized reason why it is not.
pub(crate) struct Loaded {
    pub revision: Revision,
    pub file: Result<ProjectFile, String>,
}

impl Loaded {
    /// Reads the file at `path`.
    pub(crate) fn read(path: &Path, lang: &Lang) -> Loaded {
        let revision =
            std::fs::read(path).map_or_else(|_| Revision::missing(), |b| Revision::of(&b));
        let file = ProjectFile::load(path).map_err(|e| e.localize(lang));
        Loaded { revision, file }
    }

    /// Reads the file again when its bytes differ from the revision held. Returns whether it did.
    pub(crate) fn refresh(&mut self, path: &Path, lang: &Lang) -> bool {
        let now = std::fs::read(path).map_or_else(|_| Revision::missing(), |b| Revision::of(&b));
        if now == self.revision {
            return false;
        }
        *self = Loaded::read(path, lang);
        true
    }

    /// The map file's path, when the project is readable.
    pub(crate) fn map_path(&self) -> Result<PathBuf, String> {
        self.file
            .as_ref()
            .map(ProjectFile::map_path)
            .map_err(Clone::clone)
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
pub(crate) fn state_json(loaded: &Loaded, map: &mut MapCache) -> Value {
    let broken = |error: String| json!({ "revision": loaded.revision.0, "error": error });
    let file = match &loaded.file {
        Ok(file) => file,
        Err(e) => return broken(e.clone()),
    };
    let entry = match map.get(&file.map_path()) {
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
