//! E2E tests for error cases: missing files, bad arguments, unknown verbs.
//!
//! Each case asserts the exact failure class (exit code 1 = usage/execution
//! error, distinct from the verifier's REJECT exit code 2) and that the
//! diagnostic names the offending input, so a regression that turns an error
//! into a silent success or a bare non-zero exit is caught.

use std::process::{Command, Output};

fn affi(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(args)
        .output()
        .expect("run affi")
}

fn missing_path() -> String {
    let dir = tempfile::TempDir::new().expect("tempdir");
    // The directory is removed on drop, so this path is guaranteed absent.
    dir.path().join("does-not-exist.json").display().to_string()
}

#[test]
fn verify_nonexistent_file_exits_nonzero() {
    let path = missing_path();
    let out = affi(&["receipt", "verify", &path]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "verify on missing file is an execution error, not a REJECT"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does-not-exist.json"),
        "diagnostic must name the file: {stderr}"
    );
}

#[test]
fn show_nonexistent_file_exits_nonzero() {
    let path = missing_path();
    let out = affi(&["receipt", "show", &path]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "show on missing file must exit 1"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("does-not-exist.json"),
        "diagnostic must name the file: {stderr}"
    );
}

#[test]
fn unknown_verb_exits_nonzero() {
    let out = affi(&["receipt", "notarealverb"]);
    assert_eq!(out.status.code(), Some(1), "unknown verb must exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("notarealverb"),
        "diagnostic must name the unknown verb: {stderr}"
    );
}

#[test]
fn emit_missing_required_args_exits_nonzero() {
    let out = affi(&["receipt", "emit"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "emit without required args must exit 1"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--object") && stderr.contains("--payload"),
        "diagnostic must name the missing required arguments: {stderr}"
    );
}

#[test]
fn no_args_prints_usage() {
    let out = affi(&[]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("Usage:") && text.contains("receipt"),
        "affi with no args must print usage listing the nouns: {text}"
    );
}
