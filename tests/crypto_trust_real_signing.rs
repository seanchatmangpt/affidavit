// Real-signing CLI courts for the cryptographic trust plane (`receipt sign`,
// `receipt notarize`, `receipt assemble-with-signature`,
// `receipt assemble-and-notarize`, `keys`/`envelope` on the FileKeyStore).
//
// v26.9.28 trust-plane wave, lane W4-L1. These tests replace the signature
// theater: every asserted `"signed"`/`"standing"`/`"attested"` value below is
// produced by a real RFC 6979 ECDSA seal adjudicated by the rendered
// verification engine — never a literal. Each test drives the REAL `affi`
// binary inside an isolated TempDir (the tests/cli_dispatch.rs pattern) with
// no network.
//
// FEATURE GATE: the whole file is `#![cfg(feature = "crypto-trust")]` — the
// handlers it exercises carry that gate. Run with:
//
//     cargo test --features crypto-trust --test crypto_trust_real_signing
//
// FIXTURE NOTE: the receipt fixture is seeded through the library chain law
// (ocel::build_event -> .affi/working.json -> CLI `receipt assemble`), which
// is the same path `receipt emit` drives internally. This keeps the lane
// self-sufficient against rendered-wrapper argument-name churn (`emit`'s
// r#type currently surfaces as `--r#type`).
//
// NOTE on the trailing "null" (see tests/cli_dispatch.rs): the framework
// appends a trailing "null" for unit-returning verbs, so stdout JSON is
// parsed through [`stdout_json`], which admits the object prefix — only
// directed guarantees are asserted.
#![cfg(feature = "crypto-trust")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CustodianIdentity, KeyId, KeyOrigin, KeyRecord,
    PublicKeyMaterial,
};
use affidavit::crypto_trust_seal::SealedReceipt;
use affidavit::crypto_trust_store::{checksum_for, KeyStoreFile, STORE_FORMAT};

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// The JSON object on stdout, tolerant of the framework's trailing "null".
fn stdout_json(output: &str) -> serde_json::Value {
    let end = output.rfind('}').expect("stdout carries a JSON object");
    serde_json::from_str(&output[..=end]).expect("stdout object parses")
}

/// Seed `<dir>/.affi/working.json` with one real operation-event (the exact
/// wire shape `chain::save_working` persists).
fn seed_working(dir: &TempDir) {
    use affidavit::ocel::{build_event, object_ref, SeqCounter};
    let mut counter = SeqCounter::new();
    let event = build_event(
        "attest.op",
        vec![object_ref("r1", "artifact")],
        b"real-signing-payload",
        &mut counter,
    )
    .expect("event is well-formed");
    let working = dir.path().join(".affi").join("working.json");
    fs::create_dir_all(working.parent().expect("working parent")).expect("create .affi");
    fs::write(
        &working,
        serde_json::to_vec(&vec![event]).expect("working serializes"),
    )
    .expect("seed working receipt");
}

/// Assemble through the REAL CLI into `receipt.json`.
fn assemble_receipt(dir: &TempDir) -> PathBuf {
    seed_working(dir);
    affi(dir)
        .args(["receipt", "assemble", "--out", "receipt.json"])
        .assert()
        .success();
    let receipt = dir.path().join("receipt.json");
    assert!(receipt.exists(), "assemble must produce receipt.json");
    receipt
}

/// The lowercase-hex rendering of a 32-byte secret fixed by `tag` — the raw
/// custody format the signing handlers accept (test/dev custody with real
/// bytes).
fn hex_secret(tag: u8) -> String {
    (0..32).map(|_| format!("{tag:02x}")).collect()
}

/// Write `tag`'s raw secret as a hex key FILE under `dir`.
fn write_key_file(dir: &TempDir, tag: u8) -> PathBuf {
    let key_file = dir.path().join("key.hex");
    fs::write(&key_file, hex_secret(tag)).expect("write raw secret hex file");
    key_file
}

/// The public registry record for the secret fixed by `tag` — built with the
/// same rendered primitives the handlers use, so the test-side record is the
/// handler-side record (byte-identical kid and fingerprint).
fn fixture_record(tag: u8) -> KeyRecord {
    let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    KeyRecord {
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
    }
}

/// Write `records` as a tamper-evident CTP-STORE-v1 store at
/// `<dir>/.affi/keys.json` (the FileKeyStore file law: format identity +
/// domain-separated checksum).
fn write_keystore(dir: &TempDir, records: &[KeyRecord]) -> PathBuf {
    let store = dir.path().join(".affi").join("keys.json");
    fs::create_dir_all(store.parent().expect("store parent")).expect("create .affi");
    let file = KeyStoreFile {
        format: STORE_FORMAT.to_string(),
        records: records.to_vec(),
        checksum: checksum_for(records).expect("records canonicalize"),
    };
    fs::write(
        &store,
        serde_json::to_vec_pretty(&file).expect("store file serializes"),
    )
    .expect("write key store");
    store
}

// ---------------------------------------------------------------------------
// receipt sign — the real ES256 seal
// ---------------------------------------------------------------------------

#[test]
fn sign_produces_a_sealed_receipt_that_envelope_verify_adjudicates_valid() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let key_file = write_key_file(&dir, 0x2A);

    // Human mode: the report names the real kid and the real standing.
    affi(&dir)
        .args([
            "receipt",
            "sign",
            receipt.to_str().expect("receipt path"),
            "--key-path",
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PQ-SEAL-v1"))
        .stdout(predicate::str::contains("kid afk1_"))
        .stdout(predicate::str::contains("standing VALID"));

    // The artifact is a REAL PQ-SEAL-v1 sealed receipt: base + envelope +
    // signature, envelope bound to the receipt's subject digest.
    let sealed: SealedReceipt =
        serde_json::from_str(&fs::read_to_string(dir.path().join("sealed.json")).expect("read"))
            .expect("sealed document parses as SealedReceipt");
    assert_eq!(sealed.envelope.algorithm, AlgorithmId::Es256);
    assert_eq!(sealed.envelope.audience, "affidavit.cli");
    assert!(!sealed.signature.is_empty(), "real signature bytes exist");
    assert_eq!(
        sealed.envelope.subject_digest,
        affidavit::crypto_trust_seal::subject_digest_of(&sealed.base)
            .expect("subject binding holds"),
        "the seal binds exactly the receipt it was offered with"
    );

    // JSON mode: the report keys mean something — signed true, ES256, and a
    // standing that comes from an inline engine verdict.
    let output = affi(&dir)
        .args([
            "receipt",
            "sign",
            receipt.to_str().expect("receipt path"),
            "--key-path",
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(report["signed"], serde_json::json!(true));
    assert_eq!(report["algorithm"], serde_json::json!("ES256"));
    assert_eq!(report["standing"], serde_json::json!("VALID"));
    assert_eq!(report["format"], serde_json::json!("PQ-SEAL-v1"));
    assert!(report["kid"]
        .as_str()
        .expect("kid is a string")
        .starts_with("afk1_"));
    assert_eq!(
        report["subject_digest"].as_str().expect("hex").len(),
        64,
        "subject digest is 32 bytes lowercase hex"
    );

    // Independent adjudication: the artifact verifies VALID under the fixture
    // key's public record in the tamper-evident store.
    write_keystore(&dir, &[fixture_record(0x2A)]);
    affi(&dir)
        .args([
            "envelope",
            "verify",
            "sealed.json",
            "--store",
            ".affi/keys.json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"standing\": \"VALID\""));
}

#[test]
fn sign_refuses_stdin_custody_bad_key_files_and_tampered_receipts() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let key_file = write_key_file(&dir, 0x2B);

    // Custody boundary: "-" (stdin) is refused — the plane never reads
    // secrets from a pipeline; production custody is non-exportable.
    affi(&dir)
        .args([
            "receipt",
            "sign",
            receipt.to_str().expect("receipt path"),
            "--key-path",
            "-",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"))
        .stderr(predicate::str::contains("never read from stdin"));

    // A key file that is not 64 hex characters is a typed refusal.
    let short_key = dir.path().join("short.hex");
    fs::write(&short_key, "aabb").expect("write short key");
    affi(&dir)
        .args([
            "receipt",
            "sign",
            receipt.to_str().expect("receipt path"),
            "--key-path",
            short_key.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .failure();
    assert!(
        !dir.path().join("sealed.json").exists(),
        "a refusal never writes the artifact"
    );

    // A tampered receipt never reaches a signature: the chain-recomputing
    // deserializer refuses it first.
    let original = fs::read_to_string(&receipt).expect("read receipt");
    let tampered = original.replace("attest.op", "attest.opX");
    assert_ne!(original, tampered, "tamper must mutate the receipt");
    fs::write(&receipt, tampered).expect("write tampered receipt");
    affi(&dir)
        .args([
            "receipt",
            "sign",
            receipt.to_str().expect("receipt path"),
            "--key-path",
            key_file.to_str().expect("key path"),
            "--out",
            "sealed.json",
        ])
        .assert()
        .failure();
}

// ---------------------------------------------------------------------------
// receipt notarize — request (unsigned, honest) vs attested (real seal)
// ---------------------------------------------------------------------------

#[test]
fn notarize_without_a_key_is_an_unsigned_attestation_request() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);

    let output = affi(&dir)
        .args(["receipt", "notarize", "receipt.json", "--format", "json"])
        .env("AFFI_NOTARY_KEY", "") // empty = unset: force request mode
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let sidecar = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));

    let notarization = &sidecar["notarization"];
    assert_eq!(
        notarization["status"],
        serde_json::json!("attestation_requested"),
        "no key: a request, not a proof"
    );
    assert_eq!(notarization["algorithm"], serde_json::json!("ES256"));
    assert_eq!(
        notarization["audience"],
        serde_json::json!("affidavit-notary-local")
    );
    assert!(
        notarization["standing"].is_null(),
        "an unsigned request carries no standing"
    );
    let note = notarization["note"].as_str().expect("note");
    assert!(note.contains("UNSIGNED"), "the note says no claim is made");
    assert!(
        note.contains("no external RFC 3161 TSA was contacted"),
        "the note says no external TSA was contacted"
    );
    // The request carries the real envelope document (a notary could sign it)
    // but NO signature anywhere.
    assert_eq!(
        sidecar["sealed"]["version"],
        serde_json::json!("CTP-ENVELOPE-v1")
    );
    assert!(
        sidecar["sealed"].get("signature").is_none(),
        "an unsigned request exposes no signature field"
    );
    assert_eq!(
        notarization["subject_digest"].as_str().expect("hex").len(),
        64
    );
    let _ = receipt;
}

#[test]
fn notarize_with_notary_key_attests_and_the_seal_verifies() {
    let dir = TempDir::new().expect("tempdir");
    assemble_receipt(&dir);

    let output = affi(&dir)
        .args([
            "receipt",
            "notarize",
            "receipt.json",
            "--out",
            "sidecar.json",
        ])
        .env("AFFI_NOTARY_KEY", hex_secret(0x5E))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let _ = output;

    let sidecar: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join("sidecar.json")).expect("read sidecar"),
    )
    .expect("sidecar parses");
    let notarization = &sidecar["notarization"];
    assert_eq!(notarization["status"], serde_json::json!("attested"));
    assert_eq!(notarization["standing"], serde_json::json!("VALID"));
    assert_eq!(notarization["algorithm"], serde_json::json!("ES256"));
    assert!(notarization["kid"]
        .as_str()
        .expect("kid")
        .starts_with("afk1_"));

    // The embedded seal is independently adjudicable: extract it and run the
    // real `envelope verify` against the fixture key's public record.
    let sealed_path = dir.path().join("extracted.json");
    fs::write(
        &sealed_path,
        serde_json::to_vec_pretty(&sidecar["sealed"]).expect("seal re-serializes"),
    )
    .expect("write extracted seal");
    let extracted: SealedReceipt =
        serde_json::from_str(&fs::read_to_string(&sealed_path).expect("read")).expect("parses");
    assert_eq!(
        extracted.envelope.audience, "affidavit-notary-local",
        "the attestation is bound to the notary audience"
    );

    write_keystore(&dir, &[fixture_record(0x5E)]);
    affi(&dir)
        .args([
            "envelope",
            "verify",
            "extracted.json",
            "--store",
            ".affi/keys.json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"standing\": \"VALID\""));
}

// ---------------------------------------------------------------------------
// receipt assemble-with-signature — real method, honest refusals
// ---------------------------------------------------------------------------

#[test]
fn assemble_with_signature_seals_via_key_path_env_and_verifies() {
    let dir = TempDir::new().expect("tempdir");
    seed_working(&dir);
    let key_file = write_key_file(&dir, 0x3C);

    let output = affi(&dir)
        .args([
            "receipt",
            "assemble-with-signature",
            "--signing-method",
            "affidavit-crypto-trust",
            "--format",
            "json",
        ])
        .env(
            "AFFI_SIGNING_KEY_PATH",
            key_file.to_str().expect("key path"),
        )
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(report["signed"], serde_json::json!(true));
    assert_eq!(
        report["signing_method"],
        serde_json::json!("affidavit-crypto-trust")
    );
    assert_eq!(report["algorithm"], serde_json::json!("ES256"));
    assert_eq!(report["standing"], serde_json::json!("VALID"));
    assert_eq!(report["format"], serde_json::json!("PQ-SEAL-v1"));

    // The sealed artifact exists next to the receipt and independently
    // adjudicates VALID.
    let sealed_path = dir
        .path()
        .join(report["sealed_path"].as_str().expect("sealed_path"));
    assert!(sealed_path.exists(), "sealed artifact written");
    let sealed: SealedReceipt =
        serde_json::from_str(&fs::read_to_string(&sealed_path).expect("read")).expect("parses");
    assert_eq!(sealed.envelope.audience, "affidavit.cli");

    write_keystore(&dir, &[fixture_record(0x3C)]);
    affi(&dir)
        .args([
            "envelope",
            "verify",
            sealed_path.to_str().expect("sealed path"),
            "--store",
            ".affi/keys.json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"standing\": \"VALID\""));
}

#[test]
fn assemble_with_signature_refuses_sigstore_default_and_unknown_methods() {
    let dir = TempDir::new().expect("tempdir");
    seed_working(&dir);

    // The rendered default is refused, loudly and truthfully.
    affi(&dir)
        .args(["receipt", "assemble-with-signature"])
        .env("AFFI_NOTARY_KEY", "") // no key could fake success; the METHOD refuses first
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"))
        .stderr(predicate::str::contains(
            "sigstore signing is not implemented",
        ))
        .stderr(predicate::str::contains("affidavit-crypto-trust"));

    // Any other method value is refused the same way.
    affi(&dir)
        .args([
            "receipt",
            "assemble-with-signature",
            "--signing-method",
            "transparent-counterfeit",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"));
}

#[test]
fn assemble_with_signature_without_a_key_is_r_missing_authority() {
    let dir = TempDir::new().expect("tempdir");
    seed_working(&dir);

    // The real method with NO key in the environment: typed refusal, exit 1.
    // Empty values count as absent (a variable set to nothing is no key).
    affi(&dir)
        .args([
            "receipt",
            "assemble-with-signature",
            "--signing-method",
            "affidavit-crypto-trust",
        ])
        .env("AFFI_NOTARY_KEY", "")
        .env("AFFI_SIGNING_KEY_PATH", "")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("R_missing_authority"))
        .stderr(predicate::str::contains("No key, no authority to sign"));
}

// ---------------------------------------------------------------------------
// receipt assemble-and-notarize — the composition of the two real paths
// ---------------------------------------------------------------------------

#[test]
fn assemble_and_notarize_composes_request_and_attested_paths() {
    // (a) Without a key: an honest unsigned request, "notarized": false.
    let dir = TempDir::new().expect("tempdir");
    seed_working(&dir);
    let output = affi(&dir)
        .args(["receipt", "assemble-and-notarize", "--format", "json"])
        .env("AFFI_NOTARY_KEY", "")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(report["notarized"], serde_json::json!(false));
    assert_eq!(report["status"], serde_json::json!("attestation_requested"));
    assert_eq!(
        report["notary"],
        serde_json::json!("affidavit-notary-local")
    );

    // (b) With the notary key: a real attestation, "notarized": true MEANS a
    // verified ES256 seal, and the sidecar artifact is on disk.
    let dir = TempDir::new().expect("tempdir");
    seed_working(&dir);
    let output = affi(&dir)
        .args(["receipt", "assemble-and-notarize", "--format", "json"])
        .env("AFFI_NOTARY_KEY", hex_secret(0x6F))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(report["notarized"], serde_json::json!(true));
    assert_eq!(report["status"], serde_json::json!("attested"));
    assert_eq!(report["standing"], serde_json::json!("VALID"));

    let sidecar_path = dir.path().join(format!(
        "{}.notarization.json",
        report["receipt_path"].as_str().expect("path")
    ));
    let sidecar: SealedReceipt = serde_json::from_str(
        &fs::read_to_string(&sidecar_path).expect("read notarization sidecar"),
    )
    .expect("sidecar is a verifiable sealed receipt");
    assert_eq!(sidecar.envelope.audience, "affidavit-notary-local");

    // (c) A provider value that is not implemented is refused.
    affi(&dir)
        .args([
            "receipt",
            "assemble-and-notarize",
            "--notary-provider",
            "rekor",
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"));
}

// ---------------------------------------------------------------------------
// keys + envelope on the FileKeyStore — the store law is user-facing
// ---------------------------------------------------------------------------

#[test]
fn keys_generate_and_list_round_trip_through_the_tamper_evident_store() {
    let dir = TempDir::new().expect("tempdir");

    affi(&dir)
        .args(["keys", "generate", "ES256", "alice", "--out", "keys.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("afk1_"))
        .stdout(predicate::str::contains("alice"));

    // The file IS the CTP-STORE-v1 law: format identity, records, checksum
    // over the records — recomputed independently here.
    let bytes = fs::read(dir.path().join("keys.json")).expect("store written");
    let file: KeyStoreFile = serde_json::from_slice(&bytes).expect("store parses as KeyStoreFile");
    assert_eq!(file.format, STORE_FORMAT);
    assert_eq!(file.records.len(), 1);
    assert_eq!(file.records[0].custodian.subject, "alice");
    assert_eq!(file.records[0].algorithm, AlgorithmId::Es256);
    assert_eq!(
        file.checksum,
        checksum_for(&file.records).expect("recompute"),
        "the on-disk checksum binds the stored records"
    );

    // A second generate appends (fresh keys never collide on id/fingerprint).
    affi(&dir)
        .args(["keys", "generate", "ES256", "bob", "--out", "keys.json"])
        .assert()
        .success();
    let bytes = fs::read(dir.path().join("keys.json")).expect("store reread");
    let file: KeyStoreFile = serde_json::from_slice(&bytes).expect("parses");
    assert_eq!(file.records.len(), 2);

    // list reads through the same law.
    affi(&dir)
        .args(["keys", "list", "--store", "keys.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alice"))
        .stdout(predicate::str::contains("bob"));

    // TEETH: flip one stored byte -> the checksum law refuses the listing.
    let intact = fs::read_to_string(dir.path().join("keys.json")).expect("read intact store");
    let tampered = intact.replacen("\"alice\"", "\"alixe\"", 1);
    assert_ne!(intact, tampered, "tamper must mutate a stored byte");
    fs::write(dir.path().join("keys.json"), tampered).expect("write tampered store");
    affi(&dir)
        .args(["keys", "list", "--store", "keys.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("checksum mismatch"));

    // Unknown algorithms are refused before any store mutation.
    let dir2 = TempDir::new().expect("tempdir");
    affi(&dir2)
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
    assert!(
        !dir2.path().join("keys.json").exists(),
        "a refusal never creates the store"
    );
}

#[test]
fn envelope_sign_and_verify_round_trip_through_the_file_keystore() {
    let dir = TempDir::new().expect("tempdir");
    let receipt = assemble_receipt(&dir);
    let key_file = write_key_file(&dir, 0x11);
    write_keystore(&dir, &[fixture_record(0x11)]);

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
            ".affi/keys.json",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"standing\": \"VALID\""));
}
