use std::path::{Path, PathBuf};

use freemapper_courseplan_model::{
    ControlCode, CourseKind, Event, ModelError, ProjectError, ProjectFile,
};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn code(s: &str) -> ControlCode {
    ControlCode(s.to_owned())
}

#[test]
fn loads_minimal_event() {
    let project = ProjectFile::load(&fixture("minimal.toml")).unwrap();
    match &project.event.courses[0].kind {
        CourseKind::Individual { controls } => assert_eq!(controls.len(), 2),
        other => panic!("unexpected kind {other:?}"),
    }
}

#[test]
fn load_reports_every_inconsistency() {
    let err = ProjectFile::load(&fixture("broken.toml")).unwrap_err();
    let ProjectError::Invalid { errors, .. } = &err else {
        panic!("unexpected error {err:?}");
    };
    let want = [
        ModelError::DuplicateControl(code("31")),
        ModelError::UnknownControl {
            course: "A".into(),
            code: code("44"),
        },
        ModelError::UnknownStart {
            course: "A".into(),
            id: "S9".into(),
        },
        ModelError::TeamTooSmall {
            course: "T".into(),
            team_size: 1,
        },
        ModelError::FreeControlIsMandatory {
            course: "T".into(),
            code: code("31"),
        },
    ];
    assert_eq!(errors.len(), want.len(), "{errors:?}");
    for e in &want {
        assert!(errors.contains(e), "missing {e:?} in {errors:?}");
    }
    let text = err.to_string();
    let lines: Vec<&str> = text.lines().map(str::trim).collect();
    for e in &want {
        let line = e.to_string();
        assert!(lines.contains(&line.as_str()), "no line {line:?} in {text}");
    }
}

#[test]
fn rejects_newer_format() {
    let err = ProjectFile::load(&fixture("newer_format.toml")).unwrap_err();
    let ProjectError::Invalid { errors, .. } = err else {
        panic!("expected Invalid");
    };
    assert_eq!(errors, vec![ModelError::UnsupportedFormat(2)]);
}

#[test]
fn parse_error_names_file() {
    let err = ProjectFile::load(&fixture("invalid_syntax.toml")).unwrap_err();
    let ProjectError::Parse { message, .. } = &err else {
        panic!("unexpected error {err:?}");
    };
    assert!(message.contains("invalid_syntax.toml"), "{message}");
}

#[test]
fn missing_file_is_io_error() {
    let err = ProjectFile::load(&fixture("nope.toml")).unwrap_err();
    assert!(matches!(err, ProjectError::Io { .. }));
}

#[test]
fn map_path_is_relative_to_project_file() {
    let dir = tempfile::tempdir().unwrap();
    let ev = dir.path().join("ev");
    std::fs::create_dir(&ev).unwrap();
    let text = std::fs::read_to_string(fixture("minimal.toml")).unwrap();
    let file = ev.join("event.toml");
    std::fs::write(&file, text).unwrap();
    let project = ProjectFile::load(&file).unwrap();
    assert_eq!(project.map_path(), ev.join("maps/m.omap"));
}

#[test]
fn absolute_map_path_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let abs = dir.path().join("m.omap");
    let text = std::fs::read_to_string(fixture("minimal.toml"))
        .unwrap()
        .replace("maps/m.omap", &abs.to_string_lossy().replace('\\', "/"));
    let file = dir.path().join("event.toml");
    std::fs::write(&file, text).unwrap();
    assert_eq!(ProjectFile::load(&file).unwrap().map_path(), abs);
}

#[test]
fn round_trip_keeps_unicode() {
    let project = ProjectFile::load(&fixture("unicode.toml")).unwrap();
    assert_eq!(project.event.event.title, "Training Kröpeliner Tor");
    let text = toml::to_string(&project.event).unwrap();
    let back: Event = toml::from_str(&text).unwrap();
    assert_eq!(back, project.event);
    assert_eq!(back.classes.len(), 0);
    assert_eq!(back.courses[0].name, "Weiß");
    assert_eq!(back.courses[0].classes, vec!["Anfänger".to_owned()]);
}

fn errors_of(name: &str) -> Vec<ModelError> {
    match ProjectFile::load(&fixture(name)).unwrap_err() {
        ProjectError::Invalid { errors, .. } => errors,
        other => panic!("unexpected error {other:?}"),
    }
}

fn unknown(course: &str, c: &str) -> ModelError {
    ModelError::UnknownControl {
        course: course.into(),
        code: code(c),
    }
}

#[test]
fn unknown_control_in_relay_common_part() {
    assert!(errors_of("unknown_controls.toml").contains(&unknown("R1", "44")));
}

#[test]
fn unknown_control_in_relay_fork_variant() {
    assert!(errors_of("unknown_controls.toml").contains(&unknown("R2", "45")));
}

#[test]
fn unknown_control_in_score_course() {
    assert!(errors_of("unknown_controls.toml").contains(&unknown("P", "46")));
}

#[test]
fn unknown_control_in_team_free_list() {
    assert!(errors_of("unknown_controls.toml").contains(&unknown("T", "47")));
}

#[test]
fn unknown_finish_is_reported() {
    assert!(
        errors_of("structure.toml").contains(&ModelError::UnknownFinish {
            course: "A".into(),
            id: "F9".into(),
        })
    );
}

#[test]
fn duplicate_course_is_reported() {
    assert!(errors_of("structure.toml").contains(&ModelError::DuplicateCourse("A".into())));
}

#[test]
fn duplicate_special_id_is_reported() {
    assert!(errors_of("structure.toml").contains(&ModelError::DuplicateSpecialId("S1".into())));
}

fn copy_fixture(dir: &Path, name: &str) -> PathBuf {
    let file = dir.join("event.toml");
    std::fs::copy(fixture(name), &file).unwrap();
    file
}

#[test]
fn save_keeps_comments_outside_changed_values() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.event.title = "Quay".to_owned();
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("# Training at the old harbour, drawn by hand."),
        "{text}"
    );
    assert!(text.contains("# The event itself.\n[event]"), "{text}");
    assert!(text.contains("code = 31 # the boat shed"), "{text}");
    assert!(text.contains("\"Quay\""), "{text}");
    assert!(!text.contains("Harbour\""), "{text}");
    assert!(!dir.path().join("event.toml.tmp").exists());
    assert_eq!(ProjectFile::load(&file).unwrap().event, project.event);
}

#[test]
fn save_after_removing_course_drops_it() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.courses.clear();
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("# The event itself.\n[event]"), "{text}");
    assert!(ProjectFile::load(&file).unwrap().event.courses.is_empty());
}

#[test]
fn create_refuses_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("event.toml");
    let event = ProjectFile::load(&fixture("minimal.toml")).unwrap().event;
    let mut created = ProjectFile::create(&file, event.clone()).unwrap();
    let before = std::fs::read_to_string(&file).unwrap();
    let err = ProjectFile::create(&file, event.clone()).unwrap_err();
    let ProjectError::Io { source, .. } = &err else {
        panic!("unexpected error {err:?}");
    };
    assert_eq!(source.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
    assert_eq!(ProjectFile::load(&file).unwrap().event, event);
    created.event.event.title = "Other".to_owned();
    created.save().unwrap();
    assert_eq!(ProjectFile::load(&file).unwrap().event.event.title, "Other");
}

#[test]
fn save_refuses_invalid_event() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    let mut course = project.event.courses[0].clone();
    course.name = "B".to_owned();
    match &mut course.kind {
        CourseKind::Individual { controls } => controls[0].code = code("99"),
        other => panic!("unexpected kind {other:?}"),
    }
    project.event.courses.push(course);
    let before = std::fs::read_to_string(&file).unwrap();
    let err = project.save().unwrap_err();
    assert!(matches!(err, ProjectError::Invalid { .. }), "{err:?}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
}

#[test]
fn save_after_adding_course_and_control_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    let mut control = project.event.controls[0].clone();
    control.code = code("33");
    project.event.controls.push(control);
    let mut course = project.event.courses[0].clone();
    course.name = "B".to_owned();
    match &mut course.kind {
        CourseKind::Individual { controls } => controls[0].code = code("33"),
        other => panic!("unexpected kind {other:?}"),
    }
    project.event.courses.push(course);
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("# The event itself.\n[event]"), "{text}");
    assert_eq!(
        ProjectFile::load(&file).unwrap().event,
        project.event,
        "{text}"
    );
}

fn commented_with_two_controls(dir: &Path) -> PathBuf {
    let file = dir.join("event.toml");
    let text = std::fs::read_to_string(fixture("commented.toml"))
        .unwrap()
        .replace("code = 32\n", "code = 32 # the slipway\n");
    std::fs::write(&file, text).unwrap();
    file
}

#[test]
fn adding_a_control_keeps_comments_on_the_others() {
    let dir = tempfile::tempdir().unwrap();
    let file = commented_with_two_controls(dir.path());
    let mut project = ProjectFile::load(&file).unwrap();
    let mut control = project.event.controls[0].clone();
    control.code = code("33");
    project.event.controls.push(control);
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("code = 31 # the boat shed"), "{text}");
    assert!(text.contains("code = 32 # the slipway"), "{text}");
    assert_eq!(
        ProjectFile::load(&file).unwrap().event,
        project.event,
        "{text}"
    );
}

#[test]
fn removing_a_control_from_a_course_drops_it() {
    let dir = tempfile::tempdir().unwrap();
    let file = commented_with_two_controls(dir.path());
    let mut project = ProjectFile::load(&file).unwrap();
    let mut control = project.event.controls[0].clone();
    control.code = code("33");
    project.event.controls.push(control);
    match &mut project.event.courses[0].kind {
        CourseKind::Individual { controls } => {
            let mut extra = controls[0].clone();
            extra.code = code("33");
            controls.push(extra);
        }
        other => panic!("unexpected kind {other:?}"),
    }
    project.save().unwrap();
    let mut project = ProjectFile::load(&file).unwrap();
    match &mut project.event.courses[0].kind {
        CourseKind::Individual { controls } => {
            controls.remove(1);
        }
        other => panic!("unexpected kind {other:?}"),
    }
    project.save().unwrap();
    let back = ProjectFile::load(&file).unwrap();
    let CourseKind::Individual { controls } = &back.event.courses[0].kind else {
        panic!("unexpected kind");
    };
    let codes: Vec<_> = controls.iter().map(|c| c.code.clone()).collect();
    assert_eq!(codes, vec![code("31"), code("33")]);
}

#[test]
fn clearing_an_optional_field_removes_its_key() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "minimal.toml");
    let text = std::fs::read_to_string(&file)
        .unwrap()
        .replace("finish = \"F1\"\n", "finish = \"F1\"\nclimb_m = 35\n");
    std::fs::write(&file, text).unwrap();
    let mut project = ProjectFile::load(&file).unwrap();
    assert_eq!(project.event.courses[0].climb_m, Some(35));
    project.event.courses[0].climb_m = None;
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(!text.contains("climb_m"), "{text}");
    assert_eq!(
        ProjectFile::load(&file).unwrap().event.courses[0].climb_m,
        None
    );
}

#[test]
fn failed_save_keeps_original_and_removes_tmp() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.event.title = "Quay".to_owned();
    // A non-empty directory in place of the file makes the rename fail on every platform.
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();
    std::fs::write(file.join("keep"), "x").unwrap();
    let err = project.save().unwrap_err();
    assert!(matches!(err, ProjectError::Io { .. }), "{err:?}");
    assert!(!dir.path().join("event.toml.tmp").exists());
    assert_eq!(std::fs::read_to_string(file.join("keep")).unwrap(), "x");
}

/// A file written by `create` from minimal.toml plus control 33 and course B, loaded again.
fn created_with_three_controls_and_two_courses(dir: &Path) -> ProjectFile {
    let mut event = ProjectFile::load(&fixture("minimal.toml")).unwrap().event;
    let mut control = event.controls[0].clone();
    control.code = code("33");
    control.map_mm.x = 40.0;
    event.controls.push(control);
    match &mut event.courses[0].kind {
        CourseKind::Individual { controls } => {
            let mut extra = controls[0].clone();
            extra.code = code("33");
            controls.push(extra);
        }
        other => panic!("unexpected kind {other:?}"),
    }
    let mut course = event.courses[0].clone();
    course.name = "B".to_owned();
    event.courses.push(course);
    let file = dir.join("event.toml");
    ProjectFile::create(&file, event).unwrap();
    ProjectFile::load(&file).unwrap()
}

fn save_and_reload(project: &mut ProjectFile) -> Event {
    project.save().unwrap();
    let text = std::fs::read_to_string(&project.path).unwrap();
    let back = ProjectFile::load(&project.path).unwrap().event;
    assert_eq!(back, project.event, "{text}");
    back
}

#[test]
fn save_keeps_new_order_of_controls() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = created_with_three_controls_and_two_courses(dir.path());
    project.event.controls.reverse();
    let back = save_and_reload(&mut project);
    let codes: Vec<_> = back.controls.iter().map(|c| c.code.clone()).collect();
    assert_eq!(codes, vec![code("33"), code("32"), code("31")]);
}

#[test]
fn save_keeps_new_order_of_courses() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = created_with_three_controls_and_two_courses(dir.path());
    project.event.courses.swap(0, 1);
    let back = save_and_reload(&mut project);
    let names: Vec<_> = back.courses.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, vec!["B", "A"]);
}

#[test]
fn save_keeps_new_order_of_course_controls() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = created_with_three_controls_and_two_courses(dir.path());
    match &mut project.event.courses[0].kind {
        CourseKind::Individual { controls } => controls.reverse(),
        other => panic!("unexpected kind {other:?}"),
    }
    let back = save_and_reload(&mut project);
    let CourseKind::Individual { controls } = &back.courses[0].kind else {
        panic!("unexpected kind");
    };
    let codes: Vec<_> = controls.iter().map(|c| c.code.clone()).collect();
    assert_eq!(codes, vec![code("33"), code("32"), code("31")]);
    let CourseKind::Individual { controls } = &back.courses[1].kind else {
        panic!("unexpected kind");
    };
    let codes: Vec<_> = controls.iter().map(|c| c.code.clone()).collect();
    assert_eq!(codes, vec![code("31"), code("32"), code("33")]);
}
