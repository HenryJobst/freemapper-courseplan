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

/// Runs the program in English, whatever the machine's locale.
fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(BIN)
        .current_dir(cwd)
        .env_remove("FREEMAPPER_LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C")
        .arg("--lang")
        .arg("en")
        .args(args)
        .output()
        .unwrap()
}

/// Runs the program with the given extra environment and no `--lang` of its own.
fn run_env(cwd: &Path, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut command = Command::new(BIN);
    command
        .current_dir(cwd)
        .env_remove("FREEMAPPER_LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C");
    for (name, value) in env {
        command.env(name, value);
    }
    command.args(args).output().unwrap()
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
fn init_refuses_binary_omap() {
    let dir = tempfile::tempdir().unwrap();
    // The magic of OOM's old binary format, as in the reader's `rejects_binary_omap`.
    std::fs::write(dir.path().join("old.omap"), b"OMAP\0\0\0\x01rest").unwrap();
    let out = run(dir.path(), &["init", "old.omap"]);
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(
        err.starts_with("error: cannot read map old.omap: "),
        "{err}"
    );
    assert!(err.contains("old binary format"), "{err}");
    assert!(err.contains("save it again"), "{err}");
    assert!(stdout(&out).is_empty());
    assert!(!dir.path().join("old.courseplan.toml").exists());
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

#[test]
fn init_refuses_forest_map_in_german() {
    let (dir, map) = with_map("forest-sample.omap");
    let out = run_env(
        dir.path(),
        &[],
        &["--lang", "de", "init", map.to_str().unwrap()],
    );
    assert_eq!(out.status.code(), Some(1));
    let err = stderr(&out);
    assert!(err.starts_with("Fehler: "), "{err}");
    assert!(err.contains("Waldkarten"), "{err}");
}

#[test]
fn info_prints_german_labels() {
    let (dir, map) = with_map("luebeck.omap");
    assert!(
        run(dir.path(), &["init", map.to_str().unwrap()])
            .status
            .success()
    );
    let project = dir.path().join("luebeck.courseplan.toml");
    let out = run_env(
        dir.path(),
        &[("LANG", "de_DE.UTF-8")],
        &["info", project.to_str().unwrap()],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "Titel: luebeck\nKarte: luebeck.omap\nKartenart: Sprintkarte\nPosten: 0\n\
         Bahnen: 0 (einzeln 0, Mannschaft 0, Staffel 0, Score 0)\nKategorien: 0\n"
    );
}

#[test]
fn help_in_german() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(dir.path(), &[], &["--lang", "de", "init", "--help"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let help = stdout(&out);
    assert!(help.contains("Projektdatei"), "{help}");
    assert!(help.contains("Hilfe anzeigen"), "{help}");
    assert!(!help.contains("Where to write"), "{help}");
}

#[test]
fn help_in_english_and_language_from_environment() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["init", "--help"]);
    assert!(stdout(&out).contains("Where to write the project file"));
    let out = run_env(
        dir.path(),
        &[("FREEMAPPER_LANG", "de")],
        &["info", "--help"],
    );
    assert!(
        stdout(&out).contains("Die Projektdatei"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn unknown_language_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(dir.path(), &[], &["--lang", "fr", "info", "x.toml"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        stderr(&out),
        "error: unknown language fr; available: en, de\n"
    );
}

#[test]
fn help_in_german_has_no_english_headings() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        &["--lang", "de", "--help"][..],
        &["--lang", "de", "init", "--help"][..],
    ] {
        let out = run_env(dir.path(), &[], args);
        assert!(out.status.success(), "{}", stderr(&out));
        let help = stdout(&out);
        for english in [
            "Usage:",
            "Options:",
            "Commands:",
            "Arguments:",
            "Print this message",
        ] {
            assert!(!help.contains(english), "{english} in {help}");
        }
        assert!(
            help.contains("Aufruf:") && help.contains("Optionen:"),
            "{help}"
        );
    }
}

#[test]
fn unknown_argument_in_german() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(dir.path(), &[], &["--lang", "de", "init", "--bogus"]);
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(
        err.starts_with("Fehler: Unbekanntes Argument --bogus"),
        "{err}"
    );
    assert!(!err.contains("unexpected"), "{err}");
}

#[test]
fn missing_command_in_german_and_help_subcommand() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(dir.path(), &[], &["--lang", "de"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        stderr(&out).starts_with("Fehler: Ein Befehl ist nötig"),
        "{}",
        stderr(&out)
    );
    let out = run_env(dir.path(), &[], &["--lang", "de", "help", "info"]);
    assert!(out.status.success());
    assert!(
        stdout(&out).contains("Die Projektdatei"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn unknown_command_in_german_names_it() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(dir.path(), &[], &["--lang", "de", "frob"]);
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(err.starts_with("Fehler: Unbekannter Befehl frob"), "{err}");
}

#[test]
fn info_lines_from_m1_unchanged() {
    let (dir, map) = with_map("luebeck.omap");
    assert!(
        run(dir.path(), &["init", map.to_str().unwrap()])
            .status
            .success()
    );
    let project = dir.path().join("luebeck.courseplan.toml");
    let out = run(dir.path(), &["info", project.to_str().unwrap()]);
    assert_eq!(
        stdout(&out),
        "title: luebeck\nmap: luebeck.omap\nmap kind: sprint\ncontrols: 0\n\
         courses: 0 (individual 0, team 0, relay 0, score 0)\nclasses: 0\n"
    );
}

#[test]
fn info_lists_courses_in_german_and_skips_length_of_team_courses() {
    let (dir, map) = with_map("luebeck.omap");
    assert!(
        run(dir.path(), &["init", map.to_str().unwrap()])
            .status
            .success()
    );
    let project = dir.path().join("luebeck.courseplan.toml");
    let mut toml = std::fs::read_to_string(&project).unwrap();
    toml.push_str(
        "\n[[specials]]\nkind = \"start\"\nid = \"S1\"\nmap_mm = [0.0, 0.0]\n\
         \n[[specials]]\nkind = \"finish\"\nid = \"F1\"\nmap_mm = [60.0, 0.0]\n\
         \n[[courses]]\nname = \"T\"\nkind = \"team\"\nstart = \"S1\"\nfinish = \"F1\"\n\
         team_size = 2\nmandatory = []\nfree = []\n",
    );
    std::fs::write(&project, toml).unwrap();
    let out = run_env(
        dir.path(),
        &[("LANG", "de_DE.UTF-8")],
        &["info", project.to_str().unwrap()],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stdout(&out).ends_with("Bahn T: Mannschaftsbahn, Länge noch nicht unterstützt\n"),
        "{}",
        stdout(&out)
    );
    let out = run(dir.path(), &["info", project.to_str().unwrap()]);
    assert!(
        stdout(&out).ends_with("course T: team, length not supported yet\n"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn help_takes_a_path_of_commands() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(dir.path(), &[], &["--lang", "de", "help", "control", "add"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let help = stdout(&out);
    assert!(help.starts_with("Einen Posten anlegen"), "{help}");
    assert!(help.contains("--crossing-point"), "{help}");
    let out = run(dir.path(), &["help", "course", "number"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("--after <N>"), "{}", stdout(&out));
    let out = run(dir.path(), &["help", "control", "frob"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("frob"), "{}", stderr(&out));
}

#[test]
fn help_lists_arguments_in_declaration_order() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(dir.path(), &["course", "controls", "--help"]);
    let help = stdout(&out);
    let at = |name: &str| {
        help.find(&format!("  <{name}>"))
            .unwrap_or_else(|| panic!("{name} in {help}"))
    };
    assert!(
        at("PROJECT") < at("NAME") && at("NAME") < at("CODES"),
        "{help}"
    );
    let out = run(dir.path(), &["special", "add", "--help"]);
    let help = stdout(&out);
    let at = |name: &str| {
        help.find(name)
            .unwrap_or_else(|| panic!("{name} in {help}"))
    };
    assert!(
        at("--kind") < at("--id") && at("--id") < at("--at"),
        "{help}"
    );
    assert!(
        at("--courses") < at("--lang") && at("--lang") < at("--help"),
        "{help}"
    );
}

#[test]
fn possible_values_are_shown_and_listed_in_german() {
    let dir = tempfile::tempdir().unwrap();
    let out = run_env(
        dir.path(),
        &[],
        &["--lang", "de", "special", "add", "--help"],
    );
    let help = stdout(&out);
    assert!(
        help.contains("[mögliche Werte: start, finish, marked-route,"),
        "{help}"
    );
    assert!(!help.contains("possible values"), "{help}");
    let out = run_env(
        dir.path(),
        &[],
        &[
            "--lang", "de", "special", "add", "x.toml", "--kind", "wasser", "--id", "a",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(
        err.starts_with("Fehler: Ungültiger Wert wasser für --kind <KIND>"),
        "{err}"
    );
    assert!(err.contains("Mögliche Werte: start, finish, marked-route, out-of-bounds, forbidden-route, dangerous-area, first-aid, water"), "{err}");
    let out = run(
        dir.path(),
        &["special", "add", "x.toml", "--kind", "wasser", "--id", "a"],
    );
    assert!(
        stderr(&out).contains("possible values: start, finish,"),
        "{}",
        stderr(&out)
    );
}
