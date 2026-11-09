//! Builders for check tests: a small event, a map without features or with given objects, and
//! hand-made rules and rule sets.

use std::collections::BTreeMap;

use freemapper_courseplan_model::{
    Class, Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl,
    CourseKind, Discipline, Event, EventMeta, FORMAT_VERSION, MapPoint, Sources, SpecialObject,
};
use toml::Table;

use crate::omap::{
    MapCoord, MapDocument, MapGeoref, MapObject, MapObjectKind, MapPart, SymbolSet, closed_ring,
};
use crate::rules::{Rule, RuleSet, SetRule, Severity, Source, SourceRef};

/// An ISSprOM symbol set: by id, 101 point, 502 line, 520 area, 521 area, 505.1 line, 109 point,
/// 110 point, 5051 line (not a sub-code of 505), 518 line, 709 area, 708 line.
const SYMBOLS: &str = r#"<map><georeferencing scale="4000"/><colors count="0"></colors><symbols count="11" id="ISSprOM 2019"><symbol type="1" id="0" code="101" name="pt"><point_symbol/></symbol><symbol type="2" id="1" code="502" name="ln"><line_symbol/></symbol><symbol type="4" id="2" code="520" name="ar"><area_symbol/></symbol><symbol type="4" id="3" code="521" name="building"><area_symbol/></symbol><symbol type="2" id="4" code="505.1" name="path"><line_symbol/></symbol><symbol type="1" id="5" code="109" name="knoll"><point_symbol/></symbol><symbol type="1" id="6" code="110" name="elongated knoll"><point_symbol/></symbol><symbol type="2" id="7" code="5051" name="other"><line_symbol/></symbol><symbol type="2" id="8" code="518" name="fence"><line_symbol/></symbol><symbol type="4" id="9" code="709" name="oob"><area_symbol/></symbol><symbol type="2" id="10" code="708" name="oob boundary"><line_symbol/></symbol></symbols><view><grid/></view></map>"#;

/// Symbol ids of [`map`].
pub(crate) const POINT_101: u32 = 0;
pub(crate) const AREA_520: u32 = 2;
pub(crate) const AREA_521: u32 = 3;
pub(crate) const LINE_505_1: u32 = 4;
pub(crate) const POINT_109: u32 = 5;
pub(crate) const POINT_110: u32 = 6;
pub(crate) const LINE_5051: u32 = 7;
pub(crate) const LINE_518: u32 = 8;
pub(crate) const AREA_709: u32 = 9;
pub(crate) const LINE_708: u32 = 10;

/// An ISOM symbol set: 711 line (id 0), 520 area (id 1).
const FOREST_SYMBOLS: &str = r#"<map><georeferencing scale="10000"/><colors count="0"></colors><symbols count="2" id="ISOM 2017-2"><symbol type="2" id="0" code="711" name="route"><line_symbol/></symbol><symbol type="4" id="1" code="520" name="ar"><area_symbol/></symbol></symbols><view><grid/></view></map>"#;

/// Symbol ids of [`forest_map`].
pub(crate) const FOREST_LINE_711: u32 = 0;

/// A map at `1:scale` with the given objects in one part. 1:4000 is a sprint map.
pub(crate) fn map(scale: u32, objects: Vec<MapObject>) -> MapDocument {
    map_with(SYMBOLS, scale, objects)
}

/// A forest map with the ISOM symbols at `1:scale` (above 1:5000).
pub(crate) fn forest_map(scale: u32, objects: Vec<MapObject>) -> MapDocument {
    map_with(FOREST_SYMBOLS, scale, objects)
}

fn coord(x: f64, y: f64) -> MapCoord {
    MapCoord::new((x * 1000.0).round() as i32, (y * 1000.0).round() as i32)
}

/// A point object at (x, y) map mm.
pub(crate) fn point_object(symbol_id: u32, x: f64, y: f64) -> MapObject {
    MapObject {
        symbol_id,
        kind: MapObjectKind::Point { rotation_rad: 0.0 },
        coords: vec![coord(x, y)],
        tags: vec![],
        pattern: None,
    }
}

/// An open line through the points.
pub(crate) fn line_object(symbol_id: u32, points: &[(f64, f64)]) -> MapObject {
    MapObject {
        symbol_id,
        kind: MapObjectKind::Path,
        coords: points.iter().map(|&(x, y)| coord(x, y)).collect(),
        tags: vec![],
        pattern: None,
    }
}

/// An area with one ring through the points.
pub(crate) fn area_object(symbol_id: u32, points: &[(f64, f64)]) -> MapObject {
    let ring: Vec<(i32, i32)> = points
        .iter()
        .map(|&(x, y)| ((x * 1000.0).round() as i32, (y * 1000.0).round() as i32))
        .collect();
    MapObject {
        symbol_id,
        kind: MapObjectKind::Path,
        coords: closed_ring(&ring),
        tags: vec![],
        pattern: None,
    }
}

/// The object tagged as content of the map sheet, which the map index leaves out.
pub(crate) fn sheet(mut object: MapObject) -> MapObject {
    object
        .tags
        .push(("freemapper:source".to_owned(), "sheet".to_owned()));
    object
}

fn map_with(symbols: &str, scale: u32, objects: Vec<MapObject>) -> MapDocument {
    MapDocument {
        notes: String::new(),
        georef: MapGeoref {
            scale_denominator: scale,
            declination_deg: 0.0,
            grivation_deg: 0.0,
            ref_projected: None,
            crs_spec: None,
        },
        symbol_set: SymbolSet::from_omap_xml(symbols).expect("test symbols parse"),
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

/// The rule with the values written as TOML, such as `metres = 15.0`.
pub(crate) fn with_values(mut rule: Rule, values: &str) -> Rule {
    rule.values = values.parse().expect("test values parse");
    rule
}
