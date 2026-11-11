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

/// The field inside a closed wall frame, so nothing can walk around the ends of a divider.
fn framed(more: Vec<MapFeature>) -> MapIndex {
    let mut features = vec![
        rect("401", 0.0, 0.0, 200.0, 200.0),
        line(
            "515",
            &[
                (-10.0, -10.0),
                (210.0, -10.0),
                (210.0, 210.0),
                (-10.0, 210.0),
                (-10.0, -10.0),
            ],
        ),
    ];
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
    // Slow vegetation is slower than open land; forest with good runnability barely is.
    let forest = surface(&field(vec![rect("406", 0.0, 0.0, 200.0, 200.0)]), &[]);
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
    // On the outline the point snaps to a free neighbour cell; two metres inside it cannot
    // leave without crossing the wall.
    assert!(s.legs(p(85.2, 100.0), p(180.0, 100.0)).is_ok());
    assert!(matches!(
        s.legs(p(95.0, 100.0), p(180.0, 100.0)),
        Err(ScoreError::Unreachable { .. })
    ));
}

#[test]
fn short_leg_with_a_building_gives_two_routes() {
    let params = RouteParams::default();
    // 50 m leg, 15 m building in the middle.
    let index = field(vec![rect("521", 77.0, 92.0, 92.0, 107.0)]);
    let routes = surface(&index, &[])
        .legs(p(60.0, 100.0), p(110.0, 100.0))
        .unwrap()
        .routes;
    assert_eq!(routes.len(), 2, "{routes:?}");
    assert!(routes[1].length_m <= routes[0].length_m * params.max_length_ratio);
    // 30 m leg, 10 m building.
    let index = field(vec![rect("521", 90.0, 95.0, 100.0, 105.0)]);
    let routes = surface(&index, &[])
        .legs(p(75.0, 100.0), p(105.0, 100.0))
        .unwrap()
        .routes;
    assert_eq!(routes.len(), 2, "{routes:?}");
    assert!(routes[1].length_m <= routes[0].length_m * params.max_length_ratio);
}

#[test]
fn max_routes_and_length_ratio_limit_the_alternatives() {
    // The building is off centre: the route below it is a little longer than the one above.
    let index = field(vec![rect("521", 85.0, 80.0, 115.0, 130.0)]);
    let count = |params: RouteParams| {
        let s = CostSurface::build(&index, SCALE, &[], PACE, &params).unwrap();
        s.legs(p(20.0, 100.0), p(180.0, 100.0))
            .unwrap()
            .routes
            .len()
    };
    let default = RouteParams::default();
    assert!(count(default.clone()) >= 2);
    assert_eq!(
        count(RouteParams {
            max_routes: 1,
            ..default.clone()
        }),
        1
    );
    assert_eq!(
        count(RouteParams {
            max_routes: 2,
            ..default.clone()
        }),
        2
    );
    assert_eq!(
        count(RouteParams {
            max_length_ratio: 1.0,
            ..default
        }),
        1
    );
}

#[test]
fn crossing_points_open_walls_but_not_buildings() {
    let (a, b) = (p(20.0, 100.0), p(180.0, 100.0));
    let wall = line("515", &[(100.0, -10.0), (100.0, 210.0)]);
    let closed = surface(&framed(vec![wall.clone()]), &[]);
    assert!(matches!(
        closed.legs(a.clone(), b.clone()),
        Err(ScoreError::Unreachable { .. })
    ));
    let gate = feature("519", FeatureGeometry::Point(p(100.0, 100.0)));
    let open = surface(&framed(vec![wall, gate.clone()]), &[]);
    let routes = open.legs(a.clone(), b.clone()).unwrap().routes;
    assert!((routes[0].length_m - 160.0).abs() < 4.0, "{routes:?}");
    // A building is solid: a crossing point on its outline does not open it.
    let house = rect("521", 95.0, -10.0, 105.0, 210.0);
    let solid = surface(&framed(vec![house, gate]), &[]);
    assert!(!solid.passable(p(100.0, 100.0)));
    assert!(matches!(
        solid.legs(a, b),
        Err(ScoreError::Unreachable { .. })
    ));
}

#[test]
fn a_bridge_crosses_water() {
    let (a, b) = (p(20.0, 100.0), p(180.0, 100.0));
    let river = rect("301", 90.0, -10.0, 110.0, 210.0);
    let wet = surface(&framed(vec![river.clone()]), &[]);
    assert!(matches!(
        wet.legs(a.clone(), b.clone()),
        Err(ScoreError::Unreachable { .. })
    ));
    let bridge = line("512", &[(85.0, 100.0), (115.0, 100.0)]);
    let dry = surface(&framed(vec![river, bridge]), &[]);
    assert!(dry.passable(p(100.0, 100.0)));
    assert!(dry.legs(a, b).is_ok());
}

#[test]
fn a_blocked_destination_is_unreachable() {
    let s = surface(&field(vec![rect("521", 100.0, 100.0, 140.0, 140.0)]), &[]);
    assert!(matches!(
        s.legs(p(20.0, 20.0), p(120.0, 120.0)),
        Err(ScoreError::Unreachable { .. })
    ));
}

#[test]
fn a_start_on_a_thin_wall_snaps_to_a_free_cell() {
    let wall = line("515", &[(100.0, 0.0), (100.0, 200.0)]);
    let gate = feature("519", FeatureGeometry::Point(p(100.0, 190.0)));
    let s = surface(&field(vec![wall, gate]), &[]);
    let from = p(100.2, 100.0);
    assert!(!s.passable(from.clone()));
    let routes = s.legs(from.clone(), p(60.0, 100.0)).unwrap().routes;
    assert_eq!(routes[0].points[0], from);
}

#[test]
fn absurd_extents_are_refused_without_a_panic() {
    let index = field(vec![]);
    let build = |ring: Vec<MapPoint>| {
        CostSurface::build(&index, SCALE, &[ring], PACE, &RouteParams::default()).err()
    };
    let far = build(vec![p(0.0, 0.0), p(1e12, 0.0), p(1e12, 1.0)]);
    assert!(
        matches!(far, Some(ScoreError::GridTooLarge { .. })),
        "{far:?}"
    );
    for bad in [f64::NAN, f64::INFINITY] {
        let err = build(vec![p(bad, 0.0), p(1.0, 1.0), p(2.0, 2.0)]);
        assert!(
            matches!(
                err,
                Some(ScoreError::BadRouteSetting {
                    name: "blocked",
                    ..
                })
            ),
            "{bad}"
        );
    }
    let fine = RouteParams {
        cell_m: 0.001,
        ..RouteParams::default()
    };
    assert!(matches!(
        CostSurface::build(&index, SCALE, &[], PACE, &fine),
        Err(ScoreError::GridTooLarge { .. })
    ));
}

#[test]
fn speeds_table_has_no_duplicate_codes_and_sane_factors() {
    let table = raster::Table::embedded();
    let mut seen = std::collections::HashSet::new();
    for (code, kind) in &table.0 {
        assert!(seen.insert(code.clone()), "{code} is in two classes");
        if let raster::Kind::Ground(f) | raster::Kind::Fast(f) = kind {
            assert!((0.5..=4.0).contains(f), "{code}: {f}");
        }
    }
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

#[test]
fn fastest_is_the_first_of_the_legs() {
    let s = surface(
        &framed(vec![line("515", &[(100.0, 20.0), (100.0, 180.0)])]),
        &[],
    );
    let (a, b) = (p(20.0, 100.0), p(180.0, 100.0));
    let first = s.legs(a.clone(), b.clone()).unwrap().routes.remove(0);
    assert_eq!(s.fastest(a, b).unwrap(), first);
}

#[test]
fn speed_table_reads_the_sprint_symbols_the_way_the_standard_does() {
    let table = raster::Table::embedded();
    let kind = |code: &str| table.lookup(code);
    // The impassable wall blocks; the railway helper of the same family does not.
    assert_eq!(kind("515"), Some(raster::Kind::Block));
    assert_eq!(kind("515.1.1"), Some(raster::Kind::Ground(1.0)));
    assert_eq!(kind("515.1.2"), Some(raster::Kind::Ground(1.0)));
    assert_eq!(kind("509.1"), None, "the railway line itself is crossable");
    // Forest with good runnability is not as slow as walking.
    let Some(raster::Kind::Ground(forest)) = kind("405") else {
        panic!("forest is ground");
    };
    assert!((1.0..=1.1).contains(&forest), "{forest}");
    let Some(raster::Kind::Ground(walk)) = kind("408") else {
        panic!("walk is ground");
    };
    assert!(walk > 2.0 * forest);
    assert_eq!(kind("521"), Some(raster::Kind::Solid));
    assert!(matches!(kind("501.5"), Some(raster::Kind::Fast(f)) if f < 1.0));
}
