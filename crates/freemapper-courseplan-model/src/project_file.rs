//! The project file: an event stored as TOML, loaded and validated.

use std::path::{Path, PathBuf};

use thiserror::Error;

use toml_edit::{ArrayOfTables, DocumentMut, InlineTable, Item, Table, TableLike, Value};

use crate::{Event, FORMAT_VERSION, validate};

/// An inconsistency inside an event.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum ModelError {
    #[error("control {0} is defined more than once")]
    DuplicateControl(crate::ControlCode),
    #[error("course {0} is defined more than once")]
    DuplicateCourse(String),
    #[error("class {0} is defined more than once")]
    DuplicateClass(String),
    #[error("special object id {0} is used more than once")]
    DuplicateSpecialId(String),
    #[error("special object {id} belongs to course {course}, which is not defined")]
    UnknownCourseInSpecial { id: String, course: String },
    #[error("course {course} uses control {code}, which is not defined")]
    UnknownControl {
        course: String,
        code: crate::ControlCode,
    },
    #[error("course {course} has start {id}, which is not defined")]
    UnknownStart { course: String, id: String },
    #[error("course {course} has finish {id}, which is not defined")]
    UnknownFinish { course: String, id: String },
    #[error("course {course} has team size {team_size}, at least 2 is needed")]
    TeamTooSmall { course: String, team_size: u8 },
    #[error("course {course} lists control {code} as free and as mandatory")]
    FreeControlIsMandatory {
        course: String,
        code: crate::ControlCode,
    },
    /// `what` is "control code", "special object id", "course name" or "class name".
    #[error("{what} \"{value}\" is empty or starts or ends with a space")]
    BlankName { what: &'static str, value: String },
    /// Courses name their start and finish by id, so a control code must not look like one.
    #[error("control code {0} is also the id of a special object")]
    CodeClashesWithSpecial(crate::ControlCode),
    #[error("special object {id} lists course {course} more than once")]
    DuplicateCourseInSpecial { id: String, course: String },
    /// `kind` is "marked route", "forbidden route", "out-of-bounds area" or "dangerous area".
    #[error("{kind} {id} needs at least {needed} points")]
    TooFewPoints {
        id: String,
        kind: &'static str,
        needed: usize,
    },
    #[error("event date {0} is not a date; write it as YYYY-MM-DD, without time or offset")]
    DateNotADate(String),
    #[error("project file format {0} is newer than this program supports")]
    UnsupportedFormat(u32),
    /// `table` is a dotted path such as `courses[0]` or `controls[2].description`, empty for
    /// the top level of the file.
    #[error("unknown key {key} {}", place(table))]
    UnknownKey { table: String, key: String },
}

fn place(table: &str) -> String {
    if table.is_empty() {
        "at the top level".to_owned()
    } else {
        format!("in {table}")
    }
}

fn list_errors(errors: &[ModelError]) -> String {
    errors
        .iter()
        .map(|e| format!("\n  {e}"))
        .collect::<String>()
}

/// Why a project file could not be loaded, created or saved.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("cannot read {}: {source}", path.display())]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot write {}: {source}", path.display())]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    /// `create` does not overwrite a file.
    #[error("{} already exists; choose another project file or remove it", path.display())]
    AlreadyExists { path: PathBuf },
    /// `message` already names the file.
    #[error("{message}")]
    Parse { path: PathBuf, message: String },
    #[error("{} is not a consistent event:{}", path.display(), list_errors(errors))]
    Invalid {
        path: PathBuf,
        errors: Vec<ModelError>,
    },
}

/// A loaded project file: the event and the parsed document it came from.
#[derive(Debug, Clone)]
pub struct ProjectFile {
    pub path: PathBuf,
    pub event: Event,
    /// Kept for writing changes back without losing comments and layout.
    document: DocumentMut,
}

impl ProjectFile {
    /// Reads, parses and validates the project file at `path`.
    pub fn load(path: &Path) -> Result<ProjectFile, ProjectError> {
        let text = std::fs::read_to_string(path).map_err(|source| ProjectError::Read {
            path: path.to_owned(),
            source,
        })?;
        let parse_error = |message: String| ProjectError::Parse {
            path: path.to_owned(),
            message: format!("cannot parse {}: {message}", path.display()),
        };
        let document = text
            .parse::<DocumentMut>()
            .map_err(|e| parse_error(e.to_string()))?;
        // A newer format may hold anything, so its version is read before the rest.
        let format = document.get("format").and_then(Item::as_integer);
        if let Some(format) = format.filter(|&f| f > i64::from(FORMAT_VERSION)) {
            return Err(ProjectError::Invalid {
                path: path.to_owned(),
                errors: vec![ModelError::UnsupportedFormat(
                    u32::try_from(format).unwrap_or(u32::MAX),
                )],
            });
        }
        let event: Event = toml_edit::de::from_document(document.clone())
            .map_err(|e| parse_error(e.to_string()))?;
        let mut errors = unknown_keys(&document, &fresh_document(path, &event)?);
        errors.extend(validate(&event));
        if !errors.is_empty() {
            return Err(ProjectError::Invalid {
                path: path.to_owned(),
                errors,
            });
        }
        Ok(ProjectFile {
            path: path.to_owned(),
            event,
            document,
        })
    }

    /// Writes a new project file for `event`. Refuses an inconsistent event and an existing file.
    pub fn create(path: &Path, event: Event) -> Result<ProjectFile, ProjectError> {
        check(path, &event)?;
        let document = fresh_document(path, &event)?;
        if path.exists() {
            return Err(ProjectError::AlreadyExists {
                path: path.to_owned(),
            });
        }
        write_atomic(path, &document.to_string())?;
        Ok(ProjectFile {
            path: path.to_owned(),
            event,
            document,
        })
    }

    /// Validates the event and writes it back. Values that did not change stay as written, with
    /// their comments; a changed value keeps the comment next to it and stays inline if it was.
    /// Saving an unchanged event leaves the file byte for byte as it was.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        check(&self.path, &self.event)?;
        let fresh = fresh_document(&self.path, &self.event)?;
        let mut merged = self.document.clone();
        let mut reordered = false;
        merge_tables(merged.as_table_mut(), fresh.into_table(), &mut reordered);
        if reordered {
            renumber(merged.as_table_mut(), &mut 0);
        }
        write_atomic(&self.path, &merged.to_string())?;
        self.document = merged;
        Ok(())
    }

    /// The map file: `sources.map`, relative to the folder of the project file unless absolute.
    pub fn map_path(&self) -> PathBuf {
        let folder = self.path.parent().unwrap_or_else(|| Path::new(""));
        folder.join(&self.event.sources.map)
    }
}

fn check(path: &Path, event: &Event) -> Result<(), ProjectError> {
    let errors = validate(event);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ProjectError::Invalid {
            path: path.to_owned(),
            errors,
        })
    }
}

fn fresh_document(path: &Path, event: &Event) -> Result<DocumentMut, ProjectError> {
    // The `toml` serialiser lays tables out as `[table]` sections, the way people write them;
    // `toml_edit`'s own serialiser would make inline tables of everything.
    let error = |e: String| ProjectError::Parse {
        path: path.to_owned(),
        message: format!("cannot serialise {}: {e}", path.display()),
    };
    let text = toml::to_string(event).map_err(|e| error(e.to_string()))?;
    let mut document = text
        .parse::<DocumentMut>()
        .map_err(|e| error(e.to_string()))?;
    tidy(document.as_table_mut());
    Ok(document)
}

/// Drops empty tables, which the reader would assume anyway, and the positions of the fresh
/// text, which mean nothing next to the positions of the kept document.
fn tidy(table: &mut Table) {
    table.set_position(None);
    let empty: Vec<String> = table
        .iter()
        .filter(|(_, item)| matches!(item, Item::Table(t) if t.is_empty()))
        .map(|(k, _)| k.to_owned())
        .collect();
    for key in empty {
        table.remove(&key);
    }
    for (_, item) in table.iter_mut() {
        match item {
            Item::Table(t) => tidy(t),
            Item::ArrayOfTables(a) => a.iter_mut().for_each(tidy),
            _ => {}
        }
    }
}

/// The keys of `document` that the event does not hold. Format 1 is strict, so a misspelt key
/// is an error instead of a value that silently goes missing.
///
/// `fresh` is the serialisation of the event loaded from `document`. Serde ignores unknown keys,
/// and `Course` flattens its kind, so `deny_unknown_fields` cannot catch them; a key of the file
/// that the serialisation lacks is unknown instead. The serialiser leaves out empty lists and
/// empty tables, so such a key counts as known where the format has a field of that name.
fn unknown_keys(document: &DocumentMut, fresh: &DocumentMut) -> Vec<ModelError> {
    let mut errors = Vec::new();
    walk_table(document.as_table(), fresh.as_table(), "", &mut errors);
    errors
}

/// A table or an array in the file or in the fresh serialisation, whatever its TOML form.
#[derive(Clone, Copy)]
enum Node<'a> {
    Item(&'a Item),
    Value(&'a Value),
    Table(&'a Table),
}

impl<'a> Node<'a> {
    fn table(self) -> Option<&'a dyn TableLike> {
        match self {
            Node::Item(item) => item.as_table_like(),
            Node::Value(Value::InlineTable(t)) => Some(t),
            Node::Value(_) => None,
            Node::Table(t) => Some(t),
        }
    }

    fn elements(self) -> Option<Vec<Node<'a>>> {
        match self {
            Node::Item(Item::ArrayOfTables(a)) => Some(a.iter().map(Node::Table).collect()),
            Node::Item(Item::Value(Value::Array(a))) | Node::Value(Value::Array(a)) => {
                Some(a.iter().map(Node::Value).collect())
            }
            _ => None,
        }
    }
}

fn walk(doc: Node<'_>, fresh: Node<'_>, path: &str, errors: &mut Vec<ModelError>) {
    if let (Some(d), Some(f)) = (doc.table(), fresh.table()) {
        walk_table(d, f, path, errors);
    } else if let (Some(d), Some(f)) = (doc.elements(), fresh.elements()) {
        for (i, (d, f)) in d.into_iter().zip(f).enumerate() {
            walk(d, f, &format!("{path}[{i}]"), errors);
        }
    }
}

fn walk_table(
    doc: &dyn TableLike,
    fresh: &dyn TableLike,
    path: &str,
    errors: &mut Vec<ModelError>,
) {
    for (key, item) in doc.iter() {
        let child = if path.is_empty() {
            key.to_owned()
        } else {
            format!("{path}.{key}")
        };
        match (fresh.get(key), item.as_table_like()) {
            (Some(f), _) => walk(Node::Item(item), Node::Item(f), &child, errors),
            // Every key inside a table the serialisation left out is unknown.
            (None, Some(t)) if omitted_when_empty(path, key) => {
                walk_table(t, &Table::new(), &child, errors);
            }
            (None, None) if omitted_when_empty(path, key) && is_default(item) => {}
            (None, _) => errors.push(ModelError::UnknownKey {
                table: path.to_owned(),
                key: key.to_owned(),
            }),
        }
    }
}

/// Whether the serialiser leaves out the key `key` of the table at `path` when it is empty or has
/// its default (see [`is_default`]).
///
/// `bends` and `taped` are allowed in every element of a course's lists, so an empty `bends = []` on a score
/// control or a relay part counts as known; it has no effect there.
fn omitted_when_empty(path: &str, key: &str) -> bool {
    let element_of = |array: &str| {
        path.strip_prefix(array)
            .and_then(|rest| rest.strip_prefix('['))
            .and_then(|rest| rest.strip_suffix(']'))
            .is_some_and(|index| index.bytes().all(|b| b.is_ascii_digit()))
    };
    match key {
        "classes" | "controls" | "specials" | "courses" | "accepted_findings"
            if path.is_empty() =>
        {
            true
        }
        "classes" | "start_bends" | "start_taped" => element_of("courses"),
        "description" | "kind" => element_of("controls"),
        "courses" => element_of("specials"),
        "controls" | "legs" => element_of("accepted_findings"),
        "rule_overlays" => path == "event",
        "bends" | "taped" => {
            path.starts_with("courses[") && path.contains('.') && path.ends_with(']')
        }
        _ => false,
    }
}

/// Writes next to the target and renames over it, so a failed write leaves the old file.
fn write_atomic(path: &Path, text: &str) -> Result<(), ProjectError> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let result = std::fs::write(&tmp, text).and_then(|()| std::fs::rename(&tmp, path));
    if let Err(source) = result {
        let _ = std::fs::remove_file(&tmp);
        return Err(ProjectError::Write {
            path: path.to_owned(),
            source,
        });
    }
    Ok(())
}

/// Gives every table a rising position in tree order: the order of the keys, and within an array
/// of tables the order of its elements, each followed by its own subtables.
///
/// toml_edit prints tables sorted by position, so a kept table would otherwise stay where it was
/// even after the event moved it. In a file that interleaves sections, for example `[[controls]]`,
/// `[[courses]]`, `[[controls]]`, this also groups the sections of one array together.
fn renumber(table: &mut Table, next: &mut isize) {
    table.set_position(Some(*next));
    *next += 1;
    for (_, item) in table.iter_mut() {
        match item {
            Item::Table(t) => renumber(t, next),
            Item::ArrayOfTables(a) => a.iter_mut().for_each(|t| renumber(t, next)),
            _ => {}
        }
    }
}

/// Brings `old` to the content of `new`, keeping order, comments and formatting of what is equal.
/// Sets `reordered` when an array of tables now holds kept elements in another order.
fn merge_tables(old: &mut Table, new: Table, reordered: &mut bool) {
    let entries = new.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    merge_table_like(old, entries, reordered);
}

/// [`merge_tables`] for a table or an inline table.
fn merge_table_like(old: &mut dyn TableLike, new: Vec<(String, Item)>, reordered: &mut bool) {
    // A key the serialiser leaves out holds an empty list or table in the file, which means the
    // same as no key. Keeping it keeps the file as the user wrote it.
    let stale: Vec<String> = old
        .iter()
        .filter(|(k, item)| !new.iter().any(|(n, _)| n == k) && !is_default(item))
        .map(|(k, _)| k.to_owned())
        .collect();
    for key in stale {
        old.remove(&key);
    }
    for (key, item) in new {
        match old.get_mut(&key) {
            Some(slot) => merge_item(slot, item, identity_key(&key), reordered),
            None if left_out_default(&key, &item) => {}
            None => {
                old.insert(&key, item);
            }
        }
    }
}

fn is_empty(item: &Item) -> bool {
    match item {
        Item::Table(t) => t.is_empty(),
        Item::ArrayOfTables(a) => a.is_empty(),
        Item::Value(Value::Array(a)) => a.is_empty(),
        Item::Value(Value::InlineTable(t)) => t.is_empty(),
        _ => false,
    }
}

/// An empty list or table, `false`, or `"control"` (the default `kind` of a control). The serialiser
/// leaves such a value out, so a key that holds one in the file means the same as no key.
fn is_default(item: &Item) -> bool {
    is_empty(item) || item.as_bool() == Some(false) || item.as_str() == Some("control")
}

/// A key the file leaves out keeps being left out while the event holds its default.
fn left_out_default(key: &str, item: &Item) -> bool {
    let default = match key {
        "discipline" => "training",
        "rule_profile" => crate::types::DEFAULT_RULE_PROFILE,
        _ => return false,
    };
    item.as_str() == Some(default)
}

/// The field that tells the elements of an array of tables apart, where the format has one.
fn identity_key(table_key: &str) -> Option<&'static str> {
    match table_key {
        "controls" => Some("code"),
        "courses" | "classes" => Some("name"),
        "specials" => Some("id"),
        _ => None,
    }
}

/// Merges each new element into the old element with the same identity; the new order wins.
fn merge_by_identity(old: &mut ArrayOfTables, new: ArrayOfTables, id: &str, reordered: &mut bool) {
    let mut pool: Vec<Option<Table>> = old.iter().cloned().map(Some).collect();
    old.clear();
    let mut last_kept = None;
    for fresh in new {
        let wanted = fresh.get(id).and_then(Item::as_value);
        let found = pool.iter().position(|slot| {
            let have = slot
                .as_ref()
                .and_then(|t| t.get(id))
                .and_then(Item::as_value);
            matches!((have, wanted), (Some(a), Some(b)) if same_data(a, b))
        });
        match found.and_then(|i| Some((i, pool[i].take()?))) {
            Some((index, mut kept)) => {
                if last_kept.is_some_and(|last| index < last) {
                    *reordered = true;
                }
                last_kept = Some(index);
                merge_tables(&mut kept, fresh, reordered);
                old.push(kept);
            }
            None => old.push(fresh),
        }
    }
}

fn merge_item(old: &mut Item, new: Item, identity: Option<&str>, reordered: &mut bool) {
    // The serialiser writes `[section]`s and `[[sections]]`; the file may hold the same data
    // inline, and then it stays inline.
    let new = match (&*old, new) {
        (Item::Value(Value::Array(_)), Item::ArrayOfTables(n)) => {
            Item::Value(Value::Array(n.into_array()))
        }
        (Item::Value(Value::InlineTable(_)), Item::Table(n)) => {
            Item::Value(Value::InlineTable(n.into_inline_table()))
        }
        (_, new) => new,
    };
    match (&mut *old, new) {
        (Item::Table(o), Item::Table(n)) => merge_tables(o, n, reordered),
        (Item::ArrayOfTables(o), Item::ArrayOfTables(n)) if identity.is_some() => {
            merge_by_identity(o, n, identity.unwrap_or_default(), reordered);
        }
        (Item::ArrayOfTables(o), Item::ArrayOfTables(n)) if o.len() == n.len() => {
            for (o, n) in o.iter_mut().zip(n) {
                merge_tables(o, n, reordered);
            }
        }
        (Item::Value(o), Item::Value(n)) => merge_value(o, n, reordered),
        (_, mut new) => {
            if let (Some(prefix), Some(table)) = (leading_decor(old), first_table_mut(&mut new)) {
                table.decor_mut().set_prefix(prefix);
            }
            *old = new;
        }
    }
}

/// Brings an inline value to `new`. Equal data stays as written. A changed value keeps the
/// whitespace and comment around it, an inline table keeps its keys' layout and an array of the
/// same length is merged element by element.
fn merge_value(old: &mut Value, new: Value, reordered: &mut bool) {
    if same_data(old, &new) {
        return;
    }
    match (&mut *old, new) {
        (Value::InlineTable(o), Value::InlineTable(n)) => {
            let entries = n
                .into_iter()
                .map(|(k, mut v)| {
                    v.decor_mut().clear();
                    (k.to_string(), Item::Value(v))
                })
                .collect();
            // The space before `}` belongs to the last value; it moves when the last key changes.
            let last = |t: &InlineTable| t.iter().last().map(|(k, _)| k.to_owned());
            let last_before = last(o);
            let tail = o
                .iter()
                .last()
                .and_then(|(_, v)| v.decor().suffix().cloned());
            merge_table_like(o, entries, reordered);
            if last(o) != last_before {
                if let Some(v) = last_before.and_then(|k| o.get_mut(&k)) {
                    v.decor_mut().set_suffix("");
                }
                if let (Some((_, v)), Some(tail)) = (o.iter_mut().last(), tail) {
                    v.decor_mut().set_suffix(tail);
                }
            }
        }
        (Value::Array(o), Value::Array(n)) => {
            let old_len = o.len();
            let new_len = n.len();
            for (i, mut n) in n.into_iter().enumerate() {
                if let Some(slot) = o.get_mut(i) {
                    merge_value(slot, n, reordered);
                } else {
                    match o.get(old_len.saturating_sub(1)) {
                        Some(last) => {
                            *n.decor_mut() = last.decor().clone();
                            o.push_formatted(n);
                        }
                        None => o.push(n),
                    }
                }
            }
            while o.len() > new_len {
                o.remove(o.len() - 1);
            }
        }
        (o, mut n) => {
            *n.decor_mut() = o.decor().clone();
            *o = n;
        }
    }
}

fn leading_decor(item: &Item) -> Option<toml_edit::RawString> {
    let table = match item {
        Item::Table(t) => t,
        Item::ArrayOfTables(a) => a.get(0)?,
        _ => return None,
    };
    table.decor().prefix().cloned()
}

fn first_table_mut(item: &mut Item) -> Option<&mut Table> {
    match item {
        Item::Table(t) => Some(t),
        Item::ArrayOfTables(a) => a.get_mut(0),
        _ => None,
    }
}

/// Equal data, whatever the whitespace and comments around it. An integer equals a float of the
/// same value (`10` and `10.0`), and a string equals an integer it spells (`"31"` and `31`): a
/// control code may be written either way.
fn same_data(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(a), Value::String(b)) => a.value() == b.value(),
        (Value::Integer(a), Value::Integer(b)) => a.value() == b.value(),
        (Value::Float(a), Value::Float(b)) => a.value().to_bits() == b.value().to_bits(),
        (Value::Integer(i), Value::Float(f)) | (Value::Float(f), Value::Integer(i)) => {
            (*i.value() as f64).to_bits() == f.value().to_bits()
        }
        (Value::String(s), Value::Integer(i)) | (Value::Integer(i), Value::String(s)) => {
            *s.value() == i.value().to_string()
        }
        (Value::Boolean(a), Value::Boolean(b)) => a.value() == b.value(),
        (Value::Datetime(a), Value::Datetime(b)) => a.value() == b.value(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(a, b)| same_data(a, b))
        }
        (Value::InlineTable(a), Value::InlineTable(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, a)| b.get(k).is_some_and(|b| same_data(a, b)))
        }
        _ => false,
    }
}
