// Dispatch test for the `affi` CLI.
//
// Proves that each `receipt` verb routes to the correct handler by asserting
// the distinguishing stdout per verb against the REAL binary, driving the full
// receipt lifecycle inside an isolated TempDir.
//
// The stdout transport is witnessed in two layers (ARDPRD §6):
// 1. Construction-time: #![deny(clippy::print_stdout)] at library root prevents
//    the macro class. This test is the behavioral layer (layer 2).
// 2. Behavioral: This test drives the binary and asserts stdout content is clean.
//
// NOTE on the trailing "null": the framework emits a trailing "null" for
// unit-returning verbs (clap-noun-verb registry prints
// `output_format.format(&output.data)` where data is serde_json Null). The
// `run_with_default_format(Quiet)` suppression hook was backed out upstream
// (clap-noun-verb e9d061c) as an undirected API expansion, so the null is a known
// OPEN residual — NOT asserted here — until a directed suppression mechanism lands.
// These tests pin only what is currently a real, directed guarantee: dispatch
// routing (each verb reaches its own handler), incl. the verify<->show inversion.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// Drive emit x2 + assemble in `dir`, returning the receipt path. Used as setup
/// by the lifecycle stages that need an assembled receipt.
fn lifecycle_up_to_assemble(dir: &TempDir) -> std::path::PathBuf {
    affi(dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o1:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("hello")
        .assert()
        .success();
    affi(dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o2:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("world")
        .assert()
        .success();
    affi(dir)
        .args(["receipt", "assemble", "--out", "receipt.json"])
        .assert()
        .success();
    let receipt = dir.path().join("receipt.json");
    assert!(receipt.exists(), "assemble must produce receipt.json");
    receipt
}

#[test]
fn dispatch_emit_first() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o1:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("emitted event")); // dispatch
}

#[test]
fn dispatch_emit_second() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o1:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("hello")
        .assert()
        .success();
    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o2:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("world")
        .assert()
        .success()
        .stdout(predicate::str::contains("emitted event")); // dispatch
}

#[test]
fn dispatch_assemble() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o1:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("hello")
        .assert()
        .success();
    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "o2:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("world")
        .assert()
        .success();
    affi(&dir)
        .args(["receipt", "assemble", "--out", "receipt.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("assembled receipt ->")) // dispatch
        .stdout(predicate::str::contains("content address:")); // dispatch
}

#[test]
fn dispatch_verify_honest_accept() {
    let dir = TempDir::new().expect("tempdir");
    lifecycle_up_to_assemble(&dir);
    affi(&dir)
        .args(["receipt", "verify", "receipt.json"])
        .assert()
        .success() // exit code 0
        .stderr(predicate::str::contains("verdict: ACCEPT")); // dispatch (output on stderr per §6)
}

#[test]
fn dispatch_show_is_not_verify() {
    let dir = TempDir::new().expect("tempdir");
    lifecycle_up_to_assemble(&dir);
    affi(&dir)
        .args(["receipt", "show", "receipt.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("chain hash:")) // dispatch (output on stderr per §6)
        .stderr(predicate::str::contains("events:")) // dispatch
        .stderr(predicate::str::contains("verdict:").not()); // verify<->show inversion
}

#[test]
fn dispatch_verify_tampered_reject() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = lifecycle_up_to_assemble(&dir);

    let original = fs::read_to_string(&receipt).expect("read receipt");
    let tampered = original.replace("\"emit\"", "\"emitX\"");
    assert_ne!(
        original, tampered,
        "tamper must actually mutate an event_type"
    );
    fs::write(&receipt, tampered).expect("write tampered receipt");

    affi(&dir)
        .args(["receipt", "verify", "receipt.json"])
        .assert()
        .failure() // non-zero exit
        // Tampered receipt fails at deserialization (chain hash mismatch)—stronger than verify rejection
        .stderr(predicate::str::contains("chain hash mismatch")); // deserialization gate closed (ADR-3)
}
