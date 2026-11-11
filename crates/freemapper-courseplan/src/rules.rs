//! Rule profiles: check rules as data files (ADR 0012, ADR 0025).
//!
//! The format, the parser and the merge live in the shared crate `freemapper-rules` (freemapper-core
//! ADR 0011). This module holds what is specific to this program: the embedded profiles
//! ([`EMBEDDED`]), the [`SCHEMA`] with the rule keys, table values and filters, and thin wrappers
//! that bind the schema and the embedded files. `rules/README.md` describes the file format for
//! people who add a profile.
//!
//! Each rule names a `key`, the check that implements it. [`SCHEMA`] lists the keys; a file with
//! another key is refused. The values a check reads are in the rule's `values` table, and their
//! names carry the unit. Value names without a unit are tables whose shape the schema documents.
//! A rule filters on `map_kinds`, `disciplines` and `print_scales`; [`applies`] tests a course.

use freemapper_courseplan_model::{Course, Discipline};
pub use freemapper_rules::{
    FORMAT, FilterArg, FilterItem, FilterKind, FilterSchema, KeySchema, LocalizedText, ProfileKind,
    Rule, RuleProfile, RuleSet, RulesError, Schema, SetRule, Severity, Source, SourceRef,
    SwitchedOff, TableSchema, value_name_problems, value_number,
};

use crate::map::MapKind;

/// The embedded profiles: file stem and content.
const EMBEDDED: &[(&str, &str)] = &[
    ("wkb-2026", include_str!("../rules/wkb-2026.toml")),
    ("nor-bb-2021", include_str!("../rules/nor-bb-2021.toml")),
];

/// The document key of values this program proposes where no source gives a number
/// (`[sources.claude]` in a profile); output names it as a proposal of freemapper-courseplan.
pub const PROPOSAL_DOC: &str = "claude";

/// The document key of rules the project owner set (`[sources.user]` in a profile); output names
/// it as a decision of the project owner.
pub const USER_DOC: &str = "user";

/// The filter field of the map kinds a rule applies to.
pub const FIELD_MAP_KINDS: &str = "map_kinds";
/// The filter field of the disciplines a rule applies to.
pub const FIELD_DISCIPLINES: &str = "disciplines";
/// The filter field of the print scale denominators a rule applies to.
pub const FIELD_PRINT_SCALES: &str = "print_scales";

/// What the profiles of this program may contain: the rule keys it implements (plan M2b, ruling
/// R2), the units, the table values and the filters.
pub static SCHEMA: Schema = Schema {
    keys: &[
        KeySchema {
            key: "min_distance",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "similar_features",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "map_edge",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "leg_through_control",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "leg_through_forbidden",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "acute_angle",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "straight_angle",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "number_gap",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "opposing_legs",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "code_range",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "code_pairs",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "code_confusable_next",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "same_objects_in_circle",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "start_capacity",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "first_leg_angle",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "length_table",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "difficulty_mix",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "nor_lkm",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_value_matches_code",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_code_ambiguity",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_ring",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_obvious_order",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_length_bands",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_limit_and_penalty",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_link_valid",
            values: None,
            without_filters: &[],
        },
        KeySchema {
            key: "score_mix",
            values: None,
            without_filters: &[],
        },
    ],
    units: &["metres", "mm", "degrees", "percent", "factor", "minutes"],
    tables: &[
        TableSchema {
            key: "code_range",
            value: "range",
            shape: "[min, max]: the lowest and the highest allowed code number, both included",
            check: None,
        },
        TableSchema {
            key: "code_pairs",
            value: "pairs",
            shape: "[[a, b], ...]: two codes that must not both be used in one event",
            check: None,
        },
        TableSchema {
            key: "length_table",
            value: "courses",
            shape: "[{ course, printed, categories, he_length_percent = [min, max] }, ...]: one row per \
         course of the table; categories are rule categories, printed is the row as the source \
         prints it",
            check: None,
        },
        TableSchema {
            key: "difficulty_mix",
            value: "levels",
            shape: "{ category = level, ... }: the technical difficulty (1 to 5) of each rule category",
            check: None,
        },
        TableSchema {
            key: "difficulty_mix",
            value: "easy_levels",
            shape: "[level, ...]: levels whose categories never share a course with hard_levels",
            check: None,
        },
        TableSchema {
            key: "difficulty_mix",
            value: "hard_levels",
            shape: "[level, ...]: levels whose categories never share a course with easy_levels",
            check: None,
        },
        TableSchema {
            key: "nor_lkm",
            value: "courses",
            shape: "[{ course, lkm, o_value, categories }, ...]: one row per course of the table; lkm is \
         the target in performance kilometres, o_value the orienteering difficulty 1 to 4",
            check: None,
        },
    ],
    filters: &[
        FilterSchema {
            field: FIELD_MAP_KINDS,
            kind: FilterKind::Text(&["sprint", "schoolyard", "forest"]),
        },
        FilterSchema {
            field: FIELD_DISCIPLINES,
            kind: FilterKind::Text(&["sprint", "middle", "long", "ultralong", "training"]),
        },
        FilterSchema {
            field: FIELD_PRINT_SCALES,
            kind: FilterKind::Int {
                min: 1,
                max: u32::MAX as i64,
            },
        },
    ],
    proposal_doc: PROPOSAL_DOC,
    value_check: None,
};

/// Whether the rule's filters select a course with this map kind, discipline and print scale.
pub fn applies(rule: &Rule, map_kind: MapKind, discipline: Discipline, print_scale: u32) -> bool {
    rule.applies(&[
        (FIELD_MAP_KINDS, FilterArg::Text(map_kind.name())),
        (FIELD_DISCIPLINES, FilterArg::Text(discipline.as_str())),
        (FIELD_PRINT_SCALES, FilterArg::Int(i64::from(print_scale))),
    ])
}

/// The print scale a rule filter sees for a course: its planned print scale, else the map scale.
pub fn print_scale_of(course: &Course, map_scale: u32) -> u32 {
    course.planned_print_scale.unwrap_or(map_scale)
}

/// Sets the map kinds filter of `rule`; an empty list removes it. For tests.
#[cfg(test)]
pub(crate) fn set_map_kinds(rule: &mut Rule, kinds: &[MapKind]) {
    let items = kinds
        .iter()
        .map(|k| FilterItem::Text(k.name().to_owned()))
        .collect();
    rule.set_filter(FIELD_MAP_KINDS, items);
}

/// Sets the disciplines filter of `rule`; an empty list removes it. For tests.
#[cfg(test)]
pub(crate) fn set_disciplines(rule: &mut Rule, disciplines: &[Discipline]) {
    let items = disciplines
        .iter()
        .map(|d| FilterItem::Text(d.as_str().to_owned()))
        .collect();
    rule.set_filter(FIELD_DISCIPLINES, items);
}

/// Sets the print scales filter of `rule`; an empty list removes it. For tests.
#[cfg(test)]
pub(crate) fn set_print_scales(rule: &mut Rule, scales: &[u32]) {
    let items = scales
        .iter()
        .map(|n| FilterItem::Int(i64::from(*n)))
        .collect();
    rule.set_filter(FIELD_PRINT_SCALES, items);
}

/// The ids of the embedded profiles, base profiles and overlays.
pub fn available() -> Vec<&'static str> {
    freemapper_rules::available(EMBEDDED)
}

/// The rule categories a profile names, in its length tables (`categories` of each row) and its difficulty
/// levels (the keys of `levels`), sorted.
pub fn categories(profile: &RuleProfile) -> Vec<String> {
    let mut found = std::collections::BTreeSet::new();
    for rule in &profile.rules {
        for value in rule.values.values() {
            match value {
                toml::Value::Array(rows) => {
                    for row in rows.iter().filter_map(toml::Value::as_table) {
                        let list = row.get("categories").and_then(toml::Value::as_array);
                        found.extend(
                            list.into_iter()
                                .flatten()
                                .filter_map(toml::Value::as_str)
                                .map(str::to_owned),
                        );
                    }
                }
                toml::Value::Table(levels) => found.extend(levels.keys().cloned()),
                _ => {}
            }
        }
    }
    found.into_iter().collect()
}

/// The embedded profile `id`.
pub fn profile(id: &str) -> Result<RuleProfile, RulesError> {
    freemapper_rules::profile(&SCHEMA, EMBEDDED, id)
}

/// Merges the embedded base profile `base` with the overlays, in order.
pub fn load_rule_set(base: &str, overlays: &[String]) -> Result<RuleSet, RulesError> {
    freemapper_rules::load_rule_set(&SCHEMA, EMBEDDED, base, overlays)
}

/// Parses one profile file whose file stem is `id`.
pub fn parse_profile(id: &str, text: &str) -> Result<RuleProfile, RulesError> {
    freemapper_rules::parse_profile(&SCHEMA, id, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper_courseplan_model::CourseKind;
    use std::collections::BTreeSet;
    use toml::{Table, Value};

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
        let known: BTreeSet<String> = SCHEMA.keys.iter().map(|k| s(k.key)).collect();
        assert_eq!(
            keys, known,
            "every key of the schema has a rule in an embedded profile"
        );
        let set = load_rule_set("wkb-2026", &[s("nor-bb-2021")]).unwrap();
        assert_eq!(set.base, "wkb-2026");
        assert!(set.rules.iter().any(|r| r.profile == "nor-bb-2021"));
        assert!(set.rules.iter().all(|r| set.source(r).is_some()));
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
        assert_eq!(
            rule.filter(FIELD_MAP_KINDS),
            [
                FilterItem::Text(s("sprint")),
                FilterItem::Text(s("schoolyard"))
            ]
        );
        assert_eq!(
            rule.filter(FIELD_DISCIPLINES),
            [FilterItem::Text(s("sprint")), FilterItem::Text(s("middle"))]
        );
        assert_eq!(
            rule.filter(FIELD_PRINT_SCALES),
            [FilterItem::Int(3000), FilterItem::Int(4000)]
        );
        let course = |planned: Option<u32>| Course {
            ppen_id: None,
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
            applies(
                rule,
                MapKind::Sprint,
                Discipline::Sprint,
                print_scale_of(c, map_scale),
            )
        };
        assert!(sprint(&course(Some(4000)), 5000));
        assert!(!sprint(&course(None), 5000));
        assert!(sprint(&course(None), 3000));
        assert!(!sprint(&course(Some(5000)), 4000));
        assert!(!applies(rule, MapKind::Forest, Discipline::Sprint, 4000));
        assert!(!applies(rule, MapKind::Sprint, Discipline::Long, 4000));
        // Empty filters select everything.
        let all = &parse_profile("base-x", BASE).unwrap().rules[0];
        assert!(applies(all, MapKind::Forest, Discipline::Training, 15000));
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
            assert_eq!(rule.filter(FIELD_DISCIPLINES).len(), 1, "{}", rule.id);
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
            .filter(|r| r.rule.key == key && applies(&r.rule, kind, Discipline::Long, print_scale))
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
            ppen_id: None,
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
                    value_name_problems(&SCHEMA, &rule),
                    Vec::<String>::new(),
                    "{}",
                    rule.id
                );
            }
        }
    }

    #[test]
    fn schema_is_consistent() {
        assert_eq!(SCHEMA.problems(), Vec::<String>::new());
    }

    #[test]
    fn filter_values_equal_the_enum_names() {
        let list = |field: &str| match SCHEMA.filters.iter().find(|f| f.field == field) {
            Some(FilterSchema {
                kind: FilterKind::Text(names),
                ..
            }) => names.to_vec(),
            other => panic!("{field}: {other:?}"),
        };
        let kinds: Vec<&str> = MapKind::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(list(FIELD_MAP_KINDS), kinds);
        let disciplines: Vec<&str> = Discipline::ALL.iter().map(|d| d.as_str()).collect();
        assert_eq!(list(FIELD_DISCIPLINES), disciplines);
    }

    #[test]
    fn unit_ranges_come_from_the_shared_loader() {
        let rule = |values: &str| {
            parse_profile(
                "base-x",
                &base_with_rule(&format!(
                    "id = \"basex.distance.min\"\nkey = \"min_distance\"\nseverity = \"error\"\nvalues = {values}\nsource = {{ doc = \"DOC\", section = \"1.1\" }}\n"
                )),
            )
        };
        assert!(rule("{ metres = 15.0, tolerance_percent = 6 }").is_ok());
        assert!(rule("{ distance = 15.0 }").is_err());
        assert!(rule("{ metres = -1.0 }").is_err());
        assert!(rule("{ tolerance_percent = 120 }").is_err());
    }

    /// Renders the rules of an embedded rule set as text: id, key, severity, enabled flag,
    /// filters (lists sorted), values as inline TOML and source, plus the switched-off ids.
    fn render_set(set: &RuleSet) -> String {
        fn filters(rule: &Rule) -> String {
            let mut parts = Vec::new();
            for field in [FIELD_MAP_KINDS, FIELD_DISCIPLINES, FIELD_PRINT_SCALES] {
                let mut items: Vec<String> = rule
                    .filter(field)
                    .iter()
                    .map(|item| match item {
                        FilterItem::Text(text) => text.clone(),
                        FilterItem::Int(n) => n.to_string(),
                    })
                    .collect();
                items.sort_unstable();
                if !items.is_empty() {
                    parts.push(format!("{field}=[{}]", items.join(",")));
                }
            }
            parts.join(" ")
        }
        let mut out = format!("set {} + [{}]\n", set.base, set.overlays.join(","));
        for r in &set.rules {
            let rule = &r.rule;
            out += &format!(
                "{} | {} | {} | enabled={} | {} | {} | {}:{} | from {}\n",
                rule.id,
                rule.key,
                rule.severity.as_str(),
                rule.enabled,
                filters(rule),
                Value::Table(rule.values.clone()),
                rule.source.doc,
                rule.source.section,
                r.profile
            );
        }
        for off in &set.switched_off {
            out += &format!("off {} (by {})\n", off.rule.id, off.by);
        }
        out
    }

    #[test]
    fn embedded_rule_sets_are_unchanged() {
        let mut text = render_set(&load_rule_set("wkb-2026", &[]).unwrap());
        text += &render_set(&load_rule_set("wkb-2026", &[s("nor-bb-2021")]).unwrap());
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/snapshots/embedded_rule_sets.txt"
        );
        if std::env::var_os("UPDATE_SNAPSHOT").is_some() {
            std::fs::write(path, &text).unwrap();
        }
        let expected = std::fs::read_to_string(path).unwrap();
        assert!(
            text == expected,
            "the embedded rule sets changed; rerun with UPDATE_SNAPSHOT=1 if intended"
        );
    }
}
