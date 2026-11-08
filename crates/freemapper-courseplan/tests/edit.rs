//! Tests of the edit operations.

use freemapper_courseplan::edit::*;
use freemapper_courseplan_model::{
    Class, Control, ControlCode, ControlDescription, ControlKind, Course, CourseControl,
    CourseKind, Event, EventMeta, FORMAT_VERSION, MapPoint, NumberPlacement, ScoreControl, Sources,
    SpecialObject,
};

fn code(s: &str) -> ControlCode {
    ControlCode(s.to_owned())
}

fn pt(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn cc(s: &str) -> CourseControl {
    CourseControl {
        code: code(s),
        number: None,
        bends: vec![],
        taped: false,
    }
}

fn control(s: &str) -> Control {
    Control {
        code: code(s),
        kind: ControlKind::Control,
        map_mm: pt(10.0, 10.0),
        description: ControlDescription::default(),
        number: None,
    }
}

fn course(name: &str, kind: CourseKind) -> Course {
    Course {
        name: name.to_owned(),
        classes: vec!["H21".to_owned()],
        start: "S1".to_owned(),
        finish: "F1".to_owned(),
        start_bends: vec![],
        start_taped: false,
        print_area: None,
        climb_m: None,
        planned_print_scale: None,
        kind,
    }
}

fn individual(codes: &[&str]) -> CourseKind {
    CourseKind::Individual {
        controls: codes.iter().map(|c| cc(c)).collect(),
    }
}

fn event() -> Event {
    Event {
        format: FORMAT_VERSION,
        event: EventMeta {
            title: "T".to_owned(),
            date: None,
            discipline: Default::default(),
            rule_profile: "wkb-2026".to_owned(),
            start_window_min: None,
        },
        sources: Sources {
            map: "a.omap".into(),
            ppen: None,
        },
        classes: vec![Class {
            name: "H21".to_owned(),
            rule_category: None,
            expected_starters: None,
        }],
        controls: vec![control("31"), control("32"), control("33")],
        specials: vec![
            SpecialObject::Start {
                id: "S1".to_owned(),
                map_mm: pt(0.0, 0.0),
            },
            SpecialObject::Finish {
                id: "F1".to_owned(),
                map_mm: pt(1.0, 1.0),
            },
        ],
        courses: vec![
            course("A", individual(&["31", "32"])),
            course("B", individual(&["33", "32"])),
        ],
        accepted_findings: vec![],
    }
}

fn extent() -> Option<(MapPoint, MapPoint)> {
    Some((pt(-100.0, -50.0), pt(100.0, 50.0)))
}

fn codes_of(event: &Event, name: &str) -> Vec<String> {
    match &event.courses.iter().find(|c| c.name == name).unwrap().kind {
        CourseKind::Individual { controls } => controls.iter().map(|c| c.code.0.clone()).collect(),
        _ => panic!("not individual"),
    }
}

#[test]
fn next_free_code_skips_used() {
    let mut e = event();
    e.controls = vec![control("31"), control("32"), control("34"), control("S1")];
    assert_eq!(next_free_code(&e), code("33"));
}

#[test]
fn add_control_takes_next_free_code_and_refuses_taken() {
    let mut e = event();
    let got = add_control(&mut e, extent(), pt(1.0, 2.0), None, ControlKind::Control).unwrap();
    assert_eq!(got, code("34"));
    let err = add_control(
        &mut e,
        extent(),
        pt(1.0, 2.0),
        Some(code("31")),
        ControlKind::Control,
    )
    .unwrap_err();
    assert_eq!(err, EditError::CodeTaken(code("31")));
}

#[test]
fn add_control_outside_map_is_refused() {
    let mut e = event();
    let before = e.clone();
    let err =
        add_control(&mut e, extent(), pt(100.5, 0.0), None, ControlKind::Control).unwrap_err();
    assert!(matches!(err, EditError::OutsideMap { .. }));
    let text = err.to_string();
    assert!(
        text.contains("(-100.0, -50.0)") && text.contains("(100.0, 50.0)"),
        "{text}"
    );
    assert_eq!(e, before);
    // The edge is inside, and no extent skips the check.
    add_control(
        &mut e,
        extent(),
        pt(100.0, 50.0),
        None,
        ControlKind::Control,
    )
    .unwrap();
    add_control(&mut e, None, pt(1e6, 0.0), None, ControlKind::Control).unwrap();
}

#[test]
fn move_control_checks_extent_and_existence() {
    let mut e = event();
    move_control(&mut e, extent(), &code("31"), pt(5.0, 5.0)).unwrap();
    assert_eq!(e.controls[0].map_mm, pt(5.0, 5.0));
    assert!(matches!(
        move_control(&mut e, extent(), &code("31"), pt(500.0, 0.0)),
        Err(EditError::OutsideMap { .. })
    ));
    assert!(matches!(
        move_control(&mut e, extent(), &code("99"), pt(0.0, 0.0)),
        Err(EditError::NotFound {
            what: "control",
            ..
        })
    ));
}

#[test]
fn remove_control_takes_it_out_of_courses() {
    let mut e = event();
    let courses = remove_control(&mut e, &code("32")).unwrap();
    assert_eq!(courses, ["A", "B"]);
    assert_eq!(codes_of(&e, "A"), ["31"]);
    assert_eq!(codes_of(&e, "B"), ["33"]);
    assert!(e.controls.iter().all(|c| c.code != code("32")));
}

#[test]
fn remove_control_used_by_score_course_is_refused() {
    let mut e = event();
    e.courses.push(course(
        "S",
        CourseKind::Score {
            time_limit_min: 60,
            penalty_per_min: 1,
            controls: vec![ScoreControl {
                code: code("32"),
                value: 10,
            }],
        },
    ));
    let before = e.clone();
    let err = remove_control(&mut e, &code("32")).unwrap_err();
    assert_eq!(
        err,
        EditError::InUse {
            id: "32".to_owned(),
            courses: vec!["S".to_owned()]
        }
    );
    assert_eq!(e, before);
}

#[test]
fn describe_and_central_number_edit_the_control() {
    let mut e = event();
    let d = ControlDescription {
        d: Some("1.8".to_owned()),
        ..Default::default()
    };
    describe_control(&mut e, &code("31"), d.clone()).unwrap();
    let n = NumberPlacement {
        direction: 90.0,
        gap_mm: 1.0,
    };
    set_central_number(&mut e, &code("31"), Some(n.clone())).unwrap();
    assert_eq!(e.controls[0].description, d);
    assert_eq!(e.controls[0].number, Some(n));
    set_central_number(&mut e, &code("31"), None).unwrap();
    assert_eq!(e.controls[0].number, None);
}

#[test]
fn set_course_controls_keeps_leg_data_per_visit() {
    let mut e = event();
    e.controls.push(control("34"));
    let b1 = vec![pt(1.0, 1.0)];
    let b2 = vec![pt(2.0, 2.0)];
    let CourseKind::Individual { controls } = &mut e.courses[0].kind else {
        unreachable!()
    };
    *controls = vec![
        CourseControl {
            bends: b1.clone(),
            ..cc("31")
        },
        cc("32"),
        CourseControl {
            bends: b2.clone(),
            taped: true,
            ..cc("31")
        },
    ];
    set_course_controls(&mut e, "A", &[code("31"), code("31"), code("33")]).unwrap();
    let CourseKind::Individual { controls } = &e.courses[0].kind else {
        unreachable!()
    };
    assert_eq!(controls[0].bends, b1);
    assert_eq!(controls[1].bends, b2);
    assert!(controls[1].taped && !controls[0].taped);
    assert_eq!(controls[2], cc("33"));
    assert!(matches!(
        set_course_controls(&mut e, "A", &[code("99")]),
        Err(EditError::NotFound {
            what: "control",
            ..
        })
    ));
}

#[test]
fn remove_used_start_is_refused() {
    let mut e = event();
    let err = remove_special(&mut e, "S1").unwrap_err();
    assert_eq!(
        err,
        EditError::InUse {
            id: "S1".to_owned(),
            courses: vec!["A".to_owned(), "B".to_owned()]
        }
    );
    e.courses.retain(|c| c.name == "A");
    let err = remove_special(&mut e, "S1").unwrap_err();
    assert_eq!(
        err,
        EditError::InUse {
            id: "S1".to_owned(),
            courses: vec!["A".to_owned()]
        }
    );
}

#[test]
fn add_special_checks_id_and_points() {
    let mut e = event();
    let water = |id: &str, x: f64| SpecialObject::Water {
        id: id.to_owned(),
        map_mm: pt(x, 0.0),
        courses: vec![],
    };
    add_special(&mut e, extent(), water("w1", 0.0)).unwrap();
    assert_eq!(
        add_special(&mut e, extent(), water("w1", 0.0)),
        Err(EditError::IdTaken("w1".to_owned()))
    );
    assert!(matches!(
        add_special(&mut e, extent(), water("w2", 101.0)),
        Err(EditError::OutsideMap { .. })
    ));
    let route = SpecialObject::MarkedRoute {
        id: "m".to_owned(),
        points: vec![pt(0.0, 0.0), pt(0.0, 99.0)],
        courses: vec![],
    };
    assert!(matches!(
        add_special(&mut e, extent(), route),
        Err(EditError::OutsideMap { .. })
    ));
    remove_special(&mut e, "w1").unwrap();
    assert!(matches!(
        remove_special(&mut e, "w1"),
        Err(EditError::NotFound { .. })
    ));
}

#[test]
fn add_and_remove_course() {
    let mut e = event();
    add_course(&mut e, course("C", individual(&["31"]))).unwrap();
    assert!(matches!(
        add_course(&mut e, course("C", individual(&["31"]))),
        Err(EditError::Invalid(_))
    ));
    let team = course(
        "T",
        CourseKind::Team {
            team_size: 2,
            mandatory: vec![],
            free: vec![],
        },
    );
    assert_eq!(
        add_course(&mut e, team),
        Err(EditError::NotSupported {
            course: "T".to_owned()
        })
    );
    remove_course(&mut e, "C").unwrap();
    assert!(matches!(
        remove_course(&mut e, "C"),
        Err(EditError::NotFound { what: "course", .. })
    ));
}

#[test]
fn set_leg_and_course_number() {
    let mut e = event();
    set_leg(&mut e, "A", LegStart::Start, vec![pt(3.0, 3.0)], true).unwrap();
    assert_eq!(e.courses[0].start_bends, vec![pt(3.0, 3.0)]);
    assert!(e.courses[0].start_taped);
    // The leg after the last control runs to the finish.
    set_leg(&mut e, "A", LegStart::After(1), vec![pt(4.0, 4.0)], false).unwrap();
    let CourseKind::Individual { controls } = &e.courses[0].kind else {
        unreachable!()
    };
    assert_eq!(controls[1].bends, vec![pt(4.0, 4.0)]);
    let n = NumberPlacement {
        direction: 0.0,
        gap_mm: 0.5,
    };
    set_course_number(&mut e, "A", 0, Some(n.clone())).unwrap();
    let CourseKind::Individual { controls } = &e.courses[0].kind else {
        unreachable!()
    };
    assert_eq!(controls[0].number, Some(n.clone()));
    assert!(matches!(
        set_course_number(&mut e, "A", 2, Some(n)),
        Err(EditError::BadIndex { index: 2, .. })
    ));
}

#[test]
fn failed_operation_leaves_event_unchanged() {
    let mut e = event();
    let before = e.clone();
    let err = set_leg(&mut e, "A", LegStart::After(5), vec![pt(1.0, 1.0)], true).unwrap_err();
    assert_eq!(
        err,
        EditError::BadIndex {
            course: "A".to_owned(),
            index: 5
        }
    );
    assert_eq!(e, before);
}

#[test]
fn set_course_details_changes_only_given_values() {
    let mut e = event();
    set_course_details(&mut e, "A", None, Some(Some(40)), Some(Some(4000))).unwrap();
    assert_eq!(e.courses[0].climb_m, Some(40));
    assert_eq!(e.courses[0].classes, ["H21"]);
    set_course_details(&mut e, "A", Some(vec![]), Some(None), None).unwrap();
    assert_eq!(e.courses[0].climb_m, None);
    assert_eq!(e.courses[0].planned_print_scale, Some(4000));
    assert!(e.courses[0].classes.is_empty());
}

#[test]
fn remove_class_also_clears_course_classes() {
    let mut e = event();
    add_class(
        &mut e,
        Class {
            name: "D21".to_owned(),
            rule_category: None,
            expected_starters: None,
        },
    )
    .unwrap();
    e.courses[0].classes.push("D21".to_owned());
    remove_class(&mut e, "H21").unwrap();
    assert_eq!(e.classes.len(), 1);
    assert_eq!(e.courses[0].classes, ["D21"]);
    assert!(e.courses[1].classes.is_empty());
    assert!(matches!(
        remove_class(&mut e, "H21"),
        Err(EditError::NotFound { what: "class", .. })
    ));
}

#[test]
fn team_course_edit_is_not_supported() {
    let mut e = event();
    e.courses.push(course(
        "T",
        CourseKind::Team {
            team_size: 2,
            mandatory: vec![cc("31")],
            free: vec![],
        },
    ));
    let not_supported = EditError::NotSupported {
        course: "T".to_owned(),
    };
    assert_eq!(
        set_course_controls(&mut e, "T", &[code("31")]),
        Err(not_supported.clone())
    );
    assert_eq!(
        set_leg(&mut e, "T", LegStart::Start, vec![], false),
        Err(not_supported.clone())
    );
    assert_eq!(
        set_course_number(&mut e, "T", 0, None),
        Err(not_supported.clone())
    );
    assert_eq!(
        set_course_details(&mut e, "T", None, None, None),
        Err(not_supported)
    );
    assert!(matches!(
        remove_control(&mut e, &code("31")),
        Err(EditError::InUse { .. })
    ));
}

fn water(courses: &[&str]) -> SpecialObject {
    SpecialObject::Water {
        id: "w1".to_owned(),
        map_mm: pt(0.0, 0.0),
        courses: courses.iter().map(|c| (*c).to_owned()).collect(),
    }
}

#[test]
fn remove_course_drops_it_from_specials() {
    let mut e = event();
    e.specials.push(water(&["A", "B"]));
    remove_course(&mut e, "A").unwrap();
    assert_eq!(e.specials[2].courses(), ["B"]);
    assert_eq!(e.courses.len(), 1);
}

#[test]
fn remove_course_refuses_when_a_special_would_lose_its_only_course() {
    let mut e = event();
    e.specials.push(water(&["A"]));
    let before = e.clone();
    assert_eq!(
        remove_course(&mut e, "A"),
        Err(EditError::InUse {
            id: "A".to_owned(),
            courses: vec!["w1".to_owned()]
        })
    );
    assert_eq!(e, before);
}

#[test]
fn move_unknown_control_outside_map_is_not_found() {
    let mut e = event();
    assert!(matches!(
        move_control(&mut e, extent(), &code("99"), pt(999.0, 0.0)),
        Err(EditError::NotFound {
            what: "control",
            ..
        })
    ));
}

#[test]
fn set_controls_on_team_course_is_not_supported_even_with_unknown_codes() {
    let mut e = event();
    e.courses.push(course(
        "T",
        CourseKind::Team {
            team_size: 2,
            mandatory: vec![cc("31")],
            free: vec![],
        },
    ));
    assert_eq!(
        set_course_controls(&mut e, "T", &[code("99")]),
        Err(EditError::NotSupported {
            course: "T".to_owned()
        })
    );
    assert!(matches!(
        set_course_controls(&mut e, "nope", &[code("99")]),
        Err(EditError::NotFound { what: "course", .. })
    ));
}
