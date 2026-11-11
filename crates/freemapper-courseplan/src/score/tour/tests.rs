use std::time::Instant;

use freemapper_courseplan_model::{MapPoint, ScoreSettings};

use super::*;
use crate::score::travel::{LegRoutes, Route};

/// Straight lines over plain coordinates; one coordinate unit takes `s_per_unit` seconds.
struct Fake {
    s_per_unit: f64,
}

impl TravelTime for Fake {
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
        let len = (from.x - to.x).hypot(from.y - to.y);
        Ok(LegRoutes {
            routes: vec![Route {
                points: vec![from, to],
                length_m: len,
                time_s: len * self.s_per_unit,
            }],
        })
    }
    fn passable(&self, _p: MapPoint) -> bool {
        true
    }
}

const FAKE: Fake = Fake { s_per_unit: 1.0 };

fn pt(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn sc(x: f64, y: f64, value: u8) -> ScoredControl {
    ScoredControl {
        at: pt(x, y),
        value,
    }
}

fn tour(
    controls: &[ScoredControl],
    links: &[(usize, usize)],
    start: (f64, f64),
    finish: (f64, f64),
    limit: f64,
) -> Result<Tour, ScoreError> {
    best_tour(
        controls,
        links,
        &FAKE,
        pt(start.0, start.1),
        pt(finish.0, finish.1),
        limit,
    )
}

fn adjacent(order: &[usize], a: usize, b: usize) -> bool {
    let (i, j) = (
        order.iter().position(|&c| c == a).unwrap(),
        order.iter().position(|&c| c == b).unwrap(),
    );
    i.abs_diff(j) == 1
}

/// A repeatable scatter of `n` controls with values 1 to 5 on a 400 by 400 field.
fn scatter(n: usize, mut state: u32) -> Vec<ScoredControl> {
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    };
    (0..n)
        .map(|_| {
            let x = f64::from(next() % 400);
            let y = f64::from(next() % 400);
            sc(x, y, (next() % 5 + 1) as u8)
        })
        .collect()
}

#[test]
fn best_tour_takes_the_most_points_within_the_limit() {
    let cs = [sc(10.0, 0.0, 1), sc(0.0, 10.0, 5), sc(-10.0, 0.0, 2)];
    let t = tour(&cs, &[], (0.0, 0.0), (0.0, 0.0), 22.0).unwrap();
    assert_eq!((t.order.clone(), t.points), (vec![1], 5));
    // B and C score 7 in 34.1 s; A and B would score 6 in the same time.
    let t = tour(&cs, &[], (0.0, 0.0), (0.0, 0.0), 40.0).unwrap();
    assert_eq!((t.order, t.points), (vec![1, 2], 7));
    assert!((t.time_s - (10.0 + 200f64.sqrt() + 10.0)).abs() < 1e-9);
}

#[test]
fn equal_alternatives_resolve_to_the_lower_index_order() {
    // Two controls that cost the same and score the same; only one fits.
    let cs = [sc(10.0, 0.0, 3), sc(-10.0, 0.0, 3)];
    let t = tour(&cs, &[], (0.0, 0.0), (0.0, 0.0), 25.0).unwrap();
    assert_eq!(t.order, vec![0]);
    // Both fit: the same time either way round, the lower order wins.
    let t = tour(&cs, &[], (0.0, 0.0), (0.0, 0.0), 100.0).unwrap();
    assert_eq!(t.order, vec![0, 1]);
}

#[test]
fn links_stay_consecutive_in_either_direction() {
    let cs = [sc(10.0, 0.0, 1), sc(20.0, 0.0, 1), sc(30.0, 0.0, 1)];
    for link in [(0, 2), (2, 0)] {
        let t = tour(&cs, &[link], (0.0, 0.0), (0.0, 0.0), 500.0).unwrap();
        assert_eq!(t.points, 3);
        assert!(adjacent(&t.order, 0, 2), "{:?}", t.order);
    }
}

#[test]
fn links_that_chain_stay_in_a_row() {
    let cs = [
        sc(10.0, 0.0, 1),
        sc(-20.0, 5.0, 1),
        sc(30.0, 0.0, 1),
        sc(0.0, 30.0, 1),
    ];
    let t = tour(&cs, &[(0, 1), (1, 2)], (0.0, 0.0), (0.0, 0.0), 1000.0).unwrap();
    assert_eq!(t.points, 4);
    assert!(
        adjacent(&t.order, 0, 1) && adjacent(&t.order, 1, 2),
        "{:?}",
        t.order
    );
    let p = |c| t.order.iter().position(|&x| x == c).unwrap();
    assert_eq!(p(1).abs_diff(p(0)), 1);
    assert_eq!(p(1).abs_diff(p(2)), 1);
}

#[test]
fn a_link_forces_both_controls_in_even_when_one_is_not_worth_it() {
    // Control 1 is far and worth 1; the link pulls it in with control 0 when the limit allows.
    let cs = [sc(10.0, 0.0, 5), sc(60.0, 0.0, 1), sc(0.0, 10.0, 5)];
    let t = tour(&cs, &[(0, 1)], (0.0, 0.0), (0.0, 0.0), 130.0).unwrap();
    assert!(t.order.contains(&0) && t.order.contains(&1));
    assert!(adjacent(&t.order, 0, 1));
}

#[test]
fn a_link_that_does_not_fit_the_limit_is_reported() {
    let cs = [sc(10.0, 0.0, 1), sc(1000.0, 0.0, 1), sc(0.0, 10.0, 1)];
    let err = tour(&cs, &[(0, 1)], (0.0, 0.0), (0.0, 0.0), 100.0).unwrap_err();
    assert_eq!(err, ScoreError::LinkDoesNotFit { a: 0, b: 1 });
    // The same on a chain and above the exact limit: the far link is the one named.
    let mut many = scatter(20, 5);
    many.push(sc(5000.0, 0.0, 1));
    let last = many.len() - 1;
    let err = tour(&many, &[(2, last)], (0.0, 0.0), (0.0, 0.0), 300.0).unwrap_err();
    assert_eq!(err, ScoreError::LinkDoesNotFit { a: 2, b: last });
}

#[test]
fn links_that_branch_or_close_a_ring_are_refused() {
    let cs = scatter(5, 3);
    let err = tour(&cs, &[(0, 1), (0, 2), (0, 3)], (0.0, 0.0), (0.0, 0.0), 1e6).unwrap_err();
    assert_eq!(err, ScoreError::LinksNotAChain { control: 0 });
    let err = tour(&cs, &[(0, 1), (1, 2), (2, 0)], (0.0, 0.0), (0.0, 0.0), 1e6).unwrap_err();
    assert!(matches!(err, ScoreError::LinksNotAChain { .. }));
    let err = tour(&cs, &[(0, 9)], (0.0, 0.0), (0.0, 0.0), 1e6).unwrap_err();
    assert_eq!(err, ScoreError::BadLink { a: 0, b: 9 });
}

#[test]
fn no_controls_one_control_and_all_controls() {
    let t = tour(&[], &[], (0.0, 0.0), (30.0, 40.0), 100.0).unwrap();
    assert_eq!((t.order.len(), t.points), (0, 0));
    assert!((t.time_s - 50.0).abs() < 1e-9);

    let one = [sc(0.0, 50.0, 4)];
    let t = tour(&one, &[], (0.0, 0.0), (0.0, 0.0), 100.0).unwrap();
    assert_eq!((t.order, t.points), (vec![0], 4));
    let t = tour(&one, &[], (0.0, 0.0), (0.0, 0.0), 99.0).unwrap();
    assert_eq!((t.order.len(), t.points), (0, 0));

    let cs = scatter(8, 11);
    let t = tour(&cs, &[], (200.0, 200.0), (200.0, 200.0), 1e9).unwrap();
    assert_eq!(t.order.len(), 8);
    assert_eq!(t.points, cs.iter().map(|c| u32::from(c.value)).sum::<u32>());
}

#[test]
fn a_limit_below_the_start_finish_distance_gives_an_empty_tour() {
    let cs = [sc(50.0, 0.0, 5)];
    let t = tour(&cs, &[], (0.0, 0.0), (100.0, 0.0), 10.0).unwrap();
    assert_eq!((t.order.len(), t.points), (0, 0));
    assert!((t.time_s - 100.0).abs() < 1e-9);
    // Above the exact limit as well.
    let many = scatter(20, 2);
    let t = tour(&many, &[], (0.0, 0.0), (400.0, 0.0), 10.0).unwrap();
    assert_eq!(t.points, 0);
}

fn report(cs: &[ScoredControl], limit: f64, pace_good: f64, pace_very_good: f64) -> LengthReport {
    analyse_length(
        cs,
        &[],
        &Fake {
            s_per_unit: pace_good,
        },
        &Fake {
            s_per_unit: pace_very_good,
        },
        pt(0.0, 0.0),
        pt(0.0, 0.0),
        limit,
        &ScoreSettings::default(),
    )
    .unwrap()
}

#[test]
fn report_without_issues_inside_the_bands() {
    let cs = [sc(10.0, 0.0, 9), sc(-10.0, 0.0, 1)];
    // Good: one control of two, 90 percent. Very good: both, 38 of 39 seconds.
    let r = report(&cs, 39.0, 1.0, 0.95);
    assert_eq!(
        (r.good.points, r.very_good.points, r.all_points),
        (9, 10, 10)
    );
    assert!((r.good_percent - 90.0).abs() < 1e-9);
    assert!((r.very_good_finish_percent - 38.0 / 39.0 * 100.0).abs() < 1e-9);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
}

#[test]
fn report_flags_good_runner_outside_87_to_93() {
    let cs = [sc(10.0, 0.0, 5), sc(-100.0, 0.0, 5)];
    let r = report(&cs, 30.0, 1.0, 1.0);
    assert!((r.good_percent - 50.0).abs() < 1e-9);
    assert!(r.issues.contains(&LengthIssue::GoodBelow {
        percent: 50.0,
        band: (87, 93)
    }));

    let cs = [sc(10.0, 0.0, 5), sc(-10.0, 0.0, 5)];
    let r = report(&cs, 100.0, 1.0, 0.5);
    assert!(r.issues.contains(&LengthIssue::GoodAbove {
        percent: 100.0,
        band: (87, 93)
    }));
}

#[test]
fn report_flags_very_good_runner_over_the_limit() {
    let cs = [sc(10.0, 0.0, 5), sc(-100.0, 0.0, 5)];
    let r = report(&cs, 30.0, 1.0, 1.0);
    assert!(
        r.issues
            .contains(&LengthIssue::VeryGoodMissesControls { missing: 5 })
    );
    // All controls would need 220 s against 30.
    assert!(r.issues.iter().any(
        |i| matches!(i, LengthIssue::VeryGoodOverLimit { percent } if (percent - 220.0 / 30.0 * 100.0).abs() < 1e-9)
    ));
}

#[test]
fn report_flags_very_good_runner_finishing_too_early() {
    let cs = [sc(10.0, 0.0, 9), sc(-10.0, 0.0, 1)];
    let r = report(&cs, 100.0, 5.0, 1.0);
    assert!(r.issues.iter().any(|i| matches!(
        i,
        LengthIssue::VeryGoodTooEarly {
            band: (95, 100),
            ..
        }
    )));
}

#[test]
fn same_seed_same_tour() {
    let cs = scatter(30, 9);
    let run = |seed| {
        best_tour_seeded(
            &cs,
            &[(1, 2)],
            &FAKE,
            pt(200.0, 200.0),
            pt(200.0, 200.0),
            1500.0,
            seed,
        )
        .unwrap()
    };
    assert_eq!(run(7), run(7));
    assert_eq!(run(1), run(1));
}

#[test]
fn heuristic_matches_exact_on_small_sets() {
    for (seed, links) in [
        (21, vec![]),
        (22, vec![]),
        (23, vec![(0, 3)]),
        (24, vec![]),
        (25, vec![(1, 2), (2, 5)]),
    ] {
        let cs = scatter(10, seed);
        for limit in [600.0, 900.0, 1400.0] {
            let p = Problem::new(
                &cs,
                &links,
                &FAKE,
                &pt(200.0, 200.0),
                &pt(200.0, 200.0),
                limit,
            )
            .unwrap();
            let (exact, heur) = (exact::solve(&p), heuristic::solve(&p, 1));
            let (Some(e), Some(h)) = (exact, heur) else {
                // Both agree that no tour holds the links within this limit.
                assert!(exact::solve(&p).is_none() && heuristic::solve(&p, 1).is_none());
                continue;
            };
            assert_eq!(
                p.tour(&h).points,
                p.tour(&e).points,
                "seed {seed}, limit {limit}"
            );
            assert!(p.tour(&h).time_s <= limit + 1e-6);
        }
    }
}

#[test]
#[ignore = "timing: run with --release -- --ignored --nocapture"]
fn sixty_controls_finish_in_seconds() {
    let cs = scatter(60, 4);
    let t0 = Instant::now();
    let t = best_tour(
        &cs,
        &[(3, 4)],
        &FAKE,
        pt(200.0, 200.0),
        pt(200.0, 200.0),
        2500.0,
    )
    .unwrap();
    let el = t0.elapsed();
    println!(
        "60 controls: {} points, {} controls, {:.0} s, solved in {el:?}",
        t.points,
        t.order.len(),
        t.time_s
    );
    assert!(el.as_secs() < 10);
}
