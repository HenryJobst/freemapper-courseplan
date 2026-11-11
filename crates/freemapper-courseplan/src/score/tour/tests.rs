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

/// Every link as two consecutive controls of the order, in either direction.
fn holds_links(order: &[usize], links: &[(usize, usize)]) -> bool {
    links
        .iter()
        .all(|&(a, b)| order.contains(&a) && order.contains(&b) && adjacent(order, a, b))
}

#[test]
fn heuristic_matches_exact_on_small_sets() {
    let link_sets: [Vec<(usize, usize)>; 4] = [
        vec![],
        vec![(0, 3)],
        vec![(1, 2), (2, 5)],
        vec![(4, 7), (6, 1), (2, 3)],
    ];
    let mut compared = 0;
    for seed in 21..31 {
        let cs = scatter(10, seed);
        for links in &link_sets {
            for limit in [600.0, 900.0, 1400.0] {
                let p = Problem::new(
                    &cs,
                    links,
                    &FAKE,
                    &pt(200.0, 200.0),
                    &pt(200.0, 200.0),
                    limit,
                )
                .unwrap();
                let (exact, heur) = (
                    exact::solve(&p),
                    heuristic::solve(&p, u32::from(seed as u8)),
                );
                let (Some(e), Ok(h)) = (exact, heur) else {
                    // Both agree that no tour holds the links within this limit.
                    assert!(exact::solve(&p).is_none() && heuristic::solve(&p, 1).is_err());
                    continue;
                };
                let (et, ht) = (p.tour(&e), p.tour(&h));
                assert_eq!(
                    ht.points, et.points,
                    "seed {seed}, {links:?}, limit {limit}"
                );
                assert!(ht.time_s <= limit + 1e-6);
                assert!(holds_links(&ht.order, links), "{:?}", ht.order);
                compared += 1;
            }
        }
    }
    assert!(compared > 60, "{compared}");
}

/// Best (points, time) over every ordered subset that holds the links.
fn brute_force(
    cs: &[ScoredControl],
    links: &[(usize, usize)],
    start: (f64, f64),
    limit: f64,
) -> Option<(u32, f64)> {
    fn go(
        cs: &[ScoredControl],
        links: &[(usize, usize)],
        limit: f64,
        order: &mut Vec<usize>,
        best: &mut Option<(u32, f64)>,
        start: (f64, f64),
    ) {
        let dist = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1);
        if holds_links(order, links) {
            let mut at = start;
            let mut t = 0.0;
            for &c in order.iter() {
                let q = (cs[c].at.x, cs[c].at.y);
                t += dist(at, q);
                at = q;
            }
            t += dist(at, start);
            let pts: u32 = order.iter().map(|&c| u32::from(cs[c].value)).sum();
            let ok = t <= limit + 1e-9;
            let improves = best.is_none_or(|(bp, bt)| pts > bp || (pts == bp && t < bt - 1e-9));
            if ok && improves {
                *best = Some((pts, t));
            }
        }
        for c in 0..cs.len() {
            if !order.contains(&c) {
                order.push(c);
                go(cs, links, limit, order, best, start);
                order.pop();
            }
        }
    }
    let mut best = None;
    go(cs, links, limit, &mut Vec::new(), &mut best, start);
    best
}

#[test]
fn exact_solver_matches_brute_force_with_chain_and_pair_links() {
    let sets: [Vec<(usize, usize)>; 5] = [
        vec![(0, 1), (1, 2), (4, 5)],
        vec![(2, 1), (1, 0), (5, 4)],
        vec![(1, 0), (2, 1), (4, 5)],
        vec![(3, 6)],
        vec![],
    ];
    for seed in 1..=6u32 {
        let cs = scatter(7, seed);
        for links in &sets {
            for limit in [500.0, 900.0, 1500.0] {
                let want = brute_force(&cs, links, (200.0, 200.0), limit);
                let got = tour(&cs, links, (200.0, 200.0), (200.0, 200.0), limit);
                match (want, got) {
                    (Some((wp, wt)), Ok(t)) if !t.order.is_empty() || wp == 0 => {
                        assert_eq!(t.points, wp, "seed {seed}, {links:?}, limit {limit}");
                        assert!((t.time_s - wt).abs() < 1e-6);
                        assert!(holds_links(&t.order, links));
                    }
                    (None, Err(ScoreError::LinkDoesNotFit { .. })) => {}
                    (None, Ok(t)) => assert!(links.is_empty() && t.order.is_empty()),
                    (w, g) => panic!("seed {seed}, {links:?}, limit {limit}: {w:?} against {g:?}"),
                }
            }
        }
    }
}

#[test]
fn heuristic_holds_every_link_above_the_exact_size() {
    let cs = scatter(40, 8);
    let links = [(0, 1), (1, 2), (5, 6), (9, 8), (20, 30)];
    let t = tour(&cs, &links, (200.0, 200.0), (200.0, 200.0), 1800.0).unwrap();
    assert!(t.time_s <= 1800.0 + 1e-6);
    assert!(holds_links(&t.order, &links), "{:?}", t.order);
    assert_eq!(t.time_s, {
        let p = Problem::new(
            &cs,
            &links,
            &FAKE,
            &pt(200.0, 200.0),
            &pt(200.0, 200.0),
            1800.0,
        )
        .unwrap();
        let seq = heuristic::solve(&p, ScoreSettings::default().seed).unwrap();
        p.eval(&seq)
    });
}

#[test]
fn seventy_controls_with_links_do_not_overflow_the_unit_count() {
    let cs = scatter(70, 17);
    let links = [(0, 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11), (12, 13)];
    let t = tour(&cs, &links, (200.0, 200.0), (200.0, 200.0), 1500.0).unwrap();
    assert!(t.time_s <= 1500.0 + 1e-6);
    assert!(holds_links(&t.order, &links), "{:?}", t.order);
}

#[test]
fn a_heuristic_miss_is_not_called_a_proven_misfit() {
    // Sixteen separate links and a limit that no tour holds: above 12 linked units the search
    // cannot prove it, so the error says only that nothing was found.
    let cs = scatter(40, 6);
    let links: Vec<(usize, usize)> = (0..16).map(|i| (2 * i, 2 * i + 1)).collect();
    let err = tour(&cs, &links, (200.0, 200.0), (200.0, 200.0), 150.0).unwrap_err();
    assert!(
        matches!(err, ScoreError::LinkTourNotFound { .. }),
        "{err:?}"
    );
}

#[test]
fn several_chains_are_repaired_when_the_greedy_order_fails() {
    // The limit is the exact optimum for the linked units alone, so the cheapest-insertion order
    // has to match it or a better start is needed.
    let mut greedy_failed = 0;
    for seed in 30..60u32 {
        let cs = scatter(24, seed);
        let links = [(0, 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11)];
        let p0 = Problem::new(
            &cs,
            &links,
            &FAKE,
            &pt(200.0, 200.0),
            &pt(200.0, 200.0),
            f64::INFINITY,
        )
        .unwrap();
        let linked: Vec<usize> = (0..p0.units.len())
            .filter(|&k| p0.units[k].mandatory)
            .collect();
        let opt = p0.eval(&exact::solve_units(&p0, &linked).unwrap());
        let tight = Problem::new(
            &cs,
            &links,
            &FAKE,
            &pt(200.0, 200.0),
            &pt(200.0, 200.0),
            opt + 1e-6,
        )
        .unwrap();
        greedy_failed += usize::from(heuristic::greedy_linked(&tight).is_none());
        let t = tour(&cs, &links, (200.0, 200.0), (200.0, 200.0), opt + 1e-6)
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        assert!(holds_links(&t.order, &links));
        assert!(t.time_s <= opt + 1e-6 + 1e-9);
    }
    assert!(greedy_failed > 0, "no instance made the greedy order fail");
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

/// Straight lines, except that nothing can be reached from or to `lost`.
struct Lossy {
    lost: MapPoint,
}

impl TravelTime for Lossy {
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
        if from == self.lost || to == self.lost {
            return Err(ScoreError::Unreachable { from, to });
        }
        FAKE.legs(from, to)
    }
    fn passable(&self, p: MapPoint) -> bool {
        p != self.lost
    }
}

#[test]
fn an_unreachable_control_never_gives_an_infinite_over_limit_percent() {
    // More than 12 controls, so the heuristic runs; the unlimited solve must not take the lost one.
    let mut cs = scatter(14, 7);
    cs.push(sc(999.0, 999.0, 3));
    let travel = Lossy {
        lost: pt(999.0, 999.0),
    };
    let r = analyse_length(
        &cs,
        &[],
        &travel,
        &travel,
        pt(0.0, 0.0),
        pt(0.0, 0.0),
        100_000.0,
        &ScoreSettings::default(),
    )
    .unwrap();
    assert!(
        r.issues
            .iter()
            .any(|i| matches!(i, LengthIssue::VeryGoodMissesControls { .. }))
    );
    for issue in &r.issues {
        if let LengthIssue::VeryGoodOverLimit { percent } = issue {
            panic!("an unreachable control must not give an over-limit percent: {percent}");
        }
    }
    assert!(r.good.time_s.is_finite() && r.very_good.time_s.is_finite());
}
