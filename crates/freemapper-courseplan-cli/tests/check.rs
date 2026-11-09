//! `check`, `finding accept`/`remove` and `event profile` on the command line.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn run_lang(cwd: &Path, lang: &str, args: &[&str]) -> Output {
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

fn run(cwd: &Path, args: &[&str]) -> Output {
    run_lang(cwd, "en", args)
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn code(o: &Output) -> Option<i32> {
    o.status.code()
}

/// A temp dir with the Luebeck map and a project made by `init`, with start, finish, controls
/// 31 and 32 and course A. The points lie in an open part of the map, so the map-based checks
/// find nothing.
fn project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("luebeck.omap");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/luebeck.omap"),
        &map,
    )
    .unwrap();
    let out = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let project = dir.path().join("luebeck.courseplan.toml");
    let p = project.to_str().unwrap();
    for args in [
        vec![
            "special", "add", p, "--kind", "start", "--id", "S1", "--at", "88,82",
        ],
        vec![
            "special", "add", p, "--kind", "finish", "--id", "F1", "--at", "118,82",
        ],
        vec!["control", "add", p, "--at", "93,89"],
        vec!["control", "add", p, "--at", "113,89"],
        vec![
            "course",
            "add",
            p,
            "A",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "31,32",
        ],
    ] {
        let out = run(dir.path(), &args);
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    }
    (dir, project)
}

#[test]
fn check_of_a_clean_event_exits_0() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let out = run(dir.path(), &["check", p]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    let summary = text.lines().find(|l| l.starts_with("0 errors")).unwrap();
    assert!(
        summary.contains("0 warnings, 0 hints; 0 accepted"),
        "{text}"
    );
    assert!(text.contains("--all"), "{text}");
    assert!(!text.contains("not implemented yet"), "{text}");

    let all = run(dir.path(), &["check", p, "--all"]);
    assert_eq!(code(&all), Some(0));
    let text = stdout(&all);
    assert!(text.contains("skipped:"), "{text}");
    assert!(
        text.contains("wkb2026.code.range, event: this check is not implemented yet"),
        "{text}"
    );
}

#[test]
fn check_speaks_german() {
    let (dir, project) = project();
    let out = run_lang(
        dir.path(),
        "de",
        &["check", project.to_str().unwrap(), "--all"],
    );
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("übersprungen"), "{text}");
    assert!(text.contains("noch nicht umgesetzt"), "{text}");
    assert!(text.contains("0 Fehler, 0 Warnungen, 0 Hinweise"), "{text}");
}

#[test]
fn check_exit_codes() {
    let (dir, project) = project();
    let d = dir.path();
    let missing = run(d, &["check", "nothing.toml"]);
    assert_eq!(code(&missing), Some(1));
    assert_eq!(code(&run(d, &["check"])), Some(2));
    assert_eq!(code(&run(d, &["check", "x.toml", "--bogus"])), Some(2));

    // A profile the program does not have refuses the check and lists the available ones.
    let text = std::fs::read_to_string(&project).unwrap();
    let text = text.replace("rule_profile = \"wkb-2026\"", "rule_profile = \"wkb-1999\"");
    std::fs::write(&project, text).unwrap();
    let out = run(d, &["check", project.to_str().unwrap()]);
    assert_eq!(code(&out), Some(1));
    assert!(
        stderr(&out).contains("wkb-1999") && stderr(&out).contains("wkb-2026"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn an_error_finding_exits_1_until_it_is_accepted() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    // Control 33 stands 1.5 mm (6 m at 1:4000) from 31; course B runs to it.
    for args in [
        vec!["control", "add", p, "--at", "93,90.5"],
        vec![
            "course",
            "add",
            p,
            "B",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "33",
        ],
    ] {
        let out = run(d, &args);
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    }
    let out = run(d, &["check", p]);
    assert_eq!(code(&out), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains(
            "error controls 31, 33: control 31 and control 33 are 6 m apart, at least 15 m \
             (wkb2026.distance.min_half, WKB26 B 7.2.1)"
        ),
        "{}",
        stdout(&out)
    );
    let de = run_lang(d, "de", &["check", p]);
    assert!(
        stdout(&de).contains("Posten 31 und Posten 33 liegen 6 m auseinander, mindestens 15 m"),
        "{}",
        stdout(&de)
    );

    let accept = [
        "finding",
        "accept",
        p,
        "wkb2026.distance.min_half",
        "--controls",
        "33,31",
        "--note",
        "two flags on purpose",
    ];
    assert_eq!(code(&run(d, &accept)), Some(0));
    let out = run(d, &["check", p]);
    assert_eq!(code(&out), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).contains("0 errors"), "{}", stdout(&out));
    assert!(stdout(&out).contains("1 accepted"), "{}", stdout(&out));
}

#[test]
fn check_help_names_the_exit_codes() {
    let (dir, _) = project();
    let en = stdout(&run(dir.path(), &["help", "check"]));
    assert!(en.contains("Exit code 0"), "{en}");
    let de = stdout(&run_lang(dir.path(), "de", &["check", "--help"]));
    assert!(de.contains("Exit-Code 0"), "{de}");
}

#[test]
fn finding_accept_replace_and_remove() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    // A comment in the file survives the edits.
    let text = std::fs::read_to_string(&project).unwrap();
    std::fs::write(&project, format!("# planned by H.\n{text}")).unwrap();

    let accept = |note: &str| {
        let args = [
            "finding",
            "accept",
            p,
            "wkb2026.distance.min_half",
            "--controls",
            "32, 31",
            "--note",
            note,
        ];
        run(d, &args)
    };
    let first = accept("on purpose");
    assert_eq!(code(&first), Some(0), "{}", stderr(&first));
    assert_eq!(
        stdout(&first).trim(),
        "accepted finding wkb2026.distance.min_half at controls 31, 32"
    );
    let again = accept("still on purpose");
    assert_eq!(code(&again), Some(0));
    assert!(
        stdout(&again).contains("replaced the note"),
        "{}",
        stdout(&again)
    );
    let text = std::fs::read_to_string(&project).unwrap();
    assert!(text.starts_with("# planned by H.\n"), "{text}");
    assert_eq!(text.matches("[[accepted_findings]]").count(), 1, "{text}");
    assert!(
        text.contains("controls = [31, 32]") && text.contains("still on purpose"),
        "{text}"
    );

    let leg = run(
        d,
        &[
            "finding",
            "accept",
            p,
            "wkb2026.leg.through_control",
            "--leg",
            "A:2",
            "--note",
            "n",
        ],
    );
    assert_eq!(code(&leg), Some(0), "{}", stderr(&leg));
    assert!(
        std::fs::read_to_string(&project)
            .unwrap()
            .contains("legs = [{ course = \"A\", after = 2 }]")
    );
    // A finding at two legs: --leg twice, in any order.
    let two = run(
        d,
        &[
            "finding",
            "accept",
            p,
            "wkb2026.leg.opposing",
            "--leg",
            "A:2",
            "--leg",
            "A:0",
            "--note",
            "n",
        ],
    );
    assert_eq!(code(&two), Some(0), "{}", stderr(&two));
    assert_eq!(
        stdout(&two).trim(),
        "accepted finding wkb2026.leg.opposing at course A, leg from the start and leg after 2"
    );
    assert!(
        std::fs::read_to_string(&project)
            .unwrap()
            .contains("legs = [{ course = \"A\", after = 0 }, { course = \"A\", after = 2 }]")
    );

    let remove = |args: &[&str]| {
        let mut all = vec!["finding", "remove", p];
        all.extend_from_slice(args);
        run(d, &all)
    };
    let removed = remove(&["wkb2026.distance.min_half", "--controls", "31,32"]);
    assert_eq!(code(&removed), Some(0), "{}", stderr(&removed));
    let before = std::fs::read(&project).unwrap();
    let missing = remove(&["wkb2026.distance.min_half", "--controls", "31,32"]);
    assert_eq!(code(&missing), Some(1));
    assert!(
        stderr(&missing).contains("no accepted finding"),
        "{}",
        stderr(&missing)
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn finding_accept_refusals() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    let before = std::fs::read(&project).unwrap();
    let accept = |args: &[&str]| {
        let mut all = vec!["finding", "accept", p];
        all.extend_from_slice(args);
        run(d, &all)
    };
    // Distance findings never sit at the event as a whole.
    let nowhere = accept(&["wkb2026.distance.min_half", "--note", "n"]);
    assert_eq!(code(&nowhere), Some(1));
    assert!(
        stderr(&nowhere).contains("name that place"),
        "{}",
        stderr(&nowhere)
    );
    let unknown = accept(&["wkb2026.no.such", "--controls", "31", "--note", "n"]);
    assert_eq!(code(&unknown), Some(1));
    let mixed = accept(&[
        "wkb2026.distance.min",
        "--controls",
        "31",
        "--course",
        "A",
        "--note",
        "n",
    ]);
    assert_eq!(code(&mixed), Some(1));
    let bad_leg = accept(&["wkb2026.leg.through_control", "--leg", "A3", "--note", "n"]);
    assert_eq!(code(&bad_leg), Some(2), "{}", stderr(&bad_leg));
    let no_note = accept(&["wkb2026.distance.min", "--controls", "31"]);
    assert_eq!(code(&no_note), Some(2));
    // Distance findings sit at two controls, never at a course: such an acceptance could never
    // match.
    let at_course = accept(&["wkb2026.distance.min", "--course", "A", "--note", "n"]);
    assert_eq!(code(&at_course), Some(1));
    assert_eq!(
        stderr(&at_course).trim(),
        "error: findings of rule wkb2026.distance.min sit at two or more controls, never at a course"
    );
    let de = run_lang(
        d,
        "de",
        &[
            "finding",
            "accept",
            p,
            "wkb2026.map.edge",
            "--leg",
            "A:1",
            "--note",
            "n",
        ],
    );
    assert_eq!(code(&de), Some(1));
    assert!(
        stderr(&de).contains("betreffen einen Posten, nie eine Strecke"),
        "{}",
        stderr(&de)
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);

    // A start-capacity finding can sit at the event.
    let event = accept(&["wkb2026.start.capacity", "--note", "one start box"]);
    assert_eq!(code(&event), Some(0), "{}", stderr(&event));
    assert!(
        stdout(&event).contains("at the event"),
        "{}",
        stdout(&event)
    );
}

#[test]
fn event_profile_sets_and_validates() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    let info = |lang| {
        stdout(&run_lang(d, lang, &["info", p]))
            .lines()
            .find(|l| l.starts_with("rule profile") || l.starts_with("Regelprofil"))
            .unwrap()
            .to_owned()
    };
    assert_eq!(info("en"), "rule profile: wkb-2026");

    let set = run(
        d,
        &[
            "event",
            "profile",
            p,
            "wkb-2026",
            "--overlay",
            "nor-bb-2021",
        ],
    );
    assert_eq!(code(&set), Some(0), "{}", stderr(&set));
    assert_eq!(
        stdout(&set).trim(),
        "set rule profile wkb-2026 with overlays nor-bb-2021"
    );
    assert_eq!(info("en"), "rule profile: wkb-2026, overlays: nor-bb-2021");
    assert_eq!(info("de"), "Regelprofil: wkb-2026, Aufsätze: nor-bb-2021");

    let before = std::fs::read(&project).unwrap();
    let unknown = run(d, &["event", "profile", p, "wkb-1999"]);
    assert_eq!(code(&unknown), Some(1));
    assert!(
        stderr(&unknown).contains("nor-bb-2021"),
        "{}",
        stderr(&unknown)
    );
    let overlay_as_base = run(d, &["event", "profile", p, "nor-bb-2021"]);
    assert_eq!(code(&overlay_as_base), Some(1));
    assert_eq!(std::fs::read(&project).unwrap(), before);

    let plain = run(d, &["event", "profile", p, "wkb-2026"]);
    assert_eq!(stdout(&plain).trim(), "set rule profile wkb-2026");
    assert!(
        !std::fs::read_to_string(&project)
            .unwrap()
            .contains("rule_overlays")
    );
}
