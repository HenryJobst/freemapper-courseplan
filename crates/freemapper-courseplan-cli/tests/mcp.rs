//! The MCP server spoken to over stdio, as an agent would.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

const TOOLS: [&str; 23] = [
    "event_info",
    "list_controls",
    "list_courses",
    "list_specials",
    "list_classes",
    "add_control",
    "move_control",
    "remove_control",
    "describe_control",
    "set_control_number",
    "add_special",
    "remove_special",
    "add_class",
    "remove_class",
    "add_course",
    "remove_course",
    "set_course_controls",
    "set_leg",
    "set_course_details",
    "place_numbers",
    "map_features_near",
    "map_extent",
    "init_event",
];

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    info: Value,
}

impl Session {
    fn start() -> Self {
        Self::start_in("en")
    }

    fn start_in(lang: &str) -> Self {
        Self::spawn(&["mcp"], lang)
    }

    fn spawn(args: &[&str], lang: &str) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_freemapper-courseplan"))
            .args(args)
            .env("FREEMAPPER_LANG", lang)
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .env("LANG", "C")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut session = Self {
            child,
            stdin,
            stdout,
            next_id: 1,
            info: Value::Null,
        };
        session.info = session.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1" }
            }),
        );
        session.send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        session
    }

    fn send(&mut self, message: Value) {
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let mut line = String::new();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "server closed"
            );
            let message: Value = serde_json::from_str(&line).unwrap();
            if message["id"] == id {
                return message["result"].clone();
            }
        }
    }

    /// Calls a tool; returns (is_error, text of the first content block).
    fn call(&mut self, tool: &str, arguments: Value) -> (bool, String) {
        let result = self.request(
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        );
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        (result["isError"] == true, text)
    }

    /// Calls a tool that must succeed and returns its JSON result.
    fn ok(&mut self, tool: &str, arguments: Value) -> Value {
        let (error, text) = self.call(tool, arguments);
        assert!(!error, "{tool}: {text}");
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["ok"], true, "{tool}: {text}");
        value
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// A temp dir with the Luebeck map and the path of the project `init_event` will write.
fn workspace() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("luebeck.omap");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/luebeck.omap"),
        &map,
    )
    .unwrap();
    let project = dir.path().join("luebeck.courseplan.toml");
    (dir, map, project)
}

fn point(extent: &Value, fx: f64, fy: f64) -> Value {
    let (min, max) = (&extent["min"], &extent["max"]);
    let at = |axis: &str, f: f64| {
        let (lo, hi) = (min[axis].as_f64().unwrap(), max[axis].as_f64().unwrap());
        lo + (hi - lo) * f
    };
    json!({ "x": at("x", fx), "y": at("y", fy) })
}

#[test]
fn lists_the_tools() {
    let mut session = Session::start();
    assert_eq!(session.info["serverInfo"]["name"], "freemapper-courseplan");
    let instructions = session.info["instructions"].as_str().unwrap();
    assert!(
        instructions.contains("event_info") && instructions.contains("place_numbers"),
        "{instructions}"
    );
    let tools = session.request("tools/list", json!({}));
    let listed = tools["tools"].as_array().unwrap();
    for name in TOOLS {
        let tool = listed
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("{name} is missing"));
        assert!(
            tool["description"].as_str().unwrap().len() > 8,
            "{name}: {tool}"
        );
        assert!(
            tool["inputSchema"]["properties"]["project"].is_object() || name == "init_event",
            "{name}: {tool}"
        );
    }
    assert_eq!(listed.len(), 23, "unexpected tools");
    let add = listed.iter().find(|t| t["name"] == "add_control").unwrap();
    assert!(add["inputSchema"]["properties"]["at"].is_object(), "{add}");
}

#[test]
fn german_descriptions_follow_the_language() {
    let mut session = Session::start_in("de");
    assert!(
        session.info["instructions"]
            .as_str()
            .unwrap()
            .contains("Bahnen"),
        "{}",
        session.info["instructions"]
    );
    let tools = session.request("tools/list", json!({}));
    let add = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "add_control")
        .unwrap();
    assert!(
        add["description"].as_str().unwrap().contains("Posten"),
        "{add}"
    );
    assert!(
        add["inputSchema"]["properties"]["project"]["description"]
            .as_str()
            .unwrap()
            .contains("Projektdatei"),
        "{add}"
    );
    let (_dir, map, project) = workspace();
    session.ok(
        "init_event",
        json!({ "map": map, "project": project, "title": "Test" }),
    );
    let (error, text) = session.call("event_info", json!({ "project": "x.toml" }));
    assert!(error);
    assert!(text.contains("relativer Pfad"), "{text}");
}

#[test]
fn sets_a_course_end_to_end() {
    let (dir, map, project) = workspace();
    let mut session = Session::start();
    let created = session.ok(
        "init_event",
        json!({ "map": map, "project": project, "title": "Training" }),
    );
    assert_eq!(created["project"], project.to_str().unwrap());
    assert!(project.exists());

    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let p = |fx, fy| point(&extent, fx, fy);
    session.ok(
        "add_special",
        json!({ "project": project, "kind": "start", "id": "S1", "at": p(0.4, 0.4) }),
    );
    session.ok(
        "add_special",
        json!({ "project": project, "kind": "finish", "id": "F1", "at": p(0.6, 0.6) }),
    );
    let first = session.ok(
        "add_control",
        json!({ "project": project, "at": p(0.45, 0.5) }),
    );
    assert_eq!(first["code"], "31");
    session.ok(
        "add_control",
        json!({ "project": project, "at": p(0.55, 0.45), "code": "32" }),
    );
    session.ok("add_class", json!({ "project": project, "name": "H10" }));
    session.ok(
        "add_course",
        json!({
            "project": project, "name": "A", "start": "S1", "finish": "F1",
            "controls": ["31", "32"], "classes": ["H10"], "climb_m": 12
        }),
    );
    let placed = session.ok("place_numbers", json!({ "project": project, "name": "A" }));
    assert_eq!(placed["placed"], 2);

    let info = session.ok("event_info", json!({ "project": project }));
    let courses = info["courses"].as_array().unwrap();
    assert_eq!(courses.len(), 1, "{info}");
    assert_eq!(courses[0]["name"], "A");
    assert_eq!(courses[0]["control_count"], 2);
    assert_eq!(courses[0]["controls"], json!(["31", "32"]));
    assert_eq!(courses[0]["numbers_set"], 2);
    assert_eq!(courses[0]["climb_m"], 12);
    assert!(courses[0]["length_km"].as_f64().unwrap() > 0.0, "{info}");
    assert_eq!(info["map_kind"], "sprint");

    let listed = session.ok("list_courses", json!({ "project": project }));
    assert_eq!(listed["courses"][0]["controls"], json!(["31", "32"]));
    let controls = session.ok("list_controls", json!({ "project": project }));
    assert_eq!(controls["controls"].as_array().unwrap().len(), 2);
    assert!(controls["controls"][0]["map_mm"]["x"].is_f64());
    drop(dir);
}

#[test]
fn relative_path_is_refused() {
    let mut session = Session::start();
    let dir = tempfile::tempdir().unwrap();
    let (error, text) = session.call("event_info", json!({ "project": "x.toml" }));
    assert!(error);
    assert!(text.contains("absolute path"), "{text}");
    assert!(!Path::new("x.toml").exists());
    let (error, text) = session.call(
        "init_event",
        json!({ "map": "luebeck.omap", "project": dir.path().join("p.toml") }),
    );
    assert!(error);
    assert!(text.contains("absolute path"), "{text}");
    assert!(!dir.path().join("p.toml").exists());
}

#[test]
fn a_refused_edit_leaves_the_file_unchanged() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let before = std::fs::read_to_string(&project).unwrap();
    let (error, text) = session.call(
        "add_control",
        json!({ "project": project, "at": { "x": 1.0e9, "y": 1.0e9 } }),
    );
    assert!(error);
    assert!(text.contains("outside the map"), "{text}");
    let (error, text) = session.call(
        "remove_control",
        json!({ "project": project, "code": "99" }),
    );
    assert!(error);
    assert!(text.contains("99"), "{text}");
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
}

#[test]
fn map_features_near_returns_nearest_first() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let centre = point(&extent, 0.5, 0.5);
    let near = session.ok(
        "map_features_near",
        json!({
            "project": project, "x": centre["x"], "y": centre["y"],
            "radius_mm": 30.0, "limit": 5
        }),
    );
    let features = near["features"].as_array().unwrap();
    assert!(!features.is_empty() && features.len() <= 5, "{near}");
    let distances: Vec<f64> = features
        .iter()
        .map(|f| f["distance_mm"].as_f64().unwrap())
        .collect();
    assert!(distances.windows(2).all(|w| w[0] <= w[1]), "{distances:?}");
    for f in features {
        assert!(["point", "line", "area"].contains(&f["kind"].as_str().unwrap()));
        assert!(f["part"].is_string());
        assert!(f.get("symbol_code").is_some());
    }
}

#[test]
fn course_details_legs_and_removal() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let p = |fx, fy| point(&extent, fx, fy);
    for (kind, id, at) in [("start", "S", p(0.3, 0.3)), ("finish", "F", p(0.7, 0.7))] {
        session.ok(
            "add_special",
            json!({ "project": project, "kind": kind, "id": id, "at": at }),
        );
    }
    session.ok(
        "add_special",
        json!({
            "project": project, "kind": "out_of_bounds", "id": "OOB",
            "points": [p(0.1, 0.1), p(0.2, 0.1), p(0.2, 0.2)]
        }),
    );
    for (code, at) in [("31", p(0.4, 0.5)), ("32", p(0.5, 0.4))] {
        session.ok(
            "add_control",
            json!({ "project": project, "at": at, "code": code }),
        );
    }
    session.ok(
        "add_course",
        json!({ "project": project, "name": "B", "start": "S", "finish": "F" }),
    );
    session.ok(
        "set_course_controls",
        json!({ "project": project, "name": "B", "codes": ["31", "32"] }),
    );
    session.ok(
        "set_leg",
        json!({ "project": project, "name": "B", "after": 1, "bends": [p(0.45, 0.55)], "taped": true }),
    );
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "B", "climb_m": 5, "print_scale": 4000 }),
    );
    session.ok(
        "describe_control",
        json!({ "project": project, "code": "31", "d": "1.1" }),
    );
    session.ok(
        "set_control_number",
        json!({ "project": project, "code": "31", "direction": 45.0, "gap_mm": 0.5 }),
    );
    let controls = session.ok("list_controls", json!({ "project": project }));
    assert_eq!(controls["controls"][0]["description"]["d"], "1.1");
    assert_eq!(controls["controls"][0]["number"]["direction"], 45.0);
    let courses = session.ok("list_courses", json!({ "project": project }));
    assert_eq!(courses["courses"][0]["climb_m"], 5);
    let (error, text) = session.call(
        "set_leg",
        json!({ "project": project, "name": "B", "after": 9 }),
    );
    assert!(error, "{text}");
    let (error, text) = session.call(
        "remove_control",
        json!({ "project": project, "code": "31" }),
    );
    assert!(!error, "{text}");
    assert!(text.contains("\"B\""), "{text}");
    session.ok("remove_course", json!({ "project": project, "name": "B" }));
    session.ok("remove_special", json!({ "project": project, "id": "OOB" }));
    let specials = session.ok("list_specials", json!({ "project": project }));
    assert_eq!(specials["specials"].as_array().unwrap().len(), 2);
    session.ok("add_class", json!({ "project": project, "name": "D10" }));
    session.ok("remove_class", json!({ "project": project, "name": "D10" }));
    let classes = session.ok("list_classes", json!({ "project": project }));
    assert!(classes["classes"].as_array().unwrap().is_empty());
}

#[test]
fn global_lang_before_the_subcommand_wins() {
    let session = Session::spawn(&["--lang", "de", "mcp"], "en");
    assert!(
        session.info["instructions"]
            .as_str()
            .unwrap()
            .contains("Bahnen"),
        "{}",
        session.info["instructions"]
    );
}

#[test]
fn map_features_near_limit_is_capped() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let centre = point(&extent, 0.5, 0.5);
    let args = |limit: Value| {
        json!({
            "project": project, "x": centre["x"], "y": centre["y"],
            "radius_mm": 100000.0, "limit": limit
        })
    };
    let all = session.ok("map_features_near", args(json!(100000)));
    assert_eq!(all["features"].as_array().unwrap().len(), 200, "{all}");
    let default = session.ok("map_features_near", args(Value::Null));
    assert_eq!(default["features"].as_array().unwrap().len(), 20);
}
