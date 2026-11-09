//! `export --iof-xml` on the command line.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run(cwd: &Path, lang: &str, args: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(cwd)
        .env_remove("FREEMAPPER_LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C")
        .arg("--lang")
        .arg(lang)
        .args(args)
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// A temp dir with the example project, its map next to it, and `extra` appended to the project.
fn example(extra: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(
        repo_root().join("tests/fixtures/maps/luebeck.omap"),
        dir.path().join("luebeck.omap"),
    )
    .unwrap();
    let text = std::fs::read_to_string(
        repo_root().join("examples/luebeck-sprint/luebeck-sprint.courseplan.toml"),
    )
    .unwrap()
    .replace("../../tests/fixtures/maps/luebeck.omap", "luebeck.omap");
    let project = dir.path().join("event.courseplan.toml");
    std::fs::write(&project, format!("{text}\n{extra}")).unwrap();
    (dir, project)
}

const TEAM: &str = r#"
[[courses]]
name = "Team"
start = "S1"
finish = "F1"
kind = "team"
team_size = 3
mandatory = []
free = [31, 32]
"#;

#[test]
fn writes_the_file_and_reports_the_course_count() {
    let (dir, project) = example("");
    let out = dir.path().join("out.xml");
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        stdout(&result).contains("wrote 1 course to"),
        "{}",
        stdout(&result)
    );
    let xml = std::fs::read_to_string(&out).unwrap();
    assert!(xml.contains("<CourseData"), "{xml}");
    assert!(xml.contains("<Name>A</Name>"), "{xml}");
    assert!(!dir.path().join("out.xml.tmp").exists());
}

#[test]
fn speaks_german() {
    let (dir, project) = example("");
    let out = dir.path().join("out.xml");
    let result = run(
        dir.path(),
        "de",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        stdout(&result).contains("1 Bahn nach"),
        "{}",
        stdout(&result)
    );
}

#[test]
fn names_a_skipped_team_course() {
    let (dir, project) = example(TEAM);
    let out = dir.path().join("out.xml");
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
    let text = stdout(&result);
    assert!(text.contains("left out Team: team courses"), "{text}");
    assert!(text.contains("wrote 1 course to"), "{text}");
    assert!(!std::fs::read_to_string(&out).unwrap().contains("Team"));
}

#[test]
fn without_a_format_it_fails() {
    let (dir, project) = example("");
    let result = run(dir.path(), "en", &["export", project.to_str().unwrap()]);
    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("nothing to export"),
        "{}",
        stderr(&result)
    );
}

#[test]
fn an_existing_target_is_replaced() {
    let (dir, project) = example("");
    let out = dir.path().join("out.xml");
    std::fs::write(&out, "old").unwrap();
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        std::fs::read_to_string(&out)
            .unwrap()
            .contains("<CourseData")
    );
}

#[test]
fn an_error_leaves_the_target_unchanged() {
    let bad = r#"
[[courses]]
name = "Bad"
start = "S1"
finish = "F1"
kind = "individual"

[[courses.controls]]
code = 99
"#;
    let (dir, project) = example(bad);
    let out = dir.path().join("out.xml");
    std::fs::write(&out, "old").unwrap();
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
    assert!(stderr(&result).contains("99"), "{}", stderr(&result));
    assert!(stderr(&result).contains("Bad"), "{}", stderr(&result));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "old");
    assert!(!dir.path().join("out.xml.tmp").exists());
}

#[test]
fn the_example_project_exports() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("x.xml");
    let result = run(
        &repo_root(),
        "en",
        &[
            "export",
            "examples/luebeck-sprint/luebeck-sprint.courseplan.toml",
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
}

#[test]
fn names_a_skipped_team_course_in_german() {
    let (dir, project) = example(TEAM);
    let out = dir.path().join("out.xml");
    let result = run(
        dir.path(),
        "de",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
    let text = stdout(&result);
    assert!(
        text.contains(
            "Team ausgelassen: Bahnart Mannschaftsbahn wird nicht nach IOF XML exportiert"
        ),
        "{text}"
    );
}

#[test]
fn refuses_the_project_file_as_target() {
    let (dir, project) = example("");
    let before = std::fs::read_to_string(&project).unwrap();
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            project.to_str().unwrap(),
        ],
    );
    assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
    assert!(
        stderr(&result).contains("would overwrite it"),
        "{}",
        stderr(&result)
    );
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
}

#[test]
fn refuses_the_map_as_target_also_through_another_spelling() {
    let (dir, project) = example("");
    let map = dir.path().join("luebeck.omap");
    let before = std::fs::read(&map).unwrap();
    let spelled = dir.path().join(".").join("luebeck.omap");
    let result = run(
        dir.path(),
        "de",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            spelled.to_str().unwrap(),
        ],
    );
    assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
    assert!(
        stderr(&result).contains("überschreiben"),
        "{}",
        stderr(&result)
    );
    assert_eq!(std::fs::read(&map).unwrap(), before);
}

#[test]
fn a_target_in_a_missing_directory_fails_and_names_the_path() {
    let (dir, project) = example("");
    let out = dir.path().join("missing").join("out.xml");
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            out.to_str().unwrap(),
        ],
    );
    assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
    assert!(stderr(&result).contains("out.xml"), "{}", stderr(&result));
    assert!(!out.exists());
}
