// CLI courts for the envelope listing/export surface (`affi envelope list`,
// `affi envelope export`, v26.9.28 wave 2 lane 5). Every test drives the REAL
// `affi` binary inside an isolated TempDir (the tests/crypto_trust_cli.rs and
// tests/crypto_trust_keys_cli.rs pattern) against a real assembled receipt, a
// real ES256 fixture key, and a real checksummed FileKeyStore — no mocks. The
// export's fidelity claim is proven by re-import: the emitted json form must
// re-parse to exactly the sealed envelope, and the emitted sa2a form must
// bridge back through `approval_to_envelope` to exactly the sealed envelope
// AND be signable/verifiable in the SA2A domain with the sealing key — while
// the envelope-domain signature is REFUSED over the SA2A pre-image (domain
// separation witnessed on the exported artifact, not asserted from prose).
//
// FEATURE GATE: the handlers carry `#[cfg(feature = "crypto-trust")]`. Run:
//
//     cargo test --features crypto-trust --test crypto_trust_envelope_cli
//
// DISPATCH NOTE (the designed boundary): the two verbs ride the rendered
// wrappers (src/verbs/{envelope_list,envelope_export}.rs + their `pub mod`
// lines in src/verbs/mod.rs), which ggen sync renders from the wave-2 lane-5
// ontology block (ontology/affi-cli.ttl, patch proposal
// /tmp/w5-w2-lane5/ontology_append.ttl). Until that sync lands, the binary
// refuses these invocations — so this file COMPILES against the current tree
// but every test dispatch-fails. These tests are the acceptance gate for that
// sync; teeth first.
#![cfg(feature = "crypto-trust")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

use affidavit::crypto_trust_envelope::SignatureEnvelope;
use affidavit::crypto_trust_es256::{verify_es256, Es256SigningKey};
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CustodianIdentity, KeyId, KeyOrigin, KeyRecord,
    PublicKeyMaterial,
};
use affidavit::crypto_trust_sa2a::{
    approval_to_envelope, sa2a_signing_input_checked, Sa2aApproval, SA2A_APPROVAL_TAG,
};
use affidavit::crypto_trust_seal::SealedReceipt;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// The stdout document as text (tolerant of the framework's trailing "null"):
/// everything up to the last `}` is the document.
fn stdout_text(output: Vec<u8>) -> String {
    let text = String::from_utf8(output).expect("utf8 stdout");
    let end = text.rfind('}').expect("stdout carries a JSON document");
    text[..=end].to_string()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Assemble one honest receipt in `dir` (the crypto_trust_cli lifecycle, 1
/// event), returning the receipt path.
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
        .write_stdin("envelope-cli-payload")
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

/// A real fixture key (fixed seed) + its raw-hex secret file — the custody
/// seam `envelope sign` consumes (the crypto_trust_cli.rs fixture shape).
fn fixture_key(dir: &TempDir, tag: u8) -> (std::path::PathBuf, KeyRecord, Es256SigningKey) {
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
    (key_file, record, signing)
}

/// Write `record` as a one-row key store at `.affi/keys.json` under `dir`
/// (the CTP-STORE-v1 law: records carry the domain-separated checksum).
fn write_store(dir: &TempDir, record: &KeyRecord) -> std::path::PathBuf {
    let store = dir.path().join(".affi").join("keys.json");
    fs::create_dir_all(store.parent().expect("store parent")).expect("create .affi");
    let records = vec![record.clone()];
    let checksum = affidavit::crypto_trust_store::checksum_for(&records);
    let file = affidavit::crypto_trust_store::KeyStoreFile {
        format: affidavit::crypto_trust_store::STORE_FORMAT.to_string(),
        records,
        checksum: checksum.expect("fixture records checksum"),
    };
    fs::write(&store, serde_json::to_vec(&file).expect("store serializes"))
        .expect("write key store");
    store
}

/// The full honest sealing flow: assemble a receipt, seal it under the
/// fixture key, return the sealed document parsed test-side plus its path.
fn sealed_document(dir: &TempDir, tag: u8) -> (SealedReceipt, std::path::PathBuf) {
    let receipt = assemble_receipt(dir);
    let (key_file, record, _signing) = fixture_key(dir, tag);
    write_store(dir, &record);
    affi(dir)
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
    let sealed_path = dir.path().join("sealed.json");
    let sealed: SealedReceipt =
        serde_json::from_str(&fs::read_to_string(&sealed_path).expect("sealed readable"))
            .expect("sealed parses");
    (sealed, sealed_path)
}

// -- teeth first: the honest round trips ---------------------------------------

#[test]
fn envelope_list_shows_sealed_key_records_and_is_read_only() {
    let dir = TempDir::new().expect("tempdir");
    let (_key_file, record, _signing) = fixture_key(&dir, 1);
    let store = write_store(&dir, &record);
    let store_before = fs::read(&store).expect("store readable");

    let output = affi(&dir)
        .args([
            "envelope",
            "list",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listing = String::from_utf8(output).expect("utf8 listing");
    // kid, algorithm, PROFILE (the graph law profile = f(algorithm)),
    // fingerprint, custodian — one row per record.
    assert!(
        listing.contains(&record.id.to_string()),
        "kid listed: {listing}"
    );
    assert!(listing.contains("ES256"), "algorithm listed: {listing}");
    assert!(listing.contains("CLASSICAL"), "profile listed: {listing}");
    assert!(
        listing.contains(&record.fingerprint.as_hex()),
        "fingerprint listed: {listing}"
    );
    assert!(
        listing.contains("fixture-custodian"),
        "custodian listed: {listing}"
    );
    // Read-only law: the listing must not mutate the store it lists.
    assert_eq!(
        fs::read(&store).expect("store readable after list"),
        store_before,
        "envelope list must not write the store"
    );
}

#[test]
fn envelope_list_empty_store_is_quiet_success() {
    let dir = TempDir::new().expect("tempdir");
    // An absent store file is an empty store under the store law — listed as
    // such (stderr), exit success, and the run must not create a store file.
    affi(&dir)
        .args(["envelope", "list", "--store", "absent.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no keys registered"));
    assert!(
        !dir.path().join("absent.json").exists(),
        "listing must not create the store"
    );
}

#[test]
fn envelope_export_json_reimports_to_the_exact_sealed_envelope() {
    let dir = TempDir::new().expect("tempdir");
    let (sealed, _sealed_path) = sealed_document(&dir, 1);

    let output = affi(&dir)
        .args(["envelope", "export", "sealed.json", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let exported_text = stdout_text(output);
    let document: serde_json::Value =
        serde_json::from_str(&exported_text).expect("stdout document parses");
    assert_eq!(
        document["version"].as_str(),
        Some("CTP-ENVELOPE-v1"),
        "the json export is the CTP-ENVELOPE-v1 envelope document"
    );

    // The fidelity proof: the emitted document re-imports through the
    // envelope's own admission law (from_bytes) to EXACTLY the sealed
    // envelope.
    let restored = SignatureEnvelope::from_bytes(exported_text.as_bytes())
        .expect("exported document re-imports");
    assert_eq!(restored, sealed.envelope, "json export round-trips exactly");
}

#[test]
fn envelope_export_default_format_is_json_identity() {
    let dir = TempDir::new().expect("tempdir");
    let (sealed, _sealed_path) = sealed_document(&dir, 6);

    let output = affi(&dir)
        .args(["envelope", "export", "sealed.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let exported_text = stdout_text(output);
    let restored = SignatureEnvelope::from_bytes(exported_text.as_bytes())
        .expect("exported document re-imports");
    assert_eq!(restored, sealed.envelope);
}

#[test]
fn envelope_export_sa2a_verifies_through_the_sa2a_law() {
    let dir = TempDir::new().expect("tempdir");
    let (sealed, _sealed_path) = sealed_document(&dir, 2);
    let (_key_file, record, signing) = fixture_key(&dir, 2);

    let output = affi(&dir)
        .args(["envelope", "export", "sealed.json", "--format", "sa2a"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let exported_text = stdout_text(output);
    let approval: Sa2aApproval =
        serde_json::from_str(&exported_text).expect("the sa2a export is the approval document");

    // The errata field mapping, field by field (the fields the SA2A side
    // consumes): derived schema version, algorithm, key id, hex fields.
    assert_eq!(approval.v, "1");
    assert_eq!(approval.alg, "ES256");
    assert_eq!(approval.kid, record.id.to_string());
    assert_eq!(
        approval.effect_digest,
        hex_encode(&sealed.envelope.subject_digest),
        "effect_digest is the envelope subject digest, lowercase hex"
    );
    assert_eq!(
        approval.nonce,
        hex_encode(&sealed.envelope.nonce),
        "nonce is the envelope nonce, lowercase hex"
    );

    // The faithful re-import: the exported approval bridges back through the
    // rendered inverse mapping to EXACTLY the sealed envelope.
    let restored = approval_to_envelope(&approval).expect("exported approval bridges back");
    assert_eq!(restored, sealed.envelope, "sa2a export round-trips exactly");

    // The approval tag: the SA2A signed message framed from the exported
    // document is `SA2A-C2-APPROVAL-v1 || 0x00 || JCS(document)`.
    let sa2a_input = sa2a_signing_input_checked(&approval).expect("exported approval frames");
    assert!(
        sa2a_input.starts_with(SA2A_APPROVAL_TAG.as_bytes()),
        "the SA2A pre-image carries the approval tag"
    );
    assert_eq!(
        sa2a_input[SA2A_APPROVAL_TAG.len()],
        0x00,
        "the tag is NUL-framed"
    );

    // The verify path: the SAME sealing key signs the exported SA2A
    // pre-image and it verifies — the export is really signable in the SA2A
    // domain.
    let pk = match &record.public_key {
        PublicKeyMaterial::Es256Sec1(bytes) => bytes.clone(),
        other => panic!("fixture key is ES256 SEC1, got {other:?}"),
    };
    let sa2a_signature = signing.sign(&sa2a_input);
    assert!(
        matches!(verify_es256(&pk, &sa2a_input, &sa2a_signature), Ok(true)),
        "the sealing key verifies over the exported SA2A pre-image"
    );
    // FALSIFIER (domain separation on the exported artifact): the
    // envelope-domain signature from the sealed document does NOT verify over
    // the SA2A pre-image — the two wire domains are different bytes.
    assert!(
        matches!(verify_es256(&pk, &sa2a_input, &sealed.signature), Ok(false)),
        "the envelope-domain signature must be refused over the SA2A pre-image"
    );
}

// -- every refusal witnessed (exact exit code + message) -----------------------

#[test]
fn envelope_export_refuses_unknown_format_typed() {
    let dir = TempDir::new().expect("tempdir");
    let (_sealed, _sealed_path) = sealed_document(&dir, 3);

    affi(&dir)
        .args(["envelope", "export", "sealed.json", "--format", "pem"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"))
        .stderr(predicate::str::contains("pem"))
        .stderr(predicate::str::contains("json"))
        .stderr(predicate::str::contains("sa2a"));
}

#[test]
fn envelope_export_refuses_tampered_sealed_document() {
    let dir = TempDir::new().expect("tempdir");
    let (_sealed, sealed_path) = sealed_document(&dir, 4);

    // Tamper the BASE inside the sealed document: the chain law fires inside
    // deserialization — the base never becomes a SealedReceipt value, so the
    // export refuses before any wire form exists.
    let text = fs::read_to_string(&sealed_path).expect("read sealed");
    let tampered = text.replace("attest.op", "attest.opX");
    assert_ne!(text, tampered, "tamper must mutate the base event_type");
    fs::write(&sealed_path, tampered).expect("write tampered seal");

    affi(&dir)
        .args(["envelope", "export", "sealed.json", "--format", "json"])
        .assert()
        .failure()
        .code(1)
        // Anti-vacuity: the refusal must be the chain law inside
        // deserialization (a JSON error), not the pre-sync dispatch error —
        // this assertion fails while the verb is unrendered.
        .stderr(predicate::str::contains("JSON error"));
}

#[test]
fn envelope_list_refuses_tampered_store() {
    let dir = TempDir::new().expect("tempdir");
    let (_key_file, record, _signing) = fixture_key(&dir, 5);
    let store = write_store(&dir, &record);

    // Flip one record byte (custodian subject): the store law's checksum
    // fires — a tampered store is a typed refusal, never a silent listing.
    let intact = fs::read_to_string(&store).expect("store readable");
    let tampered = intact.replacen("fixture-custodian", "fixture-custodiaN", 1);
    assert_ne!(tampered, intact, "exactly one byte flipped");
    fs::write(&store, tampered).expect("write tampered store");

    affi(&dir)
        .args([
            "envelope",
            "list",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("checksum mismatch"));
}
