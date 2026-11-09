//! Tests of the start checks. Maps are 1:4000 sprint maps. The start S1 is at (0, 0).

use freemapper_courseplan_model::{Class, Event, SpecialObject};

use super::super::testing::*;
use super::super::*;

fn capacity_rule() -> Rule {
    rule("t.start.capacity", "start_capacity", Severity::Warning)
}

fn angle_rule() -> Rule {
    with_values(
        rule("t.start.angle", "first_leg_angle", Severity::Hint),
        "degrees = 30.0",
    )
}

fn class(name: &str, starters: Option<u32>) -> Class {
    Class {
        name: name.to_owned(),
        rule_category: None,
        expected_starters: starters,
    }
}

fn with_classes(
    mut c: freemapper_courseplan_model::Course,
    names: &[&str],
) -> freemapper_courseplan_model::Course {
    c.classes = names.iter().map(|n| (*n).to_owned()).collect();
    c
}

fn at_start(
    mut c: freemapper_courseplan_model::Course,
    start: &str,
) -> freemapper_courseplan_model::Course {
    c.start = start.to_owned();
    c
}

fn add_start(event: &mut Event, id: &str, x: f64, y: f64) {
    event.specials.push(SpecialObject::Start {
        id: id.to_owned(),
        map_mm: pt(x, y),
    });
}

fn capacity(event: &mut Event, window: Option<u32>, classes: Vec<Class>) -> CheckReport {
    event.event.start_window_min = window;
    event.classes = classes;
    report(event, &map(4000, vec![]), vec![capacity_rule()])
}

fn controls_at(location: &Location) -> Vec<String> {
    match location {
        Location::Controls { codes } => codes.iter().map(|c| c.0.clone()).collect(),
        other => panic!("not at controls: {other:?}"),
    }
}

fn at_control(code: &str) -> Location {
    Location::Control {
        code: super::super::testing::code(code),
    }
}

// ---------------------------------------------------------------------------------------------
// start_capacity

#[test]
fn courses_sharing_a_first_control_without_enough_room_are_reported_at_the_start() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 80.0, 0.0)],
        vec![
            with_classes(course("A", &["31", "32"]), &["H21"]),
            with_classes(course("B", &["31"]), &["D21"]),
        ],
    );
    let r = capacity(
        &mut e,
        Some(30),
        vec![class("H21", Some(31)), class("D21", Some(30))],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location, at_control("S1"));
    assert_eq!(
        en(f),
        "start S1: 3 different first controls are needed for the starters per minute, but only 1 in use; first controls shared by several courses: 31"
    );
    assert_eq!(
        de(f),
        "Start S1: Für die Starter pro Minute braucht es 3 verschiedene erste Posten, belegt sind aber nur 1; erste Posten mehrerer Bahnen: 31"
    );
}

#[test]
fn different_first_controls_give_enough_room() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 50.0)],
        vec![
            with_classes(course("A", &["31"]), &["H21"]),
            with_classes(course("B", &["32"]), &["D21"]),
        ],
    );
    let r = capacity(
        &mut e,
        Some(30),
        vec![class("H21", Some(30)), class("D21", Some(30))],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert!(r.skipped.is_empty(), "{:?}", r.skipped);
}

#[test]
fn a_single_course_with_too_many_starters_has_no_shared_control() {
    let mut e = event(
        &[("31", 50.0, 0.0)],
        vec![with_classes(course("A", &["31"]), &["H21"])],
    );
    let r = capacity(&mut e, Some(30), vec![class("H21", Some(60))]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        en(&r.findings[0]),
        "start S1: 2 different first controls are needed for the starters per minute, but only 1 in use"
    );
    assert_eq!(
        de(&r.findings[0]),
        "Start S1: Für die Starter pro Minute braucht es 2 verschiedene erste Posten, belegt sind aber nur 1"
    );
}

#[test]
fn a_missing_window_is_one_skip_of_the_event() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 50.0)],
        vec![
            with_classes(course("A", &["31"]), &["H21"]),
            with_classes(course("B", &["32"]), &["H21"]),
        ],
    );
    let r = capacity(&mut e, None, vec![class("H21", Some(500))]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1, "{:?}", r.skipped);
    assert_eq!(r.skipped[0].location, Location::Event);
    assert_eq!(r.skipped[0].reason_id, SKIP_START_WINDOW_MISSING);
}

#[test]
fn a_class_without_starters_skips_its_start_and_names_the_classes() {
    let mut e = event(
        &[("31", 50.0, 0.0)],
        vec![
            with_classes(course("A", &["31"]), &["H21", "D21"]),
            with_classes(course("B", &["31"]), &["D21", "H35"]),
        ],
    );
    let r = capacity(
        &mut e,
        Some(30),
        vec![
            class("H21", Some(100)),
            class("D21", None),
            class("H35", None),
        ],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.skipped.len(), 1, "{:?}", r.skipped);
    assert_eq!(r.skipped[0].location, at_control("S1"));
    assert_eq!(r.skipped[0].reason_id, SKIP_STARTERS_MISSING);
    assert_eq!(
        r.skipped[0].args,
        vec![
            ("start", ArgValue::Text("S1".to_owned())),
            ("classes", ArgValue::Text("D21, H35".to_owned())),
        ]
    );
}

#[test]
fn only_the_start_with_missing_starters_is_skipped() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 150.0)],
        vec![
            with_classes(course("A", &["31"]), &["H21"]),
            at_start(with_classes(course("B", &["32"]), &["D21"]), "S2"),
        ],
    );
    add_start(&mut e, "S2", 0.0, 100.0);
    let r = capacity(
        &mut e,
        Some(30),
        vec![class("H21", Some(100)), class("D21", None)],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].location, at_control("S1"));
    assert_eq!(r.skipped.len(), 1, "{:?}", r.skipped);
    assert_eq!(r.skipped[0].location, at_control("S2"));
}

#[test]
fn courses_without_classes_have_no_starters() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 50.0)],
        vec![
            course("A", &["31"]),
            with_classes(course("B", &["32"]), &["H21"]),
        ],
    );
    // B alone needs 2 and has 1; A has no classes, so its first control does not count.
    let r = capacity(&mut e, Some(30), vec![class("H21", Some(40))]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        en(&r.findings[0]),
        "start S1: 2 different first controls are needed for the starters per minute, but only 1 in use"
    );
}

#[test]
fn classes_without_starters_do_not_count() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 50.0)],
        vec![
            with_classes(course("A", &["31"]), &["H21"]),
            with_classes(course("B", &["32"]), &["D21"]),
        ],
    );
    let r = capacity(
        &mut e,
        Some(30),
        vec![class("H21", Some(0)), class("D21", Some(30))],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn courses_at_different_starts_do_not_share_a_minute() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 50.0, 100.0)],
        vec![
            with_classes(course("A", &["31"]), &["H21"]),
            at_start(with_classes(course("B", &["31"]), &["D21"]), "S2"),
        ],
    );
    add_start(&mut e, "S2", 0.0, 100.0);
    // 31 is the first control of both courses, but each start has one course and one minute's room.
    let r = capacity(
        &mut e,
        Some(30),
        vec![class("H21", Some(30)), class("D21", Some(30))],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_course_without_controls_has_starters_but_no_first_control() {
    let mut e = event(
        &[("31", 50.0, 0.0)],
        vec![with_classes(course("A", &[]), &["H21"])],
    );
    let r = capacity(&mut e, Some(30), vec![class("H21", Some(60))]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        en(&r.findings[0]),
        "start S1: 2 different first controls are needed for the starters per minute, but only 0 in use"
    );
}

#[test]
fn a_crossing_point_is_not_a_first_control() {
    let mut e = event(
        &[("X1", 20.0, 0.0), ("31", 50.0, 0.0)],
        vec![
            with_classes(course("A", &["X1", "31"]), &["H21"]),
            with_classes(course("B", &["31"]), &["D21"]),
        ],
    );
    crossing(&mut e, "X1");
    let r = capacity(
        &mut e,
        Some(30),
        vec![class("H21", Some(30)), class("D21", Some(30))],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert!(en(&r.findings[0]).ends_with("shared by several courses: 31"));
}

#[test]
fn the_start_of_a_finding_can_be_accepted() {
    let mut e = event(
        &[("31", 50.0, 0.0)],
        vec![with_classes(course("A", &["31"]), &["H21"])],
    );
    e.accepted_findings
        .push(freemapper_courseplan_model::AcceptedFinding {
            rule: "t.start.capacity".to_owned(),
            controls: vec![code("S1")],
            course: None,
            legs: Vec::new(),
            note: "ok".to_owned(),
        });
    let r = capacity(&mut e, Some(30), vec![class("H21", Some(60))]);
    assert!(r.findings.is_empty());
    assert_eq!(r.accepted.len(), 1);
}

// ---------------------------------------------------------------------------------------------
// first_leg_angle

fn angles(event: &Event) -> CheckReport {
    report(event, &map(4000, vec![]), vec![angle_rule()])
}

#[test]
fn first_legs_in_nearly_the_same_direction_are_a_hint() {
    let e = event(
        &[("31", 50.0, 0.0), ("32", 50.0, 10.0)],
        vec![course("A", &["31"]), course("B", &["32"])],
    );
    let r = angles(&e);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Hint);
    assert_eq!(controls_at(&f.location), ["31", "32"]);
    assert_eq!(
        en(f),
        "start S1: the first legs to controls 31 and 32 differ in direction by only 11°, at least 30° are wanted"
    );
    assert_eq!(
        de(f),
        "Start S1: Die ersten Strecken zu den Posten 31 und 32 unterscheiden sich in der Richtung nur um 11°, gewünscht sind mindestens 30°"
    );
}

#[test]
fn an_angle_of_exactly_the_limit_is_not_reported() {
    let theta = 30.0_f64.to_radians();
    let e = event(
        &[
            ("31", 100.0, 0.0),
            ("32", 100.0 * theta.cos(), 100.0 * theta.sin()),
        ],
        vec![course("A", &["31"]), course("B", &["32"])],
    );
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn wide_directions_are_fine() {
    let e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 50.0)],
        vec![course("A", &["31"]), course("B", &["32"])],
    );
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn courses_sharing_a_first_control_are_no_pair() {
    let e = event(
        &[("31", 50.0, 0.0), ("32", 0.0, 50.0)],
        vec![course("A", &["31", "32"]), course("B", &["31"])],
    );
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn one_course_or_none_gives_nothing() {
    let e = event(&[("31", 50.0, 0.0)], vec![course("A", &["31"])]);
    assert!(angles(&e).findings.is_empty());
    let e = event(&[("31", 50.0, 0.0)], vec![]);
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn a_pair_of_first_controls_is_reported_once() {
    let e = event(
        &[("31", 50.0, 0.0), ("32", 50.0, 10.0)],
        vec![
            course("A", &["31"]),
            course("B", &["32"]),
            course("C", &["31"]),
        ],
    );
    assert_eq!(angles(&e).findings.len(), 1);
}

#[test]
fn courses_at_different_starts_are_not_compared() {
    let mut e = event(
        &[("31", 50.0, 0.0), ("32", 50.0, 100.0)],
        vec![course("A", &["31"]), at_start(course("B", &["32"]), "S2")],
    );
    add_start(&mut e, "S2", 0.0, 100.0);
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn a_crossing_point_gives_the_direction_but_is_no_first_control() {
    let mut e = event(
        &[("X1", 50.0, 0.0), ("31", 200.0, 200.0), ("32", 50.0, 10.0)],
        vec![course("A", &["X1", "31"]), course("B", &["32"])],
    );
    crossing(&mut e, "X1");
    let r = angles(&e);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(controls_at(&r.findings[0].location), ["31", "32"]);
}

#[test]
fn start_bends_set_the_direction() {
    // Without the bend, A runs at 90° to B; the bend turns its first leg to about 6°.
    let mut a = course("A", &["31"]);
    a.start_bends.push(pt(50.0, 5.0));
    let e = event(
        &[("31", 0.0, 50.0), ("32", 50.0, 10.0)],
        vec![a, course("B", &["32"])],
    );
    assert_eq!(angles(&e).findings.len(), 1);

    // And a bend that turns a close leg away.
    let mut a = course("A", &["31"]);
    a.start_bends.push(pt(0.0, 50.0));
    let e = event(
        &[("31", 50.0, 0.0), ("32", 50.0, 10.0)],
        vec![a, course("B", &["32"])],
    );
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn a_course_without_controls_is_left_out() {
    let e = event(
        &[("31", 50.0, 0.0)],
        vec![course("A", &[]), course("B", &["31"])],
    );
    assert!(angles(&e).findings.is_empty());
}

#[test]
fn a_direction_without_length_gives_no_angle() {
    let e = event(
        &[("31", 0.0, 0.0), ("32", 50.0, 0.0)],
        vec![course("A", &["31"]), course("B", &["32"])],
    );
    assert!(angles(&e).findings.is_empty());
}
