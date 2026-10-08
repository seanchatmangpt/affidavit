//! Compact binary envelopes for receipts and proofs (postcard).
//!
//! postcard's encoding is unambiguous at the byte level (no cross-platform
//! endian or pointer-width discrepancies), making it suitable for wire
//! transport of witnesses where canonical JSON is unnecessary overhead.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Encoding is a lossless, deterministic transformation. The envelope carries
//! witnesses; it never vouches for their content.

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

/// Errors produced by binary envelope operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EnvelopeError {
    /// The value could not be encoded.
    #[error("binary envelope encode failed: {0}")]
    Encode(String),
    /// The bytes could not be decoded into the target type.
    #[error("binary envelope decode failed: {0}")]
    Decode(String),
}

/// Encode a serde value into a compact postcard envelope.
///
/// # Errors
///
/// Returns [`EnvelopeError::Encode`] if serialization fails.
pub fn seal<T: Serialize>(value: &T) -> Result<Vec<u8>, EnvelopeError> {
    postcard::to_allocvec(value).map_err(|e| EnvelopeError::Encode(e.to_string()))
}

/// Decode a value from a postcard envelope.
///
/// # Errors
///
/// Returns [`EnvelopeError::Decode`] if the bytes are not a valid encoding
/// of `T`.
pub fn unseal<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, EnvelopeError> {
    postcard::from_bytes(bytes).map_err(|e| EnvelopeError::Decode(e.to_string()))
}

/// BLAKE3 commitment over the sealed envelope bytes.
///
/// Two hosts that produce the same commitment produced the same envelope
/// bytes, byte for byte.
///
/// # Errors
///
/// See [`seal`].
pub fn sealed_commitment<T: Serialize>(
    value: &T,
) -> Result<crate::types::Blake3Hash, EnvelopeError> {
    let bytes = seal(value)?;
    Ok(crate::types::Blake3Hash::from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Witness {
        seq: u64,
        label: String,
        commitment: Vec<u8>,
    }

    fn sample() -> Witness {
        Witness {
            seq: 42,
            label: "mmr-proof".to_string(),
            commitment: vec![1u8; 32],
        }
    }

    #[test]
    fn roundtrip_is_lossless() {
        let w = sample();
        let sealed = seal(&w).expect("seal");
        let opened: Witness = unseal(&sealed).expect("unseal");
        assert_eq!(w, opened);
    }

    #[test]
    fn commitment_is_deterministic_and_discriminating() {
        let w = sample();
        let c1 = sealed_commitment(&w).expect("commitment");
        let c2 = sealed_commitment(&w).expect("commitment");
        assert_eq!(c1, c2);

        let mut other = sample();
        other.seq = 43;
        let c3 = sealed_commitment(&other).expect("commitment");
        assert_ne!(c1, c3);
    }

    #[test]
    fn corrupt_bytes_are_refused() {
        let sealed = seal(&sample()).expect("seal");
        // Corrupt the leading byte: postcard's variable-length integer for
        // the first field becomes malformed, and decoding must refuse.
        let mut corrupted = sealed.clone();
        corrupted[0] = 0xff;
        let result: Result<Witness, _> = unseal(&corrupted);
        assert!(result.is_err(), "corrupted envelope must be refused");

        // A truncated envelope must also refuse: postcard requires full
        // consumption.
        let truncated = &sealed[..sealed.len() - 1];
        let result: Result<Witness, _> = unseal(truncated);
        assert!(result.is_err(), "truncated envelope must be refused");
    }
}
