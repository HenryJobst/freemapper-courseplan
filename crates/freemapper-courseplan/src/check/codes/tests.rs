//! Tests of the control code checks. Maps are 1:4000 sprint maps, so the control circle radius is
//! 3 mm (12 m).

use freemapper_courseplan_model::Event;

use super::super::testing::*;
use super::super::*;

fn range_rule() -> Rule {
    with_values(
        rule("t.code.range", "code_range", Severity::Error),
        "range = [31, 599]\nnon_numeric_severity = \"warning\"",
    )
}

fn pairs_rule() -> Rule {
    with_values(
        rule("t.code.pairs", "code_pairs", Severity::Error),
        "pairs = [[66, 99], [68, 89], [86, 98]]",
    )
}

fn confusable_rule() -> Rule {
    rule(
        "t.code.confusable",
        "code_confusable_next",
        Severity::Warning,
    )
}

fn same_rule() -> Rule {
    with_values(
        rule("t.code.same", "same_objects_in_circle", Severity::Warning),
        "feature_search_mm = 1.0",
    )
}

/// Controls on a row, 10 mm apart, starting at x = 10, y = 20.
fn row(codes: &[&str]) -> Vec<(String, f64, f64)> {
    codes
        .iter()
        .enumerate()
        .map(|(i, c)| ((*c).to_owned(), 10.0 + 10.0 * i as f64, 20.0))
        .collect()
}

fn event_of(codes: &[&str], courses: Vec<Course>) -> Event {
    let placed = row(codes);
    let refs: Vec<(&str, f64, f64)> = placed
        .iter()
        .map(|(c, x, y)| (c.as_str(), *x, *y))
        .collect();
    event(&refs, courses)
}

fn run(e: &Event, rules: Vec<Rule>) -> CheckReport {
    report(e, &map(4000, vec![]), rules)
}

fn codes_of(f: &Finding) -> Vec<&str> {
    match &f.location {
        Location::Controls { codes } => codes.iter().map(|c| c.0.as_str()).collect(),
        Location::Control { code } => vec![code.0.as_str()],
        other => panic!("not at controls: {other:?}"),
    }
}

use freemapper_courseplan_model::Course;

// ---------------------------------------------------------------------------------------------
// code_range

#[test]
fn codes_outside_the_range_are_errors_and_the_bounds_are_allowed() {
    let e = event_of(
        &["30", "31", "599", "600"],
        vec![course("A", &["30", "31", "599", "600"])],
    );
    let r = run(&e, vec![range_rule()]);
    let found: Vec<(&str, Severity)> = r
        .findings
        .iter()
        .map(|f| (codes_of(f)[0], f.severity))
        .collect();
    assert_eq!(found, [("30", Severity::Error), ("600", Severity::Error)]);
    assert_eq!(
        en(&r.findings[0]),
        "code 30 is outside the allowed range 31 to 599"
    );
    assert_eq!(
        de(&r.findings[1]),
        "Der Code 600 liegt außerhalb des erlaubten Bereichs 31 bis 599"
    );
}

#[test]
fn a_non_numeric_code_gets_the_non_numeric_severity() {
    let e = event_of(&["X1", "40"], vec![course("A", &["X1", "40"])]);
    let r = run(&e, vec![range_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(r.findings[0].severity, Severity::Warning);
    assert_eq!(codes_of(&r.findings[0]), ["X1"]);
    assert_eq!(en(&r.findings[0]), "code X1 is not a number");
    assert_eq!(de(&r.findings[0]), "Der Code X1 ist keine Zahl");
}

#[test]
fn crossing_points_and_unused_controls_have_no_code_to_check() {
    let mut e = event_of(&["5", "6", "40"], vec![course("A", &["5", "40"])]);
    crossing(&mut e, "5");
    let r = run(&e, vec![range_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_code_in_two_courses_is_reported_once() {
    let e = event_of(
        &["20", "40"],
        vec![course("A", &["20", "40"]), course("B", &["40", "20"])],
    );
    let r = run(&e, vec![range_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn courses_without_or_with_one_control_do_not_break_the_code_checks() {
    let e = event_of(&["20"], vec![course("A", &[]), course("B", &["20"])]);
    let r = run(
        &e,
        vec![range_rule(), pairs_rule(), confusable_rule(), same_rule()],
    );
    let ids: Vec<&str> = r.findings.iter().map(|f| f.rule.as_str()).collect();
    assert_eq!(ids, ["t.code.range"]);
}

#[test]
fn a_range_rule_without_its_value_is_skipped() {
    let e = event_of(&["20"], vec![course("A", &["20"])]);
    let r = run(
        &e,
        vec![rule("t.code.range", "code_range", Severity::Error)],
    );
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].reason_id, SKIP_VALUE_MISSING);
}

// ---------------------------------------------------------------------------------------------
// code_pairs

#[test]
fn both_codes_of_a_forbidden_pair_in_the_event_are_an_error_even_in_two_courses() {
    let e = event_of(
        &["66", "99", "40"],
        vec![course("A", &["66", "40"]), course("B", &["40", "99"])],
    );
    let r = run(&e, vec![pairs_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(f.location.place(), Place::Controls);
    assert_eq!(codes_of(f), ["66", "99"]);
    assert_eq!(en(f), "codes 66 and 99 must not both be used in one event");
    assert_eq!(
        de(f),
        "Die Codes 66 und 99 dürfen nicht beide in einer Veranstaltung vorkommen"
    );
}

#[test]
fn a_pair_in_both_courses_is_reported_once() {
    let e = event_of(
        &["68", "89"],
        vec![course("A", &["68", "89"]), course("B", &["89", "68"])],
    );
    let r = run(&e, vec![pairs_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn an_unused_control_or_a_crossing_point_does_not_count_for_a_pair() {
    let mut e = event_of(
        &["66", "99", "86", "98"],
        vec![course("A", &["66", "86", "98"])],
    );
    crossing(&mut e, "98");
    let r = run(&e, vec![pairs_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

// ---------------------------------------------------------------------------------------------
// code_confusable_next

#[test]
fn codes_that_read_alike_upside_down_are_a_warning() {
    for (a, b) in [("61", "19"), ("68", "89"), ("101", "101x")] {
        let e = event_of(&[a, b], vec![course("A", &[a, b])]);
        let r = run(&e, vec![confusable_rule()]);
        let expect = usize::from(b != "101x");
        assert_eq!(r.findings.len(), expect, "{a} {b}: {:?}", r.findings);
    }
    let e = event_of(&["61", "19"], vec![course("A", &["61", "19"])]);
    let r = run(&e, vec![confusable_rule()]);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location.place(), Place::Controls);
    assert_eq!(codes_of(f), ["19", "61"]);
    assert_eq!(
        en(f),
        "codes 19 and 61 look alike when turned upside down and follow each other"
    );
    assert_eq!(
        de(f),
        "Die Codes 19 und 61 sehen auf dem Kopf gleich aus und folgen aufeinander"
    );
}

#[test]
fn codes_that_differ_only_by_six_and_nine_are_a_warning() {
    let e = event_of(&["16", "19"], vec![course("A", &["16", "19"])]);
    let r = run(&e, vec![confusable_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(
        en(&r.findings[0]),
        "codes 16 and 19 differ only by 6 and 9 and follow each other"
    );
    assert_eq!(
        de(&r.findings[0]),
        "Die Codes 16 und 19 unterscheiden sich nur durch 6 und 9 und folgen aufeinander"
    );
}

#[test]
fn other_neighbours_are_fine() {
    // 31/32, a different length, a different digit besides 6 and 9, and 2 which cannot rotate.
    let e = event_of(
        &["31", "32", "316", "3169", "62", "26"],
        vec![
            course("A", &["31", "32"]),
            course("B", &["316", "3169"]),
            course("C", &["62", "26"]),
        ],
    );
    let r = run(&e, vec![confusable_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_repeated_control_is_no_pair_and_a_butterfly_reports_each_pair_once() {
    let e = event_of(
        &["31", "61", "19"],
        vec![course("A", &["31", "31", "61", "31", "19", "61"])],
    );
    let r = run(&e, vec![confusable_rule()]);
    let found: Vec<Vec<&str>> = r.findings.iter().map(|f| codes_of(f)).collect();
    assert_eq!(found, [["19", "61"]]);
}

#[test]
fn a_crossing_point_between_two_confusable_codes_does_not_break_the_pair() {
    let mut e = event_of(&["61", "40", "19"], vec![course("A", &["61", "40", "19"])]);
    crossing(&mut e, "40");
    let r = run(&e, vec![confusable_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn confusable_codes_in_two_courses_are_reported_once_and_not_across_courses() {
    let e = event_of(
        &["61", "19", "40"],
        vec![
            course("A", &["61", "19"]),
            course("B", &["19", "61"]),
            course("C", &["61", "40"]),
        ],
    );
    let r = run(&e, vec![confusable_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);

    let e = event_of(
        &["61", "19", "40"],
        vec![course("A", &["61", "40"]), course("B", &["40", "19"])],
    );
    let r = run(&e, vec![confusable_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn non_numeric_codes_are_not_tested() {
    let e = event_of(&["6A", "A9", "6a"], vec![course("A", &["6A", "A9", "6a"])]);
    let r = run(&e, vec![confusable_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

// ---------------------------------------------------------------------------------------------
// same_objects_in_circle

fn with_column_c(e: &mut Event, code: &str) {
    let c = e.controls.iter_mut().find(|c| c.code.0 == code).unwrap();
    c.description.c = Some("1.8".to_owned());
}

fn one_control_event() -> Event {
    // Control 31 at (10, 20).
    event_of(&["31"], vec![course("A", &["31"])])
}

#[test]
fn a_second_identical_point_object_in_the_circle_without_column_c_is_a_warning() {
    let e = one_control_event();
    let objects = vec![
        point_object(POINT_113, 10.0, 20.0),
        point_object(POINT_113, 12.0, 20.0),
    ];
    let r = report(&e, &map(4000, objects), vec![same_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(f.location.place(), Place::Control);
    assert_eq!(codes_of(f), ["31"]);
    assert_eq!(
        en(f),
        "control 31 stands at symbol 113, which occurs again within its circle, and column C of the description is empty"
    );
    assert_eq!(
        de(f),
        "Der Posten 31 steht an Symbol 113, das im Postenkreis noch einmal vorkommt, und Spalte C der Postenbeschreibung ist leer"
    );
}

#[test]
fn column_c_a_far_object_or_another_symbol_give_no_finding() {
    let objects = vec![
        point_object(POINT_113, 10.0, 20.0),
        point_object(POINT_113, 12.0, 20.0),
    ];
    let mut e = one_control_event();
    with_column_c(&mut e, "31");
    let r = report(&e, &map(4000, objects), vec![same_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);

    let e = one_control_event();
    let far = vec![
        point_object(POINT_113, 10.0, 20.0),
        point_object(POINT_113, 14.0, 20.0),
    ];
    let r = report(&e, &map(4000, far), vec![same_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);

    let other = vec![
        point_object(POINT_113, 10.0, 20.0),
        point_object(POINT_109, 12.0, 20.0),
    ];
    let r = report(&e, &map(4000, other), vec![same_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_control_on_a_line_feature_has_no_finding() {
    let e = one_control_event();
    let objects = vec![
        line_object(LINE_502, &[(5.0, 20.0), (15.0, 20.0)]),
        line_object(LINE_502, &[(5.0, 21.0), (15.0, 21.0)]),
        point_object(POINT_113, 12.0, 20.0),
        point_object(POINT_113, 8.0, 20.0),
    ];
    let r = report(&e, &map(4000, objects), vec![same_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn only_point_objects_count_as_the_same_object() {
    let e = one_control_event();
    let objects = vec![
        point_object(POINT_113, 10.0, 20.0),
        line_object(LINE_502, &[(11.0, 19.0), (11.0, 21.0)]),
    ];
    let r = report(&e, &map(4000, objects), vec![same_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn crossing_points_have_no_control_feature_and_a_control_is_reported_once() {
    let objects = vec![
        point_object(POINT_113, 10.0, 20.0),
        point_object(POINT_113, 12.0, 20.0),
    ];
    let mut e = event_of(&["31"], vec![course("A", &["31"]), course("B", &["31"])]);
    let r = report(&e, &map(4000, objects.clone()), vec![same_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    crossing(&mut e, "31");
    let r = report(&e, &map(4000, objects), vec![same_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn the_profile_rules_load_and_run() {
    let rules = crate::rules::load_rule_set("wkb-2026", &[]).unwrap();
    let e = event_of(&["30"], vec![course("A", &["30"])]);
    let r = check_event(&e, &map(4000, vec![]), &rules);
    assert!(r.findings.iter().any(|f| f.rule == "wkb2026.code.range"));
}
