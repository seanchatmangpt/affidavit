// Witness: the verify operation emits an OBSERVABLE span.
//
// Failing-when-fake: this test clears the span sink, runs a real verify, and
// asserts a `verify` span was captured. If `cli::verify` did not open a span
// (remove the `trace_verify` wrapper and the span vanishes), this test FAILS.
// A green here therefore carries information — the instrumentation is real, not
// a dormant module. (Full Jaeger/OTLP export remains OPEN-substrate; this
// witnesses span emission + capture only — see src/tracing.rs honest scope.)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn verify_emits_an_observable_span() {
    let dir = TempDir::new().expect("tempdir");

    // Build an honest receipt via the binary (emit + assemble).
    let mut cmd = Command::cargo_bin("affi").expect("affi binary");
    cmd.current_dir(dir.path());
    cmd.args([
        "receipt",
        "emit",
        "--type",
        "create",
        "--object",
        "f:artifact",
        "--payload",
        "-",
    ])
    .write_stdin("content")
    .assert()
    .success();

    let mut cmd = Command::cargo_bin("affi").expect("affi binary");
    cmd.current_dir(dir.path());
    cmd.args(["receipt", "assemble", "--out", "honest.json"])
        .assert()
        .success();

    let receipt_path = dir.path().join("honest.json");
    let receipt_str = receipt_path.to_string_lossy().to_string();
    assert!(fs::metadata(&receipt_path).is_ok(), "receipt assembled");

    // Drive verify in-process so we share the thread-local span sink.
    affidavit::tracing::clear_spans();
    let (code, verdict) = affidavit::cli::verify(&receipt_str).expect("verify runs");
    assert_eq!(code, 0, "honest receipt accepts");
    assert!(verdict.accepted);

    // THE witness: a verify span was actually emitted for this receipt.
    let spans = affidavit::tracing::captured_spans();
    assert!(
        spans
            .iter()
            .any(|s| s.operation == "verify" && s.target == receipt_str),
        "verify must emit an observable span for the receipt; captured: {spans:?}"
    );
}
