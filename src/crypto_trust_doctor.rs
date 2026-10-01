//! `affi doctor` crypto-health checks for the cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Law: every check is a zero-size [`DoctorCheck`] registered into the linkme
//! [`DOCTOR_CHECKS`] distributed slice — the same discovery mechanism the
//! built-in checks use (`crate::doctor_check`) — and every check is PURE and
//! DETERMINISTIC given the store file: no check writes key material, creates
//! directories, or reads the wall clock. The self-tests derive their fixed
//! seeds and messages from the graph-declared doctor domain
//! (ctp:doctor-v1 ctp:checkDomain) through the canonical module's
//! domain-separated BLAKE3 ([`crate::crypto_trust_canonical::digest`]), so
//! doctor evidence is invalid in any other domain and the crypto-trust
//! feature's liveness is re-proven on every `affi doctor` run.
//!
//! The four checks:
//! - store integrity — opens the key store ([`STORE_FILE`]) under the full
//!   load law; [`StoreError::ChecksumMismatch`] is a TAMPER ALARM (Fail);
//!   an absent store file is Ok-with-note (no keys admitted yet is health,
//!   not an error);
//! - ES256 self-test — deterministic fixture key → sign → verify round-trip
//!   (real crypto, no mocks);
//! - envelope-law spot check — a fixed envelope judged by
//!   [`crate::crypto_trust_envelope::SignatureEnvelope::window_live`] at
//!   injected instants only (expired/not-yet boundaries are inclusive);
//! - ML-DSA-65 self-test — deterministic seed + explicit randomizer →
//!   sign → verify round-trip, with a foreign-message witness that must
//!   NOT verify.
//!
//! House law: refusal-as-value. Typed refusals surface verbatim in the
//! [`Finding`] message; nothing here panics, coerces, or fakes.

// linkme uses #[link_section] internally (required for distributed_slice).
#![allow(unsafe_code)]

use crate::crypto_trust_envelope::{EnvelopeError, SignatureEnvelope};
use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};
use crate::crypto_trust_keys::{AlgorithmId, CryptoProfile, KeyId};
use crate::crypto_trust_pqc::{ml_dsa65_from_seed, ml_dsa65_sign, ml_dsa65_verify};
use crate::crypto_trust_store::{FileKeyStore, StoreError, STORE_FILE};
use crate::doctor_check::{DoctorCheck, Finding, DOCTOR_CHECKS};
use linkme::distributed_slice;
use std::path::Path;

/// Doctor self-test domain (ctp:doctor-v1 ctp:checkDomain): every fixed
/// self-test seed and message below is
/// [`crate::crypto_trust_canonical::digest`](DOCTOR_CHECK_DOMAIN, …) —
/// domain-separated, so no doctor fixture is valid in any other domain.
pub const DOCTOR_CHECK_DOMAIN: &str = "affidavit.crypto-trust-plane.v1/doctor";

/// Check id of the key-store integrity check.
pub const STORE_INTEGRITY_CHECK_ID: &str = "crypto-store-integrity";

/// Check id of the ES256 self-test.
pub const ES256_SELFTEST_CHECK_ID: &str = "crypto-es256-selftest";

/// Check id of the envelope-law spot check.
pub const ENVELOPE_LAW_CHECK_ID: &str = "crypto-envelope-law";

/// Check id of the ML-DSA-65 self-test.
pub const PQC_SELFTEST_CHECK_ID: &str = "crypto-pqc-selftest";

/// Derives a fixed, domain-separated doctor fixture seed or message from a
/// purpose tag. Deterministic across processes and runs — the doctor's
/// evidence is a function of the graph, never of the environment.
fn doctor_fixture(purpose: &str) -> [u8; 32] {
    crate::crypto_trust_canonical::digest(DOCTOR_CHECK_DOMAIN, &[purpose.as_bytes()])
}

// ---------------------------------------------------------------------------
// (a) Store integrity
// ---------------------------------------------------------------------------

/// Store-integrity finding at an explicit path (pure; reads the file, writes
/// nothing). The registered check passes the graph-declared [`STORE_FILE`].
pub fn store_integrity_finding(path: &Path) -> Finding {
    if !path.exists() {
        return Finding::ok(
            STORE_INTEGRITY_CHECK_ID,
            format!(
                "No key store at {} — no keys admitted yet (not an error)",
                path.display()
            ),
        );
    }
    match FileKeyStore::open(path) {
        Ok(store) => match store.records() {
            Ok(records) => Finding::ok(
                STORE_INTEGRITY_CHECK_ID,
                format!(
                    "Key store at {} verified: {} record(s), checksum matches",
                    path.display(),
                    records.len()
                ),
            ),
            Err(e) => store_refusal_finding(path, &e),
        },
        Err(e) => store_refusal_finding(path, &e),
    }
}

/// Renders a typed [`StoreError`] as the check's finding. A checksum
/// divergence is the tamper alarm: a stored byte changed after the checksum
/// was written, so the store's contents are untrusted.
fn store_refusal_finding(path: &Path, e: &StoreError) -> Finding {
    match e {
        StoreError::ChecksumMismatch { .. } => Finding::fail(
            STORE_INTEGRITY_CHECK_ID,
            format!(
                "TAMPER ALARM: key store at {} refused: {e} — a stored byte changed after the checksum was written",
                path.display()
            ),
            "Treat the key store as untrusted: investigate the mutation, then re-admit keys from their custodians (a checksum divergence is evidence, never repaired in place)",
        ),
        other => Finding::fail(
            STORE_INTEGRITY_CHECK_ID,
            format!("Key store at {} refused: {other}", path.display()),
            "Inspect the key store file — the typed StoreError names the refusal (WrongFormat/Io/Canonical/Registry/Serialization)",
        ),
    }
}

struct StoreIntegrityCheck;

impl DoctorCheck for StoreIntegrityCheck {
    fn id(&self) -> &'static str {
        STORE_INTEGRITY_CHECK_ID
    }
    fn run(&self) -> Finding {
        store_integrity_finding(Path::new(STORE_FILE))
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _STORE_INTEGRITY_CHECK: &dyn DoctorCheck = &StoreIntegrityCheck;

// ---------------------------------------------------------------------------
// (b) ES256 self-test
// ---------------------------------------------------------------------------

/// ES256 self-test finding: derives the fixed fixture key from the doctor
/// domain, signs a fixed domain-derived message, verifies the round-trip.
/// Real P-256/SHA-256/RFC-6979 crypto end to end — no mocks; a failure here
/// means the crypto-trust feature itself is broken.
pub fn es256_selftest_finding() -> Finding {
    let seed = doctor_fixture("es256-signing-key");
    let msg = doctor_fixture("es256-message");
    match Es256SigningKey::from_seed(&seed) {
        Ok(key) => {
            let sig = key.sign(&msg);
            match verify_es256(&key.public_key_sec1(), &msg, &sig) {
                Ok(true) => Finding::ok(
                    ES256_SELFTEST_CHECK_ID,
                    "ES256 sign→verify round-trip passed (deterministic fixture key derived from ctp:doctor-v1 ctp:checkDomain) — classical profile live",
                ),
                Ok(false) => Finding::fail(
                    ES256_SELFTEST_CHECK_ID,
                    "ES256 self-test: a well-formed signature did NOT verify — the classical profile is broken",
                    "The crypto-trust feature is compromised or miscompiled; do not trust plane signatures until resolved",
                ),
                Err(e) => Finding::fail(
                    ES256_SELFTEST_CHECK_ID,
                    format!("ES256 self-test refused: {e}"),
                    "The crypto-trust feature is compromised or miscompiled; do not trust plane signatures until resolved",
                ),
            }
        }
        Err(e) => Finding::fail(
            ES256_SELFTEST_CHECK_ID,
            format!("ES256 self-test key derivation refused: {e}"),
            "The doctor fixture seed must derive a valid non-zero P-256 scalar; a refusal means the fixture derivation or the curve provider is broken",
        ),
    }
}

struct Es256SelftestCheck;

impl DoctorCheck for Es256SelftestCheck {
    fn id(&self) -> &'static str {
        ES256_SELFTEST_CHECK_ID
    }
    fn run(&self) -> Finding {
        es256_selftest_finding()
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _ES256_SELFTEST_CHECK: &dyn DoctorCheck = &Es256SelftestCheck;

// ---------------------------------------------------------------------------
// (c) Envelope-law spot check
// ---------------------------------------------------------------------------

/// The fixed doctor envelope: deterministic in every field, validity window
/// `[1_000, 2_000]`. Never signed, never persisted — it exists to exercise
/// the envelope window law only.
fn fixed_envelope() -> SignatureEnvelope {
    SignatureEnvelope {
        version: crate::crypto_trust_envelope::ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: KeyId("afk1_doctorfixture0000".to_string()),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce: [0x5D; 16],
        not_before: 1_000,
        expires_at: 2_000,
        subject_digest: doctor_fixture("envelope-subject"),
        audience: "affidavit.cli".to_string(),
    }
}

/// Envelope-law finding: the fixed envelope must be refused strictly below
/// its window (NotYetValid carrying not_before), live at BOTH inclusive
/// boundaries and strictly inside, and refused strictly after (Expired
/// carrying expires_at). Every instant is injected — no wall clock.
pub fn envelope_law_finding() -> Finding {
    let env = fixed_envelope();
    let mut violations: Vec<String> = Vec::new();

    match env.window_live(env.not_before - 1) {
        Err(EnvelopeError::NotYetValid(nb)) if nb == env.not_before => {}
        other => violations.push(format!(
            "not-yet boundary wrong: expected NotYetValid({}), got {other:?}",
            env.not_before
        )),
    }
    for now in [
        env.not_before,
        (env.not_before + env.expires_at) / 2,
        env.expires_at,
    ] {
        if let Err(e) = env.window_live(now) {
            violations.push(format!("window_live({now}) refused a live instant: {e}"));
        }
    }
    match env.window_live(env.expires_at + 1) {
        Err(EnvelopeError::Expired(exp)) if exp == env.expires_at => {}
        other => violations.push(format!(
            "expiry boundary wrong: expected Expired({}), got {other:?}",
            env.expires_at
        )),
    }

    if violations.is_empty() {
        Finding::ok(
            ENVELOPE_LAW_CHECK_ID,
            "Envelope window law holds: NotYetValid strictly below, live at both inclusive boundaries and inside, Expired strictly after (injected now — no wall clock)",
        )
    } else {
        Finding::fail(
            ENVELOPE_LAW_CHECK_ID,
            format!("Envelope window law violated: {}", violations.join("; ")),
            "The envelope window law is a plane invariant; do not trust envelope validity windows until resolved",
        )
    }
}

struct EnvelopeLawCheck;

impl DoctorCheck for EnvelopeLawCheck {
    fn id(&self) -> &'static str {
        ENVELOPE_LAW_CHECK_ID
    }
    fn run(&self) -> Finding {
        envelope_law_finding()
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _ENVELOPE_LAW_CHECK: &dyn DoctorCheck = &EnvelopeLawCheck;

// ---------------------------------------------------------------------------
// (d) ML-DSA-65 self-test
// ---------------------------------------------------------------------------

/// ML-DSA-65 self-test finding: derives the fixed key pair from the doctor
/// domain, signs a fixed message under a fixed explicit randomizer
/// (reproducible per FIPS 204 ML-DSA.Sign_internal), verifies the round-trip,
/// and witnesses that a foreign message does NOT verify.
pub fn pqc_selftest_finding() -> Finding {
    let seed = doctor_fixture("ml-dsa-65-seed");
    let msg = doctor_fixture("ml-dsa-65-message");
    let rnd = doctor_fixture("ml-dsa-65-rnd");
    let key = ml_dsa65_from_seed(&seed);
    match ml_dsa65_sign(&seed, &msg, &rnd) {
        Ok(sig) => {
            // Round-trip: the exact (message, signature) pair must verify.
            match ml_dsa65_verify(&key.public, &msg, &sig) {
                Ok(true) => {}
                Ok(false) => {
                    return Finding::fail(
                        PQC_SELFTEST_CHECK_ID,
                        "ML-DSA-65 self-test: a well-formed signature did NOT verify — the post-quantum profile is broken",
                        "The crypto-trust feature is compromised or miscompiled; do not trust plane signatures until resolved",
                    )
                }
                Err(e) => {
                    return Finding::fail(
                        PQC_SELFTEST_CHECK_ID,
                        format!("ML-DSA-65 self-test round-trip refused: {e}"),
                        "The crypto-trust feature is compromised or miscompiled; do not trust plane signatures until resolved",
                    )
                }
            }
            // Foreign-message witness: verification must bind the message.
            match ml_dsa65_verify(&key.public, &doctor_fixture("foreign-message"), &sig) {
                Ok(false) => Finding::ok(
                    PQC_SELFTEST_CHECK_ID,
                    "ML-DSA-65 sign→verify round-trip passed and a foreign message is refused (deterministic seed + explicit rnd) — post-quantum profile live",
                ),
                Ok(true) => Finding::fail(
                    PQC_SELFTEST_CHECK_ID,
                    "ML-DSA-65 self-test: foreign-message verify returned true, expected false — verification does not bind the message",
                    "The post-quantum profile is broken; do not trust plane signatures until resolved",
                ),
                Err(e) => Finding::fail(
                    PQC_SELFTEST_CHECK_ID,
                    format!("ML-DSA-65 self-test foreign-message verify refused: {e}"),
                    "The crypto-trust feature is compromised or miscompiled; do not trust plane signatures until resolved",
                ),
            }
        }
        Err(e) => Finding::fail(
            PQC_SELFTEST_CHECK_ID,
            format!("ML-DSA-65 self-test signing refused: {e}"),
            "Deterministic ML-DSA-65 signing must succeed for a valid 32-byte seed; a refusal means the profile is broken",
        ),
    }
}

struct PqcSelftestCheck;

impl DoctorCheck for PqcSelftestCheck {
    fn id(&self) -> &'static str {
        PQC_SELFTEST_CHECK_ID
    }
    fn run(&self) -> Finding {
        pqc_selftest_finding()
    }
}

#[distributed_slice(DOCTOR_CHECKS)]
static _PQC_SELFTEST_CHECK: &dyn DoctorCheck = &PqcSelftestCheck;

// ---------------------------------------------------------------------------
// Subset runner
// ---------------------------------------------------------------------------

/// Runs exactly this module's crypto-health checks, in law order (a)–(d).
/// The `affi doctor` dispatcher runs the whole [`DOCTOR_CHECKS`] registry;
/// this is the crypto-plane subset, for callers that want the findings
/// without the environment checks.
pub fn run_crypto_checks() -> Vec<Finding> {
    vec![
        store_integrity_finding(Path::new(STORE_FILE)),
        es256_selftest_finding(),
        envelope_law_finding(),
        pqc_selftest_finding(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DOMAIN_TAG;
    use crate::doctor_check::FindingStatus;

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(
            DOCTOR_CHECK_DOMAIN,
            "affidavit.crypto-trust-plane.v1/doctor"
        );
    }

    #[test]
    fn domain_tag_is_owned_by_canonical_module_and_domains_cohere() {
        // The canonical module is the single owner of DOMAIN_TAG (no second
        // copy may drift); the doctor domain must be a leaf under it.
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(
            DOCTOR_CHECK_DOMAIN.strip_prefix(DOMAIN_TAG),
            Some("/doctor")
        );
    }

    #[test]
    fn crypto_checks_are_registered_in_doctor_slice() {
        let registered: Vec<&str> = DOCTOR_CHECKS.iter().map(|c| c.id()).collect();
        for id in [
            STORE_INTEGRITY_CHECK_ID,
            ES256_SELFTEST_CHECK_ID,
            ENVELOPE_LAW_CHECK_ID,
            PQC_SELFTEST_CHECK_ID,
        ] {
            assert!(
                registered.contains(&id),
                "check {id} is not registered in DOCTOR_CHECKS"
            );
        }
    }

    #[test]
    fn self_test_findings_are_green_and_deterministic() {
        for f in [
            es256_selftest_finding(),
            envelope_law_finding(),
            pqc_selftest_finding(),
        ] {
            assert_eq!(f.status, FindingStatus::Ok, "{}: {}", f.id, f.message);
            let again = match f.id {
                ES256_SELFTEST_CHECK_ID => es256_selftest_finding(),
                ENVELOPE_LAW_CHECK_ID => envelope_law_finding(),
                _ => pqc_selftest_finding(),
            };
            assert_eq!(f.message, again.message, "{} must be deterministic", f.id);
        }
    }

    #[test]
    fn absent_store_is_ok_with_note() {
        let f = store_integrity_finding(Path::new("definitely/absent/doctor-store.json"));
        assert_eq!(f.status, FindingStatus::Ok);
        assert!(f.message.contains("no keys admitted"), "{}", f.message);
    }
}
