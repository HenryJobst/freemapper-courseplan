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
    assert!(list.len() >= 20, "{}", list.len());
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
    assert!(v["length"]["good"]["percent"].as_f64().unwrap() >= 87.0);
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
        "90",
        "--apply",
    ]);
    assert_eq!(o.status.code(), Some(1), "{}", out(&o));
    let e = err(&o);
    assert!(e.starts_with("error: "), "{e}");
    assert!(e.contains("percentage points"), "{e}");
    assert!(
        e.contains("would help") || e.contains("Check the start"),
        "{e}"
    );
    assert_eq!(std::fs::read(&project).unwrap(), before);
    // Over MCP the same sentences come back as an error result.
    let (is_error, text) = mcp(
        "score_draft",
        json!({"project": p(&project), "name": "Sc", "time_limit_min": 90}),
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
    assert!(err(&o).contains("Forest maps"), "{}", err(&o));
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
