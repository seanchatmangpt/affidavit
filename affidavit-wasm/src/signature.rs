//! The `verify_signature` op: stateless per-algorithm signature adjudication
//! over caller-supplied bytes — the JSON-ABI twin of the rendered plane's
//! per-alg verify lanes (AG1: ES256 / Ed25519 / ES256K).
//!
//! Boundary: this module holds no registry, no revocation, no replay state and
//! no policy — it answers ONE question: does `signature` verify over
//! `signing_input` under `public_key` for `alg`? The envelope law (window,
//! audience, profile, kid-binding, revocation, replay) lives in the rendered
//! plane's verifier; a host that needs those checks drives `verify_envelope`
//! there. Like [`crate::crypto`], it never claims more than it checked:
//! `valid: true` means exactly "the bytes verify under the key".
//!
//! Laws mirror the rendered plane exactly:
//! - ES256: P-256 ECDSA (SHA-256), DER-encoded signature, SEC1 **uncompressed**
//!   public key (`0x04 || X || Y`), high-s refused (canonical-form law).
//! - ES256K: secp256k1 ECDSA (SHA-256), **fixed 64-byte** `r||s` signature,
//!   SEC1 **compressed** public key (`0x02/0x03 || X`), high-s refused.
//! - Ed25519: raw 32-byte public key, 64-byte signature (PureEdDSA).
//!
//! Typed refusals surface as `{"ok":true,"valid":false,"refusal":"..."}` — a
//! verifier answers "no" with a reason rather than throwing — while malformed
//! *requests* (missing/non-string fields, bad hex) are structured ABI errors.

use crate::abi::{err, field, hex_decode, obj, Res};
use serde_json::{json, Map, Value};

/// Adjudicate one signature through the ABI law. See the module header for the
/// per-algorithm wire forms.
pub(crate) fn op_verify_signature(req: &Value) -> Res<Map<String, Value>> {
    let alg = field(req, "alg")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`alg` must be a string"))?;
    let key_hex = field(req, "public_key_hex")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`public_key_hex` must be a string"))?;
    let sig_hex = field(req, "signature_hex")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`signature_hex` must be a string"))?;
    let input = signing_input(req)?;

    let key =
        hex_decode(key_hex).ok_or_else(|| err("bad_field", "`public_key_hex` is not valid hex"))?;
    let sig =
        hex_decode(sig_hex).ok_or_else(|| err("bad_field", "`signature_hex` is not valid hex"))?;

    let outcome = match alg {
        "ES256" => es256(&key, &input, &sig),
        "ES256K" => es256k(&key, &input, &sig),
        "Ed25519" => ed25519(&key, &input, &sig),
        other => Err(format!(
            "unsupported_algorithm: `{other}`; supported: ES256, ES256K, Ed25519"
        )),
    };
    Ok(match outcome {
        Ok(true) => obj(json!({"valid": true, "refusal": Value::Null})),
        Ok(false) => obj(json!({"valid": false, "refusal": "verification_failed"})),
        Err(refusal) => obj(json!({"valid": false, "refusal": refusal})),
    })
}

/// Signing-input bytes from `signing_input` (UTF-8 text) or `signing_input_hex`
/// (the same dual form the `commit`/`assemble` ops accept for payloads).
fn signing_input(req: &Value) -> Res<Vec<u8>> {
    if let Some(p) = req.get("signing_input") {
        let s = p
            .as_str()
            .ok_or_else(|| err("bad_field", "`signing_input` must be a string"))?;
        return Ok(s.as_bytes().to_vec());
    }
    if let Some(p) = req.get("signing_input_hex") {
        let s = p
            .as_str()
            .ok_or_else(|| err("bad_field", "`signing_input_hex` must be a string"))?;
        return hex_decode(s)
            .ok_or_else(|| err("bad_field", "`signing_input_hex` is not valid hex"));
    }
    Err(err(
        "missing_field",
        "verify_signature needs `signing_input` or `signing_input_hex`",
    ))
}

/// A well-formed key/signature for `alg` but wrong bytes: `Ok(false)`.
/// A malformed encoding (wrong length, bad DER, bad point) is a typed refusal.
type Outcome = Result<bool, String>;

fn es256(key: &[u8], input: &[u8], sig: &[u8]) -> Outcome {
    use p256::ecdsa::signature::Verifier;
    use p256::ecdsa::{DerSignature, Signature, VerifyingKey};
    use p256::elliptic_curve::scalar::IsHigh;
    let vk = VerifyingKey::from_sec1_bytes(key).map_err(|_| {
        "malformed_public_key: ES256 needs SEC1 uncompressed (0x04 || X || Y, 65 bytes)".to_string()
    })?;
    let sig = DerSignature::from_bytes(sig)
        .map_err(|_| "malformed_signature: ES256 needs a DER ECDSA-Sig-Value".to_string())
        .and_then(|der| {
            Signature::try_from(der)
                .map_err(|_| "malformed_signature: ES256 needs a DER ECDSA-Sig-Value".to_string())
        })?;
    // Canonical-form law (BIP-62 verifier side, the rendered plane's law):
    // (r, s) and (r, n - s) are both valid ECDSA; only the low-s member is
    // admissible so a flipped-sign re-present cannot double through.
    if sig.s().is_high().into() {
        return Err(
            "malformed_signature: high-s (canonical-form law admits only low-s)".to_string(),
        );
    }
    Ok(vk.verify(input, &sig).is_ok())
}

fn es256k(key: &[u8], input: &[u8], sig: &[u8]) -> Outcome {
    use k256::ecdsa::signature::Verifier;
    use k256::ecdsa::{Signature, VerifyingKey};
    use k256::elliptic_curve::scalar::IsHigh;
    let vk = VerifyingKey::from_sec1_bytes(key).map_err(|_| {
        "malformed_public_key: ES256K needs SEC1 compressed (0x02/0x03 || X, 33 bytes)".to_string()
    })?;
    let sig = Signature::from_slice(sig).map_err(|_| {
        "malformed_signature: ES256K needs a fixed 64-byte r||s signature".to_string()
    })?;
    if sig.s().is_high().into() {
        return Err(
            "malformed_signature: high-s (canonical-form law admits only low-s)".to_string(),
        );
    }
    Ok(vk.verify(input, &sig).is_ok())
}

fn ed25519(key: &[u8], input: &[u8], sig: &[u8]) -> Outcome {
    use ed25519_dalek::{Signature, VerifyingKey};
    let vk = VerifyingKey::from_bytes(
        key.try_into()
            .map_err(|_| "malformed_public_key: Ed25519 needs 32 raw bytes".to_string())?,
    )
    .map_err(|_| "malformed_public_key: not a valid Ed25519 curve point".to_string())?;
    let sig = Signature::from_slice(sig)
        .map_err(|_| "malformed_signature: Ed25519 needs 64 bytes".to_string())?;
    Ok(vk.verify_strict(input, &sig).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::call;

    fn run(v: Value) -> Value {
        serde_json::from_slice(&call(&serde_json::to_vec(&v).unwrap())).unwrap()
    }

    // RFC 8032 Ed25519 test vector 2 (deterministic, in-tree KAT).
    const ED_KEY: &str = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c";
    const ED_INPUT: &str = "72";
    const ED_SIG: &str = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00";

    #[test]
    fn rfc8032_ed25519_vector_verifies_through_the_abi() {
        let r = run(json!({
            "op": "verify_signature", "alg": "Ed25519",
            "public_key_hex": ED_KEY, "signature_hex": ED_SIG,
            "signing_input_hex": ED_INPUT,
        }));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["op"], "verify_signature");
        assert_eq!(r["valid"], true, "{r}");
        assert_eq!(r["refusal"], Value::Null);
    }

    #[test]
    fn a_flipped_input_bit_is_valid_false_not_an_error() {
        let r = run(json!({
            "op": "verify_signature", "alg": "Ed25519",
            "public_key_hex": ED_KEY, "signature_hex": ED_SIG,
            "signing_input_hex": "73",
        }));
        assert_eq!(r["valid"], false, "{r}");
        assert_eq!(r["refusal"], "verification_failed");
    }

    #[test]
    fn malformed_encodings_are_typed_refusals() {
        let key_bad_len = run(json!({
            "op": "verify_signature", "alg": "Ed25519",
            "public_key_hex": "aabb", "signature_hex": ED_SIG,
            "signing_input_hex": ED_INPUT,
        }));
        assert!(key_bad_len["refusal"]
            .as_str()
            .unwrap()
            .starts_with("malformed_public_key"));
        let sig_bad_len = run(json!({
            "op": "verify_signature", "alg": "Ed25519",
            "public_key_hex": ED_KEY, "signature_hex": "aabb",
            "signing_input_hex": ED_INPUT,
        }));
        assert!(sig_bad_len["refusal"]
            .as_str()
            .unwrap()
            .starts_with("malformed_signature"));
        // A well-formed key that is not THE key: decided `valid:false`
        // (`verification_failed`), not a refusal — the check is honest.
        let wrong_key = run(json!({
            "op": "verify_signature", "alg": "Ed25519",
            // RFC 8032 test-vector-1 public key, wrong for this signature.
            "public_key_hex": "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
            "signature_hex": ED_SIG, "signing_input_hex": ED_INPUT,
        }));
        assert_eq!(wrong_key["refusal"], "verification_failed", "{wrong_key}");
    }

    #[test]
    fn unsupported_and_mismatched_algs_are_typed() {
        let r = run(json!({
            "op": "verify_signature", "alg": "ML_DSA65",
            "public_key_hex": ED_KEY, "signature_hex": ED_SIG,
            "signing_input_hex": ED_INPUT,
        }));
        assert_eq!(r["valid"], false, "{r}");
        assert!(r["refusal"]
            .as_str()
            .unwrap()
            .starts_with("unsupported_algorithm"));
        // Right key family, wrong algorithm claim: a typed refusal on the
        // key-format mismatch, never a false "invalid".
        let mismatch = run(json!({
            "op": "verify_signature", "alg": "ES256K",
            "public_key_hex": ED_KEY, "signature_hex": ED_SIG,
            "signing_input_hex": ED_INPUT,
        }));
        assert!(mismatch["refusal"]
            .as_str()
            .unwrap()
            .starts_with("malformed_public_key"));
    }

    /// Lowercase hex for building requests in tests.
    fn hex(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        bytes
            .iter()
            .flat_map(|b| {
                [
                    HEX[(b >> 4) as usize] as char,
                    HEX[(b & 0x0f) as usize] as char,
                ]
            })
            .collect()
    }

    // Real RFC 6979 deterministic signing through the same stacks the rendered
    // plane signs with: ES256 DER (low-s) and ES256K fixed 64-byte (low-s).
    #[test]
    fn es256_round_trip_verifies_and_rejects_drift() {
        use p256::ecdsa::signature::Signer;
        use p256::ecdsa::SigningKey;
        let key = SigningKey::from_slice(&[7u8; 32]).unwrap();
        let sec1 = key.verifying_key().to_encoded_point(false);
        let input = b"affidavit.crypto-trust-plane.v1";
        let sig: p256::ecdsa::Signature = key.sign(input);
        let sig = sig.normalize_s().unwrap_or(sig);
        let r = run(json!({
            "op": "verify_signature", "alg": "ES256",
            "public_key_hex": &hex(sec1.as_bytes()),
            "signature_hex": hex(&sig.to_der().as_bytes()),
            "signing_input": String::from_utf8_lossy(input).into_owned(),
        }));
        assert_eq!(r["valid"], true, "{r}");
        // One drifted byte: valid:false, never a false positive.
        let bad = run(json!({
            "op": "verify_signature", "alg": "ES256",
            "public_key_hex": &hex(sec1.as_bytes()),
            "signature_hex": hex(&sig.to_der().as_bytes()),
            "signing_input": "affidavit.crypto-trust-plane.v2",
        }));
        assert_eq!(bad["valid"], false, "{bad}");
    }

    #[test]
    fn es256k_round_trip_verifies_and_refuses_high_s() {
        use k256::ecdsa::signature::Signer;
        use k256::ecdsa::SigningKey;
        use k256::elliptic_curve::scalar::IsHigh;
        use k256::elliptic_curve::sec1::ToSec1Point;
        let key = SigningKey::from_slice(&[9u8; 32]).unwrap();
        let compressed: Vec<u8> = k256::PublicKey::from(key.verifying_key())
            .to_sec1_point(true)
            .as_bytes()
            .to_vec();
        let input = b"affidavit.crypto-trust-plane.v1";
        let sig: k256::ecdsa::Signature = key.sign(input);
        let low_s = if sig.s().is_high().into() {
            k256::ecdsa::Signature::from_scalars(sig.r().to_bytes(), (-*sig.s()).to_bytes())
                .unwrap()
        } else {
            sig
        };
        let sig_bytes = low_s.to_bytes();
        let r = run(json!({
            "op": "verify_signature", "alg": "ES256K",
            "public_key_hex": &hex(&compressed),
            "signature_hex": hex(&sig_bytes),
            "signing_input": String::from_utf8_lossy(input).into_owned(),
        }));
        assert_eq!(r["valid"], true, "{r}");
        // The high-s mirror is refused by the canonical-form law, though its
        // ECDSA arithmetic is valid: the mutation proves the gate non-vacuous.
        let mirror =
            k256::ecdsa::Signature::from_scalars(sig.r().to_bytes(), (-*sig.s()).to_bytes())
                .unwrap();
        let bad = run(json!({
            "op": "verify_signature", "alg": "ES256K",
            "public_key_hex": &hex(&compressed),
            "signature_hex": hex(&mirror.to_bytes()),
            "signing_input": String::from_utf8_lossy(input).into_owned(),
        }));
        assert_eq!(bad["valid"], false, "{bad}");
        assert!(bad["refusal"]
            .as_str()
            .unwrap()
            .starts_with("malformed_signature"));
    }

    #[test]
    fn request_shape_errors_follow_the_abi_conventions() {
        let code = |req: Value| -> String {
            let r = run(req);
            assert_eq!(r["ok"], false, "{r}");
            r["error"]["code"].as_str().unwrap().to_string()
        };
        assert_eq!(code(json!({"op": "verify_signature"})), "missing_field");
        assert_eq!(
            code(json!({"op": "verify_signature", "alg": "Ed25519",
                "public_key_hex": ED_KEY, "signature_hex": ED_SIG})),
            "missing_field"
        );
        assert_eq!(
            code(json!({"op": "verify_signature", "alg": "Ed25519",
                "public_key_hex": "zz", "signature_hex": ED_SIG,
                "signing_input_hex": ED_INPUT})),
            "bad_field"
        );
        assert_eq!(
            code(json!({"op": "verify_signature", "alg": 7,
                "public_key_hex": ED_KEY, "signature_hex": ED_SIG,
                "signing_input_hex": ED_INPUT})),
            "bad_field"
        );
    }
}
