use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

/// Runs the program in the given language (default English), whatever the machine's locale.
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

/// A temp dir with the Luebeck map and a project made by `init`.
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
    (dir, project)
}

/// Runs a command that must succeed and returns its first output line.
fn ok(dir: &Path, args: &[&str]) -> String {
    let out = run(dir, args);
    assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    stdout(&out).trim_end().to_owned()
}

fn info_line(dir: &Path, project: &Path, prefix: &str) -> String {
    ok(dir, &["info", project.to_str().unwrap()])
        .lines()
        .find(|l| l.starts_with(prefix))
        .unwrap()
        .to_owned()
}

#[test]
fn control_add_move_describe_remove() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();

    assert_eq!(
        ok(d, &["control", "add", p, "--at", "12.5,-30"]),
        "added control 31 at 12.5,-30"
    );
    assert_eq!(
        info_line(d, &project, "controls"),
        "controls: 1, crossing points: 0"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("31") && toml.contains("12.5"), "{toml}");

    assert_eq!(
        ok(d, &["control", "move", p, "31", "--at", "-4,5.25"]),
        "moved control 31 to -4,5.25"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("5.25") && !toml.contains("12.5"), "{toml}");

    ok(
        d,
        &["control", "describe", p, "31", "--c", "1.8", "--e", "2.1"],
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(
        toml.contains("\"1.8\"") && toml.contains("\"2.1\""),
        "{toml}"
    );
    // Unset columns stay.
    ok(d, &["control", "describe", p, "31", "--d", "5.1"]);
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(
        toml.contains("\"1.8\"") && toml.contains("\"5.1\""),
        "{toml}"
    );
    // --clear empties first.
    ok(
        d,
        &["control", "describe", p, "31", "--clear", "--h", "11.1"],
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(
        !toml.contains("\"1.8\"") && toml.contains("\"11.1\""),
        "{toml}"
    );

    assert_eq!(
        ok(
            d,
            &[
                "control",
                "number",
                p,
                "31",
                "--direction",
                "-45",
                "--gap",
                "0.5"
            ]
        ),
        "control 31 number: direction 315 degrees, gap 0.5 mm"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("direction") && toml.contains("315"), "{toml}");
    ok(d, &["control", "number", p, "31", "--clear"]);
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(!toml.contains("direction"), "{toml}");

    assert_eq!(ok(d, &["control", "remove", p, "31"]), "removed control 31");
    assert_eq!(
        info_line(d, &project, "controls"),
        "controls: 0, crossing points: 0"
    );
}

#[test]
fn control_add_picks_next_free_code() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    assert_eq!(
        ok(d, &["control", "add", p, "--at", "1,1"]),
        "added control 31 at 1,1"
    );
    assert_eq!(
        ok(d, &["control", "add", p, "--at", "2,2", "--code", "33"]),
        "added control 33 at 2,2"
    );
    assert_eq!(
        ok(d, &["control", "add", p, "--at", "3,3", "--crossing-point"]),
        "added crossing point 32 at 3,3"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("crossing_point"), "{toml}");
    assert_eq!(
        info_line(d, &project, "controls"),
        "controls: 2, crossing points: 1"
    );
}

#[test]
fn special_add_start_and_water_point() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    assert_eq!(
        ok(
            d,
            &[
                "special", "add", p, "--kind", "start", "--id", "S1", "--at", "10,10"
            ]
        ),
        "added start S1 at 10,10"
    );
    assert_eq!(
        ok(
            d,
            &[
                "special", "add", p, "--kind", "water", "--id", "W1", "--at", "20,5"
            ]
        ),
        "added water point W1 at 20,5"
    );
    assert_eq!(
        ok(
            d,
            &[
                "special",
                "add",
                p,
                "--kind",
                "marked-route",
                "--id",
                "M1",
                "--point",
                "1,1",
                "--point",
                "5,5",
                "--point",
                "-3,2"
            ]
        ),
        "added marked route M1 with 3 points"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(
        toml.contains("water") && toml.contains("marked_route"),
        "{toml}"
    );
    assert_eq!(
        ok(d, &["special", "remove", p, "W1"]),
        "removed special object W1"
    );
}

#[test]
fn special_add_unknown_course_is_refused() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let before = std::fs::read(&project).unwrap();
    let out = run(
        dir.path(),
        &[
            "special",
            "add",
            p,
            "--kind",
            "water",
            "--id",
            "W1",
            "--at",
            "1,1",
            "--courses",
            "A,B",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn special_add_wrong_combination_is_a_usage_error() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let before = std::fs::read(&project).unwrap();
    for args in [
        vec![
            "special", "add", p, "--kind", "start", "--id", "S1", "--point", "1,1", "--point",
            "2,2",
        ],
        vec![
            "special",
            "add",
            p,
            "--kind",
            "marked-route",
            "--id",
            "M",
            "--at",
            "1,1",
        ],
        vec!["special", "add", p, "--kind", "start", "--id", "S1"],
        vec![
            "special",
            "add",
            p,
            "--kind",
            "start",
            "--id",
            "S1",
            "--at",
            "1,1",
            "--courses",
            "A",
        ],
    ] {
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
        assert!(stderr(&out).starts_with("error: "), "{}", stderr(&out));
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn bad_points_name_the_value() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let before = std::fs::read(&project).unwrap();
    for bad in ["12,5;3", "1,2,3", "abc", "1;2", "nan,1", ""] {
        let out = run(dir.path(), &["control", "add", p, "--at", bad]);
        assert_eq!(out.status.code(), Some(2), "{bad}: {}", stderr(&out));
        assert!(stderr(&out).contains(bad), "{bad}: {}", stderr(&out));
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn class_add_and_remove() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    assert_eq!(
        ok(
            d,
            &[
                "class",
                "add",
                p,
                "H21",
                "--rule-category",
                "M",
                "--starters",
                "12"
            ]
        ),
        "added class H21"
    );
    assert_eq!(info_line(d, &project, "classes"), "classes: 1");
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("H21") && toml.contains("12"), "{toml}");
    assert_eq!(ok(d, &["class", "remove", p, "H21"]), "removed class H21");
    assert_eq!(info_line(d, &project, "classes"), "classes: 0");
}

#[test]
fn failed_edit_leaves_file_unchanged() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    ok(dir.path(), &["control", "add", p, "--at", "1,1"]);
    let before = std::fs::read(&project).unwrap();
    let out = run(dir.path(), &["control", "move", p, "99", "--at", "2,2"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stderr(&out).trim_end(), "error: there is no control 99");
    assert_eq!(std::fs::read(&project).unwrap(), before);

    let out = run(dir.path(), &["control", "add", p, "--at", "1000000,1"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("outside the map"), "{}", stderr(&out));
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn edit_keeps_hand_written_comments() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let mut toml = std::fs::read_to_string(&project).unwrap();
    toml.push_str("\n# my own note, keep me\n");
    std::fs::write(&project, toml).unwrap();
    ok(dir.path(), &["control", "add", p, "--at", "1,1"]);
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("# my own note, keep me"), "{toml}");
}

#[test]
fn german_texts() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    let out = run_lang(d, "de", &["control", "add", p, "--at", "12.5,-30"]);
    assert_eq!(stdout(&out).trim_end(), "Posten 31 bei 12.5,-30 angelegt");
    let out = run_lang(d, "de", &["control", "move", p, "99", "--at", "1,1"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).starts_with("Fehler: "), "{}", stderr(&out));
    let out = run_lang(d, "de", &["control", "add", p, "--at", "abc"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).starts_with("Fehler: ") && stderr(&out).contains("abc"));
    let out = run_lang(d, "de", &["control", "add", "--help"]);
    let help = stdout(&out);
    assert!(
        help.contains("Aufruf:") && help.contains("Optionen:"),
        "{help}"
    );
    assert!(!help.contains("Print help"), "{help}");
    let out = run_lang(d, "de", &["special", "--help"]);
    assert!(stdout(&out).contains("Befehle:"), "{}", stdout(&out));
}

#[test]
fn points_and_courses_in_any_order() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    ok(
        d,
        &[
            "special", "add", p, "--kind", "start", "--id", "S1", "--at", "1,1",
        ],
    );
    ok(
        d,
        &[
            "special", "add", p, "--kind", "finish", "--id", "F1", "--at", "2,2",
        ],
    );
    ok(
        d,
        &["course", "add", p, "A", "--start", "S1", "--finish", "F1"],
    );
    assert_eq!(
        ok(
            d,
            &[
                "special",
                "add",
                p,
                "--kind",
                "forbidden-route",
                "--id",
                "R1",
                "--point",
                "1,1",
                "--courses",
                "A",
                "--point",
                "-2,5",
            ]
        ),
        "added forbidden route R1 with 2 points"
    );
    let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
    let route = file.event.specials.iter().find(|s| s.id() == "R1").unwrap();
    match route {
        freemapper_courseplan_model::SpecialObject::ForbiddenRoute {
            points, courses, ..
        } => {
            assert_eq!(
                points.iter().map(|q| (q.x, q.y)).collect::<Vec<_>>(),
                [(1.0, 1.0), (-2.0, 5.0)]
            );
            assert_eq!(courses, &["A".to_owned()]);
        }
        other => panic!("{other:?}"),
    }
}

/// Start S1 at 0,0, finish F1 at 60,0, controls 31 (30,40), 32 (90,40), 33 (90,0).
fn course_setup(d: &Path, p: &str) {
    for (id, kind, at) in [("S1", "start", "0,0"), ("F1", "finish", "60,0")] {
        ok(
            d,
            &["special", "add", p, "--kind", kind, "--id", id, "--at", at],
        );
    }
    for at in ["30,40", "90,40", "90,0"] {
        ok(d, &["control", "add", p, "--at", at]);
    }
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

#[test]
fn course_cycle() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    assert_eq!(
        ok(
            d,
            &["course", "add", p, "A", "--start", "S1", "--finish", "F1"]
        ),
        "added course A"
    );
    assert_eq!(
        ok(d, &["course", "controls", p, "A", "31,32,33"]),
        "course A now visits 31, 32, 33"
    );
    assert_eq!(
        ok(
            d,
            &["course", "leg", p, "A", "--after", "1", "--bend", "60,60"]
        ),
        "set the leg after position 1 of course A: bends 1, not taped"
    );
    assert_eq!(
        ok(d, &["course", "numbers", p, "A"]),
        "placed 3 control numbers on course A"
    );
    let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
    match &file.event.courses[0].kind {
        freemapper_courseplan_model::CourseKind::Individual { controls } => {
            assert_eq!(controls.len(), 3);
            assert!(controls.iter().all(|c| c.number.is_some()));
            assert_eq!(controls[0].bends.len(), 1);
        }
        other => panic!("{other:?}"),
    }
    // Map scale 1:4000; expected length from the coordinates.
    let mm = dist((0.0, 0.0), (30.0, 40.0))
        + dist((30.0, 40.0), (60.0, 60.0))
        + dist((60.0, 60.0), (90.0, 40.0))
        + dist((90.0, 40.0), (90.0, 0.0))
        + dist((90.0, 0.0), (60.0, 0.0));
    let km = format!("{:.1}", mm * 4.0 / 1000.0);
    assert_eq!(
        info_line(d, &project, "course A"),
        format!("course A: 3 controls, {km} km, climb \u{2013} m")
    );
    ok(
        d,
        &[
            "course",
            "set",
            p,
            "A",
            "--climb",
            "35",
            "--classes",
            "H21,D21",
        ],
    );
    assert_eq!(
        info_line(d, &project, "course A"),
        format!("course A: 3 controls, {km} km, climb 35 m")
    );
    ok(
        d,
        &[
            "course",
            "set",
            p,
            "A",
            "--no-climb",
            "--print-scale",
            "10000",
        ],
    );
    assert!(info_line(d, &project, "course A").ends_with("climb \u{2013} m"));
    assert_eq!(ok(d, &["course", "remove", p, "A"]), "removed course A");
    assert!(!ok(d, &["info", p]).contains("course A"));
}

#[test]
fn course_leg_index_counts_from_one() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    // 34 is a crossing point between 31 and 32; it counts as position 2.
    ok(
        d,
        &["control", "add", p, "--at", "60,40", "--crossing-point"],
    );
    ok(
        d,
        &[
            "course",
            "add",
            p,
            "A",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "31,34,32",
        ],
    );
    let bends_of = || {
        let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
        match &file.event.courses[0].kind {
            freemapper_courseplan_model::CourseKind::Individual { controls } => controls
                .iter()
                .map(|c| (c.bends.len(), c.taped))
                .collect::<Vec<_>>(),
            other => panic!("{other:?}"),
        }
    };
    ok(
        d,
        &[
            "course", "leg", p, "A", "--after", "2", "--bend", "70,50", "--bend", "80,50",
            "--taped",
        ],
    );
    assert_eq!(bends_of(), [(0, false), (2, true), (0, false)]);
    // No --bend and no --taped clears the leg.
    ok(d, &["course", "leg", p, "A", "--after", "2"]);
    assert_eq!(bends_of(), [(0, false); 3]);
    // The last control's leg runs to the finish.
    ok(
        d,
        &["course", "leg", p, "A", "--after", "3", "--bend", "70,10"],
    );
    assert_eq!(bends_of()[2], (1, false));
    let before = std::fs::read_to_string(&project).unwrap();
    for bad in ["0", "4"] {
        let out = run(d, &["course", "leg", p, "A", "--after", bad]);
        assert_eq!(out.status.code(), Some(1), "{bad}");
        let err = stderr(&out);
        assert!(err.contains(&format!("position {bad}")), "{err}");
    }
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    // The start leg.
    ok(
        d,
        &[
            "course",
            "leg",
            p,
            "A",
            "--from-start",
            "--bend",
            "5,5",
            "--taped",
        ],
    );
    let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
    assert_eq!(file.event.courses[0].start_bends.len(), 1);
    assert!(file.event.courses[0].start_taped);
    // Exactly one of --from-start and --after.
    for args in [
        vec!["course", "leg", p, "A"],
        vec!["course", "leg", p, "A", "--from-start", "--after", "1"],
    ] {
        let out = run(d, &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn course_errors_are_localized() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    let out = run(
        d,
        &["course", "add", p, "A", "--start", "S9", "--finish", "F1"],
    );
    assert_eq!(out.status.code(), Some(1));
    let out = run(
        d,
        &[
            "course", "add", p, "A", "--start", "S1", "--finish", "F1", "--climb", "x",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    let out = run(d, &["course", "set", p, "A"]);
    assert_eq!(out.status.code(), Some(2));
    ok(
        d,
        &[
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
    );
    let out = run_lang(d, "de", &["course", "numbers", p, "A"]);
    assert_eq!(stdout(&out).trim_end(), "1 Postennummer auf Bahn A gesetzt");
    let out = run_lang(d, "de", &["course", "remove", p, "B"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).starts_with("Fehler: "), "{}", stderr(&out));
    let out = run_lang(d, "de", &["course", "--help"]);
    assert!(stdout(&out).contains("Befehle:"), "{}", stdout(&out));
}

#[test]
fn counts_use_plural_forms_and_german_decimal_comma() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    ok(
        d,
        &[
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
    );
    assert_eq!(
        ok(d, &["course", "numbers", p, "A"]),
        "placed 1 control number on course A"
    );
    assert_eq!(
        info_line(d, &project, "course A"),
        "course A: 1 control, 0.4 km, climb \u{2013} m"
    );
    ok(d, &["course", "controls", p, "A", "31,32,33"]);
    assert_eq!(
        ok(d, &["course", "numbers", p, "A", "--overwrite"]),
        "placed 3 control numbers on course A"
    );
    assert!(info_line(d, &project, "course A").starts_with("course A: 3 controls, "));
    let out = run_lang(d, "de", &["info", p]);
    let line = stdout(&out)
        .lines()
        .find(|l| l.starts_with("Bahn A"))
        .unwrap()
        .to_owned();
    assert!(
        line.starts_with("Bahn A: 3 Posten, 0,") && line.ends_with(" km, Anstieg \u{2013} m"),
        "{line}"
    );
    let out = run_lang(d, "de", &["course", "numbers", p, "A", "--overwrite"]);
    assert_eq!(
        stdout(&out).trim_end(),
        "3 Postennummern auf Bahn A gesetzt"
    );
    // A line needs two points, so "1 point" never appears; the singular is checked in the
    // catalog test of main.rs.
    let before = std::fs::read(&project).unwrap();
    let out = run(
        d,
        &[
            "special",
            "add",
            p,
            "--kind",
            "marked-route",
            "--id",
            "M1",
            "--point",
            "1,1",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("marked route M1 needs at least 2 points"),
        "{}",
        stderr(&out)
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
    assert_eq!(
        ok(
            d,
            &[
                "special",
                "add",
                p,
                "--kind",
                "marked-route",
                "--id",
                "M2",
                "--point",
                "1,1",
                "--point",
                "2,2"
            ]
        ),
        "added marked route M2 with 2 points"
    );
    for (id, kind, points, want) in [
        (
            "M3",
            "marked-route",
            vec!["1,1", "2,2"],
            "Pflichtstrecke M3 mit 2 Punkten angelegt",
        ),
        (
            "O4",
            "out-of-bounds",
            vec!["1,1", "2,2", "1,3"],
            "Sperrgebiet O4 mit 3 Punkten angelegt",
        ),
    ] {
        let mut args = vec!["special", "add", p, "--kind", kind, "--id", id];
        for point in points {
            args.extend(["--point", point]);
        }
        let out = run_lang(d, "de", &args);
        assert_eq!(stdout(&out).trim_end(), want, "{}", stderr(&out));
    }
}

#[test]
fn blank_control_code_is_refused() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let before = std::fs::read(&project).unwrap();
    for code in ["", "  "] {
        let out = run(
            dir.path(),
            &["control", "add", p, "--at", "1,1", "--code", code],
        );
        assert_eq!(out.status.code(), Some(1), "{code:?}: {}", stderr(&out));
        assert!(
            stderr(&out).contains("is empty or starts or ends with a space"),
            "{}",
            stderr(&out)
        );
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

fn course_numbers_of(project: &Path) -> Vec<Option<(f64, f64)>> {
    let file = freemapper_courseplan_model::ProjectFile::load(project).unwrap();
    match &file.event.courses[0].kind {
        freemapper_courseplan_model::CourseKind::Individual { controls } => controls
            .iter()
            .map(|c| c.number.as_ref().map(|n| (n.direction, n.gap_mm)))
            .collect(),
        other => panic!("{other:?}"),
    }
}

#[test]
fn course_number_sets_and_clears() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    ok(
        d,
        &[
            "course",
            "add",
            p,
            "A",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "31,32,33",
        ],
    );
    assert_eq!(
        ok(
            d,
            &[
                "course",
                "number",
                p,
                "A",
                "--after",
                "2",
                "--direction",
                "-90",
                "--gap",
                "1"
            ]
        ),
        "course A, position 2: number at direction 270 degrees, gap 1 mm"
    );
    assert_eq!(
        course_numbers_of(&project),
        [None, Some((270.0, 1.0)), None]
    );
    // Automatic placement keeps the course number.
    assert_eq!(
        ok(d, &["course", "numbers", p, "A"]),
        "placed 2 control numbers on course A"
    );
    assert_eq!(course_numbers_of(&project)[1], Some((270.0, 1.0)));
    assert_eq!(
        ok(d, &["course", "number", p, "A", "--after", "2", "--clear"]),
        "course A, position 2: number placement cleared"
    );
    assert_eq!(course_numbers_of(&project)[1], None);
    let before = std::fs::read(&project).unwrap();
    for after in ["0", "4"] {
        let out = run(
            d,
            &["course", "number", p, "A", "--after", after, "--clear"],
        );
        assert_eq!(out.status.code(), Some(1), "{after}: {}", stderr(&out));
    }
    for args in [
        vec!["--after", "1", "--direction", "10", "--gap", "0"],
        vec!["--after", "1", "--direction", "10", "--gap", "10.5"],
        vec!["--after", "1", "--direction", "10"],
        vec!["--after", "1", "--clear", "--gap", "1"],
        vec!["--direction", "10", "--gap", "1"],
    ] {
        let mut all = vec!["course", "number", p, "A"];
        all.extend(args.iter().copied());
        let out = run(d, &all);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn course_numbers_keep_central_numbers() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    ok(
        d,
        &[
            "course",
            "add",
            p,
            "A",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "31,32,33",
        ],
    );
    ok(
        d,
        &[
            "control",
            "number",
            p,
            "32",
            "--direction",
            "90",
            "--gap",
            "0.5",
        ],
    );
    for overwrite in [false, true] {
        let mut args = vec!["course", "numbers", p, "A"];
        if overwrite {
            args.push("--overwrite");
        }
        assert_eq!(ok(d, &args), "placed 2 control numbers on course A");
        assert_eq!(course_numbers_of(&project)[1], None);
    }
}

#[test]
fn number_direction_is_normalised_and_gap_checked() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    ok(d, &["control", "add", p, "--at", "1,1"]);
    for (direction, shown) in [
        ("-45", "315"),
        ("720.5", "0.5"),
        ("360", "0"),
        ("1e2", "100"),
    ] {
        assert_eq!(
            ok(
                d,
                &[
                    "control",
                    "number",
                    p,
                    "31",
                    "--direction",
                    direction,
                    "--gap",
                    "10"
                ]
            ),
            format!("control 31 number: direction {shown} degrees, gap 10 mm")
        );
    }
    let before = std::fs::read(&project).unwrap();
    for gap in ["0", "-1", "10.01", "1e3"] {
        let out = run(
            d,
            &[
                "control",
                "number",
                p,
                "31",
                "--direction",
                "0",
                "--gap",
                gap,
            ],
        );
        assert_eq!(out.status.code(), Some(2), "{gap}: {}", stderr(&out));
        assert!(stderr(&out).contains(gap), "{}", stderr(&out));
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn changed_legs_lose_their_bends_and_say_so() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    ok(
        d,
        &[
            "course",
            "add",
            p,
            "A",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "31,32,33",
        ],
    );
    ok(
        d,
        &["course", "leg", p, "A", "--after", "1", "--bend", "60,60"],
    );
    ok(d, &["course", "leg", p, "A", "--after", "2", "--taped"]);
    assert_eq!(
        ok(d, &["course", "controls", p, "A", "31,33,32"]),
        "course A now visits 31, 33, 32\n\
         course A: cleared the bends and taped flag of the leg after position 1, which now runs to another stop\n\
         course A: cleared the bends and taped flag of the leg after position 3, which now runs to another stop"
    );
    ok(
        d,
        &["course", "leg", p, "A", "--after", "1", "--bend", "60,60"],
    );
    assert_eq!(
        ok(d, &["control", "remove", p, "33"]),
        "removed control 33 and its visits in A\n\
         course A: cleared the bends and taped flag of the leg after position 1, which now runs to another stop"
    );
    ok(
        d,
        &["course", "leg", p, "A", "--from-start", "--bend", "5,5"],
    );
    let out = run_lang(d, "de", &["course", "controls", p, "A", "32,31"]);
    assert_eq!(
        stdout(&out).trim_end(),
        "Bahn A führt jetzt über 32, 31\n\
         Bahn A: Knickpunkte und Abflaggung der Strecke vom Start entfernt, weil sie jetzt zu einem anderen Bahnpunkt führt",
        "{}",
        stderr(&out)
    );
    // The length is the straight sum again: S1 (0,0), 32 (90,40), 31 (30,40), F1 (60,0).
    let mm = dist((0.0, 0.0), (90.0, 40.0)) + 60.0 + dist((30.0, 40.0), (60.0, 0.0));
    let km = format!("{:.1}", mm * 4.0 / 1000.0);
    assert_eq!(
        info_line(d, &project, "course A"),
        format!("course A: 2 controls, {km} km, climb \u{2013} m")
    );
}

fn special_courses_of(project: &Path, id: &str) -> Vec<String> {
    let file = freemapper_courseplan_model::ProjectFile::load(project).unwrap();
    let special = file.event.specials.iter().find(|s| s.id() == id).unwrap();
    special.courses().to_vec()
}

#[test]
fn special_courses_assign_and_widen() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    for name in ["A", "B"] {
        ok(
            d,
            &["course", "add", p, name, "--start", "S1", "--finish", "F1"],
        );
    }
    ok(
        d,
        &[
            "special", "add", p, "--kind", "water", "--id", "W1", "--at", "20,5",
        ],
    );
    assert_eq!(
        ok(d, &["special", "courses", p, "W1", "A, B"]),
        "special object W1 now belongs to A, B"
    );
    assert_eq!(special_courses_of(&project, "W1"), ["A", "B"]);
    ok(d, &["special", "courses", p, "W1", "A"]);
    let before = std::fs::read(&project).unwrap();
    let out = run_lang(d, "de", &["course", "remove", p, "A"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        stderr(&out).trim_end(),
        "Fehler: Bahn A ist die einzige Bahn von W1; ohne sie gälte W1 für alle Bahnen. \
         Ordne W1 zuerst einer anderen Bahn zu oder lösche es"
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
    for args in [
        vec!["special", "courses", p, "W1"],
        vec!["special", "courses", p, "W1", "A", "--all"],
        vec!["special", "courses", p, "W1", " , "],
    ] {
        let out = run(d, &args);
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
    }
    for args in [
        vec!["special", "courses", p, "S1", "A"],
        vec!["special", "courses", p, "W1", "A,A"],
        vec!["special", "courses", p, "W1", "Z"],
    ] {
        let out = run(d, &args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
    assert_eq!(
        ok(d, &["special", "courses", p, "W1", "--all"]),
        "special object W1 now applies to all courses"
    );
    assert!(special_courses_of(&project, "W1").is_empty());
    ok(d, &["course", "remove", p, "A"]);
}

#[test]
fn lists_and_names_are_trimmed() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    for class in ["H21", "D21"] {
        ok(d, &["class", "add", p, class]);
    }
    ok(
        d,
        &[
            "course",
            "add",
            p,
            " A ",
            "--start",
            " S1",
            "--finish",
            "F1 ",
            "--controls",
            "31, 32",
            "--classes",
            "H21, D21",
        ],
    );
    let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
    let course = &file.event.courses[0];
    assert_eq!(course.name, "A");
    assert_eq!(course.classes, ["H21", "D21"]);
    assert_eq!(
        codes(&project),
        ["31", "32"],
        "{}",
        std::fs::read_to_string(&project).unwrap()
    );
    ok(d, &["course", "set", p, "A", "--classes", " D21 ,H21 "]);
    ok(
        d,
        &[
            "special",
            "add",
            p,
            "--kind",
            "water",
            "--id",
            " W1 ",
            "--at",
            "20,5",
            "--courses",
            "A ,",
        ],
    );
    ok(d, &["control", "move", p, " 31 ", "--at", "31,41"]);
    ok(
        d,
        &["course", "leg", p, " A", "--after", "1", "--bend", "1,1"],
    );
    let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
    assert_eq!(file.event.courses[0].classes, ["D21", "H21"]);
    let water = file.event.specials.iter().find(|s| s.id() == "W1").unwrap();
    assert_eq!(water.courses(), ["A"]);
    assert_eq!(file.event.controls[0].map_mm.x, 31.0);
    ok(d, &["class", "remove", p, " H21"]);
    ok(d, &["special", "remove", p, "W1 "]);
}

fn codes(project: &Path) -> Vec<String> {
    let file = freemapper_courseplan_model::ProjectFile::load(project).unwrap();
    match &file.event.courses[0].kind {
        freemapper_courseplan_model::CourseKind::Individual { controls } => {
            controls.iter().map(|c| c.code.0.clone()).collect()
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn small_texts_of_the_final_review() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    let help = stdout(&run(d, &["control", "add", "--help"]));
    assert!(
        help.contains("Code of the new control [default: the lowest free number from 31]"),
        "{help}"
    );
    for args in [["course", "add", "--help"], ["course", "set", "--help"]] {
        let help = stdout(&run(d, &args));
        assert!(help.contains("3000 for 1:3000"), "{help}");
    }
    let out = run_lang(
        d,
        "de",
        &["control", "add", p, "--at", "3,3", "--crossing-point"],
    );
    assert_eq!(
        stdout(&out).trim_end(),
        "Querungsstelle 31 bei 3,3 angelegt"
    );
    let out = run_lang(d, "de", &["info", p]);
    assert!(
        stdout(&out).contains("Posten: 0, Querungsstellen: 1\n"),
        "{}",
        stdout(&out)
    );
    course_setup(d, p);
    ok(
        d,
        &[
            "course",
            "add",
            p,
            "A",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--controls",
            "32,31",
        ],
    );
    let out = run_lang(d, "de", &["course", "leg", p, "A", "--after", "3"]);
    assert_eq!(
        stderr(&out).trim_end(),
        "Fehler: Bahn A hat keinen Posten an Position 3"
    );
    assert_eq!(
        ok(d, &["course", "numbers", p, "A"]),
        "placed 1 control number on course A"
    );
    assert_eq!(
        ok(d, &["course", "numbers", p, "A"]),
        "placed no control numbers on course A, because every control has its number already; \
         --overwrite places the course numbers again"
    );
    assert_eq!(
        ok(d, &["course", "numbers", p, "A", "--overwrite"]),
        "placed 1 control number on course A"
    );
}

#[test]
fn refused_special_removal_leaves_the_file_byte_identical() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    ok(
        d,
        &["course", "add", p, "A", "--start", "S1", "--finish", "F1"],
    );
    let before = std::fs::read(&project).unwrap();
    let out = run(d, &["special", "remove", p, "S1"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(
        stderr(&out).trim_end(),
        "error: S1 is still used by A; change those first"
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

/// A project on a copy of the Luebeck map with one footpath (505.1) segment added; `line` is the
/// two end points in map mm.
fn project_with_footpath(line: [(f64, f64); 2]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/luebeck.omap");
    let text = std::fs::read_to_string(source).unwrap();
    let object = format!(
        "<object type=\"1\" symbol=\"120\"><coords count=\"2\">{} {};{} {};</coords></object>\n",
        (line[0].0 * 1000.0).round(),
        (line[0].1 * 1000.0).round(),
        (line[1].0 * 1000.0).round(),
        (line[1].1 * 1000.0).round(),
    );
    assert!(text.contains("</objects></part>"));
    let text = text.replacen(
        "</objects></part>",
        &format!("{object}</objects></part>"),
        1,
    );
    let map = dir.path().join("luebeck.omap");
    std::fs::write(&map, text).unwrap();
    let out = run(dir.path(), &["init", map.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
    let project = dir.path().join("luebeck.courseplan.toml");
    (dir, project)
}

fn numbers_on_course_a(d: &Path, p: &str) {
    course_setup(d, p);
    ok(
        d,
        &["course", "add", p, "A", "--start", "S1", "--finish", "F1"],
    );
    ok(d, &["course", "controls", p, "A", "31,32,33"]);
    ok(d, &["course", "numbers", p, "A"]);
}

/// A footpath across the box of the number of control 32, 5 mm out along `baseline` (the box
/// spans about 3.7 to 6.1 mm there), 6 mm long, at right angles to that direction.
fn footpath_across_number(baseline: f64) -> [(f64, f64); 2] {
    let (ux, uy) = (baseline.to_radians().cos(), -baseline.to_radians().sin());
    let (cx, cy) = (90.0 + 5.0 * ux, 40.0 + 5.0 * uy);
    [
        (cx - 3.0 * uy, cy + 3.0 * ux),
        (cx + 3.0 * uy, cy - 3.0 * ux),
    ]
}

/// The map reaches automatic placement: a footpath under the box of the number of control 32
/// makes `course numbers` keep the direction and move the number outward within the profile's
/// range (0.5 to 5 mm) instead of turning away.
#[test]
fn course_numbers_move_outward_within_the_range() {
    let (dir, project) = project();
    numbers_on_course_a(dir.path(), project.to_str().unwrap());
    let (baseline, base_gap) = course_numbers_of(&project)[1].unwrap();
    assert!((base_gap - 0.5).abs() < 1e-9);

    let (dir2, project2) = project_with_footpath(footpath_across_number(baseline));
    numbers_on_course_a(dir2.path(), project2.to_str().unwrap());
    let (moved, gap) = course_numbers_of(&project2)[1].unwrap();
    assert!(
        ((moved - baseline + 540.0).rem_euclid(360.0) - 180.0).abs() < 1e-9,
        "baseline {baseline}, with footpath {moved}"
    );
    assert!(gap > 0.5 && gap <= 5.0, "gap {gap}");
}

#[cfg(debug_assertions)]
#[test]
fn cli_reports_changed_meanwhile_with_exit_1() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let before = std::fs::read_to_string(&project).unwrap();
    let out = Command::new(BIN)
        .current_dir(dir.path())
        .env_remove("FREEMAPPER_LANG")
        .env("LANG", "C")
        .env("FREEMAPPER_TEST_TOUCH_PROJECT", "1")
        .args(["--lang", "en", "control", "add", p, "--at", "1,1"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("changed since it was read; nothing was written"),
        "{}",
        stderr(&out)
    );
    // Only the touch itself changed the file.
    assert_eq!(
        std::fs::read_to_string(&project).unwrap(),
        format!("{before}\n# touched\n")
    );
}

#[test]
fn control_code_cli() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    ok(d, &["control", "add", p, "--at", "1,1"]);
    ok(d, &["control", "add", p, "--at", "2,2"]);
    assert_eq!(
        ok(d, &["control", "code", p, "31", " K7 "]),
        "changed the code of control 31 to K7"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("K7") && !toml.contains("\"31\""), "{toml}");

    let before = std::fs::read(&project).unwrap();
    let taken = run(d, &["control", "code", p, "K7", "32"]);
    assert_eq!(taken.status.code(), Some(1));
    assert_eq!(
        stderr(&taken).trim_end(),
        "error: control 32 already exists"
    );
    let missing = run(d, &["control", "code", p, "99", "98"]);
    assert_eq!(missing.status.code(), Some(1));
    assert_eq!(std::fs::read(&project).unwrap(), before);

    let german = run_lang(d, "de", &["control", "code", p, "K7", "31"]);
    assert!(german.status.success(), "{}", stderr(&german));
    assert_eq!(
        stdout(&german).trim_end(),
        "Code von Posten K7 auf 31 geändert"
    );
}

#[test]
fn course_rename_cli() {
    let (dir, project) = project();
    let p = project.to_str().unwrap();
    let d = dir.path();
    course_setup(d, p);
    ok(
        d,
        &["course", "add", p, "A", "--start", "S1", "--finish", "F1"],
    );
    ok(
        d,
        &["course", "add", p, "B", "--start", "S1", "--finish", "F1"],
    );
    assert_eq!(
        ok(d, &["course", "rename", p, "A", "Lang"]),
        "renamed course A to Lang"
    );
    let taken = run(d, &["course", "rename", p, "B", "Lang"]);
    assert_eq!(taken.status.code(), Some(1));
    assert_eq!(
        stderr(&taken).trim_end(),
        "error: course Lang already exists"
    );
    let german = run_lang(d, "de", &["course", "rename", p, "Lang", "Kurz"]);
    assert_eq!(stdout(&german).trim_end(), "Bahn Lang in Kurz umbenannt");
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("Kurz") && !toml.contains("Lang"), "{toml}");
}
