//! Wrap-up lane: receipt verbs no first-wave lane covered —
//! `dependency-matrix`, `emit-from-sbom`, `gdpr-proof`, `hipaa`, `pci-dss`,
//! `soc2-audit` — over the real `affi` binary (Chicago style: real
//! subprocesses, real receipt stores, assertions on real process state).
//!
//! Each verb gets a positive control, a negative control, and at least one
//! value assertion on the actual stdout/artifact. Local harness.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

// value-matrix: (receipt, dependency-matrix)
// value-matrix: (receipt, emit-from-sbom)
// value-matrix: (receipt, gdpr-proof)
// value-matrix: (receipt, hipaa)
// value-matrix: (receipt, pci-dss)
// value-matrix: (receipt, soc2-audit)

const EXIT_OK: i32 = 0;

fn run_in(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out: Output = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A receipts dir with one honest assembled receipt, plus a payload file.
fn receipt_store(dir: &Path) -> PathBuf {
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).expect("mkdir receipts");
    let payload = dir.join("p.txt");
    fs::write(&payload, "wrap-up payload\n").expect("payload");
    let (code, _, stderr) = run_in(
        dir,
        &[
            "receipt",
            "emit",
            "--event-type",
            "build",
            "--object",
            "repo:git",
            "--payload",
            payload.to_str().expect("utf8"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[fixture emit] {stderr}");
    let (code, _, stderr) = run_in(dir, &["receipt", "assemble", "--out", "receipts/r1.json"]);
    assert_eq!(code, EXIT_OK, "[fixture assemble] {stderr}");
    receipts
}

const SPDX: &str = r#"{"spdxVersion":"SPDX-2.3","name":"wrapup-court","SPDXID":"SPDXRef-DOCUMENT","packages":[{"name":"zlib","versionInfo":"1.2.13","SPDXID":"SPDXRef-Package-zlib"}]}"#;

// ---------------------------------------------------------------------------
// dependency-matrix
// ---------------------------------------------------------------------------

#[test]
fn dependency_matrix_reports_pairwise_sharing_over_a_real_store() {
    let dir = TempDir::new().expect("tempdir");
    let store = receipt_store(dir.path());
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "dependency-matrix",
            "--receipts-path",
            store.to_str().expect("utf8"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[dependency-matrix] err={stderr}");
    assert!(
        stdout.contains("object,receipt_a,receipt_b"),
        "value: the CSV header names the pairwise columns; out={stdout}"
    );
}

#[test]
fn dependency_matrix_on_a_missing_store_is_typed_refused() {
    let dir = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "dependency-matrix",
            "--receipts-path",
            "no-such-dir",
        ],
    );
    assert_ne!(code, EXIT_OK, "a missing store must not exit 0");
    assert!(
        stderr.contains("Path not found"),
        "value: the refusal names the missing path law; err={stderr}"
    );
}

// ---------------------------------------------------------------------------
// emit-from-sbom
// ---------------------------------------------------------------------------

#[test]
fn emit_from_sbom_appends_component_events_to_the_working_chain() {
    let dir = TempDir::new().expect("tempdir");
    fs::write(dir.path().join("sbom.json"), SPDX).expect("write sbom");
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &["receipt", "emit-from-sbom", "--sbom-path", "sbom.json"],
    );
    assert_eq!(code, EXIT_OK, "[emit-from-sbom] err={stderr}");
    assert!(
        stdout.contains("1 components") && stdout.contains("OCEL events appended"),
        "value: stdout counts the ingested components and appended events; out={stdout}"
    );
    // The working chain must actually have grown: assemble and check event count.
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &["receipt", "assemble", "--out", "receipt.json"],
    );
    assert_eq!(code, EXIT_OK, "[assemble after sbom] err={stderr}");
    let body = fs::read_to_string(dir.path().join("receipt.json")).expect("read receipt");
    assert!(
        body.contains("zlib"),
        "value: the SBOM component id lands in the assembled chain; body={body}"
    );
}

#[test]
fn emit_from_sbom_refuses_a_missing_sbom_file() {
    let dir = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "emit-from-sbom",
            "--sbom-path",
            "no-such-sbom.json",
        ],
    );
    assert_ne!(code, EXIT_OK, "a missing SBOM must not exit 0");
    assert!(
        stderr.contains("IO failure") && stderr.contains("No such file"),
        "value: the refusal is a typed IO failure for the missing SBOM; err={stderr}"
    );
}

// ---------------------------------------------------------------------------
// statutory proofs: gdpr-proof | hipaa | pci-dss | soc2-audit
// ---------------------------------------------------------------------------

#[test]
fn gdpr_proof_reports_regulation_and_scanned_event_count() {
    let dir = TempDir::new().expect("tempdir");
    let store = receipt_store(dir.path());
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "gdpr-proof",
            "--receipts-path",
            store.to_str().expect("utf8"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[gdpr-proof] err={stderr}");
    assert!(
        stdout.contains("\"regulation\": \"GDPR\""),
        "value: the proof names its regulation; out={stdout}"
    );
    assert!(
        stdout.contains("\"receipts_analyzed\": 1"),
        "value: the proof counts the real receipts scanned; out={stdout}"
    );
}

#[test]
fn gdpr_proof_on_a_missing_store_is_typed_refused() {
    let dir = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run_in(
        dir.path(),
        &["receipt", "gdpr-proof", "--receipts-path", "no-such-dir"],
    );
    assert_ne!(code, EXIT_OK, "a missing store must not exit 0");
    assert!(
        stderr.contains("Path not found"),
        "value: the refusal names the missing path law; err={stderr}"
    );
}

#[test]
fn hipaa_proof_reports_regulation_and_phi_vocabulary() {
    let dir = TempDir::new().expect("tempdir");
    let store = receipt_store(dir.path());
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "hipaa",
            "--receipts-path",
            store.to_str().expect("utf8"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[hipaa] err={stderr}");
    assert!(
        stdout.contains("\"regulation\": \"HIPAA\""),
        "value: the proof names its regulation; out={stdout}"
    );
    assert!(
        stdout.contains("phi_vocabulary"),
        "value: the HIPAA proof carries the PHI vocabulary; out={stdout}"
    );
}

#[test]
fn hipaa_on_a_missing_store_is_typed_refused() {
    let dir = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run_in(
        dir.path(),
        &["receipt", "hipaa", "--receipts-path", "no-such-dir"],
    );
    assert_ne!(code, EXIT_OK, "a missing store must not exit 0");
    assert!(
        stderr.contains("Path not found"),
        "value: the refusal names the missing path law; err={stderr}"
    );
}

#[test]
fn pci_dss_proof_binds_audit_logs_to_the_tamper_evident_chain() {
    let dir = TempDir::new().expect("tempdir");
    let store = receipt_store(dir.path());
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "pci-dss",
            "--receipts-path",
            store.to_str().expect("utf8"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[pci-dss] err={stderr}");
    assert!(
        stdout.contains("\"regulation\": \"PCI-DSS\""),
        "value: the proof names its regulation; out={stdout}"
    );
    assert!(
        stdout.contains("tamper-evident chain"),
        "value: req_10 binds audit logs to the chain; out={stdout}"
    );
}

#[test]
fn pci_dss_on_a_missing_store_is_typed_refused() {
    let dir = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run_in(
        dir.path(),
        &["receipt", "pci-dss", "--receipts-path", "no-such-dir"],
    );
    assert_ne!(code, EXIT_OK, "a missing store must not exit 0");
    assert!(
        stderr.contains("Path not found"),
        "value: the refusal names the missing path law; err={stderr}"
    );
}

#[test]
fn soc2_audit_reports_type_ii_and_chain_backed_security() {
    let dir = TempDir::new().expect("tempdir");
    let store = receipt_store(dir.path());
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "receipt",
            "soc2-audit",
            "--receipts-path",
            store.to_str().expect("utf8"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[soc2-audit] err={stderr}");
    assert!(
        stdout.contains("SOC 2 Type II"),
        "value: the report names its type; out={stdout}"
    );
    assert!(
        stdout.contains("BLAKE3"),
        "value: the security criterion is backed by the real chain hash; out={stdout}"
    );
}

#[test]
fn soc2_audit_on_a_missing_store_is_typed_refused() {
    let dir = TempDir::new().expect("tempdir");
    let (code, _, stderr) = run_in(
        dir.path(),
        &["receipt", "soc2-audit", "--receipts-path", "no-such-dir"],
    );
    assert_ne!(code, EXIT_OK, "a missing store must not exit 0");
    assert!(
        stderr.contains("Path not found"),
        "value: the refusal names the missing path law; err={stderr}"
    );
}
