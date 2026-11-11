use freemapper_courseplan_model::{MapPoint, SpecialObject};

use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;
use crate::map::{FeatureGeometry, MapFeature};
use crate::score::travel::{LegRoutes, Route};

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

fn boulder(x: f64, y: f64) -> MapFeature {
    feature("204", FeatureGeometry::Point(p(x, y)))
}

fn route(len: f64) -> Route {
    Route {
        points: vec![],
        length_m: len,
        time_s: len,
    }
}

/// Straight single routes unless `many` says an end of the leg has several; `blocked` points are not
/// passable and legs to or from `lost` points fail.
struct Fake {
    many: Vec<MapPoint>,
    /// Discs (centre, radius in map mm) that are not passable.
    blocked: Vec<(MapPoint, f64)>,
    lost: Vec<MapPoint>,
    /// Legs between two candidates fail; legs from the start and to the finish still work.
    pairs_lost: bool,
    /// Legs from the start to anything but the finish fail.
    no_start_legs: bool,
    calls: AtomicUsize,
    /// Sites the start leg was asked for, in order.
    probed: std::sync::Mutex<Vec<MapPoint>>,
}

impl Fake {
    fn plain() -> Fake {
        Fake {
            many: vec![],
            blocked: vec![],
            lost: vec![],
            pairs_lost: false,
            no_start_legs: false,
            calls: AtomicUsize::new(0),
            probed: std::sync::Mutex::new(vec![]),
        }
    }
}

impl TravelTime for Fake {
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if from == p(0.0, 0.0) {
            self.probed.lock().unwrap().push(to.clone());
        }
        let end = |q: &MapPoint| *q == p(0.0, 0.0) || *q == p(100.0, 0.0);
        if self.lost.contains(&from)
            || self.lost.contains(&to)
            || (self.pairs_lost && !end(&from) && !end(&to))
            || (self.no_start_legs && from == p(0.0, 0.0) && to != p(100.0, 0.0))
        {
            return Err(ScoreError::Unreachable { from, to });
        }
        let len = (from.x - to.x).hypot(from.y - to.y).max(1.0);
        let n = if self.many.contains(&from) || self.many.contains(&to) {
            3
        } else {
            1
        };
        Ok(LegRoutes {
            routes: (0..n)
                .map(|i| route(len * (1.0 + 0.02 * i as f64)))
                .collect(),
        })
    }

    fn passable(&self, at: MapPoint) -> bool {
        !self
            .blocked
            .iter()
            .any(|(c, r)| (c.x - at.x).hypot(c.y - at.y) <= *r)
    }
}

fn ctx<'a>(specials: &'a [SpecialObject], existing: &'a [MapPoint]) -> SuggestContext<'a> {
    SuggestContext {
        start: p(0.0, 0.0),
        finish: p(100.0, 0.0),
        specials,
        min_distance_mm: 1.0,
        existing,
        limit: 50,
        line_margin_mm: 1.0,
    }
}

fn positions(found: &[Candidate]) -> Vec<(f64, f64)> {
    found.iter().map(|c| (c.at.x, c.at.y)).collect()
}

fn sorted(mut v: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v
}

#[test]
fn only_clear_features_are_candidates() {
    let building = feature(
        "521.3",
        FeatureGeometry::Area(vec![vec![
            p(40.0, 40.0),
            p(60.0, 40.0),
            p(60.0, 60.0),
            p(40.0, 60.0),
        ]]),
    );
    let forest = feature(
        "405",
        FeatureGeometry::Area(vec![vec![
            p(70.0, 70.0),
            p(90.0, 70.0),
            p(90.0, 90.0),
            p(70.0, 90.0),
        ]]),
    );
    let unknown = feature("999", FeatureGeometry::Point(p(80.0, 20.0)));
    let wall = feature(
        "515",
        FeatureGeometry::Line(vec![p(10.0, 70.0), p(10.0, 90.0), p(30.0, 90.0)]),
    );
    let index = MapIndex::from_features(vec![building, forest, unknown, wall, boulder(20.0, 20.0)]);
    let found = suggest(&index, &Fake::plain(), &ctx(&[], &[])).unwrap();
    assert_eq!(
        sorted(positions(&found)),
        sorted(vec![
            (40.0, 40.0),
            (60.0, 40.0),
            (60.0, 60.0),
            (40.0, 60.0),
            (10.0, 90.0),
            (20.0, 20.0)
        ])
    );
    let b = found.iter().find(|c| c.at == p(20.0, 20.0)).unwrap();
    assert_eq!(b.symbol_code.as_deref(), Some("204"));
}

#[test]
fn blocked_and_out_of_bounds_are_excluded() {
    let index = MapIndex::from_features(vec![
        boulder(20.0, 20.0),
        boulder(40.0, 20.0),
        boulder(60.0, 20.0),
        boulder(80.0, 20.0),
    ]);
    let specials = [SpecialObject::OutOfBoundsArea {
        id: "O1".into(),
        ppen_id: None,
        polygon: vec![p(35.0, 15.0), p(45.0, 15.0), p(45.0, 25.0), p(35.0, 25.0)],
        courses: vec![],
    }];
    let travel = Fake {
        blocked: vec![(p(20.0, 20.0), 0.0)],
        lost: vec![p(60.0, 20.0)],
        ..Fake::plain()
    };
    let found = suggest(&index, &travel, &ctx(&specials, &[])).unwrap();
    assert_eq!(positions(&found), vec![(80.0, 20.0)]);
}

#[test]
fn sites_on_a_forbidden_route_line_are_excluded() {
    let index = MapIndex::from_features(vec![boulder(40.0, 20.0), boulder(60.0, 20.0)]);
    // The line runs through the first site; the second is 20 mm from it, far outside the margin.
    let specials = [SpecialObject::ForbiddenRoute {
        id: "V1".into(),
        ppen_id: None,
        points: vec![p(40.0, 0.0), p(40.0, 40.0)],
        courses: vec![],
    }];
    let found = suggest(&index, &Fake::plain(), &ctx(&specials, &[])).unwrap();
    assert_eq!(positions(&found), vec![(60.0, 20.0)]);
}

#[test]
fn minimum_distance_to_start_finish_and_other_controls_holds() {
    let index = MapIndex::from_features(vec![
        boulder(5.0, 0.0),
        boulder(100.0, 6.0),
        boulder(52.0, 50.0),
        boulder(30.0, 30.0),
        boulder(33.0, 30.0),
        boulder(70.0, 70.0),
    ]);
    let existing = [p(50.0, 50.0)];
    let mut c = ctx(&[], &existing);
    c.min_distance_mm = 10.0;
    let found = suggest(&index, &Fake::plain(), &c).unwrap();
    assert_eq!(sorted(positions(&found)), vec![(30.0, 30.0), (70.0, 70.0)]);
}

#[test]
fn many_similar_routes_score_higher_than_one_obvious() {
    let index = MapIndex::from_features(vec![boulder(20.0, 20.0), boulder(80.0, 80.0)]);
    let travel = Fake {
        many: vec![p(80.0, 80.0)],
        ..Fake::plain()
    };
    let found = suggest(&index, &travel, &ctx(&[], &[])).unwrap();
    assert_eq!(found[0].at, p(80.0, 80.0));
    assert!(found[0].route_choice > found[1].route_choice);
    assert!(found.iter().all(|c| (0.0..=1.0).contains(&c.route_choice)));
    assert!(found[0].route_choice > 0.9, "{}", found[0].route_choice);
}

#[test]
fn a_clearly_shorter_route_weighs_the_choice_down() {
    struct Skewed(f64);
    impl TravelTime for Skewed {
        fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
            let len = (from.x - to.x).hypot(from.y - to.y).max(1.0);
            Ok(LegRoutes {
                routes: vec![route(len), route(len * self.0), route(len * self.0)],
            })
        }
        fn passable(&self, _: MapPoint) -> bool {
            true
        }
    }
    let index = MapIndex::from_features(vec![boulder(20.0, 20.0)]);
    let even = suggest(&index, &Skewed(1.02), &ctx(&[], &[])).unwrap();
    let skewed = suggest(&index, &Skewed(1.24), &ctx(&[], &[])).unwrap();
    assert!(even[0].route_choice > skewed[0].route_choice);
    assert!(skewed[0].route_choice > 0.0);
}

#[test]
fn crowded_surroundings_raise_difficulty() {
    let mut features = vec![boulder(150.0, 150.0)];
    for i in 0..6 {
        features.push(boulder(20.0 + 1.5 * f64::from(i), 20.0));
    }
    let index = MapIndex::from_features(features);
    let found = suggest(&index, &Fake::plain(), &ctx(&[], &[])).unwrap();
    let at = |x, y| found.iter().find(|c| c.at == p(x, y)).unwrap();
    assert!(at(20.0, 20.0).difficulty > at(150.0, 150.0).difficulty);
    assert!(found.iter().all(|c| (0.0..=1.0).contains(&c.difficulty)));
}

#[test]
fn effort_is_the_fastest_time_from_the_start() {
    let index = MapIndex::from_features(vec![boulder(30.0, 40.0)]);
    let found = suggest(&index, &Fake::plain(), &ctx(&[], &[])).unwrap();
    assert!((found[0].effort_s - 50.0).abs() < 1e-9);
}

#[test]
fn candidates_are_sorted_by_route_choice_then_position() {
    let index = MapIndex::from_features(vec![
        boulder(90.0, 10.0),
        boulder(10.0, 90.0),
        boulder(10.0, 20.0),
        boulder(50.0, 50.0),
    ]);
    let found = suggest(&index, &Fake::plain(), &ctx(&[], &[])).unwrap();
    assert_eq!(
        positions(&found),
        vec![(10.0, 20.0), (10.0, 90.0), (50.0, 50.0), (90.0, 10.0)]
    );
    assert_eq!(
        found,
        suggest(&index, &Fake::plain(), &ctx(&[], &[])).unwrap()
    );
}

#[test]
fn limit_cuts_the_list() {
    let index = MapIndex::from_features(
        (0..10)
            .map(|i| boulder(10.0 + 8.0 * f64::from(i), 40.0))
            .collect(),
    );
    let mut c = ctx(&[], &[]);
    c.limit = 3;
    let found = suggest(&index, &Fake::plain(), &c).unwrap();
    assert_eq!(found.len(), 3);
    c.limit = 0;
    assert!(suggest(&index, &Fake::plain(), &c).unwrap().is_empty());
}

#[test]
fn many_sites_stay_bounded_and_deterministic() {
    // 400 boulders: only a bounded number of legs are computed, and two runs agree.
    let index = MapIndex::from_features(
        (0..400)
            .map(|i| boulder(5.0 + 3.0 * f64::from(i % 20), 5.0 + 3.0 * f64::from(i / 20)))
            .collect(),
    );
    let mut c = ctx(&[], &[]);
    c.limit = 5;
    let fake = Fake::plain();
    let a = suggest(&index, &fake, &c).unwrap();
    assert_eq!(a.len(), 5);
    // 2 * limit = 10 sites; each costs three legs (fastest from the start, fastest to the finish, start leg
    // with alternatives), then at most 4 neighbour legs each, plus the start-to-finish probe.
    let calls = fake.calls.load(Ordering::Relaxed);
    assert!(calls <= 1 + 10 * 3 + 10 * 4, "{calls} legs");
    assert_eq!(a, suggest(&index, &Fake::plain(), &c).unwrap());
}

#[test]
fn the_selection_spreads_over_a_wide_map() {
    // 10 rows of 40 boulders on a 400 mm wide, flat map.
    let index = MapIndex::from_features(
        (0..400)
            .map(|i| {
                boulder(
                    5.0 + 10.0 * f64::from(i % 40),
                    5.0 + 10.0 * f64::from(i / 40),
                )
            })
            .collect(),
    );
    let mut c = ctx(&[], &[]);
    c.limit = 8;
    let fake = Fake::plain();
    suggest(&index, &fake, &c).unwrap();
    // The 16 sites picked for routing, before the cut by route choice.
    let mut picked = fake.probed.lock().unwrap().clone();
    picked.retain(|q| *q != p(100.0, 0.0)); // the probe from start to finish
    picked.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    picked.dedup();
    let west = picked.iter().filter(|q| q.x < 200.0).count();
    assert_eq!(picked.len(), 16);
    assert!((5..=11).contains(&west), "{west} of 16 in the west half");
    assert!(picked.iter().any(|q| q.y < 30.0) && picked.iter().any(|q| q.y > 70.0));
}

#[test]
fn unreachable_candidates_are_replaced_by_further_sites() {
    // Farthest-first tries the far end (x 130 to 230) first; those sites are lost, so the picks fail
    // before the near ones replace them.
    let at = |i: i32| p(10.0 + 20.0 * f64::from(i), 40.0);
    let index = MapIndex::from_features((0..12).map(|i| boulder(at(i).x, 40.0)).collect());
    let travel = Fake {
        lost: (6..12).map(at).collect(),
        ..Fake::plain()
    };
    let mut c = ctx(&[], &[]);
    c.limit = 3;
    let found = suggest(&index, &travel, &c).unwrap();
    assert_eq!(found.len(), 3);
    assert!(found.iter().all(|c| c.at.x < 130.0));
    // The first start leg is the probe to the finish.
    let first_try = travel.probed.lock().unwrap()[1].clone();
    assert_eq!(first_try, at(11), "the far end is tried first and fails");
}

#[test]
fn the_loop_stops_after_four_tries_per_limit_when_everything_is_lost() {
    let at = |i: i32| p(10.0 + 20.0 * f64::from(i), 40.0);
    let index = MapIndex::from_features((0..30).map(|i| boulder(at(i).x, 40.0)).collect());
    let travel = Fake {
        lost: (0..30).map(at).collect(),
        ..Fake::plain()
    };
    let mut c = ctx(&[], &[]);
    c.limit = 2;
    assert!(suggest(&index, &travel, &c).unwrap().is_empty());
    // 8 tries, one leg each, plus the start-to-finish probe of the coarse surface.
    assert_eq!(travel.calls.load(Ordering::Relaxed), 9);
}

#[test]
fn a_site_is_not_picked_twice_with_minimum_distance_zero() {
    let index = MapIndex::from_features(vec![boulder(30.0, 30.0), boulder(60.0, 60.0)]);
    let mut c = ctx(&[], &[]);
    c.min_distance_mm = 0.0;
    c.limit = 5;
    assert_eq!(suggest(&index, &Fake::plain(), &c).unwrap().len(), 2);
}

#[test]
fn a_coarse_start_leg_that_fails_drops_the_candidate_without_nan() {
    // One candidate only (no neighbours); the coarse surface loses the leg from the start.
    let index = MapIndex::from_features(vec![boulder(30.0, 30.0)]);
    let coarse = Fake {
        no_start_legs: true,
        ..Fake::plain()
    };
    let found = suggest_with(&index, &Fake::plain(), &coarse, &ctx(&[], &[])).unwrap();
    assert!(found.is_empty());
    // A coarse surface that cannot join start and finish at all gives way to the fine one.
    let cut = Fake {
        lost: vec![p(0.0, 0.0)],
        ..Fake::plain()
    };
    let found = suggest_with(&index, &Fake::plain(), &cut, &ctx(&[], &[])).unwrap();
    assert_eq!(found.len(), 1);
    // With the leg intact, a lone candidate has finite numbers.
    let found = suggest(&index, &Fake::plain(), &ctx(&[], &[])).unwrap();
    assert_eq!(found.len(), 1);
    let c = &found[0];
    assert!(
        [c.route_choice, c.difficulty, c.effort_s]
            .iter()
            .all(|v| v.is_finite())
    );
}

#[test]
fn a_coarse_surface_that_blocks_an_end_gives_way_to_the_fine_one() {
    let index = MapIndex::from_features(vec![boulder(30.0, 30.0), boulder(60.0, 60.0)]);
    let coarse = Fake {
        blocked: vec![(p(0.0, 0.0), 0.0)],
        lost: vec![p(0.0, 0.0)],
        ..Fake::plain()
    };
    let found = suggest_with(&index, &Fake::plain(), &coarse, &ctx(&[], &[])).unwrap();
    assert_eq!(found.len(), 2);
    assert_eq!(
        coarse.calls.load(Ordering::Relaxed),
        1,
        "only the end probe"
    );
}

#[test]
fn a_corner_on_a_blocked_cell_stays_when_a_free_place_is_near() {
    let building = feature(
        "521",
        FeatureGeometry::Area(vec![vec![
            p(40.0, 40.0),
            p(60.0, 40.0),
            p(60.0, 60.0),
            p(40.0, 60.0),
        ]]),
    );
    let index = MapIndex::from_features(vec![building]);
    let near = Fake {
        blocked: vec![(p(40.0, 40.0), 0.7)],
        ..Fake::plain()
    };
    let found = suggest(&index, &near, &ctx(&[], &[])).unwrap();
    assert!(found.iter().any(|c| c.at == p(40.0, 40.0)));
    let walled = Fake {
        blocked: vec![(p(40.0, 40.0), 5.0)],
        ..Fake::plain()
    };
    let found = suggest(&index, &walled, &ctx(&[], &[])).unwrap();
    assert!(!found.iter().any(|c| c.at == p(40.0, 40.0)));
    assert_eq!(found.len(), 3);
}

#[test]
fn a_corner_the_coarse_surface_cannot_route_from_is_no_candidate() {
    // The fine surface finds a free place beside the corner, but the tours route from the corner itself on
    // the coarse surface, where it is lost.
    let building = feature(
        "521",
        FeatureGeometry::Area(vec![vec![
            p(40.0, 40.0),
            p(60.0, 40.0),
            p(60.0, 60.0),
            p(40.0, 60.0),
        ]]),
    );
    let index = MapIndex::from_features(vec![building]);
    let fine = Fake {
        blocked: vec![(p(40.0, 40.0), 0.7)],
        ..Fake::plain()
    };
    let coarse = Fake {
        lost: vec![p(40.0, 40.0)],
        ..Fake::plain()
    };
    let found = suggest_with(&index, &fine, &coarse, &ctx(&[], &[])).unwrap();
    assert!(!found.iter().any(|c| c.at == p(40.0, 40.0)));
    assert_eq!(found.len(), 3);
}

#[test]
fn unreachable_start_and_finish_are_errors() {
    let index = MapIndex::from_features(vec![boulder(30.0, 30.0)]);
    let no_start = Fake {
        blocked: vec![(p(0.0, 0.0), 0.0)],
        lost: vec![p(0.0, 0.0)],
        ..Fake::plain()
    };
    assert_eq!(
        suggest(&index, &no_start, &ctx(&[], &[])),
        Err(ScoreError::StartUnreachable { at: p(0.0, 0.0) })
    );
    let no_finish = Fake {
        blocked: vec![(p(100.0, 0.0), 0.0)],
        lost: vec![p(100.0, 0.0)],
        ..Fake::plain()
    };
    assert_eq!(
        suggest(&index, &no_finish, &ctx(&[], &[])),
        Err(ScoreError::FinishUnreachable { at: p(100.0, 0.0) })
    );
}

fn surface(index: &MapIndex) -> crate::score::travel::CostSurface {
    use crate::score::travel::{CostSurface, Pace, RouteParams};
    CostSurface::build(index, 1000, &[], Pace(360), &RouteParams::default()).unwrap()
}

fn open_field() -> MapFeature {
    feature(
        "401",
        FeatureGeometry::Area(vec![vec![
            p(0.0, 0.0),
            p(200.0, 0.0),
            p(200.0, 200.0),
            p(0.0, 200.0),
        ]]),
    )
}

#[test]
fn a_start_on_a_thin_wall_snaps_and_one_inside_a_building_fails() {
    let wall = feature(
        "515",
        FeatureGeometry::Line(vec![p(100.0, 0.0), p(100.0, 200.0)]),
    );
    let building = feature(
        "521",
        FeatureGeometry::Area(vec![vec![
            p(140.0, 140.0),
            p(190.0, 140.0),
            p(190.0, 190.0),
            p(140.0, 190.0),
        ]]),
    );
    let index = MapIndex::from_features(vec![open_field(), wall, building, boulder(50.0, 100.0)]);
    let s = surface(&index);
    let mut c = ctx(&[], &[]);
    c.start = p(100.0, 100.0);
    c.finish = p(30.0, 30.0);
    c.min_distance_mm = 5.0;
    assert!(!s.passable(c.start.clone()), "the wall cell is blocked");
    assert!(suggest(&index, &s, &c).is_ok());
    c.start = p(165.0, 165.0);
    assert_eq!(
        suggest(&index, &s, &c),
        Err(ScoreError::StartUnreachable {
            at: p(165.0, 165.0)
        })
    );
}

fn routes(lengths: &[f64]) -> LegRoutes {
    LegRoutes {
        routes: lengths.iter().map(|&l| route(l)).collect(),
    }
}

#[test]
fn route_choice_of_one_leg() {
    assert_eq!(leg_choice(&routes(&[100.0]), 1.25), 0.0);
    // Two equal routes: half the richness, no penalty.
    assert!((leg_choice(&routes(&[100.0, 100.0]), 1.25) - 0.5).abs() < 1e-12);
    // The second route is beyond the ratio: it counts for nothing.
    assert_eq!(leg_choice(&routes(&[100.0, 130.0]), 1.25), 0.0);
    // Three equal routes: full; a second route at the limit halves it.
    assert!((leg_choice(&routes(&[100.0, 100.0, 100.0]), 1.25) - 1.0).abs() < 1e-12);
    assert!((leg_choice(&routes(&[100.0, 125.0, 125.0]), 1.25) - 0.5).abs() < 1e-12);
}

#[test]
fn candidates_with_unreachable_pair_legs_keep_the_start_leg() {
    let index = MapIndex::from_features(vec![boulder(30.0, 30.0), boulder(60.0, 60.0)]);
    let travel = Fake {
        pairs_lost: true,
        many: vec![p(30.0, 30.0)],
        ..Fake::plain()
    };
    let found = suggest(&index, &travel, &ctx(&[], &[])).unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|c| c.route_choice.is_finite()));
    assert!(found[0].route_choice > 0.9 && found[1].route_choice == 0.0);
}

/// Run time on the Lübeck example map: `cargo test --release -- --ignored luebeck_timing --nocapture`.
#[test]
#[ignore = "timing run, prints the duration"]
fn luebeck_timing_for_limit_60() {
    const COARSE_CELL_M: f64 = 8.0;
    use crate::score::travel::{CostSurface, Pace, RouteParams};
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let project = freemapper_courseplan_model::ProjectFile::load(std::path::Path::new(&format!(
        "{root}/examples/luebeck-sprint/luebeck-sprint.courseplan.toml"
    )))
    .unwrap();
    let doc = crate::omap::read::read_map_file(std::path::Path::new(&format!(
        "{root}/tests/fixtures/maps/luebeck.omap"
    )))
    .unwrap();
    let index = MapIndex::from_document(&doc);
    let specials = &project.event.specials;
    let at = |want_start: bool| {
        specials
            .iter()
            .find_map(|s| match s {
                SpecialObject::Start { map_mm, .. } if want_start => Some(map_mm.clone()),
                SpecialObject::Finish { map_mm, .. } if !want_start => Some(map_mm.clone()),
                _ => None,
            })
            .unwrap()
    };
    let t0 = std::time::Instant::now();
    let scale = doc.georef.scale_denominator;
    let surface =
        CostSurface::build(&index, scale, &[], Pace(360), &RouteParams::default()).unwrap();
    let coarse_params = RouteParams {
        cell_m: COARSE_CELL_M,
        ..RouteParams::default()
    };
    let coarse = CostSurface::build(&index, scale, &[], Pace(360), &coarse_params).unwrap();
    let built = t0.elapsed();
    let c = SuggestContext {
        // The example's start flag stands where the surface is blocked; use a passable control instead.
        start: project
            .event
            .controls
            .iter()
            .map(|c| c.map_mm.clone())
            .find(|q| surface.passable(q.clone()) && coarse.fastest(q.clone(), at(false)).is_ok())
            .unwrap_or_else(|| at(true)),
        finish: at(false),
        specials,
        min_distance_mm: 3.0,
        line_margin_mm: 1.0,
        existing: &[],
        limit: 60,
    };
    // The example's start flag lies inside a building (ADR 0062).
    let flagged = SuggestContext {
        start: at(true),
        ..c.clone()
    };
    assert!(matches!(
        suggest_with(&index, &surface, &coarse, &flagged),
        Err(ScoreError::StartUnreachable { .. })
    ));
    let t1 = std::time::Instant::now();
    let found = suggest_with(&index, &surface, &coarse, &c).unwrap();
    println!(
        "surface {built:?}, suggest {:?}, {} candidates",
        t1.elapsed(),
        found.len()
    );
    assert!(!found.is_empty());
}
