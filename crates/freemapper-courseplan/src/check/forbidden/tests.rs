//! Tests of `control_in_forbidden`. Maps are 1:4000 sprint maps unless named otherwise.

use freemapper_courseplan_model::{
    AcceptedFinding, CourseKind, Event, ScoreControl, SpecialObject,
};

use super::super::testing::*;
use super::super::*;
use crate::omap::MapDocument;
use crate::rules::load_rule_set;

fn forbidden_rule() -> Rule {
    with_values(
        rule(
            "t.control.forbidden",
            "control_in_forbidden",
            Severity::Error,
        ),
        r#"area_symbols = ["520", "709", "714"]"#,
    )
}

fn out_of_bounds(id: &str, corners: &[(f64, f64)], courses: &[&str]) -> SpecialObject {
    SpecialObject::OutOfBoundsArea {
        id: id.to_owned(),
        ppen_id: None,
        polygon: corners.iter().map(|&(x, y)| pt(x, y)).collect(),
        courses: courses.iter().map(|c| (*c).to_owned()).collect(),
    }
}

const BOX: [(f64, f64); 4] = [(40.0, 20.0), (60.0, 20.0), (60.0, 40.0), (40.0, 40.0)];

/// Controls 31 at (10, 30) and 32 at (50, 30); 32 lies inside `BOX`. Course A runs over both.
fn event_with_box(courses: &[&str]) -> Event {
    let mut e = event(
        &[("31", 10.0, 30.0), ("32", 50.0, 30.0)],
        vec![course("A", &["31", "32"]), course("B", &["31"])],
    );
    e.specials.push(out_of_bounds("oob", &BOX, courses));
    e
}

fn run(e: &Event, m: &MapDocument) -> CheckReport {
    report(e, m, vec![forbidden_rule()])
}

fn control_of(f: &Finding) -> &str {
    match &f.location {
        Location::Control { code } => &code.0,
        other => panic!("not at a control: {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// Special objects

#[test]
fn a_control_inside_an_out_of_bounds_special_is_an_error() {
    let r = run(&event_with_box(&[]), &map(4000, vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(control_of(f), "32");
    assert_eq!(
        en(f),
        "control 32 stands inside the out-of-bounds area oob, at 50, 30 mm on the map"
    );
    assert_eq!(
        de(f),
        "Posten 32 steht im Sperrgebiet oob, bei 50, 30 mm auf der Karte"
    );
    assert!(r.has_errors());
}

#[test]
fn a_control_outside_gives_no_finding() {
    let mut e = event_with_box(&[]);
    e.controls[1].map_mm = pt(70.0, 30.0);
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_control_on_the_edge_counts_as_inside() {
    for (x, y) in [(40.0, 30.0), (60.0, 30.0), (50.0, 20.0), (40.0, 40.0)] {
        let mut e = event_with_box(&[]);
        e.controls[1].map_mm = pt(x, y);
        let r = run(&e, &map(4000, vec![]));
        assert_eq!(r.findings.len(), 1, "({x}, {y}): {:?}", r.findings);
    }
}

#[test]
fn a_special_limited_to_other_courses_does_not_hit_the_control() {
    // 32 is on course A only; the area belongs to course B.
    let r = run(&event_with_box(&["B"]), &map(4000, vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_special_limited_to_a_course_of_the_control_hits_it() {
    let r = run(&event_with_box(&["A"]), &map(4000, vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    // A control used by two courses is hit when one of them is meant.
    let mut e = event_with_box(&["B"]);
    e.controls[0].map_mm = pt(50.0, 30.0);
    let r = run(&e, &map(4000, vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(control_of(&r.findings[0]), "31");
}

#[test]
fn a_control_of_no_course_is_not_reported() {
    let mut e = event_with_box(&[]);
    e.courses[0] = course("A", &["31"]);
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_control_used_by_two_courses_is_reported_once() {
    let mut e = event_with_box(&[]);
    e.controls[0].map_mm = pt(50.0, 30.0);
    // 31 is on A and B, 32 only on A.
    let r = run(&e, &map(4000, vec![]));
    let mut hit: Vec<&str> = r.findings.iter().map(control_of).collect();
    hit.sort_unstable();
    assert_eq!(hit, ["31", "32"]);
}

#[test]
fn other_special_objects_do_not_count() {
    let mut e = event_with_box(&[]);
    e.specials[2] = SpecialObject::DangerousArea {
        id: "danger".to_owned(),
        ppen_id: None,
        polygon: BOX.iter().map(|&(x, y)| pt(x, y)).collect(),
        courses: vec![],
    };
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn start_finish_and_crossing_points_are_not_controls() {
    let mut e = event_with_box(&[]);
    e.controls[1].map_mm = pt(70.0, 30.0);
    // The start at (0, 0) and the finish at (100, 0) lie inside a big area.
    e.specials[2] = out_of_bounds(
        "big",
        &[(-5.0, -5.0), (105.0, -5.0), (105.0, 5.0), (-5.0, 5.0)],
        &[],
    );
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    // A crossing point in the box.
    let mut e = event_with_box(&[]);
    crossing(&mut e, "32");
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_score_control_inside_is_reported() {
    let mut e = event_with_box(&[]);
    let mut score = course("S", &[]);
    score.kind = CourseKind::Score {
        time_limit_min: 30,
        penalty_per_min: Some(10),
        controls: vec![ScoreControl {
            code: code("32"),
            value: 2,
        }],
        links: vec![],
    };
    e.courses = vec![score];
    let r = run(&e, &map(4000, vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(control_of(&r.findings[0]), "32");
}

// ---------------------------------------------------------------------------------------------
// Map areas

fn event_in_map_area(symbol: u32, x: f64) -> (Event, MapDocument) {
    let e = event(
        &[("31", 10.0, 30.0), ("32", x, 30.0)],
        vec![course("A", &["31", "32"])],
    );
    (e, map(4000, vec![area_object(symbol, &BOX)]))
}

#[test]
fn a_control_inside_a_map_area_with_an_out_of_bounds_symbol_is_an_error() {
    for symbol in [AREA_520, AREA_709, AREA_714] {
        let (e, m) = event_in_map_area(symbol, 50.0);
        let r = run(&e, &m);
        assert_eq!(r.findings.len(), 1, "{symbol}: {:?}", r.findings);
        assert_eq!(control_of(&r.findings[0]), "32");
    }
    let (e, m) = event_in_map_area(AREA_709, 50.0);
    let f = &run(&e, &m).findings[0];
    assert_eq!(
        en(f),
        "control 32 stands inside a map area with the out-of-bounds symbol 709, at 50, 30 mm on the map"
    );
    assert_eq!(
        de(f),
        "Posten 32 steht in einer Kartenfläche mit dem Sperrgebiet-Symbol 709, bei 50, 30 mm auf der Karte"
    );
}

#[test]
fn a_control_outside_the_map_area_or_in_its_hole_gives_no_finding() {
    let (e, m) = event_in_map_area(AREA_709, 70.0);
    assert!(run(&e, &m).findings.is_empty());
    let e = event(&[("31", 50.0, 30.0)], vec![course("A", &["31"])]);
    let holed = area_with_hole(
        AREA_709,
        &[(30.0, 10.0), (70.0, 10.0), (70.0, 50.0), (30.0, 50.0)],
        &[(45.0, 25.0), (55.0, 25.0), (55.0, 35.0), (45.0, 35.0)],
    );
    let r = run(&e, &map(4000, vec![holed]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_control_on_the_ring_of_a_hole_is_inside() {
    let e = event(&[("31", 45.0, 30.0)], vec![course("A", &["31"])]);
    let holed = area_with_hole(
        AREA_709,
        &[(30.0, 10.0), (70.0, 10.0), (70.0, 50.0), (30.0, 50.0)],
        &[(45.0, 25.0), (55.0, 25.0), (55.0, 35.0), (45.0, 35.0)],
    );
    let r = run(&e, &map(4000, vec![holed]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn the_boundary_line_708_and_other_symbols_are_not_tested() {
    let (e, _) = event_in_map_area(AREA_709, 50.0);
    // A 708 line right through the control, and a 521 building area around it.
    let line = line_object(LINE_708, &[(50.0, 20.0), (50.0, 40.0)]);
    let building = area_object(AREA_521, &BOX);
    let r = run(&e, &map(4000, vec![line, building]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn sheet_objects_are_not_tested() {
    let (e, _) = event_in_map_area(AREA_709, 50.0);
    let r = run(&e, &map(4000, vec![sheet(area_object(AREA_709, &BOX))]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_control_in_a_special_and_a_map_area_gets_one_finding_naming_the_special() {
    let (mut e, m) = event_in_map_area(AREA_709, 50.0);
    e.specials.push(out_of_bounds("oob", &BOX, &[]));
    let r = run(&e, &m);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert!(
        en(&r.findings[0]).contains("area oob"),
        "{}",
        en(&r.findings[0])
    );
}

#[test]
fn a_map_area_hits_a_control_whatever_the_course() {
    // The map area has no course; the control is on course B only.
    let mut e = event(
        &[("31", 10.0, 30.0), ("32", 50.0, 30.0)],
        vec![course("A", &["31"]), course("B", &["32"])],
    );
    e.specials.push(out_of_bounds(
        "other",
        &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
        &["A"],
    ));
    let m = map(4000, vec![area_object(AREA_709, &BOX)]);
    let r = run(&e, &m);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn a_rule_without_symbols_checks_the_specials_only() {
    let bare = rule(
        "t.control.forbidden",
        "control_in_forbidden",
        Severity::Error,
    );
    let (e, m) = event_in_map_area(AREA_709, 50.0);
    let r = report(&e, &m, vec![bare.clone()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    let r = report(&event_with_box(&[]), &map(4000, vec![]), vec![bare]);
    assert_eq!(r.findings.len(), 1);
}

// ---------------------------------------------------------------------------------------------
// The profile

/// Symbol id of the 520 area in the forest test map.
const FOREST_AREA_520: u32 = 1;

fn profile_report(e: &Event, m: &MapDocument) -> CheckReport {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    run_checks(&CheckContext::new(e, m), &rules, CHECKS)
}

#[test]
fn the_profile_reports_it_on_sprint_maps_as_an_error_with_its_source() {
    let (e, m) = event_in_map_area(AREA_714, 50.0);
    let r = profile_report(&e, &m);
    let f = r
        .findings
        .iter()
        .find(|f| f.rule == "wkb2026.control.in_forbidden_sprint")
        .unwrap_or_else(|| panic!("{:?}", r.findings));
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(
        (f.source.doc.as_str(), f.source.section.as_str()),
        ("WKB26", "A 2.2")
    );
}

#[test]
fn the_profile_reports_it_on_forest_maps_for_520_and_709() {
    let e = event(
        &[("31", 100.0, 100.0), ("32", 500.0, 300.0)],
        vec![course("A", &["31", "32"])],
    );
    let m = forest_map(
        10000,
        vec![area_object(
            FOREST_AREA_520,
            &[
                (450.0, 250.0),
                (550.0, 250.0),
                (550.0, 350.0),
                (450.0, 350.0),
            ],
        )],
    );
    let r = profile_report(&e, &m);
    let f: Vec<_> = r
        .findings
        .iter()
        .filter(|f| f.rule.starts_with("wkb2026.control.in_forbidden"))
        .collect();
    assert_eq!(f.len(), 1, "{:?}", r.findings);
    assert_eq!(f[0].rule, "wkb2026.control.in_forbidden");
    assert_eq!(control_of(f[0]), "32");
}

#[test]
fn an_acceptance_of_the_finding_matches() {
    let mut e = event_with_box(&[]);
    e.accepted_findings.push(AcceptedFinding {
        rule: "t.control.forbidden".to_owned(),
        controls: vec![code("32")],
        course: None,
        legs: vec![],
        note: "the area was lifted for the event".to_owned(),
    });
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty());
    assert_eq!(r.accepted.len(), 1);
}

#[test]
fn the_check_reports_at_controls() {
    assert_eq!(
        check_of("control_in_forbidden").unwrap().places,
        &[Place::Control]
    );
}
