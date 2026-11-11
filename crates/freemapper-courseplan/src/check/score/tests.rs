use freemapper_courseplan_model::{
    AcceptedFinding, CourseKind, Event, ScoreControl, ScoreLink, ScoreSettings, SpecialObject,
};

use crate::check::testing::*;
use crate::check::*;

/// An event with a score course "S" from S1 (0, 0) to F1 (100, 0): the controls as (code,
/// value, x, y), limit, penalty and links.
fn score_event(
    controls: &[(&str, u32, f64, f64)],
    limit: u32,
    penalty: u32,
    links: &[(&str, &str)],
) -> Event {
    let positions: Vec<(&str, f64, f64)> = controls.iter().map(|&(c, _, x, y)| (c, x, y)).collect();
    let mut course = course("S", &[]);
    course.kind = CourseKind::Score {
        time_limit_min: limit,
        penalty_per_min: Some(penalty),
        controls: controls
            .iter()
            .map(|&(c, v, _, _)| ScoreControl {
                code: code(c),
                value: v,
            })
            .collect(),
        links: links
            .iter()
            .map(|&(a, b)| ScoreLink {
                a: code(a),
                b: code(b),
            })
            .collect(),
    };
    event(&positions, vec![course])
}

fn five() -> Event {
    score_event(
        &[
            ("31", 1, 20.0, 20.0),
            ("42", 2, 40.0, -20.0),
            ("53", 3, 60.0, 20.0),
            ("64", 4, 80.0, -20.0),
            ("75", 5, 50.0, 40.0),
        ],
        60,
        30,
        &[],
    )
}

fn run_rule(event: &Event, key: &str, values: &str) -> CheckReport {
    run_rule_on(event, &map(4000, vec![]), key, values)
}

fn run_rule_on(
    event: &Event,
    map: &crate::omap::MapDocument,
    key: &str,
    values: &str,
) -> CheckReport {
    let r = with_values(
        rule(&rule_id(key), key, Severity::Warning),
        &values.replace(", ", "\n"),
    );
    report(event, map, vec![r])
}

fn rule_id(key: &str) -> String {
    format!("t.score.{}", key.trim_start_matches("score_"))
}

fn ids(report: &CheckReport) -> Vec<&'static str> {
    report.findings.iter().map(|f| f.message_id).collect()
}

// score_value_matches_code

#[test]
fn value_and_code_agree() {
    let r = run_rule(&five(), "score_value_matches_code", "");
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_value_that_differs_from_its_code_is_a_finding() {
    let mut e = score_event(&[("31", 3, 20.0, 20.0), ("A1", 1, 40.0, 20.0)], 60, 30, &[]);
    let r = run_rule(&e, "score_value_matches_code", "");
    assert_eq!(
        ids(&r),
        ["check-score-value-mismatch", "check-score-value-no-code"]
    );
    assert_eq!(
        r.findings[0].location,
        Location::Control { code: code("31") }
    );
    assert!(
        en(&r.findings[0]).contains("value 1"),
        "{}",
        en(&r.findings[0])
    );
    assert!(
        de(&r.findings[0]).contains("Wert 1"),
        "{}",
        de(&r.findings[0])
    );
    e.courses.clear();
}

// score_code_ambiguity

#[test]
fn codes_that_do_not_read_alike_pass() {
    let r = run_rule(&five(), "score_code_ambiguity", "");
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn codes_that_read_alike_turned_are_a_finding() {
    let e = score_event(&[("68", 8, 20.0, 20.0), ("89", 9, 40.0, 20.0)], 60, 30, &[]);
    let r = run_rule(&e, "score_code_ambiguity", "");
    assert_eq!(ids(&r), ["check-score-code-alike"]);
    assert_eq!(
        r.findings[0].location,
        Location::Controls {
            codes: vec![code("68"), code("89")]
        }
    );
}

// score_ring and score_obvious_order

const RING: &str = "max_gap_degrees = 120.0, max_radius_variation_percent = 25.0";
const ORDER: &str = "min_hull_percent = 60.0, max_chain_excess_percent = 10.0";

/// Eight controls on a circle of radius 40 round the middle of start and finish (50, 0).
fn circle() -> Event {
    let controls: Vec<(String, f64, f64)> = (0..8)
        .map(|i| {
            let a = f64::from(i) * std::f64::consts::TAU / 8.0;
            (
                format!("{}{}", 31 + i, 1),
                50.0 + 40.0 * a.cos(),
                40.0 * a.sin(),
            )
        })
        .collect();
    let list: Vec<(&str, u32, f64, f64)> = controls
        .iter()
        .map(|(c, x, y)| (c.as_str(), 1, *x, *y))
        .collect();
    score_event(&list, 60, 30, &[])
}

/// Eight controls spread inside, not round the middle: no ring, hull share below 60 percent.
fn scatter() -> Event {
    score_event(
        &[
            ("31", 1, 10.0, 5.0),
            ("42", 1, 30.0, 25.0),
            ("53", 1, 45.0, 10.0),
            ("64", 1, 55.0, 30.0),
            ("75", 1, 40.0, 18.0),
            ("86", 1, 35.0, 12.0),
            ("97", 1, 25.0, 15.0),
            ("38", 1, 60.0, 20.0),
        ],
        60,
        30,
        &[],
    )
}

#[test]
fn ring_around_start_and_finish_is_found() {
    let r = run_rule(&circle(), "score_ring", RING);
    assert_eq!(ids(&r), ["check-score-ring"]);
    assert_eq!(r.findings[0].location, Location::Course("S".to_owned()));
}

#[test]
fn a_scatter_is_no_ring() {
    let r = run_rule(&scatter(), "score_ring", RING);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_loop_round_the_edge_is_an_obvious_order() {
    let r = run_rule(&circle(), "score_obvious_order", ORDER);
    assert_eq!(ids(&r), ["check-score-obvious-order"]);
}

#[test]
fn a_scatter_has_no_obvious_order() {
    let r = run_rule(&scatter(), "score_obvious_order", ORDER);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

// score_length_bands

fn open_map() -> crate::omap::MapDocument {
    map(
        4000,
        vec![
            point_object(POINT_113, -10.0, -50.0),
            point_object(POINT_113, 110.0, 50.0),
        ],
    )
}

/// Both bands wide open, so the controls decide whether the very good runner covers them.
fn wide_bands(mut e: Event) -> Event {
    e.score = Some(ScoreSettings {
        good_band_percent: (0, 100),
        very_good_band_percent: (0, 100),
        ..ScoreSettings::default()
    });
    e
}

#[test]
fn a_length_that_fits_the_bands_passes() {
    let e = wide_bands(five());
    let r = run_rule_on(&e, &open_map(), "score_length_bands", "cell_metres = 4.0");
    assert!(r.findings.is_empty(), "{:?} {:?}", r.findings, r.skipped);
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
}

#[test]
fn a_limit_that_the_very_good_runner_cannot_meet_is_a_finding() {
    let mut e = wide_bands(five());
    if let CourseKind::Score { time_limit_min, .. } = &mut e.courses[0].kind {
        *time_limit_min = 1;
    }
    let r = run_rule_on(&e, &open_map(), "score_length_bands", "cell_metres = 4.0");
    assert!(
        ids(&r).contains(&"check-score-length-very-good-misses"),
        "{:?} {:?}",
        r.findings,
        r.skipped
    );
}

#[test]
fn length_bands_are_skipped_on_a_map_without_objects() {
    let r = run_rule(&five(), "score_length_bands", "cell_metres = 4.0");
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].reason_id, SKIP_SCORE_NO_MAP);
    assert_eq!(r.skipped[0].rule.as_deref(), Some("t.score.length_bands"));
}

// score_limit_and_penalty

#[test]
fn a_limit_of_zero_does_not_panic() {
    let mut e = wide_bands(five());
    if let CourseKind::Score { time_limit_min, .. } = &mut e.courses[0].kind {
        *time_limit_min = 0;
    }
    let r = run_rule_on(&e, &open_map(), "score_length_bands", "cell_metres = 4.0");
    assert!(
        ids(&r).contains(&"check-score-length-very-good-misses"),
        "{:?} {:?}",
        r.findings,
        r.skipped
    );
    let r = run_rule(
        &e,
        "score_limit_and_penalty",
        "min_minutes = 30.0, max_minutes = 90.0",
    );
    assert_eq!(ids(&r), ["check-score-limit-range"]);
}

const LIMIT: &str = "min_minutes = 30.0, max_minutes = 90.0";

#[test]
fn a_usual_limit_with_a_fair_penalty_passes() {
    let r = run_rule(&five(), "score_limit_and_penalty", LIMIT);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_limit_outside_the_usual_range_is_a_finding() {
    let e = score_event(&[("31", 1, 20.0, 20.0)], 120, 30, &[]);
    let r = run_rule(&e, "score_limit_and_penalty", LIMIT);
    assert_eq!(ids(&r), ["check-score-limit-range"]);
}

#[test]
fn penalty_that_pays_off_is_a_warning() {
    let e = score_event(&[("31", 1, 20.0, 20.0), ("75", 5, 40.0, 20.0)], 60, 2, &[]);
    let r = run_rule(&e, "score_limit_and_penalty", LIMIT);
    assert_eq!(ids(&r), ["check-score-penalty-pays"]);
    assert_eq!(r.findings[0].severity, Severity::Warning);
    assert!(
        en(&r.findings[0]).contains("2 points"),
        "{}",
        en(&r.findings[0])
    );
}

// score_link_valid

fn linked(links: &[(&str, &str)]) -> Event {
    score_event(
        &[
            ("31", 1, 20.0, 20.0),
            ("42", 2, 40.0, 20.0),
            ("53", 3, 60.0, 20.0),
        ],
        60,
        30,
        links,
    )
}

#[test]
fn a_clear_link_passes() {
    let r = run_rule(&linked(&[("31", "42")]), "score_link_valid", "");
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn link_through_out_of_bounds_is_a_finding() {
    let mut e = linked(&[("31", "53")]);
    e.specials.push(SpecialObject::OutOfBoundsArea {
        id: "O1".to_owned(),
        ppen_id: None,
        polygon: vec![
            pt(35.0, 10.0),
            pt(45.0, 10.0),
            pt(45.0, 30.0),
            pt(35.0, 30.0),
        ],
        courses: vec![],
    });
    let r = run_rule(&e, "score_link_valid", "");
    assert_eq!(ids(&r), ["check-score-link-out-of-bounds"]);
    assert_eq!(
        r.findings[0].location,
        Location::Controls {
            codes: vec![code("31"), code("53")]
        }
    );
    assert!(en(&r.findings[0]).contains("O1"));
}

#[test]
fn a_link_through_a_forbidden_route_is_a_finding() {
    let mut e = linked(&[("31", "53")]);
    e.specials.push(SpecialObject::ForbiddenRoute {
        id: "V1".to_owned(),
        ppen_id: None,
        points: vec![pt(40.0, 0.0), pt(40.0, 40.0)],
        courses: vec![],
    });
    let r = run_rule(&e, "score_link_valid", "");
    assert_eq!(ids(&r), ["check-score-link-forbidden-route"]);
}

#[test]
fn bad_links_are_findings() {
    let e = linked(&[("31", "31"), ("31", "42"), ("42", "31"), ("31", "99")]);
    let r = run_rule(&e, "score_link_valid", "");
    assert_eq!(
        ids(&r),
        [
            "check-score-link-self",
            "check-score-link-duplicate",
            "check-score-link-not-in-course"
        ]
    );
}

#[test]
fn a_link_without_route_choice_is_a_hint_the_setter_can_accept() {
    // A wall with a gap of 4 mm: one route only.
    let objects = vec![
        point_object(POINT_113, -10.0, -50.0),
        point_object(POINT_113, 110.0, 50.0),
        line_object(LINE_518, &[(30.0, -50.0), (30.0, 18.0)]),
        line_object(LINE_518, &[(30.0, 22.0), (30.0, 50.0)]),
    ];
    let e = linked(&[("31", "42")]);
    let r = run_rule_on(&e, &map(4000, objects), "score_link_valid", "");
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
    assert_eq!(ids(&r), ["check-score-link-no-choice"]);
    assert_eq!(r.findings[0].severity, Severity::Hint);
}

// score_mix

const MIX: &str = "tolerance_percent = 15.0";

#[test]
fn an_even_mix_passes() {
    // Values 4 and 5 of 5 are hard: two of five is 40 percent.
    let r = run_rule(&five(), "score_mix", MIX);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_course_of_hard_controls_misses_the_mix() {
    let e = score_event(
        &[
            ("34", 4, 20.0, 20.0),
            ("45", 5, 30.0, 20.0),
            ("54", 4, 40.0, 20.0),
            ("65", 5, 50.0, 20.0),
            ("74", 4, 60.0, 20.0),
        ],
        60,
        30,
        &[],
    );
    let r = run_rule(&e, "score_mix", MIX);
    assert_eq!(ids(&r), ["check-score-mix"]);
    assert!(
        en(&r.findings[0]).contains("100 percent"),
        "{}",
        en(&r.findings[0])
    );
}

// score_min_gap and score_high_gap

#[test]
fn controls_closer_than_the_minimum_gap_are_flagged() {
    // Scale 1:4000: 100 m are 25 mm. 31 and 42 stand 20 mm (80 m) apart, 53 is far away.
    let e = score_event(
        &[
            ("31", 1, 20.0, 20.0),
            ("42", 2, 40.0, 20.0),
            ("53", 3, 90.0, 20.0),
        ],
        60,
        30,
        &[],
    );
    let r = run_rule(&e, "score_min_gap", "");
    assert_eq!(ids(&r), ["check-score-min-gap"]);
    assert!(
        en(&r.findings[0]).contains("80 m"),
        "{}",
        en(&r.findings[0])
    );
    let spread = score_event(
        &[
            ("31", 1, 20.0, 20.0),
            ("42", 2, 70.0, 20.0),
            ("53", 3, 20.0, 70.0),
        ],
        60,
        30,
        &[],
    );
    let far = run_rule(&spread, "score_min_gap", "");
    assert!(far.findings.is_empty(), "{:?}", far.findings);
}

#[test]
fn only_high_valued_pairs_need_the_high_value_gap() {
    // 200 m are 50 mm. 64 and 75 are high (4, 5 of 5) and 30 mm (120 m) apart; 31 and 42 are low.
    let e = score_event(
        &[
            ("31", 1, 20.0, 20.0),
            ("42", 2, 30.0, 20.0),
            ("64", 4, 20.0, 60.0),
            ("75", 5, 50.0, 60.0),
        ],
        60,
        30,
        &[],
    );
    let r = run_rule(&e, "score_high_gap", "");
    assert_eq!(ids(&r), ["check-score-high-gap"]);
    assert!(en(&r.findings[0]).contains("64"), "{}", en(&r.findings[0]));
}

// the engine

#[test]
fn a_score_course_is_no_longer_skipped_by_the_engine() {
    let r = run_rule(&circle(), "score_ring", RING);
    assert!(
        r.skipped.iter().all(|s| s.reason_id != SKIP_COURSE_KIND),
        "{:?}",
        r.skipped
    );
}

#[test]
fn team_and_relay_courses_stay_skipped() {
    let mut e = five();
    let mut team = course("T", &[]);
    team.kind = CourseKind::Team {
        team_size: 3,
        mandatory: vec![],
        free: vec![],
    };
    e.courses.push(team);
    let r = run_rule(&e, "score_ring", RING);
    let skipped: Vec<_> = r
        .skipped
        .iter()
        .filter(|s| s.reason_id == SKIP_COURSE_KIND)
        .collect();
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].location, Location::Course("T".to_owned()));
}

#[test]
fn individual_courses_do_not_reach_the_score_rules() {
    let mut e = five();
    e.courses.push(course("A", &["31", "42"]));
    let r = run_rule(&e, "score_limit_and_penalty", LIMIT);
    assert!(r.findings.is_empty());
}

#[test]
fn accepted_findings_work_on_a_score_rule() {
    let mut e = score_event(&[("31", 1, 20.0, 20.0)], 120, 30, &[]);
    e.accepted_findings = vec![
        AcceptedFinding {
            rule: "t.score.limit_and_penalty".to_owned(),
            controls: vec![],
            course: Some("S".to_owned()),
            legs: vec![],
            note: "long day".to_owned(),
        },
        AcceptedFinding {
            rule: "t.score.limit_and_penalty".to_owned(),
            controls: vec![],
            course: Some("Gone".to_owned()),
            legs: vec![],
            note: "old".to_owned(),
        },
    ];
    let r = run_rule(&e, "score_limit_and_penalty", LIMIT);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.accepted.len(), 1);
    assert_eq!(r.accepted[0].1, "long day");
    assert_eq!(r.unmatched.len(), 1);
    assert_eq!(r.unmatched[0].course.as_deref(), Some("Gone"));
}

/// 60 controls on an open 1:4000 map: `check` must stay in the range of seconds. Run with
/// `cargo test --release -- --ignored score_length_timing --nocapture`.
#[test]
#[ignore]
fn score_length_timing() {
    let controls: Vec<(String, f64, f64)> = (0..60)
        .map(|i| {
            let (c, r) = (i % 10, i / 10);
            (
                format!("{}{}", 31 + (i % 7), 1 + i % 5),
                8.0 + f64::from(c) * 9.0,
                -50.0 + f64::from(r) * 16.0,
            )
        })
        .collect();
    let list: Vec<(&str, u32, f64, f64)> = controls
        .iter()
        .map(|(c, x, y)| (c.as_str(), 1, *x, *y))
        .collect();
    let e = wide_bands(score_event(&list, 60, 30, &[]));
    let t = std::time::Instant::now();
    let big = map(
        4000,
        vec![
            point_object(POINT_113, -30.0, -170.0),
            point_object(POINT_113, 330.0, 170.0),
        ],
    );
    let r = run_rule_on(&e, &big, "score_length_bands", "cell_metres = 4.0");
    eprintln!("{:?}: {:?} {:?}", t.elapsed(), r.findings.len(), r.skipped);
}
