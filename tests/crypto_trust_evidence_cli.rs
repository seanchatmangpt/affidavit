// CLI courts for the evidence surface (`affi evidence
// journal|crl-publish|crl-apply|heads`, v26.9.28 wave 2 lane 1). Every test
// drives the REAL `affi` binary inside an isolated TempDir (the
// tests/crypto_trust_keys_cli.rs pattern) against a real ChainAssembler
// receipt, a real raw-hex custody file, a real checksummed FileKeyStore, and a
// real revocation sidecar — no mocks: the journaled standing is the verdict of
// a real VerificationEngine, the published CRL is re-verified test-side with
// the rendered `verify_publication`, and every refusal variant is witnessed by
// exact exit-code + message assertions.
//
// FEATURE GATE: the handlers carry `#[cfg(feature = "crypto-trust")]`. Run:
//
//     cargo test --features crypto-trust --test crypto_trust_evidence_cli
//
// DISPATCH NOTE: the four verbs ride the rendered wrappers
// (src/verbs/{evidence_journal,evidence_crl_publish,evidence_crl_apply,
// evidence_heads}.rs + their `pub mod` lines in src/verbs/mod.rs), which the
// lane pre-landed render-faithfully from the ontology block
// (ontology/affi-cli.ttl, EVIDENCE NOUN). The next `ggen sync run` must render
// these files byte-identically; where it does not, the render wins.
#![cfg(feature = "crypto-trust")]

use assert_cmd::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_journal::StandingJournal;
use affidavit::crypto_trust_keys::{fingerprint_public_key, AlgorithmId, PublicKeyMaterial};
use affidavit::crypto_trust_revocation::{verify_publication, SignedRevocationList};

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

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
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

/// Writes a real raw 32-byte-hex custody file for the key fixed by `tag` —
/// the documented test/dev custody source the evidence verbs accept via
/// `AFFI_SIGNING_KEY_PATH`.
fn custody_file(dir: &TempDir, tag: u8) -> PathBuf {
    let path = dir.path().join(format!("key-{tag:02x}.hex"));
    std::fs::write(&path, hex_encode(&[tag; 32])).expect("write custody file");
    path
}

fn run_journal(dir: &TempDir, custody: &Path, subject: &str) -> serde_json::Value {
    let output = affi(dir)
        .env("AFFI_SIGNING_KEY_PATH", custody)
        .args(["evidence", "journal", subject])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    stdout_json(&String::from_utf8(output).expect("utf8 stdout"))
}

/// Imports a real external ES256 public key for `tag` into `store`.
fn import_key(dir: &TempDir, tag: u8, store: &Path) -> serde_json::Value {
    let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
    let output = affi(dir)
        .args([
            "keys",
            "import",
            "ES256",
            &hex_encode(&signing.public_key_sec1()),
            &format!("custodian-{tag:02x}"),
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

// -- teeth first: the honest round trips --------------------------------------

#[test]
fn journal_records_chains_and_heads_audits() {
    let dir = TempDir::new().expect("tempdir");
    let custody = custody_file(&dir, 0x51);

    // First record: seq 0, genesis prev, VALID standing, real 64-hex head.
    let first = run_journal(&dir, &custody, "release-v26.9.28");
    assert_eq!(first["seq"].as_u64(), Some(0));
    assert_eq!(
        first["prev"].as_str().expect("prev"),
        affidavit::crypto_trust_journal::JOURNAL_GENESIS
    );
    assert_eq!(first["standing"].as_str().expect("standing"), "VALID");
    assert_eq!(first["head"].as_str().expect("head").len(), 64);
    assert_eq!(first["entries"].as_u64(), Some(1));
    assert_eq!(
        first["kid"].as_str().expect("kid"),
        es256_kid_of(0x51),
        "the journaled key is the custody key"
    );

    // The durable journal exists at the graph path.
    let journal_path = dir.path().join(".affi/standing-journal.jsonl");
    assert!(journal_path.is_file(), "journal materializes at .affi/");

    // Second record chains over the first.
    let second = run_journal(&dir, &custody, "deploy-v26.9.28");
    assert_eq!(second["seq"].as_u64(), Some(1));
    assert_eq!(second["prev"].as_str().expect("prev"), first["head"]);

    // The durable journal now holds both entries and reproduces.
    let journal_path = dir.path().join(".affi/standing-journal.jsonl");
    let loaded =
        StandingJournal::from_jsonl(&std::fs::read_to_string(&journal_path).expect("read journal"))
            .expect("honest journal reproduces");
    assert_eq!(loaded.len(), 2);

    // Heads: audit consistent, leaf set re-derived, head SIGNED under custody.
    let output = affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args(["evidence", "heads"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let heads = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(heads["entries"].as_u64(), Some(2));
    assert_eq!(heads["log_leaves"].as_u64(), Some(2));
    assert_eq!(heads["consistent"].as_bool(), Some(true));
    assert_eq!(heads["head_signed"].as_bool(), Some(true));
    assert_eq!(
        heads["head_kid"].as_str().expect("head kid"),
        es256_kid_of(0x51)
    );
    // The head is exactly the RFC 9162 head over the re-derived leaf set
    // (leaf i = BLAKE3 over entry i's envelope_commitment hex).
    let mut log = affidavit::crypto_trust_transparency::TransparencyLog::new();
    for entry in loaded.entries() {
        log.append(blake3::hash(entry.envelope_commitment.as_bytes()).into());
    }
    assert_eq!(
        heads["head"].as_str().expect("head"),
        hex_encode(&log.head())
    );
}

#[test]
fn journal_refuses_missing_custody_and_tampered_journal() {
    let dir = TempDir::new().expect("tempdir");
    let custody = custody_file(&dir, 0x52);

    // No custody: the typed no-authority refusal, non-zero exit.
    affi(&dir)
        .args(["evidence", "journal", "subject"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("REFUSED_R_missing_authority"));

    // An empty custody value counts as absent (it carries no secret).
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", "")
        .args(["evidence", "journal", "subject"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("REFUSED_R_missing_authority"));

    // Empty subject: Validation refusal.
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args(["evidence", "journal", ""])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("non-empty"));

    // Record honestly, then tamper with the durable chain: the next run
    // refuses — the load path re-verifies every byte from genesis.
    run_journal(&dir, &custody, "tamper-target");
    let journal_path = dir.path().join(".affi/standing-journal.jsonl");
    let intact = std::fs::read_to_string(&journal_path).expect("read journal");
    let tampered = intact.replace("VALID", "INVALID");
    assert_ne!(tampered, intact);
    std::fs::write(&journal_path, tampered).expect("write tampered journal");
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args(["evidence", "journal", "after-tamper"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("does not reproduce"));

    // Heads refuses the tampered journal too.
    affi(&dir)
        .args(["evidence", "heads"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("does not reproduce"));
}

#[test]
fn crl_publish_then_apply_round_trips_the_revocation_state() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let issuer = import_key(&dir, 0x53, &store);
    let issuer_kid = issuer["kid"].as_str().expect("kid").to_string();
    let revoked_kid = import_key(&dir, 0x54, &store)["kid"]
        .as_str()
        .expect("kid")
        .to_string();

    // Revoke the second key through the keys lane's own verb.
    affi(&dir)
        .args([
            "keys",
            "revoke",
            &revoked_kid,
            "compromised",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success();

    // Publish under the issuer: the sidecar's revocation state is mirrored,
    // signed, and written to --out.
    let custody = custody_file(&dir, 0x53);
    let crl_path = dir.path().join("crl.json");
    let output = affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args([
            "evidence",
            "crl-publish",
            &issuer_kid,
            "7",
            "--store",
            store.to_str().expect("utf8 store path"),
            "--out",
            crl_path.to_str().expect("utf8 crl path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let published = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(
        published["issuer_kid"].as_str().expect("issuer"),
        issuer_kid
    );
    assert_eq!(published["epoch"].as_u64(), Some(7));
    assert_eq!(published["records"].as_u64(), Some(1));
    assert_eq!(published["format"].as_str().expect("format"), "CTP-CRL-v1");
    assert!(crl_path.is_file(), "publication file created");

    // Test-side fidelity: the file parses as the rendered publication and
    // verifies under the issuer's public key.
    let signing = Es256SigningKey::from_seed(&[0x53; 32]).expect("fixture scalar");
    let crl: SignedRevocationList =
        serde_json::from_str(&std::fs::read_to_string(&crl_path).expect("read crl"))
            .expect("publication parses");
    assert_eq!(crl.epoch, 7);
    assert_eq!(crl.revoked.len(), 1);
    assert_eq!(crl.revoked[0].kid, revoked_kid);
    assert!(
        verify_publication(&crl, &signing.public_key_sec1()).expect("verification runs"),
        "the published bytes must verify under the issuer key"
    );

    // Admit the publication: signature first, then freshness, then merge.
    let output = affi(&dir)
        .args([
            "evidence",
            "crl-apply",
            crl_path.to_str().expect("utf8 crl path"),
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let applied = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(applied["applied"].as_u64(), Some(1));
    assert_eq!(applied["issuer_kid"].as_str().expect("issuer"), issuer_kid);
    assert_eq!(applied["crl_epoch"].as_u64(), Some(7));
    assert!(
        applied["new_epoch"].as_u64().expect("new epoch") > 0,
        "the merged revocation advances the epoch clock"
    );

    // Re-applying the same publication is lawful (idempotent state-wise).
    affi(&dir)
        .args([
            "evidence",
            "crl-apply",
            crl_path.to_str().expect("utf8 crl path"),
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .success();
}

#[test]
fn crl_publish_refusals_are_typed() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let issuer_kid = import_key(&dir, 0x55, &store)["kid"]
        .as_str()
        .expect("kid")
        .to_string();
    let custody = custody_file(&dir, 0x55);

    // No custody: typed no-authority refusal, and NO artifact may appear.
    affi(&dir)
        .args([
            "evidence",
            "crl-publish",
            &issuer_kid,
            "0",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("REFUSED_R_missing_authority"));
    assert!(
        !dir.path().join(".affi/crl.json").exists(),
        "no default-path artifact behind a refusal"
    );

    // Unknown issuer kid.
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args([
            "evidence",
            "crl-publish",
            "afk1_0000000000000000",
            "0",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "unknown key afk1_0000000000000000",
        ));

    // Custody mismatch: the file holds a DIFFERENT key than the issuer of
    // record — refused, never silently re-attributed.
    let wrong_custody = custody_file(&dir, 0x56);
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &wrong_custody)
        .args([
            "evidence",
            "crl-publish",
            &issuer_kid,
            "0",
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("not the requested issuer"));

    // Missing store.
    let missing = dir.path().join("missing-keys.json");
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args([
            "evidence",
            "crl-publish",
            &issuer_kid,
            "0",
            "--store",
            missing.to_str().expect("utf8 path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("key store"));
}

#[test]
fn crl_apply_refuses_unknown_issuer_and_tampered_file() {
    let dir = TempDir::new().expect("tempdir");
    let store = dir.path().join("keys.json");
    let issuer_kid = import_key(&dir, 0x57, &store)["kid"]
        .as_str()
        .expect("kid")
        .to_string();
    import_key(&dir, 0x58, &store);
    let custody = custody_file(&dir, 0x57);
    let crl_path = dir.path().join("crl.json");
    affi(&dir)
        .env("AFFI_SIGNING_KEY_PATH", &custody)
        .args([
            "evidence",
            "crl-publish",
            &issuer_kid,
            "2",
            "--store",
            store.to_str().expect("utf8 store path"),
            "--out",
            crl_path.to_str().expect("utf8 crl path"),
        ])
        .assert()
        .success();

    // Tamper with SIGNED content: admission refuses on the issuer signature.
    let intact = std::fs::read_to_string(&crl_path).expect("read crl");
    let tampered = intact.replacen("\"epoch\":2", "\"epoch\":9", 1);
    assert_ne!(tampered, intact, "fixture must flip a signed byte");
    let tampered_path = dir.path().join("crl-tampered.json");
    std::fs::write(&tampered_path, &tampered).expect("write tampered");
    affi(&dir)
        .args([
            "evidence",
            "crl-apply",
            tampered_path.to_str().expect("utf8 path"),
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("does not verify"));

    // Fail-closed: a MISSING file is refused, never an empty CRL.
    let absent = dir.path().join("absent.json");
    affi(&dir)
        .args([
            "evidence",
            "crl-apply",
            absent.to_str().expect("utf8 path"),
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("crl file"));

    // A store that does not hold the issuer: admission refused.
    let other_store = dir.path().join("other-keys.json");
    import_key(&dir, 0x59, &other_store);
    affi(&dir)
        .args([
            "evidence",
            "crl-apply",
            crl_path.to_str().expect("utf8 path"),
            "--store",
            other_store.to_str().expect("utf8 path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("unknown issuer"));

    // A foreign format stamp is refused before any admission.
    let foreign = intact.replace("CTP-CRL-v1", "CTP-CRL-v0");
    assert_ne!(foreign, intact);
    let foreign_path = dir.path().join("crl-foreign.json");
    std::fs::write(&foreign_path, &foreign).expect("write foreign");
    affi(&dir)
        .args([
            "evidence",
            "crl-apply",
            foreign_path.to_str().expect("utf8 path"),
            "--store",
            store.to_str().expect("utf8 store path"),
        ])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("wrong format"));
}

#[test]
fn evidence_verbs_verify_real_signatures_end_to_end() {
    // The one-crypto-level court: the journal entry printed by the CLI is the
    // verbatim projection of a receipt the real engine certified — its
    // receipt_hash recomputes test-side over the printed identity fields via
    // the journal's own chain law (append reproduces the same hash only for
    // identical material), and the envelope signature inside the plane
    // verifies under the custody key (witnessed by the successful VALID
    // standing, which the engine only decides after ES256 verification).
    let dir = TempDir::new().expect("tempdir");
    let custody = custody_file(&dir, 0x5A);
    let entry = run_journal(&dir, &custody, "crypto-court");
    assert_eq!(entry["standing"].as_str().expect("standing"), "VALID");
    assert_eq!(
        entry["receipt_hash"].as_str().expect("receipt hash").len(),
        64
    );
    assert_eq!(
        entry["envelope_commitment"]
            .as_str()
            .expect("commitment")
            .len(),
        64
    );
    // The journal chain anchors at genesis and the head covers the material.
    assert_eq!(
        entry["prev"].as_str().expect("prev"),
        affidavit::crypto_trust_journal::JOURNAL_GENESIS
    );

    // The audit over the durable journal reproduces the same head: the
    // printed standing is the verdict of a real engine (the handler lane
    // witnesses the ES256 layer through the rendered `verify_head`).
    let output = affi(&dir)
        .args(["evidence", "heads"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let heads = stdout_json(&String::from_utf8(output).expect("utf8 stdout"));
    assert_eq!(heads["head"].as_str().expect("head").len(), 64);
    assert_eq!(heads["consistent"].as_bool(), Some(true));
}
