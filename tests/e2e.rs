// End-to-end test of the complete receipt lifecycle.
//
// Exercises: emit → assemble → verify → show, with tamper detection.
// Witnesses that the sealed receipt seam works end-to-end through the CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

#[test]
fn e2e_complete_lifecycle_honest() {
    let dir = TempDir::new().expect("tempdir");

    // Stage 1: Emit events
    affi(&dir)
        .args([
            "receipt", "emit", "--type", "init", "--object", "app:service",
            "--payload", "-",
        ])
        .write_stdin("app initialization")
        .assert()
        .success()
        .stdout(predicate::str::contains("emitted event evt-0"));

    affi(&dir)
        .args([
            "receipt", "emit", "--type", "transform", "--object", "data:artifact",
            "--payload", "-",
        ])
        .write_stdin("data transformation")
        .assert()
        .success()
        .stdout(predicate::str::contains("emitted event evt-1"));

    affi(&dir)
        .args([
            "receipt", "emit", "--type", "release", "--object", "app:service",
            "--payload", "-",
        ])
        .write_stdin("release deployment")
        .assert()
        .success()
        .stdout(predicate::str::contains("emitted event evt-2"));

    // Stage 2: Assemble into immutable receipt
    affi(&dir)
        .args(["receipt", "assemble", "--out", "honest.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("assembled receipt ->"))
        .stdout(predicate::str::contains("content address:"));

    // Stage 3: Verify honest receipt (must ACCEPT)
    affi(&dir)
        .args(["receipt", "verify", "honest.json"])
        .assert()
        .success() // exit 0
        .stderr(predicate::str::contains("verdict: ACCEPT"))
        .stderr(predicate::str::contains("all stages passed"));

    // Stage 4: Show receipt details
    affi(&dir)
        .args(["receipt", "show", "honest.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("receipt format:"))
        .stderr(predicate::str::contains("events: 3"))
        .stderr(predicate::str::contains("evt-0"))
        .stderr(predicate::str::contains("evt-1"))
        .stderr(predicate::str::contains("evt-2"))
        .stderr(predicate::str::contains("chain hash:"));
}

#[test]
fn e2e_tamper_detection() {
    let dir = TempDir::new().expect("tempdir");

    // Build an honest receipt
    affi(&dir)
        .args([
            "receipt", "emit", "--type", "create", "--object", "file:artifact",
            "--payload", "-",
        ])
        .write_stdin("file content")
        .assert()
        .success();

    affi(&dir)
        .args(["receipt", "assemble", "--out", "tampered.json"])
        .assert()
        .success();

    // Tamper with the receipt (change an event type)
    let receipt = fs::read_to_string(dir.path().join("tampered.json"))
        .expect("read receipt");
    let tampered = receipt.replace("\"create\"", "\"forged\"");
    fs::write(dir.path().join("tampered.json"), tampered)
        .expect("write tampered receipt");

    // Tampered receipt fails at deserialization (stronger than verify rejection)
    affi(&dir)
        .args(["receipt", "verify", "tampered.json"])
        .assert()
        .failure() // non-zero exit
        .stderr(predicate::str::contains("chain hash mismatch")); // deserialization forgery gate (ADR-3)
}

#[test]
fn e2e_objectless_receipt_rejected_by_ocel_court() {
    // The "empty room behind a closed door" attack, end-to-end through the real
    // binary: a CHAIN-CONSISTENT receipt whose only event has no objects. The
    // affidavit verifier alone ACCEPTS it (chain/continuity/commitments all
    // pass) — only the wasm4pm-compat OCEL court, run inside admit() on the real
    // verify path, refuses it by name. A non-zero exit here is the witness that
    // the court is load-bearing in production, not just in unit tests.
    use affidavit::chain::{recompute_chain, FORMAT_VERSION};
    use affidavit::types::{Blake3Hash, OperationEvent};

    let dir = TempDir::new().expect("tempdir");

    let event = OperationEvent {
        id: "evt-0".to_string(),
        seq: 0,
        event_type: "create".to_string(),
        objects: vec![], // the empty room
        payload_commitment: Blake3Hash::from_bytes(b"payload"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&event)).expect("recompute chain");
    let json = serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [event],
        "chain_hash": chain_hash,
    });
    fs::write(
        dir.path().join("objectless.json"),
        serde_json::to_string(&json).expect("serialize forged receipt"),
    )
    .expect("write objectless receipt");

    affi(&dir)
        .args(["receipt", "verify", "objectless.json"])
        .assert()
        .failure() // non-zero exit — the court refused it
        .stderr(predicate::str::contains("EmptyEventObjectLinks"));
}

#[test]
fn e2e_qualified_objects() {
    let dir = TempDir::new().expect("tempdir");

    // Event with qualified object (role specified)
    affi(&dir)
        .args([
            "receipt", "emit", "--type", "transform",
            "--object", "dataset:artifact:input",
            "--payload", "-",
        ])
        .write_stdin("data transformation")
        .assert()
        .success();

    affi(&dir)
        .args(["receipt", "assemble", "--out", "qualified.json"])
        .assert()
        .success();

    // Show must display qualified object correctly (id:type/qualifier format)
    affi(&dir)
        .args(["receipt", "show", "qualified.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("dataset:artifact/input"));
}

#[test]
fn e2e_stdin_payload() {
    let dir = TempDir::new().expect("tempdir");

    // Payload from stdin (as opposed to file)
    affi(&dir)
        .args([
            "receipt", "emit", "--type", "stdin_test", "--object", "test:artifact",
            "--payload", "-",
        ])
        .write_stdin("this is from stdin")
        .assert()
        .success();

    affi(&dir)
        .args(["receipt", "assemble", "--out", "from_stdin.json"])
        .assert()
        .success();

    affi(&dir)
        .args(["receipt", "verify", "from_stdin.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("verdict: ACCEPT"));
}
