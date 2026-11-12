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
        text.contains("wkb2026.start.capacity, event: the event has no start window"),
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
    assert!(text.contains("kein Startfenster"), "{text}");
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
             required (wkb2026.distance.min_half, WKB26 B 7.2.1)"
        ),
        "{}",
        stdout(&out)
    );
    let de = run_lang(d, "de", &["check", p]);
    assert!(
        stdout(&de).contains(
            "Posten 31 und Posten 33 liegen 6 m auseinander, gefordert sind mindestens 15 m"
        ),
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
fn a_control_in_an_out_of_bounds_special_is_an_error_until_it_is_accepted() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    for args in [
        vec![
            "special",
            "add",
            p,
            "--kind",
            "out-of-bounds",
            "--id",
            "OB1",
            "--point",
            "98,84",
            "--point",
            "108,84",
            "--point",
            "108,94",
            "--point",
            "98,94",
        ],
        vec!["control", "add", p, "--at", "103,89"],
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
            "error control 33: control 33 stands inside the out-of-bounds area OB1, at 103, 89 mm on the map \
             (wkb2026.control.in_forbidden_sprint, WKB26 A 2.2)"
        ),
        "{}",
        stdout(&out)
    );
    let de = run_lang(d, "de", &["check", p]);
    assert!(
        stdout(&de).contains("Posten 33 steht im Sperrgebiet OB1, bei 103, 89 mm auf der Karte"),
        "{}",
        stdout(&de)
    );

    let accept = [
        "finding",
        "accept",
        p,
        "wkb2026.control.in_forbidden_sprint",
        "--controls",
        "33",
        "--note",
        "the area is lifted for this event",
    ];
    assert_eq!(code(&run(d, &accept)), Some(0));
    let out = run(d, &["check", p]);
    assert_eq!(code(&out), Some(0), "{}", stdout(&out));
    assert!(stdout(&out).contains("1 accepted"), "{}", stdout(&out));
}

#[test]
fn a_control_on_a_map_area_with_an_out_of_bounds_symbol_is_an_error() {
    // Gleisdreieck has a 714 area (closed area) around (88.2, -82.8) map mm.
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("gleisdreieck.omap");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/gleisdreieck/gleisdreieck.omap"),
        &map,
    )
    .unwrap();
    let out = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let project = dir.path().join("gleisdreieck.courseplan.toml");
    let p = project.to_str().unwrap();
    for args in [
        vec![
            "special",
            "add",
            p,
            "--kind",
            "start",
            "--id",
            "S1",
            "--at=100,-290",
        ],
        vec![
            "special",
            "add",
            p,
            "--kind",
            "finish",
            "--id",
            "F1",
            "--at=100,-275",
        ],
        vec!["control", "add", p, "--at=88.2,-82.8"],
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
            "31",
        ],
    ] {
        let out = run(dir.path(), &args);
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    }
    let out = run(dir.path(), &["check", p]);
    assert_eq!(code(&out), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains(
            "control 31 stands inside a map area with the out-of-bounds symbol 714, at 88.2, -82.8 mm on the map"
        ),
        "{}",
        stdout(&out)
    );
}

/// The Rummelsburger Bucht example and the same project with the finish moved into the blank part of the
/// sheet beside the diagonal strip of terrain (ADR 0084). `map_edge` does not see it, the box around the map
/// objects reaches there.
#[test]
fn a_finish_in_the_blank_part_beside_the_map_is_an_error_and_the_example_is_clean() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/rummelsburger-bucht");
    let dir = tempfile::tempdir().unwrap();
    for name in ["rummelsburg.omap", "rummelsburg-sprint.courseplan.toml"] {
        std::fs::copy(example.join(name), dir.path().join(name)).unwrap();
    }
    let project = dir.path().join("rummelsburg-sprint.courseplan.toml");
    let p = project.to_str().unwrap();
    let out = run(dir.path(), &["check", p]);
    assert_eq!(code(&out), Some(0), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("0 errors, 0 warnings, 0 hints"),
        "{}",
        stdout(&out)
    );
    // Remove the finish and put it back at 95,0: on the sheet, inside the bounding box of the map
    // objects, but far from the terrain.
    let text = std::fs::read_to_string(&project).unwrap();
    let moved = text.replace("map_mm = [10.0, 5.0]", "map_mm = [95.0, 0.0]");
    assert_ne!(text, moved, "the example's finish is no longer at 10,5");
    std::fs::write(&project, moved).unwrap();
    let out = run(dir.path(), &["check", p]);
    assert_eq!(code(&out), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains(
            "finish F1 at 95, 0 mm on the map is outside the mapped area: the nearest map object, symbol 501.11, is 60 m away, at most 40 m allowed"
        ),
        "{}",
        stdout(&out)
    );
    assert!(
        stdout(&out).contains("wkb2026.map.inside"),
        "{}",
        stdout(&out)
    );
    let de = run_lang(dir.path(), "de", &["check", p]);
    assert!(
        stdout(&de)
            .contains("Ziel F1 liegt bei 95, 0 mm auf der Karte außerhalb der kartierten Fläche"),
        "{}",
        stdout(&de)
    );
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
    assert_eq!(
        stdout(&leg).trim(),
        "accepted finding wkb2026.leg.through_control at course A, leg 32–finish"
    );
    // The file stores the ends of the leg (ruling R24).
    assert!(
        std::fs::read_to_string(&project)
            .unwrap()
            .contains("legs = [{ course = \"A\", after = 2, from = \"32\", to = \"F1\" }]")
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
        "accepted finding wkb2026.leg.opposing at course A, legs start–31 and 32–finish"
    );
    assert!(std::fs::read_to_string(&project).unwrap().contains(
        "legs = [{ course = \"A\", after = 0, from = \"S1\", to = \"31\" }, \
             { course = \"A\", after = 2, from = \"32\", to = \"F1\" }]"
    ));

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

    // A start-capacity finding sits at its start, which counts as a control.
    let event = accept(&[
        "wkb2026.start.capacity",
        "--controls",
        "S1",
        "--note",
        "one start box",
    ]);
    assert_eq!(code(&event), Some(0), "{}", stderr(&event));
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
        "set rule profile wkb-2026 with overlay nor-bb-2021"
    );
    assert_eq!(info("en"), "rule profile: wkb-2026, overlay: nor-bb-2021");
    assert_eq!(info("de"), "Regelprofil: wkb-2026, Aufsatz: nor-bb-2021");
    let de = run_lang(
        d,
        "de",
        &[
            "event",
            "profile",
            p,
            "wkb-2026",
            "--overlay",
            "nor-bb-2021",
        ],
    );
    assert_eq!(
        stdout(&de).trim(),
        "Regelprofil wkb-2026 mit dem Aufsatz nor-bb-2021 gesetzt"
    );

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

#[test]
fn an_acceptance_whose_leg_changed_is_listed_as_not_matched() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    let accept = run(
        d,
        &[
            "finding",
            "accept",
            p,
            "wkb2026.leg.through_control",
            "--leg",
            "A:1",
            "--note",
            "kept on purpose",
        ],
    );
    assert_eq!(code(&accept), Some(0), "{}", stderr(&accept));
    // Course A now runs S1–32–31–F1: the leg after position 1 is 32–31, not 31–32.
    let swap = run(d, &["course", "controls", p, "A", "32,31"]);
    assert_eq!(code(&swap), Some(0), "{}", stderr(&swap));
    // The edit drops the acceptance of the leg that is gone and says so.
    assert!(
        stdout(&swap).contains("dropped the accepted finding on the leg 31 to 32"),
        "{}",
        stdout(&swap)
    );
    // A file edited by hand can still hold an acceptance of a leg that is gone.
    let mut text = std::fs::read_to_string(&project).unwrap();
    text.push_str(
        "\n[[accepted_findings]]\nrule = \"wkb2026.leg.through_control\"\nlegs = [{ course = \"A\", after = 1, from = \"31\", to = \"32\" }]\nnote = \"kept on purpose\"\n",
    );
    std::fs::write(&project, text).unwrap();
    let out = run(d, &["check", p]);
    assert_eq!(
        code(&out),
        Some(0),
        "unmatched never count: {}",
        stdout(&out)
    );
    let text = stdout(&out);
    assert!(text.contains("0 accepted, 1 not matched, "), "{text}");
    assert!(text.contains("check --all"), "{text}");
    let all = stdout(&run(d, &["check", p, "--all"]));
    assert!(
        all.contains(
            "not matched:\n  wkb2026.leg.through_control, course A, leg 31–32\n    note: kept on purpose\n"
        ),
        "{all}"
    );
    assert!(all.contains("finding remove"), "{all}");
    let de = stdout(&run_lang(d, "de", &["check", p, "--all"]));
    assert!(
        de.contains("nicht zugeordnet:\n  wkb2026.leg.through_control, Bahn A, Strecke 31–32"),
        "{de}"
    );
    // Accepting A:1 again confirms the new leg 32–31, and remove takes that exact entry first.
    let again = run(
        d,
        &[
            "finding",
            "accept",
            p,
            "wkb2026.leg.through_control",
            "--leg",
            "A:1",
            "--note",
            "new leg",
        ],
    );
    assert_eq!(
        stdout(&again).trim(),
        "accepted finding wkb2026.leg.through_control at course A, leg 32–31"
    );
    let exact = run(
        d,
        &[
            "finding",
            "remove",
            p,
            "wkb2026.leg.through_control",
            "--leg",
            "A:1",
        ],
    );
    assert_eq!(
        stdout(&exact).trim(),
        "removed accepted finding wkb2026.leg.through_control at course A, leg 32–31"
    );
    // finding remove names the leg by position and removes it, though its ends changed.
    let removed = run(
        d,
        &[
            "finding",
            "remove",
            p,
            "wkb2026.leg.through_control",
            "--leg",
            "A:1",
        ],
    );
    assert_eq!(code(&removed), Some(0), "{}", stderr(&removed));
    assert_eq!(
        stdout(&removed).trim(),
        "removed accepted finding wkb2026.leg.through_control at course A, leg 31–32"
    );
    let after = stdout(&run(d, &["check", p]));
    assert!(after.contains("0 accepted, 6 skipped"), "{after}");
}

#[test]
fn start_window_and_class_settings_feed_the_start_capacity_check() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    let text = std::fs::read_to_string(&project).unwrap();
    std::fs::write(&project, format!("# planned by H.\n{text}")).unwrap();
    let skipped = stdout(&run(d, &["check", p, "--all"]));
    assert!(
        skipped.contains("the event has no start window; event start-window sets it"),
        "{skipped}"
    );
    let window = run(d, &["event", "start-window", p, "90"]);
    assert_eq!(code(&window), Some(0), "{}", stderr(&window));
    assert_eq!(stdout(&window).trim(), "set the start window to 90 minutes");
    let de = run_lang(d, "de", &["event", "start-window", p, "1"]);
    assert_eq!(stdout(&de).trim(), "Startfenster auf 1 Minute gesetzt");
    assert_eq!(code(&run(d, &["event", "start-window", p, "90"])), Some(0));
    for args in [
        vec!["class", "add", p, "H21"],
        vec!["course", "set", p, "A", "--classes", "H21"],
    ] {
        let out = run(d, &args);
        assert_eq!(code(&out), Some(0), "{args:?}: {}", stderr(&out));
    }
    let all = stdout(&run(d, &["check", p, "--all"]));
    assert!(
        all.contains(
            "wkb2026.start.capacity, start S1: classes without an expected number of starters: \
             H21; class set --starters sets it"
        ),
        "{all}"
    );
    let set = run(d, &["class", "set", p, "H21", "--starters", "400"]);
    assert_eq!(code(&set), Some(0), "{}", stderr(&set));
    assert_eq!(stdout(&set).trim(), "changed class H21");
    let out = stdout(&run(d, &["check", p]));
    assert!(
        out.contains(
            "start S1: 5 different first controls are needed for the starters per minute, but \
             only 1 in use"
        ),
        "{out}"
    );
    let category = run(d, &["class", "set", p, "H21", "--rule-category", "H21"]);
    assert_eq!(code(&category), Some(0), "{}", stderr(&category));
    let file = std::fs::read_to_string(&project).unwrap();
    assert!(file.starts_with("# planned by H.\n"), "{file}");
    assert!(file.contains("start_window_min = 90"), "{file}");
    assert!(
        file.contains("rule_category = \"H21\"") && file.contains("expected_starters = 400"),
        "{file}"
    );
    let cleared = run(
        d,
        &[
            "class",
            "set",
            p,
            "H21",
            "--no-rule-category",
            "--no-starters",
        ],
    );
    assert_eq!(code(&cleared), Some(0), "{}", stderr(&cleared));
    let file = std::fs::read_to_string(&project).unwrap();
    assert!(
        !file.contains("rule_category") && !file.contains("expected_starters"),
        "{file}"
    );

    // Refusals leave the file as it was.
    let before = std::fs::read(&project).unwrap();
    let zero = run(d, &["event", "start-window", p, "0"]);
    assert_eq!(code(&zero), Some(1));
    assert_eq!(
        stderr(&zero).trim(),
        "error: the start window must be at least 1 minute"
    );
    assert_eq!(code(&run(d, &["event", "start-window", p, "x"])), Some(2));
    let nothing = run(d, &["class", "set", p, "H21"]);
    assert_eq!(code(&nothing), Some(2), "{}", stderr(&nothing));
    let both = run(
        d,
        &["class", "set", p, "H21", "--starters", "3", "--no-starters"],
    );
    assert_eq!(code(&both), Some(2), "{}", stderr(&both));
    let unknown = run(d, &["class", "set", p, "D21", "--starters", "3"]);
    assert_eq!(code(&unknown), Some(1));
    assert!(
        stderr(&unknown).contains("there is no class D21"),
        "{}",
        stderr(&unknown)
    );
    let blank = run(d, &["class", "set", p, "H21", "--rule-category", " "]);
    assert_eq!(code(&blank), Some(1), "{}", stderr(&blank));
    assert!(stderr(&blank).contains("is empty"), "{}", stderr(&blank));
    assert_eq!(std::fs::read(&project).unwrap(), before);
}
