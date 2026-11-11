use std::path::{Path, PathBuf};

use freemapper_courseplan_model::{
    AcceptedFinding, ControlCode, ControlKind, CourseKind, Event, LegRef, MapPoint, ModelError,
    PrintArea, ProjectError, ProjectFile, Side, SpecialObject, SyncDecision, SyncState,
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
fn missing_file_is_read_error() {
    let err = ProjectFile::load(&fixture("nope.toml")).unwrap_err();
    assert!(matches!(err, ProjectError::Read { .. }), "{err:?}");
    assert!(err.to_string().starts_with("cannot read "), "{err}");
    assert!(err.to_string().contains("nope.toml"), "{err}");
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
fn save_refuses_a_file_changed_since_load() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    std::fs::write(&file, "other bytes\n").unwrap();
    project.event.event.title = "Quay".to_owned();
    let err = project.save().unwrap_err();
    assert!(
        matches!(&err, ProjectError::ChangedMeanwhile { path } if path == &file),
        "{err:?}"
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "other bytes\n");
    assert!(!dir.path().join("event.toml.tmp").exists());
}

#[test]
fn save_refuses_a_file_deleted_since_load() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    std::fs::remove_file(&file).unwrap();
    project.event.event.title = "Quay".to_owned();
    let err = project.save().unwrap_err();
    assert!(
        matches!(err, ProjectError::ChangedMeanwhile { .. }),
        "{err:?}"
    );
    assert!(!file.exists());
}

#[test]
fn two_saves_in_a_row_work() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.event.title = "Quay".to_owned();
    project.save().unwrap();
    project.event.event.title = "Dock".to_owned();
    project.save().unwrap();
    assert_eq!(ProjectFile::load(&file).unwrap().event.event.title, "Dock");
}

#[test]
fn create_then_save_works() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("event.toml");
    let event = ProjectFile::load(&fixture("minimal.toml")).unwrap().event;
    let mut created = ProjectFile::create(&file, event).unwrap();
    created.event.event.title = "Quay".to_owned();
    created.save().unwrap();
    assert_eq!(ProjectFile::load(&file).unwrap().event.event.title, "Quay");
}

#[test]
fn create_refuses_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("event.toml");
    let event = ProjectFile::load(&fixture("minimal.toml")).unwrap().event;
    let mut created = ProjectFile::create(&file, event.clone()).unwrap();
    let before = std::fs::read_to_string(&file).unwrap();
    let err = ProjectFile::create(&file, event.clone()).unwrap_err();
    let ProjectError::AlreadyExists { path } = &err else {
        panic!("unexpected error {err:?}");
    };
    assert_eq!(path, &file);
    assert_eq!(
        err.to_string(),
        format!(
            "{} already exists; choose another project file or remove it",
            file.display()
        )
    );
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
fn failed_save_keeps_original() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "commented.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.event.title = "Quay".to_owned();
    // A directory in place of the temporary file makes the write fail on every platform (the guard has
    // already passed, because the project file itself is untouched).
    let before = std::fs::read_to_string(&file).unwrap();
    let tmp = dir.path().join("event.toml.tmp");
    std::fs::create_dir(&tmp).unwrap();
    let err = project.save().unwrap_err();
    assert!(matches!(err, ProjectError::Write { .. }), "{err:?}");
    assert!(err.to_string().starts_with("cannot write "), "{err}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
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

#[test]
fn unchanged_save_keeps_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "sketch.toml");
    let before = std::fs::read_to_string(&file).unwrap();
    let mut project = ProjectFile::load(&file).unwrap();
    project.save().unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
}

#[test]
fn changing_a_scalar_keeps_its_comment() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "sketch.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.courses[0].climb_m = Some(40);
    project.event.controls[1].map_mm.x = 11.5;
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("climb_m = 40                    # entered by hand\n"),
        "{text}"
    );
    assert!(
        text.contains("map_mm = [11.5, 20] # at the boat shed\n"),
        "{text}"
    );
    assert_eq!(
        ProjectFile::load(&file).unwrap().event,
        project.event,
        "{text}"
    );
}

#[test]
fn changed_inline_values_stay_inline() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "sketch.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.controls[0].description.g = Some("11.3".to_owned());
    project.event.controls[0].number.as_mut().unwrap().direction = 90.0;
    match &mut project.event.courses[0].kind {
        CourseKind::Individual { controls } => controls[2].bends.push([1.0, 2.0].into()),
        other => panic!("unexpected kind {other:?}"),
    }
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains(
            "description = { d = \"1.8\", g = \"11.3\" }   # IOF 2024 symbol ids per column\n"
        ),
        "{text}"
    );
    assert!(text.contains("number = { direction = 90"), "{text}");
    assert!(
        text.contains("  { code = 31, number = { direction = 270, gap_mm = 1.0 } },\n"),
        "{text}"
    );
    assert!(
        text.contains("  { code = 33, bends = [[1.0, 2.0]] },\n"),
        "{text}"
    );
    assert!(!text.contains("[controls.description]"), "{text}");
    assert!(!text.contains("[[courses.controls]]"), "{text}");
    assert!(text.contains("code = \"33\" # quoted by hand\n"), "{text}");
    assert_eq!(
        ProjectFile::load(&file).unwrap().event,
        project.event,
        "{text}"
    );
}

#[test]
fn quoted_code_matches_integer_code() {
    let dir = tempfile::tempdir().unwrap();
    let file = copy_fixture(dir.path(), "sketch.toml");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.controls.swap(0, 2);
    project.event.controls[0].map_mm.y = 41.0;
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("code = \"33\" # quoted by hand\nmap_mm = [30, 41.0]\n"),
        "{text}"
    );
    assert_eq!(text.matches("code = \"33\"").count(), 1, "{text}");
    assert_eq!(
        ProjectFile::load(&file).unwrap().event,
        project.event,
        "{text}"
    );
}

/// minimal.toml with `from` replaced by `to`, written to `dir`.
fn edited_minimal(dir: &Path, from: &str, to: &str) -> PathBuf {
    let text = std::fs::read_to_string(fixture("minimal.toml")).unwrap();
    assert!(text.contains(from), "{from:?} not in minimal.toml");
    let file = dir.join("event.toml");
    std::fs::write(&file, text.replacen(from, to, 1)).unwrap();
    file
}

fn load_errors(file: &Path) -> Vec<ModelError> {
    match ProjectFile::load(file).unwrap_err() {
        ProjectError::Invalid { errors, .. } => errors,
        other => panic!("unexpected error {other:?}"),
    }
}

fn unknown_key(table: &str, key: &str) -> ModelError {
    ModelError::UnknownKey {
        table: table.into(),
        key: key.into(),
    }
}

#[test]
fn misspelt_key_in_course_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(
        dir.path(),
        "finish = \"F1\"\n",
        "finish = \"F1\"\nclimb = 35\n",
    );
    let err = ProjectFile::load(&file).unwrap_err();
    let ProjectError::Invalid { errors, .. } = &err else {
        panic!("unexpected error {err:?}");
    };
    assert_eq!(errors, &vec![unknown_key("courses[0]", "climb")]);
    assert!(err.to_string().contains("climb"), "{err}");
}

#[test]
fn unknown_keys_are_found_at_every_depth() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(dir.path(), "format = 1\n", "format = 1\nauthor = \"me\"\n");
    assert_eq!(load_errors(&file), vec![unknown_key("", "author")]);
    let file = edited_minimal(
        dir.path(),
        "code = 32\n",
        "code = 32\ndescription = { d = \"1.8\", x = \"2\" }\n",
    );
    assert_eq!(
        load_errors(&file),
        vec![unknown_key("controls[1].description", "x")]
    );
    let file = edited_minimal(
        dir.path(),
        "code = 32\n",
        "code = 32\ndescription = { x = \"2\" }\n",
    );
    assert_eq!(
        load_errors(&file),
        vec![unknown_key("controls[1].description", "x")]
    );
    let file = edited_minimal(dir.path(), "{ code = 32 }", "{ code = 32, value = 10 }");
    assert_eq!(
        load_errors(&file),
        vec![unknown_key("courses[0].controls[1]", "value")]
    );
    let file = edited_minimal(dir.path(), "[event]\n", "[event]\nplace = \"Potsdam\"\n");
    assert_eq!(load_errors(&file), vec![unknown_key("event", "place")]);
}

#[test]
fn empty_lists_are_known_and_kept() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(
        dir.path(),
        "finish = \"F1\"\ncontrols = [{ code = 31 }, { code = 32 }]\n",
        "finish = \"F1\"\nclasses = []\ncontrols = [{ code = 31, bends = [] }, { code = 32 }]\n\n[[accepted_findings]]\nrule = \"r\"\ncontrols = []\nnote = \"n\"\n",
    );
    let before = std::fs::read_to_string(&file).unwrap();
    let mut project = ProjectFile::load(&file).unwrap();
    project.save().unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), before);
}

#[test]
fn newer_format_is_reported_before_unknown_course_kind() {
    let dir = tempfile::tempdir().unwrap();
    let text = std::fs::read_to_string(fixture("newer_format.toml"))
        .unwrap()
        .replace("kind = \"individual\"", "kind = \"forked\"");
    assert!(text.contains("kind = \"forked\""));
    let file = dir.path().join("event.toml");
    std::fs::write(&file, text).unwrap();
    assert_eq!(load_errors(&file), vec![ModelError::UnsupportedFormat(2)]);
}

#[test]
fn event_date_with_time_or_offset_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    for value in [
        "2026-11-14T10:00:00",
        "2026-11-14T10:00:00+01:00",
        "10:00:00",
    ] {
        let file = edited_minimal(
            dir.path(),
            "title = \"Test\"\n",
            &format!("title = \"Test\"\ndate = {value}\n"),
        );
        let err = ProjectFile::load(&file).unwrap_err();
        let ProjectError::Invalid { errors, .. } = &err else {
            panic!("unexpected error {err:?}");
        };
        assert_eq!(errors, &vec![ModelError::DateNotADate(value.to_owned())]);
        assert!(err.to_string().contains(value), "{err}");
    }
    let file = edited_minimal(
        dir.path(),
        "title = \"Test\"\n",
        "title = \"Test\"\ndate = 2026-11-14\n",
    );
    assert!(ProjectFile::load(&file).is_ok());
}

#[test]
fn duplicate_class_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(
        dir.path(),
        "[[specials]]\n",
        "[[classes]]\nname = \"H21\"\n\n[[classes]]\nname = \"D21\"\n\n[[classes]]\nname = \"H21\"\n\n[[specials]]\n",
    );
    let err = ProjectFile::load(&file).unwrap_err();
    let ProjectError::Invalid { errors, .. } = &err else {
        panic!("unexpected error {err:?}");
    };
    assert_eq!(errors, &vec![ModelError::DuplicateClass("H21".into())]);
    assert!(err.to_string().contains("class H21"), "{err}");
}

fn event_text(extra_specials: &str, extra_controls: &str, course_extra: &str) -> String {
    format!(
        r#"format = 1

[event]
title = "Crossing"

[sources]
map = "maps/m.omap"

[[specials]]
kind = "start"
id = "S1"
map_mm = [0.0, 0.0]

[[specials]]
kind = "finish"
id = "F1"
map_mm = [50.0, 50.0]
{extra_specials}
[[controls]]
code = 31
map_mm = [10.0, 10.0]

[[controls]]
code = 32
map_mm = [20.0, 30.0]
{extra_controls}
[[courses]]
name = "A"
kind = "individual"
start = "S1"
finish = "F1"
{course_extra}controls = [{{ code = 31 }}, {{ code = 32 }}]
"#
    )
}

fn write_event(dir: &Path, text: &str) -> PathBuf {
    let file = dir.join("event.toml");
    std::fs::write(&file, text).unwrap();
    file
}

#[test]
fn crossing_point_control_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let text = event_text(
        "",
        "\n[[controls]]\ncode = \"X1\"\nkind = \"crossing_point\"\nmap_mm = [15.0, 15.0]\n",
        "",
    );
    let file = write_event(dir.path(), &text);
    let mut project = ProjectFile::load(&file).unwrap();
    assert_eq!(project.event.controls[0].kind, ControlKind::Control);
    assert_eq!(project.event.controls[2].kind, ControlKind::CrossingPoint);
    project.event.controls[0].map_mm.x = 11.0;
    project.save().unwrap();
    let saved = std::fs::read_to_string(&file).unwrap();
    assert_eq!(
        saved.matches("kind = \"crossing_point\"").count(),
        1,
        "{saved}"
    );
    assert!(!saved.contains("kind = \"control\""), "{saved}");
    assert_eq!(ProjectFile::load(&file).unwrap().event, project.event);
}

#[test]
fn explicit_default_values_are_known_keys_and_survive_a_save() {
    let dir = tempfile::tempdir().unwrap();
    let text = event_text(
        "\n[[specials]]\nkind = \"water\"\nid = \"w1\"\nmap_mm = [1.0, 2.0]\ncourses = []\n",
        "",
        "start_bends = []\nstart_taped = false\n",
    )
    .replace("{ code = 31 }", "{ code = 31, taped = false }")
    .replace("code = 31\n", "code = 31\nkind = \"control\"\n");
    let file = write_event(dir.path(), &text);
    let mut project = ProjectFile::load(&file).unwrap();
    project.save().unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), text);
}

#[test]
fn special_objects_have_ids_and_courses() {
    let specials = r#"
[[specials]]
kind = "marked_route"
id = "m1"
points = [[1.0, 2.0], [3.0, 4.0]]

[[specials]]
kind = "out_of_bounds_area"
id = "o1"
polygon = [[1.0, 2.0], [3.0, 4.0], [5.0, 1.0]]

[[specials]]
kind = "forbidden_route"
id = "f1"
points = [[1.0, 2.0], [3.0, 4.0]]

[[specials]]
kind = "dangerous_area"
id = "d1"
polygon = [[1.0, 2.0], [3.0, 4.0], [5.0, 1.0]]

[[specials]]
kind = "first_aid"
id = "a1"
map_mm = [1.0, 2.0]

[[specials]]
kind = "water"
id = "w1"
map_mm = [1.0, 2.0]
courses = ["A"]
"#;
    let dir = tempfile::tempdir().unwrap();
    let file = write_event(dir.path(), &event_text(specials, "", ""));
    let mut project = ProjectFile::load(&file).unwrap();
    let SpecialObject::Water { id, courses, .. } = &project.event.specials[7] else {
        panic!("{:?}", project.event.specials);
    };
    assert_eq!(
        (id.as_str(), courses.as_slice()),
        ("w1", ["A".to_owned()].as_slice())
    );
    let SpecialObject::FirstAid { courses, .. } = &project.event.specials[6] else {
        panic!("{:?}", project.event.specials);
    };
    assert!(courses.is_empty());
    project.event.specials[6] = SpecialObject::FirstAid {
        id: "a1".into(),
        ppen_id: None,
        map_mm: MapPoint { x: 9.0, y: 9.0 },
        courses: vec![],
    };
    project.save().unwrap();
    let saved = std::fs::read_to_string(&file).unwrap();
    assert_eq!(saved.matches("courses = ").count(), 1, "{saved}");
    assert_eq!(ProjectFile::load(&file).unwrap().event, project.event);
}

#[test]
fn start_leg_and_taped_leg_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let text = event_text(
        "",
        "",
        "start_bends = [[5.0, 5.0], [6.0, 7.0]]\nstart_taped = true\nprint_area = { min = [0, 0], max = [180, 260] }\n",
    )
    .replace("{ code = 31 }", "{ code = 31, taped = true }");
    let file = write_event(dir.path(), &text);
    let mut project = ProjectFile::load(&file).unwrap();
    let course = &project.event.courses[0];
    assert_eq!(course.start_bends.len(), 2);
    assert!(course.start_taped);
    assert_eq!(
        course.print_area,
        Some(PrintArea {
            min: MapPoint { x: 0.0, y: 0.0 },
            max: MapPoint { x: 180.0, y: 260.0 },
        })
    );
    let CourseKind::Individual { controls } = &course.kind else {
        panic!("not individual");
    };
    assert!(controls[0].taped && !controls[1].taped);
    project.save().unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), text);
    project.event.courses[0].start_taped = false;
    project.event.courses[0].start_bends.clear();
    project.save().unwrap();
    let saved = std::fs::read_to_string(&file).unwrap();
    assert!(!saved.contains("start_"), "{saved}");
    assert_eq!(ProjectFile::load(&file).unwrap().event, project.event);
}

fn invalid_errors(text: &str) -> Vec<ModelError> {
    let dir = tempfile::tempdir().unwrap();
    let file = write_event(dir.path(), text);
    match ProjectFile::load(&file).unwrap_err() {
        ProjectError::Invalid { errors, .. } => errors,
        other => panic!("unexpected error {other:?}"),
    }
}

#[test]
fn special_with_unknown_course_is_reported() {
    let specials = "\n[[specials]]\nkind = \"water\"\nid = \"w1\"\nmap_mm = [1.0, 2.0]\ncourses = [\"A\", \"Z\"]\n";
    assert_eq!(
        invalid_errors(&event_text(specials, "", "")),
        [ModelError::UnknownCourseInSpecial {
            id: "w1".into(),
            course: "Z".into()
        }]
    );
}

#[test]
fn duplicate_special_id_across_kinds_is_reported() {
    let specials = "\n[[specials]]\nkind = \"water\"\nid = \"F1\"\nmap_mm = [1.0, 2.0]\n";
    assert_eq!(
        invalid_errors(&event_text(specials, "", "")),
        [ModelError::DuplicateSpecialId("F1".into())]
    );
}

#[test]
fn adding_a_special_keeps_comments_on_the_others() {
    let dir = tempfile::tempdir().unwrap();
    let text = event_text(
        "\n[[specials]]\nkind = \"water\"\nid = \"w1\"\nmap_mm = [1.0, 2.0] # the fountain\n",
        "",
        "",
    )
    .replace("map_mm = [0.0, 0.0]", "map_mm = [0.0, 0.0] # the gate");
    let file = write_event(dir.path(), &text);
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.specials.insert(
        0,
        SpecialObject::FirstAid {
            id: "a1".into(),
            ppen_id: None,
            map_mm: MapPoint { x: 3.0, y: 3.0 },
            courses: vec![],
        },
    );
    project.save().unwrap();
    let saved = std::fs::read_to_string(&file).unwrap();
    assert!(saved.contains("map_mm = [0.0, 0.0] # the gate"), "{saved}");
    assert!(
        saved.contains("map_mm = [1.0, 2.0] # the fountain"),
        "{saved}"
    );
    assert_eq!(
        ProjectFile::load(&file).unwrap().event,
        project.event,
        "{saved}"
    );
}

#[test]
fn crossing_point_special_is_rejected() {
    let specials = "\n[[specials]]\nkind = \"crossing_point\"\nid = \"x1\"\nmap_mm = [1.0, 2.0]\n";
    let dir = tempfile::tempdir().unwrap();
    let file = write_event(dir.path(), &event_text(specials, "", ""));
    let err = ProjectFile::load(&file).unwrap_err();
    assert!(matches!(err, ProjectError::Parse { .. }), "{err:?}");
    assert!(err.to_string().contains("crossing_point"), "{err}");
}

#[test]
fn overlays_and_leg_keys_are_known_and_kept_byte_for_byte() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(
        dir.path(),
        "[event]\n",
        "[event]\nrule_overlays = [\"nor-bb-2021\"]\n",
    );
    let text = std::fs::read_to_string(&file).unwrap();
    let text = format!(
        "{text}\n[[accepted_findings]]\nrule = \"r\"\nlegs = [{{ course = \"A\", after = 3 }}, {{ course = \"A\", after = 0 }}]\nnote = \"n\"\n"
    );
    std::fs::write(&file, &text).unwrap();
    let mut project = ProjectFile::load(&file).unwrap();
    assert_eq!(project.event.event.rule_overlays, vec!["nor-bb-2021"]);
    assert_eq!(project.event.accepted_findings[0].legs.len(), 2);
    assert_eq!(project.event.accepted_findings[0].legs[0].after, 3);
    project.save().unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), text);
}

#[test]
fn empty_overlays_and_legs_are_known_and_not_written() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(dir.path(), "[event]\n", "[event]\nrule_overlays = []\n");
    let text = format!(
        "{}\n[[accepted_findings]]\nrule = \"r\"\nlegs = []\nnote = \"n\"\n",
        std::fs::read_to_string(&file).unwrap()
    );
    std::fs::write(&file, &text).unwrap();
    let mut project = ProjectFile::load(&file).unwrap();
    project.save().unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), text);
}

#[test]
fn a_new_finding_writes_legs_as_an_inline_array() {
    let dir = tempfile::tempdir().unwrap();
    let file = edited_minimal(dir.path(), "[event]\n", "[event]\n");
    let mut project = ProjectFile::load(&file).unwrap();
    project.event.accepted_findings.push(AcceptedFinding {
        rule: "r".to_owned(),
        controls: vec![],
        course: None,
        legs: vec![
            LegRef {
                course: "A".to_owned(),
                after: 3,
                from: "33".to_owned(),
                to: "34".to_owned(),
            },
            LegRef {
                course: "A".to_owned(),
                after: 0,
                from: "S1".to_owned(),
                to: "31".to_owned(),
            },
        ],
        note: "n".to_owned(),
    });
    project.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains(
            "legs = [{ course = \"A\", after = 3, from = \"33\", to = \"34\" }, \
             { course = \"A\", after = 0, from = \"S1\", to = \"31\" }]"
        ),
        "{text}"
    );
    assert!(ProjectFile::load(&file).is_ok());
}

#[test]
fn sync_state_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = created_with_three_controls_and_two_courses(dir.path());
    project.event.sync = Some(SyncState {
        ppen: Some("a.ppen".into()),
        base_fnv1a64: Some("cbf29ce484222325".to_owned()),
        decisions: vec![
            SyncDecision {
                key: "control/34/position".to_owned(),
                take: Side::Theirs,
                fingerprint: "0123456789abcdef".to_owned(),
            },
            SyncDecision {
                key: "course/A/stops".to_owned(),
                take: Side::Ours,
                fingerprint: "fedcba9876543210".to_owned(),
            },
        ],
    });
    let back = save_and_reload(&mut project);
    assert_eq!(back.format, 1);
    let text = std::fs::read_to_string(&project.path).unwrap();
    assert!(text.contains("[[sync.decisions]]"), "{text}");
    assert!(text.contains("take = \"theirs\""), "{text}");
    // Dropping a decision on a later save keeps the other one.
    project.event.sync.as_mut().unwrap().decisions.remove(0);
    save_and_reload(&mut project);
}

#[test]
fn project_without_sync_writes_no_sync_table() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = created_with_three_controls_and_two_courses(dir.path());
    let back = save_and_reload(&mut project);
    assert_eq!(back.sync, None);
    assert_eq!(back.format, 1);
    let text = std::fs::read_to_string(&project.path).unwrap();
    assert!(!text.contains("sync"), "{text}");
}

#[cfg(unix)]
#[test]
fn write_atomic_does_not_follow_a_symlink_at_the_tmp_name_and_keeps_the_mode() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("event.toml");
    std::fs::write(&target, "old").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
    let victim = dir.path().join("victim");
    std::fs::write(&victim, "keep").unwrap();
    symlink(&victim, dir.path().join("event.toml.tmp")).unwrap();

    freemapper_courseplan_model::write_atomic(&target, "new").unwrap();

    assert_eq!(std::fs::read_to_string(&target).unwrap(), "new");
    assert_eq!(std::fs::read_to_string(&victim).unwrap(), "keep");
    let mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o640);
    assert!(!dir.path().join("event.toml.tmp").exists());
}
