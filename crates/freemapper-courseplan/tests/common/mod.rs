//! Helpers shared by the `.ppen` import and export tests.
//!
//! The maps of the three Purple Pen test events of `tests/fixtures/ppen` (see the README there) are not in
//! the repository and none belongs to the Lübeck map, so each is fitted against a frame map: a map at the
//! file's scale whose only objects mark the corners of the controls. The fit, the mapping and the event
//! validation run as they do for a real map.

// Each test binary uses its own part of this module.
#![allow(dead_code)]

use freemapper_courseplan::omap::{
    MapCoord, MapDocument, MapGeoref, MapObject, MapObjectKind, MapPart, SymbolSet,
};
use freemapper_courseplan::ppen_import::{PpenImport, read};
use freemapper_courseplan_model::{
    Discipline, Event, EventMeta, FORMAT_VERSION, Sources, validate,
};

pub const FIXTURES: [&str; 3] = ["lincoln-park.ppen", "overlapper.ppen", "sampleevent8.ppen"];

const SYMBOLS: &str = r#"<map><georeferencing scale="1"/><colors count="0"></colors><symbols count="1" id="test"><symbol type="1" id="0" code="999" name="corner"><point_symbol/></symbol></symbols><view><grid/></view></map>"#;

pub fn bytes(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../tests/fixtures/ppen/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn corner(x: i32, y: i32) -> MapObject {
    MapObject {
        symbol_id: 0,
        kind: MapObjectKind::Point { rotation_rad: 0.0 },
        coords: vec![MapCoord::new(x, y)],
        tags: vec![],
        pattern: None,
    }
}

/// A map at the scale the file states, with a point at two opposite corners of its controls (the
/// file's y up turned into the map's y down).
pub fn frame_map(file: &[u8]) -> MapDocument {
    let raw = read(file).unwrap();
    let scale = raw
        .source
        .map_scale
        .expect("the fixture states its map scale");
    let (xs, ys): (Vec<f64>, Vec<f64>) = raw
        .controls
        .iter()
        .filter_map(|c| c.position)
        .map(|p| (p.x_mm * 1000.0, -p.y_mm * 1000.0))
        .unzip();
    let min = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min).floor() as i32;
    let max = |v: &[f64]| v.iter().copied().fold(f64::NEG_INFINITY, f64::max).ceil() as i32;
    MapDocument {
        notes: String::new(),
        georef: MapGeoref {
            scale_denominator: scale,
            ..MapGeoref::default()
        },
        symbol_set: SymbolSet::from_omap_xml(SYMBOLS).unwrap(),
        parts: vec![MapPart {
            name: "frame".into(),
            objects: vec![corner(min(&xs), min(&ys)), corner(max(&xs), max(&ys))],
        }],
        templates: vec![],
        unknown_objects: 0,
    }
}

/// The import as an event, which must pass the model's validation.
pub fn valid_event(import: &PpenImport) -> Event {
    let event = Event {
        format: FORMAT_VERSION,
        event: EventMeta {
            title: "Import".to_owned(),
            date: None,
            discipline: Discipline::Training,
            rule_profile: "wkb-2026".to_owned(),
            rule_overlays: vec![],
            start_window_min: None,
        },
        sources: Sources {
            map: "map.omap".into(),
            ppen: Some("event.ppen".into()),
        },
        classes: vec![],
        controls: import.controls.clone(),
        specials: import.specials.clone(),
        courses: import.courses.clone(),
        accepted_findings: vec![],
        sync: None,
    };
    assert_eq!(validate(&event), vec![]);
    event
}

pub mod purple_pen;
