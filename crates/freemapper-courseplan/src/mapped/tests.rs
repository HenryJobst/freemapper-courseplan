//! Tests of the mapped area and its margin (ADR 0087). Maps are at 1:1000, so one map millimetre is one metre
//! on the ground, and the raster cells are 2 m: a margin is right to about 2 m.

use freemapper_courseplan_model::MapPoint;

use super::*;
use crate::map::{FeatureGeometry, MapFeature};
use crate::rules::load_rule_set;

fn p(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn feature(code: &str, geometry: FeatureGeometry) -> MapFeature {
    MapFeature {
        symbol_code: Some(code.to_owned()),
        part: "p".into(),
        geometry,
    }
}

fn square(code: &str, x0: f64, y0: f64, x1: f64, y1: f64) -> MapFeature {
    feature(
        code,
        FeatureGeometry::Area(vec![vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)]]),
    )
}

/// A building of 300 by 300 m: the mapped area reaches 50 m beyond it.
fn one_building() -> MapIndex {
    MapIndex::from_features(vec![square("521", 0.0, 0.0, 300.0, 300.0)])
}

fn area(index: &MapIndex) -> MappedArea {
    MappedArea::build(index, 1000).expect("the map has objects")
}

#[test]
fn the_transform_gives_exact_squared_distances() {
    // A 7 by 5 grid with a seed at column 1, row 1 and another at column 5, row 3.
    let (w, h) = (7, 5);
    let mut seed = vec![false; w * h];
    seed[w + 1] = true;
    seed[3 * w + 5] = true;
    let d = squared_distance(&seed, w, h);
    for row in 0..h {
        for col in 0..w {
            let want = [(1_i64, 1_i64), (5, 3)]
                .iter()
                .map(|(c, r)| (col as i64 - c).pow(2) + (row as i64 - r).pow(2))
                .min()
                .unwrap() as f64;
            assert_eq!(d[row * w + col], want, "col {col}, row {row}");
        }
    }
}

#[test]
fn the_margin_is_zero_in_the_blank_part_and_grows_inward() {
    let a = area(&one_building());
    // 100 m beside the building, beyond the 50 m of the mapped area.
    assert_eq!(a.margin_m(&p(-100.0, 150.0)), 0.0);
    // 100 m inside the building the nearest unmapped place is 150 m away (50 m beyond the wall).
    let deep = a.margin_m(&p(100.0, 150.0));
    assert!((deep - 150.0).abs() <= 2.5, "{deep}");
    let centre = a.margin_m(&p(150.0, 150.0));
    assert!((centre - 200.0).abs() <= 2.5, "{centre}");
}

#[test]
fn just_under_on_and_just_over_the_boundary() {
    let a = area(&one_building());
    // The wall at x = 0; the edge of the mapped area at x = -50. A place at x gets the margin x + 50.
    for (x, want) in [(-4.0, 46.0), (0.0, 50.0), (4.0, 54.0)] {
        let m = a.margin_m(&p(x, 150.0));
        assert!((m - want).abs() <= 2.5, "x {x}: {m}, wanted {want}");
    }
    // The edge itself is at 50 m beyond the wall, and a place there has no margin.
    assert!(a.margin_m(&p(-50.0, 150.0)) <= 2.5);
    assert_eq!(a.margin_m(&p(-54.0, 150.0)), 0.0);
}

#[test]
fn the_edge_of_the_raster_is_the_edge_of_the_mapped_area() {
    // Far from the objects the raster ends, and so does the mapped area.
    let a = area(&one_building());
    assert_eq!(a.margin_m(&p(100_000.0, 0.0)), 0.0);
}

#[test]
fn a_gap_between_buildings_is_mapped_and_deep() {
    // Two buildings 60 m apart: the gap is within 50 m of an object everywhere, so it is mapped and a place in
    // it is far from the edge along the gap too.
    let index = MapIndex::from_features(vec![
        square("521", 0.0, 0.0, 100.0, 300.0),
        square("521", 160.0, 0.0, 260.0, 300.0),
    ]);
    let a = area(&index);
    let m = a.margin_m(&p(130.0, 150.0));
    assert!(m > 100.0, "{m}");
}

#[test]
fn layout_symbols_and_the_frame_are_no_terrain() {
    // The north lines pattern covers the whole sheet and a frame line runs round it; only the building counts.
    let pattern = square("601.6", -500.0, -500.0, 800.0, 800.0);
    let frame = feature(
        "501.1",
        FeatureGeometry::Line(vec![
            p(0.0, 0.0),
            p(300.0, 0.0),
            p(300.0, 300.0),
            p(0.0, 300.0),
            p(0.0, 0.0),
        ]),
    );
    let mut features = vec![square("521", 0.0, 0.0, 300.0, 300.0), pattern, frame];
    // Enough objects for the frame test: a handful of small ones inside the building, none at the edge.
    for k in 0..60 {
        features.push(feature(
            "204",
            FeatureGeometry::Point(p(100.0 + f64::from(k), 150.0)),
        ));
    }
    let index = MapIndex::from_features(features);
    let a = area(&index);
    assert_eq!(a.margin_m(&p(-100.0, 150.0)), 0.0, "the pattern is ignored");
}

#[test]
fn a_map_without_terrain_has_no_mapped_area() {
    assert!(MappedArea::build(&MapIndex::from_features(vec![]), 1000).is_none());
    let only_layout = MapIndex::from_features(vec![square("601.6", 0.0, 0.0, 100.0, 100.0)]);
    assert!(MappedArea::build(&only_layout, 1000).is_none());
}

#[test]
fn the_mapped_metres_match_the_rule_inside_map() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let rule = rules
        .rules
        .iter()
        .find(|r| r.rule.id == "wkb2026.map.inside")
        .expect("the rule is in the profile");
    assert_eq!(rule.rule.values["metres"].as_float(), Some(MAPPED_METRES));
}

#[test]
fn a_large_map_gets_coarser_cells_instead_of_failing() {
    // 20 km square at 1:1000 would need 100 million cells of 2 m; the raster grows its cells.
    let index = MapIndex::from_features(vec![square("521", 0.0, 0.0, 20_000.0, 20_000.0)]);
    let a = area(&index);
    assert!(a.cell_m > CELL_M);
    assert!(a.margin_m(&p(10_000.0, 10_000.0)) >= MAX_MARGIN_M - 1.0);
}

#[test]
fn a_longer_reach_maps_more_ground() {
    // A building of 300 by 300 m at 1:1000: with the default reach the mapped area ends 50 m beyond it,
    // with a reach of 100 m 100 m beyond it.
    let index = one_building();
    let short = MappedArea::build(&index, 1000).unwrap();
    let long = MappedArea::build_with_reach(&index, 1000, 100.0).unwrap();
    let at = p(375.0, 150.0);
    assert_eq!(short.margin_m(&at), 0.0);
    assert!(long.margin_m(&at) > 0.0);
    assert!(long.margin_m(&p(150.0, 150.0)) > short.margin_m(&p(150.0, 150.0)));
}
