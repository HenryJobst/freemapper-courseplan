//! Tests of the dead-end depth (ADR 0085). Maps are at 1:1000, so a map millimetre is a metre on the ground,
//! and the raster cells are 2 m.

use freemapper_courseplan_model::MapPoint;

use super::*;
use crate::map::{FeatureGeometry, MapFeature, MapIndex};
use crate::score::travel::{Pace, RouteParams};

fn p(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn building(x0: f64, y0: f64, x1: f64, y1: f64) -> MapFeature {
    MapFeature {
        symbol_code: Some("521".to_owned()),
        part: "p".into(),
        geometry: FeatureGeometry::Area(vec![vec![p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1)]]),
    }
}

fn surface(buildings: Vec<MapFeature>) -> CostSurface {
    CostSurface::build(
        &MapIndex::from_features(buildings),
        1000,
        &[],
        Pace(250),
        &RouteParams::default(),
    )
    .unwrap()
}

/// A field of 500 by 500 m, framed by buildings, with a lane of 12 m width and 150 m length leaving it at the
/// top: the lane runs from the mouth at y = 100 north to the tip at y = -50 and is closed there.
fn field_with_lane(extra: Vec<MapFeature>) -> CostSurface {
    let mut all = vec![
        // The frame of the field.
        building(-250.0, 600.0, 750.0, 700.0),
        building(-300.0, -300.0, -250.0, 700.0),
        building(750.0, -300.0, 800.0, 700.0),
        // The two sides of the lane, and its closed end.
        building(-250.0, -300.0, 244.0, 100.0),
        building(256.0, -300.0, 750.0, 100.0),
        building(244.0, -300.0, 256.0, -50.0),
    ];
    all.extend(extra);
    surface(all)
}

fn depth(s: &CostSurface, at: MapPoint) -> DeadEnd {
    s.dead_end(&at, 60.0).expect("the place can be judged")
}

#[test]
fn a_place_deep_in_a_lane_is_deep_and_one_in_the_field_is_not() {
    let s = field_with_lane(vec![]);
    // 130 m in from the mouth, 20 m from the tip: no choice before the mouth, and the mouth is far.
    let deep = depth(&s, p(250.0, -30.0));
    assert!(deep.depth_m > 100.0, "{deep:?}");
    // In the field the way splits at once.
    let open = depth(&s, p(250.0, 400.0));
    assert!(open.depth_m <= 12.0, "{open:?}");
}

#[test]
fn the_depth_is_the_walk_to_where_the_choice_starts() {
    let s = field_with_lane(vec![]);
    // 40 m up the lane from the mouth. The choice starts a little beyond the mouth, where the field opens.
    let d = depth(&s, p(250.0, 60.0)).depth_m;
    assert!((40.0..=75.0).contains(&d), "{d}");
    // Twice as far in is about twice as deep, less the opening of the field.
    let d2 = depth(&s, p(250.0, 20.0)).depth_m;
    assert!(d2 > d + 30.0, "{d2} against {d}");
}

#[test]
fn the_pocket_behind_a_place_is_measured() {
    let s = field_with_lane(vec![]);
    // 90 m from the tip: about 90 m of pocket behind the place.
    let tip = depth(&s, p(250.0, 40.0)).tip_m;
    assert!((80.0..=100.0).contains(&tip), "{tip}");
}

#[test]
fn a_through_street_is_no_dead_end() {
    // The same lane open at the far end into a second field.
    let s = surface(vec![
        building(-250.0, 600.0, 750.0, 700.0),
        building(-300.0, -700.0, -250.0, 700.0),
        building(750.0, -700.0, 800.0, 700.0),
        building(-250.0, -700.0, 750.0, -600.0),
        building(-250.0, -300.0, 244.0, 100.0),
        building(256.0, -300.0, 750.0, 100.0),
    ]);
    let middle = depth(&s, p(250.0, -100.0));
    assert!(middle.depth_m <= 12.0, "{middle:?}");
}

#[test]
fn a_small_obstacle_in_a_corridor_does_not_split_it() {
    // A kiosk of 6 by 6 m in a lane of 14 m: the runner passes it either side without a choice.
    let wide = |extra: Vec<MapFeature>| {
        let mut all = vec![
            building(-250.0, 600.0, 750.0, 700.0),
            building(-300.0, -300.0, -250.0, 700.0),
            building(750.0, -300.0, 800.0, 700.0),
            building(-250.0, -300.0, 243.0, 100.0),
            building(257.0, -300.0, 750.0, 100.0),
            building(243.0, -300.0, 257.0, -50.0),
        ];
        all.extend(extra);
        surface(all)
    };
    let plain = depth(&wide(vec![]), p(250.0, -30.0)).depth_m;
    let kiosk = depth(
        &wide(vec![building(247.0, 20.0, 253.0, 26.0)]),
        p(250.0, -30.0),
    )
    .depth_m;
    assert!(plain > 100.0 && kiosk > 100.0, "{plain} and {kiosk}");
}

#[test]
fn a_courtyard_is_no_corridor() {
    // A courtyard of 40 by 40 m with a gate of 6 m, off an open field. Its floor is open ground, where a runner
    // chooses among many ways, so a place in its middle is not deep in a dead end.
    let s = surface(vec![
        building(-250.0, 600.0, 750.0, 700.0),
        building(-300.0, -300.0, -250.0, 700.0),
        building(750.0, -300.0, 800.0, 700.0),
        building(-250.0, -300.0, 750.0, -200.0),
        building(210.0, 270.0, 270.0, 280.0),
        building(210.0, 280.0, 220.0, 320.0),
        building(260.0, 280.0, 270.0, 320.0),
        building(210.0, 320.0, 237.0, 330.0),
        building(243.0, 320.0, 270.0, 330.0),
    ]);
    let middle = depth(&s, p(240.0, 300.0));
    assert!(middle.depth_m <= 60.0, "{middle:?}");
    assert!(depth(&s, p(240.0, 450.0)).depth_m <= 12.0);
}

#[test]
fn a_region_smaller_than_the_horizon_cannot_be_judged() {
    // A closed yard of 60 by 60 m: nothing reaches the rim of the horizon.
    let s = surface(vec![
        building(-50.0, -50.0, 110.0, -10.0),
        building(-50.0, 70.0, 110.0, 110.0),
        building(-50.0, -10.0, -10.0, 70.0),
        building(70.0, -10.0, 110.0, 70.0),
    ]);
    assert_eq!(s.dead_end(&p(30.0, 30.0), 60.0), None);
}

#[test]
fn a_place_on_a_wall_stands_beside_it() {
    let s = field_with_lane(vec![]);
    // On the line of the lane's wall, 1 m inside the building: the runner stands in the lane.
    let on_wall = s.dead_end(&p(244.5, -30.0), 60.0).expect("judged");
    assert!(on_wall.depth_m > 100.0, "{on_wall:?}");
}

#[test]
fn the_same_place_gives_the_same_answer() {
    let s = field_with_lane(vec![]);
    let a = s.dead_end(&p(250.0, 20.0), 60.0);
    assert_eq!(a, s.dead_end(&p(250.0, 20.0), 60.0));
}

#[test]
fn a_place_on_a_thin_wall_between_a_dead_end_and_open_ground_takes_the_deeper_side() {
    // The west wall of the lane is only 4 m thick and open field lies beyond it. A control on that wall may be
    // approached from the field, but it also stands in the dead end, and the deeper side counts (ADR 0085).
    let s = surface(vec![
        building(-250.0, 600.0, 750.0, 700.0),
        building(-300.0, -300.0, -250.0, 700.0),
        building(750.0, -300.0, 800.0, 700.0),
        building(240.0, -300.0, 244.0, 100.0),
        building(256.0, -300.0, 750.0, 100.0),
        building(240.0, -300.0, 256.0, -50.0),
        building(-250.0, -300.0, -240.0, 100.0),
    ]);
    let on_wall = s.dead_end(&p(242.0, -30.0), 60.0).expect("judged");
    assert!(on_wall.depth_m > 100.0, "{on_wall:?}");
    let in_field = depth(&s, p(100.0, 0.0));
    assert!(in_field.depth_m <= 12.0, "{in_field:?}");
}

#[test]
fn a_free_place_beside_a_thin_wall_also_counts_the_dead_end_behind_it() {
    // A wall of 2 m between open field (west) and the lane (east). A control 1 m west of the wall stands in the
    // open, yet a few metres from the dead end, and the deeper side counts (ADR 0085).
    let s = surface(vec![
        building(-250.0, 600.0, 750.0, 700.0),
        building(-300.0, -300.0, -250.0, 700.0),
        building(750.0, -300.0, 800.0, 700.0),
        building(241.0, -300.0, 243.0, 100.0),
        building(256.0, -300.0, 750.0, 100.0),
        building(241.0, -300.0, 256.0, -50.0),
        building(-250.0, -300.0, -240.0, 100.0),
    ]);
    let beside = s.dead_end(&p(239.5, -30.0), 60.0).expect("judged");
    assert!(beside.depth_m > 100.0, "{beside:?}");
    let far = depth(&s, p(100.0, 0.0));
    assert!(far.depth_m <= 12.0, "{far:?}");
}
