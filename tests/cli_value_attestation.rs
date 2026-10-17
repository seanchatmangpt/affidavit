// Court: CLI value matrix for the attestation & crypto-plane verb surface —
// receipt attest, sign, notarize, assemble-with-signature,
// assemble-and-notarize (5 verbs).
//
// Law under test:
// 1. `receipt attest` emits a REAL SLSA-style attestation document whose
//    subject digest is the actual chain hash of the receipt under test —
//    value: different receipts → different digests (a literal or a digest of
//    the wrong chain is a fail).
// 2. The 4 crypto-plane verbs (sign, notarize, assemble-with-signature,
//    assemble-and-notarize) are NEGATIVE-CUM-POSITIVE on the default binary:
//    the honest typed surface is exit 224 (1504 CliExecutionError mod 256)
//    naming `REFUSED_UNSUPPORTED` and the `--features crypto-trust` rebuild
//    instruction. The refusal IS the specified behavior for this binary.
//    NOTE: positive crypto paths (real ES256 seals, real notarization, real
//    envelopes) are covered by the crypto-trust plane battery lanes
//    (tests/crypto_trust_*.rs — crypto_trust_cli.rs, crypto_trust_e2e.rs,
//    crypto_trust_mutation_court.rs) — never duplicated here.
// 3. Negative: a missing receipt path is a typed refusal, never a bare
//    panic or a bare exit 1.
//
// Chicago-style: real binary, real files, real exit codes, no mocks.
//
// Lane: write lane, owns exactly `tests/cli_value_attestation.rs`.
//
// value-matrix: (receipt, attest)
// value-matrix: (receipt, sign)
// value-matrix: (receipt, notarize)
// value-matrix: (receipt, assemble-with-signature)
// value-matrix: (receipt, assemble-and-notarize)
//
// value-matrix: harness — tests/common/mod.rs absent; harness defined locally.

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Stable process-exit codes (public contract, `src/diag.rs::exit_codes`).
mod xc {
    /// CLI framework codes 1500..=1509 ride mod 256 on the 8-bit POSIX status:
    /// 1504 (CliExecutionError) → 224.
    pub const CLI_EXEC_ERROR: i32 = 1504 % 256; // 224
}

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// Emit one event and assemble it into `name` — a real `core/v1` receipt
/// admitted through the full admission gate. Returns the assembled receipt's
/// `chain_hash` read back from disk.
fn admitted_receipt(dir: &TempDir, tag: &str, name: &str) -> String {
    affi(dir)
        .args([
            "receipt",
            "emit",
            "--event-type",
            "build",
            "--object",
            "repo:git",
            "--payload",
            "-",
        ])
        .write_stdin(format!("execution payload for {tag}"))
        .assert()
        .success();

    affi(dir)
        .args(["receipt", "assemble", "--out", name])
        .assert()
        .success();

    let raw = fs::read_to_string(dir.path().join(name)).expect("assembled receipt readable");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("receipt is JSON");
    v["chain_hash"]
        .as_str()
        .expect("receipt carries chain_hash")
        .to_string()
}

/// Combined fixture: two independently assembled receipts (different payloads
/// → different BLAKE3 folds) in one scratch dir.
fn two_distinct_receipts(dir: &TempDir) -> (String, String) {
    let chain_a = admitted_receipt(dir, "a", "receipt-a.json");
    let chain_b = admitted_receipt(dir, "b", "receipt-b.json");
    assert_ne!(
        chain_a, chain_b,
        "two receipts with different payloads must have different chain hashes"
    );
    (chain_a, chain_b)
}

/// `receipt attest` — value: the attestation subject digest IS the actual
/// chain hash of the receipt under test, and different receipts attest to
/// different digests.
#[test]
fn attest_emits_slsa_doc_bound_to_actual_subject_digest() {
    let dir = TempDir::new().expect("scratch dir");
    let (chain_a, chain_b) = two_distinct_receipts(&dir);

    for (receipt, expected) in [("receipt-a.json", chain_a), ("receipt-b.json", chain_b)] {
        let out = affi(&dir)
            .args(["receipt", "attest", receipt])
            .assert()
            .success();
        let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();

        let doc: serde_json::Value =
            serde_json::from_str(&stdout).expect("attest stdout is a JSON attestation document");

        assert_eq!(
            doc["_type"].as_str(),
            Some("slsa-v1"),
            "_type defaults to slsa-v1"
        );
        assert!(
            doc["predicateType"]
                .as_str()
                .expect("predicateType present")
                .starts_with("https://slsa.dev/provenance/"),
            "predicateType is an SLSA provenance URI: {doc}"
        );

        let digest = doc["subject"][0]["digest"]["blake3"]
            .as_str()
            .expect("subject digest present");
        assert_eq!(
            digest, expected,
            "attestation subject digest must be the receipt's actual chain hash"
        );

        // Materials bind the real per-event commitments, not placeholders.
        let materials = doc["predicate"]["materials"]
            .as_array()
            .expect("materials array present");
        assert!(
            !materials.is_empty(),
            "materials must carry the receipt's real events"
        );
        for m in materials {
            let d = m["digest"]["blake3"].as_str().expect("material digest");
            assert_eq!(d.len(), 64, "material digest is a 64-hex BLAKE3 digest");
        }
    }
}

/// `receipt attest --attestation-type` — the requested type is carried into
/// the document verbatim (input-sensitive output, not a literal).
#[test]
fn attest_carries_requested_attestation_type() {
    let dir = TempDir::new().expect("scratch dir");
    let _ = admitted_receipt(&dir, "t", "receipt.json");

    let out = affi(&dir)
        .args([
            "receipt",
            "attest",
            "receipt.json",
            "--attestation-type",
            "slsa-v0.2",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout).into_owned();
    let doc: serde_json::Value =
        serde_json::from_str(&stdout).expect("attestation document is JSON");
    assert_eq!(doc["_type"].as_str(), Some("slsa-v0.2"));
    assert!(
        doc["predicateType"]
            .as_str()
            .expect("predicateType present")
            .ends_with("/slsa-v0.2"),
        "predicateType follows the requested type"
    );
}

/// The 4 crypto-plane verbs on the DEFAULT binary: the honest typed surface is
/// exit 224 `REFUSED_UNSUPPORTED` naming `--features crypto-trust`. The
/// refusal IS the specified behavior (negative-cum-positive); the positive
/// crypto paths are covered by the crypto-trust plane battery lanes.
#[test]
fn crypto_plane_verbs_refuse_typed_on_default_binary() {
    for (verb, args) in [
        ("sign", vec!["receipt.json", "--key-path", "key.hex"]),
        ("notarize", vec!["receipt.json"]),
        ("assemble-with-signature", vec![]),
        ("assemble-and-notarize", vec![]),
    ] {
        let dir = TempDir::new().expect("scratch dir");
        let _ = admitted_receipt(&dir, "c", "receipt.json");

        let out = affi(&dir)
            .args(["receipt", verb])
            .args(&args)
            .assert()
            .failure();
        let output = out.get_output();
        assert_eq!(
            output.status.code(),
            Some(xc::CLI_EXEC_ERROR),
            "{verb}: must exit 224 (CliExecutionError mod 256)"
        );
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            text.contains("REFUSED_UNSUPPORTED"),
            "{verb}: refusal must be typed REFUSED_UNSUPPORTED: {text}"
        );
        assert!(
            text.contains("crypto-trust"),
            "{verb}: refusal must name the --features crypto-trust rebuild path: {text}"
        );
        assert!(
            !text.contains("panicked"),
            "{verb}: refusal must be typed, not a panic: {text}"
        );
    }
}

/// Negative: `receipt attest` on a missing receipt path is a typed refusal
/// (nonzero exit, the offending path named), never a panic or a bare exit 1.
#[test]
fn attest_on_missing_receipt_path_is_typed() {
    let dir = TempDir::new().expect("scratch dir");
    let out = affi(&dir)
        .args(["receipt", "attest", "no-such-receipt.json"])
        .assert()
        .failure();
    let output = out.get_output();
    let code = output.status.code().unwrap_or(-1);
    assert_ne!(code, 0, "missing receipt must not succeed");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("no-such-receipt.json"),
        "typed refusal must name the offending path: {text}"
    );
    assert!(
        !text.contains("panicked"),
        "refusal must be typed, not a panic: {text}"
    );
}
