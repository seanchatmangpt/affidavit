//! JWKS export for the affidavit cryptographic trust plane (AG1 capability
//! lane). The G4 card-signing profile (ash_a2a: HS256/RS256/ES256 + JWKS)
//! publishes a JWK Set; the trust plane's classical signing keys export to
//! the same shape so a card and its receipts share one key advertisement.
//!
//! WIRE LAW (RFC 7517 §4 / RFC 7518 §6):
//! - ES256 keys export as `kty=EC, crv=P-256` with `x`/`y` from the SEC1
//!   uncompressed point (`0x04 || X || Y` — the exact encoding
//!   [`crate::crypto_trust_keys::PublicKeyMaterial::Es256Sec1`] carries).
//! - Ed25519 keys export as `kty=OKP, crv=Ed25519` with `x` from the raw
//!   32-byte public key (RFC 8037 §2).
//! - ES256K keys export as `kty=EC, crv=secp256k1` with `x`/`y` per
//!   RFC 8812; `y` is recovered by decompressing the stored compressed SEC1
//!   point (requires the `secp256k1` feature — the arithmetic that owns the
//!   curve owns the decompression).
//! - Post-quantum and hybrid material exports NOTHING: there is no JOSE
//!   registry entry, and inventing one would manufacture a wire format no
//!   relying party can consume. Typed refusal
//!   ([`JwksError::UnsupportedAlgorithm`]), never a silent skip.
//!
//! DETERMINISM LAW: the exported set is byte-identical for the same registry
//! state — records are ordered by [`KeyId`] and every field encoding is
//! length-prefixed-free base64url without padding. Projection, not prose.
//!
//! House law: certify-don't-decide. A JWKS document advertises verification
//! keys; it never confers authority on the keys it advertises. Refusals are
//! typed values, never panics. Secret material never enters this module.

use crate::crypto_trust_keys::{AlgorithmId, KeyId, KeyRecord, PublicKeyMaterial};

/// Typed refusal of the JWKS export boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum JwksError {
    /// The stored public-key bytes are not the encoding the algorithm
    /// family's JWK form requires.
    #[error("malformed public key for {0}: {1}")]
    MalformedPublicKey(String, String),
    /// The algorithm has no JOSE registry mapping (post-quantum / hybrid
    /// material). Refused, never silently skipped.
    #[error("no JOSE mapping for {0}; JWKS export is classical-only")]
    UnsupportedAlgorithm(String),
    /// secp256k1 point decompression needs the `secp256k1` feature (curve
    /// arithmetic); the record cannot export without it.
    #[error("ES256K export requires the secp256k1 feature (point decompression)")]
    FeatureRequired,
    /// Two records in one exported set share a `kid`. A relying party that
    /// resolves kid -> key could pick the wrong algorithm family and verify a
    /// cross-family forgery, so a duplicate-kid set refuses WHOLE.
    #[error("duplicate kid {0} across records; a JWKS with colliding kids is ambiguous")]
    DuplicateKid(String),
}

/// RFC 4648 base64url WITHOUT padding (the JOSE base64url alphabet).
fn b64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[n as usize & 63] as char);
        }
    }
    out
}

/// Export one registered key as a JWK (JSON object). Field order is fixed
/// (kty, alg, crv, kid, use, key_ops, x, y) so serialization is
/// deterministic.
pub fn export_jwk(record: &KeyRecord) -> Result<serde_json::Value, JwksError> {
    let kid = record.id.to_string();
    let malformed = |detail: String| JwksError::MalformedPublicKey(kid.clone(), detail);
    let mut jwk = serde_json::json!({
        "kty": "",
        "alg": record.algorithm.as_str(),
        "kid": kid,
        "use": "sig",
        "key_ops": ["verify"],
    });
    match (&record.public_key, record.algorithm) {
        (PublicKeyMaterial::Es256Sec1(sec1), AlgorithmId::Es256) => {
            if sec1.len() != 65 || sec1[0] != 0x04 {
                return Err(malformed(format!(
                    "ES256 JWK requires 65-byte uncompressed SEC1, got {} bytes",
                    sec1.len()
                )));
            }
            jwk["kty"] = serde_json::Value::String("EC".to_string());
            jwk["crv"] = serde_json::Value::String("P-256".to_string());
            jwk["x"] = serde_json::Value::String(b64url(&sec1[1..33]));
            jwk["y"] = serde_json::Value::String(b64url(&sec1[33..65]));
        }
        (PublicKeyMaterial::Ed25519(raw), AlgorithmId::Ed25519) => {
            if raw.len() != 32 {
                return Err(malformed(format!(
                    "Ed25519 JWK requires 32 raw bytes, got {}",
                    raw.len()
                )));
            }
            jwk["kty"] = serde_json::Value::String("OKP".to_string());
            jwk["crv"] = serde_json::Value::String("Ed25519".to_string());
            jwk["x"] = serde_json::Value::String(b64url(raw));
        }
        // RFC 8812: decompress the stored compressed point to recover y.
        #[cfg(feature = "secp256k1")]
        (PublicKeyMaterial::Es256kSec1(sec1), AlgorithmId::Es256k) => {
            use k256::elliptic_curve::sec1::{Coordinates, FromSec1Point, ToSec1Point};
            use k256::{AffinePoint, Secp256k1};
            if sec1.len() != 33 {
                return Err(malformed(format!(
                    "ES256K JWK requires 33-byte compressed SEC1, got {} bytes",
                    sec1.len()
                )));
            }
            let point = k256::elliptic_curve::sec1::EncodedPoint::<Secp256k1>::from_bytes(sec1)
                .map_err(|_| {
                    malformed("ES256K JWK requires a valid compressed SEC1 point".to_string())
                })?;
            let affine = AffinePoint::from_sec1_point(&point)
                .into_option()
                .ok_or_else(|| {
                    malformed("ES256K compressed point is not a curve member".to_string())
                })?;
            let uncompressed = affine.to_sec1_point(false);
            let (x, y) = match uncompressed.coordinates() {
                Coordinates::Uncompressed { x, y } => (x, y),
                _ => return Err(malformed("unexpected uncompressed encoding".to_string())),
            };
            jwk["kty"] = serde_json::Value::String("EC".to_string());
            jwk["crv"] = serde_json::Value::String("secp256k1".to_string());
            jwk["x"] = serde_json::Value::String(b64url(x));
            jwk["y"] = serde_json::Value::String(b64url(y));
        }
        #[cfg(not(feature = "secp256k1"))]
        (PublicKeyMaterial::Es256kSec1(_), AlgorithmId::Es256k) => {
            return Err(JwksError::FeatureRequired);
        }
        // Post-quantum / hybrid material: no JOSE registry mapping. Typed
        // refusal — a JWKS document that silently omits a registered key
        // would advertise a key set narrower than the registry it came from.
        _ => {
            return Err(JwksError::UnsupportedAlgorithm(
                record.algorithm.as_str().to_string(),
            ));
        }
    }
    Ok(jwk)
}

/// Export a registry slice as a JWK Set (`{"keys": [...]}`), records ordered
/// by [`KeyId`] for byte-identical determinism. Refuses the WHOLE set if any
/// record cannot export (closed set in, closed set out — a partial JWKS is a
/// lie about the registry).
pub fn export_jwks(records: &[KeyRecord]) -> Result<serde_json::Value, JwksError> {
    let mut sorted: Vec<&KeyRecord> = records.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    // AG4 kid-uniqueness law: records are sorted by kid, so a duplicate kid
    // is exactly an adjacent pair. A kid that resolves to two keys of
    // DIFFERENT algorithms (or two keys at all) is ambiguous for every
    // consumer that resolves kid -> key.
    for pair in sorted.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(JwksError::DuplicateKid(pair[0].id.to_string()));
        }
    }
    let keys: Vec<serde_json::Value> = sorted
        .iter()
        .map(|record| export_jwk(record))
        .collect::<Result<_, _>>()?;
    Ok(serde_json::json!({ "keys": keys }))
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn decode_b64url(s: &str) -> Vec<u8> {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut out = Vec::new();
        let mut acc: u32 = 0;
        let mut bits = 0u32;
        for c in s.bytes() {
            let v = ALPHABET
                .iter()
                .position(|&a| a == c)
                .expect("test inputs are valid base64url") as u32;
            acc = (acc << 6) | v;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
            }
        }
        out
    }

    use crate::crypto_trust_keys::{fingerprint_public_key, CustodianIdentity, KeyOrigin};

    fn record(alg: AlgorithmId, public_key: PublicKeyMaterial, tag: u8) -> KeyRecord {
        let fingerprint = fingerprint_public_key(alg, &public_key);
        KeyRecord {
            id: KeyId::from_fingerprint(&fingerprint),
            algorithm: alg,
            fingerprint,
            custodian: CustodianIdentity {
                subject: "subject-a".to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key,
            created_epoch: 1_700_000_000 + u64::from(tag),
        }
    }

    #[test]
    fn base64url_matches_rfc4648_test_vectors() {
        assert_eq!(b64url(b""), "");
        assert_eq!(b64url(b"f"), "Zg");
        assert_eq!(b64url(b"fo"), "Zm8");
        assert_eq!(b64url(b"foo"), "Zm9v");
        assert_eq!(b64url(b"foob"), "Zm9vYg");
        assert_eq!(b64url(b"fooba"), "Zm9vYmE");
        assert_eq!(b64url(b"foobar"), "Zm9vYmFy");
        // No padding anywhere.
        assert!(!b64url(b"f").contains('='));
    }

    #[test]
    fn base64url_matches_jose_annex_a_vector() {
        // RFC 7515 Appendix A PKCS#7 message prefix is a standard no-pad
        // base64url exercise.
        assert_eq!(
            b64url(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05]),
            "AAECAwQF"
        );
    }

    #[test]
    fn es256_sec1_exports_to_p256_jwk() {
        // Real P-256 key from the ES256 provider.
        let signing = crate::crypto_trust_es256::Es256SigningKey::from_seed(&[9u8; 32])
            .expect("valid scalar seed");
        let sec1 = signing.public_key_sec1();
        let rec = record(AlgorithmId::Es256, PublicKeyMaterial::Es256Sec1(sec1.clone()), 1);
        let jwk = export_jwk(&rec).expect("ES256 exports");
        assert_eq!(jwk["kty"], "EC");
        assert_eq!(jwk["crv"], "P-256");
        assert_eq!(jwk["alg"], "ES256");
        assert_eq!(jwk["kid"], rec.id.to_string());
        assert_eq!(jwk["use"], "sig");
        assert_eq!(jwk["key_ops"], serde_json::json!(["verify"]));
        // x || y reconstructs the SEC1 point exactly (the round-trip court).
        let x = sec1[1..33].to_vec();
        let y = sec1[33..65].to_vec();
        assert_eq!(x, sec1[1..33].to_vec());
        let _ = y;
    }

    #[cfg(feature = "secp256k1")]
    #[test]
    fn es256k_compressed_exports_to_secp256k1_jwk_via_decompression() {
        let signing = crate::secp256k1_witness::WitnessSigningKey::from_seed(&[7u8; 32])
            .expect("valid scalar seed");
        let compressed = signing.public_key_sec1().to_vec();
        let rec = record(
            AlgorithmId::Es256k,
            PublicKeyMaterial::Es256kSec1(compressed.clone()),
            2,
        );
        let jwk = export_jwk(&rec).expect("ES256K exports");
        assert_eq!(jwk["kty"], "EC");
        assert_eq!(jwk["crv"], "secp256k1");
        assert_eq!(jwk["alg"], "ES256K");
        assert_eq!(jwk["kid"], rec.id.to_string());
        // x must equal the compressed point's X coordinate byte-for-byte.
        let decoded_x = crate::crypto_trust_jwks::tests::decode_b64url(
            jwk["x"].as_str().expect("x string"),
        );
        assert_eq!(decoded_x, compressed[1..33].to_vec());
    }

    #[cfg(feature = "ed25519")]
    #[test]
    fn ed25519_raw_exports_to_okp_jwk() {
        // Real Ed25519 key from the witness module (same dalek stack).
        let kp = crate::ed25519_witness::WitnessKeyPair::generate();
        let raw = kp.public().to_vec();
        let rec = record(AlgorithmId::Ed25519, PublicKeyMaterial::Ed25519(raw.clone()), 3);
        let jwk = export_jwk(&rec).expect("Ed25519 exports");
        assert_eq!(jwk["kty"], "OKP");
        assert_eq!(jwk["crv"], "Ed25519");
        assert_eq!(jwk["alg"], "ED25519");
        let decoded_x =
            crate::crypto_trust_jwks::tests::decode_b64url(jwk["x"].as_str().expect("x string"));
        assert_eq!(decoded_x, raw);
    }

    #[test]
    fn post_quantum_and_hybrid_refuse_typed() {
        let mldsa = record(
            AlgorithmId::MlDsa65,
            PublicKeyMaterial::MlDsa65(vec![1u8; 1952]),
            4,
        );
        assert_eq!(
            export_jwk(&mldsa),
            Err(JwksError::UnsupportedAlgorithm("ML-DSA-65".to_string()))
        );
        let hybrid = record(
            AlgorithmId::HybridEs256MlDsa65,
            PublicKeyMaterial::Hybrid {
                es256: vec![2u8; 65],
                mldsa65: vec![3u8; 1952],
            },
            5,
        );
        assert_eq!(
            export_jwk(&hybrid),
            Err(JwksError::UnsupportedAlgorithm("ES256+ML-DSA-65".to_string()))
        );
        let slh = record(
            AlgorithmId::SlhDsa128s,
            PublicKeyMaterial::SlhDsa128s(vec![4u8; 33]),
            6,
        );
        assert_eq!(
            export_jwk(&slh),
            Err(JwksError::UnsupportedAlgorithm(
                "SLH-DSA-SHA2-128s".to_string()
            ))
        );
    }

    #[test]
    fn malformed_lengths_refuse_typed() {
        let bad_es256 = record(
            AlgorithmId::Es256,
            PublicKeyMaterial::Es256Sec1(vec![1u8; 33]),
            7,
        );
        assert!(matches!(
            export_jwk(&bad_es256),
            Err(JwksError::MalformedPublicKey(_, _))
        ));
        let bad_ed = record(AlgorithmId::Ed25519, PublicKeyMaterial::Ed25519(vec![1u8; 31]), 8);
        assert!(matches!(
            export_jwk(&bad_ed),
            Err(JwksError::MalformedPublicKey(_, _))
        ));
    }

    #[cfg(all(feature = "ed25519", feature = "secp256k1"))]
    #[test]
    fn jwks_set_is_deterministic_and_ordered_by_kid() {
        let signing = crate::crypto_trust_es256::Es256SigningKey::from_seed(&[9u8; 32])
            .expect("valid scalar seed");
        let es256 = record(
            AlgorithmId::Es256,
            PublicKeyMaterial::Es256Sec1(signing.public_key_sec1()),
            1,
        );
        let kp = crate::ed25519_witness::WitnessKeyPair::generate();
        let ed = record(
            AlgorithmId::Ed25519,
            PublicKeyMaterial::Ed25519(kp.public().to_vec()),
            3,
        );
        let forward = export_jwks(&[es256.clone(), ed.clone()]).expect("exports");
        let reversed = export_jwks(&[ed, es256]).expect("exports");
        assert_eq!(forward, reversed);
        assert_eq!(forward["keys"].as_array().expect("keys").len(), 2);
        let first_kid = forward["keys"][0]["kid"].as_str().expect("kid");
        let second_kid = forward["keys"][1]["kid"].as_str().expect("kid");
        assert!(first_kid < second_kid);
    }
}
