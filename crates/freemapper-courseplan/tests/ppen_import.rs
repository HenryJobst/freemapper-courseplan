//! Imports the three Purple Pen test events of `tests/fixtures/ppen` (see the README there).
//!
//! Their maps are not in the repository and none belongs to the Lübeck map, so each is read on its own
//! and fitted against a frame map: a map at the file's scale whose only objects mark the corners of the
//! controls. The fit, the mapping and the event validation run as they do for a real map.

use std::path::Path;

use freemapper_courseplan::map::map_kind;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::omap::{
    MapCoord, MapDocument, MapGeoref, MapObject, MapObjectKind, MapPart, SymbolSet,
};
use freemapper_courseplan::ppen_import::{
    ImportNote, PpenImport, PpenImportError, import_ppen, read,
};
use freemapper_courseplan_model::{
    ControlKind, Course, CourseControl, CourseKind, Discipline, Event, EventMeta, FORMAT_VERSION,
    NumberPlacement, Sources, SpecialObject, validate,
};

const SYMBOLS: &str = r#"<map><georeferencing scale="1"/><colors count="0"></colors><symbols count="1" id="test"><symbol type="1" id="0" code="999" name="corner"><point_symbol/></symbol></symbols><view><grid/></view></map>"#;

fn bytes(name: &str) -> Vec<u8> {
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
fn frame_map(file: &[u8]) -> MapDocument {
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

fn import(name: &str) -> PpenImport {
    let file = bytes(name);
    import_ppen(&file, &frame_map(&file)).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// The import as an event, which must pass the model's validation.
fn valid_event(import: &PpenImport) -> Event {
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
    };
    assert_eq!(validate(&event), vec![]);
    event
}

fn course<'a>(import: &'a PpenImport, name: &str) -> &'a Course {
    import.courses.iter().find(|c| c.name == name).unwrap()
}

fn stops(course: &Course) -> Vec<&str> {
    match &course.kind {
        CourseKind::Individual { controls } => controls.iter().map(|c| c.code.0.as_str()).collect(),
        other => panic!("{other:?}"),
    }
}

fn course_controls(course: &Course) -> &[CourseControl] {
    match &course.kind {
        CourseKind::Individual { controls } => controls,
        other => panic!("{other:?}"),
    }
}

fn description<'a>(import: &'a PpenImport, code: &str) -> [Option<&'a str>; 6] {
    let d = &import
        .controls
        .iter()
        .find(|c| c.code.0 == code)
        .unwrap()
        .description;
    [&d.c, &d.d, &d.e, &d.f, &d.g, &d.h].map(|v| v.as_deref())
}

#[test]
fn lincoln_park_imports() {
    let got = import("lincoln-park.ppen");
    valid_event(&got);
    let short = course(&got, "Short");
    assert_eq!(
        (short.start.as_str(), short.finish.as_str()),
        ("STA1", "FIN1")
    );
    assert_eq!(stops(short), ["31", "32"]);
    assert_eq!(short.planned_print_scale, Some(5000));
    assert_eq!(short.ppen_id, Some(1));
    // The leg 32 → finish has one bend.
    assert_eq!(course_controls(short)[1].bends.len(), 1);
    assert_eq!(description(&got, "31")[1], Some("1.10"));
    assert_eq!(
        description(&got, "32"),
        [None, Some("5.2"), Some("5.2"), Some("10.2"), None, None]
    );
    assert_eq!(
        got.notes,
        vec![
            ImportNote::StartFinishDescription {
                id: "FIN1".to_owned()
            },
            ImportNote::GapsNotSupported {
                course: "Short".to_owned()
            },
        ]
    );
}

#[test]
fn overlapper_imports() {
    let got = import("overlapper.ppen");
    valid_event(&got);
    let names: Vec<&str> = got.courses.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Course 1", "Course 2"]);
    let cross = got.controls.iter().find(|c| c.code.0 == "CROSS1").unwrap();
    assert_eq!(cross.kind, ControlKind::CrossingPoint);
    assert_eq!(cross.description.c.as_deref(), Some("15.3"));
    assert!(
        got.notes.contains(&ImportNote::Skipped {
            course: "Score".to_owned(),
            why: "score course",
        }),
        "{:?}",
        got.notes
    );
    // Only Course 1 numbers 38, and it places the number: a central placement. The file offset
    // (4.28, -3.376) mm, y up, at print scale 7500 on a 1:10000 map (ratio 0.75) points south-east; the
    // forest circle (5 mm, line 0.35 mm) and the two-digit box of number 12 give the gap.
    let c38 = got.controls.iter().find(|c| c.code.0 == "38").unwrap();
    assert_eq!(
        c38.number,
        Some(NumberPlacement {
            direction: 321.73,
            gap_mm: 1.47,
        })
    );
    assert!(
        course_controls(course(&got, "Course 1"))
            .iter()
            .all(|c| c.number.is_none())
    );
    assert!(
        got.specials.iter().any(
            |s| matches!(s, SpecialObject::ForbiddenRoute { points, .. } if points.len() == 1)
        )
    );
    assert!(
        got.specials
            .iter()
            .any(|s| matches!(s, SpecialObject::Water { .. }))
    );
}

#[test]
fn import_keeps_special_and_start_ids() {
    let got = import("sampleevent8.ppen");
    let id_of = |id: &str| {
        got.specials
            .iter()
            .find(|s| s.id() == id)
            .unwrap_or_else(|| panic!("{id} in {:?}", got.specials))
            .ppen_id()
    };
    assert_eq!(id_of("STA1"), Some(1));
    assert_eq!(id_of("FIN1"), Some(2));
    assert_eq!(id_of("WATER1"), Some(4));
}

#[test]
fn sampleevent8_imports() {
    let got = import("sampleevent8.ppen");
    valid_event(&got);
    let exchanges = course(&got, "Exchanges");
    assert!(!stops(exchanges).contains(&"XCHG1"));
    // 2018 descriptions: the railway 5.26 is 5.24 in 2024, the bend 11.7 is 11.1.
    assert_eq!(description(&got, "41")[1], Some("5.24"));
    assert_eq!(description(&got, "41")[3], Some("11.1"));
    assert_eq!(description(&got, "43")[0], Some("0.1E"));
    assert_eq!(description(&got, "45")[4], Some("12.6SW"));
    for note in [
        ImportNote::MapExchange {
            course: "Exchanges".to_owned(),
            control: "XCHG1".to_owned(),
        },
        ImportNote::MapExchangeAtControl {
            course: "Exchanges".to_owned(),
            control: "41".to_owned(),
        },
        ImportNote::ColumnFText {
            control: "53".to_owned(),
            text: "2".to_owned(),
        },
    ] {
        assert!(got.notes.contains(&note), "{note:?} in {:?}", got.notes);
    }
}

#[test]
fn a_ppen_of_another_map_does_not_fit_luebeck() {
    let map = read_map_file(Path::new(&format!(
        "{}/../../tests/fixtures/maps/luebeck.omap",
        env!("CARGO_MANIFEST_DIR")
    )))
    .unwrap();
    assert_eq!(map_kind(&map), freemapper_courseplan::map::MapKind::Sprint);
    let err = import_ppen(&bytes("lincoln-park.ppen"), &map).unwrap_err();
    assert!(
        matches!(err, PpenImportError::Fit(_)),
        "a 1:5000 file on the 1:4000 map: {err:?}"
    );
}
