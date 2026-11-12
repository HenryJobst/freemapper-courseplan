//! `list`, `map` and `event start-window --none` on the command line (ADR 0059): `--json` equals the MCP
//! tool's answer on the Luebeck example, and the errors are the CLI's.

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

/// The `--json` output of the CLI equals the MCP answer, for one command and tool.
fn same(cli: &[&str], tool: &str, extra: Value) {
    let (_dir, project) = luebeck();
    let path = project.to_str().unwrap();
    let mut args = cli.to_vec();
    args.insert(2.min(args.len()), path);
    args.push("--json");
    let o = run("en", &args);
    assert_eq!(o.status.code(), Some(0), "{}", err(&o));
    let mut arguments = json!({ "project": path });
    for (k, v) in extra.as_object().into_iter().flatten() {
        arguments[k] = v.clone();
    }
    let via_mcp = mcp(tool, arguments);
    let via_cli: Value = serde_json::from_str(&out(&o)).unwrap();
    assert_eq!(via_cli, via_mcp);
}

#[test]
fn list_controls_json_is_the_mcp_answer() {
    same(&["list", "controls"], "list_controls", json!({}));
}

#[test]
fn list_courses_json_is_the_mcp_answer() {
    same(&["list", "courses"], "list_courses", json!({}));
}

#[test]
fn list_specials_json_is_the_mcp_answer() {
    same(&["list", "specials"], "list_specials", json!({}));
}

#[test]
fn list_classes_json_is_the_mcp_answer() {
    same(&["list", "classes"], "list_classes", json!({}));
}

#[test]
fn map_extent_json_is_the_mcp_answer() {
    same(&["map", "extent"], "map_extent", json!({}));
}

#[test]
fn map_near_json_is_the_mcp_answer() {
    same(
        &[
            "map",
            "near",
            "--at",
            "-50.1,-3.6",
            "--radius",
            "12.5",
            "--limit",
            "5",
        ],
        "map_features_near",
        json!({ "x": -50.1, "y": -3.6, "radius_mm": 12.5, "limit": 5 }),
    );
}

#[test]
fn tables_name_what_the_project_has() {
    let (_dir, project) = luebeck();
    let p = project.to_str().unwrap();
    let controls = out(&run("en", &["list", "controls", p]));
    assert!(controls.starts_with("code  kind"), "{controls}");
    assert!(controls.contains("31    control  -50.1,-3.6"), "{controls}");
    let courses = out(&run("en", &["list", "courses", p]));
    assert!(
        courses.contains("individual") && courses.contains("1.287"),
        "{courses}"
    );
    let specials = out(&run("de", &["list", "specials", p]));
    assert!(
        specials.contains("Start") && specials.contains("S1"),
        "{specials}"
    );
    let classes = out(&run("en", &["list", "classes", p]));
    assert!(classes.contains("Offen"), "{classes}");
    let extent = out(&run("de", &["map", "extent", p]));
    assert!(extent.contains("Umfang: -127.55,-102.063 bis"), "{extent}");
    let near = out(&run(
        "en",
        &[
            "map", "near", p, "--at", "0,0", "--radius", "20", "--limit", "2",
        ],
    ));
    assert!(
        near.contains("2 of ") && near.contains("objects shown"),
        "{near}"
    );
}

#[test]
fn start_window_can_be_removed_and_set_again() {
    let (_dir, project) = luebeck();
    let p = project.to_str().unwrap();
    let set = run("en", &["event", "start-window", p, "90"]);
    assert_eq!(set.status.code(), Some(0), "{}", err(&set));
    assert!(
        std::fs::read_to_string(&project)
            .unwrap()
            .contains("start_window_min")
    );
    let none = run("en", &["event", "start-window", p, "--none"]);
    assert_eq!(none.status.code(), Some(0), "{}", err(&none));
    assert_eq!(out(&none).trim(), "removed the start window");
    assert!(
        !std::fs::read_to_string(&project)
            .unwrap()
            .contains("start_window_min")
    );
    let de = run("de", &["event", "start-window", p, "--none"]);
    assert_eq!(out(&de).trim(), "Startfenster entfernt");
    let again = run("en", &["event", "start-window", p, "45"]);
    assert_eq!(again.status.code(), Some(0));
    assert!(
        std::fs::read_to_string(&project)
            .unwrap()
            .contains("start_window_min = 45")
    );
}

#[test]
fn start_window_wants_exactly_one_of_minutes_and_none() {
    let (_dir, project) = luebeck();
    let p = project.to_str().unwrap();
    assert_eq!(
        run("en", &["event", "start-window", p]).status.code(),
        Some(2)
    );
    let both = run("en", &["event", "start-window", p, "5", "--none"]);
    assert_eq!(both.status.code(), Some(2));
    assert_eq!(out(&both), "");
}

#[test]
fn errors_go_to_stderr_with_the_exit_codes() {
    let (dir, project) = luebeck();
    let p = project.to_str().unwrap();
    let missing = dir.path().join("none.courseplan.toml");
    let m = missing.to_str().unwrap();
    for args in [
        vec!["list", "controls", m],
        vec!["list", "courses", m, "--json"],
        vec!["map", "extent", m],
        vec!["map", "near", m, "--at", "0,0", "--radius", "5"],
    ] {
        let o = run("en", &args);
        assert_eq!(o.status.code(), Some(1), "{args:?}");
        assert_eq!(out(&o), "");
        assert!(err(&o).starts_with("error: "), "{}", err(&o));
    }
    for (args, needle) in [
        (
            vec!["map", "near", p, "--at", "1;2", "--radius", "5"],
            "not a point",
        ),
        (
            vec!["map", "near", p, "--at", "0,0", "--radius", "0"],
            "not a radius",
        ),
        (
            vec![
                "map", "near", p, "--at", "0,0", "--radius", "5", "--limit", "x",
            ],
            "whole number",
        ),
    ] {
        let o = run("en", &args);
        assert_eq!(o.status.code(), Some(2), "{args:?}");
        assert!(err(&o).contains(needle), "{}", err(&o));
    }
    let de = run("de", &["list", "controls", m]);
    assert!(err(&de).starts_with("Fehler: "), "{}", err(&de));
    // A map that is gone is reported without a trace on stdout.
    std::fs::remove_file(dir.path().join("luebeck.omap")).unwrap();
    let o = run("en", &["map", "extent", p]);
    assert_eq!(o.status.code(), Some(1));
    assert_eq!(out(&o), "");
}
