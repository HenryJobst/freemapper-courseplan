//! Tests of `inside_map`. Maps are 1:4000 sprint maps; 1 mm on the map is 4 m on the ground.

use freemapper_courseplan_model::{CourseKind, Event, ScoreControl, SpecialObject};

use super::super::testing::*;
use super::super::*;
use crate::omap::{MapDocument, MapObject};
use crate::rules::load_rule_set;

fn inside_rule() -> Rule {
    with_values(
        rule("t.map.inside", "inside_map", Severity::Error),
        r#"metres = 40.0
frame_percent = 80.0
ignore_symbols = ["601", "602"]"#,
    )
}

fn run(e: &Event, m: &MapDocument) -> CheckReport {
    report(e, m, vec![inside_rule()])
}

fn ring(x0: f64, y0: f64, x1: f64, y1: f64) -> [(f64, f64); 4] {
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// A diagonal strip of buildings from the top left to the bottom right, 20 mm wide, and a
/// pair of small objects in the corners that widen the extent to (0, 0) to (200, 200). The
/// bounding box of the map objects is the whole 200 by 200 mm square, but the top right and
/// the bottom left corner of it are blank.
fn strip_map(mut extra: Vec<MapObject>) -> MapDocument {
    let mut objects = vec![area_object(
        AREA_521,
        &[
            (0.0, 0.0),
            (30.0, 0.0),
            (200.0, 170.0),
            (200.0, 200.0),
            (170.0, 200.0),
            (0.0, 30.0),
        ],
    )];
    objects.append(&mut extra);
    map(4000, objects)
}

fn with_start_finish(mut e: Event, start: (f64, f64), finish: (f64, f64)) -> Event {
    for special in &mut e.specials {
        match special {
            SpecialObject::Start { map_mm, .. } => *map_mm = pt(start.0, start.1),
            SpecialObject::Finish { map_mm, .. } => *map_mm = pt(finish.0, finish.1),
            _ => {}
        }
    }
    e
}

fn place_of(f: &Finding) -> &str {
    match &f.location {
        Location::Control { code } => &code.0,
        other => panic!("not at a control: {other:?}"),
    }
}

#[test]
fn a_finish_in_the_blank_corner_of_the_extent_is_an_error() {
    // (180, 20) lies in the bounding box of the map but 92 mm from the strip.
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (180.0, 20.0),
    );
    let r = run(&e, &strip_map(vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Error);
    assert_eq!(place_of(f), "F1");
    assert_eq!(f.rule, "t.map.inside");
    assert_eq!(
        en(f),
        "finish F1 at 180, 20 mm on the map is outside the mapped area: the nearest map object, symbol 521, is 367 m away, at most 40 m allowed"
    );
    assert_eq!(
        de(f),
        "Ziel F1 liegt bei 180, 20 mm auf der Karte außerhalb der kartierten Fläche: das nächste Kartenobjekt, Symbol 521, ist 367 m entfernt, erlaubt sind höchstens 40 m"
    );
    assert!(r.has_errors());
}

#[test]
fn start_and_controls_in_the_blank_part_are_errors_too() {
    let e = with_start_finish(
        event(
            &[("31", 50.0, 50.0), ("32", 20.0, 180.0)],
            vec![course("A", &["31", "32"])],
        ),
        (190.0, 10.0),
        (100.0, 100.0),
    );
    let r = run(&e, &strip_map(vec![]));
    let found: Vec<(&str, String)> = r.findings.iter().map(|f| (place_of(f), en(f))).collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(found[0].0, "S1");
    assert!(
        found[0].1.starts_with("start S1 at 190, 10 mm"),
        "{found:?}"
    );
    assert_eq!(found[1].0, "32");
    assert!(
        found[1].1.starts_with("control 32 at 20, 180 mm"),
        "{found:?}"
    );
}

#[test]
fn positions_on_the_mapped_area_give_no_finding() {
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (190.0, 190.0),
    );
    let r = run(&e, &strip_map(vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn a_position_within_the_distance_of_an_object_is_inside() {
    // 5 mm (20 m) beside the strip, below the 40 m limit; 17 mm (68 m) is not.
    let near = event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]);
    let near = with_start_finish(near, (10.0, 10.0), (60.0, 97.07));
    assert!(run(&near, &strip_map(vec![])).findings.is_empty());
    let far = event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]);
    let far = with_start_finish(far, (10.0, 10.0), (60.0, 114.05));
    assert_eq!(run(&far, &strip_map(vec![])).findings.len(), 1);
}

#[test]
fn a_gap_inside_the_mapped_area_is_inside() {
    // A hole of 15 mm (60 m) in a large area, such as a square between buildings: its middle is 30 m
    // from the nearest edge, below the 40 m limit.
    let big = area_with_hole(
        AREA_521,
        &ring(0.0, 0.0, 200.0, 200.0),
        &ring(92.5, 92.5, 107.5, 107.5),
    );
    let e = with_start_finish(
        event(&[("31", 100.0, 100.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (190.0, 190.0),
    );
    let r = run(&e, &map(4000, vec![big]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn the_north_lines_area_is_not_map_content() {
    // The north lines pattern (601.6) covers a hexagon larger than the terrain; start and finish on it
    // only are still outside.
    let hexagon = area_object(AREA_601_6, &ring(0.0, 0.0, 200.0, 200.0));
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (180.0, 20.0),
    );
    let r = run(&e, &strip_map(vec![hexagon]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(place_of(&r.findings[0]), "F1");
}

#[test]
fn a_frame_line_around_the_sheet_is_not_map_content() {
    // A rectangle of line 502 along the extent: a control next to it in the blank corner is outside.
    let mut frame_points = ring(0.0, 0.0, 200.0, 200.0).to_vec();
    frame_points.push((0.0, 0.0));
    let frame = line_object(LINE_502, &frame_points);
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (199.0, 1.0),
    );
    let r = run(&e, &strip_map(vec![frame]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(place_of(&r.findings[0]), "F1");
}

#[test]
fn the_frame_is_found_although_the_north_lines_area_reaches_beyond_the_sheet() {
    // As on the Madella maps: the 601.6 area is larger than the sheet frame, so the frame spans
    // only part of the extent of all objects, but all of the extent of the counted ones.
    let hexagon = area_object(AREA_601_6, &ring(-30.0, -30.0, 230.0, 230.0));
    let mut frame_points = ring(0.0, 0.0, 200.0, 200.0).to_vec();
    frame_points.push((0.0, 0.0));
    let frame = line_object(LINE_502, &frame_points);
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (199.0, 1.0),
    );
    let r = run(&e, &strip_map(vec![hexagon, frame]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(place_of(&r.findings[0]), "F1");
}

#[test]
fn a_stray_object_far_outside_does_not_hide_the_frame() {
    // The frame spans 200 of 235 mm: 85 percent, so 90 percent would miss it.
    let stray = area_object(AREA_520, &ring(100.0, 233.0, 101.0, 235.0));
    let mut frame_points = ring(0.0, 0.0, 200.0, 200.0).to_vec();
    frame_points.push((0.0, 0.0));
    let frame = line_object(LINE_502, &frame_points);
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (199.0, 1.0),
    );
    let r = run(&e, &strip_map(vec![stray, frame]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert_eq!(place_of(&r.findings[0]), "F1");
}

#[test]
fn a_line_that_is_no_rectangle_is_no_frame() {
    // A long road from corner to corner of the extent is terrain.
    let road = line_object(LINE_502, &[(200.0, 0.0), (100.0, 100.0), (0.0, 200.0)]);
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (150.0, 50.0),
    );
    let r = run(&e, &strip_map(vec![road]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn the_constants_of_the_score_draft_match_the_rule() {
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let rule = rules
        .rules
        .iter()
        .find(|r| r.rule.id == "wkb2026.map.inside")
        .expect("the rule is in the profile");
    let values = &rule.rule.values;
    assert_eq!(
        values["frame_percent"].as_float(),
        Some(super::FRAME_PERCENT)
    );
    let ignored: Vec<&str> = values["ignore_symbols"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(ignored, super::IGNORED_SYMBOLS);
}

#[test]
fn a_line_that_covers_only_part_of_the_extent_stays_content() {
    let road = line_object(LINE_502, &[(120.0, 20.0), (180.0, 20.0)]);
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (180.0, 20.0),
    );
    let r = run(&e, &strip_map(vec![road]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

#[test]
fn sheet_objects_do_not_count_as_map_content() {
    let base = furniture(area_object(AREA_520, &ring(0.0, 0.0, 200.0, 200.0)), "base");
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (180.0, 20.0),
    );
    let r = run(&e, &strip_map(vec![base]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn a_crossing_point_in_the_blank_part_is_named_as_such() {
    let mut e = with_start_finish(
        event(&[("35", 180.0, 20.0)], vec![course("A", &["35"])]),
        (10.0, 10.0),
        (190.0, 190.0),
    );
    crossing(&mut e, "35");
    let r = run(&e, &strip_map(vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    assert!(en(&r.findings[0]).starts_with("crossing point 35 at"));
}

#[test]
fn a_shared_control_is_reported_once() {
    let e = with_start_finish(
        event(
            &[("31", 180.0, 20.0)],
            vec![course("A", &["31"]), course("B", &["31"])],
        ),
        (10.0, 10.0),
        (190.0, 190.0),
    );
    let r = run(&e, &strip_map(vec![]));
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn a_map_without_features_is_skipped() {
    let e = event(&[("31", 10.0, 10.0)], vec![course("A", &["31"])]);
    let r = run(&e, &map(4000, vec![]));
    assert!(r.findings.is_empty());
    assert_eq!(r.skipped[0].reason_id, SKIP_NO_MAP_EXTENT);
}

#[test]
fn score_courses_are_tested_with_start_and_finish() {
    let mut e = event(&[], vec![course("A", &[])]);
    e.courses.clear();
    let mut score = course("Score", &[]);
    score.kind = CourseKind::Score {
        time_limit_min: 15,
        penalty_per_min: Some(10),
        controls: vec![
            ScoreControl {
                code: code("31"),
                value: 3,
            },
            ScoreControl {
                code: code("32"),
                value: 2,
            },
        ],
        links: vec![],
    };
    e.courses.push(score);
    e.controls = event(&[("31", 50.0, 50.0), ("32", 190.0, 10.0)], vec![]).controls;
    let e = with_start_finish(e, (10.0, 10.0), (180.0, 20.0));
    let r = run(&e, &strip_map(vec![]));
    let places: Vec<&str> = r.findings.iter().map(place_of).collect();
    assert_eq!(places, ["32", "F1"], "{:?}", r.findings);
}

#[test]
fn an_acceptance_of_the_finding_matches() {
    let mut e = with_start_finish(
        event(&[("31", 180.0, 20.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (190.0, 190.0),
    );
    e.accepted_findings
        .push(freemapper_courseplan_model::AcceptedFinding {
            rule: "t.map.inside".to_owned(),
            controls: vec![code("31")],
            course: None,
            legs: vec![],
            note: "n".to_owned(),
        });
    let r = run(&e, &strip_map(vec![]));
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    assert_eq!(r.accepted.len(), 1);
}

#[test]
fn the_profile_reports_it_as_an_error_with_its_source() {
    let e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (180.0, 20.0),
    );
    let rules = load_rule_set("wkb-2026", &[]).unwrap();
    let r = run_checks(&CheckContext::new(&e, &strip_map(vec![])), &rules, CHECKS);
    let hit: Vec<&Finding> = r
        .findings
        .iter()
        .filter(|f| f.rule == "wkb2026.map.inside")
        .collect();
    assert_eq!(hit.len(), 1, "{:?}", r.findings);
    assert_eq!(hit[0].severity, Severity::Error);
    assert!(hit[0].source.doc.contains("BL02"));
}
