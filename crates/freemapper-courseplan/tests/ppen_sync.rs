//! `sync_ppen` on the Lübeck example linked to a Purple Pen file (spec section 7, one test per merge case).

mod common;

use common::purple_pen::{luebeck_map, purple_pen_saves, root};
use freemapper_courseplan::map::MapKind;
use freemapper_courseplan::omap::MapDocument;
use freemapper_courseplan::ppen_export::{Deletions, Target, export_ppen_as, store_ids};
use freemapper_courseplan::sync::apply::SyncNote;
use freemapper_courseplan::sync::base::{BaseState, base_state, fnv1a64};
use freemapper_courseplan::sync::decide::Take;
use freemapper_courseplan::sync::{SyncError, SyncInput, SyncPlan, sync_ppen};
use freemapper_courseplan_model::{
    ControlCode, CourseControl, CourseKind, Event, ProjectFile, Side, SyncState,
};
use std::path::{Path, PathBuf};

struct Setup {
    map: MapDocument,
    kind: MapKind,
    project: Event,
    base: String,
}

fn setup() -> Setup {
    let (map, kind) = luebeck_map();
    let dir = root().join("examples/luebeck-sprint");
    let mut project = ProjectFile::load(&dir.join("luebeck-sprint.courseplan.toml"))
        .unwrap()
        .event;
    let exported = export_ppen_as(
        &project,
        &map,
        kind,
        Target {
            base: None,
            linked: true,
            map_path: "../maps/luebeck.omap".to_owned(),
            lang: "en",
            delete: Deletions::default(),
        },
    )
    .unwrap();
    store_ids(&mut project, &exported.ids);
    Setup {
        map,
        kind,
        project,
        base: exported.text,
    }
}

impl Setup {
    fn run(&self, project: &Event, theirs: &str, take: &[Take]) -> Result<SyncPlan, SyncError> {
        sync_ppen(SyncInput {
            project,
            base: Some(self.base.as_bytes()),
            theirs: theirs.as_bytes(),
            map: &self.map,
            kind: self.kind,
            take,
            base_note: None,
        })
    }
}

fn control<'a>(e: &'a mut Event, code: &str) -> &'a mut freemapper_courseplan_model::Control {
    e.controls.iter_mut().find(|c| c.code.0 == code).unwrap()
}

#[test]
fn nothing_changed_is_a_no_op() {
    let s = setup();
    let plan = s.run(&s.project, &s.base, &[]).unwrap();
    let SyncPlan::Ready {
        project,
        ppen,
        report,
        ..
    } = plan
    else {
        panic!("conflicts")
    };
    assert!(ppen.is_none());
    assert!(report.changes.is_empty(), "{:?}", report.changes);
    let mut expected = s.project.clone();
    expected.sync = project.sync.clone();
    assert_eq!(project, expected);
}

fn ready(
    plan: Result<SyncPlan, SyncError>,
) -> (
    Event,
    Option<String>,
    String,
    freemapper_courseplan::sync::SyncReport,
) {
    match plan.unwrap() {
        SyncPlan::Ready {
            project,
            ppen,
            base,
            report,
        } => (project, ppen, base, report),
        SyncPlan::Conflicts(r) => panic!("conflicts: {:?}", r.conflicts),
    }
}

fn delete_control(e: &mut Event, code: &str) {
    e.controls.retain(|c| c.code.0 != code);
    for c in &mut e.courses {
        if let CourseKind::Individual { controls } = &mut c.kind {
            controls.retain(|cc| cc.code.0 != code);
        }
    }
}

/// Lines only in `a` and lines only in `b`.
fn line_diff(a: &str, b: &str) -> (Vec<String>, Vec<String>) {
    fn minus(x: &str, y: &str) -> Vec<String> {
        let mut rest: Vec<&str> = y.lines().collect();
        let mut out = Vec::new();
        for l in x.lines() {
            if let Some(p) = rest.iter().position(|r| *r == l) {
                rest.remove(p);
            } else {
                out.push(l.trim().to_owned());
            }
        }
        out
    }
    (minus(a, b), minus(b, a))
}

fn block<'a>(text: &'a str, code: &str) -> &'a str {
    let at = text.find(&format!("<code>{code}</code>")).unwrap();
    let from = text[..at].rfind("<control ").unwrap();
    let to = at + text[at..].find("</control>").unwrap();
    &text[from..to]
}

#[test]
fn purple_pen_defaults_are_no_change() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |_| {}, true);
    assert_ne!(theirs, s.base);
    let (_, ppen, _, report) = ready(s.run(&s.project, &theirs, &[]));
    assert!(report.changes.is_empty(), "{:?}", report.changes);
    assert!(report.conflicts.is_empty());
    assert!(ppen.is_none());
}

#[test]
fn one_side_changed_each_way() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |e| control(e, "34").map_mm.x += 1.0, false);
    let mut ours = s.project.clone();
    control(&mut ours, "31").map_mm.x += 1.0;
    let (project, ppen, _, _) = ready(s.run(&ours, &theirs, &[]));
    let moved = project.controls.iter().find(|c| c.code.0 == "34").unwrap();
    let was = s
        .project
        .controls
        .iter()
        .find(|c| c.code.0 == "34")
        .unwrap();
    assert!((moved.map_mm.x - was.map_mm.x - 1.0).abs() < 1e-3);
    let own = project.controls.iter().find(|c| c.code.0 == "31").unwrap();
    assert!((own.map_mm.x - (-49.1)).abs() < 1e-3);
    let (gone, new) = line_diff(&theirs, &ppen.unwrap());
    assert_eq!(gone.len(), 1, "{gone:?}");
    assert_eq!(new.len(), 1, "{new:?}");
    assert!(gone[0].starts_with("<location"));
}

#[test]
fn both_sides_changed_is_listed_with_both_values() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |e| control(e, "34").map_mm.x += 1.0, false);
    let mut ours = s.project.clone();
    control(&mut ours, "34").map_mm.x += 2.0;
    let SyncPlan::Conflicts(report) = s.run(&ours, &theirs, &[]).unwrap() else {
        panic!("no conflict")
    };
    assert_eq!(report.conflicts.len(), 1);
    let c = &report.conflicts[0];
    assert_eq!(c.key, "control/34/position");
    assert!(c.ours.contains(','), "{}", c.ours);
    assert!(c.theirs.contains(','), "{}", c.theirs);
    assert_ne!(c.ours, c.theirs);
}

#[test]
fn take_theirs_resolves() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |e| control(e, "34").map_mm.x += 1.0, false);
    let mut ours = s.project.clone();
    control(&mut ours, "34").map_mm.x += 2.0;
    let (project, ppen, _, _) = ready(s.run(&ours, &theirs, &[Take::All(Side::Theirs)]));
    let c = project.controls.iter().find(|c| c.code.0 == "34").unwrap();
    assert!((c.map_mm.x - (-1.2 + 1.0)).abs() < 1e-3, "{}", c.map_mm.x);
    assert!(ppen.is_none());
}

#[test]
fn deleted_on_one_side_is_deleted_on_the_other() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |e| delete_control(e, "37"), false);
    let (project, _, _, _) = ready(s.run(&s.project, &theirs, &[]));
    assert!(project.controls.iter().all(|c| c.code.0 != "37"));

    let mut ours = s.project.clone();
    delete_control(&mut ours, "36");
    let (_, ppen, _, _) = ready(s.run(&ours, &s.base, &[]));
    let ppen = ppen.unwrap();
    assert!(!ppen.contains("<code>36</code>"));
    assert!(ppen.contains("<code>35</code>"));
}

#[test]
fn renamed_course_is_a_change_not_delete_plus_add() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |e| e.courses[0].name = "Lang".to_owned(), false);
    let (project, _, _, _) = ready(s.run(&s.project, &theirs, &[]));
    assert_eq!(project.courses.len(), 1);
    assert_eq!(project.courses[0].name, "Lang");
    assert_eq!(project.courses[0].classes, s.project.courses[0].classes);
}

#[test]
fn team_course_is_reported_not_written() {
    use freemapper_courseplan::ppen_export::ExportNote;
    use freemapper_courseplan::sync::apply::SyncNote;
    use freemapper_courseplan_model::{ControlCode, Course, CourseControl};
    let s = setup();
    let mut ours = s.project.clone();
    let cc = |c: &str| CourseControl {
        code: ControlCode(c.to_owned()),
        number: None,
        bends: vec![],
        taped: false,
    };
    ours.courses.push(Course {
        name: "T".to_owned(),
        classes: vec![],
        start: ours.courses[0].start.clone(),
        finish: ours.courses[0].finish.clone(),
        start_bends: vec![],
        start_taped: false,
        print_area: None,
        climb_m: None,
        planned_print_scale: None,
        ppen_id: None,
        kind: CourseKind::Team {
            team_size: 2,
            mandatory: vec![cc("31"), cc("32")],
            free: vec![ControlCode("33".to_owned())],
        },
    });
    let (project, ppen, _, report) = ready(s.run(&ours, &s.base, &[]));
    assert!(report.notes.iter().any(|n| matches!(
        n,
        SyncNote::NotWritten(ExportNote::CourseSkipped { course, .. }) if course == "T"
    )));
    assert!(project.courses.iter().any(|c| c.name == "T"));
    assert!(ppen.is_none_or(|p| !p.contains("<name>T</name>")));
}

#[test]
fn new_control_never_takes_over_a_fresh_ppen_id() {
    let s = setup();
    let add = |e: &mut Event, code: &str, dx: f64| {
        let mut c = e
            .controls
            .iter()
            .find(|c| c.code.0 == "31")
            .unwrap()
            .clone();
        c.code.0 = code.to_owned();
        c.ppen_id = None;
        c.map_mm.x += dx;
        e.controls.push(c);
    };
    let theirs = purple_pen_saves(&s.base, |e| add(e, "39", 5.0), false);
    let mut ours = s.project.clone();
    add(&mut ours, "40", 9.0);
    let (project, ppen, _, _) = ready(s.run(&ours, &theirs, &[]));
    let ppen = ppen.unwrap();
    assert_eq!(block(&ppen, "39"), block(&theirs, "39"));
    assert!(ppen.contains("<code>40</code>"));
    let ids: Vec<_> = project
        .controls
        .iter()
        .filter(|c| c.code.0 == "39" || c.code.0 == "40")
        .map(|c| c.ppen_id)
        .collect();
    assert_eq!(ids.len(), 2);
    assert!(ids.iter().all(Option::is_some));
    assert_ne!(ids[0], ids[1]);
}

// Core guard: it drops all event ids when one code or name contradicts the base. The proposal to
// freemapper-print (`PatchOptions::ids_first`, "the caller vouches that the ids are this base's") would let
// the writer match by id there. The sync's check (`unmatched`, unit test in `sync.rs`) refuses a result in
// which the writer kept an object the merge modeled. Swapped codes and a renamed course in one sync land
// as a delete plus a new course, never as a second, ghost course.
#[test]
fn swapped_codes_and_a_renamed_course_land_without_a_ghost_course() {
    use freemapper_courseplan::edit::{rename_control, rename_course};
    use freemapper_courseplan_model::ControlCode;
    let s = setup();
    let mut ours = s.project.clone();
    let code = |c: &str| ControlCode(c.to_owned());
    rename_control(&mut ours, &code("31"), &code("tmp"));
    rename_control(&mut ours, &code("32"), &code("31"));
    rename_control(&mut ours, &code("tmp"), &code("32"));
    rename_course(&mut ours, "A", "Lang");
    // The swap and the rename land as plain changes: no ghost course, and the writer keeps no modeled object.
    let (_, ppen, _, _) = ready(s.run(&ours, &s.base, &[]));
    let ppen = ppen.expect("the swap and the rename change the file");
    assert_eq!(ppen.matches("<course ").count(), 1);
    assert!(ppen.contains("<name>Lang</name>"));
    assert!(!ppen.contains("<name>A</name>"));
}

#[test]
fn unknown_shape_is_refused() {
    let s = setup();
    let theirs = s
        .base
        .replace("</course-scribe-event>", "<future/></course-scribe-event>");
    assert_ne!(theirs, s.base);
    let result = s.run(&s.project, &theirs, &[]);
    assert!(
        matches!(result, Err(SyncError::UnknownVersion(_))),
        "{result:?}"
    );
    // Also with edits on the other side.
    let mut ours = s.project.clone();
    control(&mut ours, "31").map_mm.x += 1.0;
    let result = s.run(&ours, &theirs, &[]);
    assert!(
        matches!(result, Err(SyncError::UnknownVersion(_))),
        "{result:?}"
    );
}

fn control_id(text: &str, code: &str) -> String {
    let at = text.find(&format!("<code>{code}</code>")).unwrap();
    let from = text[..at].rfind("<control id=\"").unwrap() + "<control id=\"".len();
    text[from..from + text[from..].find('"').unwrap()].to_owned()
}

fn kind_id(text: &str, kind: &str) -> String {
    let at = text.find(&format!("kind=\"{kind}\"")).unwrap();
    let from = text[..at].rfind("<control id=\"").unwrap() + "<control id=\"".len();
    text[from..from + text[from..].find('"').unwrap()].to_owned()
}

#[test]
fn a_score_course_keeps_a_deleted_control_alive() {
    use freemapper_courseplan::sync::apply::SyncNote;
    let s = setup();
    let eol = if s.base.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (start, finish, c36) = (
        kind_id(&s.base, "start"),
        kind_id(&s.base, "finish"),
        control_id(&s.base, "36"),
    );
    let score = [
        r#"<course id="900" kind="score" order="9">"#.to_owned(),
        "<name>Sc</name>".to_owned(),
        r#"<first course-control="900" />"#.to_owned(),
        r#"<options print-scale="4000" description-kind="symbols" />"#.to_owned(),
        "</course>".to_owned(),
        format!(r#"<course-control id="900" control="{start}"><next course-control="901" /></course-control>"#),
        format!(r#"<course-control id="901" control="{c36}"><next course-control="902" /></course-control>"#),
        format!(r#"<course-control id="902" control="{finish}" />"#),
        "</course-scribe-event>".to_owned(),
    ]
    .join(eol);
    let with_score = |t: &str| {
        let (course, rest) = score.split_once(&format!("</course>{eol}")).unwrap();
        let at = t.rfind("</course>").unwrap() + "</course>".len();
        let mut out = t[..at].to_owned();
        out.push_str(eol);
        out.push_str(course);
        out.push_str("</course>");
        out.push_str(&t[at..t.rfind("</course-scribe-event>").unwrap()]);
        out.push_str(rest);
        out
    };
    let base = with_score(&s.base);
    let theirs = with_score(&s.base);
    let mut ours = s.project.clone();
    delete_control(&mut ours, "36");
    let plan = sync_ppen(SyncInput {
        project: &ours,
        base: Some(base.as_bytes()),
        theirs: theirs.as_bytes(),
        map: &s.map,
        kind: s.kind,
        take: &[],
        base_note: None,
    });
    let (project, ppen, _, report) = ready(plan);
    assert!(report.notes.contains(&SyncNote::DeletionKept {
        control: "36".to_owned(),
        course: "Sc".to_owned()
    }));
    // Course A lost 36 in the project, so the file changes; the score course keeps the control.
    let ppen = ppen.expect("course A lost 36, so the file changes");
    assert!(ppen.contains("<code>36</code>") && ppen.contains("<name>Sc</name>"));
    // M8: a control kept on purpose is a note, not a change that shows up on every sync.
    assert!(
        report.changes.iter().all(|c| c.key != "control/36"),
        "{:?}",
        report.changes
    );
    assert!(project.controls.iter().all(|c| c.code.0 != "36"));
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(root().join("tests/fixtures/ppen").join(name)).unwrap()
}

#[test]
fn real_purple_pen_4_edits_merge() {
    use freemapper_courseplan::ppen_import::import_ppen_as;
    let (map, kind) = luebeck_map();
    let base = fixture("luebeck-sprint-pp4-unchanged.ppen");
    let theirs = fixture("luebeck-sprint-pp4-changed.ppen");
    let import = import_ppen_as(base.as_bytes(), &map, kind).unwrap();
    let mut project = common::valid_event(&import);
    control(&mut project, "31").map_mm.x += 1.0;
    control(&mut project, "32").description.f = Some("10.1".to_owned());
    let plan = sync_ppen(SyncInput {
        project: &project,
        base: Some(base.as_bytes()),
        theirs: theirs.as_bytes(),
        map: &map,
        kind,
        take: &[],
        base_note: None,
    });
    let (merged, ppen, _, _) = ready(plan);
    let get = |code: &str| merged.controls.iter().find(|c| c.code.0 == code).unwrap();
    assert!((get("34").map_mm.x - 9.126054).abs() < 2e-3);
    assert!((get("34").map_mm.y - (-4.836937)).abs() < 2e-3);
    // The file's `<description box="D">` sits at control 35 (the brief said 34).
    assert!(get("35").description.d.is_some());
    get("38");
    get("39");
    let CourseKind::Individual { controls } = &merged.courses[0].kind else {
        panic!()
    };
    let codes: Vec<&str> = controls.iter().map(|c| c.code.0.as_str()).collect();
    let p34 = codes.iter().position(|c| *c == "34").unwrap();
    assert_eq!(codes[p34 - 1], "38", "{codes:?}");
    let b = merged.courses.iter().find(|c| c.name == "B").unwrap();
    assert!(b.classes.is_empty());
    assert!((get("31").map_mm.x - (-50.1 + 1.0)).abs() < 2e-3);
    assert_eq!(get("32").description.f.as_deref(), Some("10.1"));
    let (gone, new) = line_diff(&theirs, &ppen.unwrap());
    assert_eq!(
        gone,
        vec![r#"<location x="-50.1" y="3.6" />"#.to_owned()],
        "{new:?}"
    );
    // I1: course B was added by Purple Pen without pinned numbers; the sync pins none for it.
    assert!(
        new.iter().all(|l| !l.starts_with("<number-location")),
        "{new:?}"
    );
    assert!(new.iter().any(|l| l.starts_with("<location x=\"-49.1")));
    assert!(new.iter().any(|l| l.starts_with("<description box=\"F\"")));
}

const LINK: &str = "../purple-pen/luebeck.ppen";

fn recorded(base: &str) -> SyncState {
    SyncState {
        ppen: Some(PathBuf::from(LINK)),
        base_fnv1a64: Some(fnv1a64(base.as_bytes())),
        decisions: vec![],
    }
}

impl Setup {
    /// What the CLI does: judge the stored base, then sync with what that gives.
    fn run_stored(
        &self,
        project: &Event,
        stored: Option<&str>,
        theirs: &str,
    ) -> Result<SyncPlan, SyncError> {
        let state = base_state(
            project.sync.as_ref(),
            Path::new(LINK),
            stored.map(str::as_bytes),
            theirs.as_bytes(),
        );
        let (base, base_note) = state.resolve(stored.map(str::as_bytes), theirs.as_bytes());
        sync_ppen(SyncInput {
            project,
            base: base.as_deref(),
            theirs: theirs.as_bytes(),
            map: &self.map,
            kind: self.kind,
            take: &[],
            base_note,
        })
    }
}

#[test]
fn ignored_base_deletes_nothing() {
    let s = setup();
    let mut ours = s.project.clone();
    delete_control(&mut ours, "36");
    let run = |take: &[Take]| {
        sync_ppen(SyncInput {
            project: &ours,
            base: None,
            theirs: s.base.as_bytes(),
            map: &s.map,
            kind: s.kind,
            take,
            base_note: Some(SyncNote::BaseIgnored { why: "missing" }),
        })
    };
    // Without a base the differing stop list is a conflict, not a deletion.
    let SyncPlan::Conflicts(report) = run(&[]).unwrap() else {
        panic!("no conflict")
    };
    assert_eq!(report.conflicts.len(), 1);
    assert_eq!(report.conflicts[0].key, "course/A/stops");
    assert!(
        report
            .notes
            .contains(&SyncNote::BaseIgnored { why: "missing" })
    );
    // Taking the .ppen brings 36 back, and the file keeps it.
    let (project, ppen, _, report) = ready(run(&[Take::All(Side::Theirs)]));
    assert!(project.controls.iter().any(|c| c.code.0 == "36"));
    assert!(ppen.is_none_or(|p| p.contains("<code>36</code>")));
    assert!(
        report
            .notes
            .contains(&SyncNote::BaseIgnored { why: "missing" })
    );
}

#[test]
fn foreign_base_is_ignored() {
    let s = setup();
    // The stored base belongs to another event (it still has a control the project never had).
    let foreign = purple_pen_saves(&s.base, |e| delete_control(e, "37"), false);
    let mut ours = s.project.clone();
    ours.sync = Some(recorded("the base of the last sync"));
    assert_eq!(
        base_state(
            ours.sync.as_ref(),
            Path::new(LINK),
            Some(foreign.as_bytes()),
            s.base.as_bytes()
        ),
        BaseState::Ignore("hash")
    );
    let (project, ppen, _, report) = ready(s.run_stored(&ours, Some(&foreign), &s.base));
    assert!(project.controls.iter().any(|c| c.code.0 == "37"));
    assert!(ppen.is_none());
    assert!(
        report
            .notes
            .contains(&SyncNote::BaseIgnored { why: "hash" })
    );
}

#[test]
fn crash_after_ppen_write_converges() {
    let s = setup();
    let mut ours = s.project.clone();
    ours.sync = Some(recorded(&s.base));
    control(&mut ours, "31").map_mm.x += 1.0;
    let (merged, ppen, _, _) = ready(s.run_stored(&ours, Some(&s.base), &s.base));
    let written = ppen.unwrap();
    // The crash: the .ppen is new, the project and the base are still the old ones.
    let (again, ppen, base, report) = ready(s.run_stored(&ours, Some(&s.base), &written));
    assert!(report.conflicts.is_empty());
    assert!(ppen.is_none(), "the .ppen needs no second write");
    assert_eq!(base, written);
    let a = again.controls.iter().find(|c| c.code.0 == "31").unwrap();
    let b = merged.controls.iter().find(|c| c.code.0 == "31").unwrap();
    assert!((a.map_mm.x - b.map_mm.x).abs() < 1e-3);
    assert!(report.notes.is_empty(), "{:?}", report.notes);
}

#[test]
fn crash_after_project_write_repairs_base() {
    let s = setup();
    let mut ours = s.project.clone();
    ours.sync = Some(recorded(&s.base));
    control(&mut ours, "31").map_mm.x += 1.0;
    let (mut merged, ppen, _, _) = ready(s.run_stored(&ours, Some(&s.base), &s.base));
    let written = ppen.unwrap();
    // The crash: .ppen and project are written, the base copy is still the old one.
    merged.sync = Some(recorded(&written));
    assert_eq!(
        base_state(
            merged.sync.as_ref(),
            Path::new(LINK),
            Some(s.base.as_bytes()),
            written.as_bytes()
        ),
        BaseState::Repair
    );
    let (project, ppen, base, report) = ready(s.run_stored(&merged, Some(&s.base), &written));
    assert!(ppen.is_none());
    assert!(report.changes.is_empty(), "{:?}", report.changes);
    assert!(report.notes.contains(&SyncNote::BaseRepaired));
    assert_eq!(base, written);
    let moved = project.controls.iter().find(|c| c.code.0 == "31").unwrap();
    assert!((moved.map_mm.x - (-49.1)).abs() < 1e-3);
}

fn set_stops(e: &mut Event, course: &str, codes: &[&str]) {
    let c = e.courses.iter_mut().find(|c| c.name == course).unwrap();
    let CourseKind::Individual { controls } = &mut c.kind else {
        panic!("not an individual course")
    };
    let old = controls.clone();
    *controls = codes
        .iter()
        .map(|c| CourseControl {
            code: ControlCode((*c).to_owned()),
            number: old
                .iter()
                .find(|o| o.code.0 == *c)
                .and_then(|o| o.number.clone()),
            bends: vec![],
            taped: false,
        })
        .collect();
}

fn stops(e: &Event, course: &str) -> Vec<String> {
    let c = e.courses.iter().find(|c| c.name == course).unwrap();
    let CourseKind::Individual { controls } = &c.kind else {
        panic!("not an individual course")
    };
    controls.iter().map(|c| c.code.0.clone()).collect()
}

/// Purple Pen's file with course A cut to `codes` (the project's other state stays the base).
fn theirs_with_stops(s: &Setup, codes: &[&str]) -> String {
    let mut pp = s.project.clone();
    set_stops(&mut pp, "A", codes);
    pp.accepted_findings.clear();
    export_ppen_as(
        &pp,
        &s.map,
        s.kind,
        Target {
            base: Some(s.base.as_bytes()),
            linked: true,
            map_path: String::new(),
            lang: "en",
            delete: Deletions::default(),
        },
    )
    .unwrap()
    .text
}

/// C1: the project runs `31 32` twice and Purple Pen drops 31. Removing one visit of a repeated identity
/// does not converge on a re-merge, so the merge asks.
#[test]
fn a_repeated_loop_with_a_deleted_visit_asks() {
    let s = setup();
    let theirs = theirs_with_stops(&s, &["32", "33", "34", "35", "36", "37"]);
    let mut ours = s.project.clone();
    set_stops(
        &mut ours,
        "A",
        &["31", "32", "31", "32", "33", "34", "35", "36", "37"],
    );
    ours.accepted_findings.clear();
    assert!(
        matches!(s.run(&ours, &theirs, &[]).unwrap(), SyncPlan::Conflicts(_)),
        "the merge must ask"
    );
}

/// D13: after a crash between the `.ppen` write and the project write, the rerun merges the written file
/// against the old project and must give the same stops, also when the clean merge keeps a repeated stop
/// (the project adds a loop in the middle, Purple Pen drops the last stop).
#[test]
fn crash_after_ppen_write_converges_for_a_repeated_loop() {
    let s = setup();
    let theirs = theirs_with_stops(&s, &["31", "32", "33", "34", "35", "36"]);
    let mut ours = s.project.clone();
    set_stops(
        &mut ours,
        "A",
        &["31", "32", "33", "34", "33", "34", "35", "36", "37"],
    );
    ours.accepted_findings.clear();
    let SyncPlan::Ready {
        project: merged,
        ppen: Some(written),
        ..
    } = s.run(&ours, &theirs, &[]).unwrap()
    else {
        panic!("a clean merge that writes the .ppen was expected");
    };
    let expected = ["31", "32", "33", "34", "33", "34", "35", "36"];
    assert_eq!(stops(&merged, "A"), expected);
    let (again, _, _, _) = ready(s.run(&ours, &written, &[]));
    assert_eq!(stops(&again, "A"), stops(&merged, "A"));
}

fn id_of(text: &str, needle: &str) -> String {
    let at = text.find(needle).unwrap();
    let from = text[..at].rfind("<control id=\"").unwrap() + "<control id=\"".len();
    text[from..from + text[from..].find('"').unwrap()].to_owned()
}

/// Adds a course whose only course-control after the start carries a `variation` attribute: the import
/// skips it, the D11 scan must still see that it visits `control_id`.
fn with_variation_course(text: &str, control_id: &str, name: &str) -> String {
    let start = id_of(text, "kind=\"start\"");
    let finish = id_of(text, "kind=\"finish\"");
    let course = format!(
        "  <course id=\"2\" kind=\"normal\" order=\"2\">\n    <name>{name}</name>\n    <first course-control=\"100\" />\n    <options print-scale=\"4000\" description-kind=\"symbols\" />\n  </course>\n"
    );
    let ccs = format!(
        "  <course-control id=\"100\" control=\"{start}\">\n    <next course-control=\"101\" />\n  </course-control>\n  <course-control id=\"101\" control=\"{control_id}\" variation=\"A\">\n    <next course-control=\"102\" />\n  </course-control>\n  <course-control id=\"102\" control=\"{finish}\" />\n"
    );
    let at = text.find("  <course-control id=").unwrap();
    let mut out = format!("{}{}{}", &text[..at], course, &text[at..]);
    let end = out.rfind("</course-scribe-event>").unwrap();
    out.insert_str(end, &ccs);
    out
}

#[test]
fn variation_course_keeps_a_deleted_control_alive() {
    let s = setup();
    let c36 = id_of(&s.base, "<code>36</code>");
    let base = with_variation_course(&s.base, &c36, "V");
    let mut ours = s.project.clone();
    delete_control(&mut ours, "36");
    ours.accepted_findings.clear();
    let plan = sync_ppen(SyncInput {
        project: &ours,
        base: Some(base.as_bytes()),
        theirs: base.as_bytes(),
        map: &s.map,
        kind: s.kind,
        take: &[],
        base_note: None,
    });
    let (project, ppen, _, report) = ready(plan);
    assert!(
        report.notes.contains(&SyncNote::DeletionKept {
            control: "36".to_owned(),
            course: "V".to_owned()
        }),
        "{:?}",
        report.notes
    );
    assert!(
        report.changes.iter().all(|c| c.key != "control/36"),
        "{:?}",
        report.changes
    );
    let ppen = ppen.expect("course A lost 36, so the file changes");
    assert!(ppen.contains("<code>36</code>"));
    assert!(ppen.contains(&format!("control=\"{c36}\" variation=\"A\"")));
    assert!(project.controls.iter().all(|c| c.code.0 != "36"));
    assert_eq!(
        ppen.matches(&format!("\" control=\"{c36}\"")).count(),
        1,
        "only the variation course-control is left"
    );
}

/// The unchanged Purple Pen 4 file plus `controls` new controls and `courses` new courses of `stops` stops.
fn with_many_additions(base: &str, controls: u32, courses: u32, stops: u32) -> String {
    use std::fmt::Write as _;
    let mut ctl = String::new();
    for i in 0..controls {
        let id = 1000 + i;
        let (x, y) = (
            f64::from(i % 100) * 2.0 - 100.0,
            f64::from(i / 100) * 2.0 - 100.0,
        );
        write!(
            ctl,
            "  <control id=\"{id}\" kind=\"normal\">\n    <code>{}</code>\n    <location x=\"{x}\" y=\"{y}\" />\n  </control>\n",
            1000 + i
        )
        .unwrap();
    }
    let mut crs = String::new();
    let mut cc = String::new();
    let mut next_cc = 1000u32;
    for c in 0..courses {
        let first = next_cc;
        write!(
            crs,
            "  <course id=\"{}\" kind=\"normal\" order=\"{}\">\n    <name>P{c}</name>\n    <first course-control=\"{first}\" />\n    <options print-scale=\"4000\" description-kind=\"symbols\" />\n  </course>\n",
            1000 + c,
            2 + c
        )
        .unwrap();
        let mut chain = vec![1u32]; // the start control
        chain.extend((0..stops).map(|k| 1000 + (c * 7 + k * 13) % controls));
        chain.push(2); // the finish
        for (k, control) in chain.iter().enumerate() {
            write!(
                cc,
                "  <course-control id=\"{}\" control=\"{control}\"",
                next_cc + k as u32
            )
            .unwrap();
            if k + 1 < chain.len() {
                write!(
                    cc,
                    ">\n    <next course-control=\"{}\" />\n  </course-control>\n",
                    next_cc + k as u32 + 1
                )
                .unwrap();
            } else {
                cc.push_str(" />\n");
            }
        }
        next_cc += chain.len() as u32;
    }
    let at = base.find("  <course id=").unwrap();
    let mut out = format!("{}{ctl}{crs}{}", &base[..at], &base[at..]);
    let end = out.rfind("</course-scribe-event>").unwrap();
    out.insert_str(end, &cc);
    out
}

/// Syncs a Purple Pen file that adds `controls` controls and `courses` courses of 50 stops; returns the time.
fn time_large_addition(controls: u32, courses: u32) -> std::time::Duration {
    use freemapper_courseplan::ppen_import::import_ppen_as;
    let (map, kind) = luebeck_map();
    let base = fixture("luebeck-sprint-pp4-unchanged.ppen");
    let theirs = with_many_additions(&base, controls, courses, 50);
    let import = import_ppen_as(base.as_bytes(), &map, kind).unwrap();
    let project = common::valid_event(&import);
    let started = std::time::Instant::now();
    let plan = sync_ppen(SyncInput {
        project: &project,
        base: Some(base.as_bytes()),
        theirs: theirs.as_bytes(),
        map: &map,
        kind,
        take: &[],
        base_note: None,
    })
    .unwrap();
    let elapsed = started.elapsed();
    let (merged, _, _, _) = ready(Ok(plan));
    assert_eq!(
        merged.courses.len(),
        project.courses.len() + courses as usize
    );
    elapsed
}

/// I2: classify and apply used to scan every field and every course per leg (88 s on 10,000 controls). The
/// size fits a plain debug build; doubling it must stay far from quadratic.
#[test]
fn a_large_purple_pen_addition_syncs_in_reasonable_time() {
    let small = time_large_addition(1_500, 150);
    let large = time_large_addition(3_000, 300);
    assert!(
        large < std::time::Duration::from_secs(25),
        "sync took {large:?}"
    );
    let floor = std::time::Duration::from_millis(200);
    assert!(
        // Factor 5, not 3.5: linear work gives about 2, and a debug build on a loaded CI machine is noisy
        // enough to push the ratio up; the 25 s bound above is what catches the old blow-up (88 s before I2).
        large.as_secs_f64() < 5.0 * small.max(floor).as_secs_f64(),
        "t(n) = {small:?}, t(2n) = {large:?}"
    );
}

/// M9: D11 matches the courses the import skipped by Purple Pen id, not by name; a skipped course that
/// carries the name of a modeled one still keeps its control.
#[test]
fn a_skipped_course_named_like_a_modeled_one_still_keeps_its_control() {
    let s = setup();
    let c36 = id_of(&s.base, "<code>36</code>");
    let base = with_variation_course(&s.base, &c36, "A");
    let mut ours = s.project.clone();
    delete_control(&mut ours, "36");
    ours.accepted_findings.clear();
    let plan = sync_ppen(SyncInput {
        project: &ours,
        base: Some(base.as_bytes()),
        theirs: base.as_bytes(),
        map: &s.map,
        kind: s.kind,
        take: &[],
        base_note: None,
    });
    let (_, ppen, _, report) = ready(plan);
    assert!(
        report.notes.contains(&SyncNote::DeletionKept {
            control: "36".to_owned(),
            course: "A".to_owned()
        }),
        "{:?}",
        report.notes
    );
    assert!(ppen.expect("the file changes").contains("<code>36</code>"));
}

/// The changed pp4 file with course B's stop at control 4 pinned like course A's, in the layout of `text`.
fn with_pinned_b_stop(text: &str) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let at = format!("<next course-control=\"15\" />{eol}");
    let pin = format!("    <number-location x=\"0.24777585\" y=\"5.675\" />{eol}");
    let mut out = String::new();
    let (head, tail) = text.split_once(&at).expect("course-control 14");
    out.push_str(head);
    out.push_str(&at);
    out.push_str(&format!("    {}", pin.trim_start()));
    out.push_str(tail);
    out
}

/// I1: Purple Pen resets one course's number to automatic; the sync keeps it automatic there and keeps the
/// other course's placement, now and on every later sync.
#[test]
fn a_number_reset_to_automatic_in_purple_pen_sticks() {
    use freemapper_courseplan::ppen_import::import_ppen_as;
    let (map, kind) = luebeck_map();
    let theirs = fixture("luebeck-sprint-pp4-changed.ppen");
    let base = with_pinned_b_stop(&theirs);
    let import = import_ppen_as(base.as_bytes(), &map, kind).unwrap();
    let project = common::valid_event(&import);
    let run = |project: &Event, base: &str| {
        ready(sync_ppen(SyncInput {
            project,
            base: Some(base.as_bytes()),
            theirs: theirs.as_bytes(),
            map: &map,
            kind,
            take: &[],
            base_note: None,
        }))
    };
    let (merged, ppen, written_base, _) = run(&project, &base);
    // The file is Purple Pen's own: nothing to write.
    assert!(ppen.is_none(), "{:?}", ppen.map(|p| line_diff(&theirs, &p)));
    let numbers = |e: &Event, course: &str, code: &str| {
        let c = e.courses.iter().find(|c| c.name == course).unwrap();
        let CourseKind::Individual { controls } = &c.kind else {
            panic!()
        };
        let cc = controls.iter().find(|cc| cc.code.0 == code).unwrap();
        cc.number.clone().or(e
            .controls
            .iter()
            .find(|c| c.code.0 == code)
            .unwrap()
            .number
            .clone())
    };
    assert!(
        numbers(&merged, "A", "32").is_some(),
        "A keeps its placement"
    );
    assert!(numbers(&merged, "B", "32").is_none(), "B is automatic");
    // A second sync changes nothing.
    let mut again = merged.clone();
    again.sync = None;
    let (m2, p2, _, report) = run(&again, &written_base);
    assert!(p2.is_none());
    assert!(report.changes.is_empty(), "{:?}", report.changes);
    assert!(numbers(&m2, "B", "32").is_none());
}

/// Wave C, I1: a reset to automatic removes the control's central placement; team and relay stops of the
/// project that fall back to it keep the placement.
#[test]
fn a_number_reset_keeps_the_placement_of_team_and_relay_stops() {
    use freemapper_courseplan::ppen_import::import_ppen_as;
    use freemapper_courseplan_model::RelayPart;
    let (map, kind) = luebeck_map();
    let theirs = fixture("luebeck-sprint-pp4-changed.ppen");
    let base = with_pinned_b_stop(&theirs);
    let import = import_ppen_as(base.as_bytes(), &map, kind).unwrap();
    let mut project = common::valid_event(&import);
    let cc = |code: &str| CourseControl {
        code: ControlCode(code.to_owned()),
        number: None,
        bends: vec![],
        taped: false,
    };
    let template = project.courses[0].clone();
    let mut team = template.clone();
    team.name = "Team".to_owned();
    team.ppen_id = None;
    team.classes = vec![];
    team.kind = CourseKind::Team {
        team_size: 2,
        mandatory: vec![cc("32")],
        free: vec![],
    };
    let mut relay = template;
    relay.name = "Relay".to_owned();
    relay.ppen_id = None;
    relay.classes = vec![];
    relay.kind = CourseKind::Relay {
        legs: 2,
        parts: vec![
            RelayPart::Common {
                controls: vec![cc("32")],
            },
            RelayPart::Fork {
                variants: vec![vec![cc("32")], vec![cc("33")]],
            },
        ],
    };
    project.courses.push(team);
    project.courses.push(relay);
    let central_before = project
        .controls
        .iter()
        .find(|c| c.code.0 == "32")
        .unwrap()
        .number
        .clone();
    assert!(central_before.is_some(), "32 has a central placement");
    let (merged, _, _, _) = ready(sync_ppen(SyncInput {
        project: &project,
        base: Some(base.as_bytes()),
        theirs: theirs.as_bytes(),
        map: &map,
        kind,
        take: &[],
        base_note: None,
    }));
    let resolved = |e: &Event, cc: &CourseControl| {
        cc.number.clone().or(e
            .controls
            .iter()
            .find(|c| c.code.0 == cc.code.0)
            .unwrap()
            .number
            .clone())
    };
    for c in &merged.courses {
        match &c.kind {
            CourseKind::Team { mandatory, .. } => {
                assert_eq!(resolved(&merged, &mandatory[0]), central_before, "team");
            }
            CourseKind::Relay { parts, .. } => {
                for p in parts {
                    let stops: Vec<&CourseControl> = match p {
                        RelayPart::Common { controls } => controls.iter().collect(),
                        RelayPart::Fork { variants } => variants.iter().flatten().collect(),
                    };
                    for s in stops.into_iter().filter(|s| s.code.0 == "32") {
                        assert_eq!(resolved(&merged, s), central_before, "relay");
                    }
                }
            }
            _ => {}
        }
    }
}

/// Purple Pen moves the start: the project follows. The key of a special object's own field used to
/// subtract the number of controls from the object's index.
#[test]
fn a_start_moved_in_purple_pen_follows() {
    let s = setup();
    let from = "<location x=\"-97.4\" y=\"3.6\" />";
    assert!(s.base.contains(from));
    let theirs = s.base.replacen(from, "<location x=\"-97\" y=\"3.6\" />", 1);
    let (merged, _, _, report) = ready(s.run(&s.project, &theirs, &[]));
    let start = merged
        .specials
        .iter()
        .find(|x| x.id() == "S1")
        .expect("the start");
    let freemapper_courseplan_model::SpecialObject::Start { map_mm, .. } = start else {
        panic!()
    };
    assert!((map_mm.x - (-97.0)).abs() < 2e-3, "{map_mm:?}");
    assert!(
        report.changes.iter().any(|c| c.key == "special/S1"),
        "{:?}",
        report.changes
    );
}

/// M1: a conflict settled by a take is a change of the report, with the side that won; so is a change that
/// both sides made the same way.
#[test]
fn the_report_lists_settled_conflicts_and_same_changes() {
    let s = setup();
    let theirs = purple_pen_saves(&s.base, |e| control(e, "34").map_mm.x += 1.0, false);
    let mut ours = s.project.clone();
    control(&mut ours, "34").map_mm.x += 2.0;
    let (_, _, _, report) = ready(s.run(&ours, &theirs, &[Take::All(Side::Theirs)]));
    let change = report
        .changes
        .iter()
        .find(|c| c.key == "control/34/position")
        .expect("the settled conflict is listed");
    assert_eq!(
        change.side,
        freemapper_courseplan::sync::merge3::Pick::Theirs
    );
    // Both sides move 34 to the same place.
    let mut same = s.project.clone();
    control(&mut same, "34").map_mm.x += 1.0;
    let (_, _, _, report) = ready(s.run(&same, &theirs, &[]));
    let change = report
        .changes
        .iter()
        .find(|c| c.key == "control/34/position")
        .expect("the same change is listed");
    assert_eq!(change.side, freemapper_courseplan::sync::merge3::Pick::Both);
}

/// M12: after a base copy was ignored because the link changed, the project's Purple Pen ids may come from
/// another file; they must not pair objects.
#[test]
fn project_ids_do_not_pair_objects_after_another_link() {
    let s = setup();
    // Purple Pen's control 37 is called 99 there; the project still calls it 37 (same Purple Pen id).
    let theirs = s.base.replacen("<code>37</code>", "<code>99</code>", 1);
    let run = |why: &'static str| {
        sync_ppen(SyncInput {
            project: &s.project,
            base: None,
            theirs: theirs.as_bytes(),
            map: &s.map,
            kind: s.kind,
            take: &[Take::All(Side::Theirs)],
            base_note: Some(SyncNote::BaseIgnored { why }),
        })
    };
    let keys = |why: &'static str| -> Vec<String> {
        let (_, _, _, report) = ready(run(why));
        report.changes.iter().map(|c| c.key.clone()).collect()
    };
    let missing = keys("missing");
    let other = keys("other link");
    assert!(
        missing.iter().any(|k| k == "control/37/code"),
        "ids pair 37 with 99: {missing:?}"
    );
    assert!(
        !other.iter().any(|k| k == "control/37/code"),
        "ids must not pair: {other:?}"
    );
}
