#![cfg(feature = "crypto-trust")]
//! W3-L7 property-based tests over the cryptographic trust plane (quickcheck,
//! house pattern: newtype Arbitrary wrappers with bounded generation driving
//! universal properties against the PUBLIC `affidavit::crypto_trust_*` API).
//!
//! Properties:
//! - P1: ES256 sign -> verify round-trip always holds (>= 32 cases; quickcheck
//!   runs its default 100).
//! - P2: ANY single-byte signature flip must stop the signature verifying.
//! - P3: JCS idempotence — jcs(jcs(x)) == jcs(x) for generated JSON values
//!   (bounded depth/keys, integers inside the 2^53 JCS-safe range).
//! - P4: domain_separated length law:
//!   `len == DOMAIN_TAG.len() + 1 + domain.len() + 1 + sum(8 + part.len())`.
//! - P5: nonce journal window law — a repeat strictly inside the window is
//!   refused, at or beyond the boundary it restamps and is admitted.
//! - P6: envelope to_bytes -> from_bytes round-trip (and wire determinism).

use affidavit::crypto_trust_canonical::{domain_separated, jcs, DOMAIN_TAG};
use affidavit::crypto_trust_envelope::{
    EnvelopeError, NonceJournal, SignatureEnvelope, ENVELOPE_VERSION,
};
use affidavit::crypto_trust_es256::{verify_es256, Es256SigningKey};
use affidavit::crypto_trust_keys::{AlgorithmId, CryptoProfile, KeyId};
use affidavit::crypto_trust_lifecycle::NONCE_WINDOW_SECONDS;
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;

// --- Arbitrary helpers (bounded; JCS-safe integers only) --------------------

/// The largest integer exactly representable as an IEEE-754 double.
const MAX_SAFE: i64 = 9_007_199_254_740_992;

/// A 32-byte key seed.
#[derive(Clone, Debug)]
struct Seed32([u8; 32]);

impl Arbitrary for Seed32 {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut seed = [0u8; 32];
        for byte in seed.iter_mut() {
            *byte = u8::arbitrary(g);
        }
        Seed32(seed)
    }
}

/// A 16-byte nonce.
#[derive(Clone, Debug)]
struct Nonce16([u8; 16]);

impl Arbitrary for Nonce16 {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut nonce = [0u8; 16];
        for byte in nonce.iter_mut() {
            *byte = u8::arbitrary(g);
        }
        Nonce16(nonce)
    }
}

/// A JCS-safe i64: |value| <= 2^53, so `jcs` never refuses it.
fn safe_int(g: &mut Gen) -> i64 {
    i64::arbitrary(g) % MAX_SAFE
}

/// A short string from a pool that exercises escaping (backslash, quote,
/// control) and non-ASCII (BMP + supplementary plane) characters.
fn short_string(g: &mut Gen) -> String {
    const POOL: [&str; 8] = ["a", "z", "é", "\u{1F600}", "\\", "\"", "\n", "0"];
    let len = usize::arbitrary(g) % 4;
    (0..len)
        .map(|_| POOL[usize::arbitrary(g) % POOL.len()])
        .collect()
}

/// A bounded serde_json::Value: depth <= 3, <= 3 keys/elements per container,
/// every integer inside the JCS-safe range.
#[derive(Clone, Debug)]
struct ArbJson(serde_json::Value);

fn arb_value(g: &mut Gen, depth: u32) -> serde_json::Value {
    match u8::arbitrary(g) % 6 {
        0 => serde_json::Value::Null,
        1 => serde_json::Value::Bool(bool::arbitrary(g)),
        2 => serde_json::Value::from(safe_int(g)),
        3 => serde_json::Value::from(short_string(g)),
        4 if depth > 0 => {
            let len = usize::arbitrary(g) % 4;
            serde_json::Value::Array((0..len).map(|_| arb_value(g, depth - 1)).collect())
        }
        5 if depth > 0 => {
            let len = usize::arbitrary(g) % 4;
            let mut map = serde_json::Map::new();
            for _ in 0..len {
                map.insert(short_string(g), arb_value(g, depth - 1));
            }
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::Null,
    }
}

impl Arbitrary for ArbJson {
    fn arbitrary(g: &mut Gen) -> Self {
        ArbJson(arb_value(g, 3))
    }
}

/// A well-formed envelope: the version is pinned to the admitted value, all
/// integers stay inside JCS's 2^53, and binary fields are fixed-size arrays.
#[derive(Clone, Debug)]
struct ArbEnvelope(SignatureEnvelope);

impl Arbitrary for ArbEnvelope {
    fn arbitrary(g: &mut Gen) -> Self {
        let algorithms = AlgorithmId::all();
        let algorithm = algorithms[usize::arbitrary(g) % algorithms.len()];
        let profile = match u8::arbitrary(g) % 3 {
            0 => CryptoProfile::Classical,
            1 => CryptoProfile::Hybrid,
            _ => CryptoProfile::Pqc,
        };
        let nonce = Nonce16::arbitrary(g).0;
        let mut subject_digest = [0u8; 32];
        for byte in subject_digest.iter_mut() {
            *byte = u8::arbitrary(g);
        }
        ArbEnvelope(SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm,
            key_id: KeyId(format!("afk1_prop{}", u64::arbitrary(g) % 1_000)),
            profile,
            policy_epoch: u64::arbitrary(g) % 1_000_000_000,
            revocation_epoch: u64::arbitrary(g) % 1_000_000_000,
            generation: u32::arbitrary(g) % 1_000_000,
            nonce,
            not_before: u64::arbitrary(g) % 4_000_000_000,
            expires_at: u64::arbitrary(g) % 4_000_000_000,
            subject_digest,
            audience: format!("aud-{}", u64::arbitrary(g) % 100),
        })
    }
}

// --- P1: ES256 sign -> verify round-trip -------------------------------------

#[quickcheck]
fn prop_es256_sign_verify_round_trip(seed: Seed32, msg: Vec<u8>) -> TestResult {
    let key = match Es256SigningKey::from_seed(&seed.0) {
        Ok(key) => key,
        // The zero scalar (and the negligible scalar >= n band) is not a key.
        Err(_) => return TestResult::discard(),
    };
    let signature = key.sign(&msg);
    match verify_es256(&key.public_key_sec1(), &msg, &signature) {
        Ok(true) => TestResult::passed(),
        other => TestResult::error(format!("honest ES256 signature must verify: {other:?}")),
    }
}

// --- P2: any single-byte signature flip kills verification -------------------

#[quickcheck]
fn prop_es256_any_single_byte_signature_flip_fails(
    seed: Seed32,
    msg: Vec<u8>,
    index: u64,
) -> TestResult {
    let key = match Es256SigningKey::from_seed(&seed.0) {
        Ok(key) => key,
        Err(_) => return TestResult::discard(),
    };
    let mut signature = key.sign(&msg);
    assert!(!signature.is_empty(), "DER signatures are never empty");
    let pos = (index % signature.len() as u64) as usize;
    signature[pos] ^= 0x01;
    match verify_es256(&key.public_key_sec1(), &msg, &signature) {
        // A flip either breaks the DER structure (typed refusal) or changes
        // the scalar (decided false). It must NEVER verify.
        Ok(false) | Err(_) => TestResult::passed(),
        Ok(true) => TestResult::error(format!(
            "flipped byte {pos} of {} still verified",
            signature.len()
        )),
    }
}

// --- P3: JCS idempotence over generated values -------------------------------

#[quickcheck]
fn prop_jcs_is_idempotent(value: ArbJson) -> TestResult {
    let once = match jcs(&value.0) {
        Ok(text) => text,
        Err(err) => return TestResult::error(format!("generator emits JCS-safe values: {err:?}")),
    };
    let reparsed: serde_json::Value = match serde_json::from_str(&once) {
        Ok(value) => value,
        Err(err) => return TestResult::error(format!("canonical JSON must re-parse: {err}")),
    };
    match jcs(&reparsed) {
        Ok(twice) if twice == once => TestResult::passed(),
        Ok(twice) => TestResult::error(format!("jcs(jcs(x)) != jcs(x): {once:?} vs {twice:?}")),
        Err(err) => TestResult::error(format!("canonical form must re-canonicalize: {err:?}")),
    }
}

// --- P4: domain_separated length law ------------------------------------------

#[quickcheck]
fn prop_domain_separated_length_law(domain: String, parts: Vec<Vec<u8>>) -> bool {
    let refs: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
    let buf = domain_separated(&domain, &refs);
    let expected = DOMAIN_TAG.len()
        + 1
        + domain.len()
        + 1
        + parts.iter().map(|part| 8 + part.len()).sum::<usize>();
    buf.len() == expected
}

// --- P5: nonce journal window law ---------------------------------------------

#[quickcheck]
fn prop_nonce_journal_window_law(
    kid: String,
    nonce: Nonce16,
    first_seen: u64,
    delta: u64,
) -> TestResult {
    // Bound the clock so `at` never saturates and the boundary is reachable.
    let t1 = first_seen % 1_000_000;
    let delta = delta % (2 * NONCE_WINDOW_SECONDS + 10);
    let mut journal = NonceJournal::default();
    if journal
        .record(&kid, nonce.0, t1, NONCE_WINDOW_SECONDS)
        .is_err()
    {
        return TestResult::error("first sight must always be admitted");
    }

    let inside = delta < NONCE_WINDOW_SECONDS;
    let second = journal.record(&kid, nonce.0, t1 + delta, NONCE_WINDOW_SECONDS);
    match (&second, inside) {
        (Ok(()), false) => {}
        (Err(EnvelopeError::ReplayRejected(refused)), true) if *refused == kid => {}
        (other, inside) => {
            return TestResult::error(format!(
                "delta {delta} (inside window: {inside}) must be \
                 {law}: got {other:?}",
                law = if inside { "refused" } else { "admitted" }
            ))
        }
    }

    // Inside the window the refused repeat does not restamp; at or beyond the
    // boundary the admitted repeat restamps.
    let expected_stamp = if inside { t1 } else { t1 + delta };
    if journal.seen(&kid, &nonce.0) != Some(expected_stamp) {
        return TestResult::error("journal stamp must follow the window law");
    }
    TestResult::passed()
}

// --- P6: envelope wire round-trip ---------------------------------------------

#[quickcheck]
fn prop_envelope_wire_round_trip(env: ArbEnvelope) -> TestResult {
    let envelope = env.0;
    let bytes = match envelope.to_bytes() {
        Ok(bytes) => bytes,
        Err(err) => {
            return TestResult::error(format!(
                "generator emits canonicalizable envelopes: {err:?}"
            ))
        }
    };
    // The wire form is deterministic for equal envelopes.
    match envelope.to_bytes() {
        Ok(again) if again == bytes => {}
        Ok(again) => {
            return TestResult::error(format!(
                "to_bytes must be deterministic: {bytes:?} vs {again:?}"
            ))
        }
        Err(err) => return TestResult::error(format!("second canonicalization failed: {err:?}")),
    }
    match SignatureEnvelope::from_bytes(&bytes) {
        Ok(back) if back == envelope => TestResult::passed(),
        Ok(back) => TestResult::error(format!("round-trip changed the envelope: {back:?}")),
        Err(err) => TestResult::error(format!("canonical bytes must re-parse: {err:?}")),
    }
}
