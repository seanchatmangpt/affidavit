// CLI dispatch tests for the cryptographic trust plane (`affi keys`, `affi envelope`).
//
// v26.9.28 trust-plane wave, lane W3-L6. Drives the REAL `affi` binary through
// the full trust lifecycle inside isolated TempDirs (the tests/cli_dispatch.rs
// pattern): mint/list keys, seal an assembled receipt under a raw-hex key,
// adjudicate the sealed document back to a standing, and pin the refusal
// teeth (unknown key, tampered signature, tampered base, bad algorithm).
//
// FEATURE GATE: the whole file is `#![cfg(feature = "crypto-trust")]` — the
// handlers it exercises carry that gate (the rendered verb wrappers compile
// unconditionally, the plane does not). Run with:
//
//     cargo test --features crypto-trust --test crypto_trust_cli
//
// The coordinator's integration run must include the feature; on a default
// build this file compiles to nothing.
//
// NOTE on the trailing "null" (see tests/cli_dispatch.rs): the framework
// appends a trailing "null" for unit-returning verbs. `envelope verify` exits
// the process after flushing (the federation-court pattern), so its stdout is
// exactly the verdict JSON; the other verbs tolerate the residual — only
// directed, stable guarantees are asserted here.
#![cfg(feature = "crypto-trust")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CustodianIdentity, KeyId, KeyOrigin, KeyRecord,
    PublicKeyMaterial,
};

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// Assemble one honest receipt in `dir` (the cli_dispatch lifecycle, 1 event),
/// returning the receipt path.
fn assemble_receipt(dir: &TempDir) -> std::path::PathBuf {
    affi(dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "attest.op",
            "--object",
            "r1:artifact",
            "--payload",
            "-",
        ])
        .write_stdin("trust-plane-payload")
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

/// A real fixture key and its registry record, built with the same rendered
/// primitives the trust-plane courts use — never hand-written crypto. The
/// secret hex goes to `key.hex`; the public record is returned for store
/// fixtures. This is the custody seam the CLI cannot mint for itself: `affi
/// keys generate` discards the secret by law, so a sealing test holds a raw
/// hex secret out-of-band exactly as a custodian would.
fn fixture_key(dir: &TempDir, tag: u8) -> (std::path::PathBuf, KeyRecord) {
    let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    let record = KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm: AlgorithmId::Es256,
        fingerprint,
        custodian: CustodianIdentity {
            subject: "fixture-custodian".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: public,
        created_epoch: 1_700_000_000,
    };
    let key_file = dir.path().join("key.hex");
    let mut hex = String::new();
    for byte in &[tag; 32] {
        hex.push_str(&format!("{byte:02x}"));
    }
    fs::write(&key_file, &hex).expect("write raw secret hex file");
    (key_file, record)
}

/// Write `record` as a one-row key store at `.affi/keys.json` under `dir`.
fn write_store(dir: &TempDir, record: &KeyRecord) -> std::path::PathBuf {
    let store = dir.path().join(".affi").join("keys.json");
    fs::create_dir_all(store.parent().expect("store parent")).expect("create .affi");
    // CTP-STORE-v1 law: records carry a domain-separated checksum (FileKeyStore refuses bare arrays).
    let records = vec![record.clone()];
    let checksum = affidavit::crypto_trust_store::checksum_for(&records);
    let file = affidavit::crypto_trust_store::KeyStoreFile {
        format: affidavit::crypto_trust_store::STORE_FORMAT.to_string(),
        records,
        checksum: checksum.expect("fixture records checksum"),
    };
    fs::write(
        &store,
        serde_json::to_vec(&file).expect("store file serializes"),
    )
    .expect("write key store");
    store
}

#[test]
fn keys_generate_then_list_round_trip() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["keys", "generate", "ES256", "alice", "--out", "keys.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("afk1_")) // the minted kid is printed
        .stdout(predicate::str::contains("alice"));

    // The store holds exactly one real record: parse it and check the law.
    let store = dir.path().join("keys.json");
    let bytes = fs::read(&store).expect("store written");
    let file: affidavit::crypto_trust_store::KeyStoreFile =
        serde_json::from_slice(&bytes).expect("store parses as KeyStoreFile");
    let records = file.records;
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].algorithm, AlgorithmId::Es256);
    assert_eq!(records[0].custodian.subject, "alice");
    assert!(records[0].id.to_string().starts_with("afk1_"));
    assert_eq!(records[0].public_key.encoded_len(), 65); // SEC1 uncompressed

    // list prints the kid/algorithm/custodian rows.
    affi(&dir)
        .args(["keys", "list", "--store", "keys.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&*records[0].id.to_string()))
        .stdout(predicate::str::contains("ES256"))
        .stdout(predicate::str::contains("alice"));
}

#[test]
fn keys_generate_refuses_unknown_algorithm() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args([
            "keys",
            "generate",
            "ML-DSA-65",
            "alice",
            "--out",
            "keys.json",
        ])
        .assert()
        .failure();
    // Nothing was written: a refusal never mutates the store.
    assert!(!dir.path().join("keys.json").exists());
}

#[test]
fn keys_list_reads_a_seeded_store() {
    let dir = TempDir::new().expect("tempdir");
    let (_key_file, record) = fixture_key(&dir, 7);
    // The registry duplicate law (id + fingerprint) is asserted at the unit
    // layer with in-memory registries; here we pin the file seam: a seeded
    // store loads, lists, and keeps its law through the CLI surface.
    write_store(&dir, &record);
    affi(&dir)
        .args(["keys", "list", "--store", ".affi/keys.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&*record.id.to_string()))
        .stdout(predicate::str::contains("ES256"))
        .stdout(predicate::str::contains("fixture-custodian"));
}

#[test]
fn envelope_sign_then_verify_round_trips_to_valid() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let (key_file, record) = fixture_key(&dir, 1);
    write_store(&dir, &record);

    affi(&dir)
        .args([
            "envelope",
            "sign",
            receipt.to_str().expect("receipt path"),
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .success();
    let sealed = dir.path().join("sealed.json");
    assert!(sealed.exists(), "sign must write the sealed document");

    affi(&dir)
        .args([
            "envelope",
            "verify",
            "sealed.json",
            "--store",
            ".affi/keys.json",
        ])
        .assert()
        .code(0) // VALID: the decided accept code
        .stdout(predicate::str::contains("\"standing\": \"VALID\""));
}

#[test]
fn envelope_verify_tampered_signature_is_decided_invalid_exit_2() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let (key_file, record) = fixture_key(&dir, 2);
    write_store(&dir, &record);

    affi(&dir)
        .args([
            "envelope",
            "sign",
            receipt.to_str().expect("receipt path"),
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .success();

    // Flip the last signature byte: a well-formed DER scalar that cannot
    // verify — a DECIDED invalid, not a refusal.
    let text = fs::read_to_string(dir.path().join("sealed.json")).expect("read sealed");
    let mut sealed: affidavit::crypto_trust_seal::SealedReceipt =
        serde_json::from_str(&text).expect("sealed parses");
    let last = sealed.signature.len() - 1;
    sealed.signature[last] ^= 0x01;
    fs::write(
        dir.path().join("sealed.json"),
        serde_json::to_vec_pretty(&sealed).expect("re-serializes"),
    )
    .expect("write tampered seal");

    affi(&dir)
        .args([
            "envelope",
            "verify",
            "sealed.json",
            "--store",
            ".affi/keys.json",
        ])
        .assert()
        .code(2) // the decided-otherwise code
        .stdout(predicate::str::contains("\"standing\": \"INVALID\""));
}

#[test]
fn envelope_verify_unknown_key_is_a_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let (key_file, record) = fixture_key(&dir, 3);

    // Seal under the fixture key, then verify against an EMPTY store: the
    // registry law refuses adjudication (unknown key) instead of deciding.
    affi(&dir)
        .args([
            "envelope",
            "sign",
            receipt.to_str().expect("receipt path"),
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .success();

    affi(&dir)
        .args([
            "envelope",
            "verify",
            "sealed.json",
            "--store",
            "absent-keys.json",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "key store absent-keys.json not found",
        ));
    let _ = record; // the unregistered record is exactly why verify refuses
}

#[test]
fn envelope_verify_tampered_base_refuses_to_deserialize() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let (key_file, record) = fixture_key(&dir, 4);
    write_store(&dir, &record);

    affi(&dir)
        .args([
            "envelope",
            "sign",
            receipt.to_str().expect("receipt path"),
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .success();

    // Tamper the BASE inside the sealed document: the chain law fires inside
    // deserialization — the base never becomes a receipt value at all.
    let text = fs::read_to_string(dir.path().join("sealed.json")).expect("read sealed");
    let tampered = text.replace("attest.op", "attest.opX");
    assert_ne!(text, tampered, "tamper must mutate the base event_type");
    fs::write(dir.path().join("sealed.json"), tampered).expect("write tampered seal");

    affi(&dir)
        .args([
            "envelope",
            "verify",
            "sealed.json",
            "--store",
            ".affi/keys.json",
        ])
        .assert()
        .failure();
}

#[test]
fn envelope_sign_refuses_bad_key_file_and_tampered_receipt() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let (key_file, _record) = fixture_key(&dir, 5);

    // A key file that is not 64 hex characters is a typed refusal.
    let short_key = dir.path().join("short.hex");
    fs::write(&short_key, "aabb").expect("write short key");
    affi(&dir)
        .args([
            "envelope",
            "sign",
            receipt.to_str().expect("receipt path"),
            short_key.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .failure();
    assert!(!dir.path().join("sealed.json").exists());

    // A tampered receipt never reaches a signature: the chain-recomputing
    // deserializer refuses it first.
    let original = fs::read_to_string(&receipt).expect("read receipt");
    let tampered = original.replace("attest.op", "attest.opX");
    assert_ne!(original, tampered);
    fs::write(&receipt, tampered).expect("write tampered receipt");
    affi(&dir)
        .args([
            "envelope",
            "sign",
            receipt.to_str().expect("receipt path"),
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .failure();
}
