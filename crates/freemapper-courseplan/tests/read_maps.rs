//! Reads the two real maps in `tests/fixtures/maps` and queries them.

use freemapper_courseplan::map::{MapIndex, MapKind, map_kind};
use freemapper_courseplan::omap::read::read_map_file;
use std::path::Path;

fn fixture(name: &str) -> String {
    format!(
        "{}/../../tests/fixtures/maps/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Number of `<object ` elements inside `<parts>`. Symbol definitions (point symbols) contain
/// `<object ` elements of their own, which are not map objects.
fn objects_in_parts(path: &str) -> usize {
    let xml = std::fs::read_to_string(path).unwrap();
    let start = xml.find("<parts ").expect("file has <parts>");
    let end = xml.rfind("</parts>").expect("file has </parts>");
    xml[start..end].matches("<object ").count()
}

#[test]
fn luebeck_reads() {
    let path = fixture("luebeck.omap");
    let doc = read_map_file(Path::new(&path)).unwrap();
    let read: usize = doc.parts.iter().map(|p| p.objects.len()).sum();
    assert_eq!(read, objects_in_parts(&path));
    assert_eq!(doc.georef.scale_denominator, 4000);
    assert_eq!(doc.georef.crs_spec.as_deref(), Some("+init=epsg:25832"));
    assert!(doc.georef.ref_projected.is_some());
}

#[test]
fn forest_sample_reads() {
    let path = fixture("forest-sample.omap");
    let doc = read_map_file(Path::new(&path)).unwrap();
    let read: usize = doc.parts.iter().map(|p| p.objects.len()).sum();
    assert_eq!(read, objects_in_parts(&path));
    assert_eq!(read, 539);
    assert_eq!(doc.georef.scale_denominator, 10000);
}

#[test]
fn map_kinds() {
    let luebeck = read_map_file(Path::new(&fixture("luebeck.omap"))).unwrap();
    let forest = read_map_file(Path::new(&fixture("forest-sample.omap"))).unwrap();
    assert_eq!(map_kind(&luebeck), MapKind::Sprint);
    assert_eq!(map_kind(&forest), MapKind::Forest);
}

#[test]
fn luebeck_index_skips_sheet_objects() {
    let doc = read_map_file(Path::new(&fixture("luebeck.omap"))).unwrap();
    let objects: usize = doc.parts.iter().map(|p| p.objects.len()).sum();
    let index = MapIndex::from_document(&doc);
    assert!(!index.is_empty());
    assert!(index.len() < objects, "{} of {objects}", index.len());
    let (min, max) = index.extent().unwrap();
    assert!(max.x > min.x && max.y > min.y);
    let centre = freemapper_courseplan_model::MapPoint {
        x: (min.x + max.x) / 2.0,
        y: (min.y + max.y) / 2.0,
    };
    let near = index.features_near(centre, 30.0);
    assert!(near.iter().all(|(f, _)| f.part != "peripheral information"));
    assert!(near.windows(2).all(|w| w[0].1 <= w[1].1));
}
