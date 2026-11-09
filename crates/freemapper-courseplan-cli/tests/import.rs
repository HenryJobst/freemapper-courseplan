//! `init --ppen` and `import --ppen` on the command line.

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

/// A 1:5000 sprint map with a point at two opposite corners around the controls of `lincoln-park.ppen`
/// (x 69 to 141 mm, y 35 to 123 mm, y up in the file).
const MAP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<map xmlns="http://openorienteering.org/apps/mapper/xml/v2" version="9">
<notes></notes>
<georeferencing scale="5000"></georeferencing>
<colors count="0"></colors>
<barrier version="6" required="0.6.0">
<symbols count="1" id="ISSprOM 2019"><symbol type="1" id="0" code="999" name="corner"><point_symbol/></symbol></symbols>
<parts count="1" current="0"><part name="default part"><objects count="2">
<object type="0" symbol="0"><coords count="1">60000 -130000;</coords></object>
<object type="0" symbol="0"><coords count="1">150000 -30000;</coords></object>
</objects></part></parts>
<templates count="0"></templates>
<view><grid/></view>
</barrier>
</map>
"#;

/// A temp dir with the map and a copy of `lincoln-park.ppen`, changed by `edit`.
fn setup(edit: impl FnOnce(String) -> String) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("park.omap"), MAP).unwrap();
    let ppen =
        std::fs::read_to_string(repo_root().join("tests/fixtures/ppen/lincoln-park.ppen")).unwrap();
    std::fs::write(dir.path().join("park.ppen"), edit(ppen)).unwrap();
    dir
}

fn plain() -> tempfile::TempDir {
    setup(|s| s)
}

fn project(dir: &Path) -> PathBuf {
    dir.join("park.courseplan.toml")
}

fn init_empty(dir: &Path) {
    let out = run(dir, "en", &["init", "park.omap"]);
    assert!(out.status.success(), "{}", stderr(&out));
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn init_ppen_writes_controls_courses_and_the_link() {
    let dir = plain();
    let out = run(
        dir.path(),
        "en",
        &["init", "park.omap", "--ppen", "park.ppen"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let shown = stdout(&out);
    assert!(shown.contains("park.courseplan.toml"), "{shown}");
    assert!(
        shown.contains("imported 2 controls and 1 course from park.ppen"),
        "{shown}"
    );
    let toml = read(&project(dir.path()));
    assert!(toml.contains(r#"ppen = "park.ppen""#), "{toml}");
    assert!(toml.contains(r#"name = "Short""#), "{toml}");
    assert!(toml.contains("31"), "{toml}");
    let info = run(dir.path(), "en", &["info", "park.courseplan.toml"]);
    assert!(info.status.success(), "{}", stderr(&info));
}

#[test]
fn notes_come_in_the_language_of_the_user() {
    let dir = plain();
    let out = run(
        dir.path(),
        "en",
        &["init", "park.omap", "--ppen", "park.ppen"],
    );
    assert!(
        stdout(&out).contains("note: course Short has circle or leg gaps"),
        "{}",
        stdout(&out)
    );
    let dir = plain();
    let out = run(
        dir.path(),
        "de",
        &["init", "park.omap", "--ppen", "park.ppen"],
    );
    let shown = stdout(&out);
    assert!(
        shown.contains("2 Posten und 1 Bahn aus park.ppen übernommen"),
        "{shown}"
    );
    assert!(
        shown.contains("Hinweis: Bahn Short hat Lücken in Kreisen oder Strecken"),
        "{shown}"
    );
}

#[test]
fn import_into_an_empty_project_works() {
    let dir = plain();
    init_empty(dir.path());
    let out = run(
        dir.path(),
        "en",
        &["import", "park.courseplan.toml", "--ppen", "park.ppen"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let toml = read(&project(dir.path()));
    assert!(toml.contains(r#"ppen = "park.ppen""#), "{toml}");
    assert!(toml.contains(r#"name = "Short""#), "{toml}");
}

#[test]
fn import_into_a_project_with_content_needs_replace_and_changes_nothing() {
    let dir = plain();
    init_empty(dir.path());
    let args = ["import", "park.courseplan.toml", "--ppen", "park.ppen"];
    assert!(run(dir.path(), "en", &args).status.success());
    let before = read(&project(dir.path()));
    let out = run(dir.path(), "en", &args);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("run again with --replace"),
        "{}",
        stderr(&out)
    );
    assert_eq!(read(&project(dir.path())), before);
}

#[test]
fn replace_swaps_the_content_and_keeps_classes_and_findings() {
    let dir = plain();
    init_empty(dir.path());
    let args = ["import", "park.courseplan.toml", "--ppen", "park.ppen"];
    assert!(run(dir.path(), "en", &args).status.success());
    let mut toml = read(&project(dir.path()));
    toml.push_str(
        "\n[[classes]]\nname = \"Open\"\n\n[[accepted_findings]]\nrule = \"wkb26.test\"\nnote = \"intended\"\n",
    );
    std::fs::write(project(dir.path()), toml).unwrap();
    // The event now holds another course name.
    let ppen =
        read(&dir.path().join("park.ppen")).replace("<name>Short</name>", "<name>Kurz</name>");
    std::fs::write(dir.path().join("park.ppen"), ppen).unwrap();
    let out = run(
        dir.path(),
        "en",
        &[
            "import",
            "park.courseplan.toml",
            "--ppen",
            "park.ppen",
            "--replace",
        ],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let toml = read(&project(dir.path()));
    assert!(toml.contains(r#"name = "Kurz""#), "{toml}");
    assert!(!toml.contains(r#"name = "Short""#), "{toml}");
    assert!(toml.contains(r#"name = "Open""#), "{toml}");
    assert!(toml.contains("wkb26.test"), "{toml}");
}

#[test]
fn a_scale_mismatch_names_both_scales_and_writes_nothing() {
    let dir = setup(|s| s.replace(r#"scale="5000""#, r#"scale="4000""#));
    let out = run(
        dir.path(),
        "en",
        &["init", "park.omap", "--ppen", "park.ppen"],
    );
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.contains("1:4000") && err.contains("1:5000"), "{err}");
    assert!(!project(dir.path()).exists());
}

#[test]
fn an_invalid_result_is_refused_and_leaves_the_project_untouched() {
    // An out-of-bounds area of two points, which the model refuses.
    let dir = setup(|s| {
        s.replace(
            "</course-scribe-event>",
            r#"<special-object id="9" kind="out-of-bounds"><location x="80" y="80" /><location x="90" y="90" /><courses all="true" /></special-object></course-scribe-event>"#,
        )
    });
    init_empty(dir.path());
    let before = read(&project(dir.path()));
    let out = run(
        dir.path(),
        "en",
        &["import", "park.courseplan.toml", "--ppen", "park.ppen"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("OOB1"), "{}", stderr(&out));
    assert_eq!(read(&project(dir.path())), before);
    let out = run(
        dir.path(),
        "en",
        &[
            "init",
            "park.omap",
            "--ppen",
            "park.ppen",
            "--project",
            "other.toml",
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(!dir.path().join("other.toml").exists());
}

#[test]
fn a_missing_ppen_file_is_named() {
    let dir = plain();
    init_empty(dir.path());
    let out = run(
        dir.path(),
        "en",
        &["import", "park.courseplan.toml", "--ppen", "nope.ppen"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("nope.ppen"), "{}", stderr(&out));
}
