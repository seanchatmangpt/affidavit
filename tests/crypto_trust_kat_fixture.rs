#![cfg(feature = "crypto-trust")]

//! Published cross-runtime KAT fixture for the cryptographic trust plane
//! (wave 4, lane W4-L8).
//!
//! `src/crypto_trust_kat.rs` (ggen-rendered) is the single source of truth.
//! This suite proves that the committed fixture
//! `fixtures/crypto_trust_kat.json` is exactly `export_json(generate_corpus())`
//! wrapped in a canonical JCS header object, that the committed corpus
//! verifies through the Rust verifier, and that export/import is a byte fixed
//! point. The drift tripwire (`regeneration_reproduces_committed_fixture`)
//! refuses any silent change to the derivation chain: a corpus algorithm
//! change must consciously update the fixture so every downstream runtime
//! re-pins the new bytes.
//!
//! Regeneration (the exact command carried in the fixture's
//! `generated_from` note):
//!
//! ```text
//! cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 \
//!   cargo test --features crypto-trust --test crypto_trust_kat_fixture \
//!   export_fixture -- --exact --ignored
//! ```
//!
//! The independent (non-Rust) counterpart is
//! `tools/verify_crypto_trust_kat.py`, which re-derives the whole seed and
//! message chain in pure Python (BLAKE3 + the domain-separated layout) and
//! verifies the ES256 signatures and public-key derivations with the system
//! openssl.
//!
//! # Why the derivation chain is re-derived here
//!
//! `src/crypto_trust_kat.rs` gates its seed-derivation helpers
//! (`generate_corpus`, `build_vector`, `kat_seed32`, ...) behind
//! `#[cfg(test)]` — test key material is deliberately not shipped in the
//! public API. Integration test targets compile the library *without*
//! `cfg(test)`, so this suite cannot call `generate_corpus` directly.
//! Instead it carries a local, independent re-derivation of the exact chain
//! (same domain-separated digest layout, same provider calls), cited per
//! item to its source in `src/crypto_trust_kat.rs`. This is the same role
//! the Python verifier plays: a second derivation identity whose disagreement
//! with the committed fixture fires the
//! `regeneration_reproduces_committed_fixture` drift tripwire.

use std::fs;
use std::path::PathBuf;

use affidavit::crypto_trust_canonical::{digest, jcs, DOMAIN_TAG, ENVELOPE_VERSION};
use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_kat::{
    export_json, import_json, verify_corpus, KatVector, KAT_ALGORITHM_REGISTRY, KAT_DOMAIN_TAG,
    KAT_ENVELOPE_VERSION, KAT_SCHEMA_VERSION,
};
use affidavit::crypto_trust_pqc::{
    hybrid_sign, ml_dsa65_from_seed, ml_dsa65_sign, slh_dsa128s_from_seed, slh_dsa128s_sign,
    HybridSecret, SLH_DSA_128S_SEED_LEN,
};
use affidavit::secp256k1_witness::WitnessSigningKey;
use ed25519_dalek::{Signer, SigningKey as Ed25519SigningKey};
use serde_json::{json, Value};

const FIXTURE_RELPATH: &str = "fixtures/crypto_trust_kat.json";
const EXPORT_ENV: &str = "AFFIDAVIT_KAT_EXPORT_FIXTURE";

// ── Independent re-derivation of the KAT corpus ─────────────────────────────
//
// Mirrors the `#[cfg(test)]` derivation chain in `src/crypto_trust_kat.rs`
// exactly (each item cites its source). Deliberate second derivation
// identity: if this chain and the rendered chain ever disagree, the
// `regeneration_reproduces_committed_fixture` tripwire refuses the fixture.

/// Lowercase-hex encoding (source: `hex_encode` in `src/crypto_trust_kat.rs`).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// Domain-separated key seed (source: `kat_seed32`).
fn kat_seed32(index: usize, role: &str) -> [u8; 32] {
    digest(
        KAT_DOMAIN_TAG,
        &[
            b"ctp.kat.seed" as &[u8],
            role.as_bytes(),
            &(index as u64).to_be_bytes(),
        ],
    )
}

/// Explicit ML-DSA-65 signing randomizer (source: `kat_mldsa_rnd`).
fn kat_mldsa_rnd(index: usize) -> [u8; 32] {
    digest(
        KAT_DOMAIN_TAG,
        &[
            b"ctp.kat.rnd.mldsa65" as &[u8],
            &(index as u64).to_be_bytes(),
        ],
    )
}

/// SLH-DSA seed triplet `sk_seed || sk_prf || pk_seed` (source:
/// `kat_slh_seeds`).
fn kat_slh_seeds(index: usize) -> [u8; SLH_DSA_128S_SEED_LEN] {
    let mut seeds = [0u8; SLH_DSA_128S_SEED_LEN];
    for (counter, chunk) in seeds.chunks_exact_mut(16).enumerate() {
        let d = digest(
            KAT_DOMAIN_TAG,
            &[
                b"ctp.kat.seed.slh" as &[u8],
                &(index as u64).to_be_bytes(),
                &(counter as u64).to_be_bytes(),
            ],
        );
        chunk.copy_from_slice(&d[..16]);
    }
    seeds
}

/// Deterministic signed message (source: `kat_message`).
fn kat_message(index: usize) -> Vec<u8> {
    digest(
        KAT_DOMAIN_TAG,
        &[b"ctp.kat.message" as &[u8], &(index as u64).to_be_bytes()],
    )
    .to_vec()
}

/// The registry as `(algorithm name, _variant)` pairs in graph order
/// (source: `registry_entries`).
fn registry_entries() -> impl Iterator<Item = (&'static str, &'static str)> {
    KAT_ALGORITHM_REGISTRY
        .split("@@")
        .map(|group| group.split_once('|').unwrap_or((group, "")))
}

/// Builds the single vector for `algorithm` at registry position `index`
/// (source: `build_vector`; provider calls are the same public provider
/// functions the rendered chain uses).
fn build_vector(index: usize, algorithm: &str) -> Result<KatVector, String> {
    let vector_id = format!("kat-{index:03}-{algorithm}");
    let message = kat_message(index);
    let message_hex = hex_encode(&message);
    let vector = match algorithm {
        "ED25519" => {
            let seed = kat_seed32(index, "ed25519");
            let key = Ed25519SigningKey::from_bytes(&seed);
            let signature = key.sign(&message).to_bytes();
            KatVector {
                vector_id: vector_id.clone(),
                algorithm: algorithm.to_string(),
                seed_hex: hex_encode(&seed),
                public_key_hex: hex_encode(&key.verifying_key().to_bytes()),
                message_hex,
                signature_hex: hex_encode(&signature),
            }
        }
        "ES256" => {
            let seed = kat_seed32(index, "es256");
            let key = Es256SigningKey::from_seed(&seed)
                .map_err(|e| format!("{vector_id}: es256 seed refused: {e}"))?;
            let signature = key.sign(&message);
            KatVector {
                vector_id: vector_id.clone(),
                algorithm: algorithm.to_string(),
                seed_hex: hex_encode(&seed),
                public_key_hex: hex_encode(&key.public_key_sec1()),
                message_hex,
                signature_hex: hex_encode(&signature),
            }
        }
        "ML-DSA-65" => {
            let seed = kat_seed32(index, "mldsa65");
            let rnd = kat_mldsa_rnd(index);
            let keypair = ml_dsa65_from_seed(&seed).map_err(|e| format!("{vector_id}: {e}"))?;
            let signature = ml_dsa65_sign(&seed, &message, &rnd)
                .map_err(|e| format!("{vector_id}: ml-dsa sign refused: {e}"))?;
            KatVector {
                vector_id: vector_id.clone(),
                algorithm: algorithm.to_string(),
                seed_hex: hex_encode(&seed),
                public_key_hex: hex_encode(&keypair.public),
                message_hex,
                signature_hex: hex_encode(&signature),
            }
        }
        "SLH-DSA-SHA2-128s" => {
            let seeds = kat_slh_seeds(index);
            let keypair = slh_dsa128s_from_seed(&seeds).map_err(|e| format!("{vector_id}: {e}"))?;
            let signature = slh_dsa128s_sign(&seeds, &message)
                .map_err(|e| format!("{vector_id}: slh-dsa sign refused: {e}"))?;
            KatVector {
                vector_id: vector_id.clone(),
                algorithm: algorithm.to_string(),
                seed_hex: hex_encode(&seeds),
                public_key_hex: hex_encode(&keypair.public),
                message_hex,
                signature_hex: hex_encode(&signature),
            }
        }
        "ES256+ML-DSA-65" => {
            let es_seed = kat_seed32(index, "hybrid-es256");
            let ml_seed = kat_seed32(index, "hybrid-mldsa65");
            let secret = HybridSecret {
                es256: Es256SigningKey::from_seed(&es_seed)
                    .map_err(|e| format!("{vector_id}: hybrid es256 seed refused: {e}"))?,
                mldsa65_seed: ml_seed,
            };
            let signature = hybrid_sign(&secret, &message)
                .map_err(|e| format!("{vector_id}: hybrid sign refused: {e}"))?;
            let mut seed_bytes = es_seed.to_vec();
            seed_bytes.extend_from_slice(&ml_seed);
            let mut public = secret.es256.public_key_sec1();
            let ml_kp = ml_dsa65_from_seed(&ml_seed).map_err(|e| format!("{vector_id}: {e}"))?;
            public.extend_from_slice(&ml_kp.public);
            let mut signature_bytes = signature.es256_der.clone();
            signature_bytes.extend_from_slice(&signature.mldsa65);
            KatVector {
                vector_id: vector_id.clone(),
                algorithm: algorithm.to_string(),
                seed_hex: hex_encode(&seed_bytes),
                public_key_hex: hex_encode(&public),
                message_hex,
                signature_hex: hex_encode(&signature_bytes),
            }
        }
        "ES256K" => {
            let seed = kat_seed32(index, "es256k");
            let key = WitnessSigningKey::from_seed(&seed)
                .map_err(|e| format!("{vector_id}: es256k seed refused: {e}"))?;
            let signature = key.sign(&message);
            KatVector {
                vector_id: vector_id.clone(),
                algorithm: algorithm.to_string(),
                seed_hex: hex_encode(&seed),
                public_key_hex: hex_encode(&key.public_key_sec1()),
                message_hex,
                signature_hex: hex_encode(&signature),
            }
        }
        other => return Err(format!("unknown KAT algorithm: {other}")),
    };
    Ok(vector)
}

/// Regenerates the full corpus, one vector per rendered registry entry in
/// graph order (source: `generate_corpus`).
fn regenerate_corpus() -> Vec<KatVector> {
    let mut vectors = Vec::new();
    for (index, (algorithm, _variant)) in registry_entries().enumerate() {
        if let Ok(vector) = build_vector(index, algorithm) {
            vectors.push(vector);
        }
    }
    vectors
}

/// The exact header keys, in JCS (lexicographic) order. First keys of the
/// published file: schema version + generated-from note beside the corpus.
const EXPECTED_HEADER_KEYS: [&str; 3] = ["corpus", "generated_from", "schema_version"];

/// The generated-from note embedded in the fixture: provenance plus the exact
/// regeneration command.
const GENERATED_FROM: &str = "affidavit::crypto_trust_kat::generate_corpus() -> export_json (JCS, RFC 8785); deterministic under KAT_DOMAIN_TAG=affidavit.crypto-trust-plane.v1; regenerate: cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_fixture export_fixture -- --exact --ignored";

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_RELPATH)
}

fn read_committed_fixture() -> String {
    fs::read_to_string(fixture_path())
        .unwrap_or_else(|e| panic!("committed fixture {FIXTURE_RELPATH} must be readable: {e}"))
}

/// The canonical JCS envelope object published as the fixture: the corpus
/// bytes plus the schema-version and generated-from header.
fn canonical_envelope(corpus_json: &str) -> String {
    let corpus: Value =
        serde_json::from_str(corpus_json).expect("export_json output must be valid JSON");
    let envelope = json!({
        "corpus": corpus,
        "generated_from": GENERATED_FROM,
        "schema_version": KAT_SCHEMA_VERSION,
    });
    jcs(&envelope).expect("string-only envelope must canonicalize")
}

/// The committed corpus as canonical JCS bytes (exactly the embedded
/// `export_json` output).
fn committed_corpus_json() -> String {
    let raw = read_committed_fixture();
    let value: Value = serde_json::from_str(&raw).expect("committed fixture must be valid JSON");
    let corpus = value
        .get("corpus")
        .cloned()
        .expect("committed fixture must carry a corpus array");
    jcs(&corpus).expect("corpus array must canonicalize")
}

/// Exporter: regenerates `fixtures/crypto_trust_kat.json` from the rendered
/// KAT module. Guarded by [`EXPORT_ENV`] so an accidental `--ignored` run
/// refuses loudly instead of silently rewriting the committed bytes.
#[test]
#[ignore]
fn export_fixture() {
    if std::env::var(EXPORT_ENV).as_deref() != Ok("1") {
        panic!("export_fixture rewrites the committed KAT fixture; set {EXPORT_ENV}=1 to admit the export");
    }
    let canonical = canonical_envelope(&export_json(&regenerate_corpus()));
    fs::write(fixture_path(), &canonical)
        .unwrap_or_else(|e| panic!("must write {FIXTURE_RELPATH}: {e}"));
    println!(
        "wrote {FIXTURE_RELPATH} ({} bytes, canonical JCS)",
        canonical.len()
    );
}

#[test]
fn committed_fixture_header_is_exact() {
    let raw = read_committed_fixture();
    let value: Value = serde_json::from_str(&raw).expect("committed fixture must be valid JSON");
    let object = value.as_object().expect("fixture must be a JSON object");
    let keys: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(
        keys, EXPECTED_HEADER_KEYS,
        "fixture header must be exactly corpus/generated_from/schema_version in JCS order"
    );
    assert!(
        matches!(value["corpus"], Value::Array(_)),
        "corpus header field must be an array"
    );
    let note = value["generated_from"]
        .as_str()
        .expect("generated_from must be a string");
    assert!(
        note.contains("generate_corpus"),
        "generated_from must name the generating function"
    );
    assert!(
        note.contains(EXPORT_ENV),
        "generated_from must carry the exact regeneration command"
    );
    assert_eq!(
        value["schema_version"].as_str(),
        Some(KAT_SCHEMA_VERSION),
        "schema_version header must equal the rendered KAT schema version"
    );
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
fn fixture_header_agrees_with_rendered_facts() {
    assert_eq!(KAT_SCHEMA_VERSION, "CTP-KAT-v1");
    assert_eq!(
        KAT_DOMAIN_TAG, DOMAIN_TAG,
        "KAT domain tag must be the plane domain tag"
    );
    assert_eq!(
        KAT_ENVELOPE_VERSION, ENVELOPE_VERSION,
        "KAT envelope version must be the plane envelope version"
    );
}

#[test]
fn committed_corpus_imports_and_verifies() {
    let corpus_json = committed_corpus_json();
    let vectors = import_json(&corpus_json).expect("committed corpus must import");
    let expected = KAT_ALGORITHM_REGISTRY.split("@@").count();
    assert_eq!(
        vectors.len(),
        expected,
        "committed corpus must carry one vector per registry entry"
    );
    let report = verify_corpus(&vectors).expect("committed corpus must verify");
    assert_eq!(report.total, expected);
    assert_eq!(report.passed, report.total, "strict law: passed == total");
    assert!(report.total > 0);
}

#[test]
fn committed_corpus_is_byte_fixed_point() {
    let corpus_json = committed_corpus_json();
    let back = import_json(&corpus_json).expect("committed corpus must import");
    assert_eq!(
        export_json(&back),
        corpus_json,
        "export_json(import_json(fixture corpus)) must be byte-identical to the committed corpus"
    );
}

/// The drift tripwire: in-test regeneration must reproduce the committed
/// fixture byte-for-byte. A corpus algorithm change (or any rendered fact
/// feeding the derivation chain) breaks this loudly, forcing a conscious
/// fixture regeneration that every downstream runtime re-pins.
#[test]
fn regeneration_reproduces_committed_fixture() {
    let regenerated = export_json(&regenerate_corpus());
    assert_eq!(
        regenerated,
        committed_corpus_json(),
        "KAT corpus drift: the rendered derivation chain changed; regenerate the fixture \
         consciously (see the fixture's generated_from note) so downstream runtimes re-pin"
    );
    assert_eq!(
        canonical_envelope(&regenerated),
        read_committed_fixture(),
        "full envelope drift: header or corpus changed; regenerate the fixture consciously"
    );
}
