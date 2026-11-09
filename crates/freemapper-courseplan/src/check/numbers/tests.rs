//! Tests of the number gap check. Maps are 1:4000 sprint maps.

use freemapper_courseplan_model::{CourseKind, Event, NumberPlacement};

use super::super::testing::*;
use super::super::*;

fn gap_rule(id: &str, values: &str) -> Rule {
    with_values(rule(id, "number_gap", Severity::Warning), values)
}

fn range_rule() -> Rule {
    gap_rule("t.gap", "min_gap_mm = 0.5\nmax_gap_mm = 5.0")
}

fn placed(gap_mm: f64) -> Option<NumberPlacement> {
    Some(NumberPlacement {
        direction: 90.0,
        gap_mm,
    })
}

/// Two courses A and B over 31 and 32; `own` sets the own number of 31 in A.
fn gap_event(central: Option<f64>, own_a: Option<f64>) -> Event {
    let mut e = event(
        &[("31", 30.0, 10.0), ("32", 60.0, 10.0)],
        vec![course("A", &["31", "32"]), course("B", &["31", "32"])],
    );
    e.controls[0].number = central.and_then(placed);
    if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
        controls[0].number = own_a.and_then(placed);
    }
    e
}

#[test]
fn course_number_below_min_warns() {
    let r = report(
        &gap_event(None, Some(0.3)),
        &map(4000, vec![]),
        vec![range_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].message_id, "check-number-gap-small");
    assert_eq!(r.findings[0].location, leg("A", 0));
    assert_eq!(
        en(&r.findings[0]),
        "the number of control 31 is 0.3 mm from the circle, at least 0.5 mm wanted"
    );
    assert_eq!(
        de(&r.findings[0]),
        "Die Nummer von Posten 31 steht 0,3 mm vom Kreis entfernt, erwünscht sind mindestens 0,5 mm"
    );
}

#[test]
fn course_number_above_max_warns() {
    let r = report(
        &gap_event(None, Some(6.0)),
        &map(4000, vec![]),
        vec![range_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].message_id, "check-number-gap-large");
    assert_eq!(
        en(&r.findings[0]),
        "the number of control 31 is 6 mm from the circle, at most 5 mm wanted"
    );
}

#[test]
fn gap_inside_range_is_silent() {
    for gap in [0.5, 5.0] {
        let r = report(
            &gap_event(None, Some(gap)),
            &map(4000, vec![]),
            vec![range_rule()],
        );
        assert!(r.findings.is_empty(), "{gap}: {:?}", r.findings);
    }
}

#[test]
fn central_number_outside_range_reported_once() {
    let r = report(
        &gap_event(Some(7.0), None),
        &map(4000, vec![]),
        vec![range_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        r.findings[0].location,
        Location::Control { code: code("31") }
    );
}

#[test]
fn course_number_overrides_central() {
    let r = report(
        &gap_event(Some(7.0), Some(1.0)),
        &map(4000, vec![]),
        vec![range_rule()],
    );
    // Course A uses its own 1.0; course B still reports the central 7.0, once, at the control.
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        r.findings[0].location,
        Location::Control { code: code("31") }
    );
    assert!(en(&r.findings[0]).contains("7 mm"));
}

#[test]
fn stricter_sibling_reports_alone() {
    let rules = vec![
        gap_rule("t.a", "min_gap_mm = 0.5"),
        gap_rule("t.b", "min_gap_mm = 1.0"),
    ];
    let r = report(&gap_event(None, Some(0.3)), &map(4000, vec![]), rules);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].rule, "t.b");

    let rules = vec![
        gap_rule("t.a", "max_gap_mm = 5.0"),
        gap_rule("t.b", "max_gap_mm = 3.0"),
    ];
    let r = report(&gap_event(None, Some(6.0)), &map(4000, vec![]), rules);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].rule, "t.b");
}

#[test]
fn controls_without_placement_are_skipped() {
    let r = report(
        &gap_event(None, None),
        &map(4000, vec![]),
        vec![range_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}
