//! Tests of the edit operations.

use freemapper_courseplan::check::Place;
use freemapper_courseplan::edit::*;
use freemapper_courseplan_model::{
    AcceptedFinding, Class, Control, ControlCode, ControlDescription, ControlKind, Course,
    CourseControl, CourseKind, Event, EventMeta, FORMAT_VERSION, LegRef, MapPoint, NumberPlacement,
    ScoreControl, Sources, SpecialObject,
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
        ppen_id: None,
        code: code(s),
        kind: ControlKind::Control,
        map_mm: pt(10.0, 10.0),
        description: ControlDescription::default(),
        number: None,
    }
}

fn course(name: &str, kind: CourseKind) -> Course {
    Course {
        ppen_id: None,
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
            rule_overlays: Vec::new(),
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
                ppen_id: None,
                map_mm: pt(0.0, 0.0),
            },
            SpecialObject::Finish {
                id: "F1".to_owned(),
                ppen_id: None,
                map_mm: pt(1.0, 1.0),
            },
        ],
        courses: vec![
            course("A", individual(&["31", "32"])),
            course("B", individual(&["33", "32"])),
        ],
        accepted_findings: vec![],
        sync: None,
    }
}

fn extent() -> Option<(MapPoint, MapPoint)> {
    Some((pt(-100.0, -50.0), pt(100.0, 50.0)))
}

/// Sets the stops of an individual course without the edit's bookkeeping, as an edited file would.
fn raw_codes(event: &mut Event, name: &str, codes: &[&str]) {
    let course = event.courses.iter_mut().find(|c| c.name == name).unwrap();
    course.kind = individual(codes);
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
    let removed = remove_control(&mut e, &code("32")).unwrap();
    assert_eq!(removed.courses, ["A", "B"]);
    assert!(removed.cleared.is_empty());
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

fn described(c: &str, d: &str, e: &str, f: &str, g: &str) -> ControlDescription {
    let some = |s: &str| (!s.is_empty()).then(|| s.to_owned());
    ControlDescription {
        c: some(c),
        d: some(d),
        e: some(e),
        f: some(f),
        g: some(g),
        h: None,
    }
}

#[test]
fn describe_refuses_symbols_that_do_not_fit_their_column() {
    let mut e = event();
    let before = e.clone();
    // A G symbol in D, an unknown id, a D-only column D symbol in G, a bend in D.
    for (column, d) in [
        ('D', described("", "12.1N", "", "", "")),
        ('D', described("", "zzz", "", "", "")),
        ('G', described("", "", "", "", "1.3")),
        ('D', described("", "11.1", "", "", "")),
        ('C', described("1.3", "", "", "", "")),
    ] {
        match describe_control(&mut e, &code("31"), d) {
            Err(EditError::BadSymbol { column: c, .. }) => assert_eq!(c, column),
            other => panic!("{other:?}"),
        }
        assert_eq!(e, before);
    }
}

#[test]
fn describe_accepts_what_the_export_accepts() {
    let mut e = event();
    for d in [
        described("0.1N", "5.2", "", "11.1", "12.1N"),
        // E takes D symbols and its own.
        described("", "1.3", "1.3", "10.1", ""),
        described("", "1.3", "8.1", "", "12.14"),
    ] {
        describe_control(&mut e, &code("31"), d.clone()).unwrap();
        assert_eq!(e.controls[0].description, d);
    }
}

#[test]
fn describe_leaves_untouched_ids_alone() {
    let mut e = event();
    // An id written by hand or by an older version stays, as long as the edit does not change that column.
    e.controls[0].description = described("", "zzz", "", "", "");
    let d = described("0.1N", "zzz", "", "", "");
    describe_control(&mut e, &code("31"), d.clone()).unwrap();
    assert_eq!(e.controls[0].description, d);
    let changed = described("0.1N", "yyy", "", "", "");
    assert!(matches!(
        describe_control(&mut e, &code("31"), changed),
        Err(EditError::BadSymbol { column: 'D', .. })
    ));
}

#[test]
fn set_course_controls_keeps_data_per_visit() {
    let mut e = event();
    let b1 = vec![pt(1.0, 1.0)];
    let b2 = vec![pt(2.0, 2.0)];
    let number = |direction: f64| {
        Some(NumberPlacement {
            direction,
            gap_mm: 0.5,
        })
    };
    let CourseKind::Individual { controls } = &mut e.courses[0].kind else {
        unreachable!()
    };
    *controls = vec![
        CourseControl {
            bends: b1.clone(),
            number: number(10.0),
            ..cc("31")
        },
        cc("32"),
        CourseControl {
            bends: b2.clone(),
            taped: true,
            number: number(20.0),
            ..cc("31")
        },
    ];
    // The first visit of 31 still runs to 32 and keeps its bends; the second ran to the finish
    // and now runs to 33, so it loses bends and taped flag. Both keep their numbers.
    let cleared = set_course_controls(
        &mut e,
        "A",
        &[code("31"), code("32"), code("31"), code("33")],
    )
    .unwrap();
    assert_eq!(cleared.cleared, [leg("A", 3)]);
    let CourseKind::Individual { controls } = &e.courses[0].kind else {
        unreachable!()
    };
    assert_eq!(controls[0].bends, b1);
    assert_eq!(controls[0].number, number(10.0));
    assert!(controls[2].bends.is_empty() && !controls[2].taped);
    assert_eq!(controls[2].number, number(20.0));
    assert_eq!(controls[3], cc("33"));
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
        ppen_id: None,
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
        ppen_id: None,
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
    set_leg(
        &mut e,
        extent(),
        "A",
        LegStart::Start,
        vec![pt(3.0, 3.0)],
        true,
    )
    .unwrap();
    assert_eq!(e.courses[0].start_bends, vec![pt(3.0, 3.0)]);
    assert!(e.courses[0].start_taped);
    // The leg after the last control runs to the finish.
    set_leg(
        &mut e,
        extent(),
        "A",
        LegStart::After(1),
        vec![pt(4.0, 4.0)],
        false,
    )
    .unwrap();
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
    let err = set_leg(
        &mut e,
        extent(),
        "A",
        LegStart::After(5),
        vec![pt(1.0, 1.0)],
        true,
    )
    .unwrap_err();
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
        set_leg(&mut e, extent(), "T", LegStart::Start, vec![], false),
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
        ppen_id: None,
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
        Err(EditError::SpecialWouldCoverAllCourses {
            course: "A".to_owned(),
            specials: vec!["w1".to_owned()]
        })
    );
    assert_eq!(e, before);
}

#[test]
fn set_special_courses_assigns_checks_and_widens() {
    let mut e = event();
    e.specials.push(water(&[]));
    set_special_courses(&mut e, "w1", vec!["B".to_owned(), "A".to_owned()]).unwrap();
    assert_eq!(e.specials[2].courses(), ["B", "A"]);
    let before = e.clone();
    assert_eq!(
        set_special_courses(&mut e, "w1", vec!["A".to_owned(), "A".to_owned()]),
        Err(EditError::Invalid(vec![
            freemapper_courseplan_model::ModelError::DuplicateCourseInSpecial {
                id: "w1".to_owned(),
                course: "A".to_owned()
            }
        ]))
    );
    assert!(matches!(
        set_special_courses(&mut e, "w1", vec!["Z".to_owned()]),
        Err(EditError::Invalid(_))
    ));
    assert_eq!(
        set_special_courses(&mut e, "S1", vec!["A".to_owned()]),
        Err(EditError::StartOrFinishHasNoCourses("S1".to_owned()))
    );
    assert!(matches!(
        set_special_courses(&mut e, "nope", vec![]),
        Err(EditError::NotFound {
            what: "special object",
            ..
        })
    ));
    assert_eq!(e, before);
    // An empty list means all courses; after that, removing a course needs nothing more.
    set_special_courses(&mut e, "w1", vec![]).unwrap();
    assert!(e.specials[2].courses().is_empty());
    remove_course(&mut e, "A").unwrap();
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

fn leg(course: &str, after: usize) -> ClearedLeg {
    ClearedLeg {
        course: course.to_owned(),
        after,
    }
}

/// Course A from S1 (0, 0) over 31 (10, 0), 32 (20, 0), 33 (30, 0) to F1 (40, 0), every leg with a
/// bend 5 mm off the line, the leg after 32 taped, and a number at 32.
fn bent_course() -> Event {
    let mut e = event();
    for (control, x) in e.controls.iter_mut().zip([10.0, 20.0, 30.0]) {
        control.map_mm = pt(x, 0.0);
    }
    e.controls.push(Control {
        map_mm: pt(25.0, 0.0),
        ..control("34")
    });
    e.specials[1] = SpecialObject::Finish {
        id: "F1".to_owned(),
        ppen_id: None,
        map_mm: pt(40.0, 0.0),
    };
    let course = &mut e.courses[0];
    course.start_bends = vec![pt(5.0, 5.0)];
    course.kind = CourseKind::Individual {
        controls: vec![
            CourseControl {
                bends: vec![pt(15.0, 5.0)],
                ..cc("31")
            },
            CourseControl {
                bends: vec![pt(25.0, 5.0)],
                taped: true,
                number: Some(NumberPlacement {
                    direction: 90.0,
                    gap_mm: 0.5,
                }),
                ..cc("32")
            },
            CourseControl {
                bends: vec![pt(35.0, 5.0)],
                ..cc("33")
            },
        ],
    };
    e.courses.truncate(1);
    e
}

fn legs_of(e: &Event) -> Vec<(String, usize, bool)> {
    let CourseKind::Individual { controls } = &e.courses[0].kind else {
        unreachable!()
    };
    controls
        .iter()
        .map(|c| (c.code.0.clone(), c.bends.len(), c.taped))
        .collect()
}

#[test]
fn reorder_clears_only_the_legs_whose_next_stop_changed() {
    let mut e = bent_course();
    // 31 -> 32 stays, 32 -> 33 becomes 32 -> 34, 33 -> finish stays.
    let cleared = set_course_controls(
        &mut e,
        "A",
        &[code("31"), code("32"), code("34"), code("33")],
    )
    .unwrap();
    assert_eq!(cleared.cleared, [leg("A", 2)]);
    assert_eq!(
        legs_of(&e),
        [
            ("31".to_owned(), 1, false),
            ("32".to_owned(), 0, false),
            ("34".to_owned(), 0, false),
            ("33".to_owned(), 1, false),
        ]
    );
    assert_eq!(e.courses[0].start_bends.len(), 1);
    // The number belongs to the visit, not to the leg, and stays.
    let CourseKind::Individual { controls } = &e.courses[0].kind else {
        unreachable!()
    };
    assert!(controls[1].number.is_some());
}

#[test]
fn reorder_reports_the_start_leg_as_position_zero() {
    let mut e = bent_course();
    let cleared = set_course_controls(&mut e, "A", &[code("32"), code("31"), code("33")]).unwrap();
    // Start -> 31 becomes start -> 32; 31 -> 32 becomes 31 -> 33; 32 -> 33 becomes 32 -> 31.
    assert_eq!(cleared.cleared, [leg("A", 0), leg("A", 1), leg("A", 2)]);
    assert!(e.courses[0].start_bends.is_empty());
    assert_eq!(
        legs_of(&e),
        [
            ("32".to_owned(), 0, false),
            ("31".to_owned(), 0, false),
            ("33".to_owned(), 1, false),
        ]
    );
}

#[test]
fn removal_clears_the_previous_leg_and_the_length_is_the_straight_sum() {
    let mut e = bent_course();
    let removed = remove_control(&mut e, &code("32")).unwrap();
    assert_eq!(removed.courses, ["A"]);
    assert_eq!(removed.cleared, [leg("A", 1)]);
    assert_eq!(
        legs_of(&e),
        [("31".to_owned(), 0, false), ("33".to_owned(), 1, false)]
    );
    // Straight from 31 to 33 now: 5 + 5 (start leg via its bend), 20, and the 33 leg via its bend.
    let bend = (5.0f64 * 5.0 + 5.0 * 5.0).sqrt();
    let want_mm = 2.0 * bend + 20.0 + 2.0 * bend;
    let metres = freemapper_courseplan::course::course_length_m(&e, &e.courses[0], 1000).unwrap();
    assert!((metres - want_mm).abs() < 1e-9, "{metres} vs {want_mm}");

    // Without bends anywhere the length is the straight sum of the remaining stops.
    let mut e = bent_course();
    let removed = remove_control(&mut e, &code("31")).unwrap();
    assert_eq!(removed.cleared, [leg("A", 0)]);
    set_leg(&mut e, extent(), "A", LegStart::After(0), vec![], false).unwrap();
    set_leg(&mut e, extent(), "A", LegStart::After(1), vec![], false).unwrap();
    let metres = freemapper_courseplan::course::course_length_m(&e, &e.courses[0], 1000).unwrap();
    assert!((metres - 40.0).abs() < 1e-9, "{metres}");
}

// ---------------------------------------------------------------------------------------------
// Accepted findings and the rule profile

fn wkb() -> freemapper_courseplan::rules::RuleSet {
    freemapper_courseplan::rules::load_rule_set("wkb-2026", &[]).unwrap()
}

fn acceptance(rule: &str, controls: &[&str], note: &str) -> AcceptedFinding {
    AcceptedFinding {
        rule: rule.to_owned(),
        controls: controls.iter().map(|c| code(c)).collect(),
        course: None,
        legs: vec![],
        note: note.to_owned(),
    }
}

#[test]
fn accepting_a_finding_twice_replaces_its_note() {
    let mut e = event();
    let rules = wkb();
    let first = acceptance("wkb2026.distance.min", &["32", "31", "32"], "on purpose");
    assert_eq!(
        accept_finding(&mut e, &rules, first).unwrap().0,
        Accepted::Added
    );
    // Stored with the key's order, so the file reads the same way as the check output.
    assert_eq!(e.accepted_findings[0].controls, [code("31"), code("32")]);
    let again = acceptance("wkb2026.distance.min", &["31", "32"], "still on purpose");
    assert_eq!(
        accept_finding(&mut e, &rules, again).unwrap().0,
        Accepted::NoteReplaced
    );
    assert_eq!(e.accepted_findings.len(), 1);
    assert_eq!(e.accepted_findings[0].note, "still on purpose");
}

#[test]
fn accepting_needs_a_known_rule_and_a_place_that_exists() {
    let mut e = event();
    let before = e.clone();
    let rules = wkb();
    let refused = |e: &mut Event, a: AcceptedFinding| accept_finding(e, &rules, a).unwrap_err();

    let unknown = refused(&mut e, acceptance("wkb2026.no.such", &["31"], "n"));
    assert!(
        matches!(unknown, EditError::UnknownRule { .. }),
        "{unknown:?}"
    );
    let missing = refused(
        &mut e,
        acceptance("wkb2026.distance.min", &["31", "99"], "n"),
    );
    assert_eq!(
        missing,
        EditError::NotFound {
            what: "control",
            id: "99".to_owned()
        }
    );
    // A distance finding never sits at the event as a whole.
    let nowhere = refused(&mut e, acceptance("wkb2026.distance.min", &[], "n"));
    assert!(
        matches!(nowhere, EditError::FindingNeedsPlace { .. }),
        "{nowhere:?}"
    );
    // A finding has one place: a course, controls or one leg.
    let mut mixed = acceptance("wkb2026.distance.min", &["31"], "n");
    mixed.course = Some("A".to_owned());
    let mixed = refused(&mut e, mixed);
    assert!(
        matches!(mixed, EditError::FindingHasOnePlace { .. }),
        "{mixed:?}"
    );
    // Several legs are one place only inside one course, and only for a rule whose findings
    // name two legs.
    let mut two_courses = acceptance("wkb2026.leg.opposing", &[], "n");
    two_courses.legs = vec![leg_ref("A", 1), leg_ref("B", 2)];
    let two_courses = refused(&mut e, two_courses);
    assert!(
        matches!(two_courses, EditError::FindingHasOnePlace { .. }),
        "{two_courses:?}"
    );
    let mut two_legs = acceptance("wkb2026.leg.through_control", &[], "n");
    two_legs.legs = vec![leg_ref("A", 1), leg_ref("A", 2)];
    assert_eq!(
        refused(&mut e, two_legs),
        EditError::FindingWrongPlace {
            rule: "wkb2026.leg.through_control".to_owned(),
            place: Place::Legs,
            places: vec![Place::Leg],
        }
    );
    let mut one_leg = acceptance("wkb2026.leg.opposing", &[], "n");
    one_leg.legs = vec![leg_ref("A", 1)];
    assert_eq!(
        refused(&mut e, one_leg),
        EditError::FindingWrongPlace {
            rule: "wkb2026.leg.opposing".to_owned(),
            place: Place::Leg,
            places: vec![Place::Legs],
        }
    );
    let mut opposing_beyond = acceptance("wkb2026.leg.opposing", &[], "n");
    opposing_beyond.legs = vec![leg_ref("A", 0), leg_ref("A", 3)];
    assert!(matches!(
        refused(&mut e, opposing_beyond),
        EditError::BadIndex { index: 3, .. }
    ));
    let mut beyond = acceptance("wkb2026.leg.through_control", &[], "n");
    beyond.legs = vec![leg_ref("A", 3)];
    assert_eq!(
        refused(&mut e, beyond),
        EditError::BadIndex {
            course: "A".to_owned(),
            index: 3
        }
    );
    let mut no_course = acceptance("wkb2026.length.long", &[], "n");
    no_course.course = Some("Z".to_owned());
    assert!(matches!(
        refused(&mut e, no_course),
        EditError::NotFound { what: "course", .. }
    ));
    // Each key reports at its own kind of place: a distance finding at two controls, a map
    // edge finding at one, a leg finding at a leg.
    let mut at_course = acceptance("wkb2026.distance.min", &[], "n");
    at_course.course = Some("A".to_owned());
    assert_eq!(
        refused(&mut e, at_course),
        EditError::FindingWrongPlace {
            rule: "wkb2026.distance.min".to_owned(),
            place: Place::Course,
            places: vec![Place::Controls],
        }
    );
    let one = refused(&mut e, acceptance("wkb2026.distance.min", &["31"], "n"));
    assert!(
        matches!(
            one,
            EditError::FindingWrongPlace {
                place: Place::Control,
                ..
            }
        ),
        "{one:?}"
    );
    let two = refused(&mut e, acceptance("wkb2026.map.edge", &["31", "32"], "n"));
    assert!(
        matches!(two, EditError::FindingWrongPlace { .. }),
        "{two:?}"
    );
    let at_controls = refused(
        &mut e,
        acceptance("wkb2026.leg.through_forbidden_sprint", &["31", "32"], "n"),
    );
    assert!(
        matches!(
            at_controls,
            EditError::FindingWrongPlace {
                place: Place::Controls,
                ..
            }
        ),
        "{at_controls:?}"
    );
    let blank = refused(
        &mut e,
        acceptance("wkb2026.distance.min", &["31", "32"], "  "),
    );
    assert!(matches!(blank, EditError::Invalid(_)), "{blank:?}");
    assert_eq!(e, before, "refused edits leave the event as it was");

    // The last leg (after the last control, to the finish) and the start leg exist.
    for after in [0, 2] {
        let mut ok = acceptance("wkb2026.leg.through_control", &[], "n");
        ok.legs = vec![leg_ref("A", after)];
        accept_finding(&mut e, &rules, ok).unwrap();
    }
    // A start or finish is a place of a distance or map edge finding.
    accept_finding(
        &mut e,
        &rules,
        acceptance("wkb2026.distance.min", &["31", "S1"], "n"),
    )
    .unwrap();
    accept_finding(&mut e, &rules, acceptance("wkb2026.map.edge", &["F1"], "n")).unwrap();
    // An acute angle sits at the incoming leg, opposing legs at both legs, stored in key order.
    let mut angle = acceptance("wkb2026.leg.acute_angle", &[], "n");
    angle.legs = vec![leg_ref("A", 1)];
    accept_finding(&mut e, &rules, angle).unwrap();
    let mut opposing = acceptance("wkb2026.leg.opposing", &[], "n");
    opposing.legs = vec![leg_ref("A", 2), leg_ref("A", 0)];
    accept_finding(&mut e, &rules, opposing).unwrap();
    let stored = e.accepted_findings.last().unwrap();
    let ends = |after, from: &str, to: &str| LegRef {
        from: from.to_owned(),
        to: to.to_owned(),
        ..leg_ref("A", after)
    };
    // The ends come from the course as it is now (ruling R24).
    assert_eq!(stored.legs, [ends(0, "S1", "31"), ends(2, "32", "F1")]);
    // A start-capacity finding sits at its start.
    accept_finding(
        &mut e,
        &rules,
        acceptance("wkb2026.start.capacity", &["S1"], "n"),
    )
    .unwrap();
    assert_eq!(e.accepted_findings.len(), 7);
}

fn leg_ref(course: &str, after: u32) -> LegRef {
    LegRef::at(course, after)
}

#[test]
fn removing_an_accepted_finding_needs_its_key() {
    let mut e = event();
    let rules = wkb();
    accept_finding(
        &mut e,
        &rules,
        acceptance("wkb2026.distance.min", &["31", "32"], "n"),
    )
    .unwrap();
    let key = acceptance("wkb2026.distance.min", &["32", "31"], "").key();
    let other = acceptance("wkb2026.distance.min", &["31"], "").key();
    assert_eq!(
        remove_accepted_finding(&mut e, &other).unwrap_err(),
        EditError::AcceptedNotFound {
            rule: "wkb2026.distance.min".to_owned()
        }
    );
    remove_accepted_finding(&mut e, &key).unwrap();
    assert!(e.accepted_findings.is_empty());
    assert!(remove_accepted_finding(&mut e, &key).is_err());
}

#[test]
fn a_leg_acceptance_stores_the_ends_and_is_removed_by_position() {
    let mut e = event();
    let rules = wkb();
    let mut angle = acceptance("wkb2026.leg.acute_angle", &[], "n");
    // Ends given by the caller are replaced with the ends in the course.
    angle.legs = vec![LegRef {
        from: "x".to_owned(),
        to: "y".to_owned(),
        ..leg_ref("A", 1)
    }];
    accept_finding(&mut e, &rules, angle).unwrap();
    let stored = &e.accepted_findings[0].legs[0];
    assert_eq!((stored.from.as_str(), stored.to.as_str()), ("31", "32"));
    // The leg now runs 31–33; the acceptance still names 31–32 and is removed by position.
    raw_codes(&mut e, "A", &["31", "33"]);
    assert_eq!(e.accepted_findings[0].legs[0].to, "32");
    let mut named = acceptance("wkb2026.leg.acute_angle", &[], "");
    named.legs = vec![leg_ref("A", 1)];
    let removed = remove_accepted_finding(&mut e, &named.key()).unwrap();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].legs[0].to, "32");
    assert!(e.accepted_findings.is_empty());
    // A leg of a course that is gone can be removed too.
    let mut gone = acceptance("wkb2026.leg.acute_angle", &[], "n");
    gone.legs = vec![LegRef {
        from: "31".to_owned(),
        to: "32".to_owned(),
        ..leg_ref("Z", 1)
    }];
    e.accepted_findings.push(gone);
    let mut named = acceptance("wkb2026.leg.acute_angle", &[], "");
    named.legs = vec![leg_ref("Z", 1)];
    remove_accepted_finding(&mut e, &named.key()).unwrap();
    assert!(e.accepted_findings.is_empty());
}

#[test]
fn an_exact_leg_acceptance_wins_over_a_stale_one_at_the_same_position() {
    let mut e = event();
    let rules = wkb();
    let angle = |note: &str| AcceptedFinding {
        legs: vec![leg_ref("A", 1)],
        ..acceptance("wkb2026.leg.acute_angle", &[], note)
    };
    accept_finding(&mut e, &rules, angle("old leg")).unwrap();
    // Leg A:1 now runs 31–33; accepting it again stores a second entry and returns that one.
    raw_codes(&mut e, "A", &["31", "33"]);
    let (done, stored) = accept_finding(&mut e, &rules, angle("new leg")).unwrap();
    assert_eq!(done, Accepted::Added);
    assert_eq!(
        (stored.legs[0].to.as_str(), stored.note.as_str()),
        ("33", "new leg")
    );
    assert_eq!(e.accepted_findings.len(), 2);
    // Removing A:1 takes only the entry whose ends match the course now.
    let named = AcceptedFinding {
        legs: vec![leg_ref("A", 1)],
        ..acceptance("wkb2026.leg.acute_angle", &[], "")
    };
    let removed = remove_accepted_finding(&mut e, &named.key()).unwrap();
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].legs[0].to, "33");
    assert_eq!(e.accepted_findings[0].note, "old leg");
    // Without an exact match the position fallback removes every stale entry there.
    e.accepted_findings.push(AcceptedFinding {
        legs: vec![LegRef {
            from: "31".to_owned(),
            to: "39".to_owned(),
            ..leg_ref("A", 1)
        }],
        ..acceptance("wkb2026.leg.acute_angle", &[], "older leg")
    });
    let removed = remove_accepted_finding(&mut e, &named.key()).unwrap();
    assert_eq!(removed.len(), 2);
    assert!(e.accepted_findings.is_empty());
}

#[test]
fn the_start_window_is_at_least_one_minute() {
    let mut e = event();
    set_start_window(&mut e, 90).unwrap();
    assert_eq!(e.event.start_window_min, Some(90));
    let before = e.clone();
    assert_eq!(
        set_start_window(&mut e, 0).unwrap_err(),
        EditError::StartWindowZero
    );
    assert_eq!(e, before);
}

#[test]
fn the_start_window_can_be_cleared() {
    let mut e = event();
    set_start_window(&mut e, 90).unwrap();
    clear_start_window(&mut e).unwrap();
    assert_eq!(e.event.start_window_min, None);
    // Clearing what is not set is fine.
    clear_start_window(&mut e).unwrap();
}

#[test]
fn a_class_gets_and_loses_its_category_and_starters() {
    let mut e = event();
    set_class(
        &mut e,
        "H21",
        Some(Some(" H21 ".to_owned())),
        Some(Some(40)),
    )
    .unwrap();
    assert_eq!(e.classes[0].rule_category.as_deref(), Some("H21"));
    assert_eq!(e.classes[0].expected_starters, Some(40));
    // None keeps a value; Some(None) clears it.
    set_class(&mut e, "H21", None, Some(None)).unwrap();
    assert_eq!(e.classes[0].rule_category.as_deref(), Some("H21"));
    assert_eq!(e.classes[0].expected_starters, None);
    set_class(&mut e, "H21", Some(None), None).unwrap();
    assert_eq!(e.classes[0].rule_category, None);
    let before = e.clone();
    assert_eq!(
        set_class(&mut e, "H21", Some(Some("  ".to_owned())), None).unwrap_err(),
        EditError::BlankRuleCategory {
            class: "H21".to_owned()
        }
    );
    assert!(matches!(
        set_class(&mut e, "D21", None, Some(Some(3))).unwrap_err(),
        EditError::NotFound { what: "class", .. }
    ));
    assert_eq!(e, before);
}

#[test]
fn rule_profile_is_validated_before_it_is_set() {
    let mut e = event();
    let set = set_rule_profile(&mut e, "wkb-2026", &["nor-bb-2021".to_owned()]).unwrap();
    assert_eq!(set.overlays, ["nor-bb-2021"]);
    assert_eq!(e.event.rule_overlays, ["nor-bb-2021"]);
    let before = e.clone();
    let unknown = set_rule_profile(&mut e, "xyz", &[]).unwrap_err();
    assert!(unknown.to_string().contains("wkb-2026"), "{unknown}");
    assert!(set_rule_profile(&mut e, "nor-bb-2021", &[]).is_err());
    assert!(set_rule_profile(&mut e, "wkb-2026", &["wkb-2026".to_owned()]).is_err());
    assert_eq!(e, before);
    set_rule_profile(&mut e, "wkb-2026", &[]).unwrap();
    assert!(e.event.rule_overlays.is_empty());
}

// ---------------------------------------------------------------------------------------------
// Changing a code, renaming a course

#[test]
fn change_code_follows_into_courses_and_accepted_findings() {
    let mut e = event();
    let mut a = acceptance("wkb2026.distance.min", &["31", "32"], "on purpose");
    a.legs = vec![];
    e.accepted_findings.push(a);
    e.accepted_findings.push(AcceptedFinding {
        rule: "wkb2026.leg.opposing".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![LegRef {
            course: "A".to_owned(),
            after: 1,
            from: "31".to_owned(),
            to: "32".to_owned(),
        }],
        note: "n".to_owned(),
    });
    change_code(&mut e, &code("32"), &code("77")).unwrap();
    assert_eq!(codes_of(&e, "A"), ["31", "77"]);
    assert_eq!(codes_of(&e, "B"), ["33", "77"]);
    assert!(e.controls.iter().any(|c| c.code == code("77")));
    assert!(!e.controls.iter().any(|c| c.code == code("32")));
    assert_eq!(e.accepted_findings[0].controls, [code("31"), code("77")]);
    assert_eq!(e.accepted_findings[1].legs[0].to, "77");
    assert_eq!(e.accepted_findings[1].legs[0].from, "31");
}

#[test]
fn change_code_to_a_taken_code_is_refused_and_leaves_the_event_equal() {
    let mut e = event();
    let before = e.clone();
    assert_eq!(
        change_code(&mut e, &code("31"), &code("32")),
        Err(EditError::CodeTaken(code("32")))
    );
    assert_eq!(e, before);
    assert!(matches!(
        change_code(&mut e, &code("99"), &code("98")),
        Err(EditError::NotFound {
            what: "control",
            ..
        })
    ));
    for bad in ["", " 40", "40 ", &"x".repeat(65)] {
        assert!(
            matches!(
                change_code(&mut e, &code("31"), &code(bad)),
                Err(EditError::Invalid(_))
            ),
            "{bad:?}"
        );
    }
    assert_eq!(e, before);
    // The same code is no change.
    change_code(&mut e, &code("31"), &code("31")).unwrap();
    assert_eq!(e, before);
}

#[test]
fn rename_course_follows_into_specials_and_findings() {
    let mut e = event();
    e.specials.push(water(&["A", "B"]));
    e.accepted_findings.push(AcceptedFinding {
        rule: "wkb2026.leg.opposing".to_owned(),
        controls: vec![],
        course: Some("A".to_owned()),
        legs: vec![],
        note: "n".to_owned(),
    });
    e.accepted_findings.push(AcceptedFinding {
        rule: "wkb2026.leg.opposing".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![LegRef::at("A", 1)],
        note: "n".to_owned(),
    });
    change_course_name(&mut e, "A", "Lang").unwrap();
    assert_eq!(e.courses[0].name, "Lang");
    assert_eq!(e.courses[0].classes, ["H21"]);
    match &e.specials[2] {
        SpecialObject::Water { courses, .. } => assert_eq!(courses, &["Lang", "B"]),
        other => panic!("{other:?}"),
    }
    assert_eq!(e.accepted_findings[0].course.as_deref(), Some("Lang"));
    assert_eq!(e.accepted_findings[1].legs[0].course, "Lang");
}

#[test]
fn rename_course_refuses_taken_missing_blank_and_long_names() {
    let mut e = event();
    let before = e.clone();
    assert_eq!(
        change_course_name(&mut e, "A", "B"),
        Err(EditError::NameTaken("B".to_owned()))
    );
    assert!(matches!(
        change_course_name(&mut e, "Z", "Y"),
        Err(EditError::NotFound { what: "course", .. })
    ));
    for bad in ["", " X", "X ", &"x".repeat(201)] {
        assert!(
            matches!(
                change_course_name(&mut e, "A", bad),
                Err(EditError::Invalid(_))
            ),
            "{bad:?}"
        );
    }
    assert_eq!(e, before);
    change_course_name(&mut e, "A", "A").unwrap();
    assert_eq!(e, before);
}

fn leg_acceptance(course: &str, after: u32, from: &str, to: &str) -> AcceptedFinding {
    AcceptedFinding {
        rule: "wkb2026.leg.acute_angle".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![LegRef {
            course: course.to_owned(),
            after,
            from: from.to_owned(),
            to: to.to_owned(),
        }],
        note: "n".to_owned(),
    }
}

fn leg_after(e: &Event) -> Vec<u32> {
    e.accepted_findings
        .iter()
        .map(|f| f.legs[0].after)
        .collect()
}

#[test]
fn inserting_a_stop_moves_an_accepted_leg_with_it() {
    let mut e = event();
    // Course A runs S1 31 32 F1; the leg 31 to 32 leaves position 1.
    e.accepted_findings.push(leg_acceptance("A", 1, "31", "32"));
    e.accepted_findings.push(leg_acceptance("A", 2, "32", "F1"));
    let edit = set_course_controls(&mut e, "A", &[code("33"), code("31"), code("32")]).unwrap();
    assert!(edit.dropped.is_empty());
    assert_eq!(leg_after(&e), [2, 3]);
}

#[test]
fn removing_a_stop_moves_or_drops_accepted_legs() {
    let mut e = event();
    e.controls.push(control("34"));
    raw_codes(&mut e, "A", &["31", "32", "34"]);
    e.accepted_findings.push(leg_acceptance("A", 2, "32", "34"));
    e.accepted_findings.push(leg_acceptance("A", 3, "34", "F1"));
    e.accepted_findings.push(leg_acceptance("A", 1, "31", "32"));
    // Taking 31 away: the leg 31-32 is gone, the others move up by one.
    let removed = remove_control(&mut e, &code("31")).unwrap();
    assert_eq!(removed.dropped.len(), 1);
    assert_eq!(removed.dropped[0].from, "31");
    assert_eq!(leg_after(&e), [1, 2]);
    assert_eq!(e.accepted_findings[0].legs[0].to, "34");
}

#[test]
fn reordering_keeps_legs_that_survive_and_drops_the_rest() {
    let mut e = event();
    e.accepted_findings.push(leg_acceptance("A", 1, "31", "32"));
    e.accepted_findings.push(leg_acceptance("A", 2, "32", "F1"));
    let edit = set_course_controls(&mut e, "A", &[code("32"), code("31")]).unwrap();
    // 31 to 32 and 32 to the finish no longer exist.
    assert_eq!(edit.dropped.len(), 2);
    assert!(e.accepted_findings.is_empty());
}

#[test]
fn another_courses_accepted_legs_stay_when_one_course_changes() {
    let mut e = event();
    e.accepted_findings.push(leg_acceptance("B", 1, "33", "32"));
    set_course_controls(&mut e, "A", &[code("32")]).unwrap();
    assert_eq!(leg_after(&e), [1]);
}

#[test]
fn renaming_a_control_or_course_rewrites_sync_decisions() {
    use freemapper_courseplan_model::{Side, SyncDecision, SyncState};
    let mut e = event();
    let d = |key: &str| SyncDecision {
        key: key.to_owned(),
        take: Side::Ours,
        fingerprint: "f".to_owned(),
    };
    e.sync = Some(SyncState {
        decisions: vec![
            d("control/31/position"),
            d("control/310/position"),
            d("course/A/number/31"),
            d("course/A/number/31#2"),
            d("course/A/stops"),
            d("leg/31-32"),
            d("leg/S1-31"),
            d("course/B/stops"),
        ],
        ..Default::default()
    });
    change_code(&mut e, &code("31"), &code("41")).unwrap();
    change_course_name(&mut e, "A", "Lang").unwrap();
    let keys: Vec<&str> = e
        .sync
        .as_ref()
        .unwrap()
        .decisions
        .iter()
        .map(|d| d.key.as_str())
        .collect();
    assert_eq!(
        keys,
        [
            "control/41/position",
            "control/310/position",
            "course/Lang/number/41",
            "course/Lang/number/41#2",
            "course/Lang/stops",
            "leg/41-32",
            "leg/S1-41",
            "course/B/stops",
        ]
    );
}

#[test]
fn set_leg_checks_the_extent_and_caps_the_bends() {
    let mut e = event();
    let before = e.clone();
    let err = set_leg(
        &mut e,
        extent(),
        "A",
        LegStart::Start,
        vec![pt(1e9, 0.0)],
        false,
    )
    .unwrap_err();
    assert!(matches!(err, EditError::OutsideMap { .. }));
    let many = vec![pt(1.0, 1.0); MAX_BENDS + 1];
    let err = set_leg(&mut e, extent(), "A", LegStart::After(0), many, false).unwrap_err();
    assert!(matches!(
        err,
        EditError::TooManyBends {
            limit: MAX_BENDS,
            ..
        }
    ));
    assert_eq!(e, before);
    let fine = vec![pt(1.0, 1.0); MAX_BENDS];
    set_leg(&mut e, extent(), "A", LegStart::After(0), fine, false).unwrap();
    // Without a known extent, only the count is checked.
    set_leg(
        &mut e,
        None,
        "A",
        LegStart::Start,
        vec![pt(1e9, 0.0)],
        false,
    )
    .unwrap();
}
