//! The `edit` command end to end: the binary is spawned with `--no-browser` and spoken to over raw HTTP.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn command(lang: &str) -> Command {
    let mut command = Command::new(BIN);
    command
        .env_remove("FREEMAPPER_LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_MESSAGES")
        .env("LANG", "C")
        .arg("--lang")
        .arg(lang);
    command
}

/// A temp dir with the example project and its map; `extra` is appended to the project text.
fn example(extra: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::copy(
        repo_root().join("tests/fixtures/maps/luebeck.omap"),
        dir.path().join("luebeck.omap"),
    )
    .unwrap();
    let text = std::fs::read_to_string(
        repo_root().join("examples/luebeck-sprint/luebeck-sprint.courseplan.toml"),
    )
    .unwrap()
    .replace("../../tests/fixtures/maps/luebeck.omap", "luebeck.omap");
    let project = dir.path().join("event.courseplan.toml");
    std::fs::write(&project, format!("{text}\n{extra}")).unwrap();
    (dir, project)
}

/// A running `edit` process and the address it printed.
struct Editor {
    child: Child,
    url: String,
    port: u16,
    path: String,
    _dir: tempfile::TempDir,
}

impl Editor {
    fn start() -> Editor {
        let (dir, project) = example("");
        let mut child = command("en")
            .arg("edit")
            .arg(&project)
            .arg("--no-browser")
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
        let (port, path) = (port.parse().unwrap(), format!("/{path}"));
        Editor {
            child,
            url,
            port,
            path,
            _dir: dir,
        }
    }

    /// One request; the answer's status line, headers and body as text.
    fn request(&self, method: &str, path: &str, headers: &[(&str, &str)]) -> (u16, String, String) {
        self.request_at(&format!("{}{path}", self.path), method, headers)
    }

    /// Like `request`, with the whole request path. A `Host` header in `headers` replaces the default.
    fn request_at(
        &self,
        path: &str,
        method: &str,
        headers: &[(&str, &str)],
    ) -> (u16, String, String) {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        let mut head = format!("{method} {path} HTTP/1.1\r\nConnection: close\r\n");
        if !headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("host"))
        {
            head.push_str(&format!("Host: 127.0.0.1:{}\r\n", self.port));
        }
        for (name, value) in headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        if method == "POST" {
            head.push_str("Content-Length: 2\r\n");
        }
        head.push_str("\r\n");
        if method == "POST" {
            head.push_str("{}");
        }
        stream.write_all(head.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        let (head, body) = answer.split_once("\r\n\r\n").unwrap();
        let status = head.split(' ').nth(1).unwrap().parse().unwrap();
        (status, head.to_owned(), body.to_owned())
    }

    fn wait_exit(&mut self) -> std::process::ExitStatus {
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                return status;
            }
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "the editor did not stop"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn edit_prints_one_url_line_and_serves_the_page() {
    let editor = Editor::start();
    let ok = editor.url.len()
        == "http://127.0.0.1:".len() + editor.port.to_string().len() + 1 + 32 + 1
        && editor.url.ends_with('/')
        && editor
            .path
            .trim_matches('/')
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
    assert!(ok, "{}", editor.url);
    let (status, head, body) = editor.request("GET", "", &[]);
    assert_eq!(status, 200);
    assert!(
        head.to_ascii_lowercase()
            .contains("content-type: text/html"),
        "{head}"
    );
    assert!(body.contains(r#"<script type="module" src="assets/js/main.js">"#));
}

#[test]
fn edit_refuses_a_broken_project_with_exit_1() {
    let (dir, project) = example("");
    let text =
        std::fs::read_to_string(&project)
            .unwrap()
            .replacen("format = 1", "format = \"x\"", 1);
    std::fs::write(&project, text).unwrap();
    let out = command("en")
        .arg("edit")
        .arg(&project)
        .arg("--no-browser")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(out.stdout.is_empty());
    assert!(
        stderr(&out).contains("event.courseplan.toml"),
        "{}",
        stderr(&out)
    );
    drop(dir);
}

#[test]
fn quit_ends_the_process_with_0() {
    let mut editor = Editor::start();
    let (status, _, _) = editor.request(
        "POST",
        "api/quit",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", editor.port)),
            ("Content-Type", "application/json"),
        ],
    );
    assert_eq!(status, 200);
    assert_eq!(editor.wait_exit().code(), Some(0));
}

#[test]
fn editor_routes_inherit_the_guard() {
    let mut editor = Editor::start();
    let own = format!("http://127.0.0.1:{}", editor.port);
    let json = ("Content-Type", "application/json");
    let gets = [
        "api/state",
        "api/overlay",
        "api/map.svg?x0=0&y0=0&x1=9&y1=9",
        "api/revision",
    ];
    for route in gets {
        let (status, _, _) = editor.request("GET", route, &[("Origin", "http://evil.example")]);
        assert_eq!(status, 403, "{route}: foreign origin");
        let (status, _, _) = editor.request("GET", route, &[("Host", "evil.example")]);
        assert_eq!(status, 403, "{route}: foreign host");
    }
    let (status, _, _) = editor.request_at(&format!("/{}/api/state", "0".repeat(32)), "GET", &[]);
    assert_eq!(status, 404, "wrong token");
    let posts: [&[(&str, &str)]; 4] = [
        &[json],
        &[("Origin", "http://evil.example"), json],
        &[
            ("Origin", &own),
            ("Content-Type", "application/x-www-form-urlencoded"),
        ],
        &[("Origin", &own), ("Content-Type", "text/plain")],
    ];
    for headers in posts {
        let (status, _, _) = editor.request("POST", "api/quit", headers);
        assert!(
            (400..500).contains(&status) && status != 404,
            "{headers:?}: {status}"
        );
    }
    std::thread::sleep(Duration::from_millis(200));
    assert!(
        editor.child.try_wait().unwrap().is_none(),
        "the editor stopped"
    );
    let (status, _, _) = editor.request("GET", "api/texts", &[]);
    assert_eq!(status, 200);
}

/// Half a request on an open socket must not keep the editor alive (review I1).
fn stalled_socket(editor: &Editor) -> TcpStream {
    let mut stalled = TcpStream::connect(("127.0.0.1", editor.port)).unwrap();
    write!(
        stalled,
        "GET {}api/state HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n",
        editor.path, editor.port
    )
    .unwrap();
    stalled.flush().unwrap();
    std::thread::sleep(Duration::from_millis(100));
    stalled
}

#[test]
fn quit_ends_the_process_with_0_despite_a_stalled_connection() {
    let mut editor = Editor::start();
    let _stalled = stalled_socket(&editor);
    let (status, _, _) = editor.request(
        "POST",
        "api/quit",
        &[
            ("Origin", &format!("http://127.0.0.1:{}", editor.port)),
            ("Content-Type", "application/json"),
        ],
    );
    assert_eq!(status, 200);
    assert_eq!(editor.wait_exit().code(), Some(0));
}

#[cfg(unix)]
#[test]
fn ctrl_c_ends_the_process_with_0_despite_a_stalled_connection() {
    let mut editor = Editor::start();
    let _stalled = stalled_socket(&editor);
    let status = Command::new("kill")
        .args(["-INT", &editor.child.id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(editor.wait_exit().code(), Some(0));
}

#[test]
fn port_in_use_is_a_clear_error() {
    let (_dir, project) = example("");
    let held = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = held.local_addr().unwrap().port();
    let out = command("en")
        .arg("edit")
        .arg(&project)
        .arg("--no-browser")
        .arg("--port")
        .arg(port.to_string())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(out.stdout.is_empty());
    assert!(stderr(&out).contains(&port.to_string()), "{}", stderr(&out));
}
