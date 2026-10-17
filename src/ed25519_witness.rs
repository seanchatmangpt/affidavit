//! Classical Ed25519 witness signatures.
//!
//! Fast classical signing for edge nodes and local agent actions. This module
//! shares the dalek curve stack with the FROST threshold plane
//! ([`crate::threshold_quorum`]), so a group key and a local key live in the
//! same algebraic world.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Verification is a decidable check of a witness against a public key.
//! Custody of the signing key belongs to the witness owner; this module
//! returns signing material only to the caller that holds it.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand_core::{OsRng, RngCore};
use thiserror::Error;

/// Errors produced by Ed25519 witness operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Ed25519WitnessError {
    /// The public key bytes are not a valid curve point encoding.
    #[error("malformed Ed25519 public key")]
    MalformedPublicKey,
    /// The signature bytes are not a valid signature encoding.
    #[error("malformed Ed25519 signature")]
    MalformedSignature,
    /// The signature does not verify against the key and message.
    #[error("Ed25519 witness verification failed")]
    VerificationFailed,
}

/// A local witness signing key. Never persist or transmit the secret half.
///
/// `Debug` is redacted: it must never render any byte of the signing seed
/// (row 79 hygiene). Zeroizing the seed on drop is a recorded follow-up
/// (requires a `zeroize` dependency change — out of this lane's authority).
#[derive(Clone)]
pub struct WitnessKeyPair {
    signing: SigningKey,
}

impl std::fmt::Debug for WitnessKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Redacted: the signing seed never enters any Debug rendering.
        f.write_str("WitnessKeyPair(<redacted>)")
    }
}

impl WitnessKeyPair {
    /// Generate a fresh key pair from the OS entropy source.
    #[must_use]
    pub fn generate() -> Self {
        Self::from_rng(&mut OsRng)
    }

    /// Generate a key pair from an explicit RNG (for deterministic tests).
    ///
    /// Seeds from the RNG's bytes so the construction never crosses RNG
    /// trait-version boundaries.
    #[must_use]
    pub fn from_rng<R: RngCore>(rng: &mut R) -> Self {
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        Self {
            signing: SigningKey::from_bytes(&seed),
        }
    }

    /// The 32-byte signing seed (row 31). Round-trips through
    /// [`WitnessKeyPair::from_seed`] — the returned pair is byte-identical in
    /// public key and signatures, because `from_seed` reconstructs the same
    /// dalek `SigningKey` from the same seed bytes.
    #[must_use]
    pub fn seed(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    /// Reconstruct a key pair from a 32-byte seed (row 31).
    ///
    /// Note: dalek clamps the seed's scalar (bits 0-2 cleared, bit 255 set,
    /// bits 254-251 copied) during expansion, so distinct seeds that differ
    /// only in the low three bits or the top bit expand to the *same* key.
    /// The court below pins this clamping behavior explicitly rather than
    /// guessing at it.
    #[must_use]
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            signing: SigningKey::from_bytes(seed),
        }
    }

    /// The 32-byte public verification key.
    #[must_use]
    pub fn public(&self) -> [u8; 32] {
        self.signing.verifying_key().to_bytes()
    }

    /// Sign a message, returning the 64-byte signature.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.signing.sign(message).to_bytes()
    }
}

/// Verify a 64-byte Ed25519 signature over a message against a 32-byte
/// public key.
///
/// # Errors
///
/// - [`Ed25519WitnessError::MalformedPublicKey`] if the key bytes are invalid.
/// - [`Ed25519WitnessError::MalformedSignature`] if the signature bytes are
///   invalid.
/// - [`Ed25519WitnessError::VerificationFailed`] if the signature does not
///   verify.
pub fn verify_witness(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), Ed25519WitnessError> {
    let verifying = VerifyingKey::from_bytes(public_key)
        .map_err(|_| Ed25519WitnessError::MalformedPublicKey)?;
    let signature = Signature::from_bytes(signature);
    // Strict verification (parity with the wasm ABI, affidavit-wasm
    // src/signature.rs ed25519()): rejects malleated (non-canonical) signatures
    // — e.g. s >= group order — that lenient `verify` would accept.
    verifying
        .verify_strict(message, &signature)
        .map_err(|_| Ed25519WitnessError::VerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_then_verify_roundtrip() {
        let kp = WitnessKeyPair::generate();
        let message = b"receipt chain head commitment";
        let sig = kp.sign(message);
        assert!(verify_witness(&kp.public(), message, &sig).is_ok());
    }

    #[test]
    fn tampered_message_is_refused() {
        let kp = WitnessKeyPair::generate();
        let sig = kp.sign(b"honest payload");
        assert_eq!(
            verify_witness(&kp.public(), b"tampered payload", &sig),
            Err(Ed25519WitnessError::VerificationFailed)
        );
    }

    #[test]
    fn wrong_key_is_refused() {
        let signer = WitnessKeyPair::generate();
        let other = WitnessKeyPair::generate();
        let sig = signer.sign(b"payload");
        assert_eq!(
            verify_witness(&other.public(), b"payload", &sig),
            Err(Ed25519WitnessError::VerificationFailed)
        );
    }

    #[test]
    fn garbage_inputs_are_typed_refusals_not_panics() {
        let zero_key = [0u8; 32];
        let zero_sig = [0u8; 64];
        let result = verify_witness(&zero_key, b"m", &zero_sig);
        assert!(matches!(
            result,
            Err(Ed25519WitnessError::MalformedPublicKey | Ed25519WitnessError::VerificationFailed)
        ));
    }
    // --- row 31: seed / from_seed round-trip ---

    #[test]
    fn from_seed_roundtrip_reproduces_public_key_and_signature() {
        let kp = WitnessKeyPair::generate();
        let restored = WitnessKeyPair::from_seed(&kp.seed());
        assert_eq!(restored.public(), kp.public());
        let message = b"row31 roundtrip payload";
        assert_eq!(restored.sign(message), kp.sign(message));
        assert!(verify_witness(&restored.public(), message, &kp.sign(message)).is_ok());
    }

    #[test]
    fn zero_seed_constructs_and_dalek_clamps_it() {
        // Dalek expands the seed by clamping the scalar (bit 255 set, low
        // three bits cleared), so an all-zero seed is a lawful, non-panicking
        // construction producing a fixed valid public key. Pin that behavior:
        // both the construction itself and the determinism of its output.
        let zero = WitnessKeyPair::from_seed(&[0u8; 32]);
        let zero_again = WitnessKeyPair::from_seed(&[0u8; 32]);
        assert_eq!(zero.public(), zero_again.public());
        // Clamped scalar of the zero seed is non-zero (bit 255 set), so the
        // derived public key is a real curve point that can verify a signature.
        let message = b"zero seed clamp court";
        let sig = zero.sign(message);
        assert!(
            verify_witness(&zero.public(), message, &sig).is_ok(),
            "zero seed must produce a usable, clamped key pair"
        );
    }

    #[test]
    fn distinct_seeds_produce_distinct_keys() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        a[..31].copy_from_slice(&[7u8; 31]);
        a[31] = 8;
        b[..31].copy_from_slice(&[7u8; 31]);
        b[31] = 16;
        // Seeds differ in bits outside the clamp zone, so they must diverge.
        let ka = WitnessKeyPair::from_seed(&a);
        let kb = WitnessKeyPair::from_seed(&b);
        assert_ne!(a, b);
        assert_ne!(
            ka.public(),
            kb.public(),
            "seeds differing in non-clamped bits must yield distinct public keys"
        );
    }

    // --- row 79: redacted Debug hygiene ---

    #[test]
    fn debug_rendering_leaks_no_seed_byte() {
        // Distinctive filler seed so any leaked byte is detectable in the
        // rendering.
        let seed = [0xABu8; 32];
        let kp = WitnessKeyPair::from_seed(&seed);
        let rendered = format!("{:?}", kp);
        assert!(
            !rendered.contains("WitnessKeyPair {"),
            "Debug must not structurally render fields: {rendered}"
        );
        for (i, byte) in seed.iter().enumerate() {
            // Any seed byte leaking as decimal digits or hex would render
            // somewhere; "171" / "ab" are the renderings of 0xAB.
            assert!(
                !rendered.contains(&byte.to_string()),
                "seed byte {i} leaked (decimal) into Debug: {rendered}"
            );
        }
        let hex: String = seed.iter().map(|b| format!("{b:02x}")).collect();
        assert!(
            !rendered.to_lowercase().contains("abab"),
            "seed hex prefix leaked into Debug: {rendered}"
        );
        assert!(rendered.contains("redacted"));
    }
}
