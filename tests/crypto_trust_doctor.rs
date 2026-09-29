#![cfg(feature = "crypto-trust")]
//! Teeth-first integration tests for the `affi doctor` crypto-health checks
//! (wave-4 lane W4-L5).
//!
//! Order is teeth-first: the healthy plane must be all green before the
//! adversarial cases. Every check is exercised through its public finding
//! function at an explicit path (pure, deterministic given the store file),
//! and the doctor-framework registration is witnessed against the real
//! linkme [`DOCTOR_CHECKS`] slice. No test writes key material outside a
//! per-test scratch directory; no wall clock is read anywhere.

use affidavit::crypto_trust_doctor::{
    envelope_law_finding, es256_selftest_finding, pqc_selftest_finding, run_crypto_checks,
    store_integrity_finding, ENVELOPE_LAW_CHECK_ID, ES256_SELFTEST_CHECK_ID, PQC_SELFTEST_CHECK_ID,
    STORE_INTEGRITY_CHECK_ID,
};
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CustodianIdentity, KeyId, KeyOrigin, KeyRecord,
    PublicKeyMaterial,
};
use affidavit::crypto_trust_store::{
    checksum_for, FileKeyStore, KeyStoreFile, StoreError, STORE_FORMAT,
};
use affidavit::doctor_check::{FindingStatus, DOCTOR_CHECKS};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique scratch directory per call (no tempfile dependency here; std only).
fn temp_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("ctp-doctor-{tag}-{pid}-{n}"));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// A fixed, valid ES256 key record (mirrors the store module's fixture law:
/// salted material so fingerprints never collide across cases).
fn doctor_record() -> KeyRecord {
    let public_key = PublicKeyMaterial::Es256Sec1(vec![0x7A; 65]);
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public_key);
    KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm: AlgorithmId::Es256,
        fingerprint,
        custodian: CustodianIdentity {
            subject: "subject-a".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key,
        created_epoch: 1_700_000_000,
    }
}

/// Writes a self-consistent store file (records + matching checksum) into
/// `dir` and returns its path.
fn write_valid_store(dir: &Path) -> PathBuf {
    let records = vec![doctor_record()];
    let store = KeyStoreFile {
        format: STORE_FORMAT.to_string(),
        checksum: checksum_for(&records).expect("fixture records must canonicalize"),
        records,
    };
    let path = dir.join("keys.json");
    std::fs::write(
        &path,
        serde_json::to_string(&store).expect("fixture must serialize"),
    )
    .expect("write fixture store");
    path
}

/// Flips exactly one byte inside the stored records ("subject-a" becomes
/// "tubject-a"). The file stays valid JSON, so any refusal must come from
/// the checksum, not the parser.
fn tamper(path: &Path) {
    let text = std::fs::read_to_string(path).expect("read store");
    let flipped = text.replacen("\"subject-a\"", "\"tubject-a\"", 1);
    assert_ne!(flipped, text, "fixture must contain the flipped byte");
    std::fs::write(path, flipped).expect("write tampered store");
}

// ---------------------------------------------------------------------------
// Teeth first: the healthy plane is all green.
// ---------------------------------------------------------------------------

#[test]
fn teeth_healthy_plane_all_green() {
    let dir = temp_dir("teeth");
    let path = write_valid_store(&dir);

    let findings = vec![
        store_integrity_finding(&path),
        es256_selftest_finding(),
        envelope_law_finding(),
        pqc_selftest_finding(),
    ];
    for f in &findings {
        assert_eq!(f.status, FindingStatus::Ok, "{}: {}", f.id, f.message);
    }

    // The subset runner agrees with the individual findings, in law order.
    let run = run_crypto_checks();
    assert_eq!(run.len(), 4);
    // Store finding here is against the ambient STORE_FILE — absent or valid
    // are both health, so only the self-tests must match one-to-one.
    assert_eq!(run[1].id, ES256_SELFTEST_CHECK_ID);
    assert_eq!(run[2].id, ENVELOPE_LAW_CHECK_ID);
    assert_eq!(run[3].id, PQC_SELFTEST_CHECK_ID);
    assert_eq!(run[1].status, findings[1].status);
    assert_eq!(run[2].status, findings[2].status);
    assert_eq!(run[3].status, findings[3].status);
}

#[test]
fn teeth_self_tests_are_real_crypto_and_deterministic() {
    for f in [
        es256_selftest_finding(),
        envelope_law_finding(),
        pqc_selftest_finding(),
    ] {
        assert_eq!(f.status, FindingStatus::Ok, "{}: {}", f.id, f.message);
    }
    // Pure functions of the graph-derived fixtures: same call, same bytes.
    assert_eq!(
        es256_selftest_finding().message,
        es256_selftest_finding().message
    );
    assert_eq!(
        pqc_selftest_finding().message,
        pqc_selftest_finding().message
    );
    assert_eq!(
        envelope_law_finding().message,
        envelope_law_finding().message
    );
}

// ---------------------------------------------------------------------------
// Registration: the checks are discoverable through the doctor framework.
// ---------------------------------------------------------------------------

#[test]
fn crypto_checks_register_into_doctor_slice_and_run() {
    let mut seen = 0;
    for check in DOCTOR_CHECKS.iter() {
        let id = check.id();
        if matches!(
            id,
            STORE_INTEGRITY_CHECK_ID
                | ES256_SELFTEST_CHECK_ID
                | ENVELOPE_LAW_CHECK_ID
                | PQC_SELFTEST_CHECK_ID
        ) {
            seen += 1;
            let f = check.run();
            assert_eq!(f.id, id);
            // Ambient store is either absent or valid — both are health; the
            // self-tests must be green unconditionally.
            assert_eq!(f.status, FindingStatus::Ok, "{}: {}", f.id, f.message);
        }
    }
    assert_eq!(seen, 4, "all four crypto-health checks must be registered");
}

// ---------------------------------------------------------------------------
// Adversarial: tampered store raises the checksum alarm (typed variant).
// ---------------------------------------------------------------------------

#[test]
fn tampered_store_is_a_tamper_alarm_with_typed_checksum_mismatch() {
    let dir = temp_dir("tamper");
    let path = write_valid_store(&dir);
    tamper(&path);

    // Typed refusal witnessed directly, not only through the finding text.
    let err = FileKeyStore::open(&path).expect_err("tampered store must be refused");
    match &err {
        StoreError::ChecksumMismatch {
            claimed,
            recomputed,
        } => {
            assert_ne!(claimed, recomputed);
        }
        other => panic!("expected ChecksumMismatch, got {other:?}"),
    }

    let f = store_integrity_finding(&path);
    assert_eq!(f.status, FindingStatus::Fail, "{:?}", f.message);
    assert!(f.message.contains("TAMPER ALARM"), "{}", f.message);
    assert!(f.message.contains(&err.to_string()), "{}", f.message);
    assert!(f.remediation.is_some());
}

#[test]
fn foreign_format_store_fails_with_typed_wrong_format() {
    let dir = temp_dir("wrongformat");
    let path = write_valid_store(&dir);
    let text = std::fs::read_to_string(&path).expect("read store");
    let foreign = text.replace(
        &format!("\"format\":\"{STORE_FORMAT}\""),
        "\"format\":\"SOME-OTHER-v9\"",
    );
    assert_ne!(foreign, text, "fixture must contain the format field");
    std::fs::write(&path, foreign).expect("write foreign store");

    let err = FileKeyStore::open(&path).expect_err("foreign format must be refused");
    assert!(matches!(err, StoreError::WrongFormat(_)), "{err:?}");

    let f = store_integrity_finding(&path);
    assert_eq!(f.status, FindingStatus::Fail);
    assert!(f.message.contains("wrong store format"), "{}", f.message);
}

#[test]
fn absent_store_is_ok_with_note_not_an_error() {
    let dir = temp_dir("absent");
    let path = dir.join("keys.json");
    assert!(!path.exists());
    let f = store_integrity_finding(&path);
    assert_eq!(f.status, FindingStatus::Ok, "{}", f.message);
    assert!(f.message.contains("no keys admitted"), "{}", f.message);
    assert!(f.remediation.is_none(), "health needs no remediation");
}

#[test]
fn store_check_is_deterministic_given_the_same_file() {
    let dir = temp_dir("determinism");
    let path = write_valid_store(&dir);
    let a = store_integrity_finding(&path);
    let b = store_integrity_finding(&path);
    assert_eq!(a.status, b.status);
    assert_eq!(a.message, b.message);
}
