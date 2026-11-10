//! `sync` on the command line: the project and its linked Purple Pen file, with real files in a temp dir.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use freemapper_courseplan::sync::base::{base_path, fnv1a64, strip_absolute_path};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn command(cwd: &Path, lang: &str, args: &[&str]) -> Command {
    let mut c = Command::new(BIN);
    c.current_dir(cwd)
        .env_remove("FREEMAPPER_LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env_remove("FREEMAPPER_SYNC_TEST_TOUCH")
        .env("LANG", "C")
        .arg("--lang")
        .arg(lang)
        .args(args);
    c
}

fn run(cwd: &Path, lang: &str, args: &[&str]) -> Output {
    command(cwd, lang, args).output().unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// A temp dir with the Luebeck example project and its map, linked to `out.ppen` through
/// `export --ppen`. No sync ran yet.
fn linked() -> (tempfile::TempDir, PathBuf) {
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
    std::fs::write(&project, text).unwrap();
    let out = run(
        dir.path(),
        "en",
        &["export", "event.courseplan.toml", "--ppen", "out.ppen"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    (dir, project)
}

/// `linked()` after a first sync, so the base copy exists.
fn synced() -> (tempfile::TempDir, PathBuf) {
    let (dir, project) = linked();
    let out = run(dir.path(), "en", &["sync", "event.courseplan.toml"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    (dir, project)
}

fn ppen(dir: &Path) -> PathBuf {
    dir.join("out.ppen")
}

fn edit(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{from} not in {}", path.display());
    std::fs::write(path, text.replacen(from, to, 1)).unwrap();
}

fn files(dir: &Path, project: &Path) -> [Option<Vec<u8>>; 3] {
    [
        std::fs::read(ppen(dir)).ok(),
        std::fs::read(project).ok(),
        std::fs::read(base_path(project)).ok(),
    ]
}

// The control positions in the project (map y up) and in the Purple Pen file (y flipped).
const P34: &str = "map_mm = [-1.2, -12.4]";
const P35: &str = "map_mm = [-28.2, 10.2]";
const Q34: &str = "<location x=\"-1.2\" y=\"12.4\" />";
const Q35: &str = "<location x=\"-28.2\" y=\"-10.2\" />";
const Q36: &str = "<location x=\"-52.4\" y=\"-47.6\" />";

/// The project without control 36: its entry, its stop in course A and the finding about its leg.
fn delete_control_36(project: &Path) {
    edit(
        project,
        "[[controls]]\ncode = 36\nmap_mm = [-52.4, 47.6]\nppen_id = 8\n\n",
        "",
    );
    edit(
        project,
        "[[courses.controls]]\ncode = 36\n\n[courses.controls.number]\ndirection = 237.1\ngap_mm = 0.5\n\n",
        "",
    );
    let text = std::fs::read_to_string(project).unwrap();
    let cut = text.find("[[accepted_findings]]").unwrap();
    let end = text[cut..].find("\n\n").map_or(text.len(), |n| cut + n + 2);
    std::fs::write(project, format!("{}{}", &text[..cut], &text[end..])).unwrap();
}

#[test]
fn first_sync_writes_base_and_hash() {
    let (dir, project) = linked();
    let out = run(dir.path(), "en", &["sync", "event.courseplan.toml"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let base = std::fs::read(base_path(&project)).unwrap();
    assert_eq!(
        base,
        strip_absolute_path(&std::fs::read(ppen(dir.path())).unwrap())
    );
    let text = std::fs::read_to_string(&project).unwrap();
    assert!(text.contains("[sync]"), "{text}");
    assert!(
        text.contains(&format!("base_fnv1a64 = \"{}\"", fnv1a64(&base))),
        "{text}"
    );
    assert!(text.contains("ppen = \"out.ppen\""), "{text}");
}

#[test]
fn first_sync_says_so_neutrally() {
    let (dir, _project) = linked();
    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--dry-run"],
    );
    let text = stdout(&out);
    assert!(text.contains("first sync"), "{text}");
    assert!(!text.contains("no usable base copy"), "{text}");
    let de = run(
        dir.path(),
        "de",
        &["sync", "event.courseplan.toml", "--dry-run"],
    );
    assert!(stdout(&de).contains("erster Abgleich"), "{}", stdout(&de));
}

#[test]
fn damaged_base_is_still_reported() {
    let (dir, project) = synced();
    std::fs::remove_file(base_path(&project)).unwrap();
    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--dry-run"],
    );
    let text = stdout(&out);
    assert!(text.contains("no usable base copy"), "{text}");
    assert!(!text.contains("first sync"), "{text}");
}

#[test]
fn dry_run_writes_nothing() {
    let (dir, project) = synced();
    edit(&project, P34, "map_mm = [-1.0, -12.4]");
    edit(
        &ppen(dir.path()),
        Q35,
        "<location x=\"-28.0\" y=\"-10.2\" />",
    );
    let before = files(dir.path(), &project);
    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--dry-run"],
    );
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("control/34/position"), "{text}");
    assert!(text.contains("control/35/position"), "{text}");
    assert!(text.contains("Dry run"), "{text}");
    assert_eq!(files(dir.path(), &project), before);
}

#[test]
fn delete_vs_edit_writes_nothing() {
    let (dir, project) = synced();
    delete_control_36(&project);
    edit(
        &ppen(dir.path()),
        Q36,
        "<location x=\"-52.0\" y=\"-47.6\" />",
    );
    let before = files(dir.path(), &project);
    let out = run(dir.path(), "en", &["sync", "event.courseplan.toml"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(text.contains("control/36"), "{text}");
    assert!(text.contains("deleted"), "{text}");
    assert!(text.contains("-52"), "{text}");
    assert_eq!(files(dir.path(), &project), before);
}

/// Both sides move controls 34 and 35 differently.
fn two_conflicts() -> (tempfile::TempDir, PathBuf) {
    let (dir, project) = synced();
    edit(&project, P34, "map_mm = [-1.0, -12.4]");
    edit(&project, P35, "map_mm = [-28.0, 10.2]");
    edit(&ppen(dir.path()), Q34, "<location x=\"-1.4\" y=\"12.4\" />");
    edit(
        &ppen(dir.path()),
        Q35,
        "<location x=\"-28.4\" y=\"-10.2\" />",
    );
    (dir, project)
}

#[test]
fn take_key_is_stored_until_complete() {
    let (dir, project) = two_conflicts();
    let ppen_before = std::fs::read(ppen(dir.path())).unwrap();
    let out = run(
        dir.path(),
        "en",
        &[
            "sync",
            "event.courseplan.toml",
            "--take",
            "control/34/position=theirs",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(text.contains("control/35/position"), "{text}");
    assert!(!text.contains("control/34/position differs"), "{text}");
    let stored = std::fs::read_to_string(&project).unwrap();
    assert_eq!(stored.matches("[[sync.decisions]]").count(), 1, "{stored}");
    assert_eq!(std::fs::read(ppen(dir.path())).unwrap(), ppen_before);

    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--take", "ours"],
    );
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let done = std::fs::read_to_string(&project).unwrap();
    assert!(!done.contains("sync.decisions"), "{done}");
    // The stored decision won for 34 (theirs), the bare side decided 35 (ours).
    assert!(done.contains("[-1.4, -12.4]"), "{done}");
    assert!(done.contains("[-28.0, 10.2]"), "{done}");
}

#[test]
fn an_unknown_take_key_is_refused() {
    let (dir, project) = two_conflicts();
    let before = files(dir.path(), &project);
    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--take", "control/99=ours"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stderr(&out).contains("control/99"), "{}", stderr(&out));
    assert_eq!(files(dir.path(), &project), before);
}

#[test]
fn json_output_has_conflicts() {
    let (dir, _project) = two_conflicts();
    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--json"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(json["conflicts"][0]["key"], "control/34/position");
    assert_eq!(json["written"], false);
    assert_eq!(json["dry_run"], false);
    for key in ["ppen_changed", "base", "changes", "notes", "stale"] {
        assert!(json.get(key).is_some(), "{key} missing: {json}");
    }
    assert!(json["conflicts"][0]["theirs"].is_string());
}

#[test]
fn json_output_lists_changes_of_a_clean_sync() {
    let (dir, _project) = synced();
    edit(
        &ppen(dir.path()),
        Q35,
        "<location x=\"-28.4\" y=\"-10.2\" />",
    );
    let out = run(
        dir.path(),
        "en",
        &["sync", "event.courseplan.toml", "--json"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let json: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(json["written"], true);
    assert_eq!(json["changes"][0]["key"], "control/35/position");
    assert_eq!(json["changes"][0]["side"], "theirs");
}

/// Both sides change something that does not collide.
fn clean_edits(dir: &Path, project: &Path) {
    edit(project, P34, "map_mm = [-1.0, -12.4]");
    edit(&ppen(dir), Q35, "<location x=\"-28.4\" y=\"-10.2\" />");
}

#[test]
fn crash_after_ppen_write_converges() {
    let (full, full_project) = synced();
    clean_edits(full.path(), &full_project);
    let (crashed, crashed_project) = synced();
    clean_edits(crashed.path(), &crashed_project);
    assert!(
        run(full.path(), "en", &["sync", "event.courseplan.toml"])
            .status
            .success()
    );
    // The crash: the new .ppen is there, the project and the base are the old ones.
    std::fs::copy(ppen(full.path()), ppen(crashed.path())).unwrap();
    let out = run(crashed.path(), "en", &["sync", "event.courseplan.toml"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert_eq!(
        std::fs::read(&crashed_project).unwrap(),
        std::fs::read(&full_project).unwrap()
    );
    assert_eq!(
        std::fs::read(base_path(&crashed_project)).unwrap(),
        std::fs::read(base_path(&full_project)).unwrap()
    );
}

#[test]
fn crash_after_project_write_repairs_base() {
    let (dir, project) = synced();
    clean_edits(dir.path(), &project);
    let old_base = std::fs::read(base_path(&project)).unwrap();
    assert!(
        run(dir.path(), "en", &["sync", "event.courseplan.toml"])
            .status
            .success()
    );
    let new_base = std::fs::read(base_path(&project)).unwrap();
    assert_ne!(new_base, old_base);
    // The crash: the base write never happened.
    std::fs::write(base_path(&project), &old_base).unwrap();
    let ppen_before = std::fs::read(ppen(dir.path())).unwrap();
    let project_before = std::fs::read(&project).unwrap();
    let out = run(dir.path(), "en", &["sync", "event.courseplan.toml"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    assert_eq!(std::fs::read(base_path(&project)).unwrap(), new_base);
    assert_eq!(std::fs::read(ppen(dir.path())).unwrap(), ppen_before);
    assert_eq!(std::fs::read(&project).unwrap(), project_before);
    let text = stdout(&out);
    assert!(text.contains("did not write the base copy"), "{text}");
    assert!(!text.contains("control/"), "{text}");
}

#[cfg(debug_assertions)]
#[test]
fn ppen_changed_meanwhile_aborts() {
    let (dir, project) = synced();
    clean_edits(dir.path(), &project);
    let before = files(dir.path(), &project);
    let out = command(dir.path(), "en", &["sync", "event.courseplan.toml"])
        .env("FREEMAPPER_SYNC_TEST_TOUCH", "1")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("changed while the sync ran"),
        "{}",
        stderr(&out)
    );
    let after = files(dir.path(), &project);
    // Only the touch itself changed the .ppen.
    assert_eq!(after[1], before[1]);
    assert_eq!(after[2], before[2]);
    let mut touched = before[0].clone().unwrap();
    touched.push(b'\n');
    assert_eq!(after[0].as_ref(), Some(&touched));
}

#[test]
fn no_link_is_refused() {
    let (dir, project) = linked();
    edit(&project, "ppen = \"out.ppen\"\n", "");
    let out = run(dir.path(), "en", &["sync", "event.courseplan.toml"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("no link to a Purple Pen file"),
        "{}",
        stderr(&out)
    );
    assert!(!base_path(&project).exists());
}

#[test]
fn a_missing_linked_file_is_refused() {
    let (dir, project) = linked();
    std::fs::remove_file(ppen(dir.path())).unwrap();
    let out = run(dir.path(), "en", &["sync", "event.courseplan.toml"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("out.ppen"), "{}", stderr(&out));
    assert!(!base_path(&project).exists());
}

#[test]
fn german_messages() {
    let (dir, _project) = synced();
    edit(
        &ppen(dir.path()),
        Q35,
        "<location x=\"-28.4\" y=\"-10.2\" />",
    );
    let out = run(dir.path(), "de", &["sync", "event.courseplan.toml"]);
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("Abgleich mit"), "{text}");
    let (dir, _project) = two_conflicts();
    let out = run(dir.path(), "de", &["sync", "event.courseplan.toml"]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert!(text.contains("control/34/position"), "{text}");
    assert!(text.contains("Projekt"), "{text}");
}

#[test]
fn help_is_localized() {
    let dir = tempfile::tempdir().unwrap();
    let en = run(dir.path(), "en", &["sync", "--help"]);
    let en = stdout(&en);
    assert!(en.contains("--dry-run") && en.contains("--take") && en.contains("--json"));
    let de = run(dir.path(), "de", &["sync", "--help"]);
    assert!(stdout(&de).contains("Seite"), "{}", stdout(&de));
}
