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

fn kind(read: &Read) -> MapKind {
    *MapKind::ALL
        .iter()
        .find(|k| k.as_str() == read.expected.map_kind)
        .unwrap_or_else(|| panic!("{}: unknown kind {}", read.name, read.expected.map_kind))
}

#[test]
fn contract_kind() {
    each(|read| {
        assert_eq!(
            map_kind_for_file(&read.doc, &read.path),
            kind(read),
            "{}",
            read.name
        );
        // `export_ppen` has no path and asks the map alone; for what freemapper writes today the two agree.
        assert_eq!(map_kind(&read.doc), kind(read), "{}", read.name);
    });
}

#[test]
fn contract_title() {
    each(|read| {
        assert_eq!(
            title_from_config(read.dir.path()),
            Some(read.expected.title.clone()),
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
