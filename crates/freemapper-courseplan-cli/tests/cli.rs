use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A temp dir holding a copy of the named fixture map.
fn with_map(name: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join(name);
    std::fs::copy(repo_root().join("tests/fixtures/maps").join(name), &map).unwrap();
    (dir, map)
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(cwd)
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

#[test]
fn init_creates_project_next_to_map() {
    let (dir, map) = with_map("luebeck.omap");
    let out = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let project = dir.path().join("luebeck.courseplan.toml");
    assert!(project.exists());
    assert!(stdout(&out).contains("luebeck.courseplan.toml"));
    let info = run(dir.path(), &["info", project.to_str().unwrap()]);
    assert!(info.status.success(), "{}", stderr(&info));
    assert_eq!(
        stdout(&info),
        "title: luebeck\nmap: luebeck.omap\nmap kind: sprint\ncontrols: 0\n\
         courses: 0 (individual 0, team 0, relay 0, score 0)\nclasses: 0\n"
    );
}

#[test]
fn init_from_other_folder() {
    let (dir, map) = with_map("luebeck.omap");
    let elsewhere = tempfile::tempdir().unwrap();
    let out = run(
        elsewhere.path(),
        &["init", map.to_str().unwrap(), "--project", "ev/x.toml"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let project = elsewhere.path().join("ev/x.toml");
    let text = std::fs::read_to_string(&project).unwrap();
    // Relative to ev/ inside the other temp folder, the map is not a sibling, so the
    // path climbs; it must not be absolute.
    let line = text.lines().find(|l| l.starts_with("map")).unwrap();
    assert!(
        line.contains("../") && line.contains("luebeck.omap"),
        "{line}"
    );
    assert!(!line.contains('\\'));
    let third = tempfile::tempdir().unwrap();
    let info = run(third.path(), &["info", project.to_str().unwrap()]);
    assert!(info.status.success(), "{}", stderr(&info));
    assert!(stdout(&info).contains("map kind: sprint"));
    drop(dir);
}

#[test]
fn init_writes_map_relative_to_project_folder() {
    let (dir, map) = with_map("luebeck.omap");
    let project = dir.path().join("ev/x.toml");
    let out = run(
        dir.path(),
        &[
            "init",
            map.to_str().unwrap(),
            "--project",
            project.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let text = std::fs::read_to_string(&project).unwrap();
    assert!(text.contains("map = \"../luebeck.omap\""), "{text}");
}

#[test]
fn init_takes_title_from_freemapper_toml() {
    let (dir, map) = with_map("luebeck.omap");
    std::fs::write(
        dir.path().join("freemapper.toml"),
        "title = \"Altstadt Lübeck\"\n",
    )
    .unwrap();
    let out = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = std::fs::read_to_string(dir.path().join("luebeck.courseplan.toml")).unwrap();
    assert!(text.contains("title = \"Altstadt Lübeck\""), "{text}");
}

#[test]
fn init_title_option_wins() {
    let (dir, map) = with_map("luebeck.omap");
    std::fs::write(dir.path().join("freemapper.toml"), "title = \"Altstadt\"\n").unwrap();
    let out = run(
        dir.path(),
        &["init", map.to_str().unwrap(), "--title", "Training"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let text = std::fs::read_to_string(dir.path().join("luebeck.courseplan.toml")).unwrap();
    assert!(text.contains("title = \"Training\""), "{text}");
}

#[test]
fn init_refuses_forest_map() {
    let (dir, map) = with_map("forest-sample.omap");
    let out = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).starts_with("error: "), "{}", stderr(&out));
    assert!(stderr(&out).contains("Forest maps"));
    assert!(!dir.path().join("forest-sample.courseplan.toml").exists());
}

#[test]
fn init_reports_missing_map() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["init", "nowhere.omap"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("nowhere.omap"), "{}", stderr(&out));
    assert!(!dir.path().join("nowhere.courseplan.toml").exists());
}

#[test]
fn init_refuses_existing_project() {
    let (dir, map) = with_map("luebeck.omap");
    let first = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert!(first.status.success());
    let project = dir.path().join("luebeck.courseplan.toml");
    let before = std::fs::read_to_string(&project).unwrap();
    let second = run(
        dir.path(),
        &["init", map.to_str().unwrap(), "--title", "Other"],
    );
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(
        stderr(&second),
        format!(
            "error: {} already exists; choose another project file or remove it\n",
            project.display()
        )
    );
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
}

#[test]
fn info_reports_invalid_project() {
    let (dir, _map) = with_map("luebeck.omap");
    let broken = dir.path().join("broken.toml");
    std::fs::copy(
        repo_root().join("crates/freemapper-courseplan-model/tests/fixtures/broken.toml"),
        &broken,
    )
    .unwrap();
    let out = run(dir.path(), &["info", broken.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("44"), "{}", stderr(&out));
    assert!(stdout(&out).is_empty());
}
