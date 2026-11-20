// SPDX-License-Identifier: AGPL-3.0-or-later

//! The refusals and the less used shapes of the MCP tools, spoken to over stdio as an agent
//! would: wrong arguments, paths the server must not open, and every kind of special object.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Session {
    fn start() -> Self {
        Self::spawn("en", &[])
    }

    /// A server in `lang`, with the extra environment variables.
    fn spawn(lang: &str, env: &[(&str, &Path)]) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_freemapper-courseplan"));
        command
            .arg("mcp")
            .env("FREEMAPPER_LANG", lang)
            .env_remove("LC_ALL")
            .env_remove("LC_MESSAGES")
            .env("LANG", "C")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for (name, value) in env {
            command.env(name, value);
        }
        let mut child = command.spawn().unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut session = Self {
            child,
            stdin: Some(stdin),
            stdout,
            next_id: 1,
        };
        session.request(
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
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
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

    fn ok(&mut self, tool: &str, arguments: Value) -> Value {
        let (error, text) = self.call(tool, arguments);
        assert!(!error, "{tool}: {text}");
        let value: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["ok"], true, "{tool}: {text}");
        value
    }

    /// Calls a tool that must refuse; returns the text of the refusal.
    fn refused(&mut self, tool: &str, arguments: Value) -> String {
        let (error, text) = self.call(tool, arguments);
        assert!(error, "{tool} should refuse, answered: {text}");
        text
    }
}

impl Drop for Session {
    /// Closes the input so that the server exits normally and writes its coverage data.
    fn drop(&mut self) {
        drop(self.stdin.take());
        for _ in 0..100 {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A temp dir with the Luebeck map, the project path, and a project created on it.
fn event(session: &mut Session) -> (tempfile::TempDir, PathBuf, Value) {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("luebeck.omap");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/luebeck.omap"),
        &map,
    )
    .unwrap();
    let project = dir.path().join("luebeck.courseplan.toml");
    session.ok("init_event", json!({ "map": map, "project": project }));
    let extent = session.ok("map_extent", json!({ "project": project }))["extent"].clone();
    (dir, project, extent)
}

fn point(extent: &Value, fx: f64, fy: f64) -> Value {
    let at = |axis: &str, f: f64| {
        let (lo, hi) = (
            extent["min"][axis].as_f64().unwrap(),
            extent["max"][axis].as_f64().unwrap(),
        );
        lo + (hi - lo) * f
    };
    json!({ "x": at("x", fx), "y": at("y", fy) })
}

/// Start S, finish F, controls 31 and 32, class H10 and course A over both.
fn course_a(session: &mut Session, project: &Path, extent: &Value) {
    let p = |fx, fy| point(extent, fx, fy);
    for (kind, id, at) in [("start", "S", p(0.3, 0.3)), ("finish", "F", p(0.7, 0.7))] {
        session.ok(
            "add_special",
            json!({ "project": project, "kind": kind, "id": id, "at": at }),
        );
    }
    for (code, at) in [("31", p(0.4, 0.5)), ("32", p(0.5, 0.4))] {
        session.ok(
            "add_control",
            json!({ "project": project, "code": code, "at": at }),
        );
    }
    session.ok("add_class", json!({ "project": project, "name": "H10" }));
    session.ok(
        "add_course",
        json!({
            "project": project, "name": "A", "start": "S", "finish": "F",
            "controls": ["31", "32"], "classes": ["H10"]
        }),
    );
}

#[test]
fn every_kind_of_special_object_is_listed_with_its_geometry() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    course_a(&mut session, &project, &extent);
    let ring = |fx: f64| {
        json!([
            point(&extent, fx, 0.1),
            point(&extent, fx + 0.05, 0.1),
            point(&extent, fx, 0.2)
        ])
    };
    for (kind, id, geometry) in [
        ("first_aid", "FA", json!({ "at": point(&extent, 0.8, 0.8) })),
        ("water", "WA", json!({ "at": point(&extent, 0.2, 0.8) })),
        ("marked_route", "MR", json!({ "points": ring(0.1) })),
        ("forbidden_route", "FR", json!({ "points": ring(0.3) })),
        ("out_of_bounds", "OB", json!({ "points": ring(0.5) })),
        ("dangerous_area", "DA", json!({ "points": ring(0.7) })),
    ] {
        let mut args = json!({ "project": project, "kind": kind, "id": id, "courses": ["A"] });
        for (key, value) in geometry.as_object().unwrap() {
            args[key] = value.clone();
        }
        assert_eq!(session.ok("add_special", args)["id"], id);
    }
    let listed = session.ok("list_specials", json!({ "project": project }));
    let specials = listed["specials"].as_array().unwrap();
    let find = |id: &str| specials.iter().find(|s| s["id"] == id).expect(id);
    for (id, kind) in [
        ("S", "start"),
        ("F", "finish"),
        ("FA", "first_aid"),
        ("WA", "water"),
        ("MR", "marked_route"),
        ("FR", "forbidden_route"),
        ("OB", "out_of_bounds"),
        ("DA", "dangerous_area"),
    ] {
        assert_eq!(find(id)["kind"], kind, "{listed}");
    }
    // A point object has `at`, a drawn one has `points`, never both.
    for id in ["S", "F", "FA", "WA"] {
        assert!(find(id)["at"]["x"].is_f64(), "{id}: {listed}");
        assert!(find(id).get("points").is_none(), "{id}: {listed}");
    }
    for id in ["MR", "FR", "OB", "DA"] {
        assert_eq!(find(id)["points"].as_array().unwrap().len(), 3, "{id}");
        assert!(find(id).get("at").is_none(), "{id}: {listed}");
    }
    // The courses a special applies to are listed for the kinds that have them.
    assert_eq!(find("WA")["courses"], json!(["A"]));
    assert_eq!(find("OB")["courses"], json!(["A"]));
    // Each can be removed again, and only that one.
    session.ok("remove_special", json!({ "project": project, "id": "OB" }));
    let after = session.ok("list_specials", json!({ "project": project }));
    assert_eq!(
        after["specials"].as_array().unwrap().len(),
        specials.len() - 1
    );
    assert!(
        after["specials"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["id"] != "OB")
    );
}

#[test]
fn a_special_object_with_the_wrong_geometry_is_refused_with_its_kind() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    let at = point(&extent, 0.5, 0.5);
    let points = json!([at, point(&extent, 0.6, 0.5), point(&extent, 0.6, 0.6)]);
    // A single-point kind needs `at`, and no `points`.
    for args in [
        json!({ "kind": "water", "id": "W" }),
        json!({ "kind": "water", "id": "W", "at": at, "points": points }),
    ] {
        let mut args = args;
        args["project"] = json!(project);
        let text = session.refused("add_special", args);
        assert!(
            text.contains("single point") && text.contains("water point"),
            "{text}"
        );
    }
    // A drawn kind needs `points`, and no `at`.
    for args in [
        json!({ "kind": "forbidden_route", "id": "R", "at": at }),
        json!({ "kind": "forbidden_route", "id": "R" }),
    ] {
        let mut args = args;
        args["project"] = json!(project);
        let text = session.refused("add_special", args);
        assert!(
            text.contains("drawn through points") && text.contains("forbidden route"),
            "{text}"
        );
    }
    // Start and finish apply to every course.
    let text = session.refused(
        "add_special",
        json!({ "project": project, "kind": "start", "id": "S", "at": at, "courses": ["A"] }),
    );
    assert!(
        text.contains("every course") && text.contains("start"),
        "{text}"
    );
    // Nothing of that reached the file.
    let listed = session.ok("list_specials", json!({ "project": project }));
    assert_eq!(listed["specials"], json!([]));
    // An id that is taken is refused, not overwritten.
    session.ok(
        "add_special",
        json!({ "project": project, "kind": "start", "id": "S", "at": at }),
    );
    session.refused(
        "add_special",
        json!({ "project": project, "kind": "finish", "id": "S", "at": at }),
    );
}

#[test]
fn the_kind_words_of_the_refusals_are_german_in_a_german_server() {
    let mut session = Session::spawn("de", &[]);
    let (_dir, project, extent) = event(&mut session);
    let text = session.refused(
        "add_special",
        json!({ "project": project, "kind": "water", "id": "W" }),
    );
    assert!(text.contains("einzelner Punkt"), "{text}");
    assert!(!text.contains("water point"), "{text}");
    let text = session.refused(
        "add_special",
        json!({ "project": project, "kind": "start", "id": "S",
                "at": point(&extent, 0.5, 0.5), "courses": ["A"] }),
    );
    assert!(text.contains("jede Bahn"), "{text}");
}

/// Only where the home folder comes from `HOME`: on Windows it comes from the system, and the
/// test cannot move it.
#[cfg(unix)]
#[test]
fn paths_with_a_tilde_are_resolved_in_the_home_folder() {
    let home = tempfile::tempdir().unwrap();
    let mut session = Session::spawn("en", &[("HOME", home.path())]);
    let (dir, project, _) = event(&mut session);
    std::fs::copy(&project, home.path().join("moved.courseplan.toml")).unwrap();
    std::fs::copy(
        dir.path().join("luebeck.omap"),
        home.path().join("luebeck.omap"),
    )
    .unwrap();
    let info = session.ok(
        "event_info",
        json!({ "project": "~/moved.courseplan.toml" }),
    );
    assert_eq!(info["counts"]["controls"], 0, "{info}");
    // The same file under a backslash tilde, as a Windows assistant writes it.
    let info = session.ok(
        "event_info",
        json!({ "project": "~\\moved.courseplan.toml" }),
    );
    assert_eq!(info["counts"]["courses"], 0, "{info}");
    // A tilde with a user name is no home path: it is a relative path.
    let text = session.refused(
        "event_info",
        json!({ "project": "~other/x.courseplan.toml" }),
    );
    assert!(text.contains("relative path"), "{text}");
}

#[test]
fn tools_that_open_or_write_files_refuse_the_wrong_kind_of_path() {
    let mut session = Session::start();
    let (dir, project, _) = event(&mut session);
    let root = dir.path();
    let wrong = |name: &str| root.join(name).display().to_string();
    let cases = [
        (
            "export_iof_xml",
            json!({ "project": project, "path": wrong("courses.txt") }),
            "not an XML file",
        ),
        (
            "export_ppen",
            json!({ "project": project, "path": wrong("courses.xml") }),
            "not a Purple Pen event",
        ),
        (
            "import_ppen",
            json!({ "project": project, "path": wrong("courses.xml") }),
            "not a Purple Pen event",
        ),
        (
            "init_event",
            json!({ "map": wrong("map.png"), "project": wrong("new.courseplan.toml") }),
            "not an OOM map",
        ),
        (
            "event_info",
            json!({ "project": wrong("luebeck.omap") }),
            ".toml",
        ),
    ];
    for (tool, args, expected) in cases {
        let text = session.refused(tool, args);
        assert!(text.contains(expected), "{tool}: {text}");
    }
    // A relative path is refused for the same tools before anything is read.
    for (tool, args) in [
        (
            "export_iof_xml",
            json!({ "project": project, "path": "out.xml" }),
        ),
        (
            "export_ppen",
            json!({ "project": project, "path": "out.ppen" }),
        ),
        (
            "import_ppen",
            json!({ "project": project, "path": "in.ppen" }),
        ),
        (
            "init_event",
            json!({ "map": "map.omap", "project": project }),
        ),
    ] {
        let text = session.refused(tool, args);
        assert!(text.contains("relative path"), "{tool}: {text}");
    }
    assert!(!root.join("courses.txt").exists() && !root.join("courses.xml").exists());
}

#[test]
fn a_project_that_is_not_there_is_refused_with_the_reason() {
    let mut session = Session::start();
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone.courseplan.toml");
    for tool in ["event_info", "list_controls", "check", "map_extent"] {
        let text = session.refused(tool, json!({ "project": gone }));
        assert!(text.contains("gone.courseplan.toml"), "{tool}: {text}");
    }
    let text = session.refused("add_class", json!({ "project": gone, "name": "H10" }));
    assert!(text.contains("gone.courseplan.toml"), "{text}");
    // The tool does not create the file as a side effect.
    assert!(!gone.exists());
}

#[test]
fn set_leg_needs_exactly_one_target_and_a_leg_that_exists() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    course_a(&mut session, &project, &extent);
    let bend = point(&extent, 0.45, 0.45);
    for args in [
        json!({ "project": project, "name": "A" }),
        json!({ "project": project, "name": "A", "from_start": true, "after": 1 }),
    ] {
        let text = session.refused("set_leg", args);
        assert!(text.contains("from_start or after"), "{text}");
    }
    // The assistant counts legs from 1: zero and a leg past the end do not exist.
    for after in [0, 3, 40] {
        let text = session.refused(
            "set_leg",
            json!({ "project": project, "name": "A", "after": after, "bends": [bend] }),
        );
        assert!(
            text.contains("no control at position"),
            "after {after}: {text}"
        );
    }
    let text = session.refused(
        "set_leg",
        json!({ "project": project, "name": "NoSuchCourse", "after": 1, "bends": [bend] }),
    );
    assert!(text.contains("NoSuchCourse"), "{text}");
    // The first leg, from the start, takes bends and the marked-route flag.
    let set = session.ok(
        "set_leg",
        json!({ "project": project, "name": "A", "from_start": true, "bends": [bend], "taped": true }),
    );
    assert_eq!(
        (set["bends"].clone(), set["taped"].clone()),
        (json!(1), json!(true))
    );
    // The last leg is the one after the last control, and clearing it keeps the flag off.
    let set = session.ok(
        "set_leg",
        json!({ "project": project, "name": "A", "after": 2 }),
    );
    assert_eq!(
        (set["bends"].clone(), set["taped"].clone()),
        (json!(0), json!(false))
    );
}

#[test]
fn course_details_can_be_set_and_taken_back() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    course_a(&mut session, &project, &extent);
    let climb_of = |session: &mut Session| {
        let info = session.ok("list_courses", json!({ "project": project }));
        (
            info["courses"][0]["climb_m"].clone(),
            info["courses"][0]["print_scale"].clone(),
        )
    };
    assert_eq!(climb_of(&mut session), (Value::Null, Value::Null));
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "A", "climb_m": 35, "print_scale": 7500 }),
    );
    assert_eq!(climb_of(&mut session), (json!(35), json!(7500)));
    // A value wins over the flag that takes it back; the flag alone takes it back.
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "A", "climb_m": 40, "no_climb": true }),
    );
    assert_eq!(climb_of(&mut session), (json!(40), json!(7500)));
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "A", "no_climb": true, "no_print_scale": true }),
    );
    assert_eq!(climb_of(&mut session), (Value::Null, Value::Null));
    // Without any change the call is refused and names the possible ones.
    let text = session.refused(
        "set_course_details",
        json!({ "project": project, "name": "A" }),
    );
    assert!(
        text.contains("nothing to change") && text.contains("no_print_scale"),
        "{text}"
    );
    let text = session.refused(
        "set_course_details",
        json!({ "project": project, "name": "A", "no_climb": false, "no_print_scale": false }),
    );
    assert!(text.contains("nothing to change"), "{text}");
    // Classes are free names, as the design says: the list is replaced as given, and an empty
    // list takes them all away.
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "A", "classes": ["D99", "H10"] }),
    );
    let info = session.ok("list_courses", json!({ "project": project }));
    assert_eq!(info["courses"][0]["classes"], json!(["D99", "H10"]));
    session.ok(
        "set_course_details",
        json!({ "project": project, "name": "A", "classes": [] }),
    );
    let info = session.ok("list_courses", json!({ "project": project }));
    assert_eq!(info["courses"][0]["classes"], json!([]));
}

#[test]
fn number_placements_need_both_values_in_range() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    course_a(&mut session, &project, &extent);
    for (args, expected) in [
        (
            json!({ "direction": 90.0 }),
            "direction and gap_mm together",
        ),
        (json!({ "gap_mm": 1.0 }), "direction and gap_mm together"),
        (json!({}), "direction and gap_mm together"),
        (
            json!({ "direction": 90.0, "gap_mm": 1.0, "clear": true }),
            "direction and gap_mm together",
        ),
        (json!({ "direction": 90.0, "gap_mm": 0.0 }), "out of range"),
        (json!({ "direction": 90.0, "gap_mm": -1.0 }), "out of range"),
        (
            json!({ "direction": 90.0, "gap_mm": 5000.0 }),
            "out of range",
        ),
    ] {
        let mut full = json!({ "project": project, "code": "31" });
        for (k, v) in args.as_object().unwrap() {
            full[k] = v.clone();
        }
        let text = session.refused("set_control_number", full);
        assert!(text.contains(expected), "{args}: {text}");
    }
    // A direction is brought into 0 up to 360.
    let set = session.ok(
        "set_control_number",
        json!({ "project": project, "code": "31", "direction": 450.0, "gap_mm": 1.5 }),
    );
    assert_eq!(set["number"]["direction"], 90.0, "{set}");
    assert_eq!(set["number"]["gap_mm"], 1.5, "{set}");
    let set = session.ok(
        "set_control_number",
        json!({ "project": project, "code": "31", "direction": -90.0, "gap_mm": 1.5 }),
    );
    assert_eq!(set["number"]["direction"], 270.0, "{set}");
    let set = session.ok(
        "set_control_number",
        json!({ "project": project, "code": "31", "clear": true }),
    );
    assert_eq!(set["number"], Value::Null, "{set}");
    // The refusals changed nothing: the control still has no placement.
    let controls = session.ok("list_controls", json!({ "project": project }));
    assert_eq!(controls["controls"][0]["number"], Value::Null);
}

#[test]
fn controls_list_their_kind_and_the_description_columns_merge() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    session.ok(
        "add_control",
        json!({ "project": project, "code": "31", "at": point(&extent, 0.4, 0.4) }),
    );
    session.ok(
        "add_control",
        json!({ "project": project, "code": "X1", "at": point(&extent, 0.5, 0.5),
                "crossing_point": true }),
    );
    let listed = session.ok("list_controls", json!({ "project": project }));
    let kind_of = |code: &str| {
        listed["controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["code"] == code)
            .unwrap()["kind"]
            .clone()
    };
    assert_eq!(kind_of("31"), "control");
    assert_eq!(kind_of("X1"), "crossing_point");
    // Columns that are given replace, the others stay; `clear` starts from empty.
    session.ok(
        "describe_control",
        json!({ "project": project, "code": "31", "d": "1.1", "e": "2.1" }),
    );
    let merged = session.ok(
        "describe_control",
        json!({ "project": project, "code": "31", "e": "2.2" }),
    );
    assert_eq!(merged["description"]["d"], "1.1", "{merged}");
    assert_eq!(merged["description"]["e"], "2.2", "{merged}");
    let cleared = session.ok(
        "describe_control",
        json!({ "project": project, "code": "31", "clear": true, "d": "1.2" }),
    );
    assert_eq!(cleared["description"]["d"], "1.2", "{cleared}");
    assert_eq!(cleared["description"]["e"], Value::Null, "{cleared}");
    let text = session.refused(
        "describe_control",
        json!({ "project": project, "code": "99", "d": "1.1" }),
    );
    assert!(text.contains("99"), "{text}");
}

#[test]
fn a_control_code_that_is_taken_or_a_course_name_that_is_taken_is_refused() {
    let mut session = Session::start();
    let (_dir, project, extent) = event(&mut session);
    course_a(&mut session, &project, &extent);
    session.ok(
        "add_course",
        json!({ "project": project, "name": "B", "start": "S", "finish": "F", "controls": ["32"] }),
    );
    let text = session.refused(
        "set_control_code",
        json!({ "project": project, "from": "31", "to": "32" }),
    );
    assert!(text.contains("32"), "{text}");
    let text = session.refused(
        "rename_course",
        json!({ "project": project, "from": "A", "to": "B" }),
    );
    assert!(text.contains('B'), "{text}");
    session.refused(
        "rename_course",
        json!({ "project": project, "from": "Z", "to": "Y" }),
    );
    // Both refusals left the names as they were.
    let courses = session.ok("list_courses", json!({ "project": project }));
    let names: Vec<&str> = courses["courses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["A", "B"]);
    let controls = session.ok("list_controls", json!({ "project": project }));
    let codes: Vec<&str> = controls["controls"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["31", "32"]);
}
