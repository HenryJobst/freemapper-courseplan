//! Consistency checks on names, ids and geometry (final review of M2a, items 2, 3 and 13).

use freemapper_courseplan_model::{ControlCode, Event, ModelError, validate};

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
fn lines_need_two_points_and_areas_three() {
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
            few("R1", "forbidden route", 2),
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
}
