use super::*;
use freemapper_courseplan_model::{
    Class, ControlCode, ControlDescription, CourseControl, EventMeta, FORMAT_VERSION, ProjectFile,
    Sources,
};
use quick_xml::Reader;
use quick_xml::events::Event as Ev;

const CREATED: &str = "2026-11-09T12:00:00Z";

fn p(x: f64, y: f64) -> MapPoint {
    MapPoint { x, y }
}

fn ctrl(code: &str, x: f64, y: f64, kind: ControlKind) -> Control {
    Control {
        ppen_id: None,
        code: ControlCode(code.to_owned()),
        kind,
        map_mm: p(x, y),
        description: ControlDescription::default(),
        number: None,
    }
}

fn cc(code: &str) -> CourseControl {
    CourseControl {
        code: ControlCode(code.to_owned()),
        number: None,
        bends: vec![],
        taped: false,
    }
}

fn course(name: &str, kind: CourseKind) -> Course {
    Course {
        ppen_id: None,
        name: name.to_owned(),
        classes: vec![],
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

fn individual(name: &str, codes: &[&str]) -> Course {
    course(
        name,
        CourseKind::Individual {
            controls: codes.iter().map(|c| cc(c)).collect(),
        },
    )
}

/// Start at (0,0), finish at (0,100), controls 31 (100,0), 32 (100,100), crossing point 33.
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
        classes: vec![],
        controls: vec![
            ctrl("31", 10.0, 20.0, ControlKind::Control),
            ctrl("32", 100.0, 100.0, ControlKind::Control),
            ctrl("33", 100.0, 50.0, ControlKind::CrossingPoint),
        ],
        specials: vec![
            SpecialObject::Start {
                id: "S1".to_owned(),
                ppen_id: None,
                map_mm: p(0.0, 0.0),
            },
            SpecialObject::Finish {
                id: "F1".to_owned(),
                ppen_id: None,
                map_mm: p(0.0, 100.0),
            },
        ],
        courses: vec![],
        accepted_findings: vec![],
    }
}

fn export(e: &Event) -> String {
    export_iof_xml(e, 4000, MapKind::Sprint, None, CREATED)
        .unwrap()
        .xml
}

fn count(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

/// Parses the text and returns every element name, failing on malformed XML.
fn parse_names(xml: &str) -> Vec<String> {
    let mut reader = Reader::from_str(xml);
    let mut names = Vec::new();
    loop {
        match reader.read_event().unwrap() {
            Ev::Start(e) | Ev::Empty(e) => {
                names.push(e.name().as_ref().to_owned());
            }
            Ev::Eof => break,
            _ => {}
        }
    }
    names
}

/// Text content of the first element with this name, entity references resolved.
fn first_text(xml: &str, name: &str) -> String {
    nth_text(xml, name, 0)
}

/// Text content of the `n`th (from 0) element with this name, entity references resolved.
fn nth_text(xml: &str, name: &str, n: usize) -> String {
    let mut reader = Reader::from_str(xml);
    let mut inside = false;
    let mut seen = 0;
    let mut text = String::new();
    loop {
        match reader.read_event().unwrap() {
            Ev::Start(e) if !inside && e.name().as_ref() == name => {
                if seen == n {
                    inside = true;
                }
                seen += 1;
            }
            Ev::Text(t) if inside => text.push_str(&t.xml10_content()),
            Ev::GeneralRef(r) if inside => {
                let r = r.xml10_content();
                text.push_str(match &*r {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    other => panic!("entity {other}"),
                });
            }
            Ev::End(_) if inside => return text,
            Ev::Eof => panic!("no element {name}"),
            _ => {}
        }
    }
}

/// The attribute value `name` of the first element called `element`.
fn attr(xml: &str, element: &str, name: &str) -> String {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event().unwrap() {
            Ev::Start(e) | Ev::Empty(e) if e.name().as_ref() == element => {
                let a = e
                    .try_get_attribute(name)
                    .unwrap()
                    .unwrap_or_else(|| panic!("no attribute {name}"));
                return a.value.into_owned();
            }
            Ev::Eof => panic!("no element {element}"),
            _ => {}
        }
    }
}

#[test]
fn root_has_namespace_version_and_creator() {
    let xml = export(&event());
    assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert_eq!(parse_names(&xml)[0], "CourseData");
    assert_eq!(
        attr(&xml, "CourseData", "xmlns"),
        "http://www.orienteering.org/datastandard/3.0"
    );
    assert_eq!(attr(&xml, "CourseData", "iofVersion"), "3.0");
    assert_eq!(attr(&xml, "CourseData", "createTime"), CREATED);
    assert_eq!(attr(&xml, "CourseData", "creator"), "freemapper-courseplan");
}

#[test]
fn controls_are_written_once_with_y_unmirrored() {
    let mut e = event();
    e.courses = vec![individual("A", &["31", "32"]), individual("B", &["31"])];
    let xml = export(&e);
    assert!(xml.contains("<MapPosition x=\"10\" y=\"20\" unit=\"mm\"/>"));
    assert_eq!(count(&xml, "<Id>31</Id>"), 1);
    // Both courses refer to the control.
    assert_eq!(count(&xml, "<Control>31</Control>"), 2);
    assert_eq!(count(&xml, "<Control type="), 5);
}

#[test]
fn course_lists_start_controls_finish_with_leg_lengths() {
    let mut e = event();
    e.controls = vec![
        ctrl("31", 100.0, 0.0, ControlKind::Control),
        ctrl("32", 100.0, 100.0, ControlKind::Control),
    ];
    e.courses = vec![individual("A", &["31", "32"])];
    let xml = export(&e);
    assert!(xml.contains("<Length>1200</Length>"));
    let start = xml.find("<CourseControl type=\"Start\">").unwrap();
    let first = xml.find("<CourseControl type=\"Control\">").unwrap();
    let finish = xml.find("<CourseControl type=\"Finish\">").unwrap();
    assert!(start < first && first < finish);
    // The start has no leg; the three legs measure 400 m, 400 m and 400 m.
    assert!(!xml[start..first].contains("LegLength"));
    assert_eq!(count(&xml, "<LegLength>400</LegLength>"), 3);
}

#[test]
fn number_position_becomes_map_text_position() {
    let mut e = event();
    e.controls[0].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 0.5,
    });
    e.courses = vec![individual("A", &["31", "32"])];
    // Sprint at 1:4000: ring radius 3.175 + gap 0.5, box 2.4 wide, so the centre sits 4.875 east.
    let xml = export(&e);
    assert!(
        xml.contains("<MapTextPosition x=\"4.88\" y=\"0\" unit=\"mm\"/>"),
        "{xml}"
    );
    assert!(xml.contains("<MapText>1</MapText>"));
    // The second control has no placement, so no position.
    assert_eq!(count(&xml, "<MapTextPosition"), 1);

    // A course override wins: north, so the centre is above the control (y points down: negative).
    if let CourseKind::Individual { controls } = &mut e.courses[0].kind {
        controls[0].number = Some(NumberPlacement {
            direction: 90.0,
            gap_mm: 0.5,
        });
    }
    let xml = export(&e);
    // 3.675 + half box height 2 = 5.675
    assert!(
        xml.contains("<MapTextPosition x=\"0\" y=\"-5.68\" unit=\"mm\"/>"),
        "{xml}"
    );
}

#[test]
fn crossing_point_has_type_and_no_number() {
    let mut e = event();
    e.controls[2].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 0.5,
    });
    e.courses = vec![individual("A", &["31", "33", "32"])];
    let xml = export(&e);
    assert!(xml.contains("<Control type=\"CrossingPoint\">"));
    assert!(xml.contains("<CourseControl type=\"CrossingPoint\">"));
    assert!(!xml.contains("<MapText"));
    assert_eq!(count(&xml, "<MapText>"), 0);
}

#[test]
fn climb_is_written_when_set() {
    let mut e = event();
    let mut a = individual("A", &["31"]);
    a.climb_m = Some(12);
    e.courses = vec![a, individual("B", &["31"])];
    let xml = export(&e);
    assert_eq!(count(&xml, "<Climb>12</Climb>"), 1);
}

#[test]
fn classes_map_to_courses() {
    let mut e = event();
    let mut a = individual("A", &["31"]);
    a.classes = vec!["H21".to_owned(), "D21".to_owned()];
    e.classes = vec![
        Class {
            name: "H21".to_owned(),
            rule_category: None,
            expected_starters: None,
        },
        Class {
            name: "D21".to_owned(),
            rule_category: None,
            expected_starters: None,
        },
    ];
    e.courses = vec![a];
    let xml = export(&e);
    assert_eq!(count(&xml, "<ClassCourseAssignment>"), 2);
    assert!(xml.contains("<ClassName>H21</ClassName>"));
    assert!(xml.contains("<CourseName>A</CourseName>"));
    assert!(xml.find("<Course>").unwrap() < xml.find("<ClassCourseAssignment>").unwrap());
}

#[test]
fn names_are_escaped() {
    let mut e = event();
    e.event.title = "A & B <1>".to_owned();
    let mut a = individual("Bahn <A>", &["31"]);
    a.classes = vec!["Anfänger".to_owned()];
    e.courses = vec![a];
    let xml = export(&e);
    assert!(xml.contains("A &amp; B &lt;1&gt;"));
    parse_names(&xml);
    assert_eq!(first_text(&xml, "Name"), "A & B <1>");
    assert_eq!(nth_text(&xml, "Name", 1), "Bahn <A>");
    assert_eq!(first_text(&xml, "CourseName"), "Bahn <A>");
    assert_eq!(first_text(&xml, "ClassName"), "Anfänger");
}

#[test]
fn non_individual_courses_are_skipped_and_named() {
    let mut e = event();
    e.courses = vec![
        individual("A", &["31"]),
        course(
            "Team",
            CourseKind::Team {
                team_size: 3,
                mandatory: vec![],
                free: vec![],
            },
        ),
    ];
    let out = export_iof_xml(&e, 4000, MapKind::Sprint, None, CREATED).unwrap();
    assert_eq!(
        out.skipped,
        [SkippedCourse {
            name: "Team".to_owned(),
            kind: "team"
        }]
    );
    assert_eq!(count(&out.xml, "<Course>"), 1);
    assert!(!out.xml.contains("Team"));

    e.courses.remove(0);
    let out = export_iof_xml(&e, 4000, MapKind::Sprint, None, CREATED).unwrap();
    assert_eq!(count(&out.xml, "<Course>"), 0);
    assert_eq!(out.skipped.len(), 1);
}

#[test]
fn unknown_code_names_the_course() {
    let mut e = event();
    e.courses = vec![individual("Lang", &["99"])];
    let err = export_iof_xml(&e, 4000, MapKind::Sprint, None, CREATED).unwrap_err();
    assert!(matches!(&err, IofXmlError::Course { course, .. } if course == "Lang"));
    assert!(err.to_string().starts_with("course Lang: "));
}

#[test]
fn numbers_are_short_and_never_negative_zero() {
    assert_eq!(fmt(-0.001), "0");
    assert_eq!(fmt(1200.0), "1200");
    assert_eq!(fmt(4.875), "4.88");
    assert_eq!(fmt(-20.5), "-20.5");
}

fn luebeck_grid() -> freemapper_course::MapGrid {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/maps/luebeck.omap"
    );
    let doc = crate::omap::read::read_map_file(std::path::Path::new(path)).unwrap();
    freemapper_course::MapGrid::of(&doc.georef).expect("the Lübeck map is georeferenced")
}

fn luebeck_xml() -> String {
    let grid = luebeck_grid();
    export_iof_xml(&luebeck(), 4000, MapKind::Sprint, Some(&grid), CREATED)
        .unwrap()
        .xml
}

fn luebeck() -> Event {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/luebeck-sprint/luebeck-sprint.courseplan.toml"
    );
    ProjectFile::load(std::path::Path::new(path)).unwrap().event
}

#[test]
fn output_is_deterministic_and_matches_snapshot() {
    let e = luebeck();
    let grid = luebeck_grid();
    let write = || {
        export_iof_xml(&e, 4000, MapKind::Sprint, Some(&grid), CREATED)
            .unwrap()
            .xml
    };
    let xml = write();
    assert_eq!(xml, write());
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/snapshots/iof_xml_luebeck.xml"
    );
    let example = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/luebeck-sprint/course-data.xml"
    );
    if std::env::var_os("UPDATE_SNAPSHOT").is_some() {
        std::fs::write(path, &xml).unwrap();
        std::fs::write(example, &xml).unwrap();
    }
    let expected = std::fs::read_to_string(path).unwrap();
    assert!(
        xml == expected,
        "the IOF XML of the Lübeck example changed; rerun with UPDATE_SNAPSHOT=1 if intended"
    );
    let committed = std::fs::read_to_string(example).unwrap();
    assert!(
        xml == committed,
        "examples/luebeck-sprint/course-data.xml is stale; rerun with UPDATE_SNAPSHOT=1"
    );
}

/// Validates `xml` against the IOF schema with `xmllint`; skipped with a note where it is missing.
fn assert_valid(xml: &str) {
    let schema = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/iof-xml/IOF.xsd"
    );
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("out.xml");
    std::fs::write(&file, xml).unwrap();
    match std::process::Command::new("xmllint")
        .args(["--noout", "--schema", schema])
        .arg(&file)
        .output()
    {
        Ok(out) => assert!(
            out.status.success(),
            "xmllint: {}",
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(_) => eprintln!("note: xmllint not on PATH, schema validation skipped"),
    }
}

#[test]
fn output_validates_against_schema() {
    let mut e = luebeck();
    e.courses.push(individual("Leer", &[]));
    e.courses[1].start = "S1".to_owned();
    e.courses[1].finish = "F1".to_owned();
    let grid = luebeck_grid();
    let with_geo = export_iof_xml(&e, 4000, MapKind::Sprint, Some(&grid), CREATED)
        .unwrap()
        .xml;
    assert!(with_geo.contains("<Position "));
    assert_valid(&with_geo);
    assert_valid(&export(&e));
}

#[test]
fn crossing_point_event_validates_against_schema() {
    let mut e = event();
    e.controls[0].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 0.5,
    });
    e.courses = vec![individual("A", &["31", "33", "32"])];
    let xml = export(&e);
    assert!(xml.contains("CrossingPoint"));
    assert_valid(&xml);
}

#[test]
fn event_with_only_skipped_courses_validates_against_schema() {
    let mut e = event();
    e.courses = vec![course(
        "Team",
        CourseKind::Team {
            team_size: 3,
            mandatory: vec![],
            free: vec![],
        },
    )];
    let xml = export(&e);
    assert_eq!(count(&xml, "<Course>"), 0);
    assert_valid(&xml);
}

// Read-back through freemapper-course's IOF XML reader (freemapper-print's). The reader keeps the
// positions as the file gives them, so they are compared in file coordinates.

fn kind_of(event: &Event, code: &str) -> freemapper_course::ControlKind {
    use freemapper_course::ControlKind as K;
    if event
        .specials
        .iter()
        .any(|s| matches!(s, SpecialObject::Start { id, .. } if id == code))
    {
        return K::Start;
    }
    if event
        .specials
        .iter()
        .any(|s| matches!(s, SpecialObject::Finish { id, .. } if id == code))
    {
        return K::Finish;
    }
    match event
        .controls
        .iter()
        .find(|c| c.code.0 == code)
        .unwrap()
        .kind
    {
        ControlKind::Control => K::Control,
        ControlKind::CrossingPoint => K::CrossingPoint,
    }
}

fn assert_reads_back(e: &Event, map_scale: u32, map_kind: MapKind) {
    let xml = export_iof_xml(e, map_scale, map_kind, None, CREATED)
        .unwrap()
        .xml;
    let raw =
        freemapper_course::read_iof_xml(xml.as_bytes()).expect("the reader accepts our export");
    assert_eq!(
        raw.source.event_name.as_deref(),
        Some(e.event.title.as_str())
    );
    assert_eq!(raw.source.map_scale, Some(map_scale));

    // Every control and every start and finish, once, at the exported position (y not mirrored).
    let mut expected: Vec<(String, MapPoint, freemapper_course::ControlKind)> = Vec::new();
    for s in &e.specials {
        if let Some((_, id, at)) = start_or_finish(s) {
            expected.push((id.to_owned(), at.clone(), kind_of(e, id)));
        }
    }
    for c in &e.controls {
        expected.push((c.code.0.clone(), c.map_mm.clone(), kind_of(e, &c.code.0)));
    }
    assert_eq!(raw.controls.len(), expected.len());
    for (code, at, kind) in &expected {
        let c = raw
            .controls
            .iter()
            .find(|c| &c.code == code)
            .unwrap_or_else(|| panic!("control {code} missing"));
        let pos = c
            .position
            .unwrap_or_else(|| panic!("control {code} has no position"));
        assert!(
            (pos.x_mm - at.x).abs() <= 0.01,
            "{code}: x {} vs {}",
            pos.x_mm,
            at.x
        );
        assert!(
            (pos.y_mm - at.y).abs() <= 0.01,
            "{code}: y {} vs {}",
            pos.y_mm,
            at.y
        );
        assert_eq!(c.kind, *kind, "{code}");
    }

    let individual: Vec<&Course> = e
        .courses
        .iter()
        .filter(|c| matches!(c.kind, CourseKind::Individual { .. }))
        .collect();
    assert_eq!(raw.courses.len(), individual.len());
    for (course, back) in individual.iter().zip(&raw.courses) {
        let CourseKind::Individual { controls } = &course.kind else {
            unreachable!()
        };
        assert_eq!(back.name, course.name);
        let codes: Vec<&str> = std::iter::once(course.start.as_str())
            .chain(controls.iter().map(|c| c.code.0.as_str()))
            .chain(std::iter::once(course.finish.as_str()))
            .collect();
        let got: Vec<&str> = back.stops.iter().map(|s| s.control.as_str()).collect();
        assert_eq!(got, codes, "{}: order of stops", course.name);
        assert_eq!(
            back.legs.len(),
            codes.len() - 1,
            "{}: leg count",
            course.name
        );
        let want_len = course_length_m(e, course, map_scale).unwrap();
        let got_len = back.length_m.expect("length");
        assert!(
            (got_len - want_len).abs() <= 0.01,
            "{}: length",
            course.name
        );
        if let Some(climb) = course.climb_m {
            assert_eq!(back.climb_m, Some(f64::from(climb)));
        }

        let numbers = control_numbers(e, course);
        for (i, cc) in controls.iter().enumerate() {
            let stop = &back.stops[i + 1];
            let control = e.controls.iter().find(|c| c.code == cc.code).unwrap();
            let placement = cc.number.as_ref().or(control.number.as_ref());
            let number = numbers.get(i).copied().flatten();
            match number.zip(placement) {
                Some((n, p)) => {
                    let centre = number_centre(&control.map_mm, p, n, map_kind, map_scale);
                    let at = stop.number_text_at.unwrap_or_else(|| {
                        panic!("{}: stop {} has no number offset", course.name, cc.code.0)
                    });
                    assert!((at.dx_mm - (centre.x - control.map_mm.x)).abs() <= 0.01);
                    assert!((at.dy_mm - (centre.y - control.map_mm.y)).abs() <= 0.01);
                }
                None => assert!(
                    stop.number_text_at.is_none(),
                    "{}: {}",
                    course.name,
                    cc.code.0
                ),
            }
        }
        for (stop, code) in back.stops.iter().zip(&codes) {
            // The reader sets a stop kind only where the course overrides it, or none.
            if let Some(k) = stop.kind {
                assert_eq!(k, kind_of(e, code), "{}: {code}", course.name);
            }
        }
    }
}

#[test]
fn luebeck_export_reads_back() {
    let e = luebeck();
    assert!(!e.courses.is_empty());
    assert!(e.controls.iter().any(|c| c.number.is_some()) || e.courses.iter().any(|c| matches!(&c.kind, CourseKind::Individual { controls } if controls.iter().any(|cc| cc.number.is_some()))), "the example places at least one number");
    assert_reads_back(&e, 4000, MapKind::Sprint);
}

#[test]
fn crossing_point_export_reads_back() {
    let mut e = event();
    e.controls[0].number = Some(NumberPlacement {
        direction: 0.0,
        gap_mm: 0.5,
    });
    e.controls[1].number = Some(NumberPlacement {
        direction: 135.0,
        gap_mm: 0.3,
    });
    e.courses = vec![
        individual("A", &["31", "33", "32"]),
        individual("B", &["32", "31"]),
    ];
    assert_reads_back(&e, 4000, MapKind::Sprint);
    let xml = export(&e);
    let raw = freemapper_course::read_iof_xml(xml.as_bytes()).unwrap();
    let a = &raw.courses[0];
    assert_eq!(a.stops[2].control, "33");
    assert!(a.stops[2].number_text_at.is_none());
    let c33 = raw.controls.iter().find(|c| c.code == "33").unwrap();
    assert_eq!(c33.kind, freemapper_course::ControlKind::CrossingPoint);
}

#[test]
fn only_non_individual_courses_read_back_as_zero_courses() {
    let mut e = event();
    e.courses = vec![course(
        "Team",
        CourseKind::Team {
            team_size: 3,
            mandatory: vec![],
            free: vec![],
        },
    )];
    let raw = freemapper_course::read_iof_xml(export(&e).as_bytes()).unwrap();
    assert!(raw.courses.is_empty());
}

#[test]
fn controls_carry_a_position_when_the_map_is_georeferenced() {
    let xml = luebeck_xml();
    let lines: Vec<&str> = xml.lines().map(str::trim).collect();
    let controls: Vec<usize> = (0..lines.len())
        .filter(|&i| lines[i].starts_with("<Control type="))
        .collect();
    assert!(controls.len() > 3);
    for i in controls {
        assert!(lines[i + 1].starts_with("<Id>"), "{}", lines[i + 1]);
        let pos = lines[i + 2];
        assert!(pos.starts_with("<Position lat=\""), "{pos}");
        assert!(
            lines[i + 3].starts_with("<MapPosition "),
            "{}",
            lines[i + 3]
        );
        // lat="53.866…" lng="10.68…" with exactly six decimals.
        let values: Vec<&str> = pos.split('"').skip(1).step_by(2).collect();
        assert_eq!(values.len(), 2, "{pos}");
        for v in values {
            assert_eq!(v.split('.').nth(1).map(str::len), Some(6), "{pos}");
        }
    }
    assert_eq!(count(&xml, "<Position "), count(&xml, "<MapPosition "));
}

#[test]
fn geo_positions_read_back_to_the_map_position() {
    let e = luebeck();
    let grid = luebeck_grid();
    let xml = luebeck_xml();
    let raw = freemapper_course::read_iof_xml(xml.as_bytes()).unwrap();
    for c in &e.controls {
        let r = raw.controls.iter().find(|r| r.code == c.code.0).unwrap();
        let (lat, lng) = r.geo.expect("the control has a position");
        let (x, y) = grid.map_mm_of(lat, lng);
        assert!(
            (x - c.map_mm.x).abs() < 0.01 && (y - c.map_mm.y).abs() < 0.01,
            "control {}: ({x}, {y}) against ({}, {})",
            c.code.0,
            c.map_mm.x,
            c.map_mm.y
        );
    }
}

#[test]
fn east_and_north_increase_lng_and_lat() {
    let grid = luebeck_grid();
    let mut e = event();
    // East is x to the right; north is y towards the top of the paper, so y decreases.
    e.controls = vec![
        ctrl("31", 100.0, 100.0, ControlKind::Control),
        ctrl("32", 140.0, 60.0, ControlKind::Control),
    ];
    e.courses = vec![individual("A", &["31", "32"])];
    let xml = export_iof_xml(&e, 4000, MapKind::Sprint, Some(&grid), CREATED)
        .unwrap()
        .xml;
    let raw = freemapper_course::read_iof_xml(xml.as_bytes()).unwrap();
    let geo = |code: &str| {
        raw.controls
            .iter()
            .find(|c| c.code == code)
            .unwrap()
            .geo
            .unwrap()
    };
    let (a, b) = (geo("31"), geo("32"));
    assert!(b.0 > a.0, "lat {a:?} {b:?}");
    assert!(b.1 > a.1, "lng {a:?} {b:?}");
}

#[test]
fn no_position_without_georef() {
    let mut e = event();
    e.courses = vec![individual("A", &["31", "32"])];
    let out = export_iof_xml(&e, 4000, MapKind::Sprint, None, CREATED).unwrap();
    assert!(!out.xml.contains("<Position "));
    assert!(!out.geo_positions);
    let grid = luebeck_grid();
    let out = export_iof_xml(&e, 4000, MapKind::Sprint, Some(&grid), CREATED).unwrap();
    assert!(out.geo_positions);
}
