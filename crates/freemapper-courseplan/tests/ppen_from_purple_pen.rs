//! Two real Purple Pen 4.0 files of the Lübeck example (`tests/fixtures/ppen/luebeck-sprint-pp4-*.ppen`, see
//! the README there), imported against the real Lübeck map: the file Purple Pen 4.0 saved without a change
//! and the file after edits made in Purple Pen. The edits come back through the import, and an export onto
//! the file as its base gives the file back byte for byte, or changes exactly the lines the project changed.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use common::{bytes, valid_event};
use freemapper_courseplan::check::check_event_as;
use freemapper_courseplan::map::map_kind_for_file;
use freemapper_courseplan::omap::MapDocument;
use freemapper_courseplan::omap::read::read_map_file;
use freemapper_courseplan::ppen_export::{Deletions, Target, export_ppen};
use freemapper_courseplan::ppen_import::import_ppen;
use freemapper_courseplan::rules::load_rule_set;
use freemapper_courseplan_model::{Course, CourseKind, Event};

const UNCHANGED: &str = "luebeck-sprint-pp4-unchanged.ppen";
const CHANGED: &str = "luebeck-sprint-pp4-changed.ppen";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn luebeck_map() -> MapDocument {
    read_map_file(&repo_root().join("tests/fixtures/maps/luebeck.omap")).unwrap()
}

fn event_of(file: &[u8], map: &MapDocument) -> Event {
    valid_event(&import_ppen(file, map).unwrap())
}

fn example_bytes() -> Vec<u8> {
    std::fs::read(repo_root().join("examples/luebeck-sprint/luebeck-sprint.ppen")).unwrap()
}

fn export_onto(event: &Event, map: &MapDocument, base: &[u8]) -> String {
    let target = Target {
        base: Some(base),
        linked: true,
        map_path: "../maps/luebeck.omap".to_owned(),
        lang: "en",
        delete: Deletions::default(),
    };
    export_ppen(event, map, target).unwrap().text
}

fn stops(course: &Course) -> Vec<&str> {
    match &course.kind {
        CourseKind::Individual { controls } => controls.iter().map(|c| c.code.0.as_str()).collect(),
        other => panic!("{other:?}"),
    }
}

fn course<'a>(event: &'a Event, name: &str) -> &'a Course {
    event.courses.iter().find(|c| c.name == name).unwrap()
}

#[test]
fn fixtures_are_the_files_purple_pen_wrote() {
    for name in [UNCHANGED, CHANGED] {
        let text = String::from_utf8(bytes(name)).unwrap();
        assert!(text.starts_with('\u{feff}'), "{name}: byte order mark");
        // Only the map element was edited: no home path of the author.
        assert!(!text.contains("absolute-path"), "{name}");
        assert!(
            text.contains(r#"ignore-missing-fonts="false">../maps/luebeck.omap</map>"#),
            "{name}"
        );
    }
}

/// Both files import onto the real map into the project of the example, but for the Purple Pen ids:
/// the import of a file that Purple Pen 4.0 saved without a change makes what the import of our own
/// export makes. The only differences allowed are the Purple Pen ids of controls, courses and special
/// objects (`ppen_id`), which Purple Pen renumbers on save; they are cleared before the comparison.
#[test]
fn unchanged_file_imports_like_our_own_export() {
    let map = luebeck_map();
    let ours = event_of(&example_bytes(), &map);
    let theirs = event_of(&bytes(UNCHANGED), &map);
    assert_eq!(ours.courses.len(), 1);
    assert_eq!(theirs.courses.len(), 1);
    let plain = |mut e: Event| {
        for c in &mut e.controls {
            c.ppen_id = None;
        }
        for c in &mut e.courses {
            c.ppen_id = None;
        }
        e
    };
    let (ours, theirs) = (plain(ours), plain(theirs));
    assert_eq!(ours.controls, theirs.controls);
    assert_eq!(ours.courses, theirs.courses);
    assert_eq!(ours.specials.len(), theirs.specials.len());
}

#[test]
fn changed_file_imports_every_edit() {
    let map = luebeck_map();
    let event = event_of(&bytes(CHANGED), &map);
    let control = |code: &str| {
        event
            .controls
            .iter()
            .find(|c| c.code.0 == code)
            .unwrap_or_else(|| panic!("control {code}"))
    };
    // Control 34 moved: Purple Pen (9.126054, 4.836937) mm, y up, is (9.126, -4.837) y down.
    let c34 = control("34");
    assert!((c34.map_mm.x - 9.126).abs() < 0.001, "{:?}", c34.map_mm);
    assert!((c34.map_mm.y + 4.837).abs() < 0.001, "{:?}", c34.map_mm);
    // Control 35 got symbol 2.2 in box D.
    assert_eq!(control("35").description.d.as_deref(), Some("2.2"));
    // Controls 38 and 39 are new.
    let c38 = control("38");
    assert!((c38.map_mm.x - 8.180).abs() < 0.001 && (c38.map_mm.y + 42.749).abs() < 0.001);
    let c39 = control("39");
    assert!((c39.map_mm.x + 82.320).abs() < 0.001 && (c39.map_mm.y + 20.759).abs() < 0.001);

    // 38 is inserted into course A between 33 and 34.
    let a = stops(course(&event, "A"));
    let at = a.iter().position(|c| *c == "38").expect("38 in A");
    assert_eq!((a[at - 1], a[at + 1]), ("33", "34"), "{a:?}");
    assert!(!a.contains(&"39"), "{a:?}");

    // Course B is new and uses 39 and the controls Purple Pen listed for it.
    let b = stops(course(&event, "B"));
    assert_eq!(b.len(), 5, "{b:?}");
    assert!(b.contains(&"39"), "{b:?}");
    assert!(!b.contains(&"38"), "{b:?}");

    // The moved number of control 35 in course A: Purple Pen's (-1.7257366, 8.046109) mm from the
    // control, y up, instead of (-4.3672853, 3.8638477).
    let unchanged = event_of(&bytes(UNCHANGED), &map);
    let number_of = |e: &Event, code: &str| {
        let own = e
            .controls
            .iter()
            .find(|c| c.code.0 == code)
            .unwrap()
            .number
            .clone();
        let in_course = match &course(e, "A").kind {
            CourseKind::Individual { controls } => controls
                .iter()
                .find(|c| c.code.0 == code)
                .unwrap()
                .number
                .clone(),
            other => panic!("{other:?}"),
        };
        own.or(in_course)
    };
    let (before, after) = (number_of(&unchanged, "35"), number_of(&event, "35"));
    assert!(after.is_some(), "the moved number is a placement");
    assert_ne!(before, after);

    // The result still passes through the checks, as the `check` command runs them.
    let rules = load_rule_set(&event.event.rule_profile, &event.event.rule_overlays).unwrap();
    let kind = map_kind_for_file(&map, &repo_root().join("tests/fixtures/maps/luebeck.omap"));
    let report = check_event_as(&event, &map, kind, &rules);
    assert!(
        report
            .skipped
            .iter()
            .all(|s| s.location == freemapper_courseplan::check::Location::Event),
        "{:?}",
        report.skipped
    );
}

#[test]
fn reexport_onto_the_purple_pen_file_is_byte_identical() {
    let map = luebeck_map();
    for name in [UNCHANGED, CHANGED] {
        let file = bytes(name);
        let event = event_of(&file, &map);
        let out = export_onto(&event, &map, &file);
        assert_eq!(out.as_bytes(), file.as_slice(), "{name}");
    }
}

/// The lines of `from` that `to` lacks, and those it adds, as multisets.
fn line_diff(from: &str, to: &str) -> (Vec<String>, Vec<String>) {
    let count = |text: &str| {
        let mut m: BTreeMap<String, i64> = BTreeMap::new();
        for l in text.replace("\r\n", "\n").split('\n') {
            *m.entry(l.to_owned()).or_default() += 1;
        }
        m
    };
    let (a, b) = (count(from), count(to));
    let (mut removed, mut added) = (vec![], vec![]);
    for key in a
        .keys()
        .chain(b.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let d = b.get(key).copied().unwrap_or(0) - a.get(key).copied().unwrap_or(0);
        for _ in 0..d.abs() {
            if d < 0 { &mut removed } else { &mut added }.push(key.clone());
        }
    }
    (removed, added)
}

#[test]
fn two_edits_change_exactly_two_lines() {
    let map = luebeck_map();
    let file = bytes(CHANGED);
    let base = String::from_utf8(file.clone()).unwrap();
    let mut event = event_of(&file, &map);
    // Control 31 one millimetre north (y down: smaller y), and symbol F 10.1 at control 32.
    let c31 = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "31")
        .unwrap();
    c31.map_mm.y -= 1.0;
    let c32 = event
        .controls
        .iter_mut()
        .find(|c| c.code.0 == "32")
        .unwrap();
    c32.description.f = Some("10.1".to_owned());
    valid_event_check(&event);

    let out = export_onto(&event, &map, &file);
    let (removed, added) = line_diff(&base, &out);
    assert_eq!(removed.len(), 1, "{removed:?} {added:?}");
    assert_eq!(added.len(), 2, "{removed:?} {added:?}");
    assert!(removed[0].trim().starts_with("<location "), "{removed:?}");
    let located: Vec<&String> = added
        .iter()
        .filter(|l| l.trim().starts_with("<location "))
        .collect();
    assert_eq!(located.len(), 1, "{added:?}");
    let described: Vec<&String> = added
        .iter()
        .filter(|l| l.trim() == r#"<description box="F" iof-2004-ref="10.1" />"#)
        .collect();
    assert_eq!(described.len(), 1, "{added:?}");

    // Purple Pen's own parts are all still there: print areas, labels, course B.
    for part in [
        r#"left="-203.84026" top="125.69028" right="202.05174" bottom="-157.01172""#,
        r#"<labels label-kind="sequence" />"#,
        "<name>B</name>",
    ] {
        assert_eq!(
            out.matches(part).count(),
            base.matches(part).count(),
            "{part}"
        );
        assert!(out.contains(part), "{part}");
    }
    // Control 32 has no description lines in the file, so the new line follows its location.
    let pos = out
        .find(r#"<description box="F" iof-2004-ref="10.1" />"#)
        .unwrap();
    let head = &out[..pos];
    let control = &head[head.rfind("<control ").unwrap()..];
    assert!(control.contains("<code>32</code>"), "{control}");
}

fn valid_event_check(event: &Event) {
    assert_eq!(freemapper_courseplan_model::validate(event), vec![]);
}
