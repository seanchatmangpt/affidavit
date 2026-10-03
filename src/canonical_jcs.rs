//! RFC 8785 JSON Canonicalization Scheme (JCS) — canonical envelope bytes.
//!
//! A signature over JSON is meaningless unless every host serializes the
//! document to the *same bytes*. JCS fixes key ordering, whitespace, string
//! escaping, and IEEE 754 number formatting, so the canonical form is
//! bit-identical across platforms.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Canonicalization is a pure byte transformation. It certifies that two
//! hosts agree on the preimage of a commitment; it never decides whether the
//! underlying document is truthful.

use serde::Serialize;
use thiserror::Error;

/// Errors produced during JCS canonicalization.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JcsError {
    /// The value could not be serialized to canonical JSON.
    #[error("JCS serialization failed: {0}")]
    Serialization(String),
}

/// Serialize any serde value to RFC 8785 canonical JSON bytes.
///
/// # Errors
///
/// Returns [`JcsError::Serialization`] if the value cannot be serialized.
pub fn to_jcs<T: Serialize>(value: &T) -> Result<Vec<u8>, JcsError> {
    serde_jcs::to_string(value)
        .map(String::into_bytes)
        .map_err(|e| JcsError::Serialization(e.to_string()))
}

/// Canonicalize an already-parsed JSON value.
///
/// # Errors
///
/// Returns [`JcsError::Serialization`] if the value contains non-finite
/// numbers or is otherwise uncanonicalizable.
pub fn canonical_bytes(value: &serde_json::Value) -> Result<Vec<u8>, JcsError> {
    to_jcs(value)
}

/// BLAKE3 commitment over the canonical bytes of a JSON value.
///
/// This is the byte-exact preimage any verifier must reproduce.
///
/// # Errors
///
/// See [`canonical_bytes`].
pub fn canonical_commitment(
    value: &serde_json::Value,
) -> Result<crate::types::Blake3Hash, JcsError> {
    let bytes = canonical_bytes(value)?;
    Ok(crate::types::Blake3Hash::from_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_order_and_whitespace_do_not_change_commitment() {
        let a = json!({"b": 1, "a": 2});
        let b = json!({"a": 2, "b": 1});
        assert_eq!(canonical_bytes(&a), canonical_bytes(&b));
        assert_eq!(
            canonical_commitment(&a).expect("commitment"),
            canonical_commitment(&b).expect("commitment")
        );
    }

    #[test]
    fn canonical_form_is_byte_deterministic() {
        let v = json!({"name": "receipt", "seq": 7, "ratio": 1.5});
        let bytes = canonical_bytes(&v).expect("canonical bytes");
        let rendered = String::from_utf8(bytes).expect("utf8");
        // RFC 8785: keys sorted lexicographically, no whitespace.
        assert_eq!(rendered, r#"{"name":"receipt","ratio":1.5,"seq":7}"#);
    }

    #[test]
    fn distinct_values_distinct_commitments() {
        let a = canonical_commitment(&json!({"x": 1})).expect("a");
        let b = canonical_commitment(&json!({"x": 2})).expect("b");
        assert_ne!(a, b);
    }
}
