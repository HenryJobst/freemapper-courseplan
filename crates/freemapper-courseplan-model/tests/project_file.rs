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
