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

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
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
#[derive(Debug, Clone)]
pub struct WitnessKeyPair {
    signing: SigningKey,
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
    verifying
        .verify(message, &signature)
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
}
