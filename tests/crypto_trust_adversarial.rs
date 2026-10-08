#![cfg(feature = "crypto-trust")]
//! W3-L7 adversarial mutation court over the whole cryptographic trust plane
//! (keys, canonicalization, envelope, ES256/PQC providers, verification,
//! sealing). House falsifier pattern: teeth first (the honest construction
//! must pass), then ONE mutation per test, then the EXACT named refusal or
//! verdict is asserted — never a bare `is_err`.
//!
//! Wire-level note: `crypto_trust_store` is NOT declared in `src/lib.rs`, so
//! store/checksum surfaces are out of scope for this lane (documented skip).
//!
//! Determinism law: no wall-clock; the court instant is `NOW`, and every
//! integer stays inside JCS's 2^53 so pre-images are the real
//! domain-separated documents.

use affidavit::chain::ChainAssembler;
use affidavit::crypto_trust_canonical::jcs;
use affidavit::crypto_trust_envelope::{
    EnvelopeError, NonceJournal, SignatureEnvelope, ENVELOPE_VERSION,
};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
    KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial, RegistryError,
};
use affidavit::crypto_trust_lifecycle::{
    RevocationList, MAX_REVOCATION_STALENESS_SECONDS, NONCE_WINDOW_SECONDS,
};
use affidavit::crypto_trust_pqc::{
    hybrid_sign, hybrid_verify, ml_dsa65_from_seed, ml_dsa65_sign, ml_dsa65_verify,
    slh_dsa128s_from_seed, slh_dsa128s_sign, slh_dsa128s_verify, HybridSecret, HybridSignature,
    ML_DSA_65_PUBLIC_KEY_LEN, ML_DSA_65_SIGNATURE_LEN,
};
use affidavit::crypto_trust_seal::{
    seal_receipt, subject_digest_of, verify_sealed, SealError, SealedReceipt,
};
use affidavit::crypto_trust_verify::{
    CryptographicStanding, TrustPolicy, VerificationEngine, VerifyRefusal,
};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::Blake3Hash;
use affidavit::verifier::verify;

/// The court's verifier-clock instant (trust-plane epoch seconds).
const NOW: u64 = 1_700_000_500;

/// A real one-event provenance receipt from the canonical assembler.
fn real_receipt(payload: &[u8]) -> affidavit::types::Receipt {
    let mut assembler = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let event = build_event(
        "lane7.op",
        vec![object_ref("adversarial-receipt", "artifact")],
        payload,
        &mut counter,
    )
    .expect("event is well-formed");
    assembler.append(event).expect("append admitted");
    assembler.finalize()
}

/// A deterministic ES256 signing key and its registry record.
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
        created_epoch: NOW - 1_000,
    };
    (signing, record)
}

/// A registry record for `material` attributed to `algorithm`.
fn record_for(algorithm: AlgorithmId, material: PublicKeyMaterial) -> KeyRecord {
    let fingerprint = fingerprint_public_key(algorithm, &material);
    KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm,
        fingerprint,
        custodian: CustodianIdentity {
            subject: "subject-a".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: material,
        created_epoch: NOW - 1_000,
    }
}

/// A fresh, in-window ES256 envelope naming `kid` with nonce `[tag; 16]`.
fn envelope(kid: &KeyId, nonce_tag: u8) -> SignatureEnvelope {
    SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: kid.clone(),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce: [nonce_tag; 16],
        not_before: NOW - 1_000,
        expires_at: NOW + 1_000,
        subject_digest: [0x22; 32],
        // An audience admitted by the graph-default policy — the court's
        // target is envelope/signature tamper adjudication, never audience
        // policy, which the engine would refuse before adjudicating.
        audience: "affidavit.cli".to_string(),
    }
}

/// An engine holding `record` under the graph-default policy, clock at NOW.
fn engine_with(record: &KeyRecord) -> VerificationEngine {
    let mut registry = InMemoryKeyRegistry::new();
    registry.register(record.clone()).expect("register");
    VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(NOW),
    )
}

// ── teeth: the honest constructions pass, so no tamper test is vacuous ─────

#[test]
fn teeth_valid_envelope_signature_verifies() {
    let (signing, record) = es256_fixture(1);
    let env = envelope(&record.id, 0x01);
    let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
    let verdict = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("honest envelope must verify");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);
    assert_eq!(verdict.key_id, Some(record.id.clone()));
    assert_eq!(verdict.subject_digest, [0x22; 32]);
}

#[test]
fn teeth_valid_receipt_seals_and_opens() {
    let receipt = real_receipt(b"lane7-seal-teeth");
    assert!(verify(&receipt).accepted, "base receipt must certify");

    let (signing, record) = es256_fixture(2);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let mut env = envelope(&record.id, 0x02);
    env.subject_digest = subject;
    let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
    let sealed = seal_receipt(&receipt, env, signature).expect("lawful seal");

    let engine = engine_with(&record);
    let verdict = verify_sealed(&sealed, &engine).expect("sealed receipt must open");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);
    assert_eq!(verdict.subject_digest, subject);
}

#[test]
fn teeth_valid_key_registers() {
    let (_, record) = es256_fixture(3);
    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(record.clone())
        .expect("first registration");
    assert!(registry.contains(&record.id));
    assert_eq!(registry.lookup(&record.id), Some(&record));
}

#[test]
fn teeth_valid_pqc_envelopes_verify() {
    // ML-DSA-65 envelope: sign the real pre-image, verify VALID.
    let mldsa_seed = [0xA5u8; 32];
    let kp = ml_dsa65_from_seed(&mldsa_seed);
    let record = record_for(
        AlgorithmId::MlDsa65,
        PublicKeyMaterial::MlDsa65(kp.public.clone()),
    );
    let mut env = envelope(&record.id, 0x03);
    env.algorithm = AlgorithmId::MlDsa65;
    env.profile = CryptoProfile::Pqc;
    let signature = ml_dsa65_sign(
        &mldsa_seed,
        &env.signing_input_checked().expect("pre-image"),
        &[0x5A; 32],
    )
    .expect("ml-dsa sign");
    let verdict = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("honest ML-DSA envelope must verify");
    assert_eq!(verdict.standing, CryptographicStanding::Valid);

    // SLH-DSA-SHA2-128s envelope: same law.
    let slh_seeds = [0x11u8; 48];
    let slh_kp = slh_dsa128s_from_seed(&slh_seeds);
    let slh_record = record_for(
        AlgorithmId::SlhDsa128s,
        PublicKeyMaterial::SlhDsa128s(slh_kp.public.clone()),
    );
    let mut slh_env = envelope(&slh_record.id, 0x04);
    slh_env.algorithm = AlgorithmId::SlhDsa128s;
    slh_env.profile = CryptoProfile::Pqc;
    let slh_signature = slh_dsa128s_sign(
        &slh_seeds,
        &slh_env.signing_input_checked().expect("pre-image"),
    )
    .expect("slh-dsa sign");
    let slh_verdict = engine_with(&slh_record)
        .verify_envelope(&slh_env, &slh_signature)
        .expect("honest SLH-DSA envelope must verify");
    assert_eq!(slh_verdict.standing, CryptographicStanding::Valid);
}

// ── envelope field tampering: all 12 fields, exact expected outcome ─────────

#[test]
fn envelope_field_tamper_is_exactly_adjudicated_per_field() {
    type Mutation = fn(SignatureEnvelope) -> SignatureEnvelope;

    // The exact law per tampered field, over the unsigned-tamper attack
    // (attacker flips one field of a signed envelope and replays the ORIGINAL
    // signature).
    enum Expected {
        /// Decided negative: Ok with standing INVALID.
        Invalid,
        /// Refusal: the registry refuses; the evidence names the key the
        /// ENVELOPE named (`env.key_id`) — the key that could not be found.
        UnknownKeyOfEnvelopeKid,
        /// Refusal: the window gate fires with the mutated instant.
        NotYetValid(u64),
        /// Refusal: the window gate fires with the mutated instant.
        Expired(u64),
    }

    let cases: &[(&str, Mutation, Expected)] = &[
        // version is admission-checked by from_bytes, not by the engine: an
        // unsigned version flip only breaks the signature -> decided INVALID.
        (
            "version",
            |mut e| {
                e.version = "CTP-ENVELOPE-v0".to_string();
                e
            },
            Expected::Invalid,
        ),
        // The registry record serves ES256; an envelope claiming ML-DSA-65
        // disagrees with the record -> refusal before any signature work.
        (
            "algorithm",
            |mut e| {
                e.algorithm = AlgorithmId::MlDsa65;
                e
            },
            Expected::UnknownKeyOfEnvelopeKid,
        ),
        (
            "key_id",
            |mut e| {
                e.key_id = KeyId("afk1_ghostghostghost0".to_string());
                e
            },
            Expected::UnknownKeyOfEnvelopeKid,
        ),
        // Every profile is at/above the CLASSICAL floor, so the unsigned
        // profile flip reaches the signature check and decides INVALID.
        (
            "profile",
            |mut e| {
                e.profile = CryptoProfile::Hybrid;
                e
            },
            Expected::Invalid,
        ),
        (
            "policy_epoch",
            |mut e| {
                e.policy_epoch = 42;
                e
            },
            Expected::Invalid,
        ),
        // Nothing is revoked in this fixture, so the epoch flip is decided at
        // the signature stage.
        (
            "revocation_epoch",
            |mut e| {
                e.revocation_epoch = 9;
                e
            },
            Expected::Invalid,
        ),
        (
            "generation",
            |mut e| {
                e.generation = 99;
                e
            },
            Expected::Invalid,
        ),
        // The tampered nonce is journaled fresh (replay gate passes), then the
        // signature decides INVALID.
        (
            "nonce",
            |mut e| {
                e.nonce = [0x77; 16];
                e
            },
            Expected::Invalid,
        ),
        (
            "not_before",
            |mut e| {
                e.not_before = NOW + 10_000;
                e
            },
            Expected::NotYetValid(NOW + 10_000),
        ),
        (
            "expires_at",
            |mut e| {
                e.expires_at = NOW - 1;
                e
            },
            Expected::Expired(NOW - 1),
        ),
        (
            "subject_digest",
            |mut e| {
                e.subject_digest = [0x33; 32];
                e
            },
            Expected::Invalid,
        ),
        (
            "audience",
            |mut e| {
                // A DIFFERENT allowlisted audience: an unallowlisted audience
                // would refuse adjudication (AudienceRefused) before the
                // unsigned tamper could be decided INVALID — this case's
                // target is the signature, not audience policy.
                e.audience = "affidavit.seal".to_string();
                e
            },
            Expected::Invalid,
        ),
    ];
    assert_eq!(cases.len(), 12, "the graph declares exactly 12 fields");

    let (signing, record) = es256_fixture(4);

    // Teeth: the honest presentation of the same shape verifies.
    let honest_env = envelope(&record.id, 0x40);
    let honest_sig = signing.sign(&honest_env.signing_input_checked().expect("pre-image"));
    let honest = engine_with(&record)
        .verify_envelope(&honest_env, &honest_sig)
        .expect("honest presentation must verify");
    assert_eq!(honest.standing, CryptographicStanding::Valid);

    for (field, mutate, expected) in cases {
        // The signature covers the ORIGINAL envelope; the tampered envelope is
        // presented with it.
        let original = envelope(&record.id, 0x41);
        let signature = signing.sign(
            &original
                .signing_input_checked()
                .expect("original pre-image"),
        );
        let tampered = mutate(envelope(&record.id, 0x41));

        // One fresh engine per field: journal state must not couple cases.
        let outcome = engine_with(&record).verify_envelope(&tampered, &signature);
        match (&outcome, expected) {
            (Ok(verdict), Expected::Invalid) => assert_eq!(
                verdict.standing,
                CryptographicStanding::Invalid,
                "{field}: unsigned tamper must decide INVALID, got {verdict:?}"
            ),
            (Err(VerifyRefusal::UnknownKey(kid)), Expected::UnknownKeyOfEnvelopeKid) => {
                assert_eq!(kid, &tampered.key_id.to_string(), "{field}")
            }
            (Err(VerifyRefusal::NotYetValid(nb)), Expected::NotYetValid(want)) => {
                assert_eq!(nb, want, "{field}")
            }
            (Err(VerifyRefusal::Expired(ex)), Expected::Expired(want)) => {
                assert_eq!(ex, want, "{field}")
            }
            (_, _) => panic!("{field}: unexpected outcome {outcome:?}"),
        }
    }
}

// ── signature byte tampering: first / middle / last byte ────────────────────

#[test]
fn signature_first_byte_tamper_is_provider_refusal() {
    let (signing, record) = es256_fixture(5);
    let env = envelope(&record.id, 0x50);
    let mut signature = signing.sign(&env.signing_input_checked().expect("pre-image"));

    // Teeth: unflipped verifies.
    let teeth = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("honest signature verifies");
    assert_eq!(teeth.standing, CryptographicStanding::Valid);

    // Flip the DER SEQUENCE tag: the structure is refused before the curve.
    signature[0] ^= 0xFF;
    let outcome = engine_with(&record).verify_envelope(&env, &signature);
    match outcome {
        Err(VerifyRefusal::Provider(_)) => {}
        other => panic!("expected Provider refusal for broken DER, got {other:?}"),
    }
}

#[test]
fn signature_middle_byte_tamper_is_decided_invalid() {
    let (signing, record) = es256_fixture(6);
    let env = envelope(&record.id, 0x51);
    let mut signature = signing.sign(&env.signing_input_checked().expect("pre-image"));

    let teeth = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("honest signature verifies");
    assert_eq!(teeth.standing, CryptographicStanding::Valid);

    // The middle of a 70..=72-byte DER ECDSA-Sig-Value lies strictly inside
    // the r INTEGER content, so the structure stays valid and the law must
    // DECIDE: signature does not hold.
    let middle = signature.len() / 2;
    assert!(middle > 4, "fixture DER is long enough for a scalar middle");
    signature[middle] ^= 0x01;
    let verdict = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("a well-formed tampered signature is DECIDED, not refused");
    assert_eq!(verdict.standing, CryptographicStanding::Invalid);
}

#[test]
fn signature_last_byte_tamper_is_decided_invalid() {
    let (signing, record) = es256_fixture(7);
    let env = envelope(&record.id, 0x52);
    let mut signature = signing.sign(&env.signing_input_checked().expect("pre-image"));

    let teeth = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("honest signature verifies");
    assert_eq!(teeth.standing, CryptographicStanding::Valid);

    let last = signature.len() - 1;
    signature[last] ^= 0x01;
    let verdict = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("a well-formed tampered signature is DECIDED, not refused");
    assert_eq!(verdict.standing, CryptographicStanding::Invalid);
}

// ── key-substitution attacks ────────────────────────────────────────────────

#[test]
fn key_substitution_unregistered_signer_refused_unknown_key() {
    // Sign with key A; the registry holds ONLY key B. The envelope names A,
    // which the registry has never seen.
    let (signer_a, record_a) = es256_fixture(8);
    let (_, record_b) = es256_fixture(9);
    let env = envelope(&record_a.id, 0x53);
    let signature = signer_a.sign(&env.signing_input_checked().expect("pre-image"));

    let mut registry = InMemoryKeyRegistry::new();
    registry.register(record_b.clone()).expect("register B");
    let engine = VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(NOW),
    );
    match engine.verify_envelope(&env, &signature) {
        Err(VerifyRefusal::UnknownKey(kid)) => assert_eq!(kid, record_a.id.to_string()),
        other => panic!("expected UnknownKey for unregistered signer, got {other:?}"),
    }
}

#[test]
fn key_substitution_forged_envelope_for_registered_key_decided_invalid() {
    // The envelope NAMES key B (registered) but the signature is by key A's
    // secret: the engine must decide INVALID, never mint a VALID standing.
    let (signer_a, _) = es256_fixture(10);
    let (_, record_b) = es256_fixture(11);
    let env = envelope(&record_b.id, 0x54);
    let signature = signer_a.sign(&env.signing_input_checked().expect("pre-image"));

    let verdict = engine_with(&record_b)
        .verify_envelope(&env, &signature)
        .expect("adjudication happens for a known key");
    assert_eq!(verdict.standing, CryptographicStanding::Invalid);
    assert_eq!(verdict.key_id, Some(record_b.id));
}

#[test]
fn registry_duplicate_attacks_are_typed() {
    let (_, record) = es256_fixture(12);
    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(record.clone())
        .expect("first registration");

    // Same id: refused by name.
    match registry.register(record.clone()) {
        Err(RegistryError::Duplicate(id)) => assert_eq!(id, record.id),
        other => panic!("expected Duplicate, got {other:?}"),
    }
    // Same fingerprint under a different id: refused by fingerprint.
    let mut impostor = record.clone();
    impostor.id = KeyId("afk1_impostorimposter".to_string());
    match registry.register(impostor) {
        Err(RegistryError::DuplicateFingerprint(hex)) => {
            assert_eq!(hex, record.fingerprint.as_hex())
        }
        other => panic!("expected DuplicateFingerprint, got {other:?}"),
    }
}

// ── replay, revocation, staleness, expiry ───────────────────────────────────

#[test]
fn replay_same_envelope_twice_refuses_the_second_presentation() {
    let (signing, record) = es256_fixture(13);
    let engine = engine_with(&record);
    let env = envelope(&record.id, 0x55);
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));

    let first = engine
        .verify_envelope(&env, &signature)
        .expect("first presentation is fresh");
    assert_eq!(first.standing, CryptographicStanding::Valid);

    match engine.verify_envelope(&env, &signature) {
        Err(VerifyRefusal::ReplayRejected(detail)) => {
            assert!(
                detail.contains(&record.id.to_string()),
                "evidence names the kid"
            );
        }
        other => panic!("expected ReplayRejected, got {other:?}"),
    }
    // The refused repeat did not restamp the journal entry.
    assert_eq!(
        engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
        Some(NOW)
    );
}

#[test]
fn revocation_after_signing_within_grace_refuses_key_revoked() {
    let (signing, record) = es256_fixture(14);
    let kid = record.id.to_string();
    // Signed while the key was live (epoch 0, nothing revoked).
    let env = envelope(&record.id, 0x56);
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));

    let mut engine = engine_with(&record);
    // Revoked NOW: the signature's epoch 0 is stale against the new epoch,
    // but the revocation sits at delta 0 <= 300s (inside the grace), so the
    // epoch-freshness law admits — and the flat revocation law refuses.
    engine.revoke_key(&kid, NOW, "compromised".to_string());
    match engine.verify_envelope(&env, &signature) {
        Err(VerifyRefusal::KeyRevoked(refused)) => assert_eq!(refused, kid),
        other => panic!("expected KeyRevoked, got {other:?}"),
    }
}

#[test]
fn stale_epoch_forgery_past_grace_refuses_stale_revocation_epoch() {
    let (signing, record) = es256_fixture(15);
    let kid = record.id.to_string();
    let mut engine = engine_with(&record);
    // Revoked 301s ago: one second past the 300s staleness grace. An envelope
    // still naming epoch 0 is a stale-epoch forgery.
    engine.revoke_key(
        &kid,
        NOW - MAX_REVOCATION_STALENESS_SECONDS - 1,
        "compromised".to_string(),
    );
    let env = envelope(&record.id, 0x57);
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));
    match engine.verify_envelope(&env, &signature) {
        Err(VerifyRefusal::StaleRevocationEpoch(detail)) => {
            assert!(detail.contains(&kid), "evidence names the kid: {detail}");
            assert!(
                detail.contains("compromised"),
                "evidence carries the reason: {detail}"
            );
        }
        other => panic!("expected StaleRevocationEpoch, got {other:?}"),
    }
}

#[test]
fn expiry_forgery_refuses_expired() {
    let (signing, record) = es256_fixture(16);
    let mut env = envelope(&record.id, 0x58);
    env.expires_at = NOW - 1;
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));
    match engine_with(&record).verify_envelope(&env, &signature) {
        Err(VerifyRefusal::Expired(at)) => assert_eq!(at, NOW - 1),
        other => panic!("expected Expired, got {other:?}"),
    }
}

// ── envelope wire tampering ─────────────────────────────────────────────────

#[test]
fn envelope_wire_boundary_byte_tamper_refuses_malformed() {
    let env = envelope(&record_fixture_id(17), 0x59);
    let canonical = env.to_bytes().expect("canonical bytes");
    assert_eq!(
        canonical.first(),
        Some(&b'{'),
        "JCS object opens with brace"
    );
    assert_eq!(
        canonical.last(),
        Some(&b'}'),
        "JCS object closes with brace"
    );

    let mut first = canonical.clone();
    first[0] ^= 0x01;
    match SignatureEnvelope::from_bytes(&first) {
        Err(EnvelopeError::Malformed(_)) => {}
        other => panic!("expected Malformed for flipped first byte, got {other:?}"),
    }

    let mut last = canonical;
    let end = last.len() - 1;
    last[end] ^= 0x01;
    match SignatureEnvelope::from_bytes(&last) {
        Err(EnvelopeError::Malformed(_)) => {}
        other => panic!("expected Malformed for flipped last byte, got {other:?}"),
    }
}

/// Helper so the boundary-tamper test reads at the top of its section.
fn record_fixture_id(tag: u8) -> KeyId {
    es256_fixture(tag).1.id
}

#[test]
fn envelope_wire_single_byte_sweep_never_yields_a_valid_presentation() {
    let (signing, record) = es256_fixture(18);
    let env = envelope(&record.id, 0x5A);
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));
    let canonical = env.to_bytes().expect("canonical bytes");

    for idx in 0..canonical.len() {
        let mut tampered = canonical.clone();
        tampered[idx] ^= 0x01;
        match SignatureEnvelope::from_bytes(&tampered) {
            // Every flip is Malformed (structure/UTF-8 broken), WrongVersion
            // (a version digit), or a DIFFERENT well-formed envelope.
            Err(EnvelopeError::Malformed(_) | EnvelopeError::WrongVersion(_)) => {}
            Ok(parsed) => {
                assert_ne!(
                    parsed, env,
                    "byte {idx}: a flip must never reconstruct the original envelope"
                );
                let outcome = engine_with(&record).verify_envelope(&parsed, &signature);
                assert!(
                    !matches!(&outcome, Ok(v) if v.standing == CryptographicStanding::Valid),
                    "byte {idx}: flipped wire form must never verify VALID, got {outcome:?}"
                );
            }
            other => panic!("byte {idx}: unexpected outcome {other:?}"),
        }
    }
}

#[test]
fn wrong_version_wire_refused_with_payload() {
    let mut env = envelope(&record_fixture_id(19), 0x5B);
    env.version = "CTP-ENVELOPE-v0".to_string();
    let bytes = env.to_bytes().expect("v0 document still canonicalizes");
    match SignatureEnvelope::from_bytes(&bytes) {
        Err(EnvelopeError::WrongVersion(v)) => assert_eq!(v, "CTP-ENVELOPE-v0"),
        other => panic!("expected WrongVersion, got {other:?}"),
    }
}

// ── sealed-receipt tampering ────────────────────────────────────────────────

#[test]
fn sealed_receipt_base_tamper_refuses_deserialization_by_chain_law() {
    let receipt = real_receipt(b"lane7-base-tamper");
    let (signing, record) = es256_fixture(20);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let mut env = envelope(&record.id, 0x5C);
    env.subject_digest = subject;
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));
    let sealed = seal_receipt(&receipt, env, signature).expect("lawful seal");

    let json = serde_json::to_string(&sealed).expect("sealed serializes");
    let honest: Result<SealedReceipt, _> = serde_json::from_str(&json);
    assert!(honest.is_ok(), "the honest wire form must load (teeth)");

    // Flip one hex character of the base's first payload commitment without
    // re-chaining: the base Receipt's own deserializer recomputes the chain
    // and refuses, so the composite can never load.
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

    let outcome: Result<SealedReceipt, _> = serde_json::from_str(&tampered);
    let err = outcome.expect_err("tampered base must refuse to deserialize");
    let mapped: SealError = err.into();
    assert!(
        matches!(mapped, SealError::Serialization(_)),
        "the chain-law door-slam surfaces as the seal's Serialization variant"
    );
}

#[test]
fn sealed_receipt_rebound_envelope_refuses_subject_mismatch() {
    let receipt = real_receipt(b"lane7-rebound-base");
    let other = real_receipt(b"lane7-rebound-other");
    let (signing, record) = es256_fixture(21);

    let subject_other = subject_digest_of(&other).expect("digest other");
    let mut env_other = envelope(&record.id, 0x5D);
    env_other.subject_digest = subject_other;
    let signature = signing.sign(&env_other.signing_input_checked().expect("pre-image"));
    let sealed_other = seal_receipt(&other, env_other, signature).expect("lawful seal of other");

    // Re-point the attestation at a DIFFERENT base: the binding law refuses
    // before any verifier effort is spent.
    let rebound = SealedReceipt {
        base: receipt,
        envelope: sealed_other.envelope,
        signature: sealed_other.signature,
    };
    match verify_sealed(&rebound, &engine_with(&record)) {
        Err(SealError::SubjectMismatch) => {}
        other => panic!("expected SubjectMismatch, got {other:?}"),
    }
}

#[test]
fn sealed_receipt_signature_tamper_is_decided_invalid() {
    let receipt = real_receipt(b"lane7-seal-sig-tamper");
    let (signing, record) = es256_fixture(22);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let mut env = envelope(&record.id, 0x5E);
    env.subject_digest = subject;
    let mut signature = signing.sign(&env.signing_input_checked().expect("pre-image"));
    let sealed = seal_receipt(&receipt, env.clone(), signature.clone()).expect("lawful seal");

    // Teeth: untampered opens VALID on a fresh engine.
    let teeth = verify_sealed(&sealed, &engine_with(&record)).expect("opens");
    assert_eq!(teeth.standing, CryptographicStanding::Valid);

    let last = signature.len() - 1;
    signature[last] ^= 0x01;
    let tampered = seal_receipt(&receipt, env, signature).expect("lawful seal (sig not checked)");
    let verdict = verify_sealed(&tampered, &engine_with(&record)).expect("adjudication happens");
    assert_eq!(verdict.standing, CryptographicStanding::Invalid);
}

// ── fingerprint collision attempt ───────────────────────────────────────────

#[test]
fn fingerprint_collision_attempt_diverges() {
    let bytes = vec![7u8; 32];
    // Same bytes, different algorithm identity: the domain-separated digest
    // binds the algorithm name, so the collision attempt fails.
    let slh = fingerprint_public_key(
        AlgorithmId::SlhDsa128s,
        &PublicKeyMaterial::SlhDsa128s(bytes.clone()),
    );
    let mldsa = fingerprint_public_key(AlgorithmId::MlDsa65, &PublicKeyMaterial::MlDsa65(bytes));
    assert_ne!(slh, mldsa, "same bytes under two algorithms must diverge");

    // Same algorithm, different bytes: diverges.
    let es256 = AlgorithmId::Es256;
    assert_ne!(
        fingerprint_public_key(es256, &PublicKeyMaterial::Es256Sec1(vec![1; 33])),
        fingerprint_public_key(es256, &PublicKeyMaterial::Es256Sec1(vec![2; 33]))
    );
}

// ── JCS canonicalization attacks ────────────────────────────────────────────

#[test]
fn jcs_key_order_permutation_yields_identical_canonical_form() {
    let permuted: serde_json::Value =
        serde_json::from_str(r#"{"zebra":1,"alpha":{"y":true,"x":null}}"#).expect("parses");
    let sorted: serde_json::Value =
        serde_json::from_str(r#"{"alpha":{"x":null,"y":true},"zebra":1}"#).expect("parses");

    let canonical = jcs(&permuted).expect("canonicalizes");
    assert_eq!(canonical, jcs(&sorted).expect("canonicalizes"));
    assert_eq!(canonical, r#"{"alpha":{"x":null,"y":true},"zebra":1}"#);

    // Fixed point: the canonical form re-canonicalizes to itself.
    let reparsed: serde_json::Value = serde_json::from_str(&canonical).expect("canonical parses");
    assert_eq!(jcs(&reparsed).expect("re-canonicalizes"), canonical);
}

#[test]
fn jcs_unicode_key_order_follows_utf16_code_units() {
    // U+10000 (UTF-16 surrogate pair D800 DC00) must sort BEFORE U+FFFD
    // (single unit FFFD), although UTF-8 byte order says the opposite
    // (RFC 8785 §3.2.3). A byte-order canonicalizer would emit U+FFFD first.
    let value: serde_json::Value =
        serde_json::from_str(r#"{"\ufffd":1,"\ud800\udc00":2}"#).expect("parses");
    let out = jcs(&value).expect("canonicalizes");

    let mut expected = String::new();
    expected.push('{');
    expected.push('"');
    expected.push('\u{10000}');
    expected.push('"');
    expected.push_str(":2,");
    expected.push('"');
    expected.push('\u{FFFD}');
    expected.push('"');
    expected.push_str(":1}");
    assert_eq!(out, expected, "supplementary-plane key sorts before U+FFFD");

    let at_supplementary = out.find('\u{10000}').expect("supplementary key present");
    let at_bmp = out.find('\u{FFFD}').expect("BMP key present");
    assert!(at_supplementary < at_bmp);
}

// ── nonce-journal pruning: the documented window law ────────────────────────

#[test]
fn nonce_pruning_then_old_nonce_replay_is_the_documented_window_law() {
    let mut journal = NonceJournal::default();
    let nonce = [0x7Au8; 16];
    journal
        .record("kid-a", nonce, 1_000, NONCE_WINDOW_SECONDS)
        .expect("first sight admitted");

    // Prune at exactly the window boundary: the entry's delta == window, so
    // it evicts (the same condition under which a re-record would be admitted).
    let boundary = 1_000 + NONCE_WINDOW_SECONDS;
    assert_eq!(
        journal.prune(boundary, NONCE_WINDOW_SECONDS),
        1,
        "the boundary entry evicts, counted"
    );

    // The documented law: eviction is explicit, and an evicted nonce is
    // admissible again — this is the freshness window, not a bug.
    journal
        .record("kid-a", nonce, boundary, NONCE_WINDOW_SECONDS)
        .expect("evicted nonce is admissible after prune (documented window law)");
    assert_eq!(journal.seen("kid-a", &nonce), Some(boundary));

    // And the fresh stamp re-arms replay rejection inside the new window.
    match journal.record("kid-a", nonce, boundary + 1, NONCE_WINDOW_SECONDS) {
        Err(EnvelopeError::ReplayRejected(kid)) => assert_eq!(kid, "kid-a"),
        other => panic!("expected ReplayRejected against the restamped window, got {other:?}"),
    }
}

// ── determinism ─────────────────────────────────────────────────────────────

#[test]
fn verify_verdict_is_deterministic_across_fresh_engines() {
    let (signing, record) = es256_fixture(23);
    let env = envelope(&record.id, 0x5F);
    let signature = signing.sign(&env.signing_input_checked().expect("pre-image"));

    // One engine refuses the second presentation as a replay (journal law),
    // so determinism is witnessed across two FRESH engines over identical
    // inputs.
    let a = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("first adjudication");
    let b = engine_with(&record)
        .verify_envelope(&env, &signature)
        .expect("second adjudication");

    assert_eq!(a, b, "identical inputs must give identical verdicts");
    assert_eq!(
        format!("{a:?}"),
        format!("{b:?}"),
        "Debug bytes are identical"
    );

    // The signed material itself is deterministic.
    assert_eq!(env.signing_input(), env.signing_input());
    assert_eq!(
        env.to_bytes().expect("canonical"),
        env.to_bytes().expect("canonical")
    );
}

// ── PQC provider tamper matrix ──────────────────────────────────────────────

#[test]
fn pqc_provider_tamper_matrix_is_typed() {
    let msg: &[u8] = b"lane7 pqc tamper matrix";

    // ML-DSA-65: teeth, then byte-flip (decided false), then typed refusals.
    let mldsa = ml_dsa65_from_seed(&[0xA5u8; 32]);
    let mldsa_sig = ml_dsa65_sign(&mldsa.seed, msg, &[0x5Au8; 32]).expect("ml-dsa sign");
    assert!(
        ml_dsa65_verify(&mldsa.public, msg, &mldsa_sig).expect("ml-dsa teeth"),
        "untampered ML-DSA-65 must verify"
    );
    let mut mldsa_flip = mldsa_sig.clone();
    mldsa_flip[0] ^= 0x01;
    assert!(
        !ml_dsa65_verify(&mldsa.public, msg, &mldsa_flip).expect("fixed-length encoding parses"),
        "a flipped ML-DSA-65 signature byte must not verify"
    );
    let short_sig = vec![0u8; ML_DSA_65_SIGNATURE_LEN - 1];
    assert!(matches!(
        ml_dsa65_verify(&mldsa.public, msg, &short_sig),
        Err(affidavit::crypto_trust_pqc::PqcError::MalformedSignature)
    ));
    let short_pk = vec![0u8; ML_DSA_65_PUBLIC_KEY_LEN - 1];
    assert!(matches!(
        ml_dsa65_verify(&short_pk, msg, &mldsa_sig),
        Err(affidavit::crypto_trust_pqc::PqcError::MalformedPublicKey)
    ));

    // SLH-DSA-SHA2-128s: teeth, byte-flip, typed refusals.
    let slh = slh_dsa128s_from_seed(&[0x11u8; 48]);
    let slh_sig = slh_dsa128s_sign(&slh.seeds, msg).expect("slh-dsa sign");
    assert!(
        slh_dsa128s_verify(&slh.public, msg, &slh_sig).expect("slh-dsa teeth"),
        "untampered SLH-DSA must verify"
    );
    let mut slh_flip = slh_sig.clone();
    slh_flip[100] ^= 0x01;
    assert!(
        !slh_dsa128s_verify(&slh.public, msg, &slh_flip).expect("fixed-length encoding parses"),
        "a flipped SLH-DSA signature byte must not verify"
    );
    assert!(matches!(
        slh_dsa128s_verify(&slh.public, msg, &[0u8; 100]),
        Err(affidavit::crypto_trust_pqc::PqcError::MalformedSignature)
    ));
    assert!(matches!(
        slh_dsa128s_verify(&[0u8; 31], msg, &slh_sig),
        Err(affidavit::crypto_trust_pqc::PqcError::MalformedPublicKey)
    ));

    // Hybrid: BOTH halves must hold; stripping or tampering either half
    // breaks the composite.
    let secret = HybridSecret {
        es256: Es256SigningKey::from_seed(&[0x42u8; 32]).expect("valid p256 scalar"),
        mldsa65_seed: [0xA5u8; 32],
    };
    let es256_pk = secret.es256.public_key_sec1();
    let hybrid_sig = hybrid_sign(&secret, msg).expect("hybrid sign");
    assert!(
        hybrid_verify(&es256_pk, &mldsa.public, msg, &hybrid_sig).expect("hybrid teeth"),
        "untampered hybrid must verify"
    );

    // Strip the PQ half: ES256-only evidence must not pass as hybrid.
    let es_only = HybridSignature {
        es256_der: hybrid_sig.es256_der.clone(),
        mldsa65: Vec::new(),
    };
    assert!(
        !hybrid_verify(&es256_pk, &mldsa.public, msg, &es_only).unwrap_or(false),
        "a stripped hybrid must not verify (refusal counts as broken)"
    );

    // Tamper ONLY the ES256 half: typed refusal from the classical provider.
    let mut bad_es = hybrid_sig.clone();
    bad_es.es256_der[0] ^= 0xFF;
    assert!(matches!(
        hybrid_verify(&es256_pk, &mldsa.public, msg, &bad_es),
        Err(affidavit::crypto_trust_pqc::PqcError::Es256Half(_))
    ));

    // Tamper ONLY the ML-DSA half: the flipped byte may leave a well-formed
    // encoding (decided false) or fail structural decoding (typed refusal);
    // either way the composite is BROKEN — never a verdict of true.
    let mut bad_pq = hybrid_sig;
    let last = bad_pq.mldsa65.len() - 1;
    bad_pq.mldsa65[last] ^= 0x01;
    assert!(
        !hybrid_verify(&es256_pk, &mldsa.public, msg, &bad_pq).unwrap_or(false),
        "a hybrid with a tampered PQ half must not verify \
         (Ok(false) or a typed refusal both count as broken)"
    );

    // Cross-key: the PQ half under a different key must not verify.
    let other_mldsa = ml_dsa65_from_seed(&[0x3Cu8; 32]);
    let hybrid_again = hybrid_sign(&secret, msg).expect("hybrid sign");
    assert!(
        !hybrid_verify(&es256_pk, &other_mldsa.public, msg, &hybrid_again).expect("well-formed"),
        "a hybrid must not verify under a foreign PQ key"
    );
}

// ── chain-law witness without any external serializer ───────────────────────

#[test]
fn receipt_wire_tamper_refuses_at_the_public_chain_deserializer() {
    let receipt = real_receipt(b"lane7-chain-deserialize");
    let bytes = affidavit::chain::serialize_receipt(&receipt).expect("wire bytes");
    let honest: affidavit::types::Receipt =
        affidavit::chain::deserialize_receipt(&bytes).expect("honest wire loads (teeth)");
    assert_eq!(honest, receipt);

    // Flip one hex character of the first payload commitment in the wire
    // form: the public deserializer recomputes the chain and refuses.
    let marker = receipt.events[0].payload_commitment.as_hex().to_string();
    let flipped = format!(
        "{}{}",
        &marker[..marker.len() - 1],
        if marker.ends_with('0') { "1" } else { "0" }
    );
    let tampered_bytes: Vec<u8> = {
        let text = String::from_utf8(bytes).expect("wire form is UTF-8 JSON");
        let tampered = text.replace(&marker, &flipped);
        assert_ne!(text, tampered, "tamper must change the wire bytes");
        tampered.into_bytes()
    };
    assert!(
        affidavit::chain::deserialize_receipt(&tampered_bytes).is_err(),
        "tampered base wire must refuse at the chain-law deserializer"
    );
}

// `Blake3Hash` is referenced through the receipt field above; keep the import
// honest by exercising its hex rendering law here.
#[test]
fn blake3_hash_hex_is_lowercase_and_loads_back() {
    let hash = Blake3Hash::from_bytes(b"lane7");
    let hex = hash.as_hex();
    assert_eq!(hex.len(), 64);
    assert!(hex
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}
