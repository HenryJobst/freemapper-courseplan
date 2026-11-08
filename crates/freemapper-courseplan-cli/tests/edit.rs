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
    assert_eq!(info_line(d, &project, "controls"), "controls: 1");
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
        "control 31 number: direction -45 degrees, gap 0.5 mm"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("direction") && toml.contains("-45"), "{toml}");
    ok(d, &["control", "number", p, "31", "--clear"]);
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(!toml.contains("direction"), "{toml}");

    assert_eq!(ok(d, &["control", "remove", p, "31"]), "removed control 31");
    assert_eq!(info_line(d, &project, "controls"), "controls: 0");
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
        "added control 32 at 3,3"
    );
    let toml = std::fs::read_to_string(&project).unwrap();
    assert!(toml.contains("crossing_point"), "{toml}");
    assert_eq!(info_line(d, &project, "controls"), "controls: 3");
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
        assert!(err.contains(&format!("index {bad}")), "{err}");
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
    assert_eq!(
        stdout(&out).trim_end(),
        "1 Postennummern auf Bahn A gesetzt"
    );
    let out = run_lang(d, "de", &["course", "remove", p, "B"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).starts_with("Fehler: "), "{}", stderr(&out));
    let out = run_lang(d, "de", &["course", "--help"]);
    assert!(stdout(&out).contains("Befehle:"), "{}", stdout(&out));
}
