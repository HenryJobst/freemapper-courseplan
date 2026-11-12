use freemapper_courseplan_model::{ControlCode, MapPoint, ScoreSettings};

use super::*;
use crate::score::tour::analyse_length;
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

/// A runner who cannot leave the start.
struct Enclosed;

impl TravelTime for Enclosed {
    fn legs(&self, from: MapPoint, _to: MapPoint) -> Result<LegRoutes, ScoreError> {
        Err(ScoreError::StartUnreachable { at: from })
    }
    fn passable(&self, _p: MapPoint) -> bool {
        false
    }
}

/// Straight lines, but nothing can be reached from or to `lost`.
struct Lossy {
    s_per_unit: f64,
    lost: MapPoint,
}

impl TravelTime for Lossy {
    fn legs(&self, from: MapPoint, to: MapPoint) -> Result<LegRoutes, ScoreError> {
        if from == self.lost || to == self.lost {
            return Err(ScoreError::Unreachable { from, to });
        }
        Fake {
            s_per_unit: self.s_per_unit,
        }
        .legs(from, to)
    }
    fn passable(&self, p: MapPoint) -> bool {
        p != self.lost
    }
}

const GOOD: Fake = Fake { s_per_unit: 1.11 };
const VERY_GOOD: Fake = Fake { s_per_unit: 1.0 };

fn pt(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn cand(x: f64, y: f64, difficulty: f64) -> Candidate {
    Candidate {
        at: pt(x, y),
        symbol_code: None,
        route_choice: 0.5,
        difficulty,
        effort_s: x.hypot(y),
    }
}

/// A wide map: `side` x `side` sites 100 units apart, start and finish in the corner. Every third
/// site is hard.
fn grid(side: usize) -> Vec<Candidate> {
    let mut v = Vec::new();
    for r in 0..side {
        for c in 0..side {
            let hard = (r * side + c).is_multiple_of(3);
            v.push(cand(
                100.0 * (c + 1) as f64,
                100.0 * (r + 1) as f64,
                if hard { 0.9 } else { 0.1 },
            ));
        }
    }
    v
}

struct World {
    cands: Vec<Candidate>,
    links: Vec<(MapPoint, MapPoint)>,
    settings: ScoreSettings,
    limit_min: u32,
    start: MapPoint,
    finish: MapPoint,
    taken: Vec<ControlCode>,
    /// Map scale; large, so that the metre gaps of the settings are tiny next to the fixtures.
    scale: u32,
}

impl World {
    fn new(cands: Vec<Candidate>, limit_min: u32) -> World {
        World {
            cands,
            links: Vec::new(),
            settings: ScoreSettings::default(),
            limit_min,
            start: pt(0.0, 0.0),
            finish: pt(0.0, 0.0),
            taken: Vec::new(),
            scale: 1_000_000,
        }
    }

    fn run(&self) -> Result<Draft, DraftFailure> {
        draft(&DraftRequest {
            candidates: &self.cands,
            travel_good: &GOOD,
            travel_very_good: &VERY_GOOD,
            start: self.start.clone(),
            finish: self.finish.clone(),
            limit_min: self.limit_min,
            links: &self.links,
            settings: &self.settings,
            taken: &self.taken,
            scale: self.scale,
        })
    }
}

/// Time of the very good runner for every candidate at once: the length of the whole course.
fn t_all(cands: &[Candidate]) -> f64 {
    let controls: Vec<ScoredControl> = cands
        .iter()
        .map(|c| ScoredControl {
            at: c.at.clone(),
            value: 1,
        })
        .collect();
    analyse_length(
        &controls,
        &[],
        &GOOD,
        &VERY_GOOD,
        pt(0.0, 0.0),
        pt(0.0, 0.0),
        f64::INFINITY,
        &ScoreSettings::default(),
    )
    .expect("the fake has a route everywhere")
    .very_good
    .time_s
}

/// A limit in minutes that both bands can hold for this fixture. The very good runner needs
/// `t_all` for every candidate; 0.96 of it leaves a few candidates out, which is where the good
/// runner lands in its band (found once on the grid and the ring; adjust the factor here only).
fn ok_limit(cands: &[Candidate]) -> u32 {
    (0.96 * t_all(cands) / 60.0).ceil() as u32
}

/// Twice the whole course: every candidate fits and the set is still too short.
fn too_long(cands: &[Candidate]) -> u32 {
    (2.0 * t_all(cands) / 60.0).ceil() as u32
}

fn ring(n: usize) -> Vec<Candidate> {
    (0..n)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / n as f64;
            Candidate {
                at: pt(300.0 * a.cos(), 300.0 * a.sin()),
                symbol_code: None,
                route_choice: 0.5,
                difficulty: 0.3,
                effort_s: 300.0,
            }
        })
        .collect()
}

fn adjacent(order: &[usize], a: usize, b: usize) -> bool {
    let (Some(i), Some(j)) = (
        order.iter().position(|&x| x == a),
        order.iter().position(|&x| x == b),
    ) else {
        return false;
    };
    i.abs_diff(j) == 1
}

#[test]
fn a_draft_meets_both_bands() {
    let d = World::new(grid(4), ok_limit(&grid(4)))
        .run()
        .expect("a draft");
    assert!(d.report.issues.is_empty());
    assert!((87.0..=93.0).contains(&d.report.good_percent), "{d:?}");
    assert!((95.0..=100.0).contains(&d.report.very_good_finish_percent));
    let mut codes: Vec<&str> = d.controls.iter().map(|c| c.code.0.as_str()).collect();
    for c in &d.controls {
        assert!((1..=5).contains(&c.value));
        assert_eq!(
            crate::score::code::value_of_code(&c.code.0),
            Some(c.value),
            "the code carries the value"
        );
    }
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), d.controls.len(), "codes are unique");
}

#[test]
fn codes_avoid_the_taken_ones() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.taken = vec![ControlCode("31".into()), ControlCode("32".into())];
    let d = w.run().expect("a draft");
    assert!(
        d.controls
            .iter()
            .all(|c| c.code.0 != "31" && c.code.0 != "32")
    );
}

#[test]
fn mix_share_bounds_the_hard_controls() {
    let d = World::new(grid(4), ok_limit(&grid(4)))
        .run()
        .expect("a draft");
    let k = d.controls.len();
    let hard = d.controls.iter().filter(|c| c.hard).count();
    assert_eq!(hard, d.hard_actual);
    assert_eq!(hard, (0.4 * k as f64).round() as usize);
    assert_eq!(d.mix_missed_percent, None);
    for share in [20_u8, 60] {
        let mut w = World::new(grid(4), ok_limit(&grid(4)));
        w.settings.mix_hard_percent = share;
        let d = match w.run() {
            Ok(d) => d,
            Err(f) => *f.best,
        };
        let k = d.controls.len() as f64;
        let wanted = (f64::from(share) / 100.0 * k).round() as usize;
        assert_eq!(d.hard_wanted, wanted);
        match d.mix_missed_percent {
            None => assert_eq!(d.hard_actual, wanted, "share {share}"),
            Some(_) => assert_ne!(d.hard_actual, wanted, "share {share}"),
        }
    }
}

#[test]
fn effort_wins_when_mix_and_effort_conflict_and_the_report_names_it() {
    // No hard candidate at all, but 40 percent hard are asked for.
    let easy: Vec<Candidate> = grid(4)
        .into_iter()
        .map(|mut c| {
            c.difficulty = 0.1;
            c
        })
        .collect();
    let d = World::new(easy.clone(), ok_limit(&easy))
        .run()
        .expect("effort bands still hold");
    assert!(d.report.issues.is_empty());
    assert_eq!(d.hard_actual, 0);
    assert!(d.hard_wanted > 0);
    let missed = d.mix_missed_percent.expect("the missed share is named");
    assert!((missed - 100.0 * d.hard_wanted as f64 / d.controls.len() as f64).abs() < 1e-9);
}

#[test]
fn not_enough_candidates_fails_with_hint() {
    let f = World::new(grid(4)[..3].to_vec(), ok_limit(&grid(4)))
        .run()
        .expect_err("too few");
    assert_eq!(f.hint, Hint::MoreCandidates);
    assert!(!f.issues.is_empty());
}

#[test]
fn too_long_a_limit_asks_for_a_shorter_one_or_more_candidates() {
    let f = World::new(grid(4), too_long(&grid(4)))
        .run()
        .expect_err("every candidate is in");
    assert_eq!(f.hint, Hint::MoreCandidates);
    assert_eq!(f.best.controls.len(), 16);
}

#[test]
fn zero_candidates_fail() {
    let f = World::new(Vec::new(), ok_limit(&grid(4)))
        .run()
        .expect_err("nothing to draft");
    assert_eq!(f.hint, Hint::MoreCandidates);
    assert!(f.best.controls.is_empty());
}

#[test]
fn impossible_limit_fails_with_hint_and_best_attempt() {
    let f = World::new(grid(4), ok_limit(&grid(4)) / 10)
        .run()
        .expect_err("one minute is too short");
    assert_eq!(f.hint, Hint::LimitUp);
    assert!(
        !f.best.controls.is_empty(),
        "the best attempt is kept for display"
    );
    assert!(!f.issues.is_empty());
}

#[test]
fn a_limit_shorter_than_start_to_finish_fails() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)) / 2);
    w.finish = pt(1000.0, 0.0);
    let f = w.run().expect_err("the direct leg does not fit");
    assert_eq!(f.hint, Hint::LimitUp);
    assert!(f.best.controls.is_empty());
}

#[test]
fn requested_links_are_in_the_draft_and_runnable() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.links = vec![
        (pt(100.0, 100.0), pt(200.0, 200.0)),
        (pt(400.0, 400.0), pt(300.0, 400.0)),
    ];
    let d = w.run().expect("a draft with links");
    assert_eq!(d.links.len(), 2);
    for &(a, b) in &d.links {
        assert!(
            adjacent(&d.report.good.order, a, b),
            "{:?}",
            d.report.good.order
        );
        assert!(adjacent(&d.report.very_good.order, a, b));
    }
    let at = |i: usize| (d.controls[i].at.x, d.controls[i].at.y);
    assert_eq!(at(d.links[0].0), (100.0, 100.0));
    assert_eq!(at(d.links[1].1), (300.0, 400.0));
}

#[test]
fn a_link_that_cannot_be_run_in_time_fails() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)) / 2);
    w.links = vec![(pt(100.0, 100.0), pt(400.0, 400.0))];
    let f = w.run().expect_err("the link does not fit");
    assert_eq!(f.hint, Hint::LimitUp);
    assert!(
        matches!(f.error, Some(ScoreError::LinkDoesNotFit { .. })),
        "{:?}",
        f.error
    );
}

#[test]
fn equal_effort_still_spreads_the_values() {
    let d = World::new(ring(16), ok_limit(&ring(16)))
        .run()
        .expect("a draft");
    let values: Vec<u8> = d.controls.iter().map(|c| c.value).collect();
    assert!(values.len() >= 5);
    assert!(values.iter().all(|v| (1..=5).contains(v)));
    let mut distinct = values.clone();
    distinct.sort_unstable();
    distinct.dedup();
    assert!(distinct.len() == 5, "{values:?}");
}

#[test]
fn two_runs_are_byte_equal() {
    let a = format!("{:?}", World::new(grid(4), ok_limit(&grid(4))).run());
    let b = format!("{:?}", World::new(grid(4), ok_limit(&grid(4))).run());
    assert_eq!(a, b);
    let c = format!("{:?}", World::new(grid(4), too_long(&grid(4))).run());
    let d = format!("{:?}", World::new(grid(4), too_long(&grid(4))).run());
    assert_eq!(c, d);
}

#[test]
fn no_draft_when_start_is_enclosed() {
    let w = World::new(grid(4), ok_limit(&grid(4)));
    let f = draft(&DraftRequest {
        candidates: &w.cands,
        travel_good: &Enclosed,
        travel_very_good: &VERY_GOOD,
        start: w.start.clone(),
        finish: w.finish.clone(),
        limit_min: ok_limit(&w.cands),
        links: &[],
        settings: &w.settings,
        taken: &[],
        scale: 1_000_000,
    })
    .expect_err("no route from the start");
    assert_eq!(f.hint, Hint::Impossible);
    assert!(matches!(f.error, Some(ScoreError::StartUnreachable { .. })));
}

#[test]
fn start_may_equal_finish_or_not() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.start = pt(50.0, 50.0);
    w.finish = pt(50.0, 50.0);
    // Either a draft or a clean failure; the point is that equal ends work.
    match w.run() {
        Ok(d) => assert!(d.report.issues.is_empty()),
        Err(f) => assert!(f.error.is_none()),
    }
    w.finish = pt(0.0, 50.0);
    match w.run() {
        Ok(d) => assert!(d.report.issues.is_empty()),
        Err(f) => assert!(f.error.is_none()),
    }
}

#[test]
fn a_link_end_off_every_candidate_names_the_link() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.links = vec![
        (pt(100.0, 100.0), pt(200.0, 200.0)),
        (pt(1.0, 1.0), pt(2.0, 2.0)),
    ];
    let f = w.run().expect_err("no candidate there");
    assert_eq!(f.hint, Hint::Impossible);
    assert_eq!(f.error, Some(ScoreError::LinkEndNotACandidate { link: 1 }));
}

#[test]
fn a_bad_link_names_the_candidate() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.links = vec![(pt(200.0, 100.0), pt(200.0, 100.0))];
    let f = w.run().expect_err("a link needs two controls");
    assert_eq!(f.error, Some(ScoreError::BadLink { a: 1, b: 1 }));
}

#[test]
fn a_count_without_codes_is_never_a_draft() {
    // Every code of value 5 is taken, so a set that gives value 5 to any control cannot be coded.
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.taken = (3..=9).map(|t| ControlCode(format!("{t}5"))).collect();
    match w.run() {
        Ok(d) => assert!(
            d.controls
                .iter()
                .all(|c| !c.code.0.is_empty() && c.value != 5)
        ),
        Err(f) => {
            assert!(f.best.controls.iter().all(|c| !c.code.0.is_empty()));
            if f.hint == Hint::Impossible {
                assert!(f.error.is_some());
            }
        }
    }
}

#[test]
fn a_draft_raises_no_mix_finding_at_the_default_settings() {
    use crate::check::testing as t;
    use freemapper_courseplan_model::{CourseKind, ScoreControl};

    let mut drafts = 0;
    for cands in [grid(4), ring(12)] {
        let Ok(d) = World::new(cands.clone(), ok_limit(&cands)).run() else {
            continue;
        };
        drafts += 1;
        let list: Vec<(&str, f64, f64)> = d
            .controls
            .iter()
            .map(|c| (c.code.0.as_str(), c.at.x, c.at.y))
            .collect();
        let mut event = t::event(&list, vec![t::course("S", &[])]);
        event.courses[0].kind = CourseKind::Score {
            time_limit_min: 60,
            penalty_per_min: Some(30),
            controls: d
                .controls
                .iter()
                .map(|c| ScoreControl {
                    code: c.code.clone(),
                    value: u32::from(c.value),
                })
                .collect(),
            links: vec![],
        };
        let rule = t::with_values(
            t::rule("t.score.mix", "score_mix", crate::rules::Severity::Warning),
            "tolerance_percent = 15.0",
        );
        let report = t::report(&event, &t::map(4000, vec![]), vec![rule]);
        assert!(report.findings.is_empty(), "{:?}", report.findings);
    }
    assert!(drafts >= 2, "only {drafts} fixtures give a draft");
}

#[test]
fn a_pool_cut_short_by_the_code_space_asks_for_codes_not_for_the_map() {
    // value_max 1 leaves 7 codes; the ring has 20 sites and the limit fits every one of them.
    let cands = ring(20);
    let mut w = World::new(cands.clone(), too_long(&cands));
    w.settings.value_max = 1;
    let f = w.run().expect_err("seven codes cannot cover twenty sites");
    assert_eq!(f.hint, Hint::CodeSpace);
    assert_eq!(f.best.controls.len(), 7);
    // Codes outside the value range take no room: 36 and 37 are no codes of value 1 to 5... but 31 is.
    w.taken = vec![ControlCode("31".into())];
    let f = w.run().expect_err("six codes left");
    assert_eq!(f.hint, Hint::CodeSpace);
    assert_eq!(f.best.controls.len(), 6);
}

#[test]
fn a_candidate_without_a_route_from_its_position_is_left_out() {
    let mut cands = grid(4);
    let lost = pt(5000.0, 5000.0);
    cands.push(Candidate {
        effort_s: 10.0,
        ..cand(5000.0, 5000.0, 0.5)
    });
    let limit = ok_limit(&grid(4));
    let good = Lossy {
        s_per_unit: 1.11,
        lost: lost.clone(),
    };
    let very_good = Lossy {
        s_per_unit: 1.0,
        lost: lost.clone(),
    };
    let d = draft(&DraftRequest {
        candidates: &cands,
        travel_good: &good,
        travel_very_good: &very_good,
        start: pt(0.0, 0.0),
        finish: pt(0.0, 0.0),
        limit_min: limit,
        links: &[],
        settings: &ScoreSettings::default(),
        taken: &[],
        scale: 1_000_000,
    })
    .expect("the lost candidate must not block the draft");
    assert!(d.controls.iter().all(|c| c.at != lost));
}

/// Pairs of controls of the draft that are high-valued and closer than `gap` units.
fn close_high_pairs(d: &Draft, high_from: u8, gap: f64) -> usize {
    let mut n = 0;
    for (i, a) in d.controls.iter().enumerate() {
        for b in &d.controls[i + 1..] {
            if a.value >= high_from && b.value >= high_from && dist(&a.at, &b.at) < gap {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn high_valued_controls_keep_the_high_value_gap() {
    // Scale 1000: one metre is one unit. The grid sits 100 units apart, the gap asks 150.
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.scale = 1000;
    w.settings.high_value_gap_m = 150;
    let d = match w.run() {
        Ok(d) => d,
        Err(f) => *f.best,
    };
    assert!(d.controls.iter().any(|c| c.value >= 4), "{d:?}");
    assert_eq!(close_high_pairs(&d, 4, 150.0), 0, "{d:?}");
    // Without the rule the same grid does put high controls closer.
    let mut loose = World::new(grid(4), ok_limit(&grid(4)));
    loose.scale = 1000;
    loose.settings.high_value_gap_m = 1;
    let d = match loose.run() {
        Ok(d) => d,
        Err(f) => *f.best,
    };
    assert!(close_high_pairs(&d, 4, 150.0) > 0, "{d:?}");
}

#[test]
fn first_violation_names_the_later_high_control_and_spares_forced_ones() {
    let cands = vec![
        cand(0.0, 0.0, 0.1),
        cand(10.0, 0.0, 0.1),
        cand(500.0, 0.0, 0.1),
    ];
    let picks = [0, 1, 2];
    assert_eq!(
        first_violation(&picks, &[5, 4, 5], &cands, 4, 50.0, 0),
        Some(1)
    );
    assert_eq!(
        first_violation(&picks, &[5, 4, 5], &cands, 4, 50.0, 2),
        None
    );
    assert_eq!(
        first_violation(&picks, &[5, 3, 5], &cands, 4, 50.0, 0),
        None
    );
    assert_eq!(first_violation(&picks, &[5, 4, 5], &cands, 4, 5.0, 0), None);
}

/// Runs the draft of a world with a budget of local-search evaluations.
fn run_with_budget(w: &World, budget: usize) -> Result<Draft, DraftFailure> {
    draft_with_budget(
        &DraftRequest {
            candidates: &w.cands,
            travel_good: &GOOD,
            travel_very_good: &VERY_GOOD,
            start: w.start.clone(),
            finish: w.finish.clone(),
            limit_min: w.limit_min,
            links: &w.links,
            settings: &w.settings,
            taken: &w.taken,
            scale: w.scale,
        },
        budget,
    )
}

/// Limit in minutes at `pct` percent of the time the very good runner needs for the whole grid.
fn share_limit(cands: &[Candidate], pct: u32) -> u32 {
    ((t_all(cands) / 60.0) as u32 * pct / 100).max(2)
}

#[test]
fn the_local_search_finds_a_set_the_prefixes_miss() {
    // Measured once on this grid (ADR 0074): at these shares of the whole course no prefix of the spread
    // order meets both bands, while a swap or a value step does.
    let cands = grid(5);
    for pct in [30, 40, 50] {
        let w = World::new(cands.clone(), share_limit(&cands, pct));
        assert!(run_with_budget(&w, 0).is_err(), "prefixes alone at {pct}");
        let d = run_with_budget(&w, REFINE_BUDGET).expect("the local search meets the bands");
        assert!(d.report.issues.is_empty(), "{pct}: {:?}", d.report.issues);
        assert!((87.0..=93.0).contains(&d.report.good_percent));
        assert!((95.0..=100.0).contains(&d.report.very_good_finish_percent));
        for c in &d.controls {
            assert_eq!(
                crate::score::code::value_of_code(&c.code.0),
                Some(c.value),
                "the code carries the value"
            );
            assert!((1..=5).contains(&c.value));
        }
    }
}

#[test]
fn the_local_search_never_spoils_a_draft_and_is_repeatable() {
    let cands = grid(5);
    for pct in [70, 75, 80] {
        let w = World::new(cands.clone(), share_limit(&cands, pct));
        let first = run_with_budget(&w, 0).expect("prefixes meet the bands here");
        let with = run_with_budget(&w, REFINE_BUDGET).expect("still a draft");
        assert_eq!(first, with, "a draft that meets the bands is left alone");
    }
    let w = World::new(cands.clone(), share_limit(&cands, 40));
    assert_eq!(
        run_with_budget(&w, REFINE_BUDGET),
        run_with_budget(&w, REFINE_BUDGET)
    );
}

#[test]
fn the_local_search_keeps_links_and_the_high_value_gap() {
    let cands = grid(5);
    let mut w = World::new(cands.clone(), share_limit(&cands, 40));
    w.links = vec![(cands[0].at.clone(), cands[1].at.clone())];
    let d = match run_with_budget(&w, REFINE_BUDGET) {
        Ok(d) => d,
        Err(f) => *f.best,
    };
    assert_eq!(d.links.len(), 1);
    for &(a, b) in &d.links {
        for tour in [&d.report.good, &d.report.very_good] {
            assert!(
                adjacent(&tour.order, a, b),
                "linked controls run one after the other"
            );
        }
    }
    let gap = f64::from(w.settings.high_value_gap_m) * 1000.0 / f64::from(w.scale);
    assert_eq!(close_high_pairs(&d, w.settings.high_from(), gap), 0);
}

// --- The bands are targets; the tolerance only decides what may be returned (ADR 0083) --------------------------

#[test]
fn a_draft_near_the_target_succeeds_and_names_the_deviation() {
    // A band no count of this fixture hits: the good runner lands 1 to 5 points beside it.
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    w.settings.good_band_percent = (95, 95);
    w.settings.band_tolerance_percent = 0;
    let strict = w.run();
    w.settings.band_tolerance_percent = 6;
    let d = w.run().expect("within the tolerance");
    assert!(d.report.is_ok());
    if strict.is_err() {
        assert!(!d.report.issues.is_empty(), "the deviation is named: {d:?}");
        assert!(d.report.near().count() > 0);
        assert_eq!(d.report.misses().count(), 0);
    }
}

#[test]
fn an_exact_draft_stays_exact_with_a_tolerance() {
    let mut w = World::new(grid(4), ok_limit(&grid(4)));
    let exact = w.run().expect("a draft");
    assert!(exact.report.issues.is_empty());
    w.settings.band_tolerance_percent = 10;
    // The search aims at the band, not at the edge of the tolerance.
    let wide = w.run().expect("a draft");
    assert!(wide.report.issues.is_empty(), "{wide:?}");
    assert_eq!(wide.controls.len(), exact.controls.len());
}

#[test]
fn beyond_the_tolerance_a_draft_fails_with_the_misses_only() {
    let mut w = World::new(grid(4), too_long(&grid(4)));
    w.settings.band_tolerance_percent = 3;
    let f = w
        .run()
        .expect_err("every candidate is in and the set is too short");
    assert!(!f.issues.is_empty());
    assert!(f.issues.iter().all(|i| !i.is_near(3)), "{:?}", f.issues);
}
