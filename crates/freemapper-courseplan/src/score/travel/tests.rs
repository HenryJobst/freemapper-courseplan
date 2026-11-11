use freemapper_courseplan_model::MapPoint;

use super::*;
use crate::map::{FeatureGeometry, MapFeature};

/// Scale 1:1000, so one map millimetre is one metre on the ground.
const SCALE: u32 = 1000;
const PACE: Pace = Pace(360);

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

fn rect(code: &str, x0: f64, y0: f64, x1: f64, y1: f64) -> MapFeature {
    feature(
        code,
        FeatureGeometry::Area(vec![vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)]]),
    )
}

fn line(code: &str, points: &[(f64, f64)]) -> MapFeature {
    feature(
        code,
        FeatureGeometry::Line(points.iter().map(|&(x, y)| p(x, y)).collect()),
    )
}

/// A 200 m by 200 m open field (open land, factor 1.0) plus `more`.
fn field(more: Vec<MapFeature>) -> MapIndex {
    let mut features = vec![rect("401", 0.0, 0.0, 200.0, 200.0)];
    features.extend(more);
    MapIndex::from_features(features)
}

fn surface(index: &MapIndex, blocked: &[Vec<MapPoint>]) -> CostSurface {
    CostSurface::build(index, SCALE, blocked, PACE, &RouteParams::default()).unwrap()
}

#[test]
fn straight_open_field_is_a_straight_route() {
    let s = surface(&field(vec![]), &[]);
    let routes = s.legs(p(20.0, 100.0), p(180.0, 100.0)).unwrap().routes;
    assert_eq!(routes.len(), 1, "an open field has no alternative");
    assert_eq!(routes[0].points.len(), 2);
    assert!((routes[0].length_m - 160.0).abs() < 1e-9);
    // 160 m at 6:00 min/km is 57.6 s; the way to the cell centres adds up to two metres.
    assert!(
        (routes[0].time_s - 57.6).abs() < 2.0,
        "{}",
        routes[0].time_s
    );
}

#[test]
fn a_building_in_between_forces_a_detour() {
    let index = field(vec![rect("521", 85.0, 85.0, 115.0, 115.0)]);
    let routes = surface(&index, &[])
        .legs(p(20.0, 100.0), p(180.0, 100.0))
        .unwrap()
        .routes;
    let best = &routes[0];
    assert!(
        best.length_m > 160.0 && best.length_m < 200.0,
        "{}",
        best.length_m
    );
    let building = [
        p(85.0, 85.0),
        p(115.0, 85.0),
        p(115.0, 115.0),
        p(85.0, 115.0),
        p(85.0, 85.0),
    ];
    assert!(
        best.points
            .iter()
            .all(|q| !crate::map::inside_area(q, &[building.to_vec()]))
    );
}

#[test]
fn two_sides_of_a_building_give_two_alternatives() {
    let index = field(vec![rect("521", 85.0, 70.0, 115.0, 130.0)]);
    let params = RouteParams::default();
    let routes = surface(&index, &[])
        .legs(p(20.0, 100.0), p(180.0, 100.0))
        .unwrap()
        .routes;
    assert!(routes.len() >= 2, "{routes:?}");
    let best = routes[0].length_m;
    for r in &routes {
        assert!(r.length_m <= best * params.max_length_ratio + 1e-9);
    }
    // One passes above the building, one below.
    let above = routes.iter().any(|r| r.points.iter().any(|q| q.y < 70.0));
    let below = routes.iter().any(|r| r.points.iter().any(|q| q.y > 130.0));
    assert!(above && below, "{routes:?}");
}

#[test]
fn a_shifted_copy_is_not_an_alternative() {
    // A small obstacle in the middle: the second best route is the first one moved a few metres.
    let index = field(vec![rect("521", 98.0, 98.0, 102.0, 102.0)]);
    let routes = surface(&index, &[])
        .legs(p(20.0, 100.0), p(180.0, 100.0))
        .unwrap()
        .routes;
    assert_eq!(routes.len(), 1, "{routes:?}");
}

#[test]
fn blocked_special_objects_are_avoided() {
    let zone = vec![
        p(85.0, 60.0),
        p(115.0, 60.0),
        p(115.0, 140.0),
        p(85.0, 140.0),
    ];
    let s = surface(&field(vec![]), std::slice::from_ref(&zone));
    assert!(!s.passable(p(100.0, 100.0)));
    assert!(s.passable(p(50.0, 100.0)));
    let best = &s.legs(p(20.0, 100.0), p(180.0, 100.0)).unwrap().routes[0];
    assert!(best.length_m > 160.0);
    assert!(
        best.points
            .iter()
            .all(|q| !crate::map::inside_area(q, std::slice::from_ref(&zone)))
    );
    // A forbidden route is a line.
    let wall = vec![p(100.0, 0.0), p(100.0, 160.0)];
    let s = surface(&field(vec![]), &[wall]);
    let best = &s.legs(p(20.0, 100.0), p(180.0, 100.0)).unwrap().routes[0];
    assert!(best.points.iter().any(|q| q.y > 160.0), "{best:?}");
}

#[test]
fn faster_ground_wins() {
    let open = surface(&field(vec![]), &[]);
    let road = surface(
        &field(vec![line("503", &[(0.0, 100.0), (200.0, 100.0)])]),
        &[],
    );
    let (a, b) = (p(20.0, 100.0), p(180.0, 100.0));
    let slow = open.legs(a.clone(), b.clone()).unwrap().routes[0].time_s;
    let fast = road.legs(a, b).unwrap().routes[0].time_s;
    assert!(fast < slow * 0.9, "{fast} vs {slow}");
    // Forest is slower than open land.
    let forest = surface(&field(vec![rect("405", 0.0, 0.0, 200.0, 200.0)]), &[]);
    let wood = forest.legs(p(20.0, 100.0), p(180.0, 100.0)).unwrap().routes[0].time_s;
    assert!(wood > slow * 1.5, "{wood} vs {slow}");
}

#[test]
fn enclosed_point_gives_a_sentence_not_a_panic() {
    let wall = line(
        "515",
        &[
            (90.0, 90.0),
            (110.0, 90.0),
            (110.0, 110.0),
            (90.0, 110.0),
            (90.0, 90.0),
        ],
    );
    let s = surface(&field(vec![wall]), &[]);
    let err = s.legs(p(100.0, 100.0), p(180.0, 180.0)).unwrap_err();
    assert!(matches!(err, ScoreError::Unreachable { .. }), "{err:?}");
    assert!(err.to_string().contains("no route"));
    // Outside the raster is unreachable too.
    assert!(matches!(
        s.legs(p(-500.0, 0.0), p(1.0, 1.0)),
        Err(ScoreError::Unreachable { .. })
    ));
}

#[test]
fn same_input_same_output() {
    let index = field(vec![
        rect("521", 85.0, 70.0, 115.0, 130.0),
        rect("405", 130.0, 0.0, 200.0, 60.0),
    ]);
    let first = surface(&index, &[])
        .legs(p(20.0, 100.0), p(180.0, 100.0))
        .unwrap();
    let second = surface(&index, &[])
        .legs(p(20.0, 100.0), p(180.0, 100.0))
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn an_empty_map_and_bad_settings_are_errors() {
    let empty = MapIndex::from_features(vec![]);
    assert_eq!(
        CostSurface::build(&empty, SCALE, &[], PACE, &RouteParams::default()).err(),
        Some(ScoreError::MapEmpty)
    );
    let bad = RouteParams {
        cell_m: 0.0,
        ..RouteParams::default()
    };
    assert!(matches!(
        CostSurface::build(&field(vec![]), SCALE, &[], PACE, &bad),
        Err(ScoreError::BadRouteSetting { name: "cell_m", .. })
    ));
}

#[test]
fn a_point_inside_a_building_snaps_to_the_edge() {
    let index = field(vec![rect("521", 85.0, 85.0, 115.0, 115.0)]);
    let s = surface(&index, &[]);
    // Two metres inside the wall: still found, by the nearest free cell.
    assert!(s.legs(p(87.0, 100.0), p(180.0, 100.0)).is_ok());
}

/// Prints the build time on the Lübeck example map (ADR 0061). Run with `--ignored --nocapture`.
#[test]
#[ignore = "timing run on a real map"]
fn luebeck_build_time() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/maps/luebeck.omap"
    );
    let doc = crate::omap::read::read_map_file(std::path::Path::new(path)).unwrap();
    let index = MapIndex::from_document(&doc);
    let scale = doc.georef.scale_denominator;
    let started = std::time::Instant::now();
    let s = CostSurface::build(&index, scale, &[], Pace(250), &RouteParams::default()).unwrap();
    let built = started.elapsed();
    let (lo, hi) = index.extent().unwrap();
    let started = std::time::Instant::now();
    let routes = s.legs(p(lo.x + 10.0, lo.y + 10.0), p(hi.x - 10.0, hi.y - 10.0));
    let searched = started.elapsed();
    println!(
        "cells {}x{}, build {built:?}, one corner-to-corner leg {searched:?}, {:?} routes",
        s.grid.width,
        s.grid.height,
        routes.map(|r| r.routes.len())
    );
}
