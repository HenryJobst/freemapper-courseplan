//! `freemapper-courseplan update` and the daily check (ADR 0076), against the stub of
//! `tests/support/update_stub.rs`. The tests that reach the network code need a build with the
//! `test-stub` feature (`cargo test -p freemapper-courseplan-cli --features test-stub --test update`);
//! only there does the program count as a release build and take
//! `FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE`. Without the feature the file still checks the source-build
//! answers and the mode setting.

use std::path::Path;
use std::process::Command;

#[cfg(all(feature = "test-stub", debug_assertions))]
#[path = "support/update_stub.rs"]
mod support;
#[cfg(all(unix, feature = "test-stub", debug_assertions))]
use support::installable_release;
#[cfg(all(feature = "test-stub", debug_assertions))]
use support::{MANIFEST_PATH, UpdateStub, copied_program};

const BIN: &str = env!("CARGO_BIN_EXE_freemapper-courseplan");

/// `update` of `program` in English, without the update switches of the environment.
fn update_of(program: &Path) -> Command {
    let mut command = Command::new(program);
    command
        .env("FREEMAPPER_LANG", "en")
        .env_remove("FREEMAPPER_COURSEPLAN_UPDATE")
        .env_remove("FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE")
        .env_remove("FREEMAPPER_COURSEPLAN_CONFIG_DIR")
        .arg("update");
    command
}

#[test]
fn update_in_source_build_says_how_to_update() {
    for extra in [&[][..], &["--check"], &["--yes"], &["--rollback"]] {
        let output = update_of(Path::new(BIN)).args(extra).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{extra:?}: {output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout.trim_end(),
            "This freemapper-courseplan was built from source. Update it with git pull and cargo build --release --locked.",
            "{extra:?}"
        );
    }
}

#[test]
fn update_help_names_the_exit_codes_in_both_languages() {
    for (lang, expected) in [
        ("en", "Exit code 10: with --check"),
        ("de", "Exit-Code 10: mit --check"),
    ] {
        let output = Command::new(BIN)
            .args(["--lang", lang, "update", "--help"])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains(expected), "{stdout}");
    }
}

#[test]
fn update_mode_writes_settings() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    let set = |mode: &str| {
        let output = update_of(Path::new(BIN))
            .env("FREEMAPPER_COURSEPLAN_CONFIG_DIR", &config)
            .args(["--mode", mode])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.contains(&format!("Update mode set to {mode}")),
            "{stdout}"
        );
        std::fs::read_to_string(config.join("settings.toml")).unwrap()
    };
    assert!(set("install").contains("mode = \"install\""));
    assert!(set("off").contains("mode = \"off\""));
    // An unknown mode is a usage error and leaves the file alone.
    let output = update_of(Path::new(BIN))
        .env("FREEMAPPER_COURSEPLAN_CONFIG_DIR", &config)
        .args(["--mode", "sometimes"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(
        std::fs::read_to_string(config.join("settings.toml"))
            .unwrap()
            .contains("mode = \"off\"")
    );
}

/// `update` of `program`, talking to `stub`.
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_with(program: &Path, stub: &UpdateStub) -> Command {
    let mut command = update_of(program);
    command.env("FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE", &stub.base);
    command
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_check_with_stub_reports_available_exit_10() {
    let stub = UpdateStub::with_fixture();
    let output = update_with(Path::new(BIN), &stub)
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let current = env!("CARGO_PKG_VERSION");
    for expected in [
        format!(
            "freemapper-courseplan 99.0.0 is available (you have {current}), released 2026-11-20."
        ),
        "Release notes: https://github.com/HenryJobst/freemapper-courseplan/releases/tag/v99.0.0"
            .to_string(),
        // 12345678 bytes.
        "Download: 11.8 MB".to_string(),
    ] {
        assert!(stdout.contains(&expected), "{expected}\n{stdout}");
    }
    assert_eq!(
        stub.paths(),
        [
            MANIFEST_PATH.to_string(),
            format!("{MANIFEST_PATH}.minisig")
        ]
    );
}

/// A manifest of the sibling program, signed with the right key, is not accepted here.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_refuses_a_manifest_of_another_app() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let manifest = std::fs::read_to_string(dir.join("manifest-99.json"))
        .unwrap()
        .replace(
            "\"app\": \"freemapper-courseplan\"",
            "\"app\": \"freemapper\"",
        );
    let work = tempfile::tempdir().unwrap();
    let file = work.path().join("m.json");
    std::fs::write(&file, &manifest).unwrap();
    let has_minisign = Command::new("minisign")
        .arg("-v")
        .output()
        .is_ok_and(|o| o.status.success());
    if !has_minisign {
        eprintln!("skipped: no minisign on the PATH");
        return;
    }
    let signed = Command::new("minisign")
        .args(["-S", "-s"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/keys/test.key"))
        .arg("-m")
        .arg(&file)
        .args(["-t", "test"])
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert!(signed.status.success(), "{signed:?}");
    let stub = UpdateStub::serving(
        manifest.into_bytes(),
        std::fs::read(work.path().join("m.json.minisig")).unwrap(),
        Vec::new(),
    );
    let output = update_with(Path::new(BIN), &stub)
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error: the update failed: "), "{stderr}");
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_check_with_a_bad_signature_is_an_error() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let mut manifest = std::fs::read(dir.join("manifest-99.json")).unwrap();
    manifest.extend_from_slice(b"\n");
    let stub = UpdateStub::serving(
        manifest,
        std::fs::read(dir.join("manifest-99.json.minisig")).unwrap(),
        Vec::new(),
    );
    let output = update_with(Path::new(BIN), &stub)
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains(
            "error: the release could not be verified (the update signature is not valid); nothing was installed"
        ),
        "{stderr}"
    );
}

/// A release without the manifest (the URL answers 404) cannot install itself; the message says so
/// and links the release page, in the language of the run.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_of_a_release_without_a_manifest_says_so() {
    let stub = UpdateStub::start(Vec::new());
    let program = Path::new(BIN);
    let no_manifest = "error: the newest release cannot install itself (it has no update manifest); \
                       download it from https://github.com/HenryJobst/freemapper-courseplan/releases/latest\n";
    for extra in [&["--check"][..], &["--yes"]] {
        let output = update_with(program, &stub).args(extra).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{extra:?}: {output:?}");
        assert_eq!(String::from_utf8(output.stderr).unwrap(), no_manifest);
    }
    let output = update_with(program, &stub)
        .env("FREEMAPPER_LANG", "de")
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "Fehler: Die neueste Version kann sich nicht selbst installieren (ihr fehlt das \
         Aktualisierungsmanifest). Lade sie von \
         https://github.com/HenryJobst/freemapper-courseplan/releases/latest herunter\n"
    );
    let output = update_with(program, &stub)
        .args(["--version", "0.9.9"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "error: release 0.9.9 does not exist or cannot install itself (it has no update manifest); \
         see https://github.com/HenryJobst/freemapper-courseplan/releases\n"
    );
    assert!(!stub.paths().iter().any(|path| path.ends_with(".minisig")));
}

/// Only the manifest's 404 means "no manifest": a missing signature next to it is another error.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_with_a_missing_signature_is_no_missing_manifest() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let stub = UpdateStub::start(vec![(
        MANIFEST_PATH.to_string(),
        std::fs::read(dir.join("manifest-99.json")).unwrap(),
    )]);
    let output = update_with(Path::new(BIN), &stub)
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("error: the update failed: "), "{stderr}");
    assert!(stderr.contains(".minisig: HTTP 404"), "{stderr}");
}

/// `--check` exits 10 whenever a newer release exists, also one that needs a manual install.
#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_check_of_a_release_that_needs_a_manual_install_exits_10() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
    let stub = UpdateStub::serving(
        std::fs::read(dir.join("manifest-99-manual.json")).unwrap(),
        std::fs::read(dir.join("manifest-99-manual.json.minisig")).unwrap(),
        Vec::new(),
    );
    let output = update_with(Path::new(BIN), &stub)
        .arg("--check")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(10), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(&format!(
            "freemapper-courseplan 99.0.0 cannot be installed by the self-update of {}; install 98.0.0 or newer by hand, see https://github.com/HenryJobst/freemapper-courseplan/releases/tag/v99.0.0",
            env!("CARGO_PKG_VERSION")
        )),
        "{stdout}"
    );
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_without_yes_and_no_terminal_refuses() {
    let dir = tempfile::tempdir().unwrap();
    let program = copied_program(dir.path());
    let stub = UpdateStub::with_fixture();
    let output = update_with(&program, &stub)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("freemapper-courseplan 99.0.0 is available"),
        "{stdout}"
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("freemapper-courseplan update --yes"),
        "{stderr}"
    );
    assert!(!stub.asked_for_an_asset());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

/// A program folder the user cannot write is named, and nothing is downloaded.
#[test]
#[cfg(all(unix, feature = "test-stub", debug_assertions))]
fn update_in_a_read_only_folder_names_it() {
    use std::os::unix::fs::PermissionsExt as _;
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("bin");
    std::fs::create_dir(&folder).unwrap();
    let program = copied_program(&folder);
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o555)).unwrap();
    let stub = UpdateStub::with_fixture();
    let output = update_with(&program, &stub).arg("--yes").output().unwrap();
    std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let canonical = std::fs::canonicalize(&folder).unwrap();
    assert!(
        stderr.contains(&format!(
            "cannot write to the program folder {}",
            canonical.display()
        )),
        "{stderr}"
    );
    assert!(!stub.asked_for_an_asset());
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
}

/// The whole install against the stub (see `installable_release`). Without the `minisign` CLI the
/// test says so and passes.
#[test]
#[cfg(all(unix, feature = "test-stub", debug_assertions))]
fn update_yes_installs_over_a_copied_binary() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("bin");
    let work = dir.path().join("work");
    std::fs::create_dir(&folder).unwrap();
    std::fs::create_dir(&work).unwrap();
    let program = copied_program(&folder);
    let Some(stub) = installable_release(&work) else {
        return;
    };
    let output = update_with(&program, &stub).arg("--yes").output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let current = env!("CARGO_PKG_VERSION");
    assert!(
        stdout.contains("downloading and checking freemapper-courseplan 99.0.0"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!(
            "freemapper-courseplan updated from {current} to 99.0.0. Release notes: https://github.com/HenryJobst/freemapper-courseplan/releases/tag/v99.0.0"
        )),
        "{stdout}"
    );
    assert!(stdout.contains("MCP server"), "{stdout}");
    let version = |path: &Path| {
        let output = Command::new(path).arg("--version").output().unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    };
    assert_eq!(version(&program), "freemapper-courseplan 99.0.0");
    assert_eq!(
        version(&folder.join("freemapper-courseplan.previous")),
        format!("freemapper-courseplan {current}")
    );
}

#[test]
#[cfg(all(feature = "test-stub", debug_assertions))]
fn update_rollback_restores_previous() {
    let dir = tempfile::tempdir().unwrap();
    let program = copied_program(dir.path());
    let previous = dir.path().join(if cfg!(windows) {
        "freemapper-courseplan.previous.exe"
    } else {
        "freemapper-courseplan.previous"
    });
    std::fs::copy(&program, &previous).unwrap();
    // Both are the same program; the time stamp tells them apart.
    let marked = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
    std::fs::File::options()
        .write(true)
        .open(&previous)
        .unwrap()
        .set_modified(marked)
        .unwrap();
    let modified = |path: &Path| std::fs::metadata(path).unwrap().modified().unwrap();
    let stub = UpdateStub::start(Vec::new());
    let output = update_with(&program, &stub)
        .arg("--rollback")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(&format!(
            "Went back to freemapper-courseplan {}",
            env!("CARGO_PKG_VERSION")
        )),
        "{stdout}"
    );
    assert_eq!(modified(&program), marked);
    assert_ne!(modified(&previous), marked);
    assert!(stub.paths().is_empty(), "a rollback sends no request");
    // Without a previous version there is nothing to go back to.
    std::fs::remove_file(&previous).unwrap();
    let output = update_with(&program, &stub)
        .arg("--rollback")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("there is no previous version to go back to"),
        "{stderr}"
    );
}

/// The daily check after `info` (ADR 0076), against the stub. The module name puts these tests
/// under CI's `update` filter.
#[cfg(all(feature = "test-stub", debug_assertions))]
mod update_check {
    use std::path::Path;
    use std::time::{Duration, Instant};

    use freemapper_update::{RecordResult, load_record};

    use super::{BIN, Command, MANIFEST_PATH, UpdateStub, copied_program};

    /// `program` in English, talking to `stub`, without `CI` and the update switches of the
    /// environment, with its settings and record folder in `<dir>/config`.
    fn checking(program: &Path, stub: &UpdateStub, dir: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .env("FREEMAPPER_LANG", "en")
            .env_remove("CI")
            .env_remove("FREEMAPPER_COURSEPLAN_UPDATE")
            .env("FREEMAPPER_COURSEPLAN_CONFIG_DIR", dir.join("config"))
            .env("FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE", &stub.base);
        command
    }

    /// `info` of a project file that does not exist: it fails at once, without the network.
    fn failing_info<'a>(command: &'a mut Command, dir: &Path) -> &'a mut Command {
        command.arg("info").arg(dir.join("missing.courseplan.toml"))
    }

    fn record(dir: &Path) -> Option<RecordResult> {
        load_record(&dir.join("config/update-check.json")).map(|record| record.result)
    }

    fn program() -> &'static Path {
        Path::new(BIN)
    }

    fn notice() -> String {
        format!(
            "freemapper-courseplan 99.0.0 is available (you have {}). Update with: freemapper-courseplan update\n",
            env!("CARGO_PKG_VERSION")
        )
    }

    /// A check that is not done after 3 s is dropped, recorded as failed, and leaves the command's
    /// output and exit code as they are.
    #[test]
    fn slow_source_is_dropped_after_3_s_and_recorded_failed() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture_after(Duration::from_secs(5));
        let without = failing_info(
            checking(program(), &stub, dir.path()).env("CI", "1"),
            dir.path(),
        )
        .output()
        .unwrap();
        assert!(stub.paths().is_empty(), "no check when CI is set");
        let started = Instant::now();
        let with = failing_info(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        let took = started.elapsed();
        assert!(
            took >= Duration::from_millis(2900) && took < Duration::from_millis(8000),
            "{took:?}"
        );
        assert_eq!(with.status.code(), Some(1), "{with:?}");
        assert_eq!(with.status.code(), without.status.code());
        assert_eq!(with.stdout, without.stdout);
        assert_eq!(with.stderr, without.stderr);
        assert_eq!(
            record(dir.path()),
            Some(RecordResult::Failed {
                reason: "timeout".into()
            })
        );
        assert_eq!(stub.paths(), [MANIFEST_PATH]);
    }

    #[test]
    fn server_error_is_recorded_failed_and_not_shown() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::start(Vec::new());
        let without = failing_info(
            checking(program(), &stub, dir.path()).env("CI", "1"),
            dir.path(),
        )
        .output()
        .unwrap();
        let with = failing_info(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        assert_eq!(with.status.code(), without.status.code());
        assert_eq!(with.stdout, without.stdout);
        assert_eq!(with.stderr, without.stderr);
        let Some(RecordResult::Failed { reason }) = record(dir.path()) else {
            panic!("{:?}", record(dir.path()));
        };
        assert!(reason.contains("404"), "{reason}");
        assert_eq!(stub.paths(), [MANIFEST_PATH]);
    }

    #[test]
    fn notice_line_on_stderr_after_command_output() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let without = failing_info(
            checking(program(), &stub, dir.path()).env("CI", "1"),
            dir.path(),
        )
        .output()
        .unwrap();
        let with = failing_info(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        assert_eq!(with.status.code(), Some(1), "{with:?}");
        assert_eq!(with.stdout, without.stdout);
        let stderr = String::from_utf8(with.stderr).unwrap();
        let before = String::from_utf8(without.stderr).unwrap();
        assert!(before.starts_with("error: "), "{before}");
        assert_eq!(stderr, format!("{before}{}", notice()));
        assert_eq!(
            record(dir.path()),
            Some(RecordResult::Available {
                version: "99.0.0".parse().unwrap(),
                notes: "https://github.com/HenryJobst/freemapper-courseplan/releases/tag/v99.0.0"
                    .into(),
            })
        );
        // The next command the same day neither asks nor tells.
        let again = failing_info(&mut checking(program(), &stub, dir.path()), dir.path())
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(again.stderr).unwrap(), before);
        assert_eq!(stub.paths().len(), 2);
    }

    /// A release whose signature does not verify gives one warning line after the command's output,
    /// keeps the exit code, records the attempt as failed, and so says it at most once a day.
    #[test]
    fn bad_signature_gives_one_warning_line() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/update");
        let manifest = std::fs::read(fixtures.join("manifest-99.json")).unwrap();
        let mut changed = manifest.clone();
        changed.extend_from_slice(b"\n");
        for (manifest, signature, detail) in [
            (
                manifest,
                "manifest-99.json.wrong-key.minisig",
                "the update is signed with an unknown key",
            ),
            (
                changed,
                "manifest-99.json.minisig",
                "the update signature is not valid",
            ),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let stub = UpdateStub::serving(
                manifest,
                std::fs::read(fixtures.join(signature)).unwrap(),
                Vec::new(),
            );
            let without = failing_info(
                checking(program(), &stub, dir.path()).env("CI", "1"),
                dir.path(),
            )
            .output()
            .unwrap();
            let with = failing_info(&mut checking(program(), &stub, dir.path()), dir.path())
                .output()
                .unwrap();
            assert_eq!(with.status.code(), Some(1), "{with:?}");
            let before = String::from_utf8(without.stderr).unwrap();
            assert_eq!(
                String::from_utf8(with.stderr).unwrap(),
                format!(
                    "{before}warning: the newest freemapper-courseplan release could not be verified \
                     ({detail}); nothing was installed\n"
                )
            );
            let Some(RecordResult::Failed { reason }) = record(dir.path()) else {
                panic!("{:?}", record(dir.path()));
            };
            assert_eq!(reason, detail);
            let again = failing_info(&mut checking(program(), &stub, dir.path()), dir.path())
                .output()
                .unwrap();
            assert_eq!(String::from_utf8(again.stderr).unwrap(), before);
            assert_eq!(stub.paths().len(), 2);
        }
    }

    /// Mode `off` and the offline commands (`check`, `export`, `init`) never ask.
    #[test]
    fn no_check_in_mode_off_or_after_offline_commands() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        failing_info(
            checking(program(), &stub, dir.path()).env("FREEMAPPER_COURSEPLAN_UPDATE", "off"),
            dir.path(),
        )
        .output()
        .unwrap();
        for command in ["check", "export", "init"] {
            checking(program(), &stub, dir.path())
                .arg(command)
                .arg(dir.path().join("missing"))
                .output()
                .unwrap();
        }
        assert!(stub.paths().is_empty(), "{:?}", stub.paths());
        assert_eq!(record(dir.path()), None);
    }

    /// Mode `install` installs after a command that succeeded, and only then.
    #[test]
    #[cfg(unix)]
    fn install_mode_installs_only_after_success() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        let work = dir.path().join("work");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&work).unwrap();
        let program = copied_program(&folder);
        let Some(stub) = super::installable_release(&work) else {
            return;
        };
        // A command that fails: told, not installed.
        let failed = failing_info(
            checking(&program, &stub, dir.path()).env("FREEMAPPER_COURSEPLAN_UPDATE", "install"),
            dir.path(),
        )
        .output()
        .unwrap();
        assert_eq!(failed.status.code(), Some(1), "{failed:?}");
        assert!(String::from_utf8(failed.stdout).unwrap().is_empty());
        assert!(!stub.asked_for_an_asset());
        std::fs::remove_file(dir.path().join("config/update-check.json")).unwrap();
        // A command that succeeds: installed after its own output.
        let map = dir.path().join("luebeck.omap");
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/maps/luebeck.omap"),
            &map,
        )
        .unwrap();
        let made = Command::new(BIN)
            .env("FREEMAPPER_LANG", "en")
            .arg("init")
            .arg(&map)
            .output()
            .unwrap();
        assert!(made.status.success(), "{made:?}");
        let output = checking(&program, &stub, dir.path())
            .env("FREEMAPPER_COURSEPLAN_UPDATE", "install")
            .arg("info")
            .arg(dir.path().join("luebeck.courseplan.toml"))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let current = env!("CARGO_PKG_VERSION");
        let lines: Vec<&str> = stdout.lines().collect();
        let position = |start: &str| {
            lines
                .iter()
                .position(|line| line.starts_with(start))
                .unwrap_or_else(|| panic!("{start}\n{stdout}"))
        };
        assert!(
            position("title:") < position("downloading and checking freemapper-courseplan 99.0.0")
        );
        assert!(
            position("downloading and checking")
                < position(&format!(
                    "freemapper-courseplan updated from {current} to 99.0.0"
                ))
        );
        assert_eq!(
            lines.last(),
            Some(&"The new version applies from the next start of freemapper-courseplan.")
        );
        assert!(String::from_utf8(output.stderr).unwrap().is_empty());
        let version = Command::new(&program).arg("--version").output().unwrap();
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            "freemapper-courseplan 99.0.0"
        );
    }

    /// Every start deletes the stale files an update left behind, in a release build only.
    #[test]
    fn stale_files_are_removed_at_start() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        std::fs::create_dir(&folder).unwrap();
        let program = copied_program(&folder);
        let stale = folder.join(if cfg!(windows) {
            "freemapper-courseplan.stale-1700000000.exe"
        } else {
            "freemapper-courseplan.stale-1700000000"
        });
        std::fs::write(&stale, "old").unwrap();
        let stub = UpdateStub::start(Vec::new());
        // A build from source leaves them.
        let mut source_build = checking(&program, &stub, dir.path());
        source_build.env_remove("FREEMAPPER_COURSEPLAN_UPDATE_TEST_BASE");
        source_build
            .args(["update", "--mode", "notify"])
            .output()
            .unwrap();
        assert!(stale.exists());
        let output = checking(&program, &stub, dir.path())
            .args(["update", "--mode", "notify"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(!stale.exists());
        assert!(stub.paths().is_empty());
    }
}
