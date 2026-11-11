use freemapper_courseplan_model::{MapPoint, SpecialObject};

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
    blocked: Vec<MapPoint>,
    lost: Vec<MapPoint>,
}

impl Fake {
    fn plain() -> Fake {
        Fake {
            many: vec![],
            blocked: vec![],
            lost: vec![],
        }
    }
}

impl TravelTime for Fake {
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
        if self.lost.contains(&from) || self.lost.contains(&to) {
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
        !self.blocked.contains(&at)
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
        blocked: vec![p(20.0, 20.0)],
        lost: vec![p(60.0, 20.0)],
        ..Fake::plain()
    };
    let found = suggest(&index, &travel, &ctx(&specials, &[])).unwrap();
    assert_eq!(positions(&found), vec![(80.0, 20.0)]);
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
    // 400 boulders: only `limit * 4` are looked at, and two runs agree.
    let index = MapIndex::from_features(
        (0..400)
            .map(|i| boulder(5.0 + 3.0 * f64::from(i % 20), 5.0 + 3.0 * f64::from(i / 20)))
            .collect(),
    );
    let mut c = ctx(&[], &[]);
    c.limit = 5;
    let a = suggest(&index, &Fake::plain(), &c).unwrap();
    assert_eq!(a.len(), 5);
    assert_eq!(a, suggest(&index, &Fake::plain(), &c).unwrap());
}

/// Run time on the Lübeck example map: `cargo test --release -- --ignored luebeck_timing --nocapture`.
#[test]
#[ignore = "timing run, prints the duration"]
fn luebeck_timing_for_limit_60() {
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
    let surface = CostSurface::build(
        &index,
        doc.georef.scale_denominator,
        &[],
        Pace(360),
        &RouteParams::default(),
    )
    .unwrap();
    let built = t0.elapsed();
    let c = SuggestContext {
        // The example's start flag stands where the surface is blocked; use a passable control instead.
        start: project
            .event
            .controls
            .iter()
            .map(|c| c.map_mm.clone())
            .find(|q| surface.passable(q.clone()))
            .unwrap_or_else(|| at(true)),
        finish: at(false),
        specials,
        min_distance_mm: 3.0,
        existing: &[],
        limit: 60,
    };
    let t1 = std::time::Instant::now();
    let found = suggest(&index, &surface, &c).unwrap();
    println!(
        "surface {built:?}, suggest {:?}, {} candidates",
        t1.elapsed(),
        found.len()
    );
    assert!(!found.is_empty());
}
