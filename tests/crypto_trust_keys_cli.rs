// CLI courts for the keys lifecycle surface (`affi keys import|revoke|rotate`,
// v26.9.28 wave 1 lane 4). Every test drives the REAL `affi` binary inside an
// isolated TempDir (the tests/crypto_trust_real_signing.rs pattern) against a
// real FileKeyStore — no mocks: the imported keys are real external public
// material, the rotation record is verified by recomputing the rendered
// rotation pre-image and running the real ES256 verifier, and every refusal
// variant is witnessed by exact exit-code + message assertions.
//
// FEATURE GATE: the handlers carry `#[cfg(feature = "crypto-trust")]`. Run:
//
//     cargo test --features crypto-trust --test crypto_trust_keys_cli
//
// DISPATCH NOTE: the three verbs ride the rendered wrappers
// (src/verbs/{keys_import,keys_revoke,keys_rotate}.rs + their `pub mod` lines
// in src/verbs/mod.rs), which ggen sync renders from the ontology block this
// lane added. Until that sync lands, the binary refuses these invocations —
// these tests are the acceptance gate for that sync.
#![cfg(feature = "crypto-trust")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use affidavit::crypto_trust_es256::{verify_es256, Es256SigningKey};
use affidavit::crypto_trust_keys::{fingerprint_public_key, AlgorithmId, PublicKeyMaterial};
use affidavit::crypto_trust_rotation::{
    RotationRecord, ROTATION_DIGEST_LABEL, ROTATION_DOMAIN_TAG,
};
use affidavit::crypto_trust_store::FileKeyStore;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

/// The JSON object on stdout, tolerant of the framework's trailing "null"
/// (the tests/cli_dispatch.rs pattern).
fn stdout_json(output: &str) -> serde_json::Value {
    let end = output.rfind('}').expect("stdout carries a JSON object");
    serde_json::from_str(&output[..=end]).expect("stdout object parses")
}

/// The public registry record of a real external key fixed by `tag` (SEC1
/// hex), derived from a fixed seed so the expected kid/fingerprint are
/// computable test-side.
fn external_pk_hex(tag: u8) -> String {
    let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
    hex_encode(&signing.public_key_sec1())
}

fn es256_kid_of(tag: u8) -> String {
    let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    affidavit::crypto_trust_keys::KeyId::from_fingerprint(&fingerprint_public_key(
        AlgorithmId::Es256,
        &public,
    ))
    .to_string()
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

/// `affi keys generate ES256 <custodian> --out <store>`; returns the stdout
/// JSON (kid + fingerprint).
fn generate_key(dir: &TempDir, custodian: &str, store: &Path) -> serde_json::Value {
    let output = affi(dir)
        .args([
            "keys",
            "generate",
            "ES256",
            custodian,
            "--out",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    stdout_json(&String::from_utf8(output).expect("utf8 stdout"))
}

/// The revocation sidecar path the handler derives for a store at `store`
/// (`revocations.json` beside the store file; `./revocations.json` when the
/// store has no parent component).
fn sidecar_path(dir: &TempDir) -> PathBuf {
    dir.path().join("revocations.json")
}

// -- teeth first: the honest round trips --------------------------------------

#[test]
fn import_then_list_shows_generated_and_imported() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");

    let generated = generate_key(&dir, "alice", &store);
    let generated_kid = generated["kid"]
        .as_str()
        .expect("kid on stdout")
        .to_string();

    // Import a REAL externally-held ES256 public key (SEC1 hex); the kid must
    // equal the fingerprint-derived id computed test-side.
    let imported = {
        let output = affi(&dir)
            .args([
                "keys",
                "import",
                "ES256",
                &external_pk_hex(0x42),
                "external-custodian",
                "--out",
                store.to_str().expect("utf8 store path"),
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        stdout_json(&String::from_utf8(output).expect("utf8 stdout"))
    };
    assert_eq!(
        imported["kid"].as_str().expect("imported kid"),
        es256_kid_of(0x42),
        "imported kid is the fingerprint of the external key"
    );
    assert_eq!(
        imported["fingerprint"].as_str().expect("fingerprint").len(),
        64
    );
    assert_eq!(imported["origin"].as_str().expect("origin"), "IMPORTED");

    // Both records list through the store law.
    let listing = affi(&dir)
        .args([
            "keys",
            "list",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let listing = String::from_utf8(listing).expect("utf8 listing");
    assert!(
        listing.contains(&generated_kid),
        "generated kid listed: {listing}"
    );
    assert!(
        listing.contains(&es256_kid_of(0x42)),
        "imported kid listed: {listing}"
    );
    assert!(listing.contains("external-custodian"));
}

#[test]
fn revoke_appends_checksummed_sidecar_entry() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let generated = generate_key(&dir, "alice", &store);
    let kid = generated["kid"].as_str().expect("kid").to_string();

    let output = affi(&dir)
        .args([
            "keys",
            "revoke",
            &kid,
            "compromised",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(report["revoked"].as_bool(), Some(true));
    assert_eq!(report["kid"].as_str(), Some(kid.as_str()));
    assert!(
        report["revoked_at"].as_u64().expect("real epoch") > 1_700_000_000,
        "revoked_at is a real system epoch, not a literal"
    );
    assert_eq!(report["reason"].as_str(), Some("compromised"));

    // The sidecar ledger exists beside the store, carries the entry, and is
    // checksummed under the store law's format identity.
    let sidecar = fs::read_to_string(sidecar_path(&dir)).expect("sidecar readable");
    let file: serde_json::Value = serde_json::from_str(&sidecar).expect("sidecar parses");
    assert_eq!(file["format"].as_str(), Some("CTP-REVOCATIONS-v1"));
    let entries = file["entries"].as_array().expect("entries array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["kid"].as_str(), Some(kid.as_str()));
    assert_eq!(entries[0]["reason"].as_str(), Some("compromised"));
    assert!(!file["checksum"].as_str().expect("checksum").is_empty());
}

#[test]
fn rotate_writes_successor_signed_verifiable_record() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let generated = generate_key(&dir, "alice", &store);
    let old_kid = generated["kid"].as_str().expect("kid").to_string();
    let rotation_path = dir.path().join("rotation.json");

    let output = affi(&dir)
        .args([
            "keys",
            "rotate",
            &old_kid,
            "--store",
            store.to_str().expect("utf8 store path"),
            "--out",
            rotation_path.to_str().expect("utf8 rotation path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(report["rotated_from"].as_str(), Some(old_kid.as_str()));
    let new_kid = report["rotated_to"].as_str().expect("new kid").to_string();
    assert_ne!(new_kid, old_kid, "the successor is a freshly generated key");

    // Store: old record stays, the successor joined as Generated.
    let records = FileKeyStore::open(&store)
        .expect("store opens under the full law")
        .records()
        .expect("store records");
    assert_eq!(records.len(), 2, "old record stays; successor joined");
    let successor = records
        .iter()
        .find(|r| r.id.0 == new_kid)
        .expect("successor record in store");
    let successor_pk = match &successor.public_key {
        PublicKeyMaterial::Es256Sec1(bytes) => bytes.clone(),
        other => panic!("successor is ES256 SEC1, got {other:?}"),
    };

    // The artifact parses as the rendered RotationRecord type.
    let record: RotationRecord =
        serde_json::from_str(&fs::read_to_string(&rotation_path).expect("rotation readable"))
            .expect("rotation record parses");
    assert_eq!(record.old_key_id, old_kid);
    assert_eq!(record.new_key_id, new_kid);
    assert_eq!(record.from_profile, "CLASSICAL");
    assert_eq!(record.to_profile, "CLASSICAL");

    // Verify: recompute the rendered module's documented pre-image
    // (JCS of the five signed fields, domain-digested under the rotation
    // label) and run the real ES256 verifier against the successor's SEC1
    // public key from the store.
    let signed_fields = serde_json::json!({
        "old_key_id": record.old_key_id,
        "new_key_id": record.new_key_id,
        "from_profile": record.from_profile,
        "to_profile": record.to_profile,
        "rotated_at": record.rotated_at,
    });
    let canonical =
        affidavit::crypto_trust_canonical::jcs(&signed_fields).expect("jcs canonicalization");
    let record_digest = affidavit::crypto_trust_canonical::digest(
        ROTATION_DOMAIN_TAG,
        &[ROTATION_DIGEST_LABEL, canonical.as_bytes()],
    );
    assert!(
        verify_es256(&successor_pk, &record_digest, &record.successor_signature)
            .expect("well-formed signature"),
        "the successor's signature over the rendered pre-image must verify"
    );
}

// -- every refusal variant witnessed (exact exit code + message) ---------------

#[test]
fn import_refusal_variants_witnessed() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let store_arg = ["--out", store.to_str().expect("utf8 store")];

    // Bad hex: Parse refusal naming the offending character.
    affi(&dir)
        .args(["keys", "import", "ES256", "04zz", "alice"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("Parse error"))
        .stderr(predicate::str::contains("not hex"));

    // Odd-length hex: Parse refusal.
    affi(&dir)
        .args(["keys", "import", "ES256", &"0".repeat(129), "alice"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("odd length"));

    // Length mismatch: Validation naming the algorithm's expected length.
    affi(&dir)
        .args(["keys", "import", "ES256", &"04".repeat(32), "alice"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("length mismatch for ES256"))
        .stderr(predicate::str::contains("expected 65 bytes"))
        .stderr(predicate::str::contains("got 32 bytes"));

    // Unknown algorithm: REFUSED_UNSUPPORTED naming the admitted set.
    affi(&dir)
        .args(["keys", "import", "ED25519", "04a5", "alice"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"));

    // Duplicate import: the store law's exact Registry::Duplicate passthrough.
    generate_key(&dir, "seed", &dir.path().join("keys.json"));
    let kid = es256_kid_of(0x42);
    let import = |dir: &TempDir| {
        affi(dir)
            .args([
                "keys",
                "import",
                "ES256",
                &external_pk_hex(0x42),
                "dup-custodian",
            ])
            .args(store_arg)
            .assert()
    };
    import(&dir).success();
    import(&dir)
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "key refused: key registry: duplicate key",
        ))
        .stderr(predicate::str::contains(&kid));
}

#[test]
fn revoke_refusal_variants_witnessed() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let store_arg = ["--store", store.to_str().expect("utf8 store")];

    // Unknown kid: typed refusal naming the key and the store.
    affi(&dir)
        .args(["keys", "revoke", "afk1_0000000000000000", "compromised"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "unknown key afk1_0000000000000000",
        ));

    // A tampered sidecar refuses the next revoke with the checksum mismatch.
    let generated = generate_key(&dir, "alice", &store);
    let kid = generated["kid"].as_str().expect("kid").to_string();
    affi(&dir)
        .args(["keys", "revoke", &kid, "first"])
        .args(store_arg)
        .assert()
        .success();
    let sidecar_file = sidecar_path(&dir);
    let intact = fs::read_to_string(&sidecar_file).expect("sidecar readable");
    let tampered = intact.replacen("\"first\"", "\"firSt\"", 1);
    assert_ne!(tampered, intact, "exactly one byte flipped");
    fs::write(&sidecar_file, tampered).expect("write tampered sidecar");
    affi(&dir)
        .args(["keys", "revoke", &kid, "second"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("checksum mismatch"));
}

#[test]
fn rotate_refusal_variants_witnessed() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let rotation = dir.path().join("rotation.json");
    let store_arg = [
        "--store",
        store.to_str().expect("utf8 store"),
        "--out",
        rotation.to_str().expect("utf8 rot"),
    ];

    // Unknown kid.
    affi(&dir)
        .args(["keys", "rotate", "afk1_0000000000000000"])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "unknown key afk1_0000000000000000",
        ));

    // A registered ML-DSA-65 key is not an ES256 rotation subject.
    let mldsa_hex = hex_encode(&vec![
        7u8;
        AlgorithmId::MlDsa65.public_key_len().unwrap_or(1952)
    ]);
    let output = affi(&dir)
        .args(["keys", "import", "ML-DSA-65", &mldsa_hex, "pqc-custodian"])
        .args(["--out", store.to_str().expect("utf8 store")])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let imported = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    let pqc_kid = imported["kid"].as_str().expect("imported kid").to_string();
    affi(&dir)
        .args(["keys", "rotate", &pqc_kid])
        .args(store_arg)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("REFUSED_UNSUPPORTED"))
        .stderr(predicate::str::contains("requires an ES256 key"))
        .stderr(predicate::str::contains("ML-DSA-65"));
}
