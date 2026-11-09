//! Builders for check tests: a small event, a map without features or with given objects, and
//! hand-made rules and rule sets.

use std::collections::BTreeMap;

use freemapper_courseplan_model::{
    Class, Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl,
    CourseKind, Discipline, Event, EventMeta, FORMAT_VERSION, MapPoint, Sources, SpecialObject,
};
use toml::Table;

use crate::omap::{MapDocument, MapGeoref, MapObject, MapPart, SymbolSet};
use crate::rules::{Rule, RuleSet, SetRule, Severity, Source, SourceRef};

/// An ISSprOM symbol set with a point (101), a line (502) and an area (520) symbol.
const SYMBOLS: &str = r#"<map><georeferencing scale="4000"/><colors count="0"></colors><symbols count="3" id="ISSprOM 2019"><symbol type="1" id="0" code="101" name="pt"><point_symbol/></symbol><symbol type="2" id="1" code="502" name="ln"><line_symbol/></symbol><symbol type="4" id="2" code="520" name="ar"><area_symbol/></symbol></symbols><view><grid/></view></map>"#;

/// A map at `1:scale` with the given objects in one part. 1:4000 is a sprint map.
pub(crate) fn map(scale: u32, objects: Vec<MapObject>) -> MapDocument {
    MapDocument {
        notes: String::new(),
        georef: MapGeoref {
            scale_denominator: scale,
            declination_deg: 0.0,
            grivation_deg: 0.0,
            ref_projected: None,
            crs_spec: None,
        },
        symbol_set: SymbolSet::from_omap_xml(SYMBOLS).expect("test symbols parse"),
        parts: vec![MapPart {
            name: "map".into(),
            objects,
        }],
        templates: vec![],
    }
}

pub(crate) fn code(s: &str) -> ControlCode {
    ControlCode(s.to_owned())
}

pub(crate) fn pt(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

pub(crate) fn course_control(s: &str) -> CourseControl {
    CourseControl {
        code: code(s),
        number: None,
        bends: vec![],
        taped: false,
    }
}

/// An individual course from start S1 to finish F1 over `codes`.
pub(crate) fn course(name: &str, codes: &[&str]) -> Course {
    Course {
        name: name.to_owned(),
        classes: vec![],
        start: "S1".to_owned(),
        finish: "F1".to_owned(),
        start_bends: vec![],
        start_taped: false,
        print_area: None,
        climb_m: None,
        planned_print_scale: None,
        kind: CourseKind::Individual {
            controls: codes.iter().map(|c| course_control(c)).collect(),
        },
    }
}

/// An event with start S1 at (0, 0), finish F1 at (100, 0), the controls at their positions
/// in map mm, and the courses.
pub(crate) fn event(controls: &[(&str, f64, f64)], courses: Vec<Course>) -> Event {
    Event {
        format: FORMAT_VERSION,
        event: EventMeta {
            title: "Test".to_owned(),
            date: None,
            discipline: Discipline::Sprint,
            rule_profile: "wkb-2026".to_owned(),
            rule_overlays: Vec::new(),
            start_window_min: None,
        },
        sources: Sources {
            map: "map.omap".into(),
            ppen: None,
        },
        classes: vec![Class {
            name: "H21".to_owned(),
            rule_category: None,
            expected_starters: None,
        }],
        controls: controls
            .iter()
            .map(|(c, x, y)| Control {
                code: code(c),
                kind: ControlKind::Control,
                map_mm: pt(*x, *y),
                description: ControlDescription::default(),
                number: None,
            })
            .collect(),
        specials: vec![
            SpecialObject::Start {
                id: "S1".to_owned(),
                map_mm: pt(0.0, 0.0),
            },
            SpecialObject::Finish {
                id: "F1".to_owned(),
                map_mm: pt(100.0, 0.0),
            },
        ],
        courses,
        accepted_findings: vec![],
    }
}

/// A rule without filters and values, citing section 1.2 of the document `TEST`.
pub(crate) fn rule(id: &str, key: &str, severity: Severity) -> Rule {
    Rule {
        id: id.to_owned(),
        key: key.to_owned(),
        severity,
        enabled: true,
        map_kinds: vec![],
        disciplines: vec![],
        print_scales: vec![],
        values: Table::new(),
        source: SourceRef {
            doc: "TEST".to_owned(),
            section: "1.2".to_owned(),
        },
        note: None,
    }
}

/// A rule set of base profile `base` with the rules, each with the profile it came from.
pub(crate) fn rule_set(rules: Vec<(Rule, &str)>) -> RuleSet {
    let mut sources = BTreeMap::new();
    for (_, profile) in &rules {
        sources.insert(
            (*profile).to_owned(),
            BTreeMap::from([(
                "TEST".to_owned(),
                Source {
                    title: "Test rules".to_owned(),
                    url: None,
                },
            )]),
        );
    }
    RuleSet {
        base: "base".to_owned(),
        overlays: vec![],
        rules: rules
            .into_iter()
            .map(|(rule, profile)| SetRule {
                rule,
                profile: profile.to_owned(),
            })
            .collect(),
        switched_off: vec![],
        sources,
    }
}
