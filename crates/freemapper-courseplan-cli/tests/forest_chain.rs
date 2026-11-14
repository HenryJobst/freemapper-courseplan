//! A course set by hand on a forest map, from `init` to the exports and the editor (ADR 0092). The map is the
//! synthetic ISOM 2017-2 map of `tests/fixtures/maps`, 1:10000.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn command() -> Command {
    let mut command = Command::new(BIN);
    command
        .env_remove("FREEMAPPER_LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C")
        .arg("--lang")
        .arg("en");
    command
}

fn run(args: &[&str]) -> Output {
    command().args(args).output().unwrap()
}

fn ok(args: &[&str]) -> String {
    let out = run(args);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{args:?}: {}{text}",
        String::from_utf8_lossy(&out.stderr)
    );
    text
}

fn s(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn forest_course_by_hand_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("forest.omap");
    std::fs::copy(
        repo_root().join("tests/fixtures/maps/isom2017-synthetic.omap"),
        &map,
    )
    .unwrap();
    let project = dir.path().join("forest.courseplan.toml");
    let (p, m) = (s(&project), s(&map));

    ok(&["init", m]);
    let info = ok(&["info", p]);
    assert!(info.contains("map kind: forest"), "{info}");
    ok(&[
        "special", "add", p, "--kind", "start", "--id", "S1", "--at", "1,7",
    ]);
    ok(&[
        "special", "add", p, "--kind", "finish", "--id", "F1", "--at", "11,7",
    ]);
    for at in ["4,9.5", "8,9.5", "12,2"] {
        ok(&["control", "add", p, "--at", at]);
    }
    ok(&[
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
    ]);
    ok(&["course", "numbers", p, "A"]);

    // A clean course: no error.
    let check = run(&["check", p]);
    assert_eq!(
        check.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&check.stdout)
    );

    // IOF XML, validated against the schema when xmllint is there.
    let xml = dir.path().join("out.xml");
    ok(&["export", p, "--iof-xml", s(&xml)]);
    let schema = repo_root().join("tests/fixtures/iof-xml/IOF.xsd");
    match Command::new("xmllint")
        .args(["--noout", "--schema", s(&schema), s(&xml)])
        .output()
    {
        Ok(out) => assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        ),
        Err(_) => eprintln!("note: xmllint not on PATH, schema validation skipped"),
    }

    // Purple Pen: the ISOM 2017 standard and the scale of the map.
    let ppen = dir.path().join("out.ppen");
    ok(&["export", p, "--ppen", s(&ppen)]);
    let text = std::fs::read_to_string(&ppen).unwrap();
    assert!(text.contains(r#"<standards map="2017""#), "{text}");
    assert!(text.contains(r#"scale="10000""#), "{text}");
    assert!(text.contains(r#"print-scale="10000""#), "{text}");

    // A second project takes the event over from the Purple Pen file, and the sync of the first sees no change.
    let other = dir.path().join("other.courseplan.toml");
    ok(&["init", m, "--project", s(&other), "--ppen", s(&ppen)]);
    let info = ok(&["info", s(&other)]);
    assert!(info.contains("map kind: forest"), "{info}");
    assert!(info.contains("course A: 3 controls"), "{info}");
    let dry = ok(&["sync", p, "--dry-run"]);
    assert!(
        dry.contains("agree") || dry.contains("Nothing to do"),
        "{dry}"
    );
}

#[test]
fn the_editor_serves_a_forest_project() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("forest.omap");
    std::fs::copy(
        repo_root().join("tests/fixtures/maps/isom2017-synthetic.omap"),
        &map,
    )
    .unwrap();
    let project = dir.path().join("forest.courseplan.toml");
    ok(&["init", s(&map)]);
    ok(&[
        "special",
        "add",
        s(&project),
        "--kind",
        "start",
        "--id",
        "S1",
        "--at",
        "1,7",
    ]);
    let mut child = command()
        .args(["edit", s(&project), "--no-browser"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    let url = line.trim().to_owned();
    let rest = url.strip_prefix("http://127.0.0.1:").expect(&url);
    let (port, path) = rest.split_once('/').expect(&url);
    let mut stream = TcpStream::connect(("127.0.0.1", port.parse::<u16>().unwrap())).unwrap();
    write!(
        stream,
        "GET /{}/api/state HTTP/1.1\r\nConnection: close\r\nHost: 127.0.0.1:{port}\r\n\r\n",
        path.trim_end_matches('/')
    )
    .unwrap();
    let mut answer = String::new();
    let _ = stream.read_to_string(&mut answer);
    let _ = child.kill();
    let _ = child.wait();
    let body = answer.split_once("\r\n\r\n").map_or("", |(_, b)| b);
    let state: serde_json::Value =
        serde_json::from_str(body).unwrap_or_else(|_| panic!("{answer}"));
    assert_eq!(state["map"]["kind"], "forest", "{state}");
    assert_eq!(state["map"]["scale"], 10000, "{state}");
    assert!(state["map"]["extent"].is_array(), "{state}");
}
