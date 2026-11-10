//! The MCP server spoken to over stdio, as an agent would.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

const TOOLS: [&str; 34] = [
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
    "set_special_courses",
    "add_class",
    "remove_class",
    "set_class",
    "add_course",
    "remove_course",
    "set_course_controls",
    "set_leg",
    "set_course_details",
    "place_numbers",
    "set_course_number",
    "map_features_near",
    "map_extent",
    "init_event",
    "check",
    "export_iof_xml",
    "export_ppen",
    "import_ppen",
    "accept_finding",
    "remove_accepted_finding",
    "set_rule_profile",
    "set_start_window",
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
        self.request_full(method, params)["result"].clone()
    }

    /// The whole response, so a test can see a JSON-RPC `error` too.
    fn request_full(&mut self, method: &str, params: Value) -> Value {
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
                return message;
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
    assert_eq!(listed.len(), TOOLS.len(), "unexpected tools");
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

#[test]
fn non_toml_project_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let secret = dir.path().join("notes.txt");
    std::fs::write(&secret, "api_key = hunter2-secret\n").unwrap();
    let mut session = Session::start();
    for tool in ["event_info", "list_controls", "map_extent"] {
        let (error, text) = session.call(tool, json!({ "project": secret }));
        assert!(error, "{tool}: {text}");
        assert!(text.contains(".toml"), "{text}");
        assert!(!text.contains("hunter2"), "{text}");
    }
    let (error, text) = session.call("add_class", json!({ "project": secret, "name": "H10" }));
    assert!(error && !text.contains("hunter2"), "{text}");
    assert_eq!(
        std::fs::read_to_string(&secret).unwrap(),
        "api_key = hunter2-secret\n"
    );
}

#[test]
fn parse_errors_do_not_quote_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.toml");
    std::fs::write(&broken, "password = hunter2-secret = oops\n").unwrap();
    let mut session = Session::start();
    let (error, text) = session.call("event_info", json!({ "project": broken }));
    assert!(error, "{text}");
    assert!(text.contains("line 1"), "{text}");
    assert!(
        !text.contains("hunter2") && !text.contains("password"),
        "{text}"
    );
    assert!(!text.contains('^'), "{text}");
}

#[test]
fn init_event_refuses_non_map_file() {
    let dir = tempfile::tempdir().unwrap();
    let not_a_map = dir.path().join("secret.txt");
    std::fs::write(&not_a_map, "hunter2").unwrap();
    let mut session = Session::start();
    let (error, text) = session.call("init_event", json!({ "map": not_a_map }));
    assert!(error, "{text}");
    assert!(
        text.contains(".omap") && !text.contains("hunter2"),
        "{text}"
    );
    let (_work, map, _project) = workspace();
    let (error, text) = session.call(
        "init_event",
        json!({ "map": map, "project": dir.path().join("out.txt") }),
    );
    assert!(error && text.contains(".toml"), "{text}");
    assert!(!dir.path().join("out.txt").exists());
}

#[test]
fn misspelt_parameter_is_refused() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let before = std::fs::read_to_string(&project).unwrap();
    let message = session.request_full(
        "tools/call",
        json!({ "name": "set_leg", "arguments": {
            "project": project, "name": "A", "from_start": true, "bend": [{ "x": 1.0, "y": 2.0 }]
        } }),
    );
    let refused = message.get("error").is_some() || message["result"]["isError"] == true;
    assert!(refused, "{message}");
    assert!(message.to_string().contains("bend"), "{message}");
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
}

#[test]
fn schema_errors_do_not_quote_values() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("creds.toml");
    std::fs::write(&file, "format = \"hunter2\"\nuser = \"hunter2\"\n").unwrap();
    let mut session = Session::start();
    let (error, text) = session.call("event_info", json!({ "project": file }));
    assert!(error, "{text}");
    assert!(!text.contains("hunter2"), "{text}");
}

#[test]
fn map_errors_do_not_quote_the_file() {
    let (dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    std::fs::write(&map, "<map><barrier hunter2 =\n").unwrap();
    for tool in ["map_extent", "event_info"] {
        let (error, text) = session.call(tool, json!({ "project": project }));
        assert!(error, "{tool}: {text}");
        assert!(!text.contains("hunter2"), "{text}");
    }
    drop(dir);
}

#[test]
fn unknown_keys_of_a_foreign_toml_are_not_listed() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("settings.toml");
    std::fs::write(&file, "api_token = 1\npassword = 2\n[server]\nhost = 3\n").unwrap();
    let mut session = Session::start();
    let (error, text) = session.call("event_info", json!({ "project": file }));
    assert!(error, "{text}");
    for name in ["api_token", "password", "server"] {
        assert!(!text.contains(name), "{text}");
    }
}

#[test]
fn init_event_map_errors_do_not_quote_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("broken.omap");
    std::fs::write(&map, "<map><barrier hunter2 =\n").unwrap();
    let mut session = Session::start();
    let (error, text) = session.call("init_event", json!({ "map": map }));
    assert!(error, "{text}");
    assert!(!text.contains("hunter2"), "{text}");
    assert!(!dir.path().join("broken.courseplan.toml").exists());
}

#[test]
fn binary_omap_keeps_its_hint_over_mcp() {
    let (dir, map, project) = workspace();
    let mut session = Session::start();
    std::fs::write(&map, b"OMAP\0\0\0\x01rest").unwrap();
    let (error, text) = session.call("init_event", json!({ "map": map, "project": project }));
    assert!(error, "{text}");
    assert!(text.contains("old binary format"), "{text}");
    drop(dir);
}

/// Start S, finish F, controls 31 and 32 and course A over both.
fn course_a(session: &mut Session, project: &Path) -> Value {
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let p = |fx, fy| point(&extent, fx, fy);
    for (kind, id, at) in [("start", "S", p(0.3, 0.3)), ("finish", "F", p(0.7, 0.7))] {
        session.ok(
            "add_special",
            json!({ "project": project, "kind": kind, "id": id, "at": at }),
        );
    }
    for (code, at) in [("31", p(0.4, 0.5)), ("32", p(0.5, 0.4))] {
        session.ok(
            "add_control",
            json!({ "project": project, "at": at, "code": code }),
        );
    }
    session.ok(
        "add_course",
        json!({ "project": project, "name": "A", "start": "S", "finish": "F", "controls": ["31", "32"] }),
    );
    extent
}

fn course_numbers(project: &Path) -> Vec<Option<(f64, f64)>> {
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
fn set_course_number_sets_clears_and_checks_ranges() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    course_a(&mut session, &project);
    let set = session.ok(
        "set_course_number",
        json!({ "project": project, "name": "A", "after": 2, "direction": -90.0, "gap_mm": 1.0 }),
    );
    assert_eq!(set["number"]["direction"], 270.0, "{set}");
    assert_eq!(course_numbers(&project), [None, Some((270.0, 1.0))]);
    let placed = session.ok("place_numbers", json!({ "project": project, "name": "A" }));
    assert_eq!(placed["placed"], 1);
    assert_eq!(course_numbers(&project)[1], Some((270.0, 1.0)));
    session.ok(
        "set_course_number",
        json!({ "project": project, "name": "A", "after": 2, "clear": true }),
    );
    assert_eq!(course_numbers(&project)[1], None);
    let before = std::fs::read_to_string(&project).unwrap();
    for args in [
        json!({ "project": project, "name": "A", "after": 3, "clear": true }),
        json!({ "project": project, "name": "A", "after": 0, "clear": true }),
        json!({ "project": project, "name": "A", "after": 1, "direction": 0.0, "gap_mm": 0.0 }),
        json!({ "project": project, "name": "A", "after": 1, "direction": 0.0, "gap_mm": 10.5 }),
        json!({ "project": project, "name": "A", "after": 1, "direction": 0.0 }),
        json!({ "project": project, "name": "A", "after": 1, "gap_mm": 1.0, "clear": true }),
    ] {
        let (error, text) = session.call("set_course_number", args.clone());
        assert!(error, "{args}: {text}");
    }
    let (error, text) = session.call(
        "set_control_number",
        json!({ "project": project, "code": "31", "direction": 0.0, "gap_mm": 20.0 }),
    );
    assert!(error && text.contains("10"), "{text}");
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    let set = session.ok(
        "set_control_number",
        json!({ "project": project, "code": "31", "direction": 450.0, "gap_mm": 0.5 }),
    );
    assert_eq!(set["number"]["direction"], 90.0, "{set}");
}

#[test]
fn central_number_is_kept_by_place_numbers() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    course_a(&mut session, &project);
    session.ok(
        "set_control_number",
        json!({ "project": project, "code": "31", "direction": 90.0, "gap_mm": 0.5 }),
    );
    for overwrite in [false, true] {
        let placed = session.ok(
            "place_numbers",
            json!({ "project": project, "name": "A", "overwrite": overwrite }),
        );
        assert_eq!(placed["placed"], 1);
        assert_eq!(course_numbers(&project)[0], None);
    }
}

#[test]
fn changed_legs_are_reported() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = course_a(&mut session, &project);
    let bend = point(&extent, 0.45, 0.45);
    session.ok(
        "set_leg",
        json!({ "project": project, "name": "A", "after": 1, "bends": [bend] }),
    );
    let set = session.ok(
        "set_course_controls",
        json!({ "project": project, "name": "A", "codes": ["31", "31", "32"] }),
    );
    assert_eq!(
        set["cleared_legs"],
        json!([{ "course": "A", "after": 1 }]),
        "{set}"
    );
    let set = session.ok(
        "set_course_controls",
        json!({ "project": project, "name": "A", "codes": ["31", "32"] }),
    );
    assert_eq!(set["cleared_legs"], json!([]), "{set}");
    session.ok(
        "set_leg",
        json!({ "project": project, "name": "A", "after": 1, "bends": [bend], "taped": true }),
    );
    let removed = session.ok(
        "remove_control",
        json!({ "project": project, "code": "32" }),
    );
    assert_eq!(removed["left_courses"], json!(["A"]), "{removed}");
    assert_eq!(
        removed["cleared_legs"],
        json!([{ "course": "A", "after": 1 }]),
        "{removed}"
    );
}

#[test]
fn set_special_courses_assigns_and_widens() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = course_a(&mut session, &project);
    session.ok(
        "add_special",
        json!({ "project": project, "kind": "water", "id": "W1", "at": point(&extent, 0.5, 0.5) }),
    );
    let set = session.ok(
        "set_special_courses",
        json!({ "project": project, "id": "W1", "courses": ["A"] }),
    );
    assert_eq!(set["courses"], json!(["A"]), "{set}");
    let before = std::fs::read_to_string(&project).unwrap();
    let (error, text) = session.call("remove_course", json!({ "project": project, "name": "A" }));
    assert!(error && text.contains("W1"), "{text}");
    for args in [
        json!({ "project": project, "id": "W1" }),
        json!({ "project": project, "id": "W1", "courses": ["A"], "all": true }),
        json!({ "project": project, "id": "W1", "courses": [] }),
        json!({ "project": project, "id": "W1", "courses": ["A", "A"] }),
        json!({ "project": project, "id": "S", "courses": ["A"] }),
    ] {
        let (error, text) = session.call("set_special_courses", args.clone());
        assert!(error, "{args}: {text}");
    }
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    let set = session.ok(
        "set_special_courses",
        json!({ "project": project, "id": "W1", "all": true }),
    );
    assert_eq!(set["courses"], json!([]), "{set}");
    session.ok("remove_course", json!({ "project": project, "name": "A" }));
}

#[test]
fn names_and_lists_are_trimmed_and_codes_may_be_integers() {
    let (_dir, map, project) = workspace();
    let mut session = Session::start();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let p = |fx, fy| point(&extent, fx, fy);
    for (kind, id, at) in [("start", " S", p(0.3, 0.3)), ("finish", "F ", p(0.7, 0.7))] {
        session.ok(
            "add_special",
            json!({ "project": project, "kind": kind, "id": id, "at": at }),
        );
    }
    let added = session.ok(
        "add_control",
        json!({ "project": project, "at": p(0.4, 0.5), "code": 31 }),
    );
    assert_eq!(added["code"], "31", "{added}");
    session.ok(
        "add_control",
        json!({ "project": project, "at": p(0.5, 0.4), "code": " 32 " }),
    );
    for class in ["H10", "D10"] {
        session.ok("add_class", json!({ "project": project, "name": class }));
    }
    session.ok(
        "add_course",
        json!({
            "project": project, "name": " A ", "start": "S ", "finish": " F",
            "controls": [31, " 32"], "classes": [" H10", "D10 "]
        }),
    );
    let file = freemapper_courseplan_model::ProjectFile::load(&project).unwrap();
    let course = &file.event.courses[0];
    assert_eq!(course.name, "A");
    assert_eq!((course.start.as_str(), course.finish.as_str()), ("S", "F"));
    assert_eq!(course.classes, ["H10", "D10"]);
    let set = session.ok(
        "set_course_controls",
        json!({ "project": project, "name": "A", "codes": [32, "31 ", 32] }),
    );
    assert_eq!(set["controls"], json!(["32", "31", "32"]), "{set}");
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "A ", "classes": ["D10 "] }),
    );
    session.ok(
        "set_control_number",
        json!({ "project": project, "code": 31, "direction": 0.0, "gap_mm": 1.0 }),
    );
    session.ok(
        "describe_control",
        json!({ "project": project, "code": 32, "d": "1.1" }),
    );
    session.ok(
        "move_control",
        json!({ "project": project, "code": 32, "at": p(0.55, 0.4) }),
    );
    let removed = session.ok("remove_control", json!({ "project": project, "code": 32 }));
    assert_eq!(removed["left_courses"], json!(["A"]), "{removed}");
    let (error, text) = session.call("remove_control", json!({ "project": project, "code": 3.5 }));
    assert!(error, "{text}");
}

/// The tools added in the final review follow ADR 0023: no file content in their errors.
#[test]
fn new_tools_do_not_quote_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.toml");
    std::fs::write(&broken, "magic = sesame-4711 = oops\n").unwrap();
    let foreign = dir.path().join("settings.toml");
    std::fs::write(&foreign, "sesame_key = 1\n[vault]\nopen = 2\n").unwrap();
    let mut session = Session::start();
    for project in [&broken, &foreign] {
        for (tool, args) in [
            (
                "set_course_number",
                json!({ "project": project, "name": "A", "after": 1, "clear": true }),
            ),
            (
                "set_special_courses",
                json!({ "project": project, "id": "W1", "all": true }),
            ),
        ] {
            let (error, text) = session.call(tool, args);
            assert!(error, "{tool}: {text}");
            for word in ["sesame", "4711", "magic", "vault"] {
                assert!(!text.contains(word), "{tool}: {text}");
            }
        }
    }
}

/// A project with start, finish, controls 31 and 32 and course A. The points lie in an open part
/// of the Lübeck map, so the map-based checks find nothing.
fn checked_workspace(session: &mut Session) -> (tempfile::TempDir, PathBuf) {
    let (dir, map, project) = workspace();
    session.ok("init_event", json!({ "map": map, "project": project }));
    let p = |x: f64, y: f64| json!({ "x": x, "y": y });
    session.ok(
        "add_special",
        json!({ "project": project, "kind": "start", "id": "S1", "at": p(88.0, 82.0) }),
    );
    session.ok(
        "add_special",
        json!({ "project": project, "kind": "finish", "id": "F1", "at": p(118.0, 82.0) }),
    );
    session.ok(
        "add_control",
        json!({ "project": project, "at": p(93.0, 89.0) }),
    );
    session.ok(
        "add_control",
        json!({ "project": project, "at": p(113.0, 89.0) }),
    );
    session.ok(
        "add_course",
        json!({ "project": project, "name": "A", "start": "S1", "finish": "F1", "controls": [31, 32] }),
    );
    (dir, project)
}

#[test]
fn check_returns_findings_accepted_and_skipped() {
    let mut session = Session::start();
    let (_dir, project) = checked_workspace(&mut session);
    let report = session.ok("check", json!({ "project": project }));
    assert_eq!(report["rule_profile"], "wkb-2026");
    assert_eq!(report["findings"], json!([]), "{report}");
    assert_eq!(report["accepted"], json!([]));
    let skipped = report["skipped"].as_array().unwrap();
    assert!(!skipped.is_empty());
    assert_eq!(report["summary"]["skipped"], skipped.len());
    assert_eq!(report["summary"]["errors"], 0);
    for s in skipped {
        assert!(s["rule"].as_str().unwrap().starts_with("wkb2026."), "{s}");
        assert_eq!(s["profile"], "wkb-2026");
        assert!(s["location"]["kind"].is_string(), "{s}");
        assert!(
            s["reason_id"].as_str().unwrap().starts_with("check-skip-"),
            "{s}"
        );
        assert!(s["args"].is_object(), "{s}");
        assert!(s["text"].as_str().unwrap().len() > 8, "{s}");
    }
    let info = session.ok("event_info", json!({ "project": project }));
    assert_eq!(info["rule_profile"], "wkb-2026");
    assert_eq!(info["rule_overlays"], json!([]));
}

#[test]
fn check_speaks_german() {
    let mut session = Session::start_in("de");
    let (_dir, project) = checked_workspace(&mut session);
    let report = session.ok("check", json!({ "project": project }));
    let texts: Vec<&str> = report["skipped"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["text"].as_str().unwrap())
        .collect();
    assert!(
        texts.contains(&"Die Veranstaltung hat kein Startfenster; event start-window setzt es"),
        "{texts:?}"
    );
    let (error, text) = session.call(
        "remove_accepted_finding",
        json!({ "project": project, "rule": "wkb2026.distance.min", "controls": [31] }),
    );
    assert!(error);
    assert!(text.contains("keinen akzeptierten Befund"), "{text}");
}

#[test]
fn accept_and_remove_findings() {
    let mut session = Session::start();
    let (_dir, project) = checked_workspace(&mut session);
    let accepted = session.ok(
        "accept_finding",
        json!({
            "project": project, "rule": " wkb2026.distance.min_half ", "controls": [32, "31"],
            "note": "on purpose"
        }),
    );
    assert_eq!(accepted["rule"], "wkb2026.distance.min_half");
    assert_eq!(accepted["controls"], json!(["31", "32"]));
    assert_eq!(accepted["replaced"], false);
    let again = session.ok(
        "accept_finding",
        json!({
            "project": project, "rule": "wkb2026.distance.min_half", "controls": ["31", "32"],
            "note": "still on purpose"
        }),
    );
    assert_eq!(again["replaced"], true);
    let leg = session.ok(
        "accept_finding",
        json!({
            "project": project, "rule": "wkb2026.leg.through_control",
            "legs": [{ "course": "A", "after": 2 }], "note": "n"
        }),
    );
    // The server stores the ends of the leg from the course (ruling R24).
    assert_eq!(
        leg["legs"],
        json!([{ "course": "A", "after": 2, "from": "32", "to": "F1" }])
    );
    let text = std::fs::read_to_string(&project).unwrap();
    assert_eq!(text.matches("[[accepted_findings]]").count(), 2, "{text}");
    assert!(text.contains("still on purpose"), "{text}");

    let before = std::fs::read_to_string(&project).unwrap();
    for (args, expected) in [
        (
            json!({ "project": project, "rule": "wkb2026.distance.min_half", "note": "n" }),
            "name that place",
        ),
        (
            json!({ "project": project, "rule": "wkb2026.no.such", "controls": [31], "note": "n" }),
            "not an active rule",
        ),
        (
            json!({
                "project": project, "rule": "wkb2026.leg.through_control", "note": "n",
                "legs": [{ "course": "A", "after": 9 }]
            }),
            "position 9",
        ),
    ] {
        let (error, text) = session.call("accept_finding", args);
        assert!(error, "{text}");
        assert!(text.contains(expected), "{text}");
    }
    let message = session.request_full(
        "tools/call",
        json!({ "name": "accept_finding", "arguments": {
            "project": project, "rule": "wkb2026.code.range", "control": [31], "note": "n"
        } }),
    );
    let refused = message.get("error").is_some() || message["result"]["isError"] == true;
    assert!(refused, "{message}");
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);

    session.ok(
        "remove_accepted_finding",
        json!({ "project": project, "rule": "wkb2026.distance.min_half", "controls": [31, 32] }),
    );
    let (error, _) = session.call(
        "remove_accepted_finding",
        json!({ "project": project, "rule": "wkb2026.distance.min_half", "controls": [31, 32] }),
    );
    assert!(error);
}

#[test]
fn set_rule_profile_validates_first() {
    let mut session = Session::start();
    let (_dir, project) = checked_workspace(&mut session);
    let set = session.ok(
        "set_rule_profile",
        json!({ "project": project, "base": "wkb-2026", "overlays": ["nor-bb-2021"] }),
    );
    assert_eq!(set["rule_overlays"], json!(["nor-bb-2021"]));
    let info = session.ok("event_info", json!({ "project": project }));
    assert_eq!(info["rule_overlays"], json!(["nor-bb-2021"]));
    let report = session.ok("check", json!({ "project": project }));
    assert_eq!(report["rule_overlays"], json!(["nor-bb-2021"]));

    let before = std::fs::read_to_string(&project).unwrap();
    let (error, text) = session.call(
        "set_rule_profile",
        json!({ "project": project, "base": "wkb-1999" }),
    );
    assert!(error);
    assert!(
        text.contains("wkb-2026") && text.contains("nor-bb-2021"),
        "{text}"
    );
    let (error, _) = session.call(
        "set_rule_profile",
        json!({ "project": project, "base": "nor-bb-2021" }),
    );
    assert!(error);
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
}

#[test]
fn check_tools_do_not_quote_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("broken.toml");
    std::fs::write(&broken, "magic = sesame-4711 = oops\n").unwrap();
    let foreign = dir.path().join("settings.toml");
    std::fs::write(&foreign, "sesame_key = 1\n[vault]\nopen = 2\n").unwrap();
    let mut session = Session::start();
    for project in [&broken, &foreign] {
        for (tool, args) in [
            ("check", json!({ "project": project })),
            (
                "accept_finding",
                json!({ "project": project, "rule": "r", "controls": [31], "note": "n" }),
            ),
            (
                "remove_accepted_finding",
                json!({ "project": project, "rule": "r", "controls": [31] }),
            ),
            (
                "set_rule_profile",
                json!({ "project": project, "base": "wkb-2026" }),
            ),
        ] {
            let (error, text) = session.call(tool, args);
            assert!(error, "{tool}: {text}");
            for word in ["sesame", "4711", "magic", "vault"] {
                assert!(!text.contains(word), "{tool}: {text}");
            }
        }
    }
    let (error, text) = session.call("check", json!({ "project": dir.path().join("x.txt") }));
    assert!(error);
    assert!(text.contains("not a project file"), "{text}");
}

#[test]
fn an_acceptance_that_matches_nothing_is_returned_as_unmatched() {
    let mut session = Session::start();
    let (_dir, project) = checked_workspace(&mut session);
    session.ok(
        "accept_finding",
        json!({
            "project": project, "rule": "wkb2026.leg.through_control",
            "legs": [{ "course": "A", "after": 1 }], "note": "kept"
        }),
    );
    session.ok(
        "set_course_controls",
        json!({ "project": project, "name": "A", "codes": [32, 31] }),
    );
    let report = session.ok("check", json!({ "project": project }));
    assert_eq!(report["summary"]["unmatched"], 1, "{report}");
    assert_eq!(report["summary"]["errors"], 0);
    let u = &report["unmatched"][0];
    assert_eq!(u["rule"], "wkb2026.leg.through_control");
    assert_eq!(u["note"], "kept");
    assert_eq!(
        u["location"],
        json!({ "kind": "leg", "course": "A", "after": 1, "from": "31", "to": "32" })
    );
    assert_eq!(u["text"], "course A, leg 31–32");
    // Accepting leg A:1 again stores a new entry 32–31 and returns it, not the stale 31–32.
    let again = session.ok(
        "accept_finding",
        json!({
            "project": project, "rule": "wkb2026.leg.through_control",
            "legs": [{ "course": "A", "after": 1 }], "note": "new leg"
        }),
    );
    assert_eq!(again["note"], "new leg");
    assert_eq!(
        again["legs"],
        json!([{ "course": "A", "after": 1, "from": "32", "to": "31" }])
    );
    assert_eq!(again["replaced"], false);
    // Removing A:1 takes the exact entry first, the stale one stays unmatched.
    let exact = session.ok(
        "remove_accepted_finding",
        json!({
            "project": project, "rule": "wkb2026.leg.through_control",
            "legs": [{ "course": "A", "after": 1 }]
        }),
    );
    assert_eq!(exact["removed"].as_array().unwrap().len(), 1);
    assert_eq!(exact["removed"][0]["note"], "new leg");
    // remove_accepted_finding then matches the stale leg by position and returns what it removed.
    let removed = session.ok(
        "remove_accepted_finding",
        json!({
            "project": project, "rule": "wkb2026.leg.through_control",
            "legs": [{ "course": "A", "after": 1 }]
        }),
    );
    assert_eq!(removed["removed"][0]["legs"][0]["to"], "32");
    let report = session.ok("check", json!({ "project": project }));
    assert_eq!(report["unmatched"], json!([]));
}

#[test]
fn start_window_and_class_settings() {
    let mut session = Session::start();
    let (_dir, project) = checked_workspace(&mut session);
    let text = std::fs::read_to_string(&project).unwrap();
    std::fs::write(&project, format!("# planned by H.\n{text}")).unwrap();
    let set = session.ok(
        "set_start_window",
        json!({ "project": project, "minutes": 90 }),
    );
    assert_eq!(set["start_window_min"], 90);
    session.ok("add_class", json!({ "project": project, "name": "H21" }));
    let class = session.ok(
        "set_class",
        json!({
            "project": project, "name": "H21", "rule_category": "H21", "expected_starters": 40
        }),
    );
    assert_eq!(
        (
            &class["name"],
            &class["rule_category"],
            &class["expected_starters"]
        ),
        (&json!("H21"), &json!("H21"), &json!(40))
    );
    let cleared = session.ok(
        "set_class",
        json!({ "project": project, "name": "H21", "no_expected_starters": true }),
    );
    assert_eq!(cleared["rule_category"], "H21");
    assert_eq!(cleared["expected_starters"], Value::Null);
    let file = std::fs::read_to_string(&project).unwrap();
    assert!(file.starts_with("# planned by H.\n"), "{file}");
    assert!(file.contains("start_window_min = 90"), "{file}");

    let before = std::fs::read_to_string(&project).unwrap();
    for (tool, args, expected) in [
        (
            "set_start_window",
            json!({ "project": project, "minutes": 0 }),
            "at least 1 minute",
        ),
        (
            "set_class",
            json!({ "project": project, "name": "H21" }),
            "nothing to change",
        ),
        (
            "set_class",
            json!({ "project": project, "name": "D21", "expected_starters": 3 }),
            "no class D21",
        ),
        (
            "set_class",
            json!({ "project": project, "name": "H21", "rule_category": " " }),
            "is empty",
        ),
        (
            "set_class",
            json!({
                "project": project, "name": "H21", "rule_category": "H21",
                "no_rule_category": true
            }),
            "rule_category cannot be used together with no_rule_category",
        ),
        (
            "set_class",
            json!({
                "project": project, "name": "H21", "expected_starters": 3,
                "no_expected_starters": true
            }),
            "expected_starters cannot be used together with no_expected_starters",
        ),
    ] {
        let (error, text) = session.call(tool, args);
        assert!(error, "{tool}: {text}");
        assert!(text.contains(expected), "{tool}: {text}");
    }
    for (tool, args) in [
        (
            "set_start_window",
            json!({ "project": project, "minutes": 5, "seconds": 3 }),
        ),
        (
            "set_class",
            json!({ "project": project, "name": "H21", "category": "H21" }),
        ),
    ] {
        let message =
            session.request_full("tools/call", json!({ "name": tool, "arguments": args }));
        let refused = message.get("error").is_some() || message["result"]["isError"] == true;
        assert!(refused, "{tool}: {message}");
    }
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    // A project file outside the rules is refused without quoting it.
    let dir = tempfile::tempdir().unwrap();
    let foreign = dir.path().join("settings.toml");
    std::fs::write(&foreign, "sesame_key = 1\n").unwrap();
    for (tool, args) in [
        (
            "set_start_window",
            json!({ "project": foreign, "minutes": 5 }),
        ),
        (
            "set_class",
            json!({ "project": foreign, "name": "H21", "expected_starters": 3 }),
        ),
        (
            "set_start_window",
            json!({ "project": "rel/x.toml", "minutes": 5 }),
        ),
    ] {
        let (error, text) = session.call(tool, args);
        assert!(error, "{tool}: {text}");
        assert!(!text.contains("sesame"), "{tool}: {text}");
    }
}

#[test]
fn export_iof_xml_writes_the_file() {
    let (dir, map, project) = workspace();
    let target = dir.path().join("out.xml");
    let mut session = Session::start();
    session.ok(
        "init_event",
        json!({ "map": map, "project": project, "title": "Test" }),
    );
    let done = session.ok(
        "export_iof_xml",
        json!({ "project": project, "path": target }),
    );
    assert_eq!(done["courses"], 0, "{done}");
    assert_eq!(done["skipped"], json!([]), "{done}");
    assert_eq!(done["geo_positions"], json!(true), "{done}");
    assert_eq!(done["path"], json!(target.display().to_string()), "{done}");
    assert!(
        std::fs::read_to_string(&target)
            .unwrap()
            .contains("<CourseData")
    );
    let (error, text) = session.call(
        "export_iof_xml",
        json!({ "project": project, "path": dir.path().join("out.txt") }),
    );
    assert!(error);
    assert!(text.contains(".xml"), "{text}");
}

#[test]
fn export_ppen_returns_path_courses_skipped_and_notes() {
    let (dir, map, project) = workspace();
    let target = dir.path().join("out.ppen");
    let mut session = Session::start();
    session.ok(
        "init_event",
        json!({ "map": map, "project": project, "title": "Test" }),
    );
    let done = session.ok("export_ppen", json!({ "project": project, "path": target }));
    assert_eq!(done["courses"], 0, "{done}");
    assert_eq!(done["skipped"], json!([]), "{done}");
    assert_eq!(done["notes"], json!([]), "{done}");
    assert_eq!(done["path"], json!(target.display().to_string()), "{done}");
    assert!(
        std::fs::read_to_string(&target)
            .unwrap()
            .contains("<course-scribe-event")
    );
    let again = session.ok(
        "export_ppen",
        json!({ "project": project, "path": target, "new": true }),
    );
    assert_eq!(again["courses"], 0, "{again}");
    let (error, text) = session.call(
        "export_ppen",
        json!({ "project": project, "path": dir.path().join("out.txt") }),
    );
    assert!(error);
    assert!(text.contains(".ppen"), "{text}");
}

#[test]
fn export_ppen_says_whether_it_patched_and_how_many_ids_it_stored() {
    let (dir, map, project) = workspace();
    let target = dir.path().join("out.ppen");
    let mut session = Session::start();
    session.ok(
        "init_event",
        json!({ "map": map, "project": project, "title": "Training" }),
    );
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    let p = |fx, fy| point(&extent, fx, fy);
    for (kind, id, at) in [("start", "S1", p(0.4, 0.4)), ("finish", "F1", p(0.6, 0.6))] {
        session.ok(
            "add_special",
            json!({ "project": project, "kind": kind, "id": id, "at": at }),
        );
    }
    for (code, at) in [("31", p(0.45, 0.5)), ("32", p(0.55, 0.45))] {
        session.ok(
            "add_control",
            json!({ "project": project, "at": at, "code": code }),
        );
    }
    session.ok(
        "add_course",
        json!({
            "project": project, "name": "A", "start": "S1", "finish": "F1",
            "controls": ["31", "32"]
        }),
    );
    // A new file links the project and stores an id for the start, the finish, both controls and the
    // course.
    let done = session.ok("export_ppen", json!({ "project": project, "path": target }));
    assert_eq!(done["courses"], 1, "{done}");
    assert_eq!(done["patched"], false, "{done}");
    assert_eq!(done["ids_stored"], 5, "{done}");
    assert_eq!(done["notes"], json!([]), "{done}");
    let again = session.ok("export_ppen", json!({ "project": project, "path": target }));
    assert_eq!(again["patched"], true, "{again}");
    assert_eq!(again["ids_stored"], 0, "{again}");
    session.ok(
        "add_control",
        json!({ "project": project, "at": p(0.5, 0.55), "code": "33" }),
    );
    let third = session.ok("export_ppen", json!({ "project": project, "path": target }));
    assert_eq!(third["patched"], true, "{third}");
    assert_eq!(third["ids_stored"], 1, "{third}");
}

#[test]
fn import_ppen_takes_over_the_event_and_returns_counts_and_notes() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("park.omap");
    std::fs::write(&map, PARK_MAP).unwrap();
    let ppen = dir.path().join("park.ppen");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/ppen/lincoln-park.ppen"),
        &ppen,
    )
    .unwrap();
    let project = dir.path().join("park.courseplan.toml");
    let mut session = Session::start();
    session.ok(
        "init_event",
        json!({ "map": map, "project": project, "title": "Park" }),
    );
    let done = session.ok("import_ppen", json!({ "project": project, "path": ppen }));
    assert_eq!(done["controls"], 2, "{done}");
    assert_eq!(done["courses"], 1, "{done}");
    let notes = done["notes"].as_array().unwrap();
    assert!(
        notes.iter().any(|n| n
            .as_str()
            .unwrap()
            .contains("course Short has circle or leg gaps")),
        "{done}"
    );
    let before = std::fs::read_to_string(&project).unwrap();
    let (error, text) = session.call("import_ppen", json!({ "project": project, "path": ppen }));
    assert!(error);
    assert!(text.contains("replace"), "{text}");
    assert_eq!(std::fs::read_to_string(&project).unwrap(), before);
    let again = session.ok(
        "import_ppen",
        json!({ "project": project, "path": ppen, "replace": true }),
    );
    assert_eq!(again["courses"], 1, "{again}");
    let (error, text) = session.call(
        "import_ppen",
        json!({ "project": project, "path": dir.path().join("x.txt") }),
    );
    assert!(error);
    assert!(text.contains(".ppen"), "{text}");
}

/// A 1:5000 sprint map around the controls of `lincoln-park.ppen`.
const PARK_MAP: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<map xmlns="http://openorienteering.org/apps/mapper/xml/v2" version="9">
<notes></notes>
<georeferencing scale="5000"></georeferencing>
<colors count="0"></colors>
<barrier version="6" required="0.6.0">
<symbols count="1" id="ISSprOM 2019"><symbol type="1" id="0" code="999" name="corner"><point_symbol/></symbol></symbols>
<parts count="1" current="0"><part name="default part"><objects count="2">
<object type="0" symbol="0"><coords count="1">60000 -130000;</coords></object>
<object type="0" symbol="0"><coords count="1">150000 -30000;</coords></object>
</objects></part></parts>
<templates count="0"></templates>
<view><grid/></view>
</barrier>
</map>
"#;
