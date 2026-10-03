//! Secp256k1 witness verification (BIP-340 Schnorr + ECDSA).
//!
//! Verifies receipts emitted by Bitcoin/EVM-adjacent infrastructure without
//! linking against the C `libsecp256k1`. Pure Rust, constant-time arithmetic.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! This is a **verification-only** surface by design: the kernel certifies
//! foreign identities; it does not originate them. Signing belongs to the
//! upstream wallet/HSM that owns the secret key.

use k256::ecdsa::signature::Verifier;
use k256::ecdsa::{Signature as EcdsaSignature, VerifyingKey as EcdsaVerifyingKey};
use k256::schnorr::{Signature as SchnorrSignature, VerifyingKey as SchnorrVerifyingKey};
use thiserror::Error;

/// Errors produced by secp256k1 witness verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Secp256k1WitnessError {
    /// The 32-byte x-only key is not a valid BIP-340 verifying key.
    #[error("malformed BIP-340 x-only public key")]
    MalformedSchnorrKey,
    /// The 64-byte Schnorr signature is malformed or fails verification.
    #[error("BIP-340 Schnorr verification failed")]
    SchnorrVerificationFailed,
    /// The SEC1-encoded ECDSA public key is malformed.
    #[error("malformed ECDSA public key")]
    MalformedEcdsaKey,
    /// The ECDSA signature is malformed or fails verification.
    #[error("ECDSA verification failed")]
    EcdsaVerificationFailed,
}

/// Verify a 64-byte BIP-340 Schnorr signature against a 32-byte x-only
/// public key, where the 32-byte message **is** the signed preimage (raw
/// BIP-340 semantics — this is what Bitcoin/EVM test vectors exercise).
///
/// # Errors
///
/// - [`Secp256k1WitnessError::MalformedSchnorrKey`] if the key bytes are
///   invalid.
/// - [`Secp256k1WitnessError::SchnorrVerificationFailed`] if the signature is
///   malformed or does not verify.
pub fn verify_bip340_raw(
    x_only_public_key: &[u8; 32],
    message: &[u8; 32],
    signature: &[u8; 64],
) -> Result<(), Secp256k1WitnessError> {
    let x_bytes: k256::FieldBytes = (*x_only_public_key).into();
    let verifying = SchnorrVerifyingKey::from_bytes(&x_bytes)
        .map_err(|_| Secp256k1WitnessError::MalformedSchnorrKey)?;
    let signature = SchnorrSignature::from_slice(signature)
        .map_err(|_| Secp256k1WitnessError::SchnorrVerificationFailed)?;
    verifying
        .verify_raw(message, &signature)
        .map_err(|_| Secp256k1WitnessError::SchnorrVerificationFailed)
}

/// Verify a 64-byte BIP-340 Schnorr signature over a message that is
/// SHA-256 pre-hashed by the library before the BIP-340 challenge
/// (`Verifier::verify` semantics).
///
/// # Errors
///
/// See [`verify_bip340_raw`] for the typed refusal set.
pub fn verify_bip340(
    x_only_public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), Secp256k1WitnessError> {
    let x_bytes: k256::FieldBytes = (*x_only_public_key).into();
    let verifying = SchnorrVerifyingKey::from_bytes(&x_bytes)
        .map_err(|_| Secp256k1WitnessError::MalformedSchnorrKey)?;
    let signature = SchnorrSignature::from_slice(signature)
        .map_err(|_| Secp256k1WitnessError::SchnorrVerificationFailed)?;
    verifying
        .verify(message, &signature)
        .map_err(|_| Secp256k1WitnessError::SchnorrVerificationFailed)
}

/// Verify a DER or fixed-width ECDSA signature over a message (SHA-256
/// digest is applied by the library) against a 33-byte compressed SEC1 key.
///
/// # Errors
///
/// - [`Secp256k1WitnessError::MalformedEcdsaKey`] if the key encoding is
///   invalid.
/// - [`Secp256k1WitnessError::EcdsaVerificationFailed`] if the signature is
///   malformed or does not verify.
pub fn verify_ecdsa(
    compressed_sec1_public_key: &[u8; 33],
    message: &[u8],
    signature: &[u8],
) -> Result<(), Secp256k1WitnessError> {
    let verifying = EcdsaVerifyingKey::from_sec1_bytes(compressed_sec1_public_key)
        .map_err(|_| Secp256k1WitnessError::MalformedEcdsaKey)?;
    let signature = EcdsaSignature::from_slice(signature)
        .map_err(|_| Secp256k1WitnessError::EcdsaVerificationFailed)?;
    verifying
        .verify(message, &signature)
        .map_err(|_| Secp256k1WitnessError::EcdsaVerificationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    // BIP-340 test vector 0 (per k256's own conformance suite).
    const BIP340_PK: [u8; 32] = [
        0xF9, 0x30, 0x8A, 0x01, 0x92, 0x58, 0xC3, 0x10, 0x49, 0x34, 0x4F, 0x85, 0xF8, 0x9D, 0x52,
        0x29, 0xB5, 0x31, 0xC8, 0x45, 0x83, 0x6F, 0x99, 0xB0, 0x86, 0x01, 0xF1, 0x13, 0xBC, 0xE0,
        0x36, 0xF9,
    ];
    const BIP340_MSG: [u8; 32] = [0u8; 32];
    const BIP340_SIG: [u8; 64] = [
        0xE9, 0x07, 0x83, 0x1F, 0x80, 0x84, 0x8D, 0x10, 0x69, 0xA5, 0x37, 0x1B, 0x40, 0x24, 0x10,
        0x36, 0x4B, 0xDF, 0x1C, 0x5F, 0x83, 0x07, 0xB0, 0x08, 0x4C, 0x55, 0xF1, 0xCE, 0x2D, 0xCA,
        0x82, 0x15, 0x25, 0xF6, 0x6A, 0x4A, 0x85, 0xEA, 0x8B, 0x71, 0xE4, 0x82, 0xA7, 0x4F, 0x38,
        0x2D, 0x2C, 0xE5, 0xEB, 0xEE, 0xE8, 0xFD, 0xB2, 0x17, 0x2F, 0x47, 0x7D, 0xF4, 0x90, 0x0D,
        0x31, 0x05, 0x36, 0xC0,
    ];

    #[test]
    fn bip340_official_vector_0_verifies_raw() {
        assert!(verify_bip340_raw(&BIP340_PK, &BIP340_MSG, &BIP340_SIG).is_ok());
    }

    #[test]
    fn bip340_tampered_message_refused_raw() {
        let mut msg = BIP340_MSG;
        msg[0] ^= 0x01;
        assert_eq!(
            verify_bip340_raw(&BIP340_PK, &msg, &BIP340_SIG),
            Err(Secp256k1WitnessError::SchnorrVerificationFailed)
        );
    }

    #[test]
    fn bip340_tampered_signature_refused_raw() {
        let mut sig = BIP340_SIG;
        sig[0] ^= 0x01;
        assert_eq!(
            verify_bip340_raw(&BIP340_PK, &BIP340_MSG, &sig),
            Err(Secp256k1WitnessError::SchnorrVerificationFailed)
        );
    }

    #[test]
    fn malformed_schnorr_key_is_typed_refusal_raw() {
        let bad_key = [0xffu8; 32];
        assert_eq!(
            verify_bip340_raw(&bad_key, &BIP340_MSG, &BIP340_SIG),
            Err(Secp256k1WitnessError::MalformedSchnorrKey)
        );
    }
}
