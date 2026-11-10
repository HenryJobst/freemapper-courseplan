//! Exports projects to Purple Pen events (`.ppen`): onto the three test events of `tests/fixtures/ppen`
//! they were imported from (fitted against frame maps, see `common`), and from scratch for the Lübeck
//! example.

mod common;

use std::path::{Path, PathBuf};

use common::{FIXTURES, bytes, frame_map, valid_event};
use freemapper_course::{KeptObject, RawEvent, read_ppen};
use freemapper_courseplan::omap::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::ppen_export::{
    ExportNote, PpenExport, PpenExportError, Target, export_ppen,
};
use freemapper_courseplan::ppen_import::import_ppen;
use freemapper_courseplan_model::{
    Control, ControlCode, ControlKind, Course, CourseControl, CourseKind, Event, MapPoint,
    NumberPlacement, ProjectFile, SpecialObject,
};

fn target(base: Option<&[u8]>, linked: bool) -> Target<'_> {
    Target {
        base,
        linked,
        map_path: "map.omap".to_owned(),
        lang: "en",
    }
}

/// A fixture imported onto its frame map, as a project.
fn imported(name: &str) -> (Vec<u8>, MapDocument, Event) {
    let file = bytes(name);
    let map = frame_map(&file);
    let import = import_ppen(&file, &map).unwrap_or_else(|e| panic!("{name}: {e}"));
    let event = valid_event(&import);
    (file, map, event)
}

fn export(event: &Event, map: &MapDocument, base: &[u8], linked: bool) -> PpenExport {
    export_ppen(event, map, target(Some(base), linked)).unwrap_or_else(|e| panic!("{e}"))
}

fn read(text: &str) -> RawEvent {
    read_ppen(text.as_bytes()).unwrap()
}

fn ccs(course: &mut Course) -> &mut Vec<CourseControl> {
    match &mut course.kind {
        CourseKind::Individual { controls } => controls,
        other => panic!("{other:?}"),
    }
}

fn course_mut<'a>(event: &'a mut Event, name: &str) -> &'a mut Course {
    event.courses.iter_mut().find(|c| c.name == name).unwrap()
}

/// The lines of `a` that differ from `b`, which must have as many.
fn changed_lines(a: &str, b: &str) -> Vec<String> {
    let (a, b): (Vec<&str>, Vec<&str>) = (a.split('\n').collect(), b.split('\n').collect());
    assert_eq!(a.len(), b.len(), "{}", b.join("\n"));
    a.iter()
        .zip(&b)
        .filter(|(x, y)| x != y)
        .map(|(_, y)| y.to_string())
        .collect()
}

#[test]
fn import_then_export_onto_the_same_file_is_byte_identical() {
    for name in FIXTURES {
        let (file, map, event) = imported(name);
        for linked in [true, false] {
            let out = export(&event, &map, &file, linked);
            assert_eq!(
                out.text.as_bytes(),
                file.as_slice(),
                "{name}, linked {linked}"
            );
            assert_eq!(out.courses, event.courses.len(), "{name}");
        }
    }
}

#[test]
fn the_ids_of_the_base_are_returned() {
    let (file, map, event) = imported("lincoln-park.ppen");
    let out = export(&event, &map, &file, true);
    assert_eq!(
        out.ids.controls,
        vec![(ControlCode("31".into()), 2), (ControlCode("32".into()), 3)]
    );
    assert_eq!(out.ids.courses, vec![("Short".to_owned(), 1)]);
    assert_eq!(
        out.ids.specials,
        vec![("STA1".to_owned(), 1), ("FIN1".to_owned(), 5)]
    );
}

#[test]
fn moved_control_changes_only_that_control() {
    for (name, code) in [
        ("lincoln-park.ppen", "31"),
        ("overlapper.ppen", "38"),
        ("sampleevent8.ppen", "41"),
    ] {
        let (file, map, mut event) = imported(name);
        let c = event
            .controls
            .iter_mut()
            .find(|c| c.code.0 == code)
            .unwrap();
        c.map_mm.x += 1.0;
        for linked in [true, false] {
            let out = export(&event, &map, &file, linked);
            let text = String::from_utf8(file.clone()).unwrap();
            assert_eq!(
                changed_lines(&text, &out.text).len(),
                1,
                "{name}: {}",
                out.text
            );
            let base = read(&text);
            let mut back = read(&out.text);
            let was = base.controls.iter().find(|c| c.code == code).unwrap();
            let now = back.controls.iter_mut().find(|c| c.code == code).unwrap();
            let (p, q) = (was.position.unwrap(), now.position.unwrap());
            assert!((q.x_mm - (p.x_mm + 1.0)).abs() < 0.001, "{name}: {q:?}");
            assert_eq!(q.y_mm as f32, p.y_mm as f32, "{name}");
            now.position = was.position;
            assert_eq!(back, base, "{name}");
        }
    }
}

#[test]
fn changed_course_keeps_what_the_project_does_not_hold() {
    // Lincoln Park: circle gaps, a leg gap and the course's own print scale; sampleevent8: a map
    // exchange control in the course, a map exchange and a map flip at controls, number offsets.
    for (name, course) in [
        ("lincoln-park.ppen", "Short"),
        ("sampleevent8.ppen", "Exchanges"),
        ("overlapper.ppen", "Course 1"),
    ] {
        let (file, map, mut event) = imported(name);
        course_mut(&mut event, course).climb_m = Some(20);
        let out = export(&event, &map, &file, false);
        let text = String::from_utf8(file).unwrap();
        let mut base = read(&text);
        let back = read(&out.text);
        base.courses
            .iter_mut()
            .find(|c| c.name == course)
            .unwrap()
            .climb_m = Some(20.0);
        assert_eq!(back, base, "{name}");
    }
}

#[test]
fn changed_description_keeps_column_f_text() {
    // Control 53 of sampleevent8 has the text "2" in column F.
    let (file, map, mut event) = imported("sampleevent8.ppen");
    let c = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "53")
        .unwrap();
    c.description.h = None;
    let out = export(&event, &map, &file, true);
    let back = read(&out.text);
    let d = back
        .controls
        .iter()
        .find(|c| c.code == "53")
        .unwrap()
        .description
        .clone()
        .unwrap();
    assert_eq!(d.column_f_text.as_deref(), Some("2"));
    assert_eq!(d.columns[5], None);
    assert_eq!(d.standard, "2018");
}

#[test]
fn the_bend_goes_to_box_g_in_a_2004_file() {
    // Lincoln Park states no description standard, which is 2004; 2004 has the bend in box G.
    let (file, map, mut event) = imported("lincoln-park.ppen");
    let c = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "31")
        .unwrap();
    c.description.f = Some("11.1".to_owned());
    let out = export(&event, &map, &file, true);
    let d = read(&out.text)
        .controls
        .into_iter()
        .find(|c| c.code == "31")
        .unwrap()
        .description
        .unwrap();
    assert_eq!(d.standard, "2004");
    assert_eq!(
        (d.columns[3].as_deref(), d.columns[4].as_deref()),
        (None, Some("11.7"))
    );
    let back = import_ppen(out.text.as_bytes(), &map).unwrap();
    let c = back.controls.iter().find(|c| c.code.0 == "31").unwrap();
    assert_eq!(c.description.f.as_deref(), Some("11.1"));
}

#[test]
fn changed_number_rewrites_only_its_offset() {
    // Control 38 is numbered by Course 1 only, centrally.
    let (file, map, mut event) = imported("overlapper.ppen");
    let c = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "38")
        .unwrap();
    let p = c.number.as_mut().unwrap();
    p.direction = (p.direction + 90.0) % 360.0;
    let out = export(&event, &map, &file, true);
    let text = String::from_utf8(file).unwrap();
    let mut base = read(&text);
    let back = read(&out.text);
    let course = |ev: &RawEvent| -> usize {
        ev.courses
            .iter()
            .position(|c| c.name == "Course 1")
            .unwrap()
    };
    let (i, j) = (course(&base), course(&back));
    let at = base.courses[i]
        .stops
        .iter()
        .position(|s| s.control == "38")
        .unwrap();
    assert_ne!(
        back.courses[j].stops[at].number_text_at,
        base.courses[i].stops[at].number_text_at
    );
    base.courses[i].stops[at].number_text_at = back.courses[j].stops[at].number_text_at;
    assert_eq!(back, base);
    // The new placement comes back from the file.
    let again = import_ppen(out.text.as_bytes(), &map).unwrap();
    let c38 = again.controls.iter().find(|c| c.code.0 == "38").unwrap();
    let want = event.controls.iter().find(|c| c.code.0 == "38").unwrap();
    let (x, y) = (c38.number.clone().unwrap(), want.number.clone().unwrap());
    assert!(
        (x.direction - y.direction).abs() <= 0.01 + 1e-9,
        "{x:?} {y:?}"
    );
    assert!((x.gap_mm - y.gap_mm).abs() <= 0.01 + 1e-9, "{x:?} {y:?}");
}

#[test]
fn course_without_print_scale_keeps_purple_pens_default() {
    // Without a print scale Purple Pen prints at 1:15000; the import leaves the planned print scale
    // empty, which the project reads as the map scale. A changed course keeps the file's meaning.
    let text = String::from_utf8(bytes("lincoln-park.ppen")).unwrap();
    let file = text.replace(
        "    <options print-scale=\"5000\" description-kind=\"symbols\" />\r\n",
        "",
    );
    assert_ne!(file, text);
    let map = frame_map(file.as_bytes());
    let import = import_ppen(file.as_bytes(), &map).unwrap();
    let mut event = valid_event(&import);
    assert_eq!(event.courses[0].planned_print_scale, None);
    event.courses[0].climb_m = Some(10);
    let out = export(&event, &map, file.as_bytes(), true);
    let (was, now) = (&read(&file).courses[0], &read(&out.text).courses[0]);
    assert_eq!(now.print_scale_hint.unwrap_or(15000), 15000);
    assert_eq!(now.course_obj_ratio, was.course_obj_ratio);
    assert_eq!(now.climb_m, Some(10.0));
}

#[test]
fn removed_stop_rewrites_only_its_course() {
    let (file, map, mut event) = imported("overlapper.ppen");
    let removed = ccs(course_mut(&mut event, "Course 1")).remove(2).code.0;
    let out = export(&event, &map, &file, true);
    let text = String::from_utf8(file).unwrap();
    let base = read(&text);
    let back = read(&out.text);
    let stops = |ev: &RawEvent| -> Vec<String> {
        let c = ev.courses.iter().find(|c| c.name == "Course 1").unwrap();
        c.stops.iter().map(|s| s.control.clone()).collect()
    };
    let mut expected = stops(&base);
    let at = expected.iter().position(|c| *c == removed).unwrap();
    expected.remove(at);
    assert_eq!(stops(&back), expected);
    // The control stays; only explicit deletions delete.
    assert!(back.controls.iter().any(|c| c.code == removed));
    for name in ["Course 2", "Score"] {
        let a = base.courses.iter().find(|c| c.name == name);
        let b = back.courses.iter().find(|c| c.name == name);
        assert_eq!(a, b, "{name}");
    }
    assert_eq!(
        out.text.matches("<course-control id=").count(),
        text.matches("<course-control id=").count() - 1
    );
}

#[test]
fn deleted_control_is_kept_and_reported() {
    let (file, map, mut event) = imported("overlapper.ppen");
    let course2: Vec<String> = match &event.courses[1].kind {
        CourseKind::Individual { controls } => controls.iter().map(|c| c.code.0.clone()).collect(),
        other => panic!("{other:?}"),
    };
    let course1 = ccs(course_mut(&mut event, "Course 1"));
    let at = course1
        .iter()
        .position(|c| !course2.contains(&c.code.0))
        .unwrap();
    let code = course1.remove(at).code.0;
    event.controls.retain(|c| c.code.0 != code);
    let out = export(&event, &map, &file, true);
    assert!(read(&out.text).controls.iter().any(|c| c.code == code));
    assert!(
        out.notes.iter().any(
            |n| matches!(n, ExportNote::Kept(KeptObject::Control { code: c, .. }) if *c == code)
        ),
        "{:?}",
        out.notes
    );
}

#[test]
fn kept_objects_are_reported() {
    let (file, map, event) = imported("overlapper.ppen");
    let out = export(&event, &map, &file, true);
    assert!(
        out.notes.contains(&ExportNote::Kept(KeptObject::Course {
            id: 3,
            name: "Score".to_owned()
        })),
        "{:?}",
        out.notes
    );
}

#[test]
fn hostile_and_foreign_bases_are_refused_or_kept() {
    let (file, map, event) = imported("lincoln-park.ppen");
    // Not a .ppen at all.
    assert!(matches!(
        export_ppen(&event, &map, target(Some(b"not a course file"), true)),
        Err(PpenExportError::Base(_))
    ));
    // A Purple Pen version the writer does not know.
    let text = String::from_utf8(file.clone()).unwrap();
    let newer = text.replace(
        "<course-scribe-event>",
        "<course-scribe-event version=\"9\">",
    );
    assert!(export_ppen(&event, &map, target(Some(newer.as_bytes()), true)).is_err());
    // An entity bomb.
    let bomb =
        "<!DOCTYPE x [<!ENTITY a \"aaaaaaaaaa\">]><course-scribe-event>&a;</course-scribe-event>";
    assert!(export_ppen(&event, &map, target(Some(bomb.as_bytes()), true)).is_err());
    // The event of another map (1:10000 against this 1:5000 map).
    let other = bytes("overlapper.ppen");
    assert!(matches!(
        export_ppen(&event, &map, target(Some(&other), false)),
        Err(PpenExportError::Base(_))
    ));
    // A file of the same map whose controls have other codes: nothing of it is taken over, all of it
    // is kept.
    let foreign = text
        .replace("<code>31</code>", "<code>91</code>")
        .replace("<code>32</code>", "<code>92</code>");
    let out = export(&event, &map, foreign.as_bytes(), false);
    let back = read(&out.text);
    let codes: Vec<&str> = back.controls.iter().map(|c| c.code.as_str()).collect();
    for code in ["31", "32", "91", "92"] {
        assert!(codes.contains(&code), "{code} in {codes:?}");
    }
    for code in ["91", "92"] {
        assert!(out.notes.iter().any(
            |n| matches!(n, ExportNote::Kept(KeptObject::Control { code: c, .. }) if c == code)
        ));
    }
}

// ---- from scratch: the Lübeck example ----

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The Lübeck example with what the example lacks: bends, a taped leg, a planned print scale on the
/// second course, a course-control placement next to central ones, and special objects.
fn luebeck() -> (Event, MapDocument) {
    let path = repo_root().join("examples/luebeck-sprint/luebeck-sprint.courseplan.toml");
    let mut event = ProjectFile::load(&path).unwrap().event;
    let map = read_map_file(&path.parent().unwrap().join(&event.sources.map)).unwrap();
    let a = &mut event.courses[0];
    a.start_bends = vec![MapPoint { x: -80.0, y: -5.0 }];
    let controls = ccs(a);
    controls[1].bends = vec![
        MapPoint {
            x: -49.5,
            y: -30.25,
        },
        MapPoint { x: -40.0, y: -50.0 },
    ];
    controls[3].taped = true;
    let mut b = event.courses[0].clone();
    b.name = "B".to_owned();
    b.planned_print_scale = Some(5000);
    b.climb_m = None;
    let controls = ccs(&mut b);
    controls.truncate(3);
    controls[1].number = None;
    event.courses.push(b);
    let c32 = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "32")
        .unwrap();
    c32.number = Some(NumberPlacement {
        direction: 45.0,
        gap_mm: 0.8,
    });
    c32.description.d = Some("5.2".to_owned());
    event.controls.push(Control {
        code: ControlCode("CP".to_owned()),
        kind: ControlKind::CrossingPoint,
        map_mm: MapPoint { x: -60.0, y: 20.0 },
        description: Default::default(),
        number: None,
        ppen_id: None,
    });
    ccs(&mut event.courses[0]).insert(
        4,
        CourseControl {
            code: ControlCode("CP".to_owned()),
            number: None,
            bends: vec![],
            taped: false,
        },
    );
    event.specials.extend([
        SpecialObject::Water {
            id: "W1".to_owned(),
            ppen_id: None,
            map_mm: MapPoint { x: -30.0, y: 0.0 },
            courses: vec![],
        },
        SpecialObject::ForbiddenRoute {
            id: "X1".to_owned(),
            ppen_id: None,
            points: vec![MapPoint { x: -20.0, y: -20.0 }],
            courses: vec!["B".to_owned()],
        },
        SpecialObject::OutOfBoundsArea {
            id: "OOB1".to_owned(),
            ppen_id: None,
            polygon: vec![
                MapPoint { x: -10.0, y: -10.0 },
                MapPoint { x: -5.0, y: -10.0 },
                MapPoint { x: -5.0, y: -5.0 },
            ],
            courses: vec![],
        },
    ]);
    assert_eq!(freemapper_courseplan_model::validate(&event), vec![]);
    (event, map)
}

fn near(a: &MapPoint, b: &MapPoint, what: &str) {
    assert!(
        (a.x - b.x).abs() <= 0.001 && (a.y - b.y).abs() <= 0.001,
        "{what}: {a:?} against {b:?}"
    );
}

fn near_path(a: &[MapPoint], b: &[MapPoint], what: &str) {
    assert_eq!(a.len(), b.len(), "{what}");
    for (p, q) in a.iter().zip(b) {
        near(p, q, what);
    }
}

/// The position of a start or finish.
fn special_at<'a>(event: &'a Event, id: &str) -> &'a MapPoint {
    match event.specials.iter().find(|s| s.id() == id).unwrap() {
        SpecialObject::Start { map_mm, .. } | SpecialObject::Finish { map_mm, .. } => map_mm,
        other => panic!("{other:?}"),
    }
}

/// The placement each stop of `course` shows: the course control's own, else the control's.
fn numbers(event: &Event, course: &Course) -> Vec<Option<NumberPlacement>> {
    match &course.kind {
        CourseKind::Individual { controls } => controls
            .iter()
            .map(|cc| {
                let c = event.controls.iter().find(|c| c.code == cc.code).unwrap();
                match c.kind {
                    ControlKind::Control => cc.number.clone().or(c.number.clone()),
                    ControlKind::CrossingPoint => None,
                }
            })
            .collect(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn export_from_scratch_reads_back() {
    let (event, map) = luebeck();
    let out = export_ppen(&event, &map, target(None, true)).unwrap();
    assert_eq!(out.courses, 2);
    // Purple Pen stores no code for starts, finishes and crossing points; that is expected and not noted.
    assert!(
        !out.notes
            .iter()
            .any(|n| matches!(n, ExportNote::CodeNotStored { .. })),
        "{:?}",
        out.notes
    );
    let back = import_ppen(out.text.as_bytes(), &map).unwrap();
    let back = valid_event(&back);

    assert_eq!(back.controls.len(), event.controls.len());
    for c in &event.controls {
        let b = back
            .controls
            .iter()
            .find(|b| b.code.0 == rename(&c.code.0))
            .unwrap_or_else(|| {
                panic!(
                    "{} in {:?}",
                    c.code.0,
                    back.controls
                        .iter()
                        .map(|c| (&c.code.0, c.kind))
                        .collect::<Vec<_>>()
                )
            });
        assert_eq!(b.kind, c.kind, "{}", c.code.0);
        near(&b.map_mm, &c.map_mm, &c.code.0);
        assert_eq!(b.description, c.description, "{}", c.code.0);
    }
    fn rename(id: &str) -> &str {
        match id {
            "S1" => "STA1",
            "F1" => "FIN1",
            "CP" => "CROSS1",
            other => other,
        }
    }
    for id in ["S1", "F1"] {
        near(special_at(&back, rename(id)), special_at(&event, id), id);
    }
    assert_eq!(back.courses.len(), event.courses.len());
    for c in &event.courses {
        let b = back.courses.iter().find(|b| b.name == c.name).unwrap();
        assert_eq!(
            (b.start.as_str(), b.finish.as_str()),
            (rename(&c.start), rename(&c.finish))
        );
        near_path(&b.start_bends, &c.start_bends, &c.name);
        assert_eq!(b.start_taped, c.start_taped, "{}", c.name);
        assert_eq!(b.climb_m, c.climb_m, "{}", c.name);
        // A course without a planned print scale is printed at the map scale, which the file states.
        let scale = map.georef.scale_denominator;
        assert_eq!(
            b.planned_print_scale.unwrap_or(scale),
            c.planned_print_scale.unwrap_or(scale),
            "{}",
            c.name
        );
        let (CourseKind::Individual { controls: bs }, CourseKind::Individual { controls: cs }) =
            (&b.kind, &c.kind)
        else {
            panic!("individual courses");
        };
        assert_eq!(bs.len(), cs.len(), "{}", c.name);
        for (x, y) in bs.iter().zip(cs) {
            assert_eq!(x.code.0, rename(&y.code.0), "{}", c.name);
            near_path(&x.bends, &y.bends, &c.name);
            assert_eq!(x.taped, y.taped, "{} {}", c.name, y.code.0);
        }
        for (n, (x, y)) in numbers(&back, b)
            .into_iter()
            .zip(numbers(&event, c))
            .enumerate()
        {
            match (x, y) {
                (Some(x), Some(y)) => {
                    let what = format!("{} stop {n}", c.name);
                    let turn = (x.direction - y.direction + 540.0) % 360.0 - 180.0;
                    assert!(turn.abs() <= 0.01 + 1e-9, "{what}: {x:?} against {y:?}");
                    assert!(
                        (x.gap_mm - y.gap_mm).abs() <= 0.01 + 1e-9,
                        "{what}: {x:?} against {y:?}"
                    );
                }
                (x, y) => assert_eq!(x, y, "{} stop {n}", c.name),
            }
        }
    }

    let others = |e: &Event| -> Vec<SpecialObject> {
        e.specials
            .iter()
            .filter(|s| {
                !matches!(
                    s,
                    SpecialObject::Start { .. } | SpecialObject::Finish { .. }
                )
            })
            .cloned()
            .collect()
    };
    let (a, b) = (others(&event), others(&back));
    assert_eq!(a.len(), b.len(), "{b:?}");
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.courses(), y.courses(), "{x:?}");
        match (x, y) {
            (SpecialObject::Water { map_mm: p, .. }, SpecialObject::Water { map_mm: q, .. }) => {
                near(p, q, "water")
            }
            (
                SpecialObject::ForbiddenRoute { points: p, .. },
                SpecialObject::ForbiddenRoute { points: q, .. },
            ) => near_path(p, q, "forbidden route"),
            (
                SpecialObject::OutOfBoundsArea { polygon: p, .. },
                SpecialObject::OutOfBoundsArea { polygon: q, .. },
            ) => near_path(p, q, "out of bounds"),
            other => panic!("{other:?}"),
        }
    }
}

#[test]
fn same_project_twice_same_bytes() {
    let (event, map) = luebeck();
    let one = export_ppen(&event, &map, target(None, true)).unwrap();
    let two = export_ppen(&event, &map, target(None, true)).unwrap();
    assert_eq!(one.text, two.text);
    assert_eq!(one.ids, two.ids);
    let (file, map, mut event) = imported("sampleevent8.ppen");
    course_mut(&mut event, "Exchanges").climb_m = Some(7);
    let one = export(&event, &map, &file, true);
    let two = export(&event, &map, &file, true);
    assert_eq!(one.text, two.text);
}

#[test]
fn exporting_onto_its_own_export_changes_nothing() {
    let (mut event, map) = luebeck();
    let first = export_ppen(&event, &map, target(None, true)).unwrap();
    // Before and after the assigned ids are written back into the project, as `export --ppen` does.
    let again = export(&event, &map, first.text.as_bytes(), true);
    assert_eq!(again.text, first.text);
    for (code, id) in &first.ids.controls {
        let c = event.controls.iter_mut().find(|c| c.code == *code).unwrap();
        c.ppen_id = Some(*id);
    }
    for (name, id) in &first.ids.courses {
        event
            .courses
            .iter_mut()
            .find(|c| c.name == *name)
            .unwrap()
            .ppen_id = Some(*id);
    }
    let again = export(&event, &map, first.text.as_bytes(), true);
    assert_eq!(again.text, first.text);
    assert_eq!(again.ids, first.ids);
}

#[test]
fn a_comment_with_a_multibyte_character_does_not_upset_the_patch() {
    // The example has no descriptions, so the patch takes the standard from the base file's element.
    let path = repo_root().join("examples/luebeck-sprint/luebeck-sprint.courseplan.toml");
    let event = ProjectFile::load(&path).unwrap().event;
    let map = read_map_file(&path.parent().unwrap().join(&event.sources.map)).unwrap();
    let first = export_ppen(&event, &map, target(None, true)).unwrap();
    for comment in [
        "<!-- <standards description=\u{e9}> -->",
        "<!-- <standards description=\u{e4}\"x\"> -->",
    ] {
        let hostile = first.text.replacen(
            "<course-scribe-event",
            &format!("{comment}<course-scribe-event"),
            1,
        );
        assert_ne!(hostile, first.text);
        let out = export(&event, &map, hostile.as_bytes(), true);
        assert_eq!(out.text, hostile, "{comment}");
    }
}

#[test]
fn copied_circle_gaps_are_bounded() {
    use freemapper_course::model::{MAX_POINTS_PER_PATH, MAX_TOTAL_POINTS};
    // Control 31 of Lincoln Park with the most circle gaps the reader takes at one control and scale.
    let text = String::from_utf8(bytes("lincoln-park.ppen")).unwrap();
    let many = vec!["1:2"; MAX_POINTS_PER_PATH].join(",");
    let hostile = text.replace(
        "<circle-gaps scale=\"5000\">224.4232:330.1188</circle-gaps>",
        &format!("<circle-gaps scale=\"5000\">{many}</circle-gaps>"),
    );
    assert_ne!(hostile, text);
    let map = frame_map(hostile.as_bytes());
    let import = import_ppen(hostile.as_bytes(), &map).unwrap();
    let mut event = valid_event(&import);
    // A course that visits 31 often enough that its stops would copy more gaps than the reader allows.
    let visits = MAX_TOTAL_POINTS / MAX_POINTS_PER_PATH + 1;
    let mut long = event.courses[0].clone();
    long.name = "Long".to_owned();
    let stops = ccs(&mut long);
    let template = stops[0].clone();
    stops.clear();
    for i in 0..2 * visits {
        let mut cc = template.clone();
        cc.code = ControlCode(if i % 2 == 0 { "31" } else { "32" }.to_owned());
        cc.bends.clear();
        stops.push(cc);
    }
    event.courses.push(long);
    let got = export_ppen(&event, &map, target(Some(hostile.as_bytes()), false));
    assert!(
        matches!(got, Err(PpenExportError::CircleGapLimit { limit }) if limit == MAX_TOTAL_POINTS),
        "{got:?}"
    );
}

#[test]
fn a_pair_visited_twice_keeps_its_map_exchange() {
    // Exchanges runs 47, map exchange 81, 71. The crafted file goes on 71, 47, 71, 77, so the pair 47 to
    // 71 comes twice: once through the map exchange and once straight.
    let text = String::from_utf8(bytes("sampleevent8.ppen")).unwrap();
    let old = "  <course-control id=\"307\" control=\"71\">\r\n    <next course-control=\"308\" />\r\n  </course-control>\r\n";
    let new = concat!(
        "  <course-control id=\"307\" control=\"71\">\r\n    <next course-control=\"317\" />\r\n  </course-control>\r\n",
        "  <course-control id=\"317\" control=\"47\">\r\n    <next course-control=\"318\" />\r\n  </course-control>\r\n",
        "  <course-control id=\"318\" control=\"71\">\r\n    <next course-control=\"308\" />\r\n  </course-control>\r\n",
    );
    assert!(text.contains(old));
    let file = text.replace(old, new);
    let map = frame_map(file.as_bytes());
    let import = import_ppen(file.as_bytes(), &map).unwrap();
    let mut event = valid_event(&import);
    let course = course_mut(&mut event, "Exchanges");
    let codes: Vec<&str> = match &course.kind {
        CourseKind::Individual { controls } => controls.iter().map(|c| c.code.0.as_str()).collect(),
        other => panic!("{other:?}"),
    };
    let at = codes.windows(4).position(|w| w == ["47", "71", "47", "71"]);
    assert!(at.is_some(), "{codes:?}");
    course.climb_m = Some(20);
    let out = export(&event, &map, file.as_bytes(), false);
    let mut base = read(&file);
    base.courses
        .iter_mut()
        .find(|c| c.name == "Exchanges")
        .unwrap()
        .climb_m = Some(20.0);
    assert_eq!(read(&out.text), base);
}

#[test]
fn unlinked_starts_finishes_and_crossing_points_pair_by_their_order() {
    use freemapper_course::ControlKind as PKind;
    // From scratch and without a link the project keeps S1, F1 and CP, while a re-import names them STA1,
    // FIN1 and CROSS1. Neither id nor name pairs them, so the k-th of each kind pairs with the k-th.
    let (mut event, map) = luebeck();
    for s in &mut event.specials {
        if let SpecialObject::Start { ppen_id, .. } | SpecialObject::Finish { ppen_id, .. } = s {
            *ppen_id = None;
        }
    }
    let first = export_ppen(&event, &map, target(None, false)).unwrap();
    let start = event
        .specials
        .iter_mut()
        .find_map(|s| match s {
            SpecialObject::Start { map_mm, .. } => Some(map_mm),
            _ => None,
        })
        .unwrap();
    start.x += 7.4;
    let moved_start = start.clone();
    let cp = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "CP")
        .unwrap();
    cp.map_mm.y += 2.0;
    let moved_cp = cp.map_mm.clone();
    let out = export(&event, &map, first.text.as_bytes(), false);
    let back = read(&out.text);
    let of = |kind| -> Vec<_> { back.controls.iter().filter(|c| c.kind == kind).collect() };
    let starts = of(PKind::Start);
    assert_eq!(starts.len(), 1, "{}", out.text);
    let at = starts[0].position.unwrap();
    assert!((at.x_mm - moved_start.x).abs() < 0.001 && (at.y_mm + moved_start.y).abs() < 0.001);
    assert_eq!(of(PKind::Finish).len(), 1);
    let crossings = of(PKind::CrossingPoint);
    assert_eq!(crossings.len(), 1, "{}", out.text);
    let at = crossings[0].position.unwrap();
    assert!((at.x_mm - moved_cp.x).abs() < 0.001 && (at.y_mm + moved_cp.y).abs() < 0.001);
    assert!(
        !out.notes.iter().any(|n| matches!(n, ExportNote::Kept(_))),
        "{:?}",
        out.notes
    );
    // The ids come back under the project's names.
    assert!(
        out.ids.specials.iter().any(|(id, _)| id == "S1"),
        "{:?}",
        out.ids
    );
    assert!(
        out.ids.controls.iter().any(|(code, _)| code.0 == "CP"),
        "{:?}",
        out.ids
    );
    // Only the moved objects changed.
    let text = first.text.clone();
    assert_eq!(changed_lines(&text, &out.text).len(), 2, "{}", out.text);
}
