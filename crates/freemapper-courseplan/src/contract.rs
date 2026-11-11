//! The reader side of the suite contract (ADR 0041): what freemapper-courseplan derives from the fixtures
//! of `freemapper-contract` equals their `expected.toml`. The fixtures are maps and sidecars exactly as
//! freemapper wrote them. Every test name starts with `contract_`; freemapper-core's CI will run them by
//! that prefix.

use std::path::PathBuf;

use freemapper_contract::{Expected, Fixture, fixtures};

use crate::init::title_from_config;
use crate::map::{MapIndex, MapKind, is_sheet_furniture, map_kind, map_kind_for_file};
use crate::omap::MapDocument;
use crate::omap::read::read_map_file;

/// A fixture in a folder of its own, read with freemapper-courseplan's reader.
struct Read {
    name: &'static str,
    dir: tempfile::TempDir,
    path: PathBuf,
    doc: MapDocument,
    expected: Expected,
}

fn read(fixture: &Fixture) -> Read {
    let dir = tempfile::tempdir().unwrap();
    let path = fixture.write_into(dir.path()).unwrap();
    let doc = read_map_file(&path).unwrap();
    Read {
        name: fixture.name,
        dir,
        path,
        doc,
        expected: fixture.expected(),
    }
}

fn each(test: impl Fn(&Read)) {
    assert!(!fixtures().is_empty());
    for fixture in fixtures() {
        test(&read(fixture));
    }
}

/// The kind the sidecar must give: `None` for the fixture whose kind no reader knows.
fn sidecar_kind(read: &Read) -> Option<MapKind> {
    let name = read.expected.map_kind.as_str();
    let kind = MapKind::from_name(name);
    assert!(kind.is_some() || name == "unknown", "{}: {name}", read.name);
    kind
}

#[test]
fn contract_kind() {
    each(|read| {
        let for_file = map_kind_for_file(&read.doc, &read.path);
        match sidecar_kind(read) {
            Some(kind) => assert_eq!(for_file, kind, "{}", read.name),
            // A kind courseplan does not know counts as absent: the map decides, and no error is raised.
            None => assert_eq!(for_file, map_kind(&read.doc), "{}", read.name),
        }
        // `export_ppen` has no path and asks the map alone; for every fixture the two agree.
        assert_eq!(map_kind(&read.doc), for_file, "{}", read.name);
    });
}

#[test]
fn contract_title() {
    each(|read| {
        assert_eq!(
            title_from_config(read.dir.path()),
            read.expected.title,
            "{}",
            read.name
        );
    });
}

#[test]
fn contract_classes() {
    each(|read| {
        let objects = || read.doc.parts.iter().flat_map(|p| &p.objects);
        let counts = read.expected.objects;
        assert_eq!(
            objects().filter(|o| is_sheet_furniture(&o.tags)).count(),
            counts.sheet + counts.base,
            "{}: sheet objects and base area",
            read.name
        );
        assert_eq!(
            objects().count(),
            counts.sheet + counts.base + counts.north_lines + counts.content,
            "{}: all objects",
            read.name
        );
    });
}

#[test]
fn contract_extent() {
    each(|read| {
        let index = MapIndex::from_document(&read.doc);
        // Neither the sheet objects nor the base area are features.
        assert!(
            index
                .features()
                .iter()
                .all(|f| f.part != "peripheral information" && f.part != "base"),
            "{}",
            read.name
        );
        let (min, max) = index.extent().unwrap();
        // Map coordinates are 1/1000 mm, the index counts millimetres; both have y pointing south.
        let content = read
            .expected
            .bbox
            .content
            .unwrap()
            .map(|v| f64::from(v) / 1000.0);
        let extent = [min.x, min.y, max.x, max.y];
        assert!(
            extent[0] >= content[0]
                && extent[1] >= content[1]
                && extent[2] <= content[2]
                && extent[3] <= content[3],
            "{}: extent {extent:?}, content {content:?}",
            read.name
        );
        let sheet = read
            .expected
            .bbox
            .sheet
            .unwrap()
            .map(|v| f64::from(v) / 1000.0);
        assert!(
            extent[3] < sheet[1],
            "{}: extent {extent:?}, sheet objects {sheet:?}",
            read.name
        );
    });
}
