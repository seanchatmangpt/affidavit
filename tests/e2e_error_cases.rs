//! E2E tests for error cases: missing files, bad arguments, unknown verbs.

use std::process::Command;

#[test]
fn verify_nonexistent_file_exits_nonzero() {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(["receipt", "verify", "/tmp/does-not-exist-affi-test.json"])
        .output()
        .expect("run affi");
    assert!(!out.status.success(), "verify on nonexistent file must exit non-zero");
}

#[test]
fn show_nonexistent_file_exits_nonzero() {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(["receipt", "show", "/tmp/does-not-exist-affi-test.json"])
        .output()
        .expect("run affi");
    assert!(!out.status.success(), "show on nonexistent file must exit non-zero");
}

#[test]
fn unknown_verb_exits_nonzero() {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(["receipt", "notarealverb"])
        .output()
        .expect("run affi");
    assert!(!out.status.success(), "unknown verb must exit non-zero");
}

#[test]
fn emit_missing_required_args_exits_nonzero() {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(["receipt", "emit"])  // missing --type, --object, --payload
        .output()
        .expect("run affi");
    assert!(!out.status.success(), "emit without required args must exit non-zero");
}

#[test]
fn no_args_exits_nonzero_or_prints_help() {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .output()
        .expect("run affi");
    // Either exits non-zero OR prints help text (some CLIs exit 0 for no args)
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success() || stdout.contains("USAGE") || stdout.contains("Usage") || stderr.contains("USAGE"),
        "affi with no args must exit non-zero or print usage"
    );
}
