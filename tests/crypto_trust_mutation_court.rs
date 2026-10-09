//! Mutation-court killing assertions for the crypto trust surface
//! (`just mutate-crypto`).
//!
//! Each test pins behavior that survived the baseline cargo-mutants pass
//! (129 mutants: 116 already caught by the existing suite). Classification
//! of the 13 baseline survivors:
//!
//! - 3 x "delete match arm" in `verify_signature_bytes` for MlDsa65,
//!   SlhDsa128s, Hybrid - killed by the per-algorithm test below.
//! - 2 x "delete match arm" for Ed25519/Es256k - equivalent UNDER COURT
//!   FEATURES: those arms are `#[cfg]`-gated on `ed25519`/`secp256k1`,
//!   which the court's feature set does not enable, so deleting a
//!   not-compiled arm is no behavior change (their verification under
//!   those features is pinned by `tests/crypto_trust_e2e.rs`).
//! - `decode_hex` `||`->`&&` (receipts_certified.rs:268): equivalent - a
//!   non-hexdigit byte fails the `u8::from_str_radix` parse to `None`
//!   anyway, so both operators decide identically for every input.
//! - The remaining 7 are killed by the tests in this file.

#![cfg(all(feature = "crypto-trust", feature = "certified-receipts"))]

use affidavit::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
    KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial, RegistryError,
};
use affidavit::crypto_trust_lifecycle::RevocationList;
use affidavit::crypto_trust_pqc::{
    hybrid_sign, ml_dsa65_sign, slh_dsa128s_from_seed, slh_dsa128s_sign, HybridSecret,
};
use affidavit::crypto_trust_verify::{
    CryptographicStanding, TrustPolicy, VerificationEngine, VerifyRefusal,
};
use affidavit::receipts_certified::{
    build_canonical_subject, certify_paid_delivery_payload, verify_certified_paid_delivery,
    CertifiedReceiptEnvelope,
};

const NOW: u64 = 1_700_000_500;
const PAYLOAD_HASH: &str = "3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f";
const SUBJECT: &str = "subject-a";

/// The domain-separated digest of a subject string.
fn subject_digest(subject: &str) -> [u8; 32] {
    use affidavit::crypto_trust_keys::DOMAIN_TAG;
    affidavit::crypto_trust_canonical::digest(DOMAIN_TAG, &[subject.as_bytes()])
}

/// Registers `record` under the graph-default policy clocked at NOW.
fn engine_with(record: KeyRecord) -> VerificationEngine {
    let mut registry = InMemoryKeyRegistry::new();
    registry.register(record).expect("register");
    VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(NOW),
    )
}

/// Builds a registry record over `public_key` for `subject`.
fn record_for(subject: &str, algorithm: AlgorithmId, public_key: PublicKeyMaterial) -> KeyRecord {
    let fingerprint = fingerprint_public_key(algorithm, &public_key);
    KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm,
        fingerprint,
        custodian: CustodianIdentity {
            subject: subject.to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key,
        created_epoch: NOW - 1_000,
    }
}

/// A fresh, in-window envelope for `record` at `profile`.
fn envelope_for(record: &KeyRecord, profile: CryptoProfile, nonce: [u8; 16]) -> SignatureEnvelope {
    SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: record.algorithm,
        key_id: record.id.clone(),
        profile,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce,
        not_before: NOW - 1_000,
        expires_at: NOW + 1_000,
        subject_digest: subject_digest(SUBJECT),
        audience: "affidavit.cli".to_string(),
    }
}

// == gap 1: engine accessors must surface the engine's real state ============

#[test]
fn crypto_trust_mutation_court_engine_accessors_surface_registered_state() {
    let signing = Es256SigningKey::from_seed(&[0x11u8; 32]).expect("valid scalar seed");
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let record = record_for(SUBJECT, AlgorithmId::Es256, public);
    let mut engine = engine_with(record.clone());

    // registry(): must show the registered key, not a phantom empty registry.
    assert_eq!(engine.registry().len(), 1);
    assert!(engine.registry().contains(&record.id));

    // register_key(): must actually register, refusing duplicate ids.
    match engine.register_key(record.clone()) {
        Err(RegistryError::Duplicate(_)) => {}
        other => panic!("duplicate id must be refused, got {other:?}"),
    }

    // revocations(): must surface the revocation the engine governs by
    // (revoke_key advances the epoch to max(revoked_at) + 1).
    let kid = record.id.to_string();
    engine.revoke_key(&kid, NOW, "compromised".to_string());
    assert_eq!(engine.revocations().current_epoch(), NOW + 1);
}

// == gap 2: every non-cfg-gated admitted algorithm verifies its own sig ======
//
// Killing the "delete match arm" mutants in verify_signature_bytes: a deleted
// arm routes the presentation to the `_ => Err(UnknownKey)` catch-all, so a
// VALID decision under each algorithm is the killing assertion.

#[test]
fn crypto_trust_mutation_court_every_admitted_algorithm_verifies_its_own_signature() {
    use affidavit::crypto_trust_pqc::{
        ml_dsa65_from_seed, ML_DSA_65_SEED_LEN, SLH_DSA_128S_SEED_LEN,
    };

    // -- ML-DSA-65 --
    let mldsa_seed = [0xA5u8; ML_DSA_65_SEED_LEN];
    let mldsa_kp = ml_dsa65_from_seed(&mldsa_seed);
    let mldsa_record = record_for(
        SUBJECT,
        AlgorithmId::MlDsa65,
        PublicKeyMaterial::MlDsa65(mldsa_kp.public.clone()),
    );
    let env = envelope_for(&mldsa_record, CryptoProfile::Pqc, [0x21u8; 16]);
    let input = env.signing_input_checked().expect("canonical pre-image");
    let sig = ml_dsa65_sign(&mldsa_seed, &input, &[0u8; 32]).expect("ml-dsa sign");
    let verdict = engine_with(mldsa_record)
        .verify_envelope(&env, &sig)
        .expect("ml-dsa-65 presentation must adjudicate");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);

    // -- SLH-DSA-SHA2-128s --
    let slh_seeds = [0x5Au8; SLH_DSA_128S_SEED_LEN];
    let slh_kp = slh_dsa128s_from_seed(&slh_seeds);
    let slh_record = record_for(
        SUBJECT,
        AlgorithmId::SlhDsa128s,
        PublicKeyMaterial::SlhDsa128s(slh_kp.public.clone()),
    );
    let env = envelope_for(&slh_record, CryptoProfile::Pqc, [0x22u8; 16]);
    let input = env.signing_input_checked().expect("canonical pre-image");
    let sig = slh_dsa128s_sign(&slh_seeds, &input).expect("slh-dsa sign");
    let verdict = engine_with(slh_record)
        .verify_envelope(&env, &sig)
        .expect("slh-dsa presentation must adjudicate");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);

    // -- Hybrid ES256 + ML-DSA-65 (signature is JSON of both halves) --
    let hybrid = HybridSecret {
        es256: Es256SigningKey::from_seed(&[0x77u8; 32]).expect("valid p256 scalar"),
        mldsa65_seed: [0x66u8; ML_DSA_65_SEED_LEN],
    };
    let es256_public = hybrid.es256.public_key_sec1();
    let mldsa_public = ml_dsa65_from_seed(&hybrid.mldsa65_seed).public;
    let hybrid_record = record_for(
        SUBJECT,
        AlgorithmId::HybridEs256MlDsa65,
        PublicKeyMaterial::Hybrid {
            es256: es256_public,
            mldsa65: mldsa_public,
        },
    );
    let env = envelope_for(&hybrid_record, CryptoProfile::Hybrid, [0x23u8; 16]);
    let input = env.signing_input_checked().expect("canonical pre-image");
    let sig = hybrid_sign(&hybrid, &input).expect("hybrid sign");
    let sig_json = serde_json::to_vec(&sig).expect("hybrid signature serializes");
    let verdict = engine_with(hybrid_record)
        .verify_envelope(&env, &sig_json)
        .expect("hybrid presentation must adjudicate");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);
}

// == gap 3: the certified-receipt envelope window is a fixed 660s layout =====
//
// Killing the `expires_at = now + VALIDITY_WINDOW_SECONDS` `+`->`*` mutant:
// the window must be exactly not_before(=now-60) .. expires_at(=now+600).

#[test]
fn crypto_trust_mutation_court_certified_envelope_window_is_660_seconds() {
    let signing = Es256SigningKey::generate().expect("key");
    let certified =
        certify_paid_delivery_payload(PAYLOAD_HASH, SUBJECT, &signing).expect("certify");
    assert_eq!(
        certified.envelope.expires_at - certified.envelope.not_before,
        60 + affidavit::receipts_certified::VALIDITY_WINDOW_SECONDS,
        "validity window layout: not_before=now-60, expires_at=now+600"
    );
}

// == gap 4: every receipt-envelope linkage field refuses ALONE ==============
//
// Killing the `||`->`&&` mutants in the linkage check: under `&&` all four
// fields must mismatch before refusing, so tampering ONE field would ride.
// Each single-field tamper must refuse with the LINKAGE refusal (not the
// later receipt re-audit refusal), which is exactly the discriminating
// observable.

#[test]
fn crypto_trust_mutation_court_linkage_refuses_each_tampered_field_alone() {
    let signing = Es256SigningKey::generate().expect("key");
    let base = certify_paid_delivery_payload(PAYLOAD_HASH, SUBJECT, &signing).expect("certify");

    // -- receipt.subject bound to a different canonical subject --
    let mut certified = base.clone();
    certified.receipt.subject = build_canonical_subject(PAYLOAD_HASH, "subject-b");
    match verify_certified_paid_delivery(&certified, PAYLOAD_HASH, SUBJECT) {
        Err(VerifyRefusal::Provider(msg)) => {
            assert!(msg.contains("linkage mismatch"), "got: {msg}");
        }
        other => panic!("subject linkage tamper must refuse, got {other:?}"),
    }

    // -- receipt.key_id naming another key --
    let mut certified = base.clone();
    certified.receipt.key_id = "afk1_ffffffffffffffff".to_string();
    match verify_certified_paid_delivery(&certified, PAYLOAD_HASH, SUBJECT) {
        Err(VerifyRefusal::Provider(msg)) => {
            assert!(msg.contains("linkage mismatch"), "got: {msg}");
        }
        other => panic!("key_id linkage tamper must refuse, got {other:?}"),
    }

    // -- receipt.algorithm naming another algorithm --
    let mut certified = base.clone();
    certified.receipt.algorithm = "ML-DSA-65".to_string();
    match verify_certified_paid_delivery(&certified, PAYLOAD_HASH, SUBJECT) {
        Err(VerifyRefusal::Provider(msg)) => {
            assert!(msg.contains("linkage mismatch"), "got: {msg}");
        }
        other => panic!("algorithm linkage tamper must refuse, got {other:?}"),
    }
}

// == gap 5: the envelope_commitment linkage leg is load-bearing =============
//
// Killing the `receipt.envelope_commitment != envelope_commitment` leg's
// `||`->`&&` by tampering ONLY the envelope commitment: a receipt minted
// over a DIFFERENT envelope must not ride along.

#[test]
fn crypto_trust_mutation_court_linkage_refuses_a_foreign_envelope_commitment() {
    let signing = Es256SigningKey::generate().expect("key");
    let mut certified =
        certify_paid_delivery_payload(PAYLOAD_HASH, SUBJECT, &signing).expect("certify");
    // Re-point the receipt at a different envelope commitment (64 hex chars,
    // clearly different, but every OTHER linkage field still matches).
    certified.receipt.envelope_commitment = "f".repeat(64);
    match verify_certified_paid_delivery(&certified, PAYLOAD_HASH, SUBJECT) {
        Err(VerifyRefusal::Provider(msg)) => {
            assert!(msg.contains("linkage mismatch"), "got: {msg}");
        }
        other => panic!("commitment tamper must refuse, got {other:?}"),
    }
}
