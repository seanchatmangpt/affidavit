//! omni-evidence/v1 court: affidavit certifies bundles from its two sibling
//! producers (cgap gate runs, ash_pplan execution receipts).
//!
//! Real collaborators: the built `affi` binary over the committed fixture
//! bundles in `fixtures/omni/` (byte copies of real producer output, see its
//! README). Lifecycle per bundle: emit-batch (relative payload paths resolve
//! against the batch file) -> assemble -> verify ACCEPT (exit 0). Positive
//! control (anti-vacuity): flipping one byte of one sealed event makes verify
//! REJECT (exit 2), so an ACCEPT here cannot be vacuous.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "affi-omni-{tag}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&d).expect("create scratch dir");
    d
}

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .env_remove("RUST_LOG")
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn affi binary");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn bundle(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/omni")
        .join(name)
        .join("batch.json")
}

/// Seal `name`'s bundle in a fresh cwd; return (dir, receipt path, events).
fn seal(name: &str) -> (PathBuf, PathBuf, Vec<serde_json::Value>) {
    let dir = scratch(name);
    let batch = bundle(name);
    let batch_s = batch.to_str().expect("utf8 path");
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(&batch).expect("read batch")).expect("json");

    // Payload paths are relative to the bundle; cwd is elsewhere on purpose.
    let (code, stdout, stderr) = run(&dir, &["receipt", "emit-batch", "--batch-file", batch_s]);
    assert_eq!(code, 0, "{name}: emit-batch exit; stderr={stderr}");
    let n = entries.len();
    assert!(
        stdout.contains(&format!("{n}/{n}")),
        "{name}: expected {n}/{n} emitted, got {stdout:?}"
    );

    let (code, _, stderr) = run(&dir, &["receipt", "assemble", "--out", "receipt.json"]);
    assert_eq!(code, 0, "{name}: assemble exit; stderr={stderr}");
    let receipt = dir.join("receipt.json");
    let sealed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&receipt).expect("read receipt")).expect("json");
    let events = sealed["events"].as_array().expect("events array").clone();
    assert_eq!(events.len(), n, "{name}: every bundle entry sealed");
    for (ev, entry) in events.iter().zip(&entries) {
        assert_eq!(ev["event_type"], entry["event_type"], "{name}: type kept");
    }
    (dir, receipt, events)
}

fn verify(dir: &Path) -> i32 {
    run(dir, &["receipt", "verify", "receipt.json"]).0
}

fn certify_then_tamper(name: &str) {
    let (dir, receipt, _) = seal(name);
    assert_eq!(verify(&dir), 0, "{name}: honest bundle must ACCEPT");

    // Positive control: retype one sealed event; the chain must break.
    let raw = fs::read_to_string(&receipt).expect("read receipt");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("json");
    let ty = v["events"][0]["event_type"]
        .as_str()
        .expect("type")
        .to_string();
    v["events"][0]["event_type"] = serde_json::Value::String(format!("{ty}.forged"));
    fs::write(&receipt, serde_json::to_vec_pretty(&v).expect("encode")).expect("write");
    assert_eq!(verify(&dir), 2, "{name}: tampered bundle must REJECT");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cgap_gate_bundle_certifies_and_tamper_rejects() {
    certify_then_tamper("cgap");
}

#[test]
fn ash_pplan_execution_bundle_certifies_and_tamper_rejects() {
    certify_then_tamper("ash_pplan");
}

/// Commitments are BLAKE3 over the exact payload bytes the producer wrote:
/// the bundle is the evidence, affidavit only seals it.
#[test]
fn commitments_bind_the_producer_payload_bytes() {
    for name in ["cgap", "ash_pplan"] {
        let (dir, _, events) = seal(name);
        let root = bundle(name).parent().expect("bundle dir").to_path_buf();
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&fs::read_to_string(bundle(name)).expect("read")).expect("json");
        for (ev, entry) in events.iter().zip(&entries) {
            let bytes = fs::read(root.join(entry["payload"].as_str().expect("payload")))
                .expect("payload file");
            assert_eq!(
                ev["payload_commitment"].as_str().expect("commitment"),
                blake3::hash(&bytes).to_hex().as_str(),
                "{name}: commitment = blake3(payload bytes)"
            );
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
