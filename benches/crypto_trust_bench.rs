//! Benchmarks for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Real cryptography only — no mocks: keys derived via `from_seed`, real
//! sign/verify per admitted algorithm, JCS over a ~1 KiB object, and the
//! envelope signing pre-image. Run with:
//! `cargo bench --features crypto-trust` (the bench target declares
//! `required-features = ["crypto-trust"]` in the consumer Cargo.toml, because
//! benching the plane requires the plane's feature).
//!
//! Deterministic inputs throughout: benches measure the operation, never
//! input generation, so every key, message, and object is a fixed literal.

// Consumed query columns (bench.rq): domain_tag, envelope_version, canon_name, digest_alg, algs.

use affidavit::crypto_trust_canonical::jcs;
use affidavit::crypto_trust_envelope::SignatureEnvelope;
use affidavit::crypto_trust_es256::{verify_es256, Es256SigningKey};
use affidavit::crypto_trust_keys::{AlgorithmId, CryptoProfile, KeyId};
use affidavit::crypto_trust_pqc::{
    hybrid_sign, hybrid_verify, ml_dsa65_from_seed, ml_dsa65_sign, ml_dsa65_verify,
    slh_dsa128s_from_seed, slh_dsa128s_sign, slh_dsa128s_verify, HybridSecret,
    SLH_DSA_128S_SEED_LEN,
};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

/// Envelope version under bench (`ctp:policy-v1` `ctp:envelopeVersion`),
/// rendered from the graph and used as the benched envelope's version field.
const BENCH_ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";

/// Trust-plane domain tag (`ctp:policy-v1` `ctp:domainTag`), rendered from the
/// graph; held equal to `crypto_trust_canonical::DOMAIN_TAG` at bench time.
const BENCH_DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Canonicalization identity (`ctp:canon-JCS` `ctp:canonicalizationName`),
/// rendered from the graph; held equal to
/// `crypto_trust_canonical::CANONICALIZATION` at bench time.
const BENCH_CANONICALIZATION: &str = "JCS-RFC8785";

/// Digest algorithm over domain-separated bytes (`ctp:canon-JCS`
/// `ctp:digestAlgorithm`), rendered from the graph; held equal to
/// `crypto_trust_canonical::DIGEST_ALGORITHM` at bench time.
const BENCH_DIGEST_ALGORITHM: &str = "BLAKE3";

/// Algorithm registry rendered from the graph (bench.rq): `name|rustVariant`
/// per `@@` group, ordered by `ctp:algorithmName` — the set of algorithms the
/// benches must cover.
const BENCH_ALGORITHM_REGISTRY: &str = "ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s";

/// The bench's graph-fact teeth, checked at the start of EVERY bench: the
/// rendered bindings must equal the plane's own rendered consts — same graph,
/// one projection per fact. A drift refuses the bench run loudly instead of
/// measuring against drifted constants.
fn graph_facts_hold() {
    assert_eq!(
        BENCH_ENVELOPE_VERSION,
        affidavit::crypto_trust_envelope::ENVELOPE_VERSION
    );
    assert_eq!(
        BENCH_DOMAIN_TAG,
        affidavit::crypto_trust_canonical::DOMAIN_TAG
    );
    assert_eq!(
        BENCH_CANONICALIZATION,
        affidavit::crypto_trust_canonical::CANONICALIZATION
    );
    assert_eq!(
        BENCH_DIGEST_ALGORITHM,
        affidavit::crypto_trust_canonical::DIGEST_ALGORITHM
    );
    // Every algorithm the benches exercise must be admitted by the graph, and
    // every rendered registry entry must carry a name.
    let registry_names: Vec<&str> = BENCH_ALGORITHM_REGISTRY
        .split("@@")
        .map(|g| g.split_once('|').map(|(n, _)| n).unwrap_or(g))
        .collect();
    assert!(
        registry_names.iter().all(|n| !n.is_empty()),
        "registry must render non-empty algorithm names"
    );
    for benched in ["ES256", "ES256+ML-DSA-65", "ML-DSA-65", "SLH-DSA-SHA2-128s"] {
        assert!(
            registry_names.contains(&benched),
            "benched algorithm {benched} must be admitted by the graph registry"
        );
    }
}

/// The fixed bench message (every signing/verifying bench signs these bytes).
const BENCH_MESSAGE: &[u8] = b"affidavit crypto-trust-plane benchmark message";

/// A deterministic ~1 KiB JSON object: 16 fields x 64-character values,
/// exactly 1250 bytes in canonical (JCS) form.
fn bench_object_1kb() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for i in 0..16u64 {
        map.insert(
            format!("field_{i:02}"),
            serde_json::Value::String("0123456789abcdef".repeat(4)),
        );
    }
    serde_json::Value::Object(map)
}

/// The benched envelope: the same shape the envelope module's own tests pin,
/// under the graph-rendered version.
fn bench_envelope() -> SignatureEnvelope {
    SignatureEnvelope {
        version: BENCH_ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::HybridEs256MlDsa65,
        key_id: KeyId("k-bench-1".to_string()),
        profile: CryptoProfile::Hybrid,
        policy_epoch: 7,
        revocation_epoch: 3,
        generation: 2,
        nonce: [0x10; 16],
        not_before: 1_000,
        expires_at: 2_000,
        subject_digest: [0x22; 32],
        audience: "affidavit.verifier".to_string(),
    }
}

fn bench_es256_sign(c: &mut Criterion) {
    graph_facts_hold();
    let key = Es256SigningKey::from_seed(&[0x42u8; 32]).expect("valid p256 scalar");
    c.bench_function("ctp_es256_sign", |b| {
        b.iter(|| key.sign(black_box(BENCH_MESSAGE)))
    });
}

fn bench_es256_verify(c: &mut Criterion) {
    graph_facts_hold();
    let key = Es256SigningKey::from_seed(&[0x42u8; 32]).expect("valid p256 scalar");
    let pk = key.public_key_sec1();
    let sig = key.sign(BENCH_MESSAGE);
    c.bench_function("ctp_es256_verify", |b| {
        b.iter(|| {
            verify_es256(black_box(&pk), black_box(BENCH_MESSAGE), black_box(&sig)).expect("verify")
        })
    });
}

fn bench_ml_dsa65_sign(c: &mut Criterion) {
    graph_facts_hold();
    let seed = [0xA5u8; 32];
    let rnd = [0x5Au8; 32];
    c.bench_function("ctp_ml_dsa65_sign", |b| {
        b.iter(|| {
            ml_dsa65_sign(black_box(&seed), black_box(BENCH_MESSAGE), black_box(&rnd))
                .expect("sign")
        })
    });
}

fn bench_ml_dsa65_verify(c: &mut Criterion) {
    graph_facts_hold();
    let seed = [0xA5u8; 32];
    let rnd = [0x5Au8; 32];
    let kp = ml_dsa65_from_seed(&seed);
    let sig = ml_dsa65_sign(&seed, BENCH_MESSAGE, &rnd).expect("sign");
    c.bench_function("ctp_ml_dsa65_verify", |b| {
        b.iter(|| {
            ml_dsa65_verify(
                black_box(&kp.public),
                black_box(BENCH_MESSAGE),
                black_box(&sig),
            )
            .expect("verify")
        })
    });
}

fn bench_slh_dsa128s_sign(c: &mut Criterion) {
    graph_facts_hold();
    let seeds = [0x11u8; SLH_DSA_128S_SEED_LEN];
    c.bench_function("ctp_slh_dsa128s_sign", |b| {
        b.iter(|| slh_dsa128s_sign(black_box(&seeds), black_box(BENCH_MESSAGE)).expect("sign"))
    });
}

fn bench_slh_dsa128s_verify(c: &mut Criterion) {
    graph_facts_hold();
    let seeds = [0x11u8; SLH_DSA_128S_SEED_LEN];
    let kp = slh_dsa128s_from_seed(&seeds);
    let sig = slh_dsa128s_sign(&seeds, BENCH_MESSAGE).expect("sign");
    c.bench_function("ctp_slh_dsa128s_verify", |b| {
        b.iter(|| {
            slh_dsa128s_verify(
                black_box(&kp.public),
                black_box(BENCH_MESSAGE),
                black_box(&sig),
            )
            .expect("verify")
        })
    });
}

fn bench_hybrid_sign(c: &mut Criterion) {
    graph_facts_hold();
    let secret = hybrid_secret();
    c.bench_function("ctp_hybrid_sign", |b| {
        b.iter(|| hybrid_sign(black_box(&secret), black_box(BENCH_MESSAGE)).expect("sign"))
    });
}

fn bench_hybrid_verify(c: &mut Criterion) {
    graph_facts_hold();
    let secret = hybrid_secret();
    let es_pk = secret.es256.public_key_sec1();
    let ml_pk = ml_dsa65_from_seed(&secret.mldsa65_seed).public;
    let sig = hybrid_sign(&secret, BENCH_MESSAGE).expect("sign");
    c.bench_function("ctp_hybrid_verify", |b| {
        b.iter(|| {
            hybrid_verify(
                black_box(&es_pk),
                black_box(&ml_pk),
                black_box(BENCH_MESSAGE),
                black_box(&sig),
            )
            .expect("verify")
        })
    });
}

fn bench_jcs_1kb(c: &mut Criterion) {
    graph_facts_hold();
    let value = bench_object_1kb();
    c.bench_function("ctp_jcs_1kb_object", |b| {
        b.iter(|| jcs(black_box(&value)).expect("canonicalizes"))
    });
}

fn bench_envelope_signing_input(c: &mut Criterion) {
    graph_facts_hold();
    let envelope = bench_envelope();
    c.bench_function("ctp_envelope_signing_input", |b| {
        b.iter(|| envelope.signing_input())
    });
}

/// Deterministic hybrid secret for the hybrid benches.
fn hybrid_secret() -> HybridSecret {
    HybridSecret {
        es256: Es256SigningKey::from_seed(&[0x42u8; 32]).expect("valid p256 scalar"),
        mldsa65_seed: [0xA5u8; 32],
    }
}

criterion_group!(
    benches,
    bench_es256_sign,
    bench_es256_verify,
    bench_ml_dsa65_sign,
    bench_ml_dsa65_verify,
    bench_slh_dsa128s_sign,
    bench_slh_dsa128s_verify,
    bench_hybrid_sign,
    bench_hybrid_verify,
    bench_jcs_1kb,
    bench_envelope_signing_input,
);
criterion_main!(benches);
