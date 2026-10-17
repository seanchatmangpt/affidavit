//! Cycle-4 register row 80: strict-mode refusal vectors through
//! `verify_witness` (dalek `verify_strict`).
//!
//! Three refusal classes beyond the malleability court
//! (tests/ed25519_strict_parity.rs):
//!
//! 1. small-order / identity public key A — the identity encoding decompresses
//!    fine, so it must be refused by the strict equation path, not by key
//!    parsing;
//! 2. non-canonical R — an R encoding that is not a canonical point encoding
//!    (y >= p) or a small-order R;
//! 3. non-canonical A encoding — refused at key admission.
//!
//! Every vector must be refused with a typed outcome (never `Ok`, never a
//! panic), and the per-class variant is pinned exactly. The honest-signer
//! corpus (8 fresh keypairs) must not be refused — strict mode buys nothing
//! at the cost of collateral refusal of honest witnesses.
//!
//! Chicago: real signer over the OS RNG, assertions on real verification
//! outcomes. The identity-A vector with R = B, s = 1 satisfies the
//! verification equation sB = R + kA for every message under a weak key, so a
//! regression to dalek's lenient `verify` flips this court.

use affidavit::ed25519_witness::{verify_witness, Ed25519WitnessError, WitnessKeyPair};

/// The compressed Edwards encoding of the basepoint B (y = 4/5, sign 0),
/// byte-identical to the vector in tests/ed25519_strict_parity.rs.
const BASEPOINT_COMPRESSED: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

/// Encodings that decompress to small-order points: identity (order 1),
/// (0,1) (order 8), and (0,-1) (order 2).
fn small_order_a_keys() -> Vec<(&'static str, [u8; 32])> {
    let mut order2 = [0u8; 32];
    order2[0] = 0xec;
    for b in order2.iter_mut().skip(1).take(30) {
        *b = 0xff;
    }
    order2[31] = 0x7f;
    let mut order8 = [0u8; 32];
    order8[0] = 1;
    vec![
        ("identity O (order 1)", [0u8; 32]),
        ("(0,1) (order 8)", order8),
        ("(0,-1) (order 2)", order2),
    ]
}

#[test]
fn small_order_a_keys_are_refused_with_typed_failure() {
    let sig = {
        let mut s = [0u8; 64];
        s[..32].copy_from_slice(&BASEPOINT_COMPRESSED); // R = B
        s[32] = 1; // s = 1 (canonical)
        s
    };
    for (label, key) in small_order_a_keys() {
        for message in [&b"row-80 strict vector"[..], b"", b"receipt chain head"] {
            assert_eq!(
                verify_witness(&key, message, &sig),
                Err(Ed25519WitnessError::VerificationFailed),
                "small-order A vector {label} was accepted for message {message:?}"
            );
        }
    }
}

/// Non-canonical R encodings over a REAL signer's strong key: y >= p
/// (all-0xFF R fails decompression) and small-order R (the identity encoding).
/// Both must be refused with `VerificationFailed`, never accepted and never a
/// panic.
#[test]
fn non_canonical_r_is_refused() {
    let kp = WitnessKeyPair::generate();
    let message = b"row-80 strict vector: non-canonical R";
    let honest = kp.sign(message);
    let mut bad_r = honest;
    bad_r[..32].copy_from_slice(&[0xffu8; 32]); // y = 2^256-1 >= p: non-canonical
    let mut small_r = honest;
    small_r[..32].fill(0); // R = O: small order
    for (label, sig) in [("non-canonical R", &bad_r), ("small-order R", &small_r)] {
        assert_eq!(
            verify_witness(&kp.public(), message, sig),
            Err(Ed25519WitnessError::VerificationFailed),
            "{label} was accepted"
        );
    }
}

/// Non-canonical A encoding (y = 2^256-1 >= p, sign bit 1). Observed dalek 2
/// behavior (this court's own receipt, 2026-10-17): `VerifyingKey::from_bytes`
/// admits the encoding, so refusal happens through the strict equation path —
/// the SAME typed outcome as the small-order and non-canonical-R classes:
/// `VerificationFailed`. Not accepted, not a panic, not laundered.
#[test]
fn non_canonical_a_encoding_is_refused_strictly() {
    let kp = WitnessKeyPair::generate();
    let message = b"row-80 strict vector: non-canonical A";
    let honest = kp.sign(message);
    assert_eq!(
        verify_witness(&[0xffu8; 32], message, &honest),
        Err(Ed25519WitnessError::VerificationFailed)
    );
}

/// Uniformity sweep: every strict-refusal vector in this court refuses with a
/// typed outcome drawn from the witness error enum — never `Ok`.
#[test]
fn all_vectors_refuse_never_ok() {
    let kp = WitnessKeyPair::generate();
    let message = b"row-80 uniformity";
    let honest = kp.sign(message);
    let mut sig = honest;
    sig[32..].fill(0xff); // non-canonical s
    let results = [
        verify_witness(&[0u8; 32], message, &honest),
        verify_witness(&[0xffu8; 32], message, &honest),
        verify_witness(&kp.public(), message, &sig),
    ];
    assert!(
        results.iter().all(|r| r.is_err()),
        "a refusal vector was accepted: {results:?}"
    );
}

/// No collateral: 8 fresh honest keypairs sign and verify under strict mode.
#[test]
fn honest_signer_corpus_is_not_refused() {
    for i in 0..8 {
        let kp = WitnessKeyPair::generate();
        let message = format!("row-80 honest corpus witness {i}");
        let sig = kp.sign(message.as_bytes());
        assert_eq!(
            verify_witness(&kp.public(), message.as_bytes(), &sig),
            Ok(()),
            "honest signer {i} was refused by strict verification"
        );
    }
}

/// Deterministic RNG (xorshift64* over a fixed seed) so the Debug-hygiene
/// court knows the exact seed bytes that must not leak.
struct FixedRng(u64);

impl rand_core::RngCore for FixedRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(8) {
            let bytes = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// Register row 79 — Debug hygiene: the `Debug` rendering of a key pair must
/// contain no byte value of the signing seed and no hex rendering of it (or
/// of the derived public key). If a future refactor reintroduces
/// `#[derive(Debug)]` over the dalek `SigningKey`, this court fails.
#[test]
fn debug_rendering_never_leaks_seed_bytes() {
    let mut rng = FixedRng(0x0DDB1A5E_5EED5EED);
    for case in 0..4 {
        let kp = WitnessKeyPair::from_rng(&mut rng);
        let public = kp.public();
        let debug = format!("{kp:?}");
        assert!(
            debug.contains("redacted"),
            "Debug rendering lost its redaction marker: {debug}"
        );
        // No seed byte value may appear verbatim (as a decimal byte) — but the
        // redacted string contains no digits at all, so assert the stronger
        // structural fact: the rendering is exactly the redacted literal.
        assert_eq!(
            debug, "WitnessKeyPair(<redacted>)",
            "Debug rendering is not the redacted literal (case {case}): {debug}"
        );
        // Belt-and-braces: no 4-byte window of the public key may appear in
        // hex. (Single-byte fragments are too weak — e.g. "da" occurs inside
        // the word "redacted" itself — so the court pins 32-bit windows.)
        for window in public.windows(4) {
            let hex: String = window.iter().map(|b| format!("{b:02x}")).collect();
            assert!(
                !debug.to_lowercase().contains(&hex),
                "Debug rendering leaks public-key window {hex}: {debug}"
            );
        }
    }
}
