//! `score suggest|draft|length` on the command line and over MCP (ADR 0068), on a small synthetic map.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

#[path = "support/synthetic.rs"]
mod synthetic;

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn run(args: &[&str]) -> Output {
    Command::new(BIN)
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

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn json_of(o: &Output) -> Value {
    assert_eq!(o.status.code(), Some(0), "{}", err(o));
    serde_json::from_str(&out(o)).unwrap()
}

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = synthetic::project(dir.path());
    (dir, project)
}

fn p(project: &Path) -> &str {
    project.to_str().unwrap()
}

fn mcp(tool: &str, arguments: Value) -> (bool, String) {
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
        json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}}),
    );
    ask(0, "notifications/initialized", json!({}));
    let result = ask(
        2,
        "tools/call",
        json!({ "name": tool, "arguments": arguments }),
    );
    let _ = child.kill();
    let _ = child.wait();
    (
        result["isError"] == true,
        result["content"][0]["text"].as_str().unwrap().to_owned(),
    )
}

const LIMIT: &str = "6";

#[test]
fn suggest_returns_ranked_candidates_and_writes_nothing() {
    let (_dir, project) = fixture();
    let before = std::fs::read(&project).unwrap();
    let v = json_of(&run(&["score", "suggest", p(&project), "--json"]));
    let list = v["candidates"].as_array().unwrap();
    assert!(list.len() >= 10, "{}", list.len());
    for c in list {
        assert!(c["at"]["x"].is_number() && c["symbol_code"].is_string());
        assert!(c["effort_s"].as_f64().unwrap() > 0.0);
        assert!((1..=5).contains(&c["value"].as_u64().unwrap()));
    }
    let choice: Vec<f64> = list
        .iter()
        .map(|c| c["route_choice"].as_f64().unwrap())
        .collect();
    assert!(choice.windows(2).all(|w| w[0] >= w[1]), "{choice:?}");
    assert_eq!(std::fs::read(&project).unwrap(), before);
    let text = run(&["score", "suggest", p(&project), "--limit", "3"]);
    assert!(out(&text).contains("3 candidates"), "{}", out(&text));
}

#[test]
fn draft_without_apply_writes_nothing() {
    let (_dir, project) = fixture();
    let before = std::fs::read(&project).unwrap();
    let v = json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--json",
    ]));
    assert_eq!(v["applied"], false);
    assert!(!v["controls"].as_array().unwrap().is_empty());
    assert_eq!(v["length"]["ok"], true, "{v}");
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn draft_with_apply_creates_a_checked_course() {
    let (_dir, project) = fixture();
    let v = json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--apply",
        "--json",
    ]));
    assert_eq!(v["applied"], true);
    let n = v["controls"].as_array().unwrap().len();
    let courses = json_of(&run(&["list", "courses", p(&project), "--json"]));
    let sc = courses["courses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Sc")
        .unwrap();
    assert_eq!(sc["kind"], "score");
    let controls = json_of(&run(&["list", "controls", p(&project), "--json"]));
    assert_eq!(controls["controls"].as_array().unwrap().len(), n);
    // `check` exits with 1 for a finding of severity error; the score rules may warn but not err.
    let check = run(&["check", p(&project)]);
    assert_eq!(
        check.status.code(),
        Some(0),
        "{}\n{}",
        out(&check),
        err(&check)
    );
    // A second apply replaces the controls instead of adding to them.
    let again = json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--apply",
        "--json",
    ]));
    assert_eq!(again["controls"], v["controls"]);
    let controls = json_of(&run(&["list", "controls", p(&project), "--json"]));
    assert_eq!(controls["controls"].as_array().unwrap().len(), n);
}

#[test]
fn length_reports_the_bands() {
    let (_dir, project) = fixture();
    json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--apply",
        "--json",
    ]));
    let v = json_of(&run(&["score", "length", p(&project), "Sc", "--json"]));
    assert_eq!(v["length"]["ok"], true, "{v}");
    let before = std::fs::read(&project).unwrap();
    // A shorter limit puts the very good runner out of reach of some controls: the miss is reported.
    let o = run(&[
        "score",
        "course",
        "set",
        p(&project),
        "Sc",
        "--time-limit",
        "3",
    ]);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let after = std::fs::read(&project).unwrap();
    assert_ne!(before, after);
    let v = json_of(&run(&["score", "length", p(&project), "Sc", "--json"]));
    assert_eq!(v["length"]["ok"], false);
    assert!(!v["length"]["issues"].as_array().unwrap().is_empty());
    let text = run(&["score", "length", p(&project), "Sc"]);
    assert!(out(&text).contains("good runner"), "{}", out(&text));
    assert_eq!(std::fs::read(&project).unwrap(), after);
}

#[test]
fn failed_draft_exits_with_one_and_names_the_hint() {
    let (_dir, project) = fixture();
    let before = std::fs::read(&project).unwrap();
    let o = run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        "1",
        "--apply",
    ]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let e = err(&o);
    assert!(e.starts_with("error: "), "{e}");
    assert!(e.contains("percentage points"), "{e}");
    assert!(
        e.contains("would help")
            || e.contains("Check the start")
            || e.contains("Raise the highest value"),
        "{e}"
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
    // Over MCP the same sentences come back as an error result.
    let (is_error, text) = mcp(
        "score_draft",
        json!({"project": p(&project), "name": "Sc", "time_limit_min": 1}),
    );
    assert!(is_error);
    assert_eq!(format!("error: {text}\n"), e);
}

#[test]
fn cli_and_mcp_answers_are_equal() {
    let (_a, cli) = fixture();
    let (_b, via) = fixture();
    for (args, tool, arguments) in [
        (
            vec!["score", "suggest", p(&cli), "--limit", "8", "--json"],
            "score_suggest",
            json!({"limit": 8}),
        ),
        (
            vec![
                "score",
                "draft",
                p(&cli),
                "Sc",
                "--time-limit",
                LIMIT,
                "--json",
            ],
            "score_draft",
            json!({"name": "Sc", "time_limit_min": 6}),
        ),
    ] {
        let mut arguments = arguments;
        arguments["project"] = json!(p(&via));
        let (is_error, text) = mcp(tool, arguments);
        assert!(!is_error, "{text}");
        assert_eq!(
            json_of(&run(&args)),
            serde_json::from_str::<Value>(&text).unwrap()
        );
    }
}

#[test]
fn forest_map_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/forest-sample.omap"),
        dir.path().join("forest.omap"),
    )
    .unwrap();
    let project = dir.path().join("event.courseplan.toml");
    std::fs::write(
        &project,
        "format = 1\n\n[event]\ntitle = \"F\"\ndiscipline = \"training\"\nrule_profile = \"wkb-2026\"\n\n[sources]\nmap = \"forest.omap\"\n\n[[specials]]\nkind = \"start\"\nid = \"S1\"\nmap_mm = [0.0, 0.0]\n\n[[specials]]\nkind = \"finish\"\nid = \"F1\"\nmap_mm = [1.0, 1.0]\n",
    )
    .unwrap();
    let o = run(&["score", "suggest", p(&project)]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("forest maps"), "{}", err(&o));
}

#[test]
fn start_inside_a_building_gives_a_sentence() {
    let dir = tempfile::tempdir().unwrap();
    let project = synthetic::write_project(dir.path(), (-60.0, 0.0), (-92.0, 18.0));
    let o = run(&["score", "suggest", p(&project)]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("start at"), "{}", err(&o));
    assert!(err(&o).contains("blocked area"), "{}", err(&o));
}

#[test]
fn suggest_takes_start_and_finish_from_the_named_course() {
    // The first start lies inside a building; the course uses the second one.
    let dir = tempfile::tempdir().unwrap();
    let project = synthetic::write_project(dir.path(), (-60.0, 0.0), (-92.0, 18.0));
    let mut text = std::fs::read_to_string(&project).unwrap();
    text.push_str("\n[[specials]]\nkind = \"start\"\nid = \"S2\"\nmap_mm = [-92.0, -18.0]\n");
    std::fs::write(&project, text).unwrap();
    let made = run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--start",
        "S2",
        "--apply",
    ]);
    assert_eq!(made.status.code(), Some(0), "{}", err(&made));
    let plain = run(&["score", "suggest", p(&project)]);
    assert_eq!(
        plain.status.code(),
        Some(1),
        "the first start is unreachable"
    );
    let v = json_of(&run(&[
        "score",
        "suggest",
        p(&project),
        "--course",
        "Sc",
        "--json",
    ]));
    assert!(v["count"].as_u64().unwrap() > 0);
    assert_eq!(v["start"]["x"].as_f64().unwrap(), -92.0);
    let (is_error, text) = mcp(
        "score_suggest",
        json!({"project": p(&project), "course": "Sc"}),
    );
    assert!(!is_error, "{text}");
    let (is_error, _) = mcp(
        "score_suggest",
        json!({"project": p(&project), "course": "Nope"}),
    );
    assert!(is_error);
}

#[test]
fn draft_is_deterministic() {
    let (_a, one) = fixture();
    let (_b, two) = fixture();
    let args = |project: &Path| {
        run(&[
            "score",
            "draft",
            p(project),
            "Sc",
            "--time-limit",
            LIMIT,
            "--apply",
            "--json",
        ])
    };
    assert_eq!(out(&args(&one)), out(&args(&two)));
    assert_eq!(std::fs::read(&one).unwrap(), std::fs::read(&two).unwrap());
}

fn ok(args: &[&str]) -> Output {
    let o = run(args);
    assert_eq!(o.status.code(), Some(0), "{args:?}: {}", err(&o));
    o
}

fn course(project: &Path, name: &str) -> Value {
    let all = json_of(&run(&["list", "courses", p(project), "--json"]));
    all["courses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
        .clone()
}

fn codes(project: &Path) -> Vec<String> {
    let all = json_of(&run(&["list", "controls", p(project), "--json"]));
    all["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["code"].to_string().trim_matches('"').to_owned())
        .collect()
}

#[test]
fn draft_on_a_course_of_another_kind_is_refused() {
    let (_dir, project) = fixture();
    ok(&[
        "course",
        "add",
        p(&project),
        "A",
        "--start",
        "S1",
        "--finish",
        "F1",
    ]);
    let before = std::fs::read(&project).unwrap();
    let o = run(&[
        "score",
        "draft",
        p(&project),
        "A",
        "--time-limit",
        LIMIT,
        "--apply",
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("score course"), "{}", err(&o));
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn redraft_keeps_the_course_details_and_the_other_courses_controls() {
    let (_dir, project) = fixture();
    // A control that an individual course uses must survive, and its code must stay out of the draft.
    ok(&[
        "control",
        "add",
        p(&project),
        "--at",
        "-80,0",
        "--code",
        "31",
    ]);
    ok(&[
        "course",
        "add",
        p(&project),
        "A",
        "--start",
        "S1",
        "--finish",
        "F1",
        "--controls",
        "31",
    ]);
    ok(&[
        "course",
        "add",
        p(&project),
        "Sc",
        "--kind",
        "score",
        "--start",
        "S1",
        "--finish",
        "F1",
        "--time-limit",
        LIMIT,
        "--penalty",
        "9",
    ]);
    let first = json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--apply",
        "--json",
    ]));
    let drafted: Vec<String> = first["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["code"].to_string().trim_matches('"').to_owned())
        .collect();
    assert!(!drafted.contains(&"31".to_owned()), "{drafted:?}");
    let all = codes(&project);
    assert_eq!(all.len(), drafted.len() + 1, "{all:?}");
    let again = json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--apply",
        "--json",
    ]));
    assert_eq!(again["replaced"], true);
    assert_eq!(codes(&project).len(), drafted.len() + 1);
    assert!(codes(&project).contains(&"31".to_owned()));
    let sc = course(&project, "Sc");
    assert_eq!(sc["start"], "S1");
    assert_eq!(sc["finish"], "F1");
    assert_eq!(sc["score"]["penalty_per_min"], 9);
    assert_eq!(sc["score"]["penalty_is_default"], false);
    assert_eq!(
        course(&project, "A")["controls"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn limit_zero_returns_no_candidates() {
    let (_dir, project) = fixture();
    let v = json_of(&run(&[
        "score",
        "suggest",
        p(&project),
        "--limit",
        "0",
        "--json",
    ]));
    assert_eq!(v["count"], 0);
    assert!(v["candidates"].as_array().unwrap().is_empty());
}

#[test]
fn length_of_an_unknown_course_is_refused() {
    let (_dir, project) = fixture();
    let o = run(&["score", "length", p(&project), "Nope"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("Nope"), "{}", err(&o));
}

#[test]
fn a_link_position_that_is_no_candidate_is_refused() {
    let (_dir, project) = fixture();
    let before = std::fs::read(&project).unwrap();
    let o = run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--link",
        "-80,0;-80,5",
        "--apply",
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(err(&o).contains("lies on no candidate"), "{}", err(&o));
    assert_eq!(std::fs::read(&project).unwrap(), before);
}

#[test]
fn apply_with_links_stores_them() {
    let (_dir, project) = fixture();
    let v = json_of(&run(&["score", "suggest", p(&project), "--json"]));
    let list = v["candidates"].as_array().unwrap();
    let at = |c: &Value| {
        (
            c["at"]["x"].as_f64().unwrap(),
            c["at"]["y"].as_f64().unwrap(),
        )
    };
    // The two corners of one building that lie nearest to each other.
    let a = at(&list[0]);
    let b = list[1..]
        .iter()
        .map(at)
        .min_by(|p, q| {
            let d = |r: &(f64, f64)| (r.0 - a.0).hypot(r.1 - a.1);
            d(p).total_cmp(&d(q))
        })
        .unwrap();
    let link = format!("{},{};{},{}", a.0, a.1, b.0, b.1);
    let v = json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--link",
        &link,
        "--apply",
        "--json",
    ]));
    assert_eq!(v["links"].as_array().unwrap().len(), 1, "{v}");
    let stored = course(&project, "Sc");
    assert_eq!(
        stored["score"]["links"].as_array().unwrap().len(),
        1,
        "{stored}"
    );
}

// --- Bands are targets with a tolerance (ADR 0083) -------------------------------------------------------------

#[test]
fn the_bands_and_the_tolerance_are_settings() {
    let (_dir, project) = fixture();
    let v = json_of(&run(&[
        "score",
        "settings",
        p(&project),
        "--good-band",
        "80-95",
        "--very-good-band",
        "90-100",
        "--band-tolerance",
        "5",
        "--json",
    ]));
    assert_eq!(v["good_band_percent"], json!([80, 95]), "{v}");
    assert_eq!(v["very_good_band_percent"], json!([90, 100]));
    assert_eq!(v["band_tolerance_percent"], 5);
    // The text answer names them.
    let o = ok(&["score", "settings", p(&project), "--band-tolerance", "4"]);
    assert!(
        out(&o).contains("tolerance 4 percentage points"),
        "{}",
        out(&o)
    );
    assert!(out(&o).contains("band 80-95 percent"), "{}", out(&o));
    // Bad values are refused and leave the file alone.
    let before = std::fs::read(&project).unwrap();
    for (flag, value) in [
        ("--band-tolerance", "11"),
        ("--good-band", "95-80"),
        ("--good-band", "87"),
        ("--very-good-band", "70-100"),
        ("--good-band", "87-101"),
    ] {
        let o = run(&["score", "settings", p(&project), flag, value]);
        assert_ne!(o.status.code(), Some(0), "{flag} {value}");
        assert_eq!(std::fs::read(&project).unwrap(), before, "{flag} {value}");
    }
    // German texts.
    let o = Command::new(BIN)
        .args([
            "--lang",
            "de",
            "score",
            "settings",
            p(&project),
            "--band-tolerance",
            "3",
        ])
        .output()
        .unwrap();
    assert!(out(&o).contains("Toleranz 3 Prozentpunkte"), "{}", out(&o));
}

#[test]
fn a_length_report_is_in_band_near_or_missed() {
    let (_dir, project) = fixture();
    json_of(&run(&[
        "score",
        "draft",
        p(&project),
        "Sc",
        "--time-limit",
        LIMIT,
        "--apply",
        "--json",
    ]));
    let report = || json_of(&run(&["score", "length", p(&project), "Sc", "--json"]));
    let v = report();
    let percent = v["length"]["good"]["percent"].as_f64().unwrap();
    assert!(
        ["in_band", "near"].contains(&v["length"]["state"].as_str().unwrap()),
        "{v}"
    );
    // Move the good band so that its top lies 4 to 5 points below the result: beyond a tolerance of 3, within 5.
    let high = percent.floor() as i64 - 4;
    assert!(high >= 10, "{percent}");
    let band = format!("{}-{high}", high - 10);
    for (tolerance, state) in [("3", "missed"), ("5", "near")] {
        ok(&[
            "score",
            "settings",
            p(&project),
            "--good-band",
            &band,
            "--very-good-band",
            "90-100",
            "--band-tolerance",
            tolerance,
        ]);
        let v = report();
        assert_eq!(v["length"]["state"], state, "{tolerance}: {v}");
        assert_eq!(v["length"]["good"]["state"], state);
        assert_eq!(v["length"]["ok"], state == "near");
        assert_eq!(
            v["length"]["tolerance_percent"]
                .as_u64()
                .unwrap()
                .to_string(),
            tolerance
        );
    }
    // The text names the deviation and the state.
    let o = run(&["score", "length", p(&project), "Sc"]);
    assert!(
        out(&o).contains("within the tolerance of 5 percentage points"),
        "{}",
        out(&o)
    );
    assert!(out(&o).contains("near the target, no miss"), "{}", out(&o));
    // The check reports a miss, not a near deviation.
    let check = |tol: &str| {
        ok(&["score", "settings", p(&project), "--band-tolerance", tol]);
        out(&run(&["check", p(&project)]))
    };
    assert!(
        !check("5").contains("score_length_bands") && !check("5").contains("length_bands"),
        "near is no finding"
    );
    assert!(check("3").contains("length_bands"), "a miss is a finding");
}

/// A project on the synthetic ISOM 2017-2 forest map with start and finish and the controls 31 and 32.
fn forest_project() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("forest.omap");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/maps/isom2017-synthetic.omap"),
        &map,
    )
    .unwrap();
    let project = dir.path().join("forest.courseplan.toml");
    let ok = |o: Output| assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    ok(run(&["init", p(&map)]));
    ok(run(&[
        "special",
        "add",
        p(&project),
        "--kind",
        "start",
        "--id",
        "S1",
        "--at",
        "1,7",
    ]));
    ok(run(&[
        "special",
        "add",
        p(&project),
        "--kind",
        "finish",
        "--id",
        "F1",
        "--at",
        "11,7",
    ]));
    ok(run(&["control", "add", p(&project), "--at", "4,9.5"]));
    ok(run(&["control", "add", p(&project), "--at", "8,9.5"]));
    ok(run(&[
        "course",
        "add",
        p(&project),
        "SC",
        "--kind",
        "score",
        "--time-limit",
        "45",
        "--start",
        "S1",
        "--finish",
        "F1",
    ]));
    (dir, project)
}

const FOREST_REFUSAL: &str = "do not work on forest maps";

#[test]
fn score_commands_refuse_a_forest_map_before_any_raster() {
    let (_dir, project) = forest_project();
    let started = std::time::Instant::now();
    for args in [
        vec!["score", "suggest", p(&project)],
        vec!["score", "draft", p(&project), "SC", "--time-limit", "15"],
        vec!["score", "length", p(&project), "SC"],
    ] {
        let o = run(&args);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert!(err(&o).starts_with("error: "), "{}", err(&o));
    }
    assert!(started.elapsed().as_secs() < 5, "{:?}", started.elapsed());
}

#[test]
fn score_draft_and_length_on_a_forest_course_say_so() {
    let (_dir, project) = forest_project();
    for args in [
        vec!["score", "suggest", p(&project), "--course", "SC"],
        vec!["score", "draft", p(&project), "SC", "--time-limit", "15"],
        vec!["score", "length", p(&project), "SC"],
    ] {
        let o = run(&args);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert!(err(&o).contains(FOREST_REFUSAL), "{args:?}: {}", err(&o));
    }
}

#[test]
fn the_mcp_score_tools_refuse_a_forest_map() {
    let (_dir, project) = forest_project();
    for (tool, args) in [
        ("score_suggest", json!({ "project": p(&project) })),
        (
            "score_draft",
            json!({ "project": p(&project), "name": "SC", "time_limit_min": 15 }),
        ),
        (
            "score_length",
            json!({ "project": p(&project), "name": "SC" }),
        ),
    ] {
        let (is_error, text) = mcp(tool, args);
        assert!(is_error, "{tool}: {text}");
        assert!(text.contains(FOREST_REFUSAL), "{tool}: {text}");
    }
}

#[test]
fn a_score_course_set_by_hand_works_on_a_forest_map() {
    let (_dir, project) = forest_project();
    let ok = |o: Output| assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    ok(run(&["score", "controls", p(&project), "SC", "31,32"]));
    let o = run(&["check", p(&project), "--all"]);
    assert_eq!(o.status.code(), Some(0), "{}{}", out(&o), err(&o));
    // The length bands need route times, which a forest map does not get: listed as not checked.
    assert!(
        out(&o).contains("wkb2026.score.length_bands"),
        "{}",
        out(&o)
    );
    assert!(
        out(&o).contains("route times are not computed on a forest map"),
        "{}",
        out(&o)
    );
}
