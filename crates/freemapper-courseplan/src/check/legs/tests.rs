//! Tests of the leg shape checks. Maps are 1:4000 sprint maps, so one map mm is 4 m: the
//! opposing-legs values of 10 m and 30 m are 2.5 mm and 7.5 mm.

use freemapper_courseplan_model::{AcceptedFinding, Control, Event, LegRef, SpecialObject};

use super::super::testing::*;
use super::super::*;
use crate::rules::{SwitchedOff, load_rule_set};

fn warning_rule() -> Rule {
    with_values(
        rule("t.leg.acute", "acute_angle", Severity::Warning),
        "degrees = 45.0",
    )
}

fn hint_rule() -> Rule {
    with_values(
        rule("t.leg.acute_hint", "acute_angle", Severity::Hint),
        "degrees = 90.0",
    )
}

fn straight_rule() -> Rule {
    with_values(
        rule("t.leg.straight", "straight_angle", Severity::Hint),
        "degrees = 170.0",
    )
}

fn opposing_rule() -> Rule {
    with_values(
        rule("t.leg.opposing", "opposing_legs", Severity::Warning),
        "metres = 10.0\nmin_length_metres = 30.0\nmin_angle_degrees = 135.0",
    )
}

fn set_finish(event: &mut Event, x: f64, y: f64) {
    for special in &mut event.specials {
        if let SpecialObject::Finish { map_mm, .. } = special {
            *map_mm = pt(x, y);
        }
    }
}

fn set_start(event: &mut Event, x: f64, y: f64) {
    for special in &mut event.specials {
        if let SpecialObject::Start { map_mm, .. } = special {
            *map_mm = pt(x, y);
        }
    }
}

/// Course A: S1 (0, 0), 31 (50, 0), then 32 at `degrees` from the leg back to the start, and the
/// finish straight on from 32, so only the angle at 31 is acute.
fn angle_event(degrees: f64) -> Event {
    let theta = (180.0 - degrees).to_radians();
    let (dx, dy) = (theta.cos(), theta.sin());
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 50.0 + 40.0 * dx, 40.0 * dy)],
        vec![course("A", &["31", "32"])],
    );
    set_finish(&mut e, 50.0 + 80.0 * dx, 80.0 * dy);
    e
}

/// As `angle_event`, but the finish turns 90 degrees at 32, so only the angle at 31 is a candidate
/// for a straight angle (the angle at 32 is 90 degrees).
fn straight_event(degrees: f64) -> Event {
    let theta = (180.0 - degrees).to_radians();
    let (dx, dy) = (theta.cos(), theta.sin());
    let (x, y) = (50.0 + 40.0 * dx, 40.0 * dy);
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", x, y)],
        vec![course("A", &["31", "32"])],
    );
    set_finish(&mut e, x - 40.0 * dy, y + 40.0 * dx);
    e
}

fn legs_at(location: &Location) -> (&str, Vec<u32>) {
    match location {
        Location::Legs { course, legs } => {
            (course.as_str(), legs.iter().map(|l| l.after).collect())
        }
        other => panic!("not at legs: {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// acute_angle

#[test]
fn an_accepted_angle_comes_back_when_its_leg_runs_to_another_control() {
    // Ruling R24: the acceptance stores the leg's ends, so it no longer matches once the
    // control at position 1 is replaced, even though the angle and the position stay.
    let mut e = angle_event(30.0);
    let rules = rule_set(vec![(warning_rule(), "base")]);
    let mut acceptance = AcceptedFinding {
        rule: "t.leg.acute".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![LegRef::at("A", 0)],
        note: "on purpose".to_owned(),
    };
    crate::edit::accept_finding(&mut e, &rules, acceptance.clone()).unwrap();
    let map = map(4000, vec![]);
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    assert_eq!((r.findings.len(), r.accepted.len()), (0, 1), "{r:?}");
    assert_eq!(
        r.accepted[0].0.location.text(&lang_of("de")),
        "Bahn A, Strecke Start–31"
    );
    // Control 39 takes the place of 31, at the same spot.
    let at = e.controls[0].map_mm.clone();
    e.controls.push(Control {
        code: code("39"),
        map_mm: at,
        ..e.controls[0].clone()
    });
    // Edited in the file, not through the edit operation, which would move or drop the acceptance.
    if let freemapper_courseplan_model::CourseKind::Individual { controls } = &mut e.courses[0].kind
    {
        controls[0].code = code("39");
    }
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    assert_eq!(r.findings.len(), 1, "{r:?}");
    assert_eq!(r.findings[0].location, leg("A", 0));
    assert_eq!(
        r.findings[0].location.text(&lang_of("en")),
        "course A, leg start–39"
    );
    assert!(r.accepted.is_empty());
    assert_eq!(r.unmatched.len(), 1);
    // Accepting it again stores the new ends.
    acceptance.note = "still on purpose".to_owned();
    crate::edit::accept_finding(&mut e, &rules, acceptance).unwrap();
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    assert_eq!((r.findings.len(), r.accepted.len()), (0, 1), "{r:?}");
}

#[test]
fn an_acute_angle_at_a_control_is_reported_at_the_incoming_leg() {
    let e = angle_event(30.0);
    let r = report(&e, &map(4000, vec![]), vec![warning_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location, leg("A", 0));
    assert_eq!(
        en(f),
        "the legs at control 31 form an angle of 30°, at least 45° required"
    );
    assert_eq!(
        de(f),
        "Die Strecken am Posten 31 bilden einen Winkel von 30°, gefordert sind mindestens 45°"
    );
    // A wide angle gives nothing.
    let r = report(
        &angle_event(100.0),
        &map(4000, vec![]),
        vec![warning_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_place_gets_only_the_strictest_rule_of_the_key() {
    let rules = || vec![warning_rule(), hint_rule()];
    // 30° breaks both limits: only the warning.
    let r = report(&angle_event(30.0), &map(4000, vec![]), rules());
    let found: Vec<(&str, Severity)> = r
        .findings
        .iter()
        .map(|f| (f.rule.as_str(), f.severity))
        .collect();
    assert_eq!(found, [("t.leg.acute", Severity::Warning)]);
    // 60° breaks only the hint's limit.
    let r = report(&angle_event(60.0), &map(4000, vec![]), rules());
    let found: Vec<(&str, Severity)> = r
        .findings
        .iter()
        .map(|f| (f.rule.as_str(), f.severity))
        .collect();
    assert_eq!(found, [("t.leg.acute_hint", Severity::Hint)]);
    assert_eq!(
        en(&r.findings[0]),
        "the legs at control 31 form an angle of 60°, at least 90° required"
    );
    // Accepting the warning at 30° does not bring up the hint.
    let mut e = angle_event(30.0);
    e.accepted_findings.push(AcceptedFinding {
        rule: "t.leg.acute".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![LegRef {
            course: "A".to_owned(),
            after: 0,
            from: "S1".to_owned(),
            to: "31".to_owned(),
        }],
        note: "on purpose".to_owned(),
    });
    let r = report(&e, &map(4000, vec![]), rules());
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.accepted.len(), 1);
    assert_eq!(r.accepted[0].0.rule, "t.leg.acute");
}

#[test]
fn a_stricter_rule_for_another_print_scale_does_not_silence_the_hint() {
    let mut warning = warning_rule();
    crate::rules::set_print_scales(&mut warning, &[10000]);
    let r = report(
        &angle_event(30.0),
        &map(4000, vec![]),
        vec![warning, hint_rule()],
    );
    let rules: Vec<&str> = r.findings.iter().map(|f| f.rule.as_str()).collect();
    assert_eq!(rules, ["t.leg.acute_hint"]);
}

#[test]
fn a_switched_off_warning_leaves_the_angle_to_the_hint() {
    // As an overlay with `enabled = false` would: the 45° warning moves to `switched_off`.
    let mut rules = load_rule_set("wkb-2026", &[]).unwrap();
    let i = rules
        .rules
        .iter()
        .position(|r| r.rule.id == "wkb2026.leg.acute_angle")
        .unwrap();
    let off = rules.rules.remove(i);
    rules.switched_off.push(SwitchedOff {
        rule: off.rule,
        profile: off.profile,
        by: "test-overlay".to_owned(),
    });
    let e = angle_event(30.0);
    let map = map(4000, vec![]);
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    let acute: Vec<(&str, String)> = r
        .findings
        .iter()
        .filter(|f| f.rule.starts_with("wkb2026.leg.acute"))
        .map(|f| (f.rule.as_str(), en(f)))
        .collect();
    assert_eq!(
        acute,
        [(
            "wkb2026.leg.acute_angle_hint",
            "the legs at control 31 form an angle of 30°, at least 90° required".to_owned()
        )]
    );
}

#[test]
fn the_wkb_profile_gives_one_warning_for_a_sharp_angle() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let e = angle_event(30.0);
    let map = map(4000, vec![]);
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    let acute: Vec<&Finding> = r
        .findings
        .iter()
        .filter(|f| f.rule.starts_with("wkb2026.leg.acute"))
        .collect();
    assert_eq!(acute.len(), 1, "{:?}", r.findings);
    assert_eq!(acute[0].rule, "wkb2026.leg.acute_angle");
    assert_eq!(acute[0].severity, Severity::Warning);
    let e = angle_event(70.0);
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    let acute: Vec<&str> = r
        .findings
        .iter()
        .filter(|f| f.rule.starts_with("wkb2026.leg.acute"))
        .map(|f| f.rule.as_str())
        .collect();
    assert_eq!(acute, ["wkb2026.leg.acute_angle_hint"]);
}

#[test]
fn angles_use_the_last_bend_before_and_the_first_bend_after_the_control() {
    // Straight through 31, but the leg after 31 first bends back towards the start.
    let mut a = course("A", &["31"]);
    bend_after(&mut a, 0, 20.0, 10.0);
    let e = event(&[("31", 50.0, 0.0)], vec![a]);
    let r = report(&e, &map(4000, vec![]), vec![warning_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    // atan(10 / 30) = 18.4°, shown rounded down.
    assert_eq!(
        en(&r.findings[0]),
        "the legs at control 31 form an angle of 18°, at least 45° required"
    );
    // A bend before 31 changes the incoming direction: (0, 1) against (-3, 1) is 71.6°.
    let mut a = course("A", &["31"]);
    a.start_bends = vec![pt(50.0, 30.0)];
    bend_after(&mut a, 0, 20.0, 10.0);
    let e = event(&[("31", 50.0, 0.0)], vec![a]);
    let r = report(&e, &map(4000, vec![]), vec![warning_rule(), hint_rule()]);
    let found: Vec<(&str, String)> = r
        .findings
        .iter()
        .map(|f| (f.rule.as_str(), en(f)))
        .collect();
    assert_eq!(
        found,
        [(
            "t.leg.acute_hint",
            "the legs at control 31 form an angle of 71°, at least 90° required".to_owned()
        )]
    );
}

#[test]
fn start_and_finish_have_no_angle_and_courses_without_controls_are_checked() {
    // S1 (0, 0) and F1 (100, 0): the course without controls has no angle.
    let e = event(&[], vec![course("A", &[])]);
    let r = report(
        &e,
        &map(4000, vec![]),
        vec![warning_rule(), opposing_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
    // One control: one angle, at 31. The finish lies back beside the start, so S1 and F1 would
    // be sharp if they counted.
    let mut e = event(&[("31", 50.0, 0.0)], vec![course("A", &["31"])]);
    set_finish(&mut e, 0.0, 10.0);
    let r = report(&e, &map(4000, vec![]), vec![warning_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].location, leg("A", 0));
    assert!(
        en(&r.findings[0]).contains("angle of 11°"),
        "{}",
        en(&r.findings[0])
    );
}

#[test]
fn an_acute_angle_at_a_crossing_point_is_named_as_such() {
    let mut e = angle_event(30.0);
    crossing(&mut e, "31");
    let r = report(&e, &map(4000, vec![]), vec![warning_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        en(&r.findings[0]),
        "the legs at crossing point 31 form an angle of 30°, at least 45° required"
    );
    assert_eq!(
        de(&r.findings[0]),
        "Die Strecken an der Querungsstelle 31 bilden einen Winkel von 30°, gefordert sind mindestens 45°"
    );
}

#[test]
fn each_visit_of_a_butterfly_control_has_its_own_angle() {
    // 31 at the centre, loop 32-33 to the north, loop 34-35 to the south. The first visit of 31
    // turns by 56°; the second (33 to 34) and the third (35 to the finish) are wide.
    let e = event(
        &[
            ("31", 50.0, 0.0),
            ("32", 30.0, 30.0),
            ("33", 70.0, 30.0),
            ("34", 70.0, -30.0),
            ("35", 30.0, -30.0),
        ],
        vec![course("A", &["31", "32", "33", "31", "34", "35", "31"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![warning_rule(), hint_rule()]);
    let found: Vec<(LegAt, &str)> = r
        .findings
        .iter()
        .map(|f| (leg_of(&f.location), f.rule.as_str()))
        .collect();
    let hint = "t.leg.acute_hint";
    assert_eq!(
        found,
        [
            (leg("A", 0), hint),
            (leg("A", 1), hint),
            (leg("A", 2), hint),
            (leg("A", 4), hint),
            (leg("A", 5), hint),
        ]
    );
    assert!(en(&r.findings[0]).contains("control 31 form an angle of 56°"));
}

#[test]
fn a_repeated_control_or_a_bend_on_the_control_gives_no_angle() {
    // 31 twice in a row: the leg between has no length.
    let e = event(&[("31", 50.0, 0.0)], vec![course("A", &["31", "31"])]);
    let r = report(
        &e,
        &map(4000, vec![]),
        vec![warning_rule(), opposing_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    // A bend on the control: the outgoing direction has no length.
    let mut a = course("A", &["31"]);
    bend_after(&mut a, 0, 50.0, 0.0);
    bend_after(&mut a, 0, 20.0, 10.0);
    let e = event(&[("31", 50.0, 0.0)], vec![a]);
    let r = report(&e, &map(4000, vec![]), vec![warning_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_taped_leg_has_its_angle_checked() {
    let mut e = angle_event(30.0);
    taped_after(&mut e.courses[0], 0);
    e.courses[0].start_taped = true;
    let r = report(&e, &map(4000, vec![]), vec![warning_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn a_rule_without_degrees_is_skipped() {
    let bare = rule("t.leg.acute", "acute_angle", Severity::Warning);
    let r = report(&angle_event(30.0), &map(4000, vec![]), vec![bare]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].reason_id, SKIP_VALUE_MISSING);
    assert_eq!(
        r.skipped[0].text(&lang_of("en")),
        "the rule has no value degrees"
    );
}

// ---------------------------------------------------------------------------------------------
// opposing_legs

/// Course A: S1 (0, 0) east to 31 (60, 0), north to 32 (60, 2), west to 33 (0, 2) and north to
/// the finish at (0, 40). The leg from the start and the leg after 2 run 8 m apart against each
/// other for 240 m.
fn opposing_event() -> Event {
    let mut e = event(
        &[("31", 60.0, 0.0), ("32", 60.0, 2.0), ("33", 0.0, 2.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    set_finish(&mut e, 0.0, 40.0);
    e
}

#[test]
fn legs_running_against_each_other_are_a_warning_naming_both() {
    let r = report(&opposing_event(), &map(4000, vec![]), vec![opposing_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(legs_at(&f.location), ("A", vec![0, 2]));
    assert_eq!(
        f.location.text(&lang_of("en")),
        "course A, legs start–31 and 32–33"
    );
    assert_eq!(
        f.location.text(&lang_of("de")),
        "Bahn A, Strecken Start–31 und 32–33"
    );
    assert_eq!(
        en(f),
        "the two legs run in opposite directions for 240 m at most 10 m apart, less than 30 m \
         allowed"
    );
    assert_eq!(
        de(f),
        "Die beiden Strecken verlaufen auf 240 m gegenläufig in höchstens 10 m Abstand, erlaubt \
         sind weniger als 30 m"
    );
}

#[test]
fn legs_after_controls_are_named_by_their_positions() {
    // The start further south, so the opposing legs are the leg after 2 and the leg after 4.
    let mut e = event(
        &[
            ("31", 10.0, -30.0),
            ("32", 10.0, 0.0),
            ("33", 60.0, 0.0),
            ("34", 60.0, 2.0),
            ("35", 10.0, 2.0),
        ],
        vec![course("A", &["31", "32", "33", "34", "35"])],
    );
    set_start(&mut e, 10.0, -60.0);
    set_finish(&mut e, 10.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    let places: Vec<(&str, Vec<u32>)> = r.findings.iter().map(|f| legs_at(&f.location)).collect();
    assert_eq!(places, [("A", vec![2, 4])], "{:?}", r.findings);
    assert_eq!(
        r.findings[0].location.text(&lang_of("en")),
        "course A, legs 32–33 and 34–35"
    );
}

#[test]
fn legs_further_apart_or_in_the_same_direction_give_nothing() {
    // 33 and the finish 3 mm (12 m) north of the first leg.
    let mut e = event(
        &[("31", 60.0, 0.0), ("32", 60.0, 3.0), ("33", 0.0, 3.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    set_finish(&mut e, 0.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    // Same direction: a square, then the leg to the finish runs east 2 mm beside the leg from
    // the start.
    let mut e = event(
        &[
            ("31", 60.0, 0.0),
            ("32", 60.0, 40.0),
            ("33", 0.0, 40.0),
            ("34", 0.0, 2.0),
        ],
        vec![course("A", &["31", "32", "33", "34"])],
    );
    set_finish(&mut e, 60.0, 2.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_short_opposing_stretch_is_below_the_minimum_length() {
    // The leg after 2 runs back along the first leg for 7 mm (28 m) only.
    let mut e = event(
        &[("31", 60.0, 0.0), ("32", 60.0, 2.0), ("33", 53.0, 2.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    set_finish(&mut e, 53.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    // Within 2.5 mm of the leg after 2 means x from 53 - 1.5 to 60: 8.5 mm along the first leg,
    // but only 7 mm along the leg after 2; the shorter counts.
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn legs_meeting_at_a_control_are_exempt_near_it() {
    // A dog-leg at 31: back along the leg from the start, 0.3 mm beside it, for 9 mm. 2.5 mm
    // around 31 are exempt, so 6.5 mm (26 m) count: below 30 m.
    let mut e = event(
        &[("31", 60.0, 0.0), ("32", 51.0, 0.3)],
        vec![course("A", &["31", "32"])],
    );
    set_finish(&mut e, 51.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    // 11 mm back: 8.5 mm (34 m) count.
    let mut e = event(
        &[("31", 60.0, 0.0), ("32", 49.0, 0.3)],
        vec![course("A", &["31", "32"])],
    );
    set_finish(&mut e, 49.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(legs_at(&r.findings[0].location), ("A", vec![0, 1]));
    assert!(
        en(&r.findings[0]).contains("for 34 m"),
        "{}",
        en(&r.findings[0])
    );
}

#[test]
fn a_one_control_course_that_returns_along_its_first_leg() {
    let mut e = event(&[("31", 50.0, 0.0)], vec![course("A", &["31"])]);
    set_finish(&mut e, 0.0, 2.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(legs_at(&r.findings[0].location), ("A", vec![0, 1]));
}

#[test]
fn legs_meeting_at_a_crossing_point_are_exempt_near_it() {
    // As the short dog-leg above, at a crossing point.
    let mut e = event(
        &[("31", 60.0, 0.0), ("32", 51.0, 0.3)],
        vec![course("A", &["31", "32"])],
    );
    crossing(&mut e, "31");
    set_finish(&mut e, 51.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_butterfly_centre_is_a_meeting_point() {
    // Both loops leave and return through 31 in the same directions: no opposing legs.
    let e = event(
        &[
            ("31", 50.0, 0.0),
            ("32", 30.0, 30.0),
            ("33", 52.0, 30.0),
            ("34", 48.0, -30.0),
            ("35", 70.0, -30.0),
        ],
        vec![course("A", &["31", "32", "33", "31", "34", "35", "31"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn opposing_stretches_of_a_butterfly_far_from_its_centre_count() {
    // The second loop runs back along the first: out along the return leg of loop 1, back
    // along its middle leg, home along its first leg.
    let e = event(
        &[
            ("31", 50.0, 0.0),
            ("32", 30.0, 30.0),
            ("33", 70.0, 30.0),
            ("34", 70.0, 31.0),
            ("35", 30.0, 31.0),
        ],
        vec![course("A", &["31", "32", "33", "31", "34", "35", "31"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    let places: Vec<(&str, Vec<u32>)> = r.findings.iter().map(|f| legs_at(&f.location)).collect();
    assert_eq!(
        places,
        [("A", vec![1, 6]), ("A", vec![2, 5]), ("A", vec![3, 4])],
        "{:?}",
        r.findings
    );
}

#[test]
fn a_taped_leg_is_checked_like_any_other() {
    let mut e = opposing_event();
    e.courses[0].start_taped = true;
    taped_after(&mut e.courses[0], 1);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn a_leg_with_bends_is_compared_along_its_bends() {
    // The leg after 2 runs from 32 (60, 2) to 33 (0, 30). Straight it leaves the leg from the
    // start at once; with a bend at (0, 2) it runs back along it.
    let setup = |bend: Option<(f64, f64)>| {
        let mut a = course("A", &["31", "32", "33"]);
        if let Some((x, y)) = bend {
            bend_after(&mut a, 1, x, y);
        }
        let mut e = event(
            &[("31", 60.0, 0.0), ("32", 60.0, 2.0), ("33", 0.0, 30.0)],
            vec![a],
        );
        set_finish(&mut e, 0.0, 60.0);
        report(&e, &map(4000, vec![]), vec![opposing_rule()])
    };
    let r = setup(None);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    let r = setup(Some((0.0, 2.0)));
    let places: Vec<(&str, Vec<u32>)> = r.findings.iter().map(|f| legs_at(&f.location)).collect();
    assert_eq!(places, [("A", vec![0, 2])], "{:?}", r.findings);
    // A bend that takes the opposing leg away removes the finding.
    let mut e = opposing_event();
    let CourseKind::Individual { controls } = &mut e.courses[0].kind else {
        unreachable!()
    };
    controls[1].bends.push(pt(30.0, 20.0));
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

/// Course A: the leg from the start runs east along y = 0 to 31 (60, 0); the leg after 2 crosses
/// it head-on at (30, 0), turned by `degrees` from the opposite direction, 25 mm to either side.
fn crossing_event(degrees: f64) -> Event {
    let (c, s) = (degrees.to_radians().cos(), degrees.to_radians().sin());
    let mut e = event(
        &[
            ("31", 60.0, 0.0),
            ("32", 30.0 + 25.0 * c, 25.0 * s),
            ("33", 30.0 - 25.0 * c, -25.0 * s),
        ],
        vec![course("A", &["31", "32", "33"])],
    );
    set_finish(&mut e, 30.0 - 25.0 * c, -60.0);
    e
}

#[test]
fn legs_crossing_head_on_at_a_shallow_angle_are_opposing() {
    // Ruling R17: near a crossing at angle a, each leg is within 10 m of the other for
    // 2 × 10 m / sin a, which reaches 30 m below about 41.8°.
    let r = report(
        &crossing_event(40.0),
        &map(4000, vec![]),
        vec![opposing_rule()],
    );
    let places: Vec<(&str, Vec<u32>)> = r.findings.iter().map(|f| legs_at(&f.location)).collect();
    assert_eq!(places, [("A", vec![0, 2])], "{:?}", r.findings);
    // 20 m / sin 40° = 31.1 m.
    assert!(
        en(&r.findings[0]).contains("for 31 m"),
        "{}",
        en(&r.findings[0])
    );
    // At 50° the directions differ by 130°, below 135°, and the stretch would be 26 m.
    let r = report(
        &crossing_event(50.0),
        &map(4000, vec![]),
        vec![opposing_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn legs_of_different_courses_are_not_compared() {
    // A runs east from the start along y = 0; B runs west along y = 2 in its leg after 2.
    let mut e = event(
        &[
            ("31", 60.0, 0.0),
            ("32", 60.0, 40.0),
            ("41", 60.0, -20.0),
            ("42", 60.0, 2.0),
            ("43", 0.0, 2.0),
        ],
        vec![course("A", &["31", "32"]), course("B", &["41", "42", "43"])],
    );
    set_finish(&mut e, 0.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![opposing_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn an_opposing_rule_without_its_values_is_skipped() {
    let bare = with_values(
        rule("t.leg.opposing", "opposing_legs", Severity::Warning),
        "metres = 10.0\nmin_angle_degrees = 135.0",
    );
    let r = report(&opposing_event(), &map(4000, vec![]), vec![bare]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(
        r.skipped[0].text(&lang_of("en")),
        "the rule has no value min_length_metres"
    );
}

#[test]
fn the_wkb_profile_warns_about_opposing_legs() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let e = opposing_event();
    let map = map(4000, vec![]);
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    let opposing: Vec<&Finding> = r
        .findings
        .iter()
        .filter(|f| f.rule == "wkb2026.leg.opposing")
        .collect();
    assert_eq!(opposing.len(), 1, "{:?}", r.findings);
    assert_eq!(opposing[0].severity, Severity::Warning);
    // Accepted with both legs, in any order.
    let mut e = e;
    e.accepted_findings.push(AcceptedFinding {
        rule: "wkb2026.leg.opposing".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![
            LegRef {
                course: "A".to_owned(),
                after: 2,
                from: "32".to_owned(),
                to: "33".to_owned(),
            },
            LegRef {
                course: "A".to_owned(),
                after: 0,
                from: "S1".to_owned(),
                to: "31".to_owned(),
            },
        ],
        note: "n".to_owned(),
    });
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    assert!(r.findings.iter().all(|f| f.rule != "wkb2026.leg.opposing"));
    assert_eq!(r.accepted.len(), 1);
}

// ---------------------------------------------------------------------------------------------
// straight_angle

#[test]
fn a_straight_angle_at_a_control_is_reported_at_the_incoming_leg() {
    let r = report(
        &straight_event(175.0),
        &map(4000, vec![]),
        vec![straight_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].location, leg("A", 0));
    assert_eq!(
        en(&r.findings[0]),
        "the legs at control 31 form an angle of 175°, at most 170° wanted"
    );
    assert_eq!(
        de(&r.findings[0]),
        "Die Strecken am Posten 31 bilden einen Winkel von 175°, erwünscht sind höchstens 170°"
    );
    let r = report(
        &straight_event(160.0),
        &map(4000, vec![]),
        vec![straight_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_straight_angle_is_measured_to_the_bend_after_the_control() {
    // 31 lies straight between S1 and 32, but the leg after 31 first bends back: the angle is
    // measured to that bend, so it is sharp and no straight angle.
    let mut a = course("A", &["31", "32"]);
    bend_after(&mut a, 0, 20.0, 20.0);
    let mut e = event(&[("31", 50.0, 0.0), ("32", 90.0, 0.0)], vec![a]);
    set_finish(&mut e, 90.0, 40.0);
    let r = report(&e, &map(4000, vec![]), vec![straight_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_straight_angle_shows_rounded_up() {
    let r = report(
        &straight_event(170.4),
        &map(4000, vec![]),
        vec![straight_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        en(&r.findings[0]),
        "the legs at control 31 form an angle of 171°, at most 170° wanted"
    );
}

#[test]
fn straight_angle_skips_crossing_points() {
    let mut e = straight_event(175.0);
    crossing(&mut e, "31");
    let r = report(&e, &map(4000, vec![]), vec![straight_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn only_the_most_extreme_straight_rule_reports() {
    let warning = with_values(
        rule(
            "t.leg.straight_warning",
            "straight_angle",
            Severity::Warning,
        ),
        "degrees = 175.0",
    );
    let r = report(
        &straight_event(178.0),
        &map(4000, vec![]),
        vec![straight_rule(), warning.clone()],
    );
    let found: Vec<(&str, Severity)> = r
        .findings
        .iter()
        .map(|f| (f.rule.as_str(), f.severity))
        .collect();
    assert_eq!(found, [("t.leg.straight_warning", Severity::Warning)]);
    // 172° breaks only the hint.
    let r = report(
        &straight_event(172.0),
        &map(4000, vec![]),
        vec![straight_rule(), warning],
    );
    let rules: Vec<&str> = r.findings.iter().map(|f| f.rule.as_str()).collect();
    assert_eq!(rules, ["t.leg.straight"]);
}

#[test]
fn the_wkb_profile_hints_at_straight_angles() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let e = straight_event(175.0);
    let map = map(4000, vec![]);
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    let found: Vec<&Finding> = r
        .findings
        .iter()
        .filter(|f| f.rule.starts_with("wkb2026.leg.straight"))
        .collect();
    assert_eq!(found.len(), 1, "{:?}", r.findings);
    assert_eq!(found[0].rule, "wkb2026.leg.straight_angle_hint");
    assert_eq!(found[0].severity, Severity::Hint);
}
