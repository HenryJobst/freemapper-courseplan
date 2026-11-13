//! Score course editing on the command line and over MCP (ADR 0067): the commands round-trip through the
//! project file, `--json` equals the MCP tool's answer, and refusals read as one sentence.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A temp dir with the Luebeck example project, its map beside it, and the project path.
fn luebeck() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(
        repo_root().join("tests/fixtures/maps/luebeck.omap"),
        dir.path().join("luebeck.omap"),
    )
    .unwrap();
    let text = std::fs::read_to_string(
        repo_root().join("tests/fixtures/luebeck-sprint/luebeck-sprint.courseplan.toml"),
    )
    .unwrap()
    .replace("../maps/luebeck.omap", "luebeck.omap");
    let project = dir.path().join("event.courseplan.toml");
    std::fs::write(&project, text).unwrap();
    (dir, project)
}

fn run(lang: &str, args: &[&str]) -> Output {
    Command::new(BIN)
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

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// What the MCP tool answers over stdio, parsed.
fn mcp(tool: &str, arguments: Value) -> Value {
    let mut child = Command::new(BIN)
        .arg("mcp")
        .env("FREEMAPPER_LANG", "en")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut ask = |id: u64, method: &str, params: Value| -> Value {
        let mut message = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        if id == 0 {
            message.as_object_mut().unwrap().remove("id");
        }
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
        if id == 0 {
            return Value::Null;
        }
        loop {
            let mut line = String::new();
            assert!(stdout.read_line(&mut line).unwrap() > 0, "server closed");
            let reply: Value = serde_json::from_str(&line).unwrap();
            if reply["id"] == id {
                return reply["result"].clone();
            }
        }
    };
    ask(
        1,
        "initialize",
        json!({
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1" }
        }),
    );
    ask(0, "notifications/initialized", json!({}));
    let result = ask(
        2,
        "tools/call",
        json!({ "name": tool, "arguments": arguments }),
    );
    let _ = child.kill();
    let _ = child.wait();
    assert_ne!(result["isError"], true, "{result}");
    serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap()
}

/// A Luebeck copy with the score course "Sc" (45 minutes) holding 31, 32 and 33, made by the CLI.
fn with_score_course() -> (tempfile::TempDir, PathBuf) {
    let (dir, project) = luebeck();
    let p = project.to_str().unwrap();
    for args in [
        vec![
            "course",
            "add",
            p,
            "Sc",
            "--kind",
            "score",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--time-limit",
            "45",
        ],
        vec!["score", "controls", p, "Sc", "31,32,33"],
    ] {
        let o = run("en", &args);
        assert_eq!(o.status.code(), Some(0), "{args:?}: {}", err(&o));
    }
    (dir, project)
}

/// Runs the CLI command with `--json` on one copy and the MCP tool on another copy of the same state, and
/// compares the answers and the files they leave.
fn same(prefix: &[&str], rest: &[&str], tool: &str, arguments: Value) {
    let (_a, cli_project) = with_score_course();
    let (_b, mcp_project) = with_score_course();
    let mut args: Vec<&str> = prefix.to_vec();
    args.push(cli_project.to_str().unwrap());
    args.extend_from_slice(rest);
    args.push("--json");
    let o = run("en", &args);
    assert_eq!(o.status.code(), Some(0), "{args:?}: {}", err(&o));
    let mut arguments = arguments;
    arguments["project"] = json!(mcp_project.to_str().unwrap());
    let via_mcp = mcp(tool, arguments);
    let via_cli: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(via_cli, via_mcp);
    let norm = |p: &Path| std::fs::read_to_string(p).unwrap();
    assert_eq!(norm(&cli_project), norm(&mcp_project));
}

fn course_of(project: &Path, name: &str) -> Value {
    let o = run(
        "en",
        &["list", "courses", project.to_str().unwrap(), "--json"],
    );
    let all: Value = serde_json::from_str(&out(&o)).unwrap();
    all["courses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
        .clone()
}

#[test]
fn info_names_controls_points_and_limit_of_a_score_course() {
    let (_dir, project) = with_score_course();
    let o = run("en", &["info", project.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let text = out(&o);
    assert!(text.contains("score course Sc: 3 controls, "), "{text}");
    assert!(text.contains("points, time limit 45 minutes"), "{text}");
    assert!(!text.contains("not supported yet"), "{text}");
}

/// The Luebeck score project (tests/fixtures/luebeck-sprint, once an example) stays inside the bands and free of
/// findings.
#[test]
fn luebeck_score_example_meets_its_bands_and_check_is_quiet() {
    let project = repo_root().join("tests/fixtures/luebeck-sprint/luebeck-score.courseplan.toml");
    let p = project.to_str().unwrap();
    let o = run("en", &["score", "length", p, "Score 15"]);
    assert_eq!(o.status.code(), Some(0), "{}{}", out(&o), err(&o));
    assert!(
        out(&o).contains("both runners are inside the bands"),
        "{}",
        out(&o)
    );
    let o = run("en", &["check", p]);
    assert_eq!(o.status.code(), Some(0), "{}{}", out(&o), err(&o));
    assert!(
        out(&o).contains("0 errors, 0 warnings, 0 hints; 2 accepted"),
        "{}",
        out(&o)
    );
}

/// The Gleisdreieck score example (examples/gleisdreieck/README.en.md): 32 minutes, both runners in their band or near
/// the target (ADR 0083) on a fresh evaluation, exit 0, and `check` quiet apart from accepted findings: three code pairs
/// whose notes are true (each note names the distance of the two controls) and the share of hard controls. No control stands deeper than 60 m in a dead end
/// and no position is closer than 50 m to the edge of the mapped area (ADR 0085, ADR 0087), which the quiet `check`
/// shows.
#[test]
fn gleisdreieck_score_example_reaches_its_targets_and_check_is_quiet() {
    let project = repo_root().join("examples/gleisdreieck/gleisdreieck-score.courseplan.toml");
    let p = project.to_str().unwrap();
    let o = run("en", &["score", "length", p, "Score 32", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}{}", out(&o), err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(v["controls"], 26, "{v}");
    assert_eq!(v["time_limit_min"], 32);
    assert_eq!(v["length"]["ok"], true, "{v}");
    let state = v["length"]["state"].as_str().unwrap();
    assert!(["in_band", "near"].contains(&state), "{v}");
    let o = run("en", &["check", p]);
    assert_eq!(o.status.code(), Some(0), "{}{}", out(&o), err(&o));
    assert!(
        out(&o).contains("0 errors, 0 warnings, 0 hints; 4 accepted"),
        "{}",
        out(&o)
    );
    // Each accepted note says how far the two controls stand apart; the project says where they are.
    let text = std::fs::read_to_string(&project).unwrap();
    let doc: toml::Value = toml::from_str(&text).unwrap();
    let at = |code: i64| {
        let c = doc["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["code"].as_integer() == Some(code))
            .unwrap_or_else(|| panic!("control {code}"));
        let m = c["map_mm"].as_array().unwrap();
        (m[0].as_float().unwrap(), m[1].as_float().unwrap())
    };
    let accepted = doc["accepted_findings"].as_array().unwrap();
    assert_eq!(accepted.len(), 4);
    let mix = accepted
        .iter()
        .filter(|f| f["rule"].as_str() == Some("wkb2026.score.mix"))
        .count();
    assert_eq!(mix, 1, "the share of hard controls is accepted once");
    for finding in accepted
        .iter()
        .filter(|f| f["rule"].as_str() == Some("wkb2026.score.code_ambiguity"))
    {
        let codes: Vec<i64> = finding["controls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_integer().unwrap())
            .collect();
        let (a, b) = (at(codes[0]), at(codes[1]));
        let metres = ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt() * 4.0;
        let note = finding["note"].as_str().unwrap();
        let stated: f64 = note
            .split("rund ")
            .nth(1)
            .and_then(|rest| rest.split(' ').next())
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("a distance in the note: {note}"));
        assert!((metres - stated).abs() <= 10.0, "{note}: {metres} m");
    }
}

/// A draft never holds a control that `check` reports as standing in an out-of-bounds area (ADR 0082), deeper in a
/// dead end than the limit (ADR 0085) or closer than the margin to the edge of the mapped area (ADR 0087), on both
/// example maps and with several seeds. A draft that misses its targets writes nothing and proves nothing, so only
/// written drafts are checked; at least one per map must be written.
#[test]
fn drafts_on_the_example_maps_keep_to_the_limits_of_check() {
    for (map, start, finish, limit) in [
        (
            "examples/gleisdreieck/gleisdreieck.omap",
            "100,-290",
            "100,-275",
            "20",
        ),
        (
            "examples/rummelsburger-bucht/rummelsburg.omap",
            "3,0",
            "10,5",
            "12",
        ),
    ] {
        let mut written = 0;
        for seed in ["1", "2", "3"] {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("m.omap");
            std::fs::copy(repo_root().join(map), &target).unwrap();
            let project = dir.path().join("m.courseplan.toml");
            let p = project.to_str().unwrap();
            for args in [
                vec!["init", target.to_str().unwrap()],
                vec![
                    "special", "add", p, "--kind", "start", "--id", "S", "--at", start,
                ],
                vec![
                    "special", "add", p, "--kind", "finish", "--id", "F", "--at", finish,
                ],
                vec!["score", "settings", p, "--seed", seed],
            ] {
                let o = run("en", &args);
                assert_eq!(o.status.code(), Some(0), "{map} {args:?}: {}", err(&o));
            }
            let o = run(
                "en",
                &[
                    "score",
                    "draft",
                    p,
                    "T",
                    "--time-limit",
                    limit,
                    "--start",
                    "S",
                    "--finish",
                    "F",
                    "--apply",
                ],
            );
            if o.status.code() != Some(0) {
                continue;
            }
            written += 1;
            let checked = run("en", &["check", p]);
            for rule in [
                "in_forbidden",
                "control.dead_end",
                "map.margin",
                "map.inside",
            ] {
                assert!(
                    !out(&checked).contains(rule),
                    "{map}, seed {seed}: {rule}: {}",
                    out(&checked)
                );
            }
        }
        assert!(written > 0, "{map}: no draft was written");
    }
}

#[test]
fn score_course_round_trips_through_cli() {
    let (_dir, project) = luebeck();
    let p = project.to_str().unwrap();
    let ok = |args: &[&str]| {
        let o = run("en", args);
        assert_eq!(o.status.code(), Some(0), "{args:?}: {}", err(&o));
        out(&o)
    };
    let said = ok(&[
        "course",
        "add",
        p,
        "Sc",
        "--kind",
        "score",
        "--start",
        "S1",
        "--finish",
        "F1",
        "--time-limit",
        "45",
    ]);
    assert!(
        said.contains("45 minutes, 5 points per minute over the limit (the default)"),
        "{said}"
    );
    assert_eq!(
        course_of(&project, "Sc")["score"]["penalty_is_default"],
        true
    );
    // The individual command is unchanged and the default kind.
    ok(&[
        "course",
        "add",
        p,
        "Lang",
        "--start",
        "S1",
        "--finish",
        "F1",
        "--controls",
        "31,32",
    ]);
    let said = ok(&["score", "controls", p, "Sc", "31,34=4,32"]);
    assert!(
        said.contains("3 controls, 7 points (31=1, 34=4, 32=2)"),
        "{said}"
    );
    ok(&["score", "value", p, "Sc", "32", "2"]);
    ok(&["score", "link", "add", p, "Sc", "31", "34"]);
    ok(&["score", "link", "add", p, "Sc", "32", "34"]);
    ok(&["score", "link", "remove", p, "Sc", "34", "32"]);
    ok(&["score", "settings", p, "--seed", "9", "--good-pace", "4:20"]);
    ok(&[
        "score",
        "course",
        "set",
        p,
        "Sc",
        "--time-limit",
        "60",
        "--penalty",
        "4",
    ]);
    ok(&["course", "set", p, "Sc", "--classes", "Offen"]);
    // A stored penalty is not the default; --no-penalty goes back.
    assert_eq!(
        course_of(&project, "Sc")["score"]["penalty_is_default"],
        false
    );
    let said = ok(&["score", "course", "set", p, "Sc", "--no-penalty"]);
    assert!(
        said.contains("5 points per minute over the limit (the default)"),
        "{said}"
    );
    assert!(
        !std::fs::read_to_string(&project)
            .unwrap()
            .contains("penalty_per_min")
    );
    ok(&["score", "course", "set", p, "Sc", "--penalty", "4"]);
    let course = course_of(&project, "Sc");
    assert_eq!(course["kind"], "score");
    assert_eq!(course["classes"], json!(["Offen"]));
    assert_eq!(
        course["score"],
        json!({
            "time_limit_min": 60,
            "penalty_per_min": 4,
            "penalty_is_default": false,
            "controls": [
                {"code": "31", "value": 1},
                {"code": "34", "value": 4},
                {"code": "32", "value": 2},
            ],
            "links": [{"a": "31", "b": "34"}],
        })
    );
    let file = std::fs::read_to_string(&project).unwrap();
    assert!(
        file.contains("[score]") && file.contains("seed = 9"),
        "{file}"
    );
    assert!(file.contains("good_pace_s_per_km = 260"), "{file}");
    // Recoding a control gives it the value of its new code, and the link follows.
    ok(&["control", "code", p, "31", "38"]);
    let course = course_of(&project, "Sc");
    assert_eq!(
        course["score"]["controls"][0],
        json!({"code": "38", "value": 8})
    );
    assert_eq!(course["score"]["links"], json!([{"a": "38", "b": "34"}]));
    // The file loads again and the checks run on it.
    let checked = run("en", &["check", p]);
    assert!(
        matches!(checked.status.code(), Some(0 | 1)),
        "{}",
        err(&checked)
    );
    // The German help and confirmation exist.
    let help = out(&run("de", &["score", "link", "add", "--help"]));
    assert!(help.contains("direkt nacheinander"), "{help}");
    let said = out(&run("de", &["score", "link", "add", p, "Sc", "38", "32"]));
    assert!(said.contains("verbunden"), "{said}");
}

#[test]
fn value_must_match_code_or_is_refused() {
    let (_dir, project) = with_score_course();
    let p = project.to_str().unwrap();
    let before = std::fs::read(&project).unwrap();
    let o = run("en", &["score", "controls", p, "Sc", "33=5"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(
        err(&o).contains("carries the value 3 in its code, not 5; change the code instead"),
        "{}",
        err(&o)
    );
    let o = run("en", &["score", "value", p, "Sc", "32", "4"]);
    assert_eq!(o.status.code(), Some(1), "{}", err(&o));
    let o = run("de", &["score", "value", p, "Sc", "32", "4"]);
    assert!(
        err(&o).contains("Ändere stattdessen den Code"),
        "{}",
        err(&o)
    );
    let o = run("en", &["score", "controls", p, "Sc", "31=x"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn link_ops_enforce_the_model_rules() {
    let (_dir, project) = with_score_course();
    let p = project.to_str().unwrap();
    let link = |args: &[&str]| {
        let mut all = vec!["score", "link"];
        all.extend_from_slice(&args[..1]);
        all.push(p);
        all.extend_from_slice(&args[1..]);
        run("en", &all)
    };
    assert_eq!(link(&["add", "Sc", "31", "32"]).status.code(), Some(0));
    for args in [
        ["add", "Sc", "32", "31"],    // twice, other direction
        ["add", "Sc", "31", "31"],    // to itself
        ["add", "Sc", "31", "35"],    // not in the course
        ["remove", "Sc", "31", "33"], // not there
        ["add", "Lang", "31", "32"],  // not a score course
    ] {
        let o = link(&args);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert!(err(&o).starts_with("error: "), "{}", err(&o));
    }
    assert_eq!(link(&["add", "Sc", "32", "33"]).status.code(), Some(0));
    let o = link(&["add", "Sc", "33", "31"]);
    assert!(err(&o).contains("closes a loop"), "{}", err(&o));
}

#[test]
fn score_options_of_course_add_are_checked() {
    let (_dir, project) = luebeck();
    let p = project.to_str().unwrap();
    let before = std::fs::read(&project).unwrap();
    for args in [
        vec![
            "course", "add", p, "X", "--kind", "score", "--start", "S1", "--finish", "F1",
        ],
        vec![
            "course",
            "add",
            p,
            "X",
            "--kind",
            "score",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--time-limit",
            "30",
            "--controls",
            "31",
        ],
        vec![
            "course",
            "add",
            p,
            "X",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--time-limit",
            "30",
        ],
    ] {
        let o = run("en", &args);
        assert_eq!(o.status.code(), Some(2), "{args:?}: {}", err(&o));
    }
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn add_score_course_json_is_the_mcp_answer() {
    let (_a, cli) = luebeck();
    let (_b, via) = luebeck();
    let o = run(
        "en",
        &[
            "course",
            "add",
            cli.to_str().unwrap(),
            "Sc",
            "--kind",
            "score",
            "--start",
            "S1",
            "--finish",
            "F1",
            "--time-limit",
            "45",
            "--penalty",
            "6",
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let via_mcp = mcp(
        "add_score_course",
        json!({ "project": via.to_str().unwrap(), "name": "Sc", "start": "S1", "finish": "F1",
                "time_limit_min": 45, "penalty_per_min": 6 }),
    );
    assert_eq!(serde_json::from_str::<Value>(&out(&o)).unwrap(), via_mcp);
    assert_eq!(via_mcp["penalty_per_min"], 6);
    assert_eq!(
        std::fs::read_to_string(&cli).unwrap(),
        std::fs::read_to_string(&via).unwrap()
    );
}

#[test]
fn set_score_controls_json_is_the_mcp_answer() {
    same(
        &["score", "controls"],
        &["Sc", "35,31=1"],
        "set_score_controls",
        json!({ "name": "Sc", "controls": [{ "code": 35 }, { "code": "31", "value": 1 }] }),
    );
}

#[test]
fn set_control_value_json_is_the_mcp_answer() {
    same(
        &["score", "value"],
        &["Sc", "32", "2"],
        "set_control_value",
        json!({ "name": "Sc", "code": "32", "value": 2 }),
    );
}

#[test]
fn score_link_json_is_the_mcp_answer() {
    same(
        &["score", "link", "add"],
        &["Sc", "31", "32"],
        "add_score_link",
        json!({ "name": "Sc", "a": "31", "b": 32 }),
    );
    // Remove needs a link first: set it up in both copies through the other door.
    let (_a, cli) = with_score_course();
    let (_b, via) = with_score_course();
    for project in [&cli, &via] {
        let o = run(
            "en",
            &[
                "score",
                "link",
                "add",
                project.to_str().unwrap(),
                "Sc",
                "32",
                "31",
            ],
        );
        assert_eq!(o.status.code(), Some(0));
    }
    let o = run(
        "en",
        &[
            "score",
            "link",
            "remove",
            cli.to_str().unwrap(),
            "Sc",
            "31",
            "32",
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let via_mcp = mcp(
        "remove_score_link",
        json!({ "project": via.to_str().unwrap(), "name": "Sc", "a": "31", "b": "32" }),
    );
    assert_eq!(serde_json::from_str::<Value>(&out(&o)).unwrap(), via_mcp);
    assert_eq!(via_mcp["links"], json!([]));
}

#[test]
fn set_score_settings_json_is_the_mcp_answer() {
    same(
        &["score", "settings"],
        &[
            "--value-max",
            "7",
            "--good-pace",
            "4:20",
            "--very-good-pace",
            "230",
            "--mix-hard",
            "30",
            "--seed",
            "3",
        ],
        "set_score_settings",
        json!({ "value_max": 7, "good_pace_s_per_km": 260, "very_good_pace_s_per_km": 230,
                "mix_hard_percent": 30, "seed": 3 }),
    );
}

#[test]
fn set_score_course_details_no_penalty_json_is_the_mcp_answer() {
    same(
        &["score", "course", "set"],
        &["Sc", "--time-limit", "50", "--no-penalty"],
        "set_score_course_details",
        json!({ "name": "Sc", "time_limit_min": 50, "no_penalty": true }),
    );
}

#[test]
fn set_score_course_details_json_is_the_mcp_answer() {
    same(
        &["score", "course", "set"],
        &["Sc", "--time-limit", "60", "--penalty", "3"],
        "set_score_course_details",
        json!({ "name": "Sc", "time_limit_min": 60, "penalty_per_min": 3 }),
    );
}

#[test]
fn an_empty_code_is_refused_not_dropped() {
    let (_dir, project) = with_score_course();
    let before = std::fs::read(&project).unwrap();
    let mut child = Command::new(BIN)
        .arg("mcp")
        .env("FREEMAPPER_LANG", "en")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let messages = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": { "name": "test", "version": "1" } } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized", "params": {} }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "set_score_controls",
            "arguments": { "project": project.to_str().unwrap(), "name": "Sc",
                           "controls": [{ "code": "31" }, { "code": " " }] } } }),
    ];
    for m in messages {
        writeln!(stdin, "{m}").unwrap();
    }
    stdin.flush().unwrap();
    let reply = loop {
        let mut line = String::new();
        assert!(stdout.read_line(&mut line).unwrap() > 0, "server closed");
        let v: Value = serde_json::from_str(&line).unwrap();
        if v["id"] == 2 {
            break v;
        }
    };
    let _ = child.kill();
    let _ = child.wait();
    assert_eq!(reply["result"]["isError"], true, "{reply}");
    assert!(
        reply["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("needs a code"),
        "{reply}"
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn score_settings_needs_a_switch() {
    let (_dir, project) = luebeck();
    let o = run("en", &["score", "settings", project.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

#[test]
fn csv_and_sheet_files_equal_over_cli_and_mcp() {
    let (dir, project) = with_score_course();
    let p = project.to_str().unwrap();
    let cli_csv = dir.path().join("cli.csv");
    let cli_md = dir.path().join("cli.md");
    let o = run(
        "en",
        &[
            "export",
            p,
            "--course",
            "Sc",
            "--csv",
            cli_csv.to_str().unwrap(),
            "--sheet",
            cli_md.to_str().unwrap(),
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("Sc"), "{}", out(&o));
    let mcp_csv = dir.path().join("mcp.csv");
    let mcp_md = dir.path().join("mcp.md");
    let a = mcp(
        "export_score_csv",
        json!({ "project": p, "course": "Sc", "path": mcp_csv.to_str().unwrap() }),
    );
    assert_eq!(a["controls"], 3, "{a}");
    let b = mcp(
        "export_score_sheet",
        json!({ "project": p, "course": "Sc", "path": mcp_md.to_str().unwrap() }),
    );
    assert_eq!(b["course"], "Sc", "{b}");
    let read = |f: &Path| std::fs::read_to_string(f).unwrap();
    assert_eq!(read(&cli_csv), read(&mcp_csv));
    assert_eq!(read(&cli_md), read(&mcp_md));
    assert!(read(&cli_csv).starts_with("code,value,x_mm,y_mm\n31,1,"));
    assert!(read(&cli_md).contains("Time limit: 45 minutes"));
    // An existing file is replaced.
    std::fs::write(&cli_csv, "old").unwrap();
    let o = run(
        "en",
        &[
            "export",
            p,
            "--course",
            "Sc",
            "--csv",
            cli_csv.to_str().unwrap(),
        ],
    );
    assert_eq!(o.status.code(), Some(0));
    assert!(read(&cli_csv).starts_with("code,"));
}

#[test]
fn export_refuses_a_non_score_course() {
    let (dir, project) = luebeck();
    let p = project.to_str().unwrap();
    let target = dir.path().join("x.csv");
    let list = run("en", &["list", "courses", p, "--json"]);
    let all: Value = serde_json::from_str(&out(&list)).unwrap();
    let name = all["courses"][0]["name"].as_str().unwrap().to_owned();
    let o = run(
        "en",
        &[
            "export",
            p,
            "--course",
            &name,
            "--csv",
            target.to_str().unwrap(),
        ],
    );
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(
        err(&o).trim(),
        format!("error: course {name} is not a score course")
    );
    assert!(!target.exists());
    let o = run("en", &["export", p, "--csv", target.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

#[test]
fn iof_xml_and_ppen_export_with_a_score_course() {
    let (dir, project) = with_score_course();
    let p = project.to_str().unwrap();
    let xml = dir.path().join("o.xml");
    let ppen = dir.path().join("o.ppen");
    let o = run(
        "en",
        &[
            "export",
            p,
            "--iof-xml",
            xml.to_str().unwrap(),
            "--ppen",
            ppen.to_str().unwrap(),
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let text = std::fs::read_to_string(&xml).unwrap();
    assert!(text.contains("<Score>1</Score>"), "{text}");
    let stdout = out(&o);
    assert!(
        stdout.contains("left out Sc: score courses are not exported to Purple Pen"),
        "{stdout}"
    );
    assert!(!stdout.contains("IOF XML"), "{stdout}");
}

#[test]
fn course_without_csv_or_sheet_is_refused() {
    let (dir, project) = with_score_course();
    let target = dir.path().join("o.xml");
    let o = run(
        "en",
        &[
            "export",
            project.to_str().unwrap(),
            "--course",
            "Sc",
            "--iof-xml",
            target.to_str().unwrap(),
        ],
    );
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
    assert!(!target.exists());
}

/// A draft refuses a start and finish that are further apart than `start_finish_max_m` or closer than `map_margin_m`
/// to the edge of the mapped area (ADR 0086, ADR 0087), names the setting and writes nothing; the settings change that.
#[test]
fn a_draft_refuses_ends_that_break_the_settings() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("m.omap");
    std::fs::copy(
        repo_root().join("examples/gleisdreieck/gleisdreieck.omap"),
        &target,
    )
    .unwrap();
    let project = dir.path().join("m.courseplan.toml");
    let p = project.to_str().unwrap();
    let ok = |args: &[&str]| {
        let o = run("en", args);
        assert_eq!(o.status.code(), Some(0), "{args:?}: {}", err(&o));
    };
    ok(&["init", target.to_str().unwrap()]);
    ok(&[
        "special", "add", p, "--kind", "start", "--id", "S", "--at", "100,-290",
    ]);
    // 600 m from the start in a straight line.
    ok(&[
        "special", "add", p, "--kind", "finish", "--id", "F", "--at", "100,-140",
    ]);
    let draft = |extra: &[&str]| {
        let mut args = vec![
            "score",
            "draft",
            p,
            "T",
            "--time-limit",
            "20",
            "--start",
            "S",
            "--finish",
            "F",
        ];
        args.extend_from_slice(extra);
        run("en", &args)
    };
    let o = draft(&[]);
    assert_eq!(o.status.code(), Some(1), "{}{}", out(&o), err(&o));
    let message = err(&o);
    assert!(
        message.contains("are 600 m apart") && message.contains("start_finish_max_m"),
        "{message}"
    );
    assert!(
        message.contains("(100, -290)") && message.contains("(100, -140)"),
        "{message}"
    );
    // The refusal is cheap: it comes before any map search, and nothing is written.
    assert!(
        !std::fs::read_to_string(&project)
            .unwrap()
            .contains("[[courses]]")
    );
    // A start 39 m from the edge of the mapped area breaks the margin of 50 m.
    ok(&[
        "special",
        "add",
        p,
        "--kind",
        "start",
        "--id",
        "E",
        "--at",
        "-64.643,-171.607",
    ]);
    ok(&[
        "special", "add", p, "--kind", "finish", "--id", "G", "--at", "-60,-165",
    ]);
    let o = run(
        "en",
        &[
            "score",
            "draft",
            p,
            "T",
            "--time-limit",
            "20",
            "--start",
            "E",
            "--finish",
            "G",
        ],
    );
    assert_eq!(o.status.code(), Some(1), "{}{}", out(&o), err(&o));
    let message = err(&o);
    assert!(
        message.contains("The start at (-64.643, -171.607)")
            && message.contains("from the edge of the mapped area")
            && message.contains("map_margin_m"),
        "{message}"
    );
    // The settings are stored and come back as the tool answers them.
    let o = run(
        "en",
        &[
            "score",
            "settings",
            p,
            "--dead-end-max",
            "80",
            "--start-finish-max",
            "700",
            "--map-margin",
            "30",
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(
        (
            &v["dead_end_max_m"],
            &v["start_finish_max_m"],
            &v["map_margin_m"]
        ),
        (&json!(80), &json!(700), &json!(30))
    );
    // Out of range is refused with one sentence.
    let o = run("en", &["score", "settings", p, "--dead-end-max", "10"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("dead_end_max_m"), "{}", err(&o));
}

#[test]
fn reject_site_json_is_the_mcp_answer() {
    same(
        &["score", "reject"],
        &["--at", "40,30.5"],
        "reject_site",
        json!({ "at": { "x": 40.0, "y": 30.5 } }),
    );
}

/// A project on the Rummelsburger Bucht map with a start and a finish, in a temp dir.
fn rummelsburg() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("m.omap");
    std::fs::copy(
        repo_root().join("examples/rummelsburger-bucht/rummelsburg.omap"),
        &target,
    )
    .unwrap();
    let project = dir.path().join("m.courseplan.toml");
    let p = project.to_str().unwrap();
    for args in [
        vec!["init", target.to_str().unwrap()],
        vec![
            "special", "add", p, "--kind", "start", "--id", "S", "--at", "3,0",
        ],
        vec![
            "special", "add", p, "--kind", "finish", "--id", "F", "--at", "10,5",
        ],
    ] {
        let o = run("en", &args);
        assert_eq!(o.status.code(), Some(0), "{args:?}: {}", err(&o));
    }
    (dir, project)
}

/// The positions of the candidates `score suggest` lists for the project.
fn suggested(project: &Path) -> Vec<(f64, f64)> {
    let o = run(
        "en",
        &[
            "score",
            "suggest",
            project.to_str().unwrap(),
            "--limit",
            "40",
            "--json",
        ],
    );
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let answer: Value = serde_json::from_str(&out(&o)).unwrap();
    answer["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["at"]["x"].as_f64().unwrap(),
                c["at"]["y"].as_f64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn a_rejected_site_stays_out_of_suggest_until_it_is_taken_back() {
    let (_dir, project) = rummelsburg();
    let path = project.to_str().unwrap();
    let before = suggested(&project);
    assert!(before.len() > 2, "{before:?}");
    let (x, y) = before[0];
    // The position given is 0.1 mm beside the site, well within the 3 m of tolerance at any sprint scale.
    let at = format!("{},{}", x + 0.1, y);
    let exact = format!("{x},{y}");
    let o = run("en", &["score", "reject", path, "--at", &at]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("rejected the site at"), "{}", out(&o));
    let after = suggested(&project);
    assert!(
        !after.iter().any(|&(ax, ay)| (ax - x).hypot(ay - y) < 0.01),
        "the rejected site came back: {after:?}"
    );
    // The project file keeps it.
    let text = std::fs::read_to_string(&project).unwrap();
    assert!(text.contains("rejected_sites"), "{text}");
    // Rejecting the same site again says so and changes nothing.
    let again = run("en", &["score", "reject", path, "--at", &exact]);
    assert_eq!(again.status.code(), Some(0));
    assert!(out(&again).contains("already"), "{}", out(&again));
    // Taking it back lets it return.
    let o = run("de", &["score", "unreject", path, "--at", &at]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    assert!(out(&o).contains("Ablehnung"), "{}", out(&o));
    assert!(
        suggested(&project)
            .iter()
            .any(|&(ax, ay)| (ax - x).hypot(ay - y) < 0.01)
    );
    // A site that is not rejected is an error with exit 1.
    let o = run("en", &["score", "unreject", path, "--at", &at]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("rejected site"), "{}", err(&o));
}
