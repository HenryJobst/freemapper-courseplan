//! Rule profiles: check rules as data files (ADR 0012, ADR 0025).
//!
//! A rule profile is a TOML file in `rules/`, embedded with `include_str!`. A base profile (such as
//! `wkb-2026`) holds a full rule set; an overlay (such as `nor-bb-2021`) names its base and adds
//! rules, replaces base rules with the same `id`, or switches them off with `enabled = false`.
//! [`load_rule_set`] merges one base and its overlays into a [`RuleSet`] that remembers where each
//! rule came from. `rules/README.md` describes the file format for people who add a profile.
//!
//! Each rule names a `key`, the check that implements it. [`KNOWN_KEYS`] lists the keys; a file
//! with another key is refused. The values a check reads are in the rule's `values` table, and
//! their names carry the unit ([`UNITS`]). Value names without a unit are tables whose shape
//! [`TABLE_VALUES`] documents.

use std::collections::BTreeMap;

use freemapper_courseplan_model::{Course, Discipline};
use toml::{Table, Value};

use crate::map::MapKind;

/// The profile file format this program reads.
pub const FORMAT: i64 = 1;

/// The embedded profiles: file stem and content.
const EMBEDDED: &[(&str, &str)] = &[
    ("wkb-2026", include_str!("../rules/wkb-2026.toml")),
    ("nor-bb-2021", include_str!("../rules/nor-bb-2021.toml")),
];

/// The rule keys this program implements (plan M2b, ruling R2).
pub const KNOWN_KEYS: &[&str] = &[
    "min_distance",
    "similar_features",
    "map_edge",
    "leg_through_control",
    "leg_through_forbidden",
    "acute_angle",
    "straight_angle",
    "opposing_legs",
    "code_range",
    "code_pairs",
    "code_confusable_next",
    "same_objects_in_circle",
    "start_capacity",
    "first_leg_angle",
    "length_table",
    "difficulty_mix",
    "nor_lkm",
];

/// Units a numeric value name ends in (`metres`, `min_length_metres`, `tolerance_percent`).
pub const UNITS: &[&str] = &["metres", "mm", "degrees", "percent", "factor"];

/// Value names that hold a table or a list instead of a number with a unit: rule key, value name
/// and the shape of the value.
pub const TABLE_VALUES: &[(&str, &str, &str)] = &[
    (
        "code_range",
        "range",
        "[min, max]: the lowest and the highest allowed code number, both included",
    ),
    (
        "code_pairs",
        "pairs",
        "[[a, b], ...]: two codes that must not both be used in one event",
    ),
    (
        "length_table",
        "courses",
        "[{ course, printed, categories, he_length_percent = [min, max] }, ...]: one row per \
         course of the table; categories are rule categories, printed is the row as the source \
         prints it",
    ),
    (
        "difficulty_mix",
        "levels",
        "{ category = level, ... }: the technical difficulty (1 to 5) of each rule category",
    ),
    (
        "difficulty_mix",
        "easy_levels",
        "[level, ...]: levels whose categories never share a course with hard_levels",
    ),
    (
        "difficulty_mix",
        "hard_levels",
        "[level, ...]: levels whose categories never share a course with easy_levels",
    ),
    (
        "nor_lkm",
        "courses",
        "[{ course, lkm, o_value, categories }, ...]: one row per course of the table; lkm is \
         the target in performance kilometres, o_value the orienteering difficulty 1 to 4",
    ),
];

/// A text in several languages, keyed by language code. English is always present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedText(pub BTreeMap<String, String>);

impl LocalizedText {
    /// The text in `lang` (a language code such as `de`), else the English text.
    pub fn get(&self, lang: &str) -> &str {
        self.0
            .get(lang)
            .or_else(|| self.0.get("en"))
            .map(String::as_str)
            .unwrap_or("")
    }
}

/// How serious a finding of a rule is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Hint,
}

impl Severity {
    /// The name used in profile files.
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Hint => "hint",
        }
    }

    pub(crate) fn parse(text: &str) -> Option<Severity> {
        [Severity::Error, Severity::Warning, Severity::Hint]
            .into_iter()
            .find(|s| s.as_str() == text)
    }
}

/// Whether a profile stands alone or extends a base profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKind {
    Base,
    Overlay,
}

/// A document a profile cites, listed under `[sources.<doc>]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub title: String,
    pub url: Option<String>,
}

/// Where a rule comes from: the document key of the profile's `[sources]` and the section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRef {
    pub doc: String,
    pub section: String,
}

/// One check rule of a profile.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Stable dotted id, `<profile>.<family>.<name>`.
    pub id: String,
    /// The check that implements the rule, one of [`KNOWN_KEYS`].
    pub key: String,
    pub severity: Severity,
    /// `false` in an overlay switches the base rule with the same id off.
    pub enabled: bool,
    /// Map kinds the rule applies to; empty means all.
    pub map_kinds: Vec<MapKind>,
    /// Disciplines the rule applies to; empty means all.
    pub disciplines: Vec<Discipline>,
    /// Print scale denominators the rule applies to; empty means all. See [`print_scale_of`].
    pub print_scales: Vec<u32>,
    /// The values the check reads; names carry their unit.
    pub values: Table,
    pub source: SourceRef,
    /// Shown with the finding.
    pub note: Option<LocalizedText>,
}

impl Rule {
    /// Whether the rule's filters select a course with this map kind, discipline and print scale.
    pub fn applies(&self, map_kind: MapKind, discipline: Discipline, print_scale: u32) -> bool {
        (self.map_kinds.is_empty() || self.map_kinds.contains(&map_kind))
            && (self.disciplines.is_empty() || self.disciplines.contains(&discipline))
            && (self.print_scales.is_empty() || self.print_scales.contains(&print_scale))
    }
}

/// The print scale a rule filter sees for a course: its planned print scale, else the map scale.
pub fn print_scale_of(course: &Course, map_scale: u32) -> u32 {
    course.planned_print_scale.unwrap_or(map_scale)
}

/// One parsed profile file.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfile {
    /// The file stem, such as `wkb-2026`.
    pub id: String,
    pub kind: ProfileKind,
    /// The base profile an overlay extends.
    pub base: Option<String>,
    pub name: LocalizedText,
    /// The date the rules apply from, as written in the file.
    pub valid_from: Option<String>,
    pub sources: BTreeMap<String, Source>,
    pub rules: Vec<Rule>,
}

/// A rule of a rule set with the profile it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct SetRule {
    pub rule: Rule,
    pub profile: String,
}

/// A rule that a profile switched off with `enabled = false`.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchedOff {
    /// The rule as it was before it was switched off.
    pub rule: Rule,
    /// The profile the switched-off rule came from.
    pub profile: String,
    /// The profile that switched it off.
    pub by: String,
}

/// A base profile merged with its overlays: the rules a check runs.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet {
    pub base: String,
    pub overlays: Vec<String>,
    /// Active rules in file order: base rules first, then rules the overlays add.
    pub rules: Vec<SetRule>,
    pub switched_off: Vec<SwitchedOff>,
    /// The `[sources]` of every profile in the set, by profile id.
    pub sources: BTreeMap<String, BTreeMap<String, Source>>,
}

impl RuleSet {
    /// The active rule with this id.
    pub fn rule(&self, id: &str) -> Option<&SetRule> {
        self.rules.iter().find(|r| r.rule.id == id)
    }

    /// The document a rule cites, from the profile the rule came from.
    pub fn source(&self, rule: &SetRule) -> Option<&Source> {
        self.sources.get(&rule.profile)?.get(&rule.rule.source.doc)
    }
}

/// Why a rule profile or rule set cannot be loaded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RulesError {
    #[error("unknown rule profile {id}; available: {available}")]
    UnknownProfile { id: String, available: String },
    #[error("rule profile {0} is an overlay, not a base profile")]
    NotABase(String),
    #[error("rule profile {0} is a base profile, not an overlay")]
    NotAnOverlay(String),
    #[error("overlay {overlay} extends {extends}, not {base}")]
    OverlayForAnotherBase {
        overlay: String,
        extends: String,
        base: String,
    },
    #[error("rule profile {profile} is malformed: {message}")]
    Malformed { profile: String, message: String },
    #[error("rule {rule} in profile {profile} has the unknown key {key}")]
    UnknownKey {
        profile: String,
        rule: String,
        key: String,
    },
    #[error("overlay {overlay} switches off rule {rule}, which the base profile does not have")]
    UnknownRule { overlay: String, rule: String },
}

/// The document key of values this program proposes where no source gives a number
/// (`[sources.claude]` in a profile); output names it as a proposal of freemapper-courseplan.
pub const PROPOSAL_DOC: &str = "claude";

/// The ids of the embedded profiles, base profiles and overlays.
pub fn available() -> Vec<&'static str> {
    EMBEDDED.iter().map(|(id, _)| *id).collect()
}

/// The embedded profile `id`.
pub fn profile(id: &str) -> Result<RuleProfile, RulesError> {
    profile_in(EMBEDDED, id)
}

/// Merges the embedded base profile `base` with the overlays, in order.
pub fn load_rule_set(base: &str, overlays: &[String]) -> Result<RuleSet, RulesError> {
    load_from(EMBEDDED, base, overlays)
}

fn profile_in(files: &[(&str, &str)], id: &str) -> Result<RuleProfile, RulesError> {
    match files.iter().find(|(stem, _)| *stem == id) {
        Some((stem, text)) => parse_profile(stem, text),
        None => Err(RulesError::UnknownProfile {
            id: id.to_owned(),
            available: files
                .iter()
                .map(|(stem, _)| *stem)
                .collect::<Vec<_>>()
                .join(", "),
        }),
    }
}

fn load_from(
    files: &[(&str, &str)],
    base: &str,
    overlays: &[String],
) -> Result<RuleSet, RulesError> {
    let base_profile = profile_in(files, base)?;
    if base_profile.kind != ProfileKind::Base {
        return Err(RulesError::NotABase(base.to_owned()));
    }
    let mut set = RuleSet {
        base: base.to_owned(),
        overlays: overlays.to_vec(),
        rules: Vec::new(),
        switched_off: Vec::new(),
        sources: BTreeMap::new(),
    };
    merge(&mut set, base_profile)?;
    for overlay in overlays {
        let profile = profile_in(files, overlay)?;
        if profile.kind != ProfileKind::Overlay {
            return Err(RulesError::NotAnOverlay(overlay.clone()));
        }
        let extends = profile.base.clone().unwrap_or_default();
        if extends != base {
            return Err(RulesError::OverlayForAnotherBase {
                overlay: overlay.clone(),
                extends,
                base: base.to_owned(),
            });
        }
        merge(&mut set, profile)?;
    }
    Ok(set)
}

/// Adds the rules of `profile` to `set`: a rule with a known id replaces the rule, a new id is
/// added, and `enabled = false` switches the rule off.
fn merge(set: &mut RuleSet, profile: RuleProfile) -> Result<(), RulesError> {
    let from = profile.id.clone();
    set.sources.insert(from.clone(), profile.sources);
    for rule in profile.rules {
        let active = set.rules.iter().position(|r| r.rule.id == rule.id);
        let off = set.switched_off.iter().position(|r| r.rule.id == rule.id);
        if rule.enabled {
            let entry = SetRule {
                rule,
                profile: from.clone(),
            };
            match active {
                Some(i) => set.rules[i] = entry,
                None => {
                    if let Some(i) = off {
                        set.switched_off.remove(i);
                    }
                    set.rules.push(entry);
                }
            }
        } else if let Some(i) = active {
            let old = set.rules.remove(i);
            set.switched_off.push(SwitchedOff {
                rule: old.rule,
                profile: old.profile,
                by: from.clone(),
            });
        } else if off.is_some() {
            // Already switched off by an earlier profile.
        } else if profile.kind == ProfileKind::Base {
            set.switched_off.push(SwitchedOff {
                rule,
                profile: from.clone(),
                by: from.clone(),
            });
        } else {
            return Err(RulesError::UnknownRule {
                overlay: from,
                rule: rule.id,
            });
        }
    }
    Ok(())
}

/// Parses one profile file whose file stem is `id`.
pub fn parse_profile(id: &str, text: &str) -> Result<RuleProfile, RulesError> {
    let bad = |message: String| RulesError::Malformed {
        profile: id.to_owned(),
        message,
    };
    let doc: Table = text
        .parse()
        .map_err(|e: toml::de::Error| bad(e.to_string()))?;
    only_fields(&doc, &["format", "profile", "sources", "rules"], "the file").map_err(bad)?;
    match doc.get("format") {
        Some(Value::Integer(FORMAT)) => {}
        Some(Value::Integer(n)) if *n > FORMAT => {
            return Err(bad(format!(
                "format {n} is newer than this program reads ({FORMAT})"
            )));
        }
        Some(other) => return Err(bad(format!("format {other} is not {FORMAT}"))),
        None => return Err(bad(format!("format = {FORMAT} is missing"))),
    }

    let head = table(&doc, "profile", "the file").map_err(bad)?;
    only_fields(
        head,
        &["id", "kind", "base", "name", "valid_from"],
        "[profile]",
    )
    .map_err(bad)?;
    let file_id = string(head, "id", "[profile]").map_err(bad)?;
    if file_id != id {
        return Err(bad(format!(
            "[profile] id {file_id} differs from the file name {id}"
        )));
    }
    let kind = match string(head, "kind", "[profile]").map_err(bad)?.as_str() {
        "base" => ProfileKind::Base,
        "overlay" => ProfileKind::Overlay,
        other => return Err(bad(format!("kind {other} is neither base nor overlay"))),
    };
    let base = optional_string(head, "base", "[profile]").map_err(bad)?;
    match (kind, &base) {
        (ProfileKind::Overlay, None) => return Err(bad("an overlay needs a base".to_owned())),
        (ProfileKind::Base, Some(_)) => {
            return Err(bad("a base profile has no base".to_owned()));
        }
        _ => {}
    }
    let name = localized(head.get("name"), "[profile] name").map_err(bad)?;
    let valid_from = match head.get("valid_from") {
        None => None,
        Some(Value::Datetime(date)) => Some(date.to_string()),
        Some(Value::String(date)) => Some(date.clone()),
        Some(_) => return Err(bad("[profile] valid_from is not a date".to_owned())),
    };

    let mut sources = BTreeMap::new();
    if let Some(value) = doc.get("sources") {
        let Value::Table(entries) = value else {
            return Err(bad("[sources] is not a table".to_owned()));
        };
        for (doc_key, entry) in entries {
            let place = format!("[sources.{doc_key}]");
            let Value::Table(entry) = entry else {
                return Err(bad(format!("{place} is not a table")));
            };
            only_fields(entry, &["title", "url"], &place).map_err(bad)?;
            sources.insert(
                doc_key.clone(),
                Source {
                    title: string(entry, "title", &place).map_err(bad)?,
                    url: optional_string(entry, "url", &place).map_err(bad)?,
                },
            );
        }
    }

    let mut rules: Vec<Rule> = Vec::new();
    let entries = match doc.get("rules") {
        None => Vec::new(),
        Some(Value::Array(entries)) => entries.clone(),
        Some(_) => return Err(bad("rules is not a list of [[rules]]".to_owned())),
    };
    for (index, entry) in entries.iter().enumerate() {
        let Value::Table(entry) = entry else {
            return Err(bad(format!("rule {} is not a table", index + 1)));
        };
        let rule = parse_rule(id, entry, index)?;
        if rules.iter().any(|r| r.id == rule.id) {
            return Err(bad(format!("rule {} is defined twice", rule.id)));
        }
        if !sources.contains_key(&rule.source.doc) {
            return Err(bad(format!(
                "rule {} cites source {}, which [sources] does not list",
                rule.id, rule.source.doc
            )));
        }
        rules.push(rule);
    }

    Ok(RuleProfile {
        id: id.to_owned(),
        kind,
        base,
        name,
        valid_from,
        sources,
        rules,
    })
}

fn parse_rule(profile: &str, entry: &Table, index: usize) -> Result<Rule, RulesError> {
    let bad = |message: String| RulesError::Malformed {
        profile: profile.to_owned(),
        message,
    };
    let place = format!("rule {}", index + 1);
    let id = string(entry, "id", &place).map_err(bad)?;
    let place = format!("rule {id}");
    only_fields(
        entry,
        &[
            "id",
            "key",
            "severity",
            "enabled",
            "map_kinds",
            "disciplines",
            "print_scales",
            "values",
            "source",
            "note",
        ],
        &place,
    )
    .map_err(bad)?;
    let well_formed = id.split('.').count() >= 3
        && id.split('.').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        });
    if !well_formed {
        return Err(bad(format!(
            "rule id {id} is not of the form <profile>.<family>.<name> in lower case"
        )));
    }
    let key = string(entry, "key", &place).map_err(bad)?;
    if !KNOWN_KEYS.contains(&key.as_str()) {
        return Err(RulesError::UnknownKey {
            profile: profile.to_owned(),
            rule: id,
            key,
        });
    }
    let severity_text = string(entry, "severity", &place).map_err(bad)?;
    let severity = Severity::parse(&severity_text).ok_or_else(|| {
        bad(format!(
            "{place}: severity {severity_text} is not error, warning or hint"
        ))
    })?;
    let enabled = match entry.get("enabled") {
        None => true,
        Some(Value::Boolean(enabled)) => *enabled,
        Some(_) => return Err(bad(format!("{place}: enabled is not true or false"))),
    };
    let map_kinds = strings(entry, "map_kinds", &place)
        .map_err(bad)?
        .iter()
        .map(|kind| {
            MapKind::ALL
                .into_iter()
                .find(|k| k.as_str() == kind)
                .ok_or_else(|| bad(format!("{place}: unknown map kind {kind}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let disciplines = strings(entry, "disciplines", &place)
        .map_err(bad)?
        .iter()
        .map(|discipline| {
            Discipline::ALL
                .into_iter()
                .find(|d| d.as_str() == discipline)
                .ok_or_else(|| bad(format!("{place}: unknown discipline {discipline}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let print_scales = match entry.get("print_scales") {
        None => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::Integer(n) if *n > 0 && *n <= i64::from(u32::MAX) => Ok(*n as u32),
                other => Err(bad(format!("{place}: print scale {other} is no scale"))),
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => return Err(bad(format!("{place}: print_scales is not a list"))),
    };
    let values = match entry.get("values") {
        None => Table::new(),
        Some(Value::Table(values)) => values.clone(),
        Some(_) => return Err(bad(format!("{place}: values is not a table"))),
    };
    let source = match entry.get("source") {
        Some(Value::Table(source)) => {
            let where_ = format!("{place} source");
            only_fields(source, &["doc", "section"], &where_).map_err(bad)?;
            SourceRef {
                doc: string(source, "doc", &where_).map_err(bad)?,
                section: string(source, "section", &where_).map_err(bad)?,
            }
        }
        _ => return Err(bad(format!("{place} has no source = {{ doc, section }}"))),
    };
    let note = match entry.get("note") {
        None => None,
        value => Some(localized(value, &format!("{place} note")).map_err(bad)?),
    };
    let rule = Rule {
        id,
        key,
        severity,
        enabled,
        map_kinds,
        disciplines,
        print_scales,
        values,
        source,
        note,
    };
    let problems = value_name_problems(&rule);
    if !problems.is_empty() {
        return Err(bad(format!(
            "rule {}: value names without a unit: {}",
            rule.id,
            problems.join(", ")
        )));
    }
    Ok(rule)
}

/// Value names of `rule` that carry no unit and are no documented table.
///
/// Text, true/false and lists of texts need no unit. A number or a list of numbers needs a name
/// that is a unit or ends in `_<unit>`. Anything else must be listed in [`TABLE_VALUES`] for the
/// rule's key.
pub fn value_name_problems(rule: &Rule) -> Vec<String> {
    fn is_text(value: &Value) -> bool {
        match value {
            Value::String(_) | Value::Boolean(_) | Value::Datetime(_) => true,
            Value::Array(items) => items.iter().all(is_text),
            _ => false,
        }
    }
    fn is_number(value: &Value) -> bool {
        match value {
            Value::Integer(_) | Value::Float(_) => true,
            Value::Array(items) => items
                .iter()
                .all(|v| matches!(v, Value::Integer(_) | Value::Float(_))),
            _ => false,
        }
    }
    let has_unit = |name: &str| {
        UNITS
            .iter()
            .any(|unit| name == *unit || name.ends_with(&format!("_{unit}")))
    };
    let documented = |name: &str| {
        TABLE_VALUES
            .iter()
            .any(|(key, value, _)| *key == rule.key && *value == name)
    };
    rule.values
        .iter()
        .filter(|(name, value)| {
            !(is_text(value) || (is_number(value) && has_unit(name)) || documented(name))
        })
        .map(|(name, _)| name.clone())
        .collect()
}

fn only_fields(table: &Table, allowed: &[&str], place: &str) -> Result<(), String> {
    match table.keys().find(|k| !allowed.contains(&k.as_str())) {
        Some(unknown) => Err(format!("{place} has the unknown field {unknown}")),
        None => Ok(()),
    }
}

fn table<'a>(table: &'a Table, name: &str, place: &str) -> Result<&'a Table, String> {
    match table.get(name) {
        Some(Value::Table(inner)) => Ok(inner),
        Some(_) => Err(format!("{name} in {place} is not a table")),
        None => Err(format!("{place} has no [{name}]")),
    }
}

fn string(table: &Table, name: &str, place: &str) -> Result<String, String> {
    optional_string(table, name, place)?.ok_or_else(|| format!("{place} has no {name}"))
}

fn optional_string(table: &Table, name: &str, place: &str) -> Result<Option<String>, String> {
    match table.get(name) {
        None => Ok(None),
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text.clone())),
        Some(_) => Err(format!("{name} in {place} is not a text")),
    }
}

fn strings(table: &Table, name: &str, place: &str) -> Result<Vec<String>, String> {
    match table.get(name) {
        None => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::String(text) => Ok(text.clone()),
                _ => Err(format!("{name} in {place} is not a list of texts")),
            })
            .collect(),
        Some(_) => Err(format!("{name} in {place} is not a list")),
    }
}

fn localized(value: Option<&Value>, place: &str) -> Result<LocalizedText, String> {
    let Some(Value::Table(texts)) = value else {
        return Err(format!("{place} is not {{ en = \"...\", de = \"...\" }}"));
    };
    let mut map = BTreeMap::new();
    for (lang, text) in texts {
        match text {
            Value::String(text) if !text.trim().is_empty() => {
                map.insert(lang.clone(), text.clone());
            }
            _ => return Err(format!("{place}: the {lang} text is empty or no text")),
        }
    }
    if !map.contains_key("en") {
        return Err(format!("{place} has no English text (en)"));
    }
    Ok(LocalizedText(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::CourseKind;
    use std::collections::BTreeSet;

    const BASE: &str = r#"
format = 1

[profile]
id = "base-x"
kind = "base"
name = { de = "Basis", en = "Base" }
valid_from = 2026-03-01

[sources.DOC]
title = "Some rules"
url = "https://example.org/rules.pdf"

[[rules]]
id = "basex.distance.min"
key = "min_distance"
severity = "error"
values = { metres = 30.0 }
source = { doc = "DOC", section = "1.1" }

[[rules]]
id = "basex.code.range"
key = "code_range"
severity = "error"
values = { range = [31, 599] }
source = { doc = "DOC", section = "1.2" }
"#;

    const OVERLAY: &str = r#"
format = 1

[profile]
id = "over-x"
kind = "overlay"
base = "base-x"
name = { de = "Aufsatz", en = "Overlay" }

[sources.REG]
title = "Regional rules"

[[rules]]
id = "basex.distance.min"
key = "min_distance"
severity = "warning"
values = { metres = 50.0 }
source = { doc = "REG", section = "§ 4" }

[[rules]]
id = "overx.map.edge"
key = "map_edge"
severity = "warning"
values = { metres = 10.0 }
source = { doc = "REG", section = "§ 5" }
"#;

    const SWITCH_OFF: &str = r#"
format = 1

[profile]
id = "off-x"
kind = "overlay"
base = "base-x"
name = { en = "Switch off" }

[sources.REG]
title = "Regional rules"

[[rules]]
id = "basex.code.range"
key = "code_range"
severity = "error"
enabled = false
values = { range = [31, 599] }
source = { doc = "REG", section = "§ 6" }
"#;

    const OTHER_BASE: &str = r#"
format = 1

[profile]
id = "other-base"
kind = "base"
name = { en = "Other" }

[sources.DOC]
title = "Other rules"
"#;

    const FOREIGN_OVERLAY: &str = r#"
format = 1

[profile]
id = "foreign-x"
kind = "overlay"
base = "other-base"
name = { en = "Foreign" }

[sources.DOC]
title = "Other rules"
"#;

    fn files() -> Vec<(&'static str, &'static str)> {
        vec![
            ("base-x", BASE),
            ("over-x", OVERLAY),
            ("off-x", SWITCH_OFF),
            ("other-base", OTHER_BASE),
            ("foreign-x", FOREIGN_OVERLAY),
        ]
    }

    fn s(text: &str) -> String {
        text.to_owned()
    }

    /// `BASE` with its first rule replaced by `rule`.
    fn base_with_rule(rule: &str) -> String {
        let head = BASE.split("[[rules]]").next().unwrap();
        format!("{head}[[rules]]\n{rule}")
    }

    #[test]
    fn every_embedded_profile_parses() {
        assert_eq!(available(), vec!["wkb-2026", "nor-bb-2021"]);
        let mut keys = BTreeSet::new();
        for id in available() {
            let profile = profile(id).unwrap_or_else(|e| panic!("{id}: {e}"));
            assert_eq!(profile.id, id);
            assert!(!profile.rules.is_empty(), "{id} has no rules");
            for lang in ["de", "en"] {
                assert!(
                    profile.name.0.contains_key(lang),
                    "{id}: name has no {lang}"
                );
                for rule in &profile.rules {
                    if let Some(note) = &rule.note {
                        assert!(note.0.contains_key(lang), "{}: note has no {lang}", rule.id);
                    }
                }
            }
            keys.extend(profile.rules.iter().map(|r| r.key.clone()));
        }
        let known: BTreeSet<String> = KNOWN_KEYS.iter().map(|k| s(k)).collect();
        assert_eq!(
            keys, known,
            "every known key has a rule in an embedded profile"
        );
        let set = load_rule_set("wkb-2026", &[s("nor-bb-2021")]).unwrap();
        assert_eq!(set.base, "wkb-2026");
        assert!(set.rules.iter().any(|r| r.profile == "nor-bb-2021"));
        assert!(set.rules.iter().all(|r| set.source(r).is_some()));
    }

    #[test]
    fn every_rule_has_a_known_key_and_a_source_in_its_file() {
        for id in available() {
            let profile = profile(id).unwrap();
            let prefix = format!("{}.", id.replace('-', ""));
            let base_ids: Vec<String> = match &profile.base {
                Some(base) => self::profile(base)
                    .unwrap()
                    .rules
                    .into_iter()
                    .map(|r| r.id)
                    .collect(),
                None => vec![],
            };
            for rule in &profile.rules {
                assert!(KNOWN_KEYS.contains(&rule.key.as_str()), "{}", rule.id);
                assert!(
                    profile.sources.contains_key(&rule.source.doc),
                    "{}: source {} not listed",
                    rule.id,
                    rule.source.doc
                );
                assert!(!rule.source.section.trim().is_empty(), "{}", rule.id);
                assert!(
                    rule.id.starts_with(&prefix) || base_ids.contains(&rule.id),
                    "{} does not start with {prefix}",
                    rule.id
                );
                if rule.source.doc == "claude" {
                    assert!(
                        rule.note.is_some(),
                        "{}: Claude's call needs a note",
                        rule.id
                    );
                }
            }
        }
        let unknown = base_with_rule(
            "id = \"basex.x.y\"\nkey = \"no_such_check\"\nseverity = \"hint\"\nsource = { doc = \"DOC\", section = \"1\" }\n",
        );
        assert_eq!(
            parse_profile("base-x", &unknown),
            Err(RulesError::UnknownKey {
                profile: s("base-x"),
                rule: s("basex.x.y"),
                key: s("no_such_check"),
            })
        );
        let unlisted = base_with_rule(
            "id = \"basex.x.y\"\nkey = \"map_edge\"\nseverity = \"hint\"\nsource = { doc = \"NOPE\", section = \"1\" }\n",
        );
        assert!(matches!(
            parse_profile("base-x", &unlisted),
            Err(RulesError::Malformed { .. })
        ));
        let no_source =
            base_with_rule("id = \"basex.x.y\"\nkey = \"map_edge\"\nseverity = \"hint\"\n");
        assert!(matches!(
            parse_profile("base-x", &no_source),
            Err(RulesError::Malformed { .. })
        ));
    }

    #[test]
    fn profile_format_must_be_one() {
        assert!(parse_profile("base-x", BASE).is_ok());
        for text in [
            BASE.replacen("format = 1", "", 1),
            BASE.replacen("format = 1", "format = 2", 1),
            BASE.replacen("format = 1", "format = \"1\"", 1),
        ] {
            assert!(
                matches!(
                    parse_profile("base-x", &text),
                    Err(RulesError::Malformed { .. })
                ),
                "{text}"
            );
        }
        // The id in the file must be the file stem.
        assert!(matches!(
            parse_profile("base-y", BASE),
            Err(RulesError::Malformed { .. })
        ));
        // Unknown fields are typos, not extensions.
        let typo = base_with_rule(
            "id = \"basex.x.y\"\nkey = \"map_edge\"\nseverity = \"hint\"\nmap_kind = [\"sprint\"]\nsource = { doc = \"DOC\", section = \"1\" }\n",
        );
        assert!(matches!(
            parse_profile("base-x", &typo),
            Err(RulesError::Malformed { .. })
        ));
    }

    #[test]
    fn overlay_replaces_base_rule() {
        let set = load_from(&files(), "base-x", &[s("over-x")]).unwrap();
        let rule = set.rule("basex.distance.min").unwrap();
        assert_eq!(rule.profile, "over-x");
        assert_eq!(rule.rule.severity, Severity::Warning);
        assert_eq!(rule.rule.values.get("metres"), Some(&Value::Float(50.0)));
        assert_eq!(set.source(rule).unwrap().title, "Regional rules");
        assert_eq!(rule.rule.source.section, "§ 4");
        // The replaced rule keeps its place; a new rule comes after the base rules.
        let ids: Vec<&str> = set.rules.iter().map(|r| r.rule.id.as_str()).collect();
        assert_eq!(
            ids,
            ["basex.distance.min", "basex.code.range", "overx.map.edge"]
        );
        assert_eq!(set.rule("basex.code.range").unwrap().profile, "base-x");
        assert_eq!(set.overlays, vec![s("over-x")]);
        assert!(set.switched_off.is_empty());
    }

    #[test]
    fn overlay_can_switch_a_rule_off() {
        let set = load_from(&files(), "base-x", &[s("off-x")]).unwrap();
        assert!(set.rule("basex.code.range").is_none());
        assert!(set.rule("basex.distance.min").is_some());
        assert_eq!(set.switched_off.len(), 1);
        let off = &set.switched_off[0];
        assert_eq!(off.rule.id, "basex.code.range");
        assert_eq!(off.profile, "base-x");
        assert_eq!(off.by, "off-x");
        // Switching off a rule the base does not have is a mistake in the overlay.
        let wrong = SWITCH_OFF.replace("basex.code.range", "basex.code.nope");
        let files = [("base-x", BASE), ("off-x", wrong.as_str())];
        assert_eq!(
            load_from(&files, "base-x", &[s("off-x")]),
            Err(RulesError::UnknownRule {
                overlay: s("off-x"),
                rule: s("basex.code.nope"),
            })
        );
    }

    #[test]
    fn overlay_for_another_base_is_refused() {
        assert_eq!(
            load_from(&files(), "base-x", &[s("foreign-x")]),
            Err(RulesError::OverlayForAnotherBase {
                overlay: s("foreign-x"),
                extends: s("other-base"),
                base: s("base-x"),
            })
        );
        assert_eq!(
            load_from(&files(), "over-x", &[]),
            Err(RulesError::NotABase(s("over-x")))
        );
        assert_eq!(
            load_from(&files(), "base-x", &[s("other-base")]),
            Err(RulesError::NotAnOverlay(s("other-base")))
        );
    }

    #[test]
    fn unknown_profile_is_refused() {
        let expected = |id: &str| {
            Err(RulesError::UnknownProfile {
                id: s(id),
                available: s("wkb-2026, nor-bb-2021"),
            })
        };
        assert_eq!(load_rule_set("wkb-2025", &[]), expected("wkb-2025"));
        assert_eq!(
            load_rule_set("wkb-2026", &[s("nor-xx")]),
            expected("nor-xx")
        );
        assert_eq!(
            profile("nope").map(|p| p.id),
            Err(RulesError::UnknownProfile {
                id: s("nope"),
                available: s("wkb-2026, nor-bb-2021"),
            })
        );
    }

    #[test]
    fn print_scale_filter_selects_courses() {
        let rule = base_with_rule(
            "id = \"basex.distance.min\"\nkey = \"min_distance\"\nseverity = \"error\"\nmap_kinds = [\"sprint\", \"schoolyard\"]\ndisciplines = [\"sprint\", \"middle\"]\nprint_scales = [3000, 4000]\nvalues = { metres = 15.0 }\nsource = { doc = \"DOC\", section = \"1.1\" }\n",
        );
        let rule = &parse_profile("base-x", &rule).unwrap().rules[0];
        assert_eq!(rule.map_kinds, [MapKind::Sprint, MapKind::Schoolyard]);
        assert_eq!(rule.disciplines, [Discipline::Sprint, Discipline::Middle]);
        assert_eq!(rule.print_scales, [3000, 4000]);
        let course = |planned: Option<u32>| Course {
            name: s("A"),
            classes: vec![],
            start: s("S1"),
            finish: s("F1"),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: planned,
            kind: CourseKind::Individual { controls: vec![] },
        };
        // A planned print scale wins over the map scale.
        assert_eq!(print_scale_of(&course(Some(4000)), 5000), 4000);
        assert_eq!(print_scale_of(&course(None), 5000), 5000);
        let sprint = |c: &Course, map_scale| {
            rule.applies(
                MapKind::Sprint,
                Discipline::Sprint,
                print_scale_of(c, map_scale),
            )
        };
        assert!(sprint(&course(Some(4000)), 5000));
        assert!(!sprint(&course(None), 5000));
        assert!(sprint(&course(None), 3000));
        assert!(!sprint(&course(Some(5000)), 4000));
        assert!(!rule.applies(MapKind::Forest, Discipline::Sprint, 4000));
        assert!(!rule.applies(MapKind::Sprint, Discipline::Long, 4000));
        // Empty filters select everything.
        let all = &parse_profile("base-x", BASE).unwrap().rules[0];
        assert!(all.applies(MapKind::Forest, Discipline::Training, 15000));
    }

    /// The rows of a documented table value as tables.
    fn rows<'a>(rule: &'a Rule, name: &str) -> Vec<&'a Table> {
        match rule.values.get(name) {
            Some(Value::Array(rows)) => rows
                .iter()
                .map(|row| row.as_table().expect("row is a table"))
                .collect(),
            other => panic!("{}: {name} is {other:?}", rule.id),
        }
    }

    #[test]
    fn embedded_tables_are_consistent() {
        let set = load_rule_set("wkb-2026", &[s("nor-bb-2021")]).unwrap();
        let mut tables = 0;
        for SetRule { rule, .. } in &set.rules {
            if !["length_table", "nor_lkm"].contains(&rule.key.as_str()) {
                continue;
            }
            tables += 1;
            assert_eq!(rule.disciplines.len(), 1, "{}", rule.id);
            let mut seen = BTreeSet::new();
            let mut courses = BTreeSet::new();
            for row in rows(rule, "courses") {
                assert!(courses.insert(row["course"].to_string()), "{}", rule.id);
                for category in row["categories"].as_array().unwrap() {
                    let category = category.as_str().unwrap();
                    assert!(!category.contains(' '), "{}: {category}", rule.id);
                    assert!(
                        seen.insert(category.to_owned()),
                        "{}: {category} twice",
                        rule.id
                    );
                }
                if rule.key == "length_table" {
                    let range = row["he_length_percent"].as_array().unwrap();
                    let (min, max) = (range[0].as_integer(), range[1].as_integer());
                    assert!(min <= max && min > Some(0), "{}: {row}", rule.id);
                } else {
                    assert!(row["lkm"].as_float().unwrap() > 0.0, "{}", rule.id);
                    let o = row["o_value"].as_integer().unwrap();
                    assert!((1..=4).contains(&o), "{}: {row}", rule.id);
                }
            }
            if rule.key == "length_table" {
                assert!(seen.contains("HE"), "{}", rule.id);
            }
        }
        assert_eq!(tables, 6, "three WKB tables and three NOR tables");
        let mix = &set.rule("wkb2026.difficulty.mix").unwrap().rule;
        let levels = mix.values["levels"].as_table().unwrap();
        assert_eq!(levels.len(), 40);
        assert!(
            levels
                .values()
                .all(|l| (1..=5).contains(&l.as_integer().unwrap()))
        );
        assert_eq!(levels["D-20"].as_integer(), Some(4));
        assert_eq!(levels["HE"].as_integer(), Some(5));
    }

    /// Ids of the active rules with `key` that select a course at `print_scale` on a map of `kind`.
    fn applying(set: &RuleSet, key: &str, kind: MapKind, print_scale: u32) -> Vec<String> {
        set.rules
            .iter()
            .filter(|r| r.rule.key == key && r.rule.applies(kind, Discipline::Long, print_scale))
            .map(|r| r.rule.id.clone())
            .collect()
    }

    #[test]
    fn distance_rules_follow_the_print_scale() {
        // Ruling R4: WKB26 B 7.2.1 halves the distances at 1:3000 and 1:4000.
        let set = load_rule_set("wkb-2026", &[]).unwrap();
        let metres = |id: &str| set.rule(id).unwrap().rule.values["metres"].as_float();
        for kind in [MapKind::Sprint, MapKind::Forest] {
            assert_eq!(
                applying(&set, "min_distance", kind, 4000),
                ["wkb2026.distance.min_half"]
            );
            assert_eq!(
                applying(&set, "min_distance", kind, 10000),
                ["wkb2026.distance.min"]
            );
        }
        assert_eq!(metres("wkb2026.distance.min_half"), Some(15.0));
        assert_eq!(metres("wkb2026.distance.min"), Some(30.0));
        let similar = [
            (
                MapKind::Sprint,
                4000,
                "wkb2026.distance.similar_sprint",
                30.0,
            ),
            (
                MapKind::Sprint,
                10000,
                "wkb2026.distance.similar_sprint_full",
                60.0,
            ),
            (MapKind::Forest, 4000, "wkb2026.distance.similar_half", 30.0),
            (MapKind::Forest, 10000, "wkb2026.distance.similar", 60.0),
        ];
        for (kind, scale, id, expected) in similar {
            assert_eq!(applying(&set, "similar_features", kind, scale), [id]);
            assert_eq!(metres(id), Some(expected), "{id}");
        }
        // A course printed at 1:4000 from a 1:10000 map gets the halved rule.
        let course = Course {
            name: s("A"),
            classes: vec![],
            start: s("S1"),
            finish: s("F1"),
            start_bends: vec![],
            start_taped: false,
            print_area: None,
            climb_m: None,
            planned_print_scale: Some(4000),
            kind: CourseKind::Individual { controls: vec![] },
        };
        let scale = print_scale_of(&course, 10000);
        assert_eq!(
            applying(&set, "min_distance", MapKind::Forest, scale),
            ["wkb2026.distance.min_half"]
        );
    }

    #[test]
    fn value_names_carry_units() {
        for id in available() {
            for rule in profile(id).unwrap().rules {
                assert_eq!(
                    value_name_problems(&rule),
                    Vec::<String>::new(),
                    "{}",
                    rule.id
                );
            }
        }
        for (values, ok) in [
            ("{ metres = 15.0 }", true),
            ("{ min_length_metres = 30.0, degrees = 45 }", true),
            ("{ tolerance_percent = 6, climb_factor = 10 }", true),
            ("{ he_length_percent = [63, 67] }", true),
            (
                "{ impassable = [\"520\", \"708\"], groups = [[\"111\", \"112\"]] }",
                true,
            ),
            ("{ distance = 15.0 }", false),
            ("{ metresx = 15.0 }", false),
            ("{ courses = [{ lkm = 2.1 }] }", false),
            ("{ ranges = { a = 1 } }", false),
        ] {
            let text = base_with_rule(&format!(
                "id = \"basex.distance.min\"\nkey = \"min_distance\"\nseverity = \"error\"\nvalues = {values}\nsource = {{ doc = \"DOC\", section = \"1.1\" }}\n"
            ));
            let parsed = parse_profile("base-x", &text);
            assert_eq!(parsed.is_ok(), ok, "{values}: {parsed:?}");
        }
        // A documented table is fine for its own key only.
        let text = base_with_rule(
            "id = \"basex.code.pairs\"\nkey = \"code_pairs\"\nseverity = \"error\"\nvalues = { pairs = [[66, 99]] }\nsource = { doc = \"DOC\", section = \"1.1\" }\n",
        );
        assert!(parse_profile("base-x", &text).is_ok());
        let text = text.replace("code_pairs", "map_edge");
        assert!(parse_profile("base-x", &text).is_err());
    }
}
