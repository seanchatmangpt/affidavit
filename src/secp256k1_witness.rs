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

use k256::ecdsa::signature::{Signer, Verifier};
use k256::ecdsa::{
    Signature as EcdsaSignature, SigningKey as EcdsaSigningKey, VerifyingKey as EcdsaVerifyingKey,
};
use k256::elliptic_curve::scalar::IsHigh;
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
    // AG4 malleability closure (BIP-62 semantics, the same law the ES256
    // provider enforces): for one mathematical signature, (r, s) and its
    // mirror (r, n - s) are BOTH valid — a verifier that admits both lets a
    // flipped-sign re-present double through any signature-keyed surface.
    // The high-s member refuses; the signing path already emits only low-s.
    if bool::from(signature.s().is_high()) {
        return Err(Secp256k1WitnessError::EcdsaVerificationFailed);
    }
    verifying
        .verify(message, &signature)
        .map_err(|_| Secp256k1WitnessError::EcdsaVerificationFailed)
}

/// AG1 capability lane: an secp256k1 ECDSA signing key for ENVELOPE signing
/// through the certify path (RFC 6979 deterministic nonces — the same key and
/// message always yield the same DER signature, so signing stays replayable
/// byte-identically; canonical low-s emission, `s → n − s` when high-s, the
/// emitter side of the malleability closure the plane's ES256 provider
/// already enforces). This is the secp256k1 analogue of
/// `crypto_trust_es256::Es256SigningKey`; the verification-only doctrine of
/// the rest of this module is untouched — Schnorr/BIP-340 stays verify-only.
pub struct WitnessSigningKey {
    signing: EcdsaSigningKey,
}

/// Errors of the ES256K envelope signing surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WitnessSigningError {
    /// The seed does not decode to a valid non-zero secp256k1 scalar.
    #[error("seed is not a valid non-zero secp256k1 scalar")]
    InvalidSeed,
}

impl WitnessSigningKey {
    /// Derives a signing key from a 32-byte big-endian scalar seed
    /// (KEY derivation for deterministic test/fixture keys; the nonce law is
    /// RFC 6979 itself).
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self, WitnessSigningError> {
        let signing =
            EcdsaSigningKey::from_slice(seed).map_err(|_| WitnessSigningError::InvalidSeed)?;
        Ok(WitnessSigningKey { signing })
    }

    /// The public key, compressed SEC1 (`0x02/03 || X`, 33 bytes — the
    /// encoding `PublicKeyMaterial::Es256kSec1` carries and the JWKS export
    /// decompresses).
    pub fn public_key_sec1(&self) -> [u8; 33] {
        let encoded = self.signing.verifying_key().to_sec1_point(true);
        let mut out = [0u8; 33];
        out.copy_from_slice(encoded.as_bytes());
        out
    }

    /// Signs a message with RFC 6979 deterministic ECDSA (secp256k1, SHA-256)
    /// and returns the fixed-width 64-byte `r||s` encoding (the encoding
    /// [`verify_ecdsa`] parses), normalized to the canonical low-s member of
    /// its malleability class.
    pub fn sign(&self, msg: &[u8]) -> [u8; 64] {
        let signature: EcdsaSignature = self.signing.sign(msg);
        let canonical = signature.normalize_s();
        let mut out = [0u8; 64];
        out.copy_from_slice(&canonical.to_bytes());
        out
    }
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

#[cfg(test)]
mod ag1_signing_tests {
    use super::*;

    #[test]
    fn rfc6979_sign_then_verify_ecdsa_roundtrip() {
        let signing = WitnessSigningKey::from_seed(&[9u8; 32]).expect("seed");
        let msg = b"envelope pre-image bytes";
        let sig = signing.sign(msg);
        assert!(verify_ecdsa(&signing.public_key_sec1(), msg, &sig).is_ok());
        let mut tampered = sig;
        tampered[0] ^= 1;
        assert!(verify_ecdsa(&signing.public_key_sec1(), msg, &tampered).is_err());
    }

    #[test]
    fn deterministic_rfc6979_same_key_message_same_signature() {
        let a = WitnessSigningKey::from_seed(&[9u8; 32]).expect("seed");
        let b = WitnessSigningKey::from_seed(&[9u8; 32]).expect("seed");
        assert_eq!(a.sign(b"m"), b.sign(b"m"));
    }
}
