//! The update tools and the note of the MCP server (ADR 0076). A build from source neither checks nor
//! installs; the stub tests need `--features test-stub` (see `tests/update.rs`).

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

#[cfg(all(feature = "test-stub", debug_assertions))]
#[path = "support/update_stub.rs"]
mod support;

/// A server over stdio; `env` sets or removes (`None`) variables on top of English texts.
struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Session {
    fn start(program: &Path, env: &[(&str, Option<&str>)]) -> Self {
        let mut command = Command::new(program);
        command
            .arg("mcp")
            .env("FREEMAPPER_LANG", "en")
            .env_remove("CI")
            .env_remove("FREEMAPPER_COURSEPLAN_UPDATE")
            .env_remove("FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE")
            .env_remove("FREEMAPPER_COURSEPLAN_CONFIG_DIR");
        for (name, value) in env {
            match value {
                Some(value) => command.env(name, value),
                None => command.env_remove(name),
            };
        }
        let mut child = command
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

    /// Calls a tool; returns (is_error, the text of every content block).
    fn call_all(&mut self, tool: &str, arguments: Value) -> (bool, Vec<String>) {
        let result = self.request(
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        );
        let texts = result["content"]
            .as_array()
            .unwrap()
            .iter()
            .map(|block| block["text"].as_str().unwrap_or_default().to_string())
            .collect();
        (result["isError"] == true, texts)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

/// A call that fails at once without the network: a project that does not exist.
#[cfg(all(feature = "test-stub", debug_assertions))]
fn failing_call() -> Value {
    json!({ "project": "/nonexistent/x.courseplan.toml" })
}

#[test]
fn the_update_tools_in_a_source_build_say_how_to_update() {
    let mut session = Session::start(Path::new(BIN), &[]);
    let said = "This freemapper-courseplan was built from source. Update it with git pull and cargo build --release --locked.";
    for (tool, arguments) in [
        ("check_update", json!({})),
        ("install_update", json!({ "confirm": true })),
        ("install_update", json!({})),
    ] {
        let (error, texts) = session.call_all(tool, arguments);
        assert!(!error, "{tool}");
        assert_eq!(texts, [said]);
    }
}

#[test]
fn the_update_tools_are_described_in_both_languages() {
    for (lang, expected) in [
        ("en", "Whether a newer freemapper-courseplan"),
        ("de", "Ob es eine neuere freemapper-courseplan"),
    ] {
        let mut session = Session::start(Path::new(BIN), &[("FREEMAPPER_LANG", Some(lang))]);
        let tools = session.request("tools/list", json!({}));
        let tool = tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"] == "check_update")
            .unwrap();
        assert!(
            tool["description"].as_str().unwrap().starts_with(expected),
            "{tool}"
        );
        let install = tools["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"] == "install_update")
            .unwrap();
        assert!(
            install["inputSchema"]["properties"]["confirm"]["description"]
                .as_str()
                .unwrap()
                .len()
                > 8
        );
    }
}

#[cfg(all(feature = "test-stub", debug_assertions))]
mod with_stub {
    use super::support::UpdateStub;
    use super::*;

    const NOTES: &str = "https://github.com/HenryJobst/freemapper-courseplan/releases/tag/v99.0.0";

    /// A server talking to `stub`, with its settings and record in `dir`.
    fn serving(stub: &UpdateStub, dir: &Path) -> Session {
        Session::start(
            Path::new(BIN),
            &[
                (
                    "FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE",
                    Some(stub.base.as_str()),
                ),
                (
                    "FREEMAPPER_COURSEPLAN_CONFIG_DIR",
                    Some(dir.join("config").to_str().unwrap()),
                ),
            ],
        )
    }

    fn note() -> String {
        format!(
            "Note for the person: freemapper-courseplan 99.0.0 is available (this is {}). Ask them before calling install_update.",
            env!("CARGO_PKG_VERSION")
        )
    }

    #[test]
    fn check_update_reports_the_release_and_installs_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let mut session = serving(&stub, dir.path());
        let (error, texts) = session.call_all("check_update", json!({}));
        assert!(!error, "{texts:?}");
        assert_eq!(texts.len(), 1, "{texts:?}");
        assert!(
            texts[0].contains("freemapper-courseplan 99.0.0 is available"),
            "{texts:?}"
        );
        assert!(texts[0].contains(NOTES), "{texts:?}");
        assert!(!stub.asked_for_an_asset());
    }

    #[test]
    fn install_update_needs_confirm_true() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let mut session = serving(&stub, dir.path());
        for arguments in [json!({}), json!({ "confirm": false })] {
            let (error, texts) = session.call_all("install_update", arguments);
            assert!(error, "{texts:?}");
            assert_eq!(
                texts,
                [
                    "nothing was installed; ask the person first and call install_update with confirm: true once they agree"
                ]
            );
        }
        assert!(!stub.asked_for_an_asset());
    }

    /// The daily check of the server adds one note to the first result of a normal tool, and only
    /// to that one.
    #[test]
    fn the_first_tool_result_carries_the_note_once() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let mut session = serving(&stub, dir.path());
        let record = dir.path().join("config/update-check.json");
        while !record.exists() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // The check settled and recorded; the note reaches the next result.
        let (_, texts) = session.call_all("list_controls", failing_call());
        assert_eq!(texts.last().unwrap(), &note(), "{texts:?}");
        assert_eq!(texts.len(), 2, "{texts:?}");
        let (_, again) = session.call_all("list_controls", failing_call());
        assert_eq!(again.len(), 1, "{again:?}");
    }

    /// An update tool that answers gives the news itself; no note follows it.
    #[test]
    fn an_update_tool_gives_the_news_instead_of_the_note() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let mut session = serving(&stub, dir.path());
        let record = dir.path().join("config/update-check.json");
        while !record.exists() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let (error, texts) = session.call_all("check_update", json!({}));
        assert!(!error);
        assert_eq!(texts.len(), 1);
        let (_, after) = session.call_all("list_controls", failing_call());
        assert_eq!(after.len(), 1, "{after:?}");
    }

    /// A release that does not verify gives a note that nothing was installed.
    #[test]
    fn a_bad_signature_gives_a_note() {
        let dir = tempfile::tempdir().unwrap();
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
        let stub = UpdateStub::serving(
            std::fs::read(fixtures.join("manifest-99.json")).unwrap(),
            std::fs::read(fixtures.join("manifest-99.json.wrong-key.minisig")).unwrap(),
            Vec::new(),
        );
        let mut session = serving(&stub, dir.path());
        let record = dir.path().join("config/update-check.json");
        while !record.exists() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let (_, texts) = session.call_all("list_controls", failing_call());
        assert!(
            texts.last().unwrap().starts_with("Note for the person: the newest freemapper-courseplan release could not be verified"),
            "{texts:?}"
        );
    }

    /// Mode `install` does not apply to the server: it tells and waits for install_update.
    #[test]
    fn mode_install_does_not_install_from_the_server() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let mut session = Session::start(
            Path::new(BIN),
            &[
                (
                    "FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE",
                    Some(stub.base.as_str()),
                ),
                (
                    "FREEMAPPER_COURSEPLAN_CONFIG_DIR",
                    Some(dir.path().join("config").to_str().unwrap()),
                ),
                ("FREEMAPPER_COURSEPLAN_UPDATE", Some("install")),
            ],
        );
        let record = dir.path().join("config/update-check.json");
        while !record.exists() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let (_, texts) = session.call_all("list_controls", failing_call());
        assert_eq!(texts.last().unwrap(), &note());
        assert!(!stub.asked_for_an_asset());
    }

    /// The whole install over MCP, then the restart request. Skipped without `minisign`.
    #[cfg(unix)]
    #[test]
    fn install_update_with_confirm_installs_and_asks_for_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        let work = dir.path().join("work");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&work).unwrap();
        let program = support::copied_program(&folder);
        let Some(stub) = support::installable_release(&work) else {
            return;
        };
        let mut session = Session::start(
            &program,
            &[
                (
                    "FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE",
                    Some(stub.base.as_str()),
                ),
                (
                    "FREEMAPPER_COURSEPLAN_CONFIG_DIR",
                    Some(dir.path().join("config").to_str().unwrap()),
                ),
                ("FREEMAPPER_COURSEPLAN_UPDATE", Some("off")),
            ],
        );
        let (error, texts) = session.call_all("install_update", json!({ "confirm": true }));
        assert!(!error, "{texts:?}");
        assert_eq!(texts.len(), 1, "{texts:?}");
        assert!(
            texts[0].contains(&format!(
                "freemapper-courseplan updated from {} to 99.0.0",
                env!("CARGO_PKG_VERSION")
            )),
            "{texts:?}"
        );
        assert!(
            texts[0].contains("Restart the freemapper-courseplan MCP server now"),
            "{texts:?}"
        );
        let version = Command::new(&program).arg("--version").output().unwrap();
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            "freemapper-courseplan 99.0.0"
        );
    }
}
