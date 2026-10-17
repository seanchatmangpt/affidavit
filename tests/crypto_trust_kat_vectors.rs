#![cfg(feature = "crypto-trust")]

//! Multi-surface KAT vector fixture for the cryptographic trust plane
//! (wave 5, wave-1 lane 10).
//!
//! The raw-signature-primitive KAT corpus (`src/crypto_trust_kat.rs` +
//! `fixtures/crypto_trust_kat.json`) pins the primitives. This suite pins the
//! HIGHER surfaces so a second runtime can reproduce full pipeline bytes:
//!
//! - **envelope** — the full 12-field [`SignatureEnvelope`] wire form:
//!   `signing_input` (the exact domain-separated signing pre-image) and
//!   `to_bytes` (the JCS canonical document);
//! - **seal** — full wire artifacts: real `ChainAssembler` receipts over fixed
//!   events -> `subject_digest_of` -> envelope -> RFC 6979 ES256 signature ->
//!   [`SealedReceipt`] JCS bytes, verified through the real
//!   `crypto_trust_seal::verify_sealed` engine path;
//! - **rotation** — successor-signed [`RotationRecord`] wire bytes
//!   (CLASSICAL -> HYBRID), verified through `verify_rotation`.
//!
//! Everything is deterministic by law: domain-derived seeds
//! (`ctp.katvectors.*`, disjoint from the KAT corpus's `ctp.kat.*` labels) and
//! fixed time constants — no wall clock anywhere.
//!
//! Drift tripwire: in-test regeneration must reproduce the committed fixture
//! byte-for-byte. Any surface-law change (envelope fields, domain tag,
//! digest/canonicalization law, chain genesis, signing laws) breaks this
//! loudly and forces a conscious fixture regeneration that downstream
//! runtimes re-pin.
//!
//! Tamper teeth: one flipped byte per surface must refuse or decide invalid —
//! the vectors bind the bytes, not the idea of the bytes.
//!
//! Honest scope: `tools/verify_crypto_trust_kat.py` does NOT cover these
//! surfaces (its JCS port refuses numeric fields and it has no ES256-envelope
//! or ML-DSA path). At generation time, its pure-python `blake3_32` +
//! `domain_separated` primitives re-derived all 16 seed pre-images plus the
//! envelope nonce/subject laws and matched this fixture (recorded in the
//! fixture's `provenance.cross_check_python_tool`); extending the tool to
//! these surfaces is future work.
//!
//! Regeneration (the exact command carried in the fixture's provenance):
//!
//! ```text
//! cd <repo root> && cargo test --features crypto-trust --test crypto_trust_kat_vectors
//!   # export: AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE=1 cargo test --features crypto-trust \
//!   #          --test crypto_trust_kat_vectors export_fixture -- --exact --ignored
//! ```

use std::fs;
use std::path::PathBuf;

use affidavit::chain::{content_address, ChainAssembler};
use affidavit::crypto_trust_canonical::{digest, jcs, DOMAIN_TAG};
use affidavit::crypto_trust_envelope::{
    EnvelopeError, NonceJournal, SignatureEnvelope, ENVELOPE_FIELDS, ENVELOPE_VERSION,
};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
    KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
};
use affidavit::crypto_trust_lifecycle::RevocationList;
use affidavit::crypto_trust_pqc::{ml_dsa65_from_seed, HybridSecret};
use affidavit::crypto_trust_rotation::{rotate_es256_to_hybrid, verify_rotation, RotationError};
use affidavit::crypto_trust_seal::{
    seal_receipt, subject_digest_of, verify_sealed, SealError, SealedReceipt,
};
use affidavit::crypto_trust_verify::{CryptographicStanding, TrustPolicy, VerificationEngine};
use affidavit::ocel::{build_event, object_ref, qualified_object_ref, SeqCounter};
use serde_json::{json, Value};

const FIXTURE_RELPATH: &str = "fixtures/crypto_trust_kat_vectors.json";
const EXPORT_ENV: &str = "AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE";

/// Schema version of the multi-surface vector file.
const SCHEMA_VERSION: &str = "CTP-KATVECTORS-v1";
/// Deterministic seed count per surface (indexes 0..N).
const VECTOR_COUNT: usize = 3;

// Fixed time constants — no wall clock anywhere in the derivation. These MUST
// stay identical to the constants the fixture was generated under; the drift
// tripwire enforces it.
const NOT_BEFORE: u64 = 1_700_000_000;
const EXPIRES_AT: u64 = 4_102_444_800; // 2100-01-01T00:00:00Z, JCS-safe
const VERIFIER_NOW: u64 = 1_700_000_500;
const ROTATED_AT: u64 = 1_700_000_100;
const KEY_CREATED_EPOCH: u64 = 1_699_990_000;
const POLICY_EPOCH: u64 = 1;
const REVOCATION_EPOCH: u64 = 0;
const GENERATION: u32 = 1;
const AUDIENCE: &str = "affidavit.kat";
const SEAL_AUDIENCE: &str = "affidavit.seal";
const CUSTODIAN_SUBJECT: &str = "affidavit-kat-subject";

const GENERATING_COMMIT: &str = "48563e42e5c6f6aa0d600fc8baf0d7455c37f7ac";
const CROSS_CHECK_SENTINEL: &str = "PASS: 16/16 seed pre-images";

// ── derivation law (identical to the generator bin named in provenance) ─────

/// Domain-derived 32-byte key seed: the KAT-module house law applied to this
/// file's labels, so no seed collides with the raw-primitive KAT corpus.
fn seed32(index: usize, role: &str) -> [u8; 32] {
    digest(
        DOMAIN_TAG,
        &[
            b"ctp.katvectors.seed" as &[u8],
            role.as_bytes(),
            &(index as u64).to_be_bytes(),
        ],
    )
}

/// Domain-derived 16-byte envelope nonce for `index`.
fn nonce16(index: usize) -> [u8; 16] {
    let d = digest(
        DOMAIN_TAG,
        &[
            b"ctp.katvectors.nonce" as &[u8],
            &(index as u64).to_be_bytes(),
        ],
    );
    let mut nonce = [0u8; 16];
    nonce.copy_from_slice(&d[..16]);
    nonce
}

/// Domain-derived subject digest for the standalone envelope surface (the
/// seal surface binds the real `subject_digest_of` instead).
fn envelope_subject(index: usize) -> [u8; 32] {
    digest(
        DOMAIN_TAG,
        &[
            b"ctp.katvectors.subject" as &[u8],
            &(index as u64).to_be_bytes(),
        ],
    )
}

/// Domain-derived payload for seal event `ev` of vector `index`.
fn seal_payload(index: usize, ev: usize) -> Vec<u8> {
    digest(
        DOMAIN_TAG,
        &[
            b"ctp.katvectors.seal-payload" as &[u8],
            &(index as u64).to_be_bytes(),
            &(ev as u64).to_be_bytes(),
        ],
    )
    .to_vec()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_decode(hex: &str) -> Vec<u8> {
    assert!(hex.len() % 2 == 0, "hex must be byte-aligned: {hex}");
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).expect("valid hex digit"))
        .collect()
}

/// Key material for one envelope-vector family: public key, total seed bytes,
/// and the role names those bytes concatenate in order.
struct FamilyMaterial {
    algorithm: AlgorithmId,
    profile: CryptoProfile,
    public_key: PublicKeyMaterial,
    seed_bytes: Vec<u8>,
    seed_roles: &'static [&'static str],
}

fn family_material(index: usize) -> FamilyMaterial {
    match index {
        0 => {
            let seed = seed32(index, "es256");
            let key = Es256SigningKey::from_seed(&seed).expect("es256 fixture scalar");
            FamilyMaterial {
                algorithm: AlgorithmId::Es256,
                profile: CryptoProfile::Classical,
                public_key: PublicKeyMaterial::Es256Sec1(key.public_key_sec1()),
                seed_bytes: seed.to_vec(),
                seed_roles: &["es256"],
            }
        }
        1 => {
            let seed = seed32(index, "mldsa65");
            let kp = ml_dsa65_from_seed(&seed);
            FamilyMaterial {
                algorithm: AlgorithmId::MlDsa65,
                profile: CryptoProfile::Pqc,
                public_key: PublicKeyMaterial::MlDsa65(kp.public),
                seed_bytes: seed.to_vec(),
                seed_roles: &["mldsa65"],
            }
        }
        2 => {
            let es_seed = seed32(index, "hybrid-es256");
            let ml_seed = seed32(index, "hybrid-mldsa65");
            let es = Es256SigningKey::from_seed(&es_seed).expect("hybrid es256 fixture scalar");
            let ml = ml_dsa65_from_seed(&ml_seed);
            let mut seed_bytes = es_seed.to_vec();
            seed_bytes.extend_from_slice(&ml_seed);
            FamilyMaterial {
                algorithm: AlgorithmId::HybridEs256MlDsa65,
                profile: CryptoProfile::Hybrid,
                public_key: PublicKeyMaterial::Hybrid {
                    es256: es.public_key_sec1(),
                    mldsa65: ml.public,
                },
                seed_bytes,
                seed_roles: &["hybrid-es256", "hybrid-mldsa65"],
            }
        }
        other => panic!("no envelope family for index {other}"),
    }
}

fn profile_wire(profile: CryptoProfile) -> &'static str {
    match profile {
        CryptoProfile::Classical => "CLASSICAL",
        CryptoProfile::Hybrid => "HYBRID",
        CryptoProfile::Pqc => "PQC",
    }
}

/// The full 12-field envelope for `index` under its family law.
fn rebuild_envelope(index: usize) -> (SignatureEnvelope, FamilyMaterial, KeyId) {
    let family = family_material(index);
    let fingerprint = fingerprint_public_key(family.algorithm, &family.public_key);
    let kid = KeyId::from_fingerprint(&fingerprint);
    let envelope = SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: family.algorithm,
        key_id: kid.clone(),
        profile: family.profile,
        policy_epoch: POLICY_EPOCH,
        revocation_epoch: REVOCATION_EPOCH,
        generation: GENERATION,
        nonce: nonce16(index),
        not_before: NOT_BEFORE,
        expires_at: EXPIRES_AT,
        subject_digest: envelope_subject(index),
        audience: AUDIENCE.to_string(),
    };
    (envelope, family, kid)
}

/// A real ES256 signing key + its registry record for the seal surface.
fn seal_key(index: usize) -> (Es256SigningKey, KeyRecord) {
    let seed = seed32(index, "seal-es256");
    let signing = Es256SigningKey::from_seed(&seed).expect("seal fixture scalar");
    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    let record = KeyRecord {
        id: KeyId::from_fingerprint(&fingerprint),
        algorithm: AlgorithmId::Es256,
        fingerprint,
        custodian: CustodianIdentity {
            subject: CUSTODIAN_SUBJECT.to_string(),
            device: None,
            org: None,
        },
        origin: KeyOrigin::Generated,
        public_key: public,
        created_epoch: KEY_CREATED_EPOCH,
    };
    (signing, record)
}

/// A real multi-event receipt from the canonical assembler: two fixed events
/// per index (attest over an artifact, witness over a process), payload law
/// `ctp.katvectors.seal-payload`, seq from a fresh SeqCounter.
fn seal_receipt_base(index: usize) -> affidavit::types::Receipt {
    let mut assembler = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let attest = build_event(
        "kat.attest",
        vec![qualified_object_ref(
            format!("kat-object-{index}"),
            "artifact",
            "subject",
        )],
        &seal_payload(index, 0),
        &mut counter,
    )
    .expect("attest event is well-formed");
    assembler.append(attest).expect("append admitted");
    let witness = build_event(
        "kat.witness",
        vec![object_ref(format!("kat-witness-{index}"), "process")],
        &seal_payload(index, 1),
        &mut counter,
    )
    .expect("witness event is well-formed");
    assembler.append(witness).expect("append admitted");
    assembler.finalize()
}

/// The engine that adjudicates seal vector `index`: exactly the fixture key,
/// live window at VERIFIER_NOW, empty revocations, fresh nonce journal.
fn seal_engine(record: &KeyRecord) -> VerificationEngine {
    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(record.clone())
        .expect("register fixture key");
    VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(VERIFIER_NOW),
    )
}

/// Full wire artifact for seal vector `index`.
fn rebuild_sealed(index: usize) -> SealedReceipt {
    let (signing, record) = seal_key(index);
    let receipt = seal_receipt_base(index);
    let subject = subject_digest_of(&receipt).expect("subject digest");
    let envelope = SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: record.id.clone(),
        profile: CryptoProfile::Classical,
        policy_epoch: POLICY_EPOCH,
        revocation_epoch: REVOCATION_EPOCH,
        generation: GENERATION,
        nonce: nonce16(index),
        not_before: NOT_BEFORE,
        expires_at: EXPIRES_AT,
        subject_digest: subject,
        audience: SEAL_AUDIENCE.to_string(),
    };
    let signing_input = envelope
        .signing_input_checked()
        .expect("canonical pre-image");
    let signature = signing.sign(&signing_input);
    seal_receipt(&receipt, envelope, signature).expect("lawful seal")
}

/// Successor-signed rotation record for `index`.
fn rebuild_rotation(index: usize) -> affidavit::crypto_trust_rotation::RotationRecord {
    let old_seed = seed32(index, "rotation-old-es256");
    let old = Es256SigningKey::from_seed(&old_seed).expect("old fixture scalar");
    let new_es_seed = seed32(index, "rotation-new-es256");
    let new_ml_seed = seed32(index, "rotation-new-mldsa65");
    let secret = HybridSecret {
        es256: Es256SigningKey::from_seed(&new_es_seed).expect("successor es256 fixture scalar"),
        mldsa65_seed: new_ml_seed,
    };
    rotate_es256_to_hybrid(&old, &secret, ROTATED_AT).expect("admitted rotation")
}

/// The full canonical fixture document (before JCS).
fn build_fixture() -> Value {
    let envelope: Vec<Value> = (0..VECTOR_COUNT)
        .map(|index| {
            let (envelope, family, kid) = rebuild_envelope(index);
            let signing_input = envelope
                .signing_input_checked()
                .expect("canonical pre-image");
            let canonical_bytes = envelope.to_bytes().expect("canonical document");
            json!({
                "algorithm": family.algorithm.as_str(),
                "audience": AUDIENCE,
                "canonical_bytes_hex": hex_encode(&canonical_bytes),
                "expires_at": EXPIRES_AT,
                "generation": GENERATION,
                "key_id": kid.to_string(),
                "nonce_hex": hex_encode(&nonce16(index)),
                "not_before": NOT_BEFORE,
                "policy_epoch": POLICY_EPOCH,
                "profile": profile_wire(family.profile),
                "public_key_hex": hex_encode(&family.public_key.canonical_bytes()),
                "revocation_epoch": REVOCATION_EPOCH,
                "seed_hex": hex_encode(&family.seed_bytes),
                "seed_roles": family.seed_roles,
                "signing_input_hex": hex_encode(&signing_input),
                "subject_digest_hex": hex_encode(&envelope_subject(index)),
                "vector_id": format!("env-{index:03}-{}", family.algorithm.as_str()),
            })
        })
        .collect();
    let rotation: Vec<Value> = (0..VECTOR_COUNT)
        .map(|index| {
            let old_seed = seed32(index, "rotation-old-es256");
            let new_es_seed = seed32(index, "rotation-new-es256");
            let new_ml_seed = seed32(index, "rotation-new-mldsa65");
            let record = rebuild_rotation(index);
            let secret_es = Es256SigningKey::from_seed(&new_es_seed).expect("es fixture scalar");
            let es_pk = secret_es.public_key_sec1();
            let ml_pk = ml_dsa65_from_seed(&new_ml_seed).public;
            let record_value = serde_json::to_value(&record).expect("record serializes");
            let canonical = jcs(&record_value).expect("record JCS bytes");
            json!({
                "from_profile": "CLASSICAL",
                "new_es256_seed_hex": hex_encode(&new_es_seed),
                "new_key_id": record.new_key_id,
                "new_mldsa65_seed_hex": hex_encode(&new_ml_seed),
                "new_public_key_es256_hex": hex_encode(&es_pk),
                "new_public_key_mldsa65_hex": hex_encode(&ml_pk),
                "old_key_id": record.old_key_id,
                "old_seed_hex": hex_encode(&old_seed),
                "record_json_hex": hex_encode(canonical.as_bytes()),
                "rotated_at": ROTATED_AT,
                "to_profile": "HYBRID",
                "vector_id": format!("rot-{index:03}-CLASSICAL-HYBRID"),
            })
        })
        .collect();
    let seal: Vec<Value> = (0..VECTOR_COUNT)
        .map(|index| {
            let (_, record) = seal_key(index);
            let sealed = rebuild_sealed(index);
            let address = content_address(&sealed.base).expect("content address");
            let sealed_value = serde_json::to_value(&sealed).expect("sealed serializes");
            let canonical = jcs(&sealed_value).expect("sealed JCS bytes");
            json!({
                "algorithm": "ES256",
                "audience": SEAL_AUDIENCE,
                "chain_hash_hex": sealed.base.chain_hash.as_hex(),
                "content_address_hex": address.as_hex(),
                "envelope_signing_input_hex": hex_encode(&sealed.envelope.signing_input_checked().expect("canonical pre-image")),
                "events": [
                    {"event_type": "kat.attest", "objects": [
                        {"id": format!("kat-object-{index}"), "obj_type": "artifact", "qualifier": "subject"}]},
                    {"event_type": "kat.witness", "objects": [
                        {"id": format!("kat-witness-{index}"), "obj_type": "process"}]}
                ],
                "key_id": record.id.to_string(),
                "nonce_hex": hex_encode(&nonce16(index)),
                "public_key_hex": hex_encode(&record.public_key.canonical_bytes()),
                "sealed_json_hex": hex_encode(canonical.as_bytes()),
                "seed_hex": hex_encode(&seed32(index, "seal-es256")),
                "seed_role": "seal-es256",
                "signature_hex": hex_encode(&sealed.signature),
                "subject_digest_hex": hex_encode(&sealed.envelope.subject_digest),
                "vector_id": format!("seal-{index:03}-ES256"),
            })
        })
        .collect();
    json!({
        "provenance": {
            "branch": "feat/v26.9.28-crypto-trust-plane",
            "cross_check_python_tool": "PASS: 16/16 seed pre-images re-derived with tools/verify_crypto_trust_kat.py pure-python blake3_32 + domain_separated match the fixture seed fields under the ctp.katvectors.seed law (envelope 4, seal 3, rotation 9); the envelope nonce (first 16 bytes) and standalone subject laws likewise re-derived and match (3/3); the tool itself still covers only the raw-primitive KAT surface.",
            "determinism": [
                "seeds: seed32(index, role) = BLAKE3(domain_separated(DOMAIN_TAG, [\"ctp.katvectors.seed\", role, index_be8])), DOMAIN_TAG=affidavit.crypto-trust-plane.v1; roles disjoint from the raw-primitive KAT corpus",
                "envelope nonces: BLAKE3(domain_separated(DOMAIN_TAG, [\"ctp.katvectors.nonce\", index_be8]))[..16]; standalone envelope subjects: BLAKE3(domain_separated(DOMAIN_TAG, [\"ctp.katvectors.subject\", index_be8]))",
                "seal event payloads: BLAKE3(domain_separated(DOMAIN_TAG, [\"ctp.katvectors.seal-payload\", index_be8, event_be8])); events appended to a fresh ChainAssembler+SeqCounter in the order given in each vector's events array",
                "no wall clock: not_before=1700000000, expires_at=4102444800, verifier_now=1700000500, rotated_at=1700000100, key created_epoch=1699990000 are fixed constants",
                "ES256 signing is RFC 6979 deterministic ECDSA; rotation's ML-DSA-65 half uses the domain-bound explicit randomizer (hybrid_sign); receipt chain genesis binds the package version",
                "envelope subjects on the seal surface are the real subject_digest_of(receipt); the standalone envelope surface uses the ctp.katvectors.subject law above",
            ],
            "generating_commit": GENERATING_COMMIT,
            "generator": "scratch bin katvec-gen (lane W5-W1-L10) over the rendered affidavit::crypto_trust_* modules; the in-test derivation in tests/crypto_trust_kat_vectors.rs is the durable authority and byte-compares every vector on every run",
            "package_version": env!("CARGO_PKG_VERSION"),
            "python_tool_scope": "tools/verify_crypto_trust_kat.py verifies the raw-signature-primitive surface (fixtures/crypto_trust_kat.json) only. It does NOT cover these envelope/seal/rotation surfaces: its JCS port refuses numeric fields and it carries no ES256-envelope or ML-DSA verification path. Shared assumptions it independently pins: the DOMAIN_TAG value, the domain_separated pre-image layout, BLAKE3, and ES256 RFC 6979 determinism. Extending the python tool to these surfaces is future work.",
            "regenerate": "cd <repo root> && cargo test --features crypto-trust --test crypto_trust_kat_vectors (drift tripwire byte-compares every vector); export: AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_vectors export_fixture -- --exact --ignored",
            "schema": "envelope vectors bind fixed custodian/SubjectIdentity content (custodian subject affidavit-kat-subject, audience affidavit.kat) plus the full 12-field envelope wire form; seal vectors are full wire artifacts: ChainAssembler receipts over fixed events -> subject_digest_of -> envelope -> RFC 6979 ES256 signature -> SealedReceipt JCS bytes; rotation vectors are successor-signed RotationRecord JCS bytes (CLASSICAL->HYBRID)",
        },
        "schema_version": SCHEMA_VERSION,
        "surfaces": {"envelope": envelope, "rotation": rotation, "seal": seal},
    })
}

// ── fixture access ───────────────────────────────────────────────────────────

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_RELPATH)
}

fn read_committed_fixture() -> String {
    fs::read_to_string(fixture_path())
        .unwrap_or_else(|e| panic!("committed fixture {FIXTURE_RELPATH} must be readable: {e}"))
}

fn committed_fixture_value() -> Value {
    let raw = read_committed_fixture();
    serde_json::from_str(&raw).expect("committed fixture must be valid JSON")
}

/// Offset of `needle` inside `haystack`; panics with context when absent.
fn find_sub(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .position(|w| w == needle)
        .unwrap_or_else(|| {
            panic!(
                "marker {:?} must be present in the wire bytes",
                String::from_utf8_lossy(needle)
            )
        })
}

/// Flips one bit of the byte at `offset` in place. Hex-digit and letter
/// markers stay printable ASCII, so the JSON text remains parseable — the
/// flip changes the VALUE, never the transport shape.
fn flip_byte_at(bytes: &mut [u8], offset: usize) {
    bytes[offset] ^= 0x01;
}

/// Flips the last decimal digit of element `element` of the JSON number array
/// introduced by `marker`. A last-digit flip changes the value by exactly one
/// (never out of the u8 range), so the wire still parses — the tamper must be
/// refused by the SURFACE law, not by a transport parse error.
fn flip_last_digit_of_element(bytes: &mut [u8], marker: &[u8], element: usize) {
    let start = find_sub(bytes, marker);
    let after = start + marker.len();
    let close = bytes[start..]
        .iter()
        .position(|b| *b == b']')
        .expect("array must close")
        + start;
    let mut commas = 0usize;
    let mut end = close;
    for (offset, b) in bytes[after..close].iter().enumerate() {
        if *b == b',' {
            if commas == element {
                end = after + offset;
                break;
            }
            commas += 1;
        }
    }
    assert!(end > after, "array must have an element {element}");
    flip_byte_at(bytes, end - 1);
}

// ── header / provenance ──────────────────────────────────────────────────────

#[test]
fn committed_fixture_header_is_exact() {
    let value = committed_fixture_value();
    let object = value.as_object().expect("fixture must be a JSON object");
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        ["provenance", "schema_version", "surfaces"],
        "fixture header must be exactly provenance/schema_version/surfaces in JCS order"
    );
    assert_eq!(value["schema_version"].as_str(), Some(SCHEMA_VERSION));

    let provenance = value["provenance"].as_object().expect("provenance object");
    assert_eq!(
        provenance["generating_commit"].as_str(),
        Some(GENERATING_COMMIT),
        "provenance must name the generating commit"
    );
    assert_eq!(
        provenance["package_version"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "provenance package version must equal the building crate version (chain genesis binds it)"
    );
    let scope = provenance["python_tool_scope"]
        .as_str()
        .expect("python_tool_scope must be a string");
    assert!(
        scope.contains("does NOT cover"),
        "provenance must carry the honest python-tool scope note"
    );
    let cross = provenance["cross_check_python_tool"]
        .as_str()
        .expect("cross_check must be a string");
    assert!(
        cross.contains(CROSS_CHECK_SENTINEL),
        "provenance must record the seed-law cross-check outcome"
    );
    assert!(
        provenance["regenerate"]
            .as_str()
            .expect("regenerate must be a string")
            .contains(EXPORT_ENV),
        "provenance must carry the exact regeneration/export commands"
    );

    let surfaces = value["surfaces"].as_object().expect("surfaces object");
    let mut names: Vec<&str> = surfaces.keys().map(String::as_str).collect();
    names.sort_unstable();
    assert_eq!(names, ["envelope", "rotation", "seal"]);
    for (name, count) in [
        ("envelope", VECTOR_COUNT),
        ("rotation", VECTOR_COUNT),
        ("seal", VECTOR_COUNT),
    ] {
        assert_eq!(
            surfaces[name].as_array().map(Vec::len),
            Some(count),
            "surface {name} must carry {count} vectors"
        );
    }
}

#[test]
fn committed_fixture_is_jcs_canonical() {
    let raw = read_committed_fixture();
    let value: Value = serde_json::from_str(&raw).expect("committed fixture must be valid JSON");
    let canonical = jcs(&value).expect("fixture must canonicalize");
    assert_eq!(
        canonical, raw,
        "committed fixture bytes must already be canonical JCS (RFC 8785)"
    );
}

#[test]
fn committed_vector_ids_are_exact() {
    let value = committed_fixture_value();
    let ids = |surface: &str| -> Vec<String> {
        value["surfaces"][surface]
            .as_array()
            .expect("surface array")
            .iter()
            .map(|v| v["vector_id"].as_str().expect("vector_id").to_string())
            .collect()
    };
    assert_eq!(
        ids("envelope"),
        [
            "env-000-ES256",
            "env-001-ML-DSA-65",
            "env-002-ES256+ML-DSA-65"
        ]
    );
    assert_eq!(
        ids("seal"),
        ["seal-000-ES256", "seal-001-ES256", "seal-002-ES256"]
    );
    assert_eq!(
        ids("rotation"),
        [
            "rot-000-CLASSICAL-HYBRID",
            "rot-001-CLASSICAL-HYBRID",
            "rot-002-CLASSICAL-HYBRID"
        ]
    );
}

// ── envelope surface: byte-exact reproduction of the signing pre-image ──────

#[test]
fn envelope_vectors_reproduce_byte_for_byte() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["envelope"][index];
        let (envelope, family, kid) = rebuild_envelope(index);

        // Key identity binds: derived kid and public key match the vector.
        assert_eq!(
            kid.to_string(),
            vector["key_id"].as_str().expect("key_id"),
            "env {index}: derived key id must match"
        );
        assert_eq!(
            hex_encode(&family.public_key.canonical_bytes()),
            vector["public_key_hex"].as_str().expect("public_key_hex"),
            "env {index}: derived public key must match"
        );
        assert_eq!(
            hex_encode(&family.seed_bytes),
            vector["seed_hex"].as_str().expect("seed_hex"),
            "env {index}: seed material must match the role law"
        );
        assert_eq!(
            family.algorithm.as_str(),
            vector["algorithm"].as_str().expect("algorithm")
        );
        assert_eq!(
            profile_wire(family.profile),
            vector["profile"].as_str().expect("profile")
        );

        // THE vector: the exact signing pre-image and canonical bytes.
        let signing_input = envelope
            .signing_input_checked()
            .expect("canonical pre-image");
        assert_eq!(
            hex_encode(&signing_input),
            vector["signing_input_hex"]
                .as_str()
                .expect("signing_input_hex"),
            "env {index}: signing pre-image drift"
        );
        let canonical_bytes = envelope.to_bytes().expect("canonical document");
        assert_eq!(
            hex_encode(&canonical_bytes),
            vector["canonical_bytes_hex"]
                .as_str()
                .expect("canonical_bytes_hex"),
            "env {index}: canonical document drift"
        );

        // The canonical bytes parse back to the same envelope, live in-window.
        let back =
            SignatureEnvelope::from_bytes(&canonical_bytes).expect("canonical bytes must parse");
        assert_eq!(back, envelope, "env {index}: round trip must be exact");
        envelope
            .window_live(VERIFIER_NOW)
            .expect("fixed window must be live at the verifier constant");

        // The document embeds exactly the graph-declared 12-field registry.
        let doc = envelope.envelope_document();
        let obj = doc.as_object().expect("document object");
        let names: std::collections::BTreeSet<&str> = obj.keys().map(String::as_str).collect();
        let registry: std::collections::BTreeSet<&str> =
            ENVELOPE_FIELDS.iter().map(|(n, _)| *n).collect();
        assert_eq!(names, registry, "env {index}: field registry must hold");
    }
}

// ── seal surface: full wire artifacts verify VALID through the real engine ──

#[test]
fn seal_vectors_verify_valid_through_the_real_engine() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["seal"][index];
        let sealed_bytes = hex_decode(vector["sealed_json_hex"].as_str().expect("sealed_json_hex"));

        // The committed wire bytes load: the base receipt's chain law re-fires
        // inside deserialization, so this already certifies the base.
        let sealed: SealedReceipt =
            serde_json::from_slice(&sealed_bytes).expect("committed seal wire bytes must load");

        // Regenerated base must equal the committed base, byte-law complete.
        let receipt = seal_receipt_base(index);
        assert_eq!(
            sealed.base, receipt,
            "seal {index}: regenerated base must equal committed base"
        );
        assert_eq!(
            sealed.base.chain_hash.as_hex(),
            vector["chain_hash_hex"].as_str().expect("chain_hash_hex")
        );
        let address = content_address(&receipt).expect("content address");
        assert_eq!(
            address.as_hex(),
            vector["content_address_hex"]
                .as_str()
                .expect("content_address_hex")
        );

        // Envelope + signature wire fields match the vector's explicit forms.
        assert_eq!(
            hex_encode(&sealed.envelope.signing_input_checked().expect("pre-image")),
            vector["envelope_signing_input_hex"]
                .as_str()
                .expect("signing_input_hex")
        );
        assert_eq!(
            hex_encode(&sealed.signature),
            vector["signature_hex"].as_str().expect("signature_hex")
        );
        assert_eq!(
            hex_encode(&sealed.envelope.subject_digest),
            vector["subject_digest_hex"]
                .as_str()
                .expect("subject_digest_hex")
        );

        // THE adjudication: real engine over the committed bytes -> VALID.
        let (_, record) = seal_key(index);
        assert_eq!(
            record.id.to_string(),
            vector["key_id"].as_str().expect("key_id"),
            "seal {index}: derived registry key must match"
        );
        let engine = seal_engine(&record);
        let verdict = verify_sealed(&sealed, &engine).expect("verify sealed must adjudicate");
        assert_eq!(
            verdict.standing,
            CryptographicStanding::Valid,
            "seal {index}: committed wire bytes must verify VALID"
        );
        assert_eq!(verdict.subject_digest, sealed.envelope.subject_digest);
        assert_eq!(verdict.key_id, Some(record.id.clone()));
    }
}

// ── rotation surface: successor-signed record bytes verify ──────────────────

#[test]
fn rotation_vectors_verify_and_reproduce() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["rotation"][index];
        let record = rebuild_rotation(index);

        // Wire bytes reproduce byte-for-byte.
        let record_value = serde_json::to_value(&record).expect("record serializes");
        let canonical = jcs(&record_value).expect("record JCS bytes");
        assert_eq!(
            hex_encode(canonical.as_bytes()),
            vector["record_json_hex"].as_str().expect("record_json_hex"),
            "rotation {index}: record wire bytes drift"
        );

        // Successor keys re-derive; the record verifies against them.
        let es_pk = hex_decode(vector["new_public_key_es256_hex"].as_str().expect("es pk"));
        let ml_pk = hex_decode(
            vector["new_public_key_mldsa65_hex"]
                .as_str()
                .expect("ml pk"),
        );
        match verify_rotation(&record, &es_pk, &ml_pk) {
            Ok(true) => {}
            other => panic!(
                "rotation {index}: committed record must verify against the successor keys, got {other:?}"
            ),
        }
        assert_eq!(
            record.old_key_id,
            vector["old_key_id"].as_str().expect("old_key_id")
        );
        assert_eq!(
            record.new_key_id,
            vector["new_key_id"].as_str().expect("new_key_id")
        );
        assert_eq!(record.rotated_at, ROTATED_AT);
    }
}

// ── drift tripwire ───────────────────────────────────────────────────────────

#[test]
fn regeneration_reproduces_committed_fixture() {
    let regenerated = build_fixture();
    let canonical = jcs(&regenerated).expect("regenerated fixture must canonicalize");
    assert_eq!(
        canonical,
        read_committed_fixture(),
        "multi-surface vector drift: a rendered surface law changed; regenerate the fixture \
         consciously (see the provenance regenerate note) so downstream runtimes re-pin"
    );
}

// ── tamper teeth: one flipped byte per surface must refuse or diverge ───────

#[test]
fn envelope_tamper_teeth_flip_bytes_refuse_or_diverge() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["envelope"][index];
        let (envelope, _, _) = rebuild_envelope(index);
        let honest_signing_input = envelope
            .signing_input_checked()
            .expect("canonical pre-image");

        // Tooth 1: flip one byte of the version inside the canonical document
        // -> the envelope module's own law refuses by typed variant.
        let mut bytes = hex_decode(vector["canonical_bytes_hex"].as_str().expect("canonical"));
        let version_offset = find_sub(&bytes, ENVELOPE_VERSION.as_bytes());
        flip_byte_at(&mut bytes, version_offset);
        match SignatureEnvelope::from_bytes(&bytes) {
            Err(EnvelopeError::WrongVersion(v)) => {
                assert_ne!(v, ENVELOPE_VERSION, "flipped version must differ");
            }
            other => panic!("env {index}: flipped version byte must WrongVersion, got {other:?}"),
        }

        // Tooth 2: flip one byte of the key id -> parses, but the envelope no
        // longer equals the honest value and the signing pre-image diverges:
        // the vectors bind exact bytes, not near-misses.
        let mut bytes = hex_decode(vector["canonical_bytes_hex"].as_str().expect("canonical"));
        let kid_offset = find_sub(&bytes, b"afk1_");
        flip_byte_at(&mut bytes, kid_offset + "afk1_".len());
        let parsed = SignatureEnvelope::from_bytes(&bytes).expect("kid flip stays parseable");
        assert_ne!(
            parsed, envelope,
            "env {index}: flipped kid must change the envelope"
        );
        assert_ne!(
            parsed.signing_input_checked().expect("pre-image"),
            honest_signing_input,
            "env {index}: flipped kid must change the signing pre-image"
        );
    }
}

#[test]
fn seal_tamper_base_byte_refuses_to_load() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["seal"][index];
        let sealed_bytes = hex_decode(vector["sealed_json_hex"].as_str().expect("sealed_json_hex"));

        // Teeth before tamper: the honest wire form loads.
        let honest: SealedReceipt =
            serde_json::from_slice(&sealed_bytes).expect("honest seal wire bytes must load");

        // Flip one hex digit of the base's first payload commitment -> the
        // chain law refuses inside deserialization (typed Serialization map).
        let marker = honest.base.events[0]
            .payload_commitment
            .as_hex()
            .to_string();
        let mut tampered = sealed_bytes.clone();
        let offset = find_sub(&tampered, marker.as_bytes());
        flip_byte_at(&mut tampered, offset + marker.len() - 1);
        assert_ne!(tampered, sealed_bytes, "tamper must change the wire bytes");
        let outcome: Result<SealedReceipt, _> = serde_json::from_slice(&tampered);
        let err = outcome.expect_err("tampered base must refuse to load");
        let mapped: SealError = err.into();
        assert!(
            matches!(mapped, SealError::Serialization(_)),
            "tampered base must surface as the typed serialization refusal"
        );
    }
}

#[test]
fn seal_tamper_subject_byte_refuses_binding() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["seal"][index];
        let sealed_bytes = hex_decode(vector["sealed_json_hex"].as_str().expect("sealed_json_hex"));

        // Flip one digit inside the envelope's subject_digest array: the wire
        // still parses (the envelope carries no self-digest), but the seal
        // binding must refuse: the envelope now attests a different subject.
        let mut tampered = sealed_bytes.clone();
        flip_last_digit_of_element(&mut tampered, b"\"subject_digest\":[", 0);
        let rebound: SealedReceipt =
            serde_json::from_slice(&tampered).expect("subject-digit flip stays parseable");
        let (_, record) = seal_key(index);
        let engine = seal_engine(&record);
        match verify_sealed(&rebound, &engine) {
            Err(SealError::SubjectMismatch) => {}
            other => {
                panic!("seal {index}: flipped subject byte must SubjectMismatch, got {other:?}")
            }
        }
    }
}

#[test]
fn seal_tamper_signature_byte_decides_invalid() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["seal"][index];
        let sealed_bytes = hex_decode(vector["sealed_json_hex"].as_str().expect("sealed_json_hex"));
        let mut sealed: SealedReceipt =
            serde_json::from_slice(&sealed_bytes).expect("honest seal wire bytes must load");

        // In-memory copy, one signature byte flipped: a DECIDED negative
        // (Ok verdict, standing Invalid), never a refusal, never valid.
        let last = sealed.signature.len() - 1;
        sealed.signature[last] ^= 0x01;
        let (_, record) = seal_key(index);
        let engine = seal_engine(&record);
        let verdict = verify_sealed(&sealed, &engine).expect("adjudication must happen");
        assert_eq!(
            verdict.standing,
            CryptographicStanding::Invalid,
            "seal {index}: flipped signature byte must decide INVALID"
        );
    }
}

#[test]
fn rotation_tamper_signature_byte_refuses_record_tampered() {
    let value = committed_fixture_value();
    for index in 0..VECTOR_COUNT {
        let vector = &value["surfaces"]["rotation"][index];
        let record_bytes = hex_decode(vector["record_json_hex"].as_str().expect("record_json_hex"));
        let es_pk = hex_decode(vector["new_public_key_es256_hex"].as_str().expect("es pk"));
        let ml_pk = hex_decode(
            vector["new_public_key_mldsa65_hex"]
                .as_str()
                .expect("ml pk"),
        );

        // Teeth before tamper: the honest record verifies against the
        // successor keys first.
        let honest: affidavit::crypto_trust_rotation::RotationRecord =
            serde_json::from_slice(&record_bytes).expect("honest record must load");
        match verify_rotation(&honest, &es_pk, &ml_pk) {
            Ok(true) => {}
            other => {
                panic!("rotation {index}: honest record must verify before tamper, got {other:?}")
            }
        }

        // Flip one digit of a successor-signature byte inside the ML-DSA-65
        // c_tilde/z region (the wire form is `es256_der || mldsa65`; the DER
        // half is self-delimiting `0x30 <len> ...`, and only the TRAILING
        // hint region can fail structural decode — c_tilde/z bytes always
        // decode and must then FAIL VERIFICATION): identity check passes,
        // signature verify is false -> typed RecordTampered. The flip is a
        // last digit (value +/-1, never out of u8 range), so the wire still
        // parses.
        assert_eq!(
            honest.successor_signature[0], 0x30,
            "hybrid opens with the ES256 DER frame"
        );
        let mldsa_offset = 2 + usize::from(honest.successor_signature[1]);
        let mut tampered = record_bytes.clone();
        flip_last_digit_of_element(
            &mut tampered,
            b"\"successor_signature\":[",
            mldsa_offset + 100,
        );
        let tampered_record: affidavit::crypto_trust_rotation::RotationRecord =
            serde_json::from_slice(&tampered).expect("signature flip stays parseable");
        match verify_rotation(&tampered_record, &es_pk, &ml_pk) {
            Err(RotationError::RecordTampered) => {}
            other => panic!(
                "rotation {index}: flipped signature byte must RecordTampered, got {other:?}"
            ),
        }
    }
}

// ── exporter (the durable regeneration path named in provenance) ────────────

/// Rewrites `fixtures/crypto_trust_kat_vectors.json` from the in-test
/// derivation. Guarded by [`EXPORT_ENV`] so an accidental `--ignored` run
/// refuses loudly instead of silently rewriting the committed bytes.
#[test]
#[ignore]
fn export_fixture() {
    if std::env::var(EXPORT_ENV).as_deref() != Ok("1") {
        panic!(
            "export_fixture rewrites the committed multi-surface vector fixture; set \
             {EXPORT_ENV}=1 to admit the export"
        );
    }
    let canonical = jcs(&build_fixture()).expect("fixture must canonicalize");
    fs::write(fixture_path(), canonical.as_bytes())
        .unwrap_or_else(|e| panic!("must write {FIXTURE_RELPATH}: {e}"));
    println!(
        "wrote {FIXTURE_RELPATH} ({} bytes, canonical JCS)",
        canonical.len()
    );
}
