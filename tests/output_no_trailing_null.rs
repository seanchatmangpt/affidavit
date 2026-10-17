//! B13 court: no verb success path may print a stray trailing `null` line.
//!
//! Root cause (fixed in vendored clap-noun-verb, `vendor/clap-noun-verb/src/
//! cli/registry.rs`): every verb returns `Result<()>`; the `#[verb]` wrapper
//! serializes the unit into `HandlerOutput.data = Value::Null`, and the
//! framework printed that formatted Null after the verb's own output unless
//! the effective format was `quiet`. The vendored patch suppresses the data
//! print when `output.data.is_null()`; verbs returning real data still print.
//!
//! Positive controls: the human-primary-output and JSON-payload assertions
//! prove the court can fail if the primary print path itself dies (a fix
//! that suppressed ALL output would fail the `contains` assertions below).

use std::process::Command;

fn affi() -> Command {
    Command::new(env!("CARGO_BIN_EXE_affi"))
}

fn assert_no_null_line(stdout: &[u8], ctx: &str) {
    let s = String::from_utf8_lossy(stdout);
    for (i, line) in s.lines().enumerate() {
        assert_ne!(
            line.trim_end(),
            "null",
            "{ctx}: line {} is a stray `null` (B13 regression). stdout:\n{s}",
            i + 1
        );
    }
}

fn tmpdir() -> tempfile::TempDir {
    tempfile::TempDir::new().expect("tempdir")
}

/// Emit one event; returns (stdout, stderr, exit status).
fn emit_in(dir: &std::path::Path, extra: &[&str]) -> (String, String, Option<i32>) {
    let payload = dir.join("p.txt");
    std::fs::write(&payload, "hi").expect("payload");
    let out = affi()
        .args([
            "receipt",
            "emit",
            "--event-type",
            "build",
            "--object",
            "repo:main",
            "--payload",
        ])
        .arg(&payload)
        .args(extra)
        .current_dir(dir)
        .output()
        .expect("emit");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn emit_success_has_no_trailing_null() {
    let dir = tmpdir();
    let (stdout, _stderr, code) = emit_in(dir.path(), &[]);
    assert_eq!(code, Some(0), "emit must succeed");
    assert_no_null_line(stdout.as_bytes(), "receipt emit (human)");
    assert!(
        stdout.contains("emitted event evt-0"),
        "primary output must be intact: {stdout}"
    );
}

#[test]
fn emit_json_success_has_no_trailing_null() {
    let dir = tmpdir();
    let (stdout, _stderr, code) = emit_in(dir.path(), &["--format", "json"]);
    assert_eq!(code, Some(0), "emit must succeed");
    assert_no_null_line(stdout.as_bytes(), "receipt emit --format json");
    assert!(
        stdout.contains("\"event_id\"") && stdout.contains("\"commitment\""),
        "json payload must be intact: {stdout}"
    );
}

#[test]
fn assemble_and_show_success_have_no_trailing_null() {
    let dir = tmpdir();
    let (_stdout, _stderr, code) = emit_in(dir.path(), &[]);
    assert_eq!(code, Some(0), "emit must succeed");

    let asm = affi()
        .args(["receipt", "assemble"])
        .current_dir(dir.path())
        .output()
        .expect("assemble");
    assert_eq!(asm.status.code(), Some(0), "assemble must succeed");
    let asm_out = String::from_utf8_lossy(&asm.stdout);
    assert_no_null_line(&asm.stdout, "receipt assemble");
    assert!(
        asm_out.contains("assembled receipt ->") && asm_out.contains("content address:"),
        "assemble primary output must be intact: {asm_out}"
    );

    // Discover the assembled receipt file (content-addressed name).
    let entry = std::fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(std::result::Result::ok)
        .find(|e| e.path().extension().is_some_and(|x| x == "json"))
        .map(|e| e.path())
        .expect("assembled receipt file must exist");
    let show = affi()
        .arg("receipt")
        .arg("show")
        .arg(&entry)
        .output()
        .expect("show");
    assert_eq!(show.status.code(), Some(0), "show must succeed");
    assert_no_null_line(&show.stdout, "receipt show");
}
