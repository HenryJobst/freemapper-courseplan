//! `export --iof-xml` and `export --ppen` on the command line.

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

// ---------------------------------------------------------------------------------------------
// export --ppen

/// The example project with `sources.ppen = link`.
fn example_linked(link: &str) -> (tempfile::TempDir, PathBuf) {
    let (dir, project) = example("");
    let text = std::fs::read_to_string(&project).unwrap();
    let text = text.replace(
        "map = \"luebeck.omap\"",
        &format!("map = \"luebeck.omap\"\nppen = \"{link}\""),
    );
    assert!(text.contains("ppen ="), "{text}");
    std::fs::write(&project, text).unwrap();
    (dir, project)
}

fn export_ppen(cwd: &Path, lang: &str, project: &Path, target: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "export",
        project.to_str().unwrap(),
        "--ppen",
        target.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    run(cwd, lang, &args)
}

#[test]
fn export_ppen_writes_and_links() {
    let (dir, project) = example_linked("out.ppen");
    let target = dir.path().join("out.ppen");
    let before = std::fs::read_to_string(&project).unwrap();
    let result = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert!(result.status.success(), "{}", stderr(&result));
    let text = stdout(&result);
    assert!(
        text.contains("wrote 1 course to the new Purple Pen event"),
        "{text}"
    );
    assert!(text.contains("stored a Purple Pen id for"), "{text}");
    let ppen = std::fs::read_to_string(&target).unwrap();
    assert!(ppen.contains("<course-scribe-event"), "{ppen}");
    let after = std::fs::read_to_string(&project).unwrap();
    assert_ne!(before, after);
    assert!(after.contains("ppen_id"), "{after}");
    assert!(after.contains("ppen = \"out.ppen\""), "{after}");
    assert!(!dir.path().join("out.ppen.tmp").exists());
    // The second export patches the file and changes neither it nor the project.
    let ppen_before = std::fs::read(&target).unwrap();
    let again = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert!(again.status.success(), "{}", stderr(&again));
    assert!(
        stdout(&again).contains("into the Purple Pen event"),
        "{}",
        stdout(&again)
    );
    assert_eq!(std::fs::read(&target).unwrap(), ppen_before);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), after);
}

#[test]
fn a_new_file_links_the_project() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    let result = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert!(result.status.success(), "{}", stderr(&result));
    let linked = std::fs::read_to_string(&project).unwrap();
    assert!(linked.contains("ppen = \"out.ppen\""), "{linked}");
    assert!(linked.contains("ppen_id"), "{linked}");
    // Move the start: the next export moves the one start of the file instead of adding a second.
    let moved = linked.replace("map_mm = [-97.4, -3.6]", "map_mm = [-90.0, -3.6]");
    assert_ne!(moved, linked);
    std::fs::write(&project, &moved).unwrap();
    let before = std::fs::read_to_string(&target).unwrap();
    let again = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert!(again.status.success(), "{}", stderr(&again));
    let after = std::fs::read_to_string(&target).unwrap();
    assert_eq!(after.matches("kind=\"start\"").count(), 1, "{after}");
    let location = |text: &str| -> String {
        let at = text.find("kind=\"start\"").unwrap();
        let rest = &text[at..];
        let from = rest.find("<location").unwrap();
        rest[from..from + rest[from..].find("/>").unwrap()].to_owned()
    };
    assert_ne!(location(&after), location(&before));
    assert!(
        location(&after).contains("x=\"-90\""),
        "{}",
        location(&after)
    );
    assert!(!stdout(&again).contains("STA1"), "{}", stdout(&again));
}

#[test]
fn new_links_the_project_too() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    std::fs::write(&target, "not a purple pen event").unwrap();
    let result = export_ppen(dir.path(), "en", &project, &target, &["--new"]);
    assert!(result.status.success(), "{}", stderr(&result));
    let text = std::fs::read_to_string(&project).unwrap();
    assert!(text.contains("ppen = \"out.ppen\""), "{text}");
}

#[test]
fn a_linked_project_keeps_its_link_on_a_new_file() {
    let (dir, project) = example_linked("a.ppen");
    let a = dir.path().join("a.ppen");
    assert!(
        export_ppen(dir.path(), "en", &project, &a, &[])
            .status
            .success()
    );
    let linked = std::fs::read_to_string(&project).unwrap();
    assert!(linked.contains("ppen = \"a.ppen\""), "{linked}");
    // A missing file and --new onto another existing file: neither moves the link or stores ids.
    let c = dir.path().join("c.ppen");
    let d = dir.path().join("d.ppen");
    std::fs::write(&d, "not a purple pen event").unwrap();
    for (target, extra) in [(&c, &[][..]), (&d, &["--new"][..])] {
        let result = export_ppen(dir.path(), "en", &project, target, extra);
        assert!(result.status.success(), "{}", stderr(&result));
        assert!(target.exists());
        assert!(
            !stdout(&result).contains("sources.ppen"),
            "{}",
            stdout(&result)
        );
        assert_eq!(std::fs::read_to_string(&project).unwrap(), linked);
    }
    // --new onto the linked file stays linked to it.
    let result = export_ppen(dir.path(), "en", &project, &a, &["--new"]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        std::fs::read_to_string(&project)
            .unwrap()
            .contains("ppen = \"a.ppen\"")
    );
}

#[test]
fn an_existing_foreign_file_does_not_link() {
    let (dir, project) = example("");
    // The file comes from a copy of the project, which it links; the project itself stays unlinked.
    let copy = dir.path().join("copy.courseplan.toml");
    std::fs::copy(&project, &copy).unwrap();
    let target = dir.path().join("out.ppen");
    assert!(
        export_ppen(dir.path(), "en", &copy, &target, &[])
            .status
            .success()
    );
    let before = std::fs::read_to_string(&project).unwrap();
    let file_before = std::fs::read(&target).unwrap();
    let result = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    assert_eq!(std::fs::read(&target).unwrap(), file_before);
}

/// `text` with the Purple Pen ids `a` and `b` of two normal controls swapped, codes kept: the control
/// elements and every reference to them.
fn swap_control_ids(text: &str, a: u32, b: u32) -> String {
    let mut out = text.to_owned();
    for (pattern, hold) in [
        ("<control id=\"{}\"", "<control id=\"#{}\""),
        (" control=\"{}\"", " control=\"#{}\""),
        ("start-control=\"{}\"", "start-control=\"#{}\""),
        ("end-control=\"{}\"", "end-control=\"#{}\""),
    ] {
        let of = |p: &str, n: u32| p.replace("{}", &n.to_string());
        out = out
            .replace(&of(pattern, a), &of(hold, b))
            .replace(&of(pattern, b), &of(hold, a));
    }
    out.replace("=\"#", "=\"")
}

#[test]
fn ids_only_for_the_linked_file() {
    let (dir, project) = example_linked("a.ppen");
    let a = dir.path().join("a.ppen");
    let b = dir.path().join("b.ppen");
    assert!(
        export_ppen(dir.path(), "en", &project, &a, &[])
            .status
            .success()
    );
    // b.ppen holds the same controls, but 31 and 32 have each other's Purple Pen id. Ids taken over
    // from the project would pair 31 with 32 there.
    let text = std::fs::read_to_string(&a).unwrap();
    assert!(
        text.contains("<control id=\"3\" kind=\"normal\">\r\n    <code>31</code>"),
        "{text}"
    );
    assert!(
        text.contains("<control id=\"4\" kind=\"normal\">\r\n    <code>32</code>"),
        "{text}"
    );
    let swapped = swap_control_ids(&text, 3, 4);
    assert!(swapped.contains("<control id=\"4\" kind=\"normal\">\r\n    <code>31</code>"));
    std::fs::write(&b, &swapped).unwrap();
    let linked = std::fs::read_to_string(&project).unwrap();
    let result = export_ppen(dir.path(), "en", &project, &b, &[]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        !stdout(&result).contains("stored a Purple Pen id"),
        "{}",
        stdout(&result)
    );
    // The project keeps the ids of a.ppen; b.ppen keeps its own, and every control its code and place.
    assert_eq!(std::fs::read_to_string(&project).unwrap(), linked);
    assert_eq!(std::fs::read_to_string(&b).unwrap(), swapped);
}

#[test]
fn the_same_file_for_both_formats_is_refused() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    assert!(
        export_ppen(dir.path(), "en", &project, &target, &[])
            .status
            .success()
    );
    let before = std::fs::read(&target).unwrap();
    for lang in ["en", "de"] {
        let result = run(
            dir.path(),
            lang,
            &[
                "export",
                project.to_str().unwrap(),
                "--iof-xml",
                target.to_str().unwrap(),
                "--ppen",
                dir.path().join(".").join("out.ppen").to_str().unwrap(),
            ],
        );
        assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
        let want = if lang == "en" {
            "the same file"
        } else {
            "dieselbe Datei"
        };
        assert!(stderr(&result).contains(want), "{}", stderr(&result));
        assert_eq!(std::fs::read(&target).unwrap(), before);
    }
}

#[test]
fn a_directory_or_an_oversized_file_is_not_read() {
    let (dir, project) = example("");
    let folder = dir.path().join("folder.ppen");
    std::fs::create_dir(&folder).unwrap();
    let result = export_ppen(dir.path(), "en", &project, &folder, &[]);
    assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
    assert!(
        stderr(&result).contains("is not a file"),
        "{}",
        stderr(&result)
    );
    // A sparse file one byte over the limit of 50 MB.
    let big = dir.path().join("big.ppen");
    std::fs::File::create(&big)
        .unwrap()
        .set_len(50 * 1024 * 1024 + 1)
        .unwrap();
    for lang in ["en", "de"] {
        let result = export_ppen(dir.path(), lang, &project, &big, &[]);
        assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
        let want = if lang == "en" {
            "larger than 50 MB"
        } else {
            "größer als 50 MB"
        };
        assert!(stderr(&result).contains(want), "{}", stderr(&result));
    }
    // The import reads the same way.
    let result = run(
        dir.path(),
        "en",
        &[
            "import",
            project.to_str().unwrap(),
            "--ppen",
            folder.to_str().unwrap(),
            "--replace",
        ],
    );
    assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
    assert!(
        stderr(&result).contains("is not a file"),
        "{}",
        stderr(&result)
    );
}

#[cfg(unix)]
#[test]
fn a_failed_project_save_says_the_ppen_was_written() {
    use std::os::unix::fs::PermissionsExt;
    let (dir, project) = example("");
    let out = tempfile::tempdir().unwrap();
    let target = out.path().join("out.ppen");
    // The project folder takes no new file, so the atomic save of the project fails.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let results: Vec<Output> = ["en", "de"]
        .iter()
        .map(|lang| export_ppen(dir.path(), lang, &project, &target, &["--new"]))
        .collect();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    for (result, want) in results.iter().zip(["but the project", "aber das Projekt"]) {
        assert_eq!(result.status.code(), Some(1), "{}", stdout(result));
        assert!(stderr(result).contains(want), "{}", stderr(result));
        assert!(stderr(result).contains("out.ppen"), "{}", stderr(result));
    }
    assert!(target.exists());
}

#[test]
fn a_comment_in_the_base_does_not_crash_the_export() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    assert!(
        export_ppen(dir.path(), "en", &project, &target, &[])
            .status
            .success()
    );
    let text = std::fs::read_to_string(&target).unwrap().replacen(
        "<course-scribe-event",
        "<!-- <standards description=\u{e9}> --><course-scribe-event",
        1,
    );
    std::fs::write(&target, &text).unwrap();
    let result = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert_eq!(std::fs::read_to_string(&target).unwrap(), text);
}

#[test]
fn export_ppen_new_ignores_the_existing_file() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    std::fs::write(&target, "not a purple pen event").unwrap();
    let broken = export_ppen(dir.path(), "en", &project, &target, &[]);
    assert_eq!(broken.status.code(), Some(1), "{}", stdout(&broken));
    assert!(
        stderr(&broken).contains("cannot be patched"),
        "{}",
        stderr(&broken)
    );
    assert!(stderr(&broken).contains("out.ppen"), "{}", stderr(&broken));
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "not a purple pen event"
    );
    let result = export_ppen(dir.path(), "en", &project, &target, &["--new"]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        std::fs::read_to_string(&target)
            .unwrap()
            .contains("<course-scribe-event")
    );
}

#[test]
fn new_needs_ppen() {
    let (dir, project) = example("");
    let result = run(
        dir.path(),
        "en",
        &["export", project.to_str().unwrap(), "--new"],
    );
    assert!(!result.status.success());
}

#[test]
fn export_ppen_refuses_the_project_and_map_as_target() {
    let (dir, project) = example("");
    let map = dir.path().join("luebeck.omap");
    let (project_before, map_before) = (
        std::fs::read(&project).unwrap(),
        std::fs::read(&map).unwrap(),
    );
    for target in [&project, &map] {
        let result = export_ppen(dir.path(), "en", &project, target, &["--new"]);
        assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
        assert!(
            stderr(&result).contains("would overwrite it"),
            "{}",
            stderr(&result)
        );
    }
    assert_eq!(std::fs::read(&project).unwrap(), project_before);
    assert_eq!(std::fs::read(&map).unwrap(), map_before);
}

#[test]
fn refused_course_and_team_named_in_de_and_en() {
    let (dir, project) = example(TEAM);
    let target = dir.path().join("out.ppen");
    let en = export_ppen(dir.path(), "en", &project, &target, &["--new"]);
    assert!(en.status.success(), "{}", stderr(&en));
    assert!(
        stdout(&en).contains("note: left out Team: team courses are not exported to Purple Pen"),
        "{}",
        stdout(&en)
    );
    let de = export_ppen(dir.path(), "de", &project, &target, &["--new"]);
    assert!(de.status.success(), "{}", stderr(&de));
    assert!(
        stdout(&de).contains("Hinweis: Team ausgelassen: Bahnart Mannschaftsbahn wird nicht nach Purple Pen exportiert"),
        "{}",
        stdout(&de)
    );
    assert!(!std::fs::read_to_string(&target).unwrap().contains("Team"));
}

#[test]
fn unknown_version_base_exits_non_zero_and_writes_nothing() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    assert!(
        export_ppen(dir.path(), "en", &project, &target, &[])
            .status
            .success()
    );
    let base = std::fs::read_to_string(&target).unwrap().replacen(
        "<course-scribe-event>",
        "<course-scribe-event version=\"9\">",
        1,
    );
    assert!(base.contains("version=\"9\""), "{base}");
    std::fs::write(&target, &base).unwrap();
    let project_before = std::fs::read(&project).unwrap();
    for lang in ["en", "de"] {
        let result = export_ppen(dir.path(), lang, &project, &target, &[]);
        assert_eq!(result.status.code(), Some(1), "{}", stdout(&result));
        assert!(!stderr(&result).is_empty());
        assert_eq!(std::fs::read_to_string(&target).unwrap(), base);
        assert_eq!(std::fs::read(&project).unwrap(), project_before);
    }
}

#[test]
fn a_kept_object_is_reported_with_the_never_deletes_hint() {
    let (dir, project) = example("");
    let target = dir.path().join("out.ppen");
    assert!(
        export_ppen(dir.path(), "en", &project, &target, &[])
            .status
            .success()
    );
    // A project with one control fewer: the control stays in the file.
    let text = std::fs::read_to_string(&project).unwrap();
    let other = dir.path().join("other.courseplan.toml");
    let reduced = reduced_project(&text);
    std::fs::write(&other, reduced).unwrap();
    let result = export_ppen(dir.path(), "en", &other, &target, &[]);
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(
        stdout(&result).contains("it never deletes"),
        "{}",
        stdout(&result)
    );
}

/// The project text without its courses.
fn reduced_project(text: &str) -> String {
    let at = text.find("[[courses]]").expect("the example has a course");
    text[..at].to_owned()
}

#[test]
fn both_formats_in_one_call_are_written() {
    let (dir, project) = example("");
    let xml = dir.path().join("o.xml");
    let ppen = dir.path().join("o.ppen");
    let result = run(
        dir.path(),
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--iof-xml",
            xml.to_str().unwrap(),
            "--ppen",
            ppen.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", stderr(&result));
    assert!(xml.exists() && ppen.exists());
}
