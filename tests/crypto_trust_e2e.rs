#![cfg(feature = "crypto-trust")]
//! End-to-end court: ontology -> keys -> envelope -> sign -> verify -> standing -> seal -> tamper -> refuse.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//! Consumed query columns (e2e.rq): envelope_version, domain_tag, algs.
//!
//! The court is a projection of the graph: one admission test renders per
//! algorithm the ontology admits, so admitting an algorithm individual
//! extends the court without touching this file.
//!
//! Determinism law: NO wall-clock anywhere. Every sitting passes its instant
//! explicitly through `TrustPolicy::with_now`; the constants below are
//! instants on the trust-plane epoch, not readings, and every integer stays
//! inside JCS's 2^53 so signatures cover the real domain-separated pre-image
//! (never the empty fallback an uncanonicalizable envelope produces).
//!
//! Replay-journal sequencing: each adjudication of an envelope journals its
//! (kid, nonce) inside the replay window, so independent sittings of the
//! court (verify, certify, verify_sealed) each get a FRESH engine — same key
//! registry law, empty journal. The replay falsifier is the one place that
//! deliberately reuses a single engine twice.
//!
//! House law: certify-don't-decide. A VALID standing here is evidence about
//! bytes, keys, and revocation state — never authorization; authorization is
//! SA2A's decision over the minted `CryptoStandingReceipt`.

use affidavit::chain::ChainAssembler;
use affidavit::crypto_trust_envelope::{NonceJournal, SignatureEnvelope};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
    KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
};
use affidavit::crypto_trust_lifecycle::RevocationList;
use affidavit::crypto_trust_seal::{
    seal_receipt, subject_digest_of, verify_sealed, SealedReceipt, SEALED_RECEIPT_FORMAT,
};
use affidavit::crypto_trust_verify::{
    CryptoStandingReceipt, CryptographicStanding, TrustPolicy, VerificationEngine, VerifyRefusal,
    CRYPTO_STANDING_PROFILE,
};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::Receipt;
use affidavit::verifier::verify;

/// The court's pinned envelope version (ctp:policy-v1 ctp:envelopeVersion).
const COURT_ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";
/// The court's pinned domain tag (ctp:policy-v1 ctp:domainTag).
const COURT_DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// The main sitting's verifier-clock instant.
const NOW: u64 = 1_700_000_500;
/// Envelope validity window: [NOT_BEFORE, EXPIRES_AT) brackets NOW.
const NOT_BEFORE: u64 = 1_700_000_000;
const EXPIRES_AT: u64 = 1_700_003_600;
/// 400 seconds before the main sitting: past the graph's 300-second
/// revocation-staleness grace, still inside every validity window.
const NOW_MINUS_400: u64 = NOW - 400;

/// A real one-event provenance receipt from the canonical assembler.
fn real_receipt(payload: &[u8]) -> Receipt {
    let mut assembler = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let event = build_event(
        "court.op",
        vec![object_ref("e2e-receipt", "artifact")],
        payload,
        &mut counter,
    )
    .expect("event is well-formed");
    assembler.append(event).expect("append admitted");
    assembler.finalize()
}

/// A deterministic ES256 signing key + its registry record.
fn es256_fixture(tag: u8) -> (Es256SigningKey, KeyRecord) {
    let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    let record = KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm: AlgorithmId::Es256,
        fingerprint,
        custodian: CustodianIdentity {
            subject: "subject-a".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: public,
        created_epoch: NOT_BEFORE,
    };
    (signing, record)
}

/// A lawfully-formed envelope over `subject` in the court's window, with a
/// fresh nonce.
fn envelope_over(subject: [u8; 32], kid: &KeyId, nonce_tag: u8) -> SignatureEnvelope {
    SignatureEnvelope {
        version: COURT_ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: kid.clone(),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce: [nonce_tag; 16],
        not_before: NOT_BEFORE,
        expires_at: EXPIRES_AT,
        subject_digest: subject,
        // An audience admitted by the graph-default policy: the court's
        // target is the certify/seal stack, never audience policy.
        audience: "affidavit.cli".to_string(),
    }
}

/// A fresh sitting of the court: the fixture key registered, nothing revoked,
/// an empty replay journal, the graph-default policy at `now`. Each sitting
/// gets its own journal — the replay law is per-engine state (the replay
/// falsifier below is the one deliberate exception).
fn fresh_sitting(record: &KeyRecord, now: u64) -> VerificationEngine {
    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(record.clone())
        .expect("fixture key registers");
    VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(now),
    )
}

// ── per-algorithm admission: one test renders per graph algorithm ───────────
#[test]
fn court_admits_es256_exactly_as_the_graph_declares() {
    // The graph admits this algorithm; the keys module renders the variant
    // and the court holds the rendered plane to it.
    let all = AlgorithmId::all();
    let alg = AlgorithmId::Es256;
    assert!(
        all.contains(&alg),
        "Es256 must be admitted by the rendered plane"
    );
    assert!(!alg.as_str().is_empty());
    // Wire form round-trips through the keys module's serde law.
    let json = serde_json::to_string(&alg).expect("algorithm serializes");
    let back: AlgorithmId = serde_json::from_str(&json).expect("algorithm deserializes");
    assert_eq!(back, alg);
    // Assurance profile follows the pinned policy mapping.
    assert_eq!(alg.profile(), CryptoProfile::Classical);
}
#[test]
fn court_admits_hybrides256mldsa65_exactly_as_the_graph_declares() {
    // The graph admits this algorithm; the keys module renders the variant
    // and the court holds the rendered plane to it.
    let all = AlgorithmId::all();
    let alg = AlgorithmId::HybridEs256MlDsa65;
    assert!(
        all.contains(&alg),
        "HybridEs256MlDsa65 must be admitted by the rendered plane"
    );
    assert!(!alg.as_str().is_empty());
    // Wire form round-trips through the keys module's serde law.
    let json = serde_json::to_string(&alg).expect("algorithm serializes");
    let back: AlgorithmId = serde_json::from_str(&json).expect("algorithm deserializes");
    assert_eq!(back, alg);
    // Assurance profile follows the pinned policy mapping.
    assert_eq!(alg.profile(), CryptoProfile::Hybrid);
}
#[test]
fn court_admits_mldsa65_exactly_as_the_graph_declares() {
    // The graph admits this algorithm; the keys module renders the variant
    // and the court holds the rendered plane to it.
    let all = AlgorithmId::all();
    let alg = AlgorithmId::MlDsa65;
    assert!(
        all.contains(&alg),
        "MlDsa65 must be admitted by the rendered plane"
    );
    assert!(!alg.as_str().is_empty());
    // Wire form round-trips through the keys module's serde law.
    let json = serde_json::to_string(&alg).expect("algorithm serializes");
    let back: AlgorithmId = serde_json::from_str(&json).expect("algorithm deserializes");
    assert_eq!(back, alg);
    // Assurance profile follows the pinned policy mapping.
    assert_eq!(alg.profile(), CryptoProfile::Pqc);
}
#[test]
fn court_admits_slhdsa128s_exactly_as_the_graph_declares() {
    // The graph admits this algorithm; the keys module renders the variant
    // and the court holds the rendered plane to it.
    let all = AlgorithmId::all();
    let alg = AlgorithmId::SlhDsa128s;
    assert!(
        all.contains(&alg),
        "SlhDsa128s must be admitted by the rendered plane"
    );
    assert!(!alg.as_str().is_empty());
    // Wire form round-trips through the keys module's serde law.
    let json = serde_json::to_string(&alg).expect("algorithm serializes");
    let back: AlgorithmId = serde_json::from_str(&json).expect("algorithm deserializes");
    assert_eq!(back, alg);
    // Assurance profile follows the pinned policy mapping.
    assert_eq!(alg.profile(), CryptoProfile::Pqc);
}

// ── the full ES256 cycle: receipt -> subject digest -> envelope -> sign ────
// ── -> verify -> standing -> certify -> seal -> verify_sealed ───────────────

#[test]
fn es256_full_cycle_from_receipt_to_standing_to_seal() {
    // (1) A real receipt, and the teeth that its own law holds before any
    // seal is allowed to attest it.
    let receipt = real_receipt(b"e2e-full-cycle");
    assert!(verify(&receipt).accepted, "base receipt must certify");

    // (2) A real key, registered in a real registry.
    let (signing, record) = es256_fixture(7);

    // (3) The envelope binds the receipt's content address, domain-separated.
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let envelope = envelope_over(subject, &record.id, 0x21);

    // (4) Sitting one: the engine verifies the signature -> VALID.
    let engine = fresh_sitting(&record, NOW);
    let signature = signing.sign(&envelope.signing_input());
    let verdict = engine
        .verify_envelope(&envelope, &signature)
        .expect("fresh signature adjudicates");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);
    assert_eq!(verdict.subject_digest, subject);
    assert_eq!(verdict.key_id, Some(record.id.clone()));

    // (5) Sitting two: certify mints the standing receipt — evidence for the
    // SA2A authorization layer, never authorization itself. `certify_signed`
    // enforces SUBJECT BINDING: the named subject string must digest — under
    // the trust-plane domain tag — to exactly the envelope's bound digest,
    // which here is the receipt's content address.
    let subject_address = affidavit::chain::content_address(&receipt)
        .expect("content address")
        .as_hex()
        .to_string();
    let certifying = fresh_sitting(&record, NOW);
    let standing_receipt: CryptoStandingReceipt = certifying
        .certify_signed(&envelope, &signature, &subject_address, &signing)
        .expect("VALID verdict mints a receipt");
    assert_eq!(standing_receipt.profile, CRYPTO_STANDING_PROFILE);
    assert_eq!(standing_receipt.standing, CryptographicStanding::Valid);
    assert_eq!(standing_receipt.key_id, record.id.to_string());
    assert_eq!(standing_receipt.algorithm, "ES256");
    standing_receipt
        .verify()
        .expect("minted receipt re-verifies");

    // (6) Sitting three: the seal couples the receipt to the attestation and
    // re-adjudicates the whole stack.
    let sealed: SealedReceipt =
        seal_receipt(&receipt, envelope.clone(), signature.clone()).expect("lawful seal");
    assert_eq!(SEALED_RECEIPT_FORMAT, "PQ-SEAL-v1");
    let sealing = fresh_sitting(&record, NOW);
    let verdict = verify_sealed(&sealed, &sealing).expect("sealed adjudicates");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);
    assert_eq!(verdict.subject_digest, subject);
}

// ── tamper falsifiers: every refusal witnessed by exact value ───────────────

#[test]
fn wrong_signer_yields_a_decided_invalid_verdict() {
    let receipt = real_receipt(b"wrong-signer");
    let (_honest, record) = es256_fixture(8);
    let impostor = Es256SigningKey::from_seed(&[0xBB; 32]).expect("valid impostor scalar");

    let subject = subject_digest_of(&receipt).expect("subject digest");
    let envelope = envelope_over(subject, &record.id, 0x22);
    // Signed by the impostor, verified against the honest key.
    let forged = impostor.sign(&envelope.signing_input());

    let engine = fresh_sitting(&record, NOW);
    let verdict = engine
        .verify_envelope(&envelope, &forged)
        .expect("a wrong signer is adjudicated, not refused");
    assert_eq!(verdict.standing, CryptographicStanding::Invalid);
}

#[test]
fn replaying_the_same_envelope_refuses() {
    let receipt = real_receipt(b"replay");
    let (signing, record) = es256_fixture(9);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let envelope = envelope_over(subject, &record.id, 0x23);
    let signature = signing.sign(&envelope.signing_input());

    // ONE engine, TWO sittings: the (kid, nonce) pair journals on first use,
    // so the identical second adjudication is a replay. verify_envelope takes
    // only &self — the journal is interior to the engine.
    let engine = {
        let mut registry = InMemoryKeyRegistry::new();
        registry
            .register(record.clone())
            .expect("fixture key registers");
        VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(NOW),
        )
    };
    engine
        .verify_envelope(&envelope, &signature)
        .expect("first sight admits");

    match engine.verify_envelope(&envelope, &signature) {
        Err(VerifyRefusal::ReplayRejected(kid)) => {
            assert!(kid.contains(record.id.to_string().as_str()))
        }
        other => panic!("expected ReplayRejected, got {other:?}"),
    }
}

#[test]
fn revoked_key_refuses() {
    let receipt = real_receipt(b"revoked");
    let (signing, record) = es256_fixture(10);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let envelope = envelope_over(subject, &record.id, 0x24);
    let signature = signing.sign(&envelope.signing_input());

    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(record.clone())
        .expect("fixture key registers");
    let mut revocations = RevocationList::default();
    // Revoke 10 seconds before the sitting: the signature's epoch 0 lags the
    // new current epoch, but well inside the staleness grace — so the flat
    // revocation law is the door that fires, not the staleness law.
    revocations.revoke(&record.id.to_string(), NOW - 10, "compromised".to_string());
    let engine = VerificationEngine::new(
        registry,
        revocations,
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(NOW),
    );
    match engine.verify_envelope(&envelope, &signature) {
        Err(VerifyRefusal::KeyRevoked(kid)) => assert_eq!(kid, record.id.to_string()),
        other => panic!("expected KeyRevoked, got {other:?}"),
    }
}

#[test]
fn stale_revocation_epoch_refuses_past_grace() {
    let receipt = real_receipt(b"stale-epoch");
    let (signing, record) = es256_fixture(11);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    // The envelope stamps revocation epoch 0: signed before any revocation.
    let envelope = envelope_over(subject, &record.id, 0x25);
    let signature = signing.sign(&envelope.signing_input());

    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(record.clone())
        .expect("fixture key registers");
    let mut revocations = RevocationList::default();
    // Revoked 400s before the sitting: the epoch advances to
    // NOW_MINUS_400 + 1 (>> 0, so epoch 0 is stale) and the revocation sits
    // past the graph's 300-second staleness grace at `now` = NOW.
    revocations.revoke(
        &record.id.to_string(),
        NOW_MINUS_400,
        "rotated-after-compromise".to_string(),
    );
    let engine = VerificationEngine::new(
        registry,
        revocations,
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(NOW),
    );
    // The epoch-freshness law fires BEFORE the flat revocation law: the
    // signature names a stale epoch and the revocation is past grace.
    match engine.verify_envelope(&envelope, &signature) {
        Err(VerifyRefusal::StaleRevocationEpoch(detail)) => {
            assert!(
                detail.contains(&record.id.to_string()),
                "refusal names the kid: {detail}"
            );
        }
        other => panic!("expected StaleRevocationEpoch, got {other:?}"),
    }
}

#[test]
fn expired_envelope_refuses() {
    let receipt = real_receipt(b"expired");
    let (signing, record) = es256_fixture(12);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    // Valid window closed 100 seconds before the sitting.
    let mut envelope = envelope_over(subject, &record.id, 0x26);
    envelope.not_before = NOW - 200;
    envelope.expires_at = NOW - 100;
    let signature = signing.sign(&envelope.signing_input());

    let engine = fresh_sitting(&record, NOW);
    match engine.verify_envelope(&envelope, &signature) {
        Err(VerifyRefusal::Expired(expires_at)) => assert_eq!(expires_at, NOW - 100),
        other => panic!("expected Expired, got {other:?}"),
    }
}

#[test]
fn tampered_sealed_base_refuses_to_deserialize() {
    let receipt = real_receipt(b"tampered-seal");
    let (signing, record) = es256_fixture(13);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let envelope = envelope_over(subject, &record.id, 0x27);
    let signature = signing.sign(&envelope.signing_input());
    let sealed = seal_receipt(&receipt, envelope, signature).expect("lawful seal");

    let json = serde_json::to_string(&sealed).expect("sealed serializes");
    // Flip the last character of the base's first payload commitment; the
    // envelope and signature bytes are untouched.
    let marker = sealed.base.events[0]
        .payload_commitment
        .as_hex()
        .to_string();
    let flipped = format!(
        "{}{}",
        &marker[..marker.len() - 1],
        if marker.ends_with('0') { "1" } else { "0" }
    );
    let tampered = json.replace(&marker, &flipped);
    assert_ne!(json, tampered, "tamper must change the wire bytes");

    // The base receipt's chain law refuses inside deserialization: the
    // composite cannot load at all. Refusal-as-value at the wire boundary —
    // and the tampered base no longer hashes to the envelope's subject
    // digest, so verify_sealed would refuse it as a subject mismatch even if
    // a loader bypassed the chain law.
    let outcome: Result<SealedReceipt, _> = serde_json::from_str(&tampered);
    let err = outcome.expect_err("tampered base must refuse to deserialize");
    assert!(
        err.to_string().contains("chain hash mismatch"),
        "the chain law must be the refusing door, got: {err}"
    );
}

/// The court's pins agree with the modules it drove: the rendered graph row
/// equals the constants this court pinned (a half-projected ontology fails
/// here first, with zero rows upstream).
#[test]
fn court_pins_match_the_rendered_plane() {
    assert_eq!(COURT_ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
    assert_eq!(COURT_DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
    // Every algorithm the template rendered is admitted by the keys module,
    // in the same graph order (ctp:algorithmName).
    let rendered = rendered_algs();
    let plane: Vec<&'static str> = AlgorithmId::all().iter().map(|a| a.as_str()).collect();
    assert_eq!(rendered, plane);
}

/// The graph's algorithm admit list in graph order, as rendered from the
/// e2e.rq aggregate. AG1 lane: ED25519 (RFC 8032) and ES256K (RFC 8812)
/// joined the graph — keys.rs was advanced to the same admit list in the
/// same graph order (name-sorted: ED25519, ES256, ES256+ML-DSA-65, ES256K,
/// ML-DSA-65, SLH-DSA-SHA2-128s).
fn rendered_algs() -> Vec<&'static str> {
    vec![
        AlgorithmId::Ed25519.as_str(),
        AlgorithmId::Es256.as_str(),
        AlgorithmId::HybridEs256MlDsa65.as_str(),
        AlgorithmId::Es256k.as_str(),
        AlgorithmId::MlDsa65.as_str(),
        AlgorithmId::SlhDsa128s.as_str(),
    ]
}
