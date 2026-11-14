//! `init` on forest maps: ISOM 2017-2 is accepted, other forest symbol sets are refused (ADR 0092).

use std::path::PathBuf;

use freemapper_courseplan::init::{InitError, new_event};
use freemapper_courseplan::map::{ForestSymbolSet, MapKind, forest_symbol_set, map_kind_for_file};
use freemapper_courseplan::omap::read::read_map_file;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(format!(
        "{}/../../tests/fixtures/maps/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
}

/// A copy of `name` in a temporary folder, so a sidecar can sit next to it.
fn copy_map(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join(name);
    std::fs::copy(fixture(name), &target).unwrap();
    (dir, target)
}

#[test]
fn init_accepts_isom_2017_2_forest_map() {
    let map = fixture("isom2017-synthetic.omap");
    let doc = read_map_file(&map).unwrap();
    assert_eq!(forest_symbol_set(&doc), ForestSymbolSet::Isom2017_2);
    assert_eq!(map_kind_for_file(&doc, &map), MapKind::Forest);
    let dir = tempfile::tempdir().unwrap();
    let event = new_event(&map, &dir.path().join("e.courseplan.toml"), None).unwrap();
    assert_eq!(event.event.rule_profile, "wkb-2026");
}

#[test]
fn init_refuses_isom_2000_forest_map() {
    let map = fixture("forest-sample.omap");
    let doc = read_map_file(&map).unwrap();
    assert_eq!(forest_symbol_set(&doc), ForestSymbolSet::Isom2000);
    let dir = tempfile::tempdir().unwrap();
    let err = new_event(&map, &dir.path().join("e.courseplan.toml"), None).unwrap_err();
    assert!(matches!(
        err,
        InitError::ForestSymbolSetNotSupported { ref found } if found == "ISOM2000"
    ));
    let text = err.to_string();
    assert!(
        text.contains("ISOM2000") && text.contains("ISOM 2017-2"),
        "{text}"
    );
}

#[test]
fn init_refuses_forest_sidecar_with_unknown_symbol_set() {
    // A sprint map (ISSprOM) that a sidecar declares as a forest map.
    let (dir, map) = copy_map("luebeck.omap");
    std::fs::write(
        dir.path().join("freemapper.toml"),
        "map_kind = \"forest\"\n",
    )
    .unwrap();
    let doc = read_map_file(&map).unwrap();
    assert_eq!(map_kind_for_file(&doc, &map), MapKind::Forest);
    assert_eq!(
        forest_symbol_set(&doc),
        ForestSymbolSet::Unknown("ISSprOM 2019".to_owned())
    );
    let err = new_event(&map, &dir.path().join("e.courseplan.toml"), None).unwrap_err();
    assert!(
        matches!(err, InitError::ForestSymbolSetNotSupported { ref found } if found == "ISSprOM 2019"),
        "{err}"
    );
}

#[test]
fn symbol_set_names_are_told_apart() {
    use freemapper_courseplan::map::forest_symbol_set_of_name as of;
    assert_eq!(of("ISOM 2017-2"), ForestSymbolSet::Isom2017_2);
    assert_eq!(of("isom2017-2"), ForestSymbolSet::Isom2017_2);
    assert_eq!(of("ISOM2000"), ForestSymbolSet::Isom2000);
    assert_eq!(of("ISOM 2000"), ForestSymbolSet::Isom2000);
    // The first edition is not accepted (user: only ISOM 2017-2).
    assert_eq!(of("ISOM2017"), ForestSymbolSet::Unknown("ISOM2017".into()));
    assert_eq!(of(""), ForestSymbolSet::Unknown(String::new()));
}

#[test]
fn isom_sprint_at_1_4000_is_still_sprint() {
    // An ISOM-named map at 1:4000 counts as sprint, as before (ADR 0092).
    let dir = tempfile::tempdir().unwrap();
    let xml = std::fs::read_to_string(fixture("isom2017-synthetic.omap")).unwrap();
    let map = dir.path().join("isom4000.omap");
    std::fs::write(
        &map,
        xml.replace(
            "<georeferencing scale=\"10000\"",
            "<georeferencing scale=\"4000\"",
        ),
    )
    .unwrap();
    let doc = read_map_file(&map).unwrap();
    assert_eq!(map_kind_for_file(&doc, &map), MapKind::Sprint);
    new_event(&map, &dir.path().join("e.courseplan.toml"), None).unwrap();
}

#[test]
fn issprom_sprint_still_inits() {
    let map = fixture("luebeck.omap");
    let dir = tempfile::tempdir().unwrap();
    new_event(&map, &dir.path().join("e.courseplan.toml"), None).unwrap();
}
