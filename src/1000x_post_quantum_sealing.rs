//! Compatibility facade for Affidavit post-quantum cryptographic standing.
//!
//! The previous implementation in this path was explicitly mock cryptography:
//! it hashed messages with BLAKE3 while naming the result Dilithium/Kyber.
//! That forgeable implementation has been removed.
//!
//! The canonical implementation now lives in crate::crypto_trust. With the
//! pqc feature enabled, ML-DSA-65 verification is provided by the RustCrypto
//! ml-dsa crate. Private key bytes are not represented by Affidavit registry
//! types; signing capability remains behind the SigningProvider boundary.

pub use crate::crypto_trust::{
    sign_with_provider, verify_and_record, CryptoRefusal, KeyCustody, KeyRecord, KeyRegistry,
    KeyState, NonceLedger, SignatureAlgorithm, SignatureEnvelope, SigningMaterial, SigningProvider,
    VerificationContext, VerifiedSignature, SIGNATURE_PROFILE, SIGNING_DOMAIN,
};
