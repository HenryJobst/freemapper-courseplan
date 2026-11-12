//! Tests of `control_dead_end` and `start_finish_distance` (ADR 0085, ADR 0086). Maps are 1:4000, so one map
//! millimetre is 4 m on the ground.

use freemapper_courseplan_model::{ControlCode, Event, ScoreSettings};

use super::super::testing::*;
use super::super::*;
use crate::omap::{MapDocument, MapObject};

fn start_finish_rule() -> Rule {
    rule("t.course.sfd", "start_finish_distance", Severity::Hint)
}

fn dead_end_rule() -> Rule {
    rule("t.control.dead_end", "control_dead_end", Severity::Warning)
}

fn with_start_finish(mut e: Event, start: (f64, f64), finish: (f64, f64)) -> Event {
    for special in &mut e.specials {
        match special {
            freemapper_courseplan_model::SpecialObject::Start { map_mm, .. } => {
                *map_mm = pt(start.0, start.1);
            }
            freemapper_courseplan_model::SpecialObject::Finish { map_mm, .. } => {
                *map_mm = pt(finish.0, finish.1);
            }
            _ => {}
        }
    }
    e
}

fn open_map() -> MapDocument {
    map(
        4000,
        vec![area_object(
            AREA_521,
            &[(0.0, 0.0), (400.0, 0.0), (400.0, 400.0), (0.0, 400.0)],
        )],
    )
}

#[test]
fn a_finish_beyond_the_limit_is_a_hint_and_one_at_the_limit_is_not() {
    // 100 mm are 400 m: at the limit, no finding. 101 mm are 404 m: a hint.
    let e = event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]);
    let at = with_start_finish(e.clone(), (10.0, 10.0), (110.0, 10.0));
    let r = report(&at, &open_map(), vec![start_finish_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    let beyond = with_start_finish(e, (10.0, 10.0), (111.0, 10.0));
    let r = report(&beyond, &open_map(), vec![start_finish_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
    let f = &r.findings[0];
    assert_eq!(f.severity, Severity::Hint);
    assert_eq!(f.location, Location::Course("A".into()));
    assert_eq!(
        en(f),
        "Start S1 and finish F1 of course A are 404 m apart. Sprint events usually keep them within 400 m (setting start_finish_max_m)."
    );
    assert_eq!(
        de(f),
        "Start S1 und Ziel F1 der Bahn A liegen 404 m auseinander. Bei Sprints liegen sie meist höchstens 400 m auseinander (Einstellung start_finish_max_m)."
    );
}

#[test]
fn the_limit_is_the_event_setting() {
    let mut e = with_start_finish(
        event(&[("31", 50.0, 50.0)], vec![course("A", &["31"])]),
        (10.0, 10.0),
        (111.0, 10.0),
    );
    e.score = Some(ScoreSettings {
        start_finish_max_m: 500,
        ..ScoreSettings::default()
    });
    let r = report(&e, &open_map(), vec![start_finish_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
}

fn dead_end_strip() -> MapDocument {
    // A walled lane 100 m long and 12 m wide that leaves an open field: buildings on both sides, closed at the
    // far end. The lane runs from (200, 100) north to (200, 75) in map mm.
    map(
        4000,
        vec![
            area_object(
                AREA_521,
                &[(0.0, 0.0), (198.5, 0.0), (198.5, 100.0), (0.0, 100.0)],
            ),
            area_object(
                AREA_521,
                &[(201.5, 0.0), (400.0, 0.0), (400.0, 100.0), (201.5, 100.0)],
            ),
            area_object(
                AREA_521,
                &[(198.5, 0.0), (201.5, 0.0), (201.5, 75.0), (198.5, 75.0)],
            ),
            area_object(
                AREA_521,
                &[(0.0, 400.0), (400.0, 400.0), (400.0, 500.0), (0.0, 500.0)],
            ),
        ],
    )
}

fn controls_event(controls: &[(&str, f64, f64)]) -> Event {
    let codes: Vec<&str> = controls.iter().map(|c| c.0).collect();
    with_start_finish(
        event(controls, vec![course("A", &codes)]),
        (100.0, 300.0),
        (110.0, 300.0),
    )
}

#[test]
fn a_control_deep_in_a_dead_end_is_a_warning_and_one_in_the_open_is_not() {
    // The lane tip at y = 75 mm; the lane mouth at y = 100 mm: 25 mm are 100 m. A control at y = 80 is 20 m from
    // the tip and 80 m in, one at y = 200 stands in the open field below the lane.
    let e = controls_event(&[("31", 200.0, 80.0), ("32", 200.0, 200.0)]);
    let r = report(&e, &dead_end_strip(), vec![dead_end_rule()]);
    let hits: Vec<&Finding> = r.findings.iter().collect();
    assert_eq!(hits.len(), 1, "{:?}", r.findings);
    assert_eq!(hits[0].severity, Severity::Warning);
    assert_eq!(
        hits[0].location,
        Location::Control {
            code: ControlCode("31".into())
        }
    );
    assert!(en(hits[0]).starts_with("Control 31 is "), "{}", en(hits[0]));
    assert!(
        en(hits[0]).contains("deep in a dead end, more than the 60 m allowed"),
        "{}",
        en(hits[0])
    );
    assert!(
        de(hits[0]).starts_with("Posten 31 liegt "),
        "{}",
        de(hits[0])
    );
}

#[test]
fn the_dead_end_limit_is_the_event_setting() {
    let mut e = controls_event(&[("31", 200.0, 95.0)]);
    // 5 mm (20 m) in from the mouth: fine at 60 m, a warning at 20 m.
    let r = report(&e, &dead_end_strip(), vec![dead_end_rule()]);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    e.score = Some(ScoreSettings {
        dead_end_max_m: 20,
        ..ScoreSettings::default()
    });
    let r = report(&e, &dead_end_strip(), vec![dead_end_rule()]);
    assert_eq!(r.findings.len(), 1, "{:?}", r.findings);
}

#[test]
fn a_map_without_objects_skips_the_rule() {
    let e = controls_event(&[("31", 200.0, 80.0)]);
    let empty: MapDocument = map(4000, Vec::<MapObject>::new());
    let r = report(&e, &empty, vec![dead_end_rule()]);
    assert!(r.findings.is_empty());
}
