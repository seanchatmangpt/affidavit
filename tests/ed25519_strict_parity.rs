//! Differential court: host Ed25519 witness verification vs the wasm ABI.
//!
//! Defect under prosecution: `verify_witness` (src/ed25519_witness.rs) used
//! dalek's lenient `verify`, while the wasm ABI
//! (affidavit-wasm/src/signature.rs, `ed25519()`) uses `verify_strict`. The
//! two runtimes disagreed on the exact windows where dalek 2's lenient and
//! strict verification diverge (small-order R / weak public keys). A
//! malleated acceptance on host and a refusal on wasm for the same bytes is
//! a one-witness-per-claim violation across runtimes.
//!
//! ## Dalek 2.2 documented behavior (read from the vendored source, not recalled)
//!
//! - Non-canonical `s` (s >= curve order) is refused by BOTH `verify` and
//!   `verify_strict` (`InternalSignature::from_bytes` -> `check_scalar`
//!   rejects it before any equation is evaluated). An s-flip is therefore
//!   NOT a lenient/strict differential on dalek 2.2; the court does not
//!   pretend it is one.
//! - The strict-only windows are: small-order R, weak (small-order) public
//!   key A, and non-canonically encoded A. `verify` admits them; strict
//!   refuses.
//!
//! ## The differential witness (the mutation the court must flip on)
//!
//! With the weak identity key A = O ([0u8; 32], which decompresses and is
//! small-order), the pair (R = B, s = 1) satisfies the verification equation
//! sB = R + kA for EVERY message, because kA = O. Lenient `verify` accepts
//! this for any message; `verify_strict` refuses (weak key). If the host
//! fix is reverted to `verify`, `weak_key_malleation_is_refused` fails:
//! a court that cannot flip is FAIL.
//!
//! Chicago: real signer (`WitnessKeyPair` over OsRng), no mocks, assertions
//! on real verification outcomes.

use affidavit::ed25519_witness::{verify_witness, Ed25519WitnessError, WitnessKeyPair};

/// The compressed Edwards encoding of the curve basepoint B:
/// y = 4/5, sign 0. (s = 1 => sB = B, a valid point equation.)
const BASEPOINT_COMPRESSED: [u8; 32] = [
    0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
    0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
];

/// Positive control: a real signer's honest signature verifies.
#[test]
fn honest_sig_is_accepted() {
    let kp = WitnessKeyPair::generate();
    let message = b"strict-parity court: honest witness";
    let sig = kp.sign(message);
    assert_eq!(verify_witness(&kp.public(), message, &sig), Ok(()));
}

/// Corrupted-sig refusal: every single-byte mutation of an honest signature
/// (sampled over R and s halves) must be refused.
#[test]
fn corrupted_sig_corpus_is_refused() {
    let kp = WitnessKeyPair::generate();
    let message = b"strict-parity court: corpus";
    let honest = kp.sign(message);
    let mut cases: Vec<[u8; 64]> = Vec::new();
    for idx in [0usize, 15, 31, 32, 40, 63] {
        for bit in [0u8, 1, 7] {
            let mut m = honest;
            m[idx] ^= 1 << bit;
            cases.push(m);
        }
    }
    // Degenerate encodings: zero R, zero s, all-0xFF s (non-canonical s).
    let mut zero_r = honest;
    zero_r[..32].fill(0);
    cases.push(zero_r);
    let mut zero_s = honest;
    zero_s[32..].fill(0);
    cases.push(zero_s);
    let mut ff_s = honest;
    ff_s[32..].fill(0xff);
    cases.push(ff_s);
    for sig in &cases {
        assert_eq!(
            verify_witness(&kp.public(), message, sig),
            Err(Ed25519WitnessError::VerificationFailed),
            "mutation of honest sig was accepted: {sig:02x?}"
        );
    }
}

/// The differential witness. Under lenient `verify` this pair verifies for
/// ANY message over the weak identity key; `verify_strict` refuses. This is
/// the test that flips if the host path regresses to lenient verification.
#[test]
fn weak_key_malleation_is_refused() {
    // Identity point: compress(O) = [0u8; 32]; decompresses, small-order.
    let weak_key = [0u8; 32];
    let mut sig = [0u8; 64];
    sig[..32].copy_from_slice(&BASEPOINT_COMPRESSED); // R = B
    sig[32] = 1; // s = 1 (canonical); sB = B = R, so R = sB - kO for any k.
    for message in [&b"any message"[..], b"", b"receipt chain head"] {
        assert_eq!(
            verify_witness(&weak_key, message, &sig),
            Err(Ed25519WitnessError::VerificationFailed),
            "weak-key malleation was accepted for message {message:?}"
        );
    }
}

/// Strong keys must not be collateral damage: an honest signer's key still
/// verifies, and the strict path never refuses an honest signature.
#[test]
fn strict_does_not_refuse_honest_signers() {
    for _ in 0..8 {
        let kp = WitnessKeyPair::generate();
        let message = b"strict-parity court: no collateral refusal";
        let sig = kp.sign(message);
        assert_eq!(verify_witness(&kp.public(), message, &sig), Ok(()));
    }
}
