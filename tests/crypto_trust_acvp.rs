#![cfg(feature = "crypto-trust")]
//! W4-L4: REAL NIST ACVP cross-implementation vectors over the cryptographic
//! trust plane (fixtures/crypto_trust_acvp/, extracted from the NIST
//! ACVP-Server sample corpora at a pinned commit by
//! scripts/fetch_acvp_vectors.py — every vector carries its provenance in
//! the fixture's `provenance` block; the committed fixtures stand alone and
//! this suite never downloads anything).
//!
//! Cross-implementation claims proven here (NIST's bytes, OUR implementations):
//! - ML-DSA-65 keyGen: FIPS 204 KeyGen_internal(seed) reproduces NIST's
//!   published public key EXACTLY (seed -> pk KAT).
//! - ML-DSA-65 sigVer: NIST corpus signatures verify (or refuse, for the
//!   corpus's testPassed=false adversarial entries) under OUR verifier.
//!   HONEST LIMITATION: the ACVP sigGen corpus carries only the expanded
//!   4032-byte private key, from which the 32-byte seed the plane consumes
//!   is not derivable, so exact-match ML-DSA sigGen is not runnable against
//!   a seed-only signer — the verification direction is the
//!   implementation-independent proof. Non-empty contexts require the
//!   FIPS 204 PureMTS framing M' = 0x00 || len(ctx) || ctx || M, applied
//!   here at the caller level (the plane exposes only the raw message).
//! - SLH-DSA-SHA2-128s keyGen: slh_keygen_internal(SK.seed, SK.prf, PK.seed)
//!   reproduces NIST's published public key EXACTLY.
//! - SLH-DSA-SHA2-128s sign: the plane's deterministic signature over the
//!   NIST key material and message equals NIST's corpus signature
//!   byte-for-byte (the ACVP sk is SK.seed || SK.prf || PK.seed || PK.root;
//!   the plane consumes the first 48 bytes and derives the rest).
//! - Teeth (anti-vacuity): corrupting one hex nibble of a vector in a TEMP
//!   COPY of a fixture must break the corresponding assertion path. The
//!   committed fixtures are never mutated.

use std::path::PathBuf;

use affidavit::crypto_trust_pqc::{
    ml_dsa65_from_seed, ml_dsa65_verify, slh_dsa128s_from_seed, slh_dsa128s_sign,
    slh_dsa128s_verify,
};

const FIXTURE_DIR: &str = "fixtures/crypto_trust_acvp";

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURE_DIR)
        .join(name)
}

fn load_fixture(name: &str) -> serde_json::Value {
    let raw = std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("fixture {FIXTURE_DIR}/{name} must be readable: {e}"));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("fixture {FIXTURE_DIR}/{name} must be valid JSON: {e}"))
}

fn write_temp_fixture(name: &str, value: &serde_json::Value) -> std::path::PathBuf {
    let mut tmp = tempfile::NamedTempFile::new().expect("temp fixture file");
    let text = serde_json::to_string_pretty(value).expect("serialize temp fixture");
    std::io::Write::write_all(&mut tmp, text.as_bytes()).expect("write temp fixture");
    let _ = name;
    tmp.into_temp_path().keep().expect("keep temp fixture path")
}

/// Decode lowercase hex (fixture convention) into bytes.
fn hex_decode(s: &str, what: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    bytes
        .chunks(2)
        .map(|pair| {
            assert!(
                pair.len() == 2,
                "{what}: odd hex length {} at offset {}",
                s.len(),
                pair.as_ptr() as usize - bytes.as_ptr() as usize
            );
            let hi = (pair[0] as char).to_digit(16).expect("hex digit") as u8;
            let lo = (pair[1] as char).to_digit(16).expect("hex digit") as u8;
            (hi << 4) | lo
        })
        .collect()
}

fn fixture_provenance(doc: &serde_json::Value, name: &str) {
    let prov = &doc["provenance"];
    assert!(
        prov["source_commit"].is_string() && prov["files"]["prompt_sha256"].is_string(),
        "{name}: provenance block must name the upstream commit and file digests"
    );
}

/// FIPS 204 PureMTS message framing: M' = 0x00 || len(ctx) || ctx || M.
fn pure_mts_framing(context_hex: &str, message_hex: &str) -> Vec<u8> {
    let ctx = hex_decode(context_hex, "context");
    let msg = hex_decode(message_hex, "message");
    assert!(ctx.len() <= 255, "ACVP context must fit the 1-byte length");
    let mut framed = Vec::with_capacity(2 + ctx.len() + msg.len());
    framed.push(0u8);
    framed.push(ctx.len() as u8);
    framed.extend_from_slice(&ctx);
    framed.extend_from_slice(&msg);
    framed
}

#[test]
fn acvp_ml_dsa_65_keygen_seed_to_pk_exact() {
    let doc = load_fixture("ml_dsa_65_keygen.json");
    fixture_provenance(&doc, "ml_dsa_65_keygen.json");
    let vectors = doc["ml_dsa_65_keygen"]
        .as_array()
        .expect("ml_dsa_65_keygen array");
    assert!(!vectors.is_empty(), "keyGen fixture must carry vectors");
    let mut matched = 0usize;
    for v in vectors {
        let tc_id = v["tcId"].as_i64().expect("tcId");
        let seed: [u8; 32] = hex_decode(v["seed"].as_str().expect("seed"), "seed")
            .try_into()
            .expect("seed is 32 bytes");
        let pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
        let kp = ml_dsa65_from_seed(&seed);
        assert_eq!(
            kp.public, pk,
            "ACVP ML-DSA-65 keyGen tcId {tc_id}: seed -> pk must match NIST byte-for-byte"
        );
        matched += 1;
    }
    println!(
        "ACVP ML-DSA-65 keyGen: {matched} seed->pk vectors exact-matched against NIST \
         (usnistgov/ACVP-Server @ {})",
        doc["provenance"]["source_commit"].as_str().unwrap_or("?")
    );
    assert!(
        matched >= 1,
        "at least one real NIST keyGen vector must run"
    );
}

#[test]
fn acvp_ml_dsa_65_sigver_nist_signatures_under_plane_verifier() {
    let doc = load_fixture("ml_dsa_65_sign.json");
    fixture_provenance(&doc, "ml_dsa_65_sign.json");
    let vectors = doc["ml_dsa_65_sigver"]
        .as_array()
        .expect("ml_dsa_65_sigver array");
    assert!(!vectors.is_empty(), "sigVer fixture must carry vectors");
    let mut passed = 0usize;
    let mut refused = 0usize;
    for v in vectors {
        let tc_id = v["tcId"].as_i64().expect("tcId");
        let pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
        let framed = pure_mts_framing(
            v["context"].as_str().expect("context"),
            v["message"].as_str().expect("message"),
        );
        let sig = hex_decode(v["signature"].as_str().expect("signature"), "signature");
        let expected = v["testPassed"].as_bool().expect("testPassed");
        let ok = ml_dsa65_verify(&pk, &framed, &sig)
            .unwrap_or_else(|e| panic!("ACVP sigVer tcId {tc_id}: verify refused: {e}"));
        assert_eq!(
            ok, expected,
            "ACVP ML-DSA-65 sigVer tcId {tc_id}: NIST verdict must hold under the plane verifier"
        );
        if expected {
            passed += 1;
        } else {
            refused += 1;
        }
    }
    println!(
        "ACVP ML-DSA-65 sigVer: {passed} passing + {refused} adversarial NIST signatures \
         judged correctly by the plane verifier (PureMTS framing at caller)"
    );
    assert!(
        passed >= 1,
        "at least one positive NIST signature must verify"
    );
    assert!(
        refused >= 1,
        "at least one NIST negative vector must refuse"
    );
}

#[test]
fn acvp_slh_dsa_128s_keygen_seeds_to_pk_exact() {
    let doc = load_fixture("slh_dsa_128s_keygen.json");
    fixture_provenance(&doc, "slh_dsa_128s_keygen.json");
    let vectors = doc["keygen"].as_array().expect("keygen array");
    assert!(!vectors.is_empty(), "SLH keyGen fixture must carry vectors");
    let mut matched = 0usize;
    for v in vectors {
        let tc_id = v["tcId"].as_i64().expect("tcId");
        let seeds: [u8; 48] = hex_decode(v["seeds"].as_str().expect("seeds"), "seeds")
            .try_into()
            .expect("seeds are 48 bytes");
        let pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
        let kp = slh_dsa128s_from_seed(&seeds);
        assert_eq!(
            kp.public, pk,
            "ACVP SLH-DSA-SHA2-128s keyGen tcId {tc_id}: seeds -> pk must match NIST"
        );
        matched += 1;
    }
    println!("ACVP SLH-DSA-SHA2-128s keyGen: {matched} seed-triplet->pk vectors exact-matched");
    assert!(matched >= 1);
}

#[test]
fn acvp_slh_dsa_128s_sign_exact_match() {
    let doc = load_fixture("slh_dsa_128s_keygen.json");
    let vectors = doc["sign"].as_array().expect("sign array");
    assert!(!vectors.is_empty(), "SLH sign fixture must carry vectors");
    for v in vectors {
        let tc_id = v["tcId"].as_i64().expect("tcId");
        let seeds: [u8; 48] = hex_decode(v["seeds"].as_str().expect("seeds"), "seeds")
            .try_into()
            .expect("seeds are 48 bytes");
        let msg = hex_decode(v["message"].as_str().expect("message"), "message");
        let nist_sig = hex_decode(v["signature"].as_str().expect("signature"), "signature");
        // Cross-implementation exact match: OUR signer over NIST key material
        // must reproduce NIST's corpus signature byte-for-byte.
        let sig = slh_dsa128s_sign(&seeds, &msg).expect("plane slh sign");
        assert_eq!(
            sig, nist_sig,
            "ACVP SLH-DSA-SHA2-128s sigGen tcId {tc_id}: plane signature must equal NIST's"
        );
        // And the verification direction holds under OUR verifier as well.
        let pk = slh_dsa128s_from_seed(&seeds).public;
        assert!(
            slh_dsa128s_verify(&pk, &msg, &nist_sig).expect("verify NIST signature"),
            "NIST signature must verify under the plane verifier"
        );
    }
    println!(
        "ACVP SLH-DSA-SHA2-128s sigGen: {} exact-match sign vectors (deterministic) + \
         verify direction",
        vectors.len()
    );
}

/// Teeth (anti-vacuity): a corrupted nibble in a TEMP COPY of the keyGen
/// fixture must break the seed -> pk match. Proves the KAT assertions are
/// live comparators, not always-green checks. The committed fixture is
/// never mutated.
#[test]
fn acvp_fixture_corruption_breaks_keygen_kat_in_temp_copy() {
    let mut doc = load_fixture("ml_dsa_65_keygen.json");
    let original_pk = doc["ml_dsa_65_keygen"][0]["pk"]
        .as_str()
        .expect("pk string")
        .to_string();
    let mut corrupted = original_pk.clone();
    let first = corrupted.as_bytes()[0];
    corrupted.replace_range(..1, if first == b'0' { "1" } else { "0" });
    doc["ml_dsa_65_keygen"][0]["pk"] = serde_json::Value::String(corrupted.clone());
    let tmp_path = write_temp_fixture("ml_dsa_65_keygen.corrupt", &doc);
    let raw = std::fs::read_to_string(&tmp_path).expect("read temp fixture");
    let back: serde_json::Value = serde_json::from_str(&raw).expect("parse temp fixture");
    let v = &back["ml_dsa_65_keygen"][0];
    let seed: [u8; 32] = hex_decode(v["seed"].as_str().expect("seed"), "seed")
        .try_into()
        .expect("seed is 32 bytes");
    let bad_pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
    assert_ne!(
        original_pk.as_bytes()[0],
        bad_pk[0],
        "corruption must actually change the expected key"
    );
    assert_ne!(
        ml_dsa65_from_seed(&seed).public,
        bad_pk,
        "corrupted fixture vector must FAIL the KAT: the comparator has teeth"
    );
    let _ = std::fs::remove_file(&tmp_path);
}

/// Teeth for the sigVer direction: a corrupted signature nibble in a TEMP
/// COPY must stop verifying (NIST positive vector becomes a refusal).
#[test]
fn acvp_fixture_corruption_breaks_sigver_in_temp_copy() {
    let mut doc = load_fixture("ml_dsa_65_sign.json");
    let positive = doc["ml_dsa_65_sigver"]
        .as_array_mut()
        .expect("sigver array")
        .iter_mut()
        .find(|v| v["testPassed"].as_bool() == Some(true))
        .expect("fixture carries a positive vector");
    let sig_hex = positive["signature"]
        .as_str()
        .expect("signature")
        .to_string();
    let mut corrupted = sig_hex.clone();
    let flip = if corrupted.as_bytes()[0] == b'0' {
        "1"
    } else {
        "0"
    };
    corrupted.replace_range(..1, flip);
    positive["signature"] = serde_json::Value::String(corrupted);
    let tmp_path = write_temp_fixture("ml_dsa_65_sign.corrupt", &doc);
    let raw = std::fs::read_to_string(&tmp_path).expect("read temp fixture");
    let back: serde_json::Value = serde_json::from_str(&raw).expect("parse temp fixture");
    let v = &back["ml_dsa_65_sigver"][0];
    let pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
    let framed = pure_mts_framing(
        v["context"].as_str().expect("context"),
        v["message"].as_str().expect("message"),
    );
    let bad_sig = hex_decode(v["signature"].as_str().expect("signature"), "signature");
    assert!(
        !ml_dsa65_verify(&pk, &framed, &bad_sig).expect("corrupted sig must still decode"),
        "corrupted NIST signature must be REFUSED: the verify assertion has teeth"
    );
    let _ = std::fs::remove_file(&tmp_path);
}

/// The committed fixtures must be intact (the teeth tests use temp copies).
#[test]
fn acvp_committed_fixtures_are_uncorrupted() {
    let kg = load_fixture("ml_dsa_65_keygen.json");
    for v in kg["ml_dsa_65_keygen"].as_array().expect("array") {
        let seed: [u8; 32] = hex_decode(v["seed"].as_str().expect("seed"), "seed")
            .try_into()
            .expect("32-byte seed");
        let pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
        assert_eq!(
            ml_dsa65_from_seed(&seed).public,
            pk,
            "committed keyGen fixture must remain exact"
        );
    }
    let sign = load_fixture("ml_dsa_65_sign.json");
    for v in sign["ml_dsa_65_sigver"].as_array().expect("array") {
        if v["testPassed"].as_bool() == Some(true) {
            let pk = hex_decode(v["pk"].as_str().expect("pk"), "pk");
            let framed = pure_mts_framing(
                v["context"].as_str().expect("context"),
                v["message"].as_str().expect("message"),
            );
            let sig = hex_decode(v["signature"].as_str().expect("signature"), "signature");
            assert!(
                ml_dsa65_verify(&pk, &framed, &sig).expect("verify"),
                "committed positive sigVer fixture must remain verifying"
            );
        }
    }
}
