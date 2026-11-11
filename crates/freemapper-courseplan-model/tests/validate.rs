//! Consistency checks on names, ids and geometry (final review of M2a, items 2, 3 and 13).

use freemapper_courseplan_model::{
    ControlCode, Event, MAX_CODE_CHARS, MAX_NAME_CHARS, ModelError, validate,
};

/// A consistent event with start S1, finish F1, controls 31 and 32, course A and class H21, plus
/// `extra` appended as more TOML.
fn event(extra: &str) -> Event {
    let text = format!(
        r#"
format = 1

[event]
title = "T"

[sources]
map = "m.omap"

[[classes]]
name = "H21"

[[specials]]
kind = "start"
id = "S1"
map_mm = [0.0, 0.0]

[[specials]]
kind = "finish"
id = "F1"
map_mm = [50.0, 50.0]

[[controls]]
code = 31
map_mm = [10.0, 10.0]

[[controls]]
code = 32
map_mm = [20.0, 30.0]

[[courses]]
name = "A"
kind = "individual"
classes = ["H21"]
start = "S1"
finish = "F1"
controls = [{{ code = 31 }}, {{ code = 32 }}]
{extra}"#
    );
    toml::from_str(&text).unwrap()
}

#[test]
fn the_base_event_is_consistent() {
    assert_eq!(validate(&event("")), vec![]);
}

fn blank(what: &'static str, value: &str) -> ModelError {
    ModelError::BlankName {
        what,
        value: value.to_owned(),
    }
}

#[test]
fn blank_or_padded_names_are_refused() {
    let mut e = event("");
    e.controls[0].code = ControlCode(String::new());
    e.controls[1].code = ControlCode(" 32".to_owned());
    e.specials[0] = freemapper_courseplan_model::SpecialObject::Start {
        id: "S1 ".to_owned(),
        ppen_id: None,
        map_mm: freemapper_courseplan_model::MapPoint { x: 0.0, y: 0.0 },
    };
    e.courses[0].name = "\t".to_owned();
    e.classes[0].name = String::new();
    let errors = validate(&e);
    for want in [
        blank("control code", ""),
        blank("control code", " 32"),
        blank("special object id", "S1 "),
        blank("course name", "\t"),
        blank("class name", ""),
    ] {
        assert!(errors.contains(&want), "missing {want:?} in {errors:?}");
    }
}

#[test]
fn a_control_code_may_not_be_a_special_id() {
    let mut e = event("");
    e.controls[1].code = ControlCode("F1".to_owned());
    if let freemapper_courseplan_model::CourseKind::Individual { controls } = &mut e.courses[0].kind
    {
        controls[1].code = ControlCode("F1".to_owned());
    }
    assert_eq!(
        validate(&e),
        vec![ModelError::CodeClashesWithSpecial(ControlCode(
            "F1".to_owned()
        ))]
    );
}

#[test]
fn a_course_listed_twice_in_a_special_is_refused() {
    let e = event(
        r#"
[[specials]]
kind = "water"
id = "W1"
map_mm = [5.0, 5.0]
courses = ["A", "A"]
"#,
    );
    assert_eq!(
        validate(&e),
        vec![ModelError::DuplicateCourseInSpecial {
            id: "W1".to_owned(),
            course: "A".to_owned(),
        }]
    );
}

#[test]
fn lines_need_two_points_areas_three_and_a_forbidden_route_one() {
    let e = event(
        r#"
[[specials]]
kind = "marked_route"
id = "M1"
points = [[1.0, 1.0]]

[[specials]]
kind = "forbidden_route"
id = "R1"
points = []

[[specials]]
kind = "out_of_bounds_area"
id = "O1"
polygon = [[1.0, 1.0], [2.0, 2.0]]

[[specials]]
kind = "dangerous_area"
id = "D1"
polygon = [[1.0, 1.0]]

[[specials]]
kind = "marked_route"
id = "M2"
points = [[1.0, 1.0], [2.0, 2.0]]

[[specials]]
kind = "dangerous_area"
id = "D2"
polygon = [[1.0, 1.0], [2.0, 2.0], [3.0, 1.0]]

[[specials]]
kind = "forbidden_route"
id = "R2"
points = [[1.0, 1.0]]
"#,
    );
    let few = |id: &str, kind: &'static str, needed: usize| ModelError::TooFewPoints {
        id: id.to_owned(),
        kind,
        needed,
    };
    assert_eq!(
        validate(&e),
        vec![
            few("M1", "marked route", 2),
            few("R1", "forbidden route", 1),
            few("O1", "out-of-bounds area", 3),
            few("D1", "dangerous area", 3),
        ]
    );
}

#[test]
fn new_errors_read_well_in_english() {
    assert_eq!(
        blank("course name", " A").to_string(),
        "course name \" A\" is empty or starts or ends with a space"
    );
    assert_eq!(
        ModelError::CodeClashesWithSpecial(ControlCode("S1".to_owned())).to_string(),
        "control code S1 is also the id of a special object"
    );
    assert_eq!(
        ModelError::DuplicateCourseInSpecial {
            id: "W1".to_owned(),
            course: "A".to_owned()
        }
        .to_string(),
        "special object W1 lists course A more than once"
    );
    assert_eq!(
        ModelError::TooFewPoints {
            id: "O1".to_owned(),
            kind: "out-of-bounds area",
            needed: 3
        }
        .to_string(),
        "out-of-bounds area O1 needs at least 3 points"
    );
    assert_eq!(
        ModelError::TooFewPoints {
            id: "R1".to_owned(),
            kind: "forbidden route",
            needed: 1
        }
        .to_string(),
        "forbidden route R1 needs at least 1 point"
    );
}

#[test]
fn a_blank_accepted_finding_note_is_refused() {
    for note in ["", "  \t"] {
        let e = event(&format!(
            "\n[[accepted_findings]]\nrule = \"r\"\nnote = {note:?}\n"
        ));
        assert_eq!(
            validate(&e),
            vec![ModelError::BlankNote {
                rule: "r".to_owned()
            }]
        );
    }
    let e = event("\n[[accepted_findings]]\nrule = \"r\"\nnote = \"ok\"\n");
    assert_eq!(validate(&e), vec![]);
}

#[test]
fn over_long_codes_names_and_notes_are_refused() {
    let mut e = event("");
    let code = "9".repeat(MAX_CODE_CHARS + 1);
    e.controls[0].code = ControlCode(code.clone());
    e.courses[0].name = "n".repeat(MAX_NAME_CHARS + 1);
    e.classes[0].name = "c".repeat(MAX_NAME_CHARS + 1);
    let errors = validate(&e);
    let too_long = |what: &'static str, limit: usize| ModelError::TooLong {
        what,
        limit,
        len: limit + 1,
    };
    assert!(errors.contains(&too_long("control code", MAX_CODE_CHARS)));
    assert!(errors.contains(&too_long("course name", MAX_NAME_CHARS)));
    assert!(errors.contains(&too_long("class name", MAX_NAME_CHARS)));

    let long_note = "x".repeat(MAX_NAME_CHARS + 1);
    let e = event(&format!(
        "\n[[accepted_findings]]\nrule = \"r\"\nnote = \"{long_note}\"\n"
    ));
    assert_eq!(
        validate(&e),
        vec![too_long("accepted finding note", MAX_NAME_CHARS)]
    );

    // The limits themselves are fine; characters are counted, not bytes.
    let mut e = event("");
    e.controls[0].code = ControlCode("9".repeat(MAX_CODE_CHARS));
    e.courses[0].name = "ä".repeat(MAX_NAME_CHARS);
    assert!(
        !validate(&e)
            .iter()
            .any(|error| matches!(error, ModelError::TooLong { .. }))
    );
}

#[test]
fn control_and_invisible_characters_in_names_are_refused() {
    for (bad, point) in [
        ("a\u{0}b", "U+0000"),
        ("a\u{b}b", "U+000B"),
        ("a\nb", "U+000A"),
        ("a\u{202e}b", "U+202E"),
        ("a\u{2066}b", "U+2066"),
        ("a\u{200b}b", "U+200B"),
        ("a\u{feff}b", "U+FEFF"),
    ] {
        let mut e = event("");
        e.controls[0].code = ControlCode(bad.to_owned());
        e.courses[0].name = bad.to_owned();
        e.classes[0].name = bad.to_owned();
        let errors = validate(&e);
        for what in ["control code", "course name", "class name"] {
            assert!(
                errors.contains(&ModelError::InvalidCharacter {
                    what,
                    point: point.to_owned()
                }),
                "{what} with {point}: {errors:?}"
            );
        }
    }
    // Ordinary text with accents, emoji and inner spaces stays fine.
    let mut e = event("");
    e.courses[0].name = "Lang B ä 🙂".to_owned();
    assert_eq!(validate(&e), vec![]);
}

fn score(links: &str) -> String {
    format!(
        r#"
[[controls]]
code = 33
map_mm = [30.0, 30.0]

[[controls]]
code = 34
map_mm = [35.0, 30.0]

[[courses]]
name = "S"
kind = "score"
start = "S1"
finish = "F1"
time_limit_min = 45
penalty_per_min = 5
controls = [{{ code = 31, value = 1 }}, {{ code = 32, value = 2 }}, {{ code = 33, value = 3 }}, {{ code = 34, value = 4 }}]
{links}
"#
    )
}

fn code(s: &str) -> ControlCode {
    ControlCode(s.to_owned())
}

#[test]
fn valid_links_are_consistent() {
    let ok = event(&score("links = [{ a = 31, b = 32 }, { a = 32, b = 33 }]"));
    assert_eq!(validate(&ok), vec![]);
}

#[test]
fn link_to_unknown_control_is_refused() {
    let errors = validate(&event(&score("links = [{ a = 31, b = 99 }]")));
    assert_eq!(
        errors,
        vec![ModelError::LinkToUnknownControl {
            course: "S".into(),
            a: code("31"),
            b: code("99"),
            code: code("99"),
        }]
    );
}

#[test]
fn link_to_itself_is_refused() {
    let errors = validate(&event(&score("links = [{ a = 31, b = 31 }]")));
    assert_eq!(
        errors,
        vec![ModelError::LinkToItself {
            course: "S".into(),
            code: code("31"),
        }]
    );
}

#[test]
fn duplicate_link_in_either_direction_is_refused() {
    let errors = validate(&event(&score(
        "links = [{ a = 31, b = 32 }, { a = 32, b = 31 }]",
    )));
    assert_eq!(
        errors,
        vec![ModelError::DuplicateLink {
            course: "S".into(),
            a: code("32"),
            b: code("31"),
        }]
    );
}

#[test]
fn link_chain_may_not_branch() {
    let errors = validate(&event(&score(
        "links = [{ a = 31, b = 32 }, { a = 31, b = 33 }, { a = 31, b = 34 }]",
    )));
    assert_eq!(
        errors,
        vec![ModelError::LinkBranches {
            course: "S".into(),
            code: code("31"),
        }]
    );
}

#[test]
fn settings_out_of_range_are_refused() {
    let bad = |settings: &str| validate(&event(&format!("\n[score]\n{settings}\n")));
    assert_eq!(validate(&event("\n[score]\n")), vec![]);
    for (settings, what) in [
        ("value_max = 0", "value_max"),
        ("value_max = 10", "value_max"),
        ("good_band_percent = [93, 87]", "good_band_percent"),
        (
            "very_good_band_percent = [95, 101]",
            "very_good_band_percent",
        ),
        ("good_pace_s_per_km = 0", "good_pace_s_per_km"),
        ("very_good_pace_s_per_km = 0", "very_good_pace_s_per_km"),
        ("mix_hard_percent = 101", "mix_hard_percent"),
        ("value1_min_m = 0", "value1_min_m"),
        ("min_gap_m = 99", "min_gap_m"),
        ("max_reach_m = 0", "max_reach_m"),
        ("max_reach_m = 100", "max_reach_m"),
        ("high_value_gap_m = 99", "high_value_gap_m"),
    ] {
        let errors = bad(settings);
        assert_eq!(errors.len(), 1, "{settings}: {errors:?}");
        assert!(
            matches!(&errors[0], ModelError::ScoreSettingOutOfRange { what: w, .. } if *w == what),
            "{settings}: {errors:?}"
        );
    }
}

#[test]
fn links_sharing_a_control_without_branching_pass() {
    let ok = event(&score("links = [{ a = 31, b = 32 }, { a = 32, b = 33 }]"));
    assert_eq!(validate(&ok), vec![]);
}

#[test]
fn link_to_a_control_of_another_course_is_refused() {
    // 31 and 32 exist in the project; 35 exists but is not in course S.
    let extra = format!(
        "\n[[controls]]\ncode = 35\nmap_mm = [40.0, 30.0]\n{}",
        score("links = [{ a = 31, b = 35 }]")
    );
    assert_eq!(
        validate(&event(&extra)),
        vec![ModelError::LinkToUnknownControl {
            course: "S".into(),
            a: code("31"),
            b: code("35"),
            code: code("35"),
        }]
    );
}

#[test]
fn both_codes_unknown_gives_one_error() {
    let errors = validate(&event(&score("links = [{ a = 90, b = 91 }]")));
    assert_eq!(errors.len(), 1, "{errors:?}");
}

#[test]
fn a_closed_loop_of_links_is_refused() {
    let errors = validate(&event(&score(
        "links = [{ a = 31, b = 32 }, { a = 32, b = 33 }, { a = 33, b = 31 }]",
    )));
    assert_eq!(
        errors,
        vec![ModelError::LinkCycle {
            course: "S".into(),
            a: code("33"),
            b: code("31"),
        }]
    );
}

#[test]
fn very_good_pace_must_be_below_the_good_pace() {
    for settings in [
        "good_pace_s_per_km = 200",
        "very_good_pace_s_per_km = 250",
        "good_pace_s_per_km = 240\nvery_good_pace_s_per_km = 240",
    ] {
        let errors = validate(&event(&format!("\n[score]\n{settings}\n")));
        assert!(
            matches!(
                errors.as_slice(),
                [ModelError::ScoreSettingOutOfRange {
                    what: "very_good_pace_s_per_km",
                    ..
                }]
            ),
            "{settings}: {errors:?}"
        );
    }
    // A very good band that starts below the good band's top is fine.
    let ok = "\n[score]\ngood_band_percent = [87, 93]\nvery_good_band_percent = [90, 100]\n";
    assert_eq!(validate(&event(ok)), vec![]);
}
