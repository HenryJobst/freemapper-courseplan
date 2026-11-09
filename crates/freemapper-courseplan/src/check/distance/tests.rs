//! Tests of the distance checks. Maps are 1:4000 sprint maps unless named otherwise, so one map
//! mm is 4 m and the control circle radius is 3 mm (12 m).

use freemapper_courseplan_model::{ControlKind, Course, Event, SpecialObject};

use super::super::testing::*;
use super::super::*;
use super::symbol_matches;
use crate::omap::{MapDocument, MapObject};
use crate::rules::load_rule_set;
use crate::text::{Lang, parse_lang_arg};

fn lang_of(code: &str) -> Lang {
    parse_lang_arg(code).unwrap()
}

fn report(event: &Event, map: &MapDocument, rules: Vec<Rule>) -> CheckReport {
    let rules = rule_set(rules.into_iter().map(|r| (r, "base")).collect());
    run_checks(&CheckContext::new(event, map), &rules, CHECKS)
}

fn en(finding: &Finding) -> String {
    finding.text(&lang_of("en"))
}

fn de(finding: &Finding) -> String {
    finding.text(&lang_of("de"))
}

fn codes(location: &Location) -> Vec<&str> {
    match location {
        Location::Controls { codes } => codes.iter().map(|c| c.0.as_str()).collect(),
        Location::Control { code } => vec![code.0.as_str()],
        other => panic!("not at controls: {other:?}"),
    }
}

fn leg(course: &str, after: u32) -> Location {
    Location::Leg {
        course: course.to_owned(),
        after,
    }
}

fn min_rule() -> Rule {
    with_values(
        rule("t.distance.min", "min_distance", Severity::Error),
        "metres = 15.0",
    )
}

fn crossing(event: &mut Event, code: &str) {
    let control = event.controls.iter_mut().find(|c| c.code.0 == code);
    control.unwrap().kind = ControlKind::CrossingPoint;
}

fn taped_after(course: &mut Course, index: usize) {
    let CourseKind::Individual { controls } = &mut course.kind else {
        unreachable!()
    };
    controls[index].taped = true;
}

fn bend_after(course: &mut Course, index: usize, x: f64, y: f64) {
    let CourseKind::Individual { controls } = &mut course.kind else {
        unreachable!()
    };
    controls[index].bends.push(pt(x, y));
}

// ---------------------------------------------------------------------------------------------
// Symbol codes

#[test]
fn a_sub_code_matches_its_code() {
    assert!(symbol_matches("505", "505"));
    assert!(symbol_matches("505.1", "505"));
    assert!(symbol_matches("512.1", "512.1"));
    assert!(!symbol_matches("5051", "505"));
    assert!(!symbol_matches("505", "505.1"));
    assert!(!symbol_matches("50", "505"));
}

// ---------------------------------------------------------------------------------------------
// min_distance

#[test]
fn controls_closer_than_the_minimum_are_an_error() {
    let e = event(
        &[("31", 10.0, 10.0), ("32", 12.0, 10.0), ("33", 30.0, 10.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![min_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(codes(&f.location), ["31", "32"]);
    assert_eq!(
        en(f),
        "control 31 and control 32 are 8 m apart, at least 15 m"
    );
    assert_eq!(
        de(f),
        "Posten 31 und Posten 32 liegen 8 m auseinander, mindestens 15 m"
    );
    assert!(r.has_errors());
}

#[test]
fn the_start_and_crossing_points_count_the_finish_does_not() {
    // S1 at (0, 0), F1 at (100, 0).
    let mut e = event(
        &[
            ("31", 1.0, 0.0),
            ("32", 50.0, 0.0),
            ("33", 52.0, 0.0),
            ("34", 99.0, 0.0),
        ],
        vec![course("A", &["31", "32", "33", "34"])],
    );
    crossing(&mut e, "33");
    let r = report(&e, &map(4000, vec![]), vec![min_rule()]);
    let places: Vec<Vec<&str>> = r.findings.iter().map(|f| codes(&f.location)).collect();
    assert_eq!(places, [vec!["31", "S1"], vec!["32", "33"]]);
    assert_eq!(
        en(&r.findings[0]),
        "control 31 and start S1 are 4 m apart, at least 15 m"
    );
    assert_eq!(
        de(&r.findings[1]),
        "Posten 32 und Querungsstelle 33 liegen 8 m auseinander, mindestens 15 m"
    );
}

#[test]
fn a_pair_is_reported_once_and_a_control_is_never_paired_with_itself() {
    // Butterfly: 31 is visited twice; course B uses the same pair.
    let e = event(
        &[("31", 10.0, 10.0), ("32", 11.0, 10.0), ("33", 40.0, 40.0)],
        vec![
            course("A", &["31", "32", "33", "31"]),
            course("B", &["32", "31"]),
        ],
    );
    let r = report(&e, &map(4000, vec![]), vec![min_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(codes(&r.findings[0].location), ["31", "32"]);
}

#[test]
fn courses_without_controls_or_with_one_control_are_checked() {
    let e = event(
        &[("31", 2.0, 0.0), ("32", 3.0, 0.0)],
        vec![course("A", &[]), course("B", &["31"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![min_rule()]);
    // 32 belongs to no course, so it is not out in the terrain.
    let places: Vec<Vec<&str>> = r.findings.iter().map(|f| codes(&f.location)).collect();
    assert_eq!(places, [vec!["31", "S1"]]);
}

#[test]
fn a_control_of_a_course_at_another_print_scale_is_a_neighbour() {
    let mut b = course("B", &["35"]);
    b.planned_print_scale = Some(10000);
    let e = event(
        &[("31", 30.0, 30.0), ("35", 32.0, 30.0)],
        vec![course("A", &["31"]), b],
    );
    let mut half = min_rule();
    half.print_scales = vec![4000];
    let mut full = with_values(
        rule("t.distance.full", "min_distance", Severity::Error),
        "metres = 30.0",
    );
    full.print_scales = vec![10000];
    let r = report(&e, &map(4000, vec![]), vec![half, full]);
    let rules: Vec<&str> = r.findings.iter().map(|f| f.rule.as_str()).collect();
    assert_eq!(rules, ["t.distance.min", "t.distance.full"]);
}

#[test]
fn a_rule_without_its_value_is_skipped() {
    let e = event(&[("31", 1.0, 0.0)], vec![course("A", &["31"])]);
    let bare = rule("t.distance.min", "min_distance", Severity::Error);
    let r = report(&e, &map(4000, vec![]), vec![bare]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped.len(), 1);
    assert_eq!(r.skipped[0].reason_id, SKIP_VALUE_MISSING);
    assert_eq!(
        r.skipped[0].text(&lang_of("en")),
        "the rule has no value metres"
    );
}

#[test]
fn the_wkb_profile_halves_the_distance_on_sprint_maps() {
    let e = event(
        &[("31", 10.0, 10.0), ("32", 13.0, 10.0), ("33", 30.0, 10.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    let map = map(
        4000,
        vec![area_object(
            AREA_520,
            &[(-50.0, -50.0), (150.0, -50.0), (150.0, 60.0), (-50.0, 60.0)],
        )],
    );
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let r = run_checks(&CheckContext::new(&e, &map), &rules, CHECKS);
    let min: Vec<&Finding> = r
        .findings
        .iter()
        .filter(|f| f.rule.starts_with("wkb2026.distance.min"))
        .collect();
    assert_eq!(min.len(), 1, "{:?}", r.findings);
    assert_eq!(min[0].rule, "wkb2026.distance.min_half");
    assert_eq!(
        en(min[0]),
        "control 31 and control 32 are 12 m apart, at least 15 m"
    );
}

// ---------------------------------------------------------------------------------------------
// similar_features

fn similar_rule() -> Rule {
    with_values(
        rule("t.distance.similar", "similar_features", Severity::Warning),
        "metres = 30.0\nfeature_search_mm = 1.0\ngroups = [[\"109\", \"110\"], [\"505\", \"101\"]]",
    )
}

#[test]
fn controls_at_similar_features_closer_than_the_limit_are_a_warning() {
    let e = event(
        &[
            ("31", 10.0, 10.0),
            ("32", 15.0, 10.0),
            ("33", 30.0, 10.0),
            ("34", 50.0, 10.0),
        ],
        vec![course("A", &["31", "32", "33", "34"])],
    );
    let objects = vec![
        point_object(POINT_109, 10.0, 10.5),
        point_object(POINT_110, 15.0, 10.0),
        // 33 stands at a knoll too, but 15 mm (60 m) from 32.
        point_object(POINT_109, 30.0, 10.0),
    ];
    let r = report(&e, &map(4000, objects), vec![similar_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Warning);
    assert_eq!(codes(&f.location), ["31", "32"]);
    assert_eq!(
        en(f),
        "controls 31 and 32 stand at similar features (109, 110) and are 20 m apart, at least 30 m"
    );
    assert_eq!(
        de(f),
        "Posten 31 und 32 stehen an ähnlichen Objekten (109, 110) und liegen 20 m auseinander, mindestens 30 m"
    );
}

#[test]
fn identical_symbols_are_similar_and_sub_codes_match_their_group() {
    let e = event(
        &[
            ("31", 10.0, 10.0),
            ("32", 14.0, 10.0),
            ("41", 60.0, 10.0),
            ("42", 64.0, 10.0),
            ("51", 110.0, 10.0),
            ("52", 114.0, 10.0),
        ],
        vec![course("A", &["31", "32", "41", "42", "51", "52"])],
    );
    let objects = vec![
        // Two buildings, the controls at their corners.
        area_object(
            AREA_521,
            &[(5.0, 5.0), (10.0, 5.0), (10.0, 10.0), (5.0, 10.0)],
        ),
        area_object(
            AREA_521,
            &[(14.0, 10.0), (18.0, 10.0), (18.0, 14.0), (14.0, 14.0)],
        ),
        // 505.1 is a sub-code of 505, grouped with 101.
        line_object(LINE_505_1, &[(60.0, 0.0), (60.0, 20.0)]),
        point_object(POINT_101, 64.0, 10.0),
        // 5051 is not a sub-code of 505.
        line_object(LINE_5051, &[(110.0, 0.0), (110.0, 20.0)]),
        point_object(POINT_101, 114.0, 10.0),
    ];
    let r = report(&e, &map(4000, objects), vec![similar_rule()]);
    let places: Vec<Vec<&str>> = r.findings.iter().map(|f| codes(&f.location)).collect();
    assert_eq!(places, [vec!["31", "32"], vec!["41", "42"]]);
    assert!(en(&r.findings[0]).contains("(521, 521)"));
    assert!(en(&r.findings[1]).contains("(505.1, 101)"));
}

#[test]
fn a_control_without_feature_or_inside_a_large_area_gives_no_finding() {
    let e = event(
        &[("31", 10.0, 10.0), ("32", 12.0, 10.0), ("33", 14.0, 10.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    let objects = vec![
        // All three stand inside one building, far from its outline.
        area_object(
            AREA_521,
            &[(0.0, 0.0), (30.0, 0.0), (30.0, 30.0), (0.0, 30.0)],
        ),
        // A knoll 2 mm from 33, beyond the search distance.
        point_object(POINT_109, 14.0, 12.0),
    ];
    let r = report(&e, &map(4000, objects), vec![similar_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn crossing_points_have_no_control_feature() {
    let mut e = event(
        &[("31", 10.0, 10.0), ("32", 12.0, 10.0)],
        vec![course("A", &["31", "32"])],
    );
    crossing(&mut e, "32");
    let objects = vec![
        point_object(POINT_109, 10.0, 10.0),
        point_object(POINT_109, 12.0, 10.0),
    ];
    let r = report(&e, &map(4000, objects), vec![similar_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

// ---------------------------------------------------------------------------------------------
// map_edge

fn edge_rule() -> Rule {
    with_values(
        rule("t.map.edge", "map_edge", Severity::Warning),
        "metres = 10.0",
    )
}

/// A map whose extent is (-50, -50) to (150, 50).
fn framed(mut objects: Vec<MapObject>) -> MapDocument {
    objects.push(area_object(
        AREA_520,
        &[(-50.0, -50.0), (150.0, -50.0), (150.0, 50.0), (-50.0, 50.0)],
    ));
    map(4000, objects)
}

#[test]
fn controls_and_the_finish_near_the_map_edge_are_a_warning() {
    let mut e = event(
        &[("31", 148.0, 20.0), ("32", 100.0, 20.0), ("33", 10.0, 49.5)],
        vec![
            course("A", &["31", "32", "33"]),
            course("B", &["33", "31", "31"]),
        ],
    );
    if let SpecialObject::Finish { map_mm, .. } = &mut e.specials[1] {
        *map_mm = pt(149.0, 0.0);
    }
    let r = report(&e, &framed(vec![]), vec![edge_rule()]);
    let places: Vec<Vec<&str>> = r.findings.iter().map(|f| codes(&f.location)).collect();
    assert_eq!(
        places,
        [vec!["31"], vec!["33"], vec!["F1"]],
        "{:?}",
        r.findings
    );
    assert!(matches!(r.findings[0].location, Location::Control { .. }));
    assert_eq!(
        en(&r.findings[0]),
        "control 31 is 8 m from the map edge, at least 10 m"
    );
    assert_eq!(
        de(&r.findings[2]),
        "Ziel F1 liegt 4 m vom Kartenrand entfernt, mindestens 10 m"
    );
}

#[test]
fn a_control_outside_the_map_is_0_m_from_its_edge() {
    let e = event(&[("31", 200.0, 20.0)], vec![course("A", &["31"])]);
    let r = report(&e, &framed(vec![]), vec![edge_rule()]);
    assert_eq!(r.findings.len(), 1);
    assert!(
        en(&r.findings[0]).contains(" 0 m "),
        "{}",
        en(&r.findings[0])
    );
}

#[test]
fn a_map_without_features_has_no_edge_to_check() {
    let e = event(&[("31", 10.0, 10.0)], vec![course("A", &["31"])]);
    let r = report(&e, &map(4000, vec![]), vec![edge_rule()]);
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped[0].reason_id, SKIP_NO_MAP_EXTENT);
}

#[test]
fn sheet_objects_do_not_widen_the_map_edge() {
    let e = event(&[("31", 148.0, 20.0)], vec![course("A", &["31"])]);
    let sheet_frame = sheet(area_object(
        AREA_520,
        &[(-90.0, -90.0), (190.0, -90.0), (190.0, 90.0), (-90.0, 90.0)],
    ));
    let r = report(&e, &framed(vec![sheet_frame]), vec![edge_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

// ---------------------------------------------------------------------------------------------
// leg_through_control

fn through_rule() -> Rule {
    rule("t.leg.through", "leg_through_control", Severity::Warning)
}

#[test]
fn a_leg_through_another_control_of_the_course_is_a_warning() {
    let e = event(
        &[("31", 10.0, 30.0), ("32", 50.0, 30.0), ("33", 30.0, 31.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![through_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.location, leg("A", 1));
    assert_eq!(
        en(f),
        "the leg passes 4 m from control 33, inside its circle radius of 12 m"
    );
    assert_eq!(
        de(f),
        "Die Strecke führt 4 m am Posten 33 vorbei, innerhalb des Kreisradius von 12 m"
    );
}

#[test]
fn a_bend_around_the_control_or_a_control_of_another_course_gives_no_finding() {
    let mut a = course("A", &["31", "32", "33"]);
    bend_after(&mut a, 0, 30.0, 45.0);
    // 34 of course B lies on A's leg from 33 to the finish at (100, 0).
    let e = event(
        &[
            ("31", 10.0, 30.0),
            ("32", 50.0, 30.0),
            ("33", 30.0, 31.0),
            ("34", 65.0, 15.5),
        ],
        vec![a, course("B", &["34"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![through_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn the_leg_from_the_start_is_checked() {
    // S1 at (0, 0); 31 lies on the leg from the start to 32. The bend keeps the last leg from
    // 31 to the finish at (100, 0) away from 32.
    let mut a = course("A", &["32", "31"]);
    bend_after(&mut a, 1, 20.0, 20.0);
    let e = event(&[("31", 20.0, 0.5), ("32", 40.0, 0.0)], vec![a]);
    let r = report(&e, &map(4000, vec![]), vec![through_rule()]);
    let places: Vec<&Location> = r.findings.iter().map(|f| &f.location).collect();
    assert_eq!(places, [&leg("A", 0)], "{:?}", r.findings);
    assert_eq!(
        leg("A", 0).text(&lang_of("en")),
        "course A, leg from the start"
    );
}

#[test]
fn a_crossing_point_on_a_leg_is_named_as_such() {
    let mut e = event(
        &[("31", 10.0, 30.0), ("32", 50.0, 30.0), ("35", 30.0, 30.0)],
        vec![course("A", &["31", "32", "35"])],
    );
    crossing(&mut e, "35");
    let r = report(&e, &map(4000, vec![]), vec![through_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert!(
        en(&r.findings[0]).contains("0 m from crossing point 35"),
        "{}",
        en(&r.findings[0])
    );
}

#[test]
fn a_butterfly_centre_does_not_block_its_own_legs() {
    // 31 is the centre, visited three times; the loops leave and reach it.
    let e = event(
        &[
            ("31", 50.0, 50.0),
            ("32", 70.0, 60.0),
            ("33", 70.0, 40.0),
            ("34", 30.0, 60.0),
            ("35", 30.0, 40.0),
        ],
        vec![course("A", &["31", "32", "33", "31", "34", "35", "31"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![through_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_short_leg_inside_the_circles_is_not_drawn() {
    let e = event(
        &[("31", 50.0, 50.0), ("32", 54.0, 50.0), ("33", 52.0, 50.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    let r = report(&e, &map(4000, vec![]), vec![through_rule()]);
    // 31 to 32 is 4 mm, less than two radii; nothing of it is drawn.
    assert!(
        !r.findings.iter().any(|f| f.location == leg("A", 1)),
        "{:?}",
        r.findings
    );
}

// ---------------------------------------------------------------------------------------------
// leg_through_forbidden

fn forbidden_rule() -> Rule {
    with_values(
        rule("t.leg.forbidden", "leg_through_forbidden", Severity::Error),
        "impassable = [\"520\", \"518\", \"505\"]",
    )
}

/// An area 520 across the leg from (10, 30) to (50, 30).
fn blocked() -> MapObject {
    area_object(
        AREA_520,
        &[(28.0, 20.0), (32.0, 20.0), (32.0, 40.0), (28.0, 40.0)],
    )
}

fn two_controls(courses: Vec<Course>) -> Event {
    event(&[("31", 10.0, 30.0), ("32", 50.0, 30.0)], courses)
}

#[test]
fn a_leg_through_an_impassable_feature_is_an_error() {
    let e = two_controls(vec![course("A", &["31", "32"])]);
    let r = report(&e, &map(4000, vec![blocked()]), vec![forbidden_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(f.location, leg("A", 1));
    assert_eq!(
        en(f),
        "the leg crosses map symbol 520, which must not be crossed"
    );
    assert_eq!(
        de(f),
        "Die Strecke quert das Kartensymbol 520, das nicht überquert werden darf"
    );
}

#[test]
fn a_taped_leg_or_a_bend_around_the_feature_gives_no_finding() {
    let mut taped = course("A", &["31", "32"]);
    taped_after(&mut taped, 0);
    let mut around = course("B", &["31", "32"]);
    bend_after(&mut around, 0, 30.0, 45.0);
    let e = two_controls(vec![taped, around]);
    let r = report(&e, &map(4000, vec![blocked()]), vec![forbidden_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn sheet_objects_never_trigger_a_finding() {
    // Objects as in the tests above, tagged as sheet content: a forbidden area across a leg, a
    // knoll at two controls, and the map edge they would make.
    let e = event(
        &[("31", 10.0, 30.0), ("32", 12.0, 30.0), ("33", 50.0, 30.0)],
        vec![course("A", &["31", "32", "33"])],
    );
    let objects = vec![
        area_object(
            AREA_520,
            &[(28.0, 20.0), (32.0, 20.0), (32.0, 40.0), (28.0, 40.0)],
        ),
        point_object(POINT_109, 10.0, 30.0),
        point_object(POINT_110, 12.0, 30.0),
    ];
    let rules = || vec![forbidden_rule(), similar_rule(), edge_rule()];
    let tagged = map(4000, objects.iter().cloned().map(sheet).collect());
    let r = report(&e, &tagged, rules());
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    // Untagged, the same objects give findings of all three rules.
    let r = report(&e, &map(4000, objects), rules());
    let mut keys: Vec<&str> = r.findings.iter().map(|f| f.rule.as_str()).collect();
    keys.dedup();
    assert_eq!(
        keys,
        ["t.leg.forbidden", "t.distance.similar", "t.map.edge"],
        "{:?}",
        r.findings
    );
}

#[test]
fn a_crossing_point_lets_a_leg_through_a_fence() {
    let fence = line_object(LINE_518, &[(30.0, 0.0), (30.0, 60.0)]);
    let mut e = event(
        &[("31", 10.0, 30.0), ("32", 50.0, 30.0), ("35", 30.0, 30.0)],
        vec![course("A", &["31", "35", "32"]), course("B", &["31", "32"])],
    );
    crossing(&mut e, "35");
    let r = report(&e, &map(4000, vec![fence]), vec![forbidden_rule()]);
    let places: Vec<&Location> = r.findings.iter().map(|f| &f.location).collect();
    assert_eq!(places, [&leg("B", 1)], "{:?}", r.findings);
    assert!(en(&r.findings[0]).contains("518"));
}

#[test]
fn sub_codes_of_impassable_symbols_block_a_leg() {
    let e = two_controls(vec![course("A", &["31", "32"])]);
    let across = [(30.0, 25.0), (30.0, 35.0)];
    let r = report(
        &e,
        &map(4000, vec![line_object(LINE_505_1, &across)]),
        vec![forbidden_rule()],
    );
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert!(en(&r.findings[0]).contains("505.1"));
    // 5051 is not a sub-code of 505.
    let r = report(
        &e,
        &map(4000, vec![line_object(LINE_5051, &across)]),
        vec![forbidden_rule()],
    );
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn out_of_bounds_areas_and_forbidden_routes_apply_to_their_courses() {
    let mut e = two_controls(vec![
        course("A", &["31", "32"]),
        course("B", &["31", "32"]),
        course("C", &["31", "32"]),
    ]);
    e.specials.push(SpecialObject::OutOfBoundsArea {
        id: "oob".to_owned(),
        polygon: vec![
            pt(28.0, 20.0),
            pt(32.0, 20.0),
            pt(32.0, 40.0),
            pt(28.0, 40.0),
        ],
        courses: vec!["B".to_owned()],
    });
    e.specials.push(SpecialObject::ForbiddenRoute {
        id: "road".to_owned(),
        points: vec![pt(40.0, 0.0), pt(40.0, 60.0)],
        courses: vec!["C".to_owned()],
    });
    let r = report(&e, &map(4000, vec![]), vec![forbidden_rule()]);
    let found: Vec<(&Location, String)> = r.findings.iter().map(|f| (&f.location, en(f))).collect();
    assert_eq!(
        found,
        [
            (
                &leg("B", 1),
                "the leg crosses out-of-bounds area oob".to_owned()
            ),
            (
                &leg("C", 1),
                "the leg crosses forbidden route road".to_owned()
            ),
        ]
    );
    assert_eq!(de(&r.findings[0]), "Die Strecke quert das Sperrgebiet oob");
    assert_eq!(
        de(&r.findings[1]),
        "Die Strecke quert die verbotene Route road"
    );

    // A special without courses applies to all of them.
    for special in &mut e.specials {
        if let SpecialObject::OutOfBoundsArea { courses, .. } = special {
            courses.clear();
        }
    }
    let r = report(&e, &map(4000, vec![]), vec![forbidden_rule()]);
    assert_eq!(r.findings.len(), 3, "{:?}", r.findings);
}

#[test]
fn routes_that_may_be_crossed_never_block_a_leg() {
    let mut e = two_controls(vec![course("A", &["31", "32"])]);
    e.specials.push(SpecialObject::ForbiddenRoute {
        id: "road".to_owned(),
        points: vec![pt(40.0, 0.0), pt(40.0, 60.0)],
        courses: vec![],
    });
    let rule = with_values(
        rule("t.leg.forbidden", "leg_through_forbidden", Severity::Error),
        "impassable = [\"520\", \"711\"]\ncrossing_allowed = [\"711\"]",
    );
    let route = line_object(FOREST_LINE_711, &[(20.0, 0.0), (20.0, 60.0)]);
    let r = report(&e, &forest_map(10000, vec![route]), vec![rule]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn the_leg_from_the_start_of_a_one_control_course_is_checked() {
    // S1 at (0, 0) to 31.
    let e = event(
        &[("31", 40.0, 0.0)],
        vec![course("A", &["31"]), course("B", &[])],
    );
    let wall = line_object(LINE_518, &[(20.0, -10.0), (20.0, 10.0)]);
    let r = report(&e, &map(4000, vec![wall]), vec![forbidden_rule()]);
    let places: Vec<&Location> = r.findings.iter().map(|f| &f.location).collect();
    // B runs from S1 straight to F1 at (100, 0), through the wall as well.
    assert_eq!(places, [&leg("A", 0), &leg("B", 0)], "{:?}", r.findings);
}
