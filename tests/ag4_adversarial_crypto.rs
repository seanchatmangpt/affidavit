//! AG4 adversarial crypto court (hostile-integrator lane).
//!
//! Each attack executes over the REAL engine, real keys, and real wire
//! forms. A test PASSING means the attack was REFUSED; a FAILING test is a
//! WORKED attack. Attacks that worked against the pre-fix state are pinned
//! here as refusal courts for the AG4 typed fixes.
//!
//! Lanes: (1) alg confusion, (2) kid manipulation, (3) JCS pitfalls,
//! (4) domain lifting, (5) JWKS kid injection, (6) quorum manipulation,
//! (7) certified receipts; plus ES256K malleability on the AG1 path.

#![cfg(all(
    feature = "crypto-trust",
    feature = "certified-receipts",
    feature = "ed25519",
    feature = "secp256k1"
))]

use affidavit::crypto_trust_canonical::{digest, DOMAIN_TAG};
use affidavit::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_jwks::{export_jwks, JwksError};
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
    KeyFingerprint, KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
};
use affidavit::crypto_trust_lifecycle::RevocationList;
use affidavit::crypto_trust_quorum::{QuorumEngine, QuorumError, SignatureShare};
use affidavit::crypto_trust_verify::{
    CryptographicStanding, TrustPolicy, VerificationEngine, VerifyRefusal,
};
use affidavit::receipts_certified::{
    build_canonical_subject, certify_paid_delivery_payload, verify_certified_paid_delivery,
    PAID_DELIVERY_DOMAIN,
};
use affidavit::secp256k1_witness::{verify_ecdsa, WitnessSigningKey};

fn es_signer(seed_byte: u8) -> Es256SigningKey {
    let mut seed = [0x11u8; 32];
    seed[0] = seed_byte;
    Es256SigningKey::from_seed(&seed).expect("valid fixture scalar")
}

fn register(
    registry: &mut InMemoryKeyRegistry,
    algorithm: AlgorithmId,
    material: PublicKeyMaterial,
    custodian: &str,
) -> KeyId {
    let fingerprint = fingerprint_public_key(algorithm, &material);
    let record = KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm,
        fingerprint,
        custodian: CustodianIdentity {
            subject: custodian.to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: material,
        created_epoch: 1_700_000_000,
    };
    let kid = record.id.clone();
    registry.register(record).expect("fixture key registers");
    kid
}

fn engine(registry: &InMemoryKeyRegistry) -> VerificationEngine {
    VerificationEngine::new(
        registry.clone(),
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(1_700_000_500),
    )
}

fn envelope_for(
    algorithm: AlgorithmId,
    kid: &KeyId,
    subject_digest: [u8; 32],
) -> SignatureEnvelope {
    SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm,
        key_id: kid.clone(),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce: [0u8; 16],
        not_before: 0,
        expires_at: 4_102_444_800,
        subject_digest,
        audience: "affidavit.cli".to_string(),
    }
}

// ---------------------------------------------------------------
// (1) Algorithm confusion
// ---------------------------------------------------------------

/// Ed25519 key presented under an ES256K envelope: the registry record's
/// algorithm disagrees with the envelope's SIGNED algorithm claim.
#[test]
fn alg_confusion_ed_key_under_es256k_envelope_refused() {
    let mut registry = InMemoryKeyRegistry::new();
    let kp = affidavit::ed25519_witness::WitnessKeyPair::generate();
    let kid_ed = register(
        &mut registry,
        AlgorithmId::Ed25519,
        PublicKeyMaterial::Ed25519(kp.public().to_vec()),
        "attacker",
    );
    let env = envelope_for(AlgorithmId::Es256k, &kid_ed, [7u8; 32]);
    match engine(&registry).verify_envelope(&env, &[0u8; 64]) {
        Err(VerifyRefusal::UnknownKey(kid)) => assert_eq!(kid, kid_ed.to_string()),
        other => panic!("ATTACK WORKED (alg confusion): {other:?}"),
    }
}

/// An ES256K fixed-width r||s signature presented under an Ed25519 envelope
/// over the matching Ed25519 key: dalek must decide INVALID.
#[test]
fn alg_confusion_es256k_sig_bytes_under_ed25519_envelope_decided_invalid() {
    let mut registry = InMemoryKeyRegistry::new();
    let kp = affidavit::ed25519_witness::WitnessKeyPair::generate();
    let kid = register(
        &mut registry,
        AlgorithmId::Ed25519,
        PublicKeyMaterial::Ed25519(kp.public().to_vec()),
        "custodian-ed",
    );
    let env = envelope_for(
        AlgorithmId::Ed25519,
        &kid,
        digest(DOMAIN_TAG, &[b"subject-a"]),
    );
    // A real Ed25519 signature over a DIFFERENT message (right shape, wrong
    // pre-image) — the same confusion shape as swapping in foreign bytes.
    let foreign = kp.sign(b"some other pre-image entirely");
    match engine(&registry).verify_envelope(&env, &foreign) {
        Ok(v) => assert_eq!(v.standing, CryptographicStanding::Invalid, "ATTACK WORKED"),
        other => panic!("expected decided Invalid, got {other:?}"),
    }
}

// ---------------------------------------------------------------
// (2) kid manipulation
// ---------------------------------------------------------------

/// Registry refuses a second record under an existing kid.
#[test]
fn duplicate_kid_registration_refused() {
    let mut registry = InMemoryKeyRegistry::new();
    let kid = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(es_signer(0x01).public_key_sec1()),
        "a",
    );
    let dup = KeyRecord {
        id: kid.clone(),
        algorithm: AlgorithmId::Ed25519,
        fingerprint: KeyFingerprint([9u8; 32]),
        custodian: CustodianIdentity {
            subject: "b".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: PublicKeyMaterial::Ed25519(vec![2u8; 32]),
        created_epoch: 1_700_000_000,
    };
    match affidavit::crypto_trust_keys::KeyRegistry::register(&mut registry, dup) {
        Err(affidavit::crypto_trust_keys::RegistryError::Duplicate(id)) => assert_eq!(id, kid),
        other => panic!("ATTACK WORKED (duplicate kid admitted): {other:?}"),
    }
}

/// Rewriting the SIGNED key_id of a lawfully signed envelope invalidates the
/// signature (kid is inside the signed bytes).
#[test]
fn kid_swap_breaks_signature_decided_invalid() {
    let mut registry = InMemoryKeyRegistry::new();
    let kid = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(es_signer(0x02).public_key_sec1()),
        "custodian-a",
    );
    let other = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(es_signer(0x03).public_key_sec1()),
        "custodian-b",
    );
    let mut env = envelope_for(
        AlgorithmId::Es256,
        &kid,
        digest(DOMAIN_TAG, &[b"subject-a"]),
    );
    let sig = es_signer(0x02).sign(
        env.signing_input_checked()
            .expect("canonicalizes")
            .as_slice(),
    );
    env.key_id = other.clone();
    match engine(&registry).verify_envelope(&env, &sig) {
        Ok(v) => assert_eq!(
            v.standing,
            CryptographicStanding::Invalid,
            "ATTACK WORKED (kid swap verified)"
        ),
        other => panic!("expected decided Invalid, got {other:?}"),
    }
    let _ = kid;
}

/// An empty kid names no registered key: refused, never "verify against
/// everything".
#[test]
fn empty_kid_refused_unknown_key() {
    let registry = InMemoryKeyRegistry::new();
    let env = envelope_for(AlgorithmId::Es256, &KeyId(String::new()), [7u8; 32]);
    match engine(&registry).verify_envelope(&env, &[0u8; 8]) {
        Err(VerifyRefusal::UnknownKey(kid)) => assert_eq!(kid, ""),
        other => panic!("ATTACK WORKED (empty kid not UnknownKey): {other:?}"),
    }
}

// ---------------------------------------------------------------
// (3) JCS pitfalls
// ---------------------------------------------------------------

/// Duplicate JSON field in envelope wire bytes: serde must refuse, not take
/// the last occurrence.
#[test]
fn jcs_duplicate_field_refused() {
    let kid16 = "0".repeat(16);
    let doc = format!(
        r#"{{"version":"CTP-ENVELOPE-v1","version":"CTP-ENVELOPE-v0","algorithm":"ES256","key_id":"afk1_{kid16}","profile":"CLASSICAL","policy_epoch":1,"revocation_epoch":0,"generation":1,"nonce":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"not_before":0,"expires_at":4102444800,"subject_digest":[7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7],"audience":"affidavit.cli"}}"#
    );
    match SignatureEnvelope::from_bytes(doc.as_bytes()) {
        Err(affidavit::crypto_trust_envelope::EnvelopeError::Malformed(_)) => {}
        other => panic!("ATTACK WORKED (duplicate field admitted): {other:?}"),
    }
}

/// `policy_epoch: 1.0` must not coerce into the u64 field.
#[test]
fn jcs_float_epoch_refused() {
    let kid16 = "0".repeat(16);
    let doc = format!(
        r#"{{"version":"CTP-ENVELOPE-v1","algorithm":"ES256","key_id":"afk1_{kid16}","profile":"CLASSICAL","policy_epoch":1.0,"revocation_epoch":0,"generation":1,"nonce":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"not_before":0,"expires_at":4102444800,"subject_digest":[7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7],"audience":"affidavit.cli"}}"#
    );
    match SignatureEnvelope::from_bytes(doc.as_bytes()) {
        Err(affidavit::crypto_trust_envelope::EnvelopeError::Malformed(_)) => {}
        other => panic!("ATTACK WORKED (float epoch coerced): {other:?}"),
    }
}

/// NFC vs NFD spellings of the same visible subject digest differently:
/// fail-closed (availability note, not a forgery).
#[test]
fn jcs_nfc_nfd_subjects_digest_distinct() {
    let nfc = "caf\u{e9}";
    let nfd = "cafe\u{301}";
    assert_ne!(
        digest(DOMAIN_TAG, &[nfc.as_bytes()]),
        digest(DOMAIN_TAG, &[nfd.as_bytes()])
    );
}

// ---------------------------------------------------------------
// (4) Domain lifting on the paid-delivery domain
// ---------------------------------------------------------------

/// The domain string is INSIDE the canonical subject, hence inside the
/// envelope's signed subject_digest: a re-bind under a foreign domain
/// produces a different digest and cannot be minted or verified.
#[test]
fn domain_lifting_foreign_domain_refused() {
    let payload = "ab".repeat(32);
    let lawful = build_canonical_subject(&payload, "subject-a");
    let foreign = format!("attacker-paid-delivery/v1|subject-a|{payload}");
    assert_ne!(lawful, foreign);
    assert_ne!(
        digest(DOMAIN_TAG, &[lawful.as_bytes()]),
        digest(DOMAIN_TAG, &[foreign.as_bytes()])
    );
    assert!(lawful.starts_with(PAID_DELIVERY_DOMAIN));
}

// ---------------------------------------------------------------
// (5) JWKS kid injection
// ---------------------------------------------------------------

/// Two records sharing ONE kid string across DIFFERENT algorithms: the JWKS
/// export must refuse, because a consumer resolving kid -> key could pick
/// the wrong algorithm family and verify a cross-family forgery.
#[test]
fn jwks_duplicate_kid_across_algorithms_refused() {
    let kid = KeyId("afk1_deadbeefdeadbeef".to_string());
    let es = KeyRecord {
        id: kid.clone(),
        algorithm: AlgorithmId::Es256,
        fingerprint: KeyFingerprint([1u8; 32]),
        custodian: CustodianIdentity {
            subject: "a".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: PublicKeyMaterial::Es256Sec1(es_signer(0x42).public_key_sec1()),
        created_epoch: 1_700_000_000,
    };
    let ed = KeyRecord {
        id: kid.clone(),
        algorithm: AlgorithmId::Ed25519,
        fingerprint: KeyFingerprint([2u8; 32]),
        custodian: CustodianIdentity {
            subject: "b".to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: PublicKeyMaterial::Ed25519(vec![3u8; 32]),
        created_epoch: 1_700_000_000,
    };
    match export_jwks(&[es, ed]) {
        Err(JwksError::DuplicateKid(k)) => assert_eq!(k, kid.to_string()),
        other => panic!("ATTACK WORKED (duplicate-kid JWKS exported): {other:?}"),
    }
}

// ---------------------------------------------------------------
// (6) Quorum manipulation
// ---------------------------------------------------------------

/// Share reordering must be verdict-invariant.
#[test]
fn quorum_share_reorder_is_verdict_invariant() {
    let mut registry = InMemoryKeyRegistry::new();
    let a = es_signer(0x01);
    let b = es_signer(0x02);
    let c = es_signer(0x03);
    let kid_a = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(a.public_key_sec1()),
        "a",
    );
    let kid_b = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(b.public_key_sec1()),
        "b",
    );
    let kid_c = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(c.public_key_sec1()),
        "c",
    );
    let input = b"common signing input";
    let mk = |kid: &KeyId, s: &Es256SigningKey| SignatureShare {
        key_id: kid.clone(),
        algorithm: AlgorithmId::Es256,
        signature: s.sign(input),
    };
    let shares = [mk(&kid_a, &a), mk(&kid_b, &b), mk(&kid_c, &c)];
    let eng = QuorumEngine::new(&registry);
    let v1 = eng.verify_quorum(input, &shares, 2).expect("courts pass");
    let mut rev = shares.clone();
    rev.reverse();
    let v2 = eng.verify_quorum(input, &rev, 2).expect("courts pass");
    assert_eq!(v1, v2, "reorder changed the verdict");
    assert!(v1.satisfied);
}

/// Duplicate signer refused before any signature runs.
#[test]
fn quorum_duplicate_signer_refused() {
    let mut registry = InMemoryKeyRegistry::new();
    let a = es_signer(0x01);
    let kid_a = register(
        &mut registry,
        AlgorithmId::Es256,
        PublicKeyMaterial::Es256Sec1(a.public_key_sec1()),
        "a",
    );
    let input = b"input";
    let shares = [
        SignatureShare {
            key_id: kid_a.clone(),
            algorithm: AlgorithmId::Es256,
            signature: a.sign(input),
        },
        SignatureShare {
            key_id: kid_a.clone(),
            algorithm: AlgorithmId::Es256,
            signature: a.sign(input),
        },
    ];
    match QuorumEngine::new(&registry).verify_quorum(input, &shares, 2) {
        Err(QuorumError::DuplicateSigner(kid)) => assert_eq!(kid, kid_a.to_string()),
        other => panic!("ATTACK WORKED (duplicate signer counted): {other:?}"),
    }
}

/// AG1 parity: a legitimate Ed25519 share must COUNT toward quorum.
#[test]
fn quorum_ed25519_share_counts() {
    let mut registry = InMemoryKeyRegistry::new();
    let kp = affidavit::ed25519_witness::WitnessKeyPair::generate();
    let kid = register(
        &mut registry,
        AlgorithmId::Ed25519,
        PublicKeyMaterial::Ed25519(kp.public().to_vec()),
        "ed",
    );
    let input = b"common signing input";
    let share = SignatureShare {
        key_id: kid,
        algorithm: AlgorithmId::Ed25519,
        signature: kp.sign(input).to_vec(),
    };
    let verdict = QuorumEngine::new(&registry)
        .verify_quorum(input, &[share], 1)
        .expect("Ed25519 shares count toward quorum (AG1 parity gap if this refuses)");
    assert!(verdict.satisfied);
}

/// AG1 parity: a legitimate ES256K share must COUNT toward quorum.
#[test]
fn quorum_es256k_share_counts() {
    let mut registry = InMemoryKeyRegistry::new();
    let signing = WitnessSigningKey::from_seed(&[7u8; 32]).expect("seed");
    let kid = register(
        &mut registry,
        AlgorithmId::Es256k,
        PublicKeyMaterial::Es256kSec1(signing.public_key_sec1().to_vec()),
        "k1",
    );
    let input = b"common signing input";
    let share = SignatureShare {
        key_id: kid,
        algorithm: AlgorithmId::Es256k,
        signature: signing.sign(input).to_vec(),
    };
    let verdict = QuorumEngine::new(&registry)
        .verify_quorum(input, &[share], 1)
        .expect("ES256K shares count toward quorum (AG1 parity gap if this refuses)");
    assert!(verdict.satisfied);
}

// ---------------------------------------------------------------
// (7) Certified receipts
// ---------------------------------------------------------------

/// A receipt whose stored hash was rewritten to UPPERCASE hex refuses: the
/// hash law compares the recomputed lowercase canonical hash.
#[test]
fn certified_receipt_uppercase_hash_refused() {
    let signing = es_signer(0x31);
    let certified =
        certify_paid_delivery_payload(&"cd".repeat(32), "subject-a", &signing).expect("certify");
    let mut wire = serde_json::to_value(&certified.receipt).expect("receipt serializes");
    let upper = wire["receipt_hash"].as_str().expect("hex").to_uppercase();
    wire["receipt_hash"] = serde_json::Value::String(upper);
    match serde_json::from_value::<affidavit::crypto_trust_verify::CryptoStandingReceipt>(wire) {
        Err(_) => {}
        Ok(receipt) => match receipt.verify() {
            Err(affidavit::crypto_trust_verify::StandingReceiptError::ReceiptHashMismatch {
                ..
            }) => {}
            other => panic!("ATTACK WORKED (uppercase hash admitted): {other:?}"),
        },
    }
}

/// Whitespace-only subject must refuse like the empty subject (a certified
/// receipt must name a subject; " " names nobody).
#[test]
fn certified_whitespace_subject_refused() {
    let signing = es_signer(0x32);
    match certify_paid_delivery_payload(&"ef".repeat(32), "   ", &signing) {
        Err(VerifyRefusal::SubjectMismatch(_)) => {}
        other => panic!("ATTACK WORKED (whitespace subject certified): {other:?}"),
    }
}

/// RECEIPT-ENVELOPE LINKAGE: a CertifiedReceiptEnvelope whose `receipt`
/// component was SWAPPED for a valid receipt minted over a DIFFERENT
/// envelope must refuse. Pre-fix, verify_certified_paid_delivery never
/// re-audits the receipt nor binds it to the envelope, so the swap verifies.
#[test]
fn certified_receipt_envelope_swap_refused() {
    let signing = es_signer(0x33);
    let payload_a = "aa".repeat(32);
    let payload_b = "bb".repeat(32);
    let mut a =
        certify_paid_delivery_payload(&payload_a, "subject-a", &signing).expect("certify a");
    let b = certify_paid_delivery_payload(&payload_b, "subject-a", &signing).expect("certify b");
    a.receipt = b.receipt; // the swap
    match verify_certified_paid_delivery(&a, &payload_a, "subject-a") {
        Err(VerifyRefusal::Provider(msg)) => assert!(msg.contains("linkage"), "{msg}"),
        other => panic!("ATTACK WORKED (swapped receipt verified): {other:?}"),
    }
}

/// The envelope_commitment inside a minted receipt binds THIS envelope's
/// signing input — witnessed directly so the linkage court's equality is
/// non-vacuous.
#[test]
fn certified_receipt_commitment_binds_envelope_bytes() {
    let signing = es_signer(0x34);
    let a =
        certify_paid_delivery_payload(&"cc".repeat(32), "subject-a", &signing).expect("certify");
    let commitment = blake3::hash(
        a.envelope
            .signing_input_checked()
            .expect("canonicalizes")
            .as_slice(),
    )
    .to_hex()
    .to_string();
    assert_eq!(a.receipt.envelope_commitment, commitment);
    assert_eq!(
        a.receipt.subject,
        build_canonical_subject(&"cc".repeat(32), "subject-a")
    );
}

// ---------------------------------------------------------------
// ES256K signature malleability (AG1 envelope-verify path)
// ---------------------------------------------------------------

/// The ES256 signing path refuses high-s (malleability closure). The AG1
/// ES256K verifier must hold the same law: a flipped-sign re-present
/// (s -> n - s) of a lawful low-s signature must NOT verify.
#[test]
fn es256k_high_s_signature_refused() {
    let signing = WitnessSigningKey::from_seed(&[9u8; 32]).expect("seed");
    let msg = b"envelope pre-image bytes";
    let low = signing.sign(msg);
    let high = flip_to_high_s(&low);
    assert_ne!(low, high);
    // sanity: the low-s form verifies
    assert!(verify_ecdsa(&signing.public_key_sec1(), msg, &low).is_ok());
    assert!(
        verify_ecdsa(&signing.public_key_sec1(), msg, &high).is_err(),
        "ATTACK WORKED (high-s secp256k1 signature verified — malleable surface)"
    );
}

/// s -> n - s over the secp256k1 group order
/// n = 0xFFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141.
fn flip_to_high_s(sig64: &[u8; 64]) -> [u8; 64] {
    const N_MINUS_1: [u8; 32] = [
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFE, 0xBA, 0xAE, 0xDC, 0xE6, 0xAF, 0x48, 0xA0, 0x3B, 0xBF, 0xD2, 0x5E, 0x8C, 0xD0, 0x36,
        0x41, 0x40,
    ];
    let s = &sig64[32..];
    // n - s computed as a 32-byte big-int subtraction with borrow; the +1
    // relative to (n-1) is folded in by starting with n itself... we start
    // from N_MINUS_1 and borrow-correct.
    let mut out = [0u8; 64];
    out[..32].copy_from_slice(&sig64[..32]);
    let mut borrow = 0i16;
    for i in (0..32).rev() {
        let diff = 256 + (N_MINUS_1[i] as i16) - (s[i] as i16) - borrow;
        out[32 + i] = (diff & 0xFF) as u8;
        borrow = if diff < 256 { 1 } else { 0 };
    }
    // add 1: (n-1) - s + 1 = n - s
    let mut carry = 1i16;
    for i in (0..32).rev() {
        let sum = out[32 + i] as i16 + carry;
        out[32 + i] = (sum & 0xFF) as u8;
        carry = sum >> 8;
    }
    if s.iter().all(|&b| b == 0) {
        // s == 0 is degenerate; return something distinct so assert_ne holds.
        out[63] = out[63].wrapping_add(1);
    }
    out
}
