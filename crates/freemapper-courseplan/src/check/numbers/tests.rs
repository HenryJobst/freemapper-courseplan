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

#[test]
fn crossing_point_with_a_stored_number_is_skipped() {
    let mut e = gap_event(None, Some(8.0));
    crossing(&mut e, "31");
    let r = report(&e, &map(4000, vec![]), vec![range_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn rule_with_max_below_min_is_raised_like_placement() {
    let rule = gap_rule("t.swap", "min_gap_mm = 6.0\nmax_gap_mm = 5.0");
    let r = report(&gap_event(None, Some(6.0)), &map(4000, vec![]), vec![rule]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn invalid_rule_values_are_ignored() {
    let rule = gap_rule("t.nan", "min_gap_mm = nan\nmax_gap_mm = -2.0");
    let r = report(&gap_event(None, Some(8.0)), &map(4000, vec![]), vec![rule]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.skipped.len(), 1, "{:?}", r.skipped);
}

#[test]
fn placement_with_the_profile_range_passes_the_check() {
    use crate::course::control_numbers;
    use crate::map::{MapIndex, MapKind};
    use crate::numbers::{auto_place, gap_range};
    use crate::rules::load_rule_set;
    use freemapper_courseplan_model::Discipline;

    let set = load_rule_set("wkb-2026", &[]).unwrap();
    let range = gap_range(&set, MapKind::Sprint, Discipline::Sprint, 4000);
    let mut e = event(
        &[("31", 30.0, 10.0), ("32", 60.0, 10.0), ("33", 62.0, 10.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    crossing(&mut e, "32");
    // Symbols near the first control force gaps beyond the minimum.
    let doc = map(
        4000,
        vec![
            point_object(POINT_113, 34.6, 10.0),
            line_object(LINE_502, &[(33.0, 12.0), (33.0, 5.0)]),
        ],
    );
    let index = MapIndex::from_document(&doc);
    let placed = auto_place(
        &e,
        &e.courses[0],
        &index,
        4000,
        MapKind::Sprint,
        range,
        false,
    )
    .unwrap();
    assert_eq!(placed.len(), 2);
    let numbers = control_numbers(&e, &e.courses[0]);
    let mut next = placed.into_iter();
    if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
        for (k, cc) in controls.iter_mut().enumerate() {
            if numbers[k].is_some() {
                cc.number = next.next();
            }
        }
    }
    let rules: Vec<Rule> = set
        .rules
        .iter()
        .filter(|r| r.rule.key == "number_gap")
        .map(|r| r.rule.clone())
        .collect();
    assert!(!rules.is_empty());
    let r = report(&e, &doc, rules);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}
