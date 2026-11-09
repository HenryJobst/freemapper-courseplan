use freemapper_course::{FitError, Special};
use freemapper_courseplan_model::{
    Control, ControlDescription, ControlKind, Course, CourseControl, CourseKind, MapPoint,
    NumberPlacement, SpecialObject,
};

use super::*;
use crate::check::testing::{map, point_object};
use crate::omap::MapDocument;

/// A sprint map at 1:4000 whose objects span (0, -200) to (200, 200) map mm, so that every point of
/// the test files (y up, 0 to 200 mm) lies on it after the fit.
fn sprint_map() -> MapDocument {
    map(
        4000,
        vec![point_object(0, 0.0, -200.0), point_object(0, 200.0, 200.0)],
    )
}

/// A `.ppen` at map scale 1:`scale` with `body` after the event.
fn ppen_at(scale: u32, body: &str) -> String {
    format!(
        r#"<course-scribe-event>
  <event id="1"><title>Test</title><map kind="none" scale="{scale}" /></event>
{body}
</course-scribe-event>"#
    )
}

fn ppen(body: &str) -> String {
    ppen_at(4000, body)
}

fn import(body: &str) -> PpenImport {
    import_ppen(ppen(body).as_bytes(), &sprint_map()).unwrap()
}

/// Start 1 at (10, 10), finish 2 at (100, 10) and normal controls 3 (code 31) at (30, 30), 4 (code
/// 32) at (60, 60), with descriptions given by `extra` per control id.
fn controls(extra: &[(u32, &str)]) -> String {
    let more = |id: u32| {
        extra
            .iter()
            .filter(|(i, _)| *i == id)
            .map(|(_, x)| *x)
            .collect::<String>()
    };
    format!(
        r#"  <control id="1" kind="start"><location x="10" y="10" />{}</control>
  <control id="2" kind="finish"><location x="100" y="10" />{}</control>
  <control id="3" kind="normal"><code>31</code><location x="30" y="30" />{}</control>
  <control id="4" kind="normal"><code>32</code><location x="60" y="60" />{}</control>
"#,
        more(1),
        more(2),
        more(3),
        more(4)
    )
}

/// A normal course `name` with id `id` over the controls of `chain` (control ids), with course-control
/// ids from `first` on; `cc_extra` adds children to the course-control of a control id.
fn course(
    id: u32,
    name: &str,
    first: u32,
    chain: &[u32],
    options: &str,
    cc_extra: &[(u32, &str)],
) -> String {
    let mut out = format!(
        r#"  <course id="{id}" kind="normal" order="{id}"><name>{name}</name><first course-control="{first}" />{options}</course>
"#
    );
    for (i, control) in chain.iter().enumerate() {
        let cc = first + i as u32;
        let next = if i + 1 < chain.len() {
            format!(r#"<next course-control="{}" />"#, cc + 1)
        } else {
            String::new()
        };
        let extra: String = cc_extra
            .iter()
            .filter(|(c, _)| c == control)
            .map(|(_, x)| *x)
            .collect();
        out.push_str(&format!(
            r#"  <course-control id="{cc}" control="{control}">{next}{extra}</course-control>
"#
        ));
    }
    out
}

fn plain_course() -> String {
    course(
        1,
        "A",
        1,
        &[1, 3, 4, 2],
        r#"<options print-scale="4000" />"#,
        &[],
    )
}

fn control<'a>(import: &'a PpenImport, code: &str) -> &'a Control {
    import
        .controls
        .iter()
        .find(|c| c.code.0 == code)
        .unwrap_or_else(|| panic!("no control {code} in {:?}", import.controls))
}

fn course_named<'a>(import: &'a PpenImport, name: &str) -> &'a Course {
    import
        .courses
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no course {name} in {:?}", import.courses))
}

fn course_controls(course: &Course) -> &[CourseControl] {
    match &course.kind {
        CourseKind::Individual { controls } => controls,
        other => panic!("not individual: {other:?}"),
    }
}

fn pt(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 0.01
}

#[test]
fn normal_control_keeps_code_position_and_ppen_id() {
    let got = import(&format!("{}{}", controls(&[]), plain_course()));
    let c = control(&got, "31");
    assert_eq!(c.kind, ControlKind::Control);
    // y up in the file, y down in the project.
    assert_eq!(c.map_mm, pt(30.0, -30.0));
    assert_eq!(c.ppen_id, Some(3));
    assert_eq!(c.description, ControlDescription::default());
    assert_eq!(control(&got, "32").ppen_id, Some(4));
    assert_eq!(course_named(&got, "A").ppen_id, Some(1));
    assert_eq!(got.controls.len(), 2, "start and finish are no controls");
}

#[test]
fn start_and_finish_become_special_objects() {
    let got = import(&format!("{}{}", controls(&[]), plain_course()));
    assert_eq!(
        got.specials,
        vec![
            SpecialObject::Start {
                id: "STA1".to_owned(),
                map_mm: pt(10.0, -10.0),
            },
            SpecialObject::Finish {
                id: "FIN1".to_owned(),
                map_mm: pt(100.0, -10.0),
            },
        ]
    );
    let a = course_named(&got, "A");
    assert_eq!(a.start, "STA1");
    assert_eq!(a.finish, "FIN1");
    let codes: Vec<&str> = course_controls(a)
        .iter()
        .map(|c| c.code.0.as_str())
        .collect();
    assert_eq!(codes, ["31", "32"]);
    assert!(a.classes.is_empty(), "Purple Pen has no classes");
    assert!(got.notes.is_empty(), "{:?}", got.notes);
}

#[test]
fn crossing_point_keeps_kind() {
    let body = format!(
        r#"{}  <control id="5" kind="crossing-point"><location x="45" y="45" /><description box="all" iof-2004-ref="13.3" /></control>
{}"#,
        controls(&[]),
        course(1, "A", 1, &[1, 3, 5, 4, 2], "", &[])
    );
    let got = import(&body);
    let x = control(&got, "CROSS1");
    assert_eq!(x.kind, ControlKind::CrossingPoint);
    assert_eq!(x.map_mm, pt(45.0, -45.0));
    assert_eq!(x.description.c.as_deref(), Some("15.3"));
    let codes: Vec<&str> = course_controls(course_named(&got, "A"))
        .iter()
        .map(|c| c.code.0.as_str())
        .collect();
    assert_eq!(codes, ["31", "CROSS1", "32"]);
}

#[test]
fn course_stops_follow_the_chain_with_bends_and_taped_legs() {
    let legs = r#"  <leg id="1" start-control="1" end-control="3"><flagging kind="all" /></leg>
  <leg id="2" start-control="3" end-control="4"><bends><location x="40" y="35" /><location x="50" y="45" /></bends></leg>
  <leg id="3" start-control="4" end-control="2"><flagging kind="all" /><bends><location x="80" y="20" /></bends></leg>
"#;
    let got = import(&format!("{}{}{legs}", controls(&[]), plain_course()));
    let a = course_named(&got, "A");
    assert!(a.start_taped);
    assert!(a.start_bends.is_empty());
    let ccs = course_controls(a);
    assert_eq!(ccs[0].bends, vec![pt(40.0, -35.0), pt(50.0, -45.0)]);
    assert!(!ccs[0].taped);
    assert_eq!(ccs[1].bends, vec![pt(80.0, -20.0)]);
    assert!(ccs[1].taped);
}

#[test]
fn number_location_becomes_direction_and_gap() {
    // Sprint map: circle 6 mm, line 0.35 mm, so the outer radius is 3.175 mm; the box of a one-digit
    // number is 2.4 by 4 mm. Ratio 1 (print scale = map scale).
    let right = r#"<number-location x="6" y="0" />"#;
    let up = r#"<number-location x="0" y="7" />"#;
    let into = r#"<number-location x="2" y="0" />"#;
    let body = format!(
        "{}{}{}",
        controls(&[]),
        course(
            1,
            "A",
            1,
            &[1, 3, 4, 2],
            r#"<options print-scale="4000" />"#,
            &[(3, right), (4, right)]
        ),
        course(
            2,
            "B",
            11,
            &[1, 3, 4, 2],
            r#"<options print-scale="4000" />"#,
            &[(3, right), (4, up)]
        ),
    );
    let got = import(&body);
    // Both courses put 31 to the east: one central placement.
    let central = control(&got, "31").number.clone().expect("central number");
    assert!(close(central.direction, 0.0), "{central:?}");
    assert!(close(central.gap_mm, 4.8 - 3.175), "{central:?}");
    for name in ["A", "B"] {
        assert_eq!(course_controls(course_named(&got, name))[0].number, None);
    }
    // 32 differs between the courses: one placement per course.
    assert_eq!(control(&got, "32").number, None);
    let a = course_controls(course_named(&got, "A"))[1]
        .number
        .clone()
        .unwrap();
    assert!(close(a.direction, 0.0) && close(a.gap_mm, 1.625), "{a:?}");
    let b = course_controls(course_named(&got, "B"))[1]
        .number
        .clone()
        .unwrap();
    // Up in the file is north on the map: 90 degrees.
    assert!(
        close(b.direction, 90.0) && close(b.gap_mm, 5.0 - 3.175),
        "{b:?}"
    );
    assert!(got.notes.is_empty(), "{:?}", got.notes);

    let inside = import(&format!(
        "{}{}",
        controls(&[]),
        course(
            1,
            "A",
            1,
            &[1, 3, 4, 2],
            r#"<options print-scale="4000" />"#,
            &[(3, into)]
        )
    ));
    let n = control(&inside, "31").number.clone().unwrap();
    assert!(close(n.gap_mm, 0.8 - 3.175), "{n:?}");
    assert_eq!(
        inside.notes,
        vec![ImportNote::NumberIntoCircle {
            control: "31".to_owned()
        }]
    );
}

#[test]
fn description_columns_c_to_h_map_to_iof_2024_ids() {
    let d31 = r#"<description box="C" iof-2004-ref="0.1N" /><description box="D" iof-2004-ref="5.24" /><description box="E" iof-2004-ref="8.4" /><description box="F" iof-2004-ref="9.1high" /><description box="G" iof-2004-ref="11.8NE" /><description box="H" iof-2004-ref="12.4" />"#;
    let d32 = r#"<description box="D" iof-2004-ref="5.2" /><description box="F" iof-2004-ref="10.1single" /><description box="G" iof-2004-ref="11.15" /><description box="H" iof-2004-ref="12.1" />"#;
    let got = import(&format!(
        "{}{}",
        controls(&[(3, d31), (4, d32)]),
        plain_course()
    ));
    let some = |s: &str| Some(s.to_owned());
    assert_eq!(
        control(&got, "31").description,
        ControlDescription {
            c: some("0.1N"),
            d: some("5.22"),
            e: some("8.4"),
            f: some("9.1"),
            g: some("12.7NE"),
            h: some("13.3"),
        }
    );
    // "Crossing (objects of the same kind)": the 2024 standard names both features, so D goes to E too.
    assert_eq!(
        control(&got, "32").description,
        ControlDescription {
            c: None,
            d: some("5.2"),
            e: some("5.2"),
            f: some("10.1"),
            g: some("12.14"),
            h: some("13.1"),
        }
    );
    assert!(got.notes.is_empty(), "{:?}", got.notes);
}

#[test]
fn bend_in_column_g_moves_to_column_f() {
    let d =
        r#"<description box="D" iof-2004-ref="5.2" /><description box="G" iof-2004-ref="11.7" />"#;
    let got = import(&format!("{}{}", controls(&[(3, d)]), plain_course()));
    let desc = &control(&got, "31").description;
    assert_eq!(desc.f.as_deref(), Some("11.1"));
    assert_eq!(desc.g, None);
}

#[test]
fn unmapped_2004_symbol_is_only_noted_not_kept() {
    let d = r#"<description box="D" iof-2004-ref="1.99" /><description box="H" iof-2004-ref="12.3" /><description box="F">5x5|3</description>"#;
    let got = import(&format!("{}{}", controls(&[(3, d)]), plain_course()));
    let desc = &control(&got, "31").description;
    // 2004 12.3 is not 2024 12.3 ("West part"): an id without a 2024 counterpart stays out of the column.
    assert_eq!(desc.d, None);
    assert_eq!(desc.h, None);
    assert_eq!(desc.f, None);
    assert_eq!(
        got.notes,
        vec![
            ImportNote::UnmappedSymbol {
                control: "31".to_owned(),
                column: 'D',
                id: "1.99".to_owned(),
            },
            ImportNote::UnmappedSymbol {
                control: "31".to_owned(),
                column: 'H',
                id: "12.3".to_owned(),
            },
            ImportNote::ColumnFText {
                control: "31".to_owned(),
                text: "5x5|3".to_owned(),
            },
        ]
    );
}

#[test]
fn climb_print_scale_and_secondary_title() {
    let body = format!(
        "{}{}",
        controls(&[]),
        course(
            1,
            "A",
            1,
            &[1, 3, 4, 2],
            r#"<secondary-title>Men|Women</secondary-title><options print-scale="5000" climb="35.4" course-length="2100" />"#,
            &[]
        )
    );
    let got = import(&body);
    let a = course_named(&got, "A");
    assert_eq!(a.climb_m, Some(35));
    assert_eq!(a.planned_print_scale, Some(5000));
    assert_eq!(
        got.notes,
        vec![ImportNote::SecondaryTitle {
            course: "A".to_owned(),
            title: "Men|Women".to_owned(),
        }]
    );
}

#[test]
fn special_objects_map_to_their_counterparts() {
    let specials = r#"  <special-object id="1" kind="out-of-bounds"><location x="10" y="10" /><location x="20" y="10" /><location x="20" y="20" /><courses all="true" /></special-object>
  <special-object id="2" kind="dangerous-area"><location x="30" y="10" /><location x="40" y="10" /><location x="40" y="20" /><courses><course course="2" /></courses></special-object>
  <special-object id="3" kind="forbidden-route"><location x="50" y="50" /></special-object>
  <special-object id="4" kind="first-aid"><location x="70" y="70" /><courses><course course="1" /><course course="2" /></courses></special-object>
  <special-object id="5" kind="water"><location x="80" y="80" /></special-object>
  <special-object id="6" kind="boundary"><location x="1" y="1" /><location x="2" y="2" /></special-object>
  <special-object id="7" kind="line"><location x="1" y="1" /><location x="2" y="2" /></special-object>
  <special-object id="8" kind="text"><text>Hi</text><location x="1" y="5" /><location x="5" y="1" /></special-object>
  <special-object id="9" kind="white-out"><location x="1" y="1" /><location x="5" y="1" /><location x="5" y="5" /></special-object>
"#;
    let body = format!(
        "{}{}{}{specials}",
        controls(&[]),
        plain_course(),
        course(2, "B", 11, &[1, 4, 2], "", &[])
    );
    let got = import(&body);
    let names = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        got.specials[2..],
        [
            SpecialObject::OutOfBoundsArea {
                id: "OOB1".to_owned(),
                polygon: vec![pt(10.0, -10.0), pt(20.0, -10.0), pt(20.0, -20.0)],
                courses: vec![],
            },
            SpecialObject::DangerousArea {
                id: "DANGER1".to_owned(),
                polygon: vec![pt(30.0, -10.0), pt(40.0, -10.0), pt(40.0, -20.0)],
                courses: names(&["B"]),
            },
            SpecialObject::ForbiddenRoute {
                id: "FORBID1".to_owned(),
                points: vec![pt(50.0, -50.0)],
                courses: vec![],
            },
            SpecialObject::FirstAid {
                id: "AID1".to_owned(),
                map_mm: pt(70.0, -70.0),
                courses: names(&["A", "B"]),
            },
            SpecialObject::Water {
                id: "WATER1".to_owned(),
                map_mm: pt(80.0, -80.0),
                courses: vec![],
            },
        ]
    );
    assert_eq!(
        got.notes,
        vec![
            ImportNote::SpecialSkipped { kind: "boundary" },
            ImportNote::SpecialSkipped { kind: "line" },
            ImportNote::SpecialSkipped { kind: "text" },
            ImportNote::SpecialSkipped { kind: "white-out" },
        ]
    );
}

#[test]
fn marked_route_special_becomes_a_marked_route() {
    let raw = read(ppen(&format!("{}{}", controls(&[]), plain_course())).as_bytes()).unwrap();
    let mut fitted = freemapper_course::fit(raw, &sprint_map(), &FitOptions::default())
        .unwrap()
        .event;
    fitted.specials.push(Special::MarkedRoute {
        path: vec![MapCoord::new(1000, -2000), MapCoord::new(3000, -4000)],
        courses: CourseFilter::Only(vec!["A".to_owned()]),
    });
    let got = from_fitted(fitted, MapKind::Sprint);
    assert_eq!(
        got.specials.last(),
        Some(&SpecialObject::MarkedRoute {
            id: "ROUTE1".to_owned(),
            points: vec![pt(1.0, -2.0), pt(3.0, -4.0)],
            courses: vec!["A".to_owned()],
        })
    );
}

#[test]
fn special_of_skipped_courses_only_is_left_out() {
    let specials = r#"  <special-object id="5" kind="water"><location x="80" y="80" /><courses><course course="3" /></courses></special-object>
"#;
    let score = r#"  <course id="3" kind="score" order="3"><name>S</name><first course-control="21" /></course>
  <course-control id="21" control="3" />
"#;
    let got = import(&format!(
        "{}{}{score}{specials}",
        controls(&[]),
        plain_course()
    ));
    assert_eq!(got.specials.len(), 2, "{:?}", got.specials);
    assert_eq!(
        got.notes,
        vec![
            ImportNote::Skipped {
                course: "S".to_owned(),
                why: "score course",
            },
            ImportNote::SpecialWithoutCourse { kind: "water" },
        ]
    );
}

#[test]
fn variation_course_is_skipped_and_named() {
    let body = format!(
        r#"{}{}  <course id="2" kind="normal" order="2"><name>Fork</name><first course-control="11" /></course>
  <course-control id="11" control="1"><next course-control="12" /></course-control>
  <course-control id="12" control="3" variation="A"><next course-control="13" /></course-control>
  <course-control id="13" control="2" />
"#,
        controls(&[]),
        plain_course()
    );
    let got = import(&body);
    assert_eq!(
        got.courses
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>(),
        ["A"]
    );
    assert_eq!(
        got.notes,
        vec![ImportNote::Skipped {
            course: "Fork".to_owned(),
            why: "variations",
        }]
    );
}

#[test]
fn course_without_start_or_finish_is_skipped() {
    let body = format!(
        "{}{}{}",
        controls(&[]),
        course(1, "NoStart", 1, &[3, 4, 2], "", &[]),
        course(2, "NoFinish", 11, &[1, 3, 4], "", &[])
    );
    let got = import(&body);
    assert!(got.courses.is_empty());
    assert_eq!(
        got.notes,
        vec![
            ImportNote::Skipped {
                course: "NoStart".to_owned(),
                why: "no start",
            },
            ImportNote::Skipped {
                course: "NoFinish".to_owned(),
                why: "no finish",
            },
        ]
    );
}

#[test]
fn map_exchange_control_becomes_a_bend() {
    let body = format!(
        r#"{}  <control id="5" kind="map-exchange"><location x="45" y="45" /></control>
{}  <leg id="1" start-control="3" end-control="5"><bends><location x="35" y="40" /></bends></leg>
  <leg id="2" start-control="5" end-control="4"><bends><location x="50" y="55" /></bends></leg>
"#,
        controls(&[]),
        course(1, "A", 1, &[1, 3, 5, 4, 2], "", &[])
    );
    let got = import(&body);
    assert!(got.controls.iter().all(|c| c.code.0 != "XCHG1"));
    let ccs = course_controls(course_named(&got, "A"));
    assert_eq!(ccs.len(), 2);
    assert_eq!(
        ccs[0].bends,
        vec![pt(35.0, -40.0), pt(45.0, -45.0), pt(50.0, -55.0)]
    );
    // Purple Pen tapes the leg into a map exchange on its own; the leg out is not taped.
    assert!(!ccs[0].taped);
    assert_eq!(
        got.notes,
        vec![ImportNote::MapExchange {
            course: "A".to_owned(),
            control: "XCHG1".to_owned(),
        }]
    );
}

#[test]
fn circle_gaps_are_noted_per_course() {
    let gap = r#"<circle-gaps scale="4000">10:20</circle-gaps>"#;
    let leg_gap = r#"  <leg id="1" start-control="3" end-control="4"><gaps><gap start="1" length="2" /></gaps></leg>
"#;
    let body = format!(
        "{}{}{}{leg_gap}",
        controls(&[(4, gap)]),
        course(
            1,
            "A",
            1,
            &[1, 4, 2],
            r#"<options print-scale="4000" />"#,
            &[]
        ),
        course(
            2,
            "B",
            11,
            &[1, 3, 4, 2],
            r#"<options print-scale="10000" />"#,
            &[]
        ),
    );
    let got = import(&body);
    assert_eq!(got.courses.len(), 2);
    let codes = |name: &str| -> Vec<String> {
        course_controls(course_named(&got, name))
            .iter()
            .map(|cc| cc.code.0.clone())
            .collect()
    };
    assert_eq!(codes("A"), ["32"]);
    assert_eq!(codes("B"), ["31", "32"]);
    assert_eq!(control(&got, "32").map_mm, pt(60.0, -60.0));
    assert_eq!(
        got.notes,
        vec![
            ImportNote::GapsNotSupported {
                course: "A".to_owned()
            },
            ImportNote::GapsNotSupported {
                course: "B".to_owned()
            },
        ]
    );
}

#[test]
fn scale_mismatch_is_an_error() {
    let file = ppen_at(10000, &format!("{}{}", controls(&[]), plain_course()));
    let err = import_ppen(file.as_bytes(), &sprint_map()).unwrap_err();
    assert!(
        matches!(
            err,
            PpenImportError::Fit(FitError::ScaleMismatch {
                file: 10000,
                map: 4000
            })
        ),
        "{err:?}"
    );
}

#[test]
fn unreadable_file_is_a_read_error() {
    let err = import_ppen(b"<course-scribe-event>", &sprint_map()).unwrap_err();
    assert!(matches!(err, PpenImportError::Read(_)), "{err:?}");
}

#[test]
fn unknown_elements_arrive_as_notes() {
    let body = format!("{}{}  <future-thing />\n", controls(&[]), plain_course());
    let got = import(&body);
    assert_eq!(
        got.notes,
        vec![ImportNote::UnknownElement("future-thing".to_owned())]
    );
}

#[test]
fn a_number_left_to_purple_pen_in_one_course_keeps_the_others_per_course() {
    let right = r#"<number-location x="6" y="0" />"#;
    let body = format!(
        "{}{}{}",
        controls(&[]),
        course(
            1,
            "A",
            1,
            &[1, 3, 4, 2],
            r#"<options print-scale="4000" />"#,
            &[(3, right)]
        ),
        course(2, "B", 11, &[1, 3, 4, 2], "", &[])
    );
    let got = import(&body);
    // Course B leaves 31 to Purple Pen's own placement, so A's placement stays A's.
    assert_eq!(control(&got, "31").number, None);
    let a = course_controls(course_named(&got, "A"))[0].number.clone();
    assert_eq!(
        a,
        Some(NumberPlacement {
            direction: 0.0,
            gap_mm: 1.63,
        })
    );
    assert_eq!(course_controls(course_named(&got, "B"))[0].number, None);
}

/// Every description symbol id of Purple Pen's `symbols.xml` (`201417f9`), columns C to H and the
/// instruction rows.
const PURPLE_PEN_IDS: &str = "0.1N 0.2NE 0.1E 0.2SE 0.1S 0.2SW 0.1W 0.2NW 0.3 0.4 0.5 1.1 1.2 1.3 1.4 1.5 1.6 1.7 1.8 1.9 1.10 1.11 1.12 1.13 1.14 1.15 1.16 2.1 2.2 2.3 2.4 2.5 2.6 2.7 2.8 2.9 2.10 3.1 3.2 3.3 3.4 3.5 3.6 3.7 3.8 3.9 3.10 3.11 4.1 4.2 4.3 4.4 4.5 4.6 4.7 4.8 4.9 4.10 5.1 5.2 5.3 5.4 5.5 5.6 5.7 5.8 5.9 5.10 5.11 5.12 5.13 5.14 5.15 5.16 5.17 5.18 5.19 5.20 5.23 5.24 5.25 5.26 6.1 6.2 8.1 8.2 8.3 8.4 8.5 8.6 8.7 8.8 8.9 8.10 8.11 10.1 10.1single 10.2 10.2single 11.7 9.1high 9.1deep 9.2 9.3high 9.3deep 9.4 11.1N 11.1NE 11.1E 11.1SE 11.1S 11.1SW 11.1W 11.1NW 11.2N 11.2NE 11.2E 11.2SE 11.2S 11.2SW 11.2W 11.2NW 11.3N 11.3NE 11.3E 11.3SE 11.3S 11.3SW 11.3W 11.3NW 11.4N 11.4NE 11.4E 11.4SE 11.4S 11.4SW 11.4W 11.4NW 11.5N 11.5NE 11.5E 11.5SE 11.5S 11.5SW 11.5W 11.5NW 11.6N 11.6NE 11.6E 11.6SE 11.6S 11.6SW 11.6W 11.6NW 11.8N 11.8NE 11.8E 11.8SE 11.8S 11.8SW 11.8W 11.8NW 11.14N 11.14NE 11.14E 11.14SE 11.14S 11.14SW 11.14W 11.14NW 11.9 11.10 11.11 11.12 11.13 11.15 11.15single 12.1 12.2 12.3 12.4 13.1 13.2 13.3 13.4 13.5 13.5control 15.6 13.6 14.1 14.2 14.3";

#[test]
fn every_purple_pen_id_has_a_2024_number_but_radio_control_and_map_issue() {
    let unmapped: Vec<&str> = PURPLE_PEN_IDS
        .split(' ')
        .filter(|id| iof_2024(id).is_none())
        .collect();
    assert_eq!(unmapped, ["12.3", "13.6"]);
    let to = |id: &str| iof_2024(id).map(|m| m.id);
    for (from, want) in [
        ("1.10", "1.10"),
        ("2.10", "2.10"),
        ("5.20", "5.20"),
        ("5.23", "5.21"),
        ("5.26", "5.24"),
        ("9.3deep", "9.3"),
        ("10.2single", "10.2"),
        ("11.7", "11.1"),
        ("11.1N", "12.1N"),
        ("11.6SW", "12.6SW"),
        ("11.8W", "12.7W"),
        ("11.9", "12.8"),
        ("11.10", "12.9"),
        ("11.11", "12.10"),
        ("11.13", "12.11"),
        ("11.14NE", "12.12NE"),
        ("11.12", "12.13"),
        ("11.15single", "12.14"),
        ("12.2", "13.2"),
        ("12.4", "13.3"),
        ("13.3", "15.3"),
        ("13.5control", "15.5"),
        ("15.6", "15.6"),
        ("14.3", "16.3"),
    ] {
        assert_eq!(to(from).as_deref(), Some(want), "{from}");
    }
    for nonsense in ["", "1", "1.17", "11.1", "1.1N", "x.1", "11.7NE", "9.1wide"] {
        assert_eq!(to(nonsense), None, "{nonsense}");
    }
}

#[test]
fn unused_control_without_a_position_is_dropped_with_a_note() {
    let lost = r#"  <control id="9" kind="normal"><code>77</code></control>
"#;
    let got = import(&format!("{}{lost}{}", controls(&[]), plain_course()));
    assert!(got.controls.iter().all(|c| c.code.0 != "77"));
    assert!(
        got.notes.contains(&ImportNote::ControlWithoutPosition {
            control: "77".to_owned()
        }),
        "{:?}",
        got.notes
    );
}

#[test]
fn a_far_number_gap_is_clamped_to_the_maximum_and_noted() {
    let far = r#"<number-location x="90" y="0" />"#;
    let got = import(&format!(
        "{}{}",
        controls(&[]),
        course(
            1,
            "A",
            1,
            &[1, 3, 4, 2],
            r#"<options print-scale="4000" />"#,
            &[(3, far)]
        )
    ));
    let n = control(&got, "31").number.clone().unwrap();
    assert!(close(n.gap_mm, crate::numbers::MAX_GAP_MM), "{n:?}");
    assert_eq!(
        got.notes,
        vec![ImportNote::NumberGapClamped {
            control: "31".to_owned()
        }]
    );
}

#[test]
fn courses_with_one_name_get_distinct_names_and_the_special_notes_it() {
    let water = r#"  <special-object id="5" kind="water"><location x="80" y="80" /><courses><course course="2" /></courses></special-object>
"#;
    let second = course(2, "A", 11, &[1, 4, 2], "", &[]);
    let got = import(&format!(
        "{}{}{second}{water}",
        controls(&[]),
        plain_course()
    ));
    let names: Vec<&str> = got.courses.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["A", "A (2)"]);
    let SpecialObject::Water { courses, .. } = got.specials.last().unwrap() else {
        panic!("{:?}", got.specials);
    };
    // The file names the course twice as "A", so the special goes to both.
    assert_eq!(courses, &["A".to_owned(), "A (2)".to_owned()]);
    assert!(got.notes.contains(&ImportNote::CourseRenamed {
        from: "A".to_owned(),
        to: "A (2)".to_owned()
    }));
    assert!(got.notes.contains(&ImportNote::SpecialCourseAmbiguous {
        kind: "water",
        name: "A".to_owned()
    }));
}

/// `count` controls with codes 100 and up, all at (30, 30), plus start 1 and finish 2.
fn many_controls(count: u32) -> String {
    let mut out = String::from(
        r#"  <control id="1" kind="start"><location x="10" y="10" /></control>
  <control id="2" kind="finish"><location x="100" y="10" /></control>
"#,
    );
    for i in 0..count {
        out.push_str(&format!(
            r#"  <control id="{id}" kind="normal"><code>{code}</code><location x="30" y="30" /></control>
"#,
            id = 100 + i,
            code = 100 + i
        ));
    }
    out
}

#[test]
fn many_clamped_numbers_give_one_note_per_control_and_stay_fast() {
    let (controls_n, courses_n) = (400_u32, 5_u32);
    let far = r#"<number-location x="90" y="0" />"#;
    let chain: Vec<u32> = std::iter::once(1)
        .chain(100..100 + controls_n)
        .chain(std::iter::once(2))
        .collect();
    let extras: Vec<(u32, &str)> = (100..100 + controls_n).map(|c| (c, far)).collect();
    let mut body = many_controls(controls_n);
    for k in 0..courses_n {
        body.push_str(&course(
            k + 1,
            &format!("C{k}"),
            1000 * (k + 1),
            &chain,
            r#"<options print-scale="4000" />"#,
            &extras,
        ));
    }
    let started = std::time::Instant::now();
    let got = import(&body);
    assert!(started.elapsed().as_secs() < 20, "{:?}", started.elapsed());
    assert_eq!(got.courses.len(), courses_n as usize);
    let clamped = got
        .notes
        .iter()
        .filter(|n| matches!(n, ImportNote::NumberGapClamped { .. }))
        .count();
    assert_eq!(clamped, controls_n as usize);
}

#[test]
fn a_special_with_a_point_off_the_map_is_left_out_and_noted() {
    let specials = r#"  <special-object id="5" kind="water"><location x="500" y="500" /><courses all="true" /></special-object>
  <special-object id="6" kind="first-aid"><location x="80" y="80" /><courses all="true" /></special-object>
  <special-object id="7" kind="out-of-bounds"><location x="10" y="10" /><location x="20" y="10" /><location x="-900" y="20" /><courses all="true" /></special-object>
"#;
    let got = import(&format!("{}{}{specials}", controls(&[]), plain_course()));
    let kinds: Vec<&SpecialObject> = got.specials.iter().skip(2).collect();
    assert_eq!(kinds.len(), 1, "{kinds:?}");
    assert!(matches!(kinds[0], SpecialObject::FirstAid { .. }));
    assert_eq!(
        got.notes,
        vec![
            ImportNote::SpecialOffMap { kind: "water" },
            ImportNote::SpecialOffMap {
                kind: "out-of-bounds"
            },
        ]
    );
}

#[test]
fn a_purple_pen_4_course_with_unknown_attribute_and_element_imports_with_notes() {
    let body = format!(
        r#"{}  <course id="1" kind="normal" order="1" future-flag="yes"><name>A</name><first course-control="1" /><options print-scale="4000" future-option="3" /><future-child detail="x" /></course>
  <course-control id="1" control="1"><next course-control="2" /></course-control>
  <course-control id="2" control="3"><next course-control="3" /></course-control>
  <course-control id="3" control="2" />
"#,
        controls(&[])
    );
    let got = import(&body);
    assert_eq!(got.courses.len(), 1);
    assert_eq!(course_controls(&got.courses[0]).len(), 1);
    assert!(
        got.notes
            .iter()
            .any(|n| matches!(n, ImportNote::UnknownElement(_))),
        "{:?}",
        got.notes
    );
}
