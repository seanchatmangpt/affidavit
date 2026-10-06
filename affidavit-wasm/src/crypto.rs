//! The signature-envelope binding check: `verify_signature_input` recomputes
//! the exact bytes an envelope's signature must cover and compares them with
//! the caller's expectation — **without any cryptography**.
//!
//! Boundary (the wasm twin of the rendered plane's trust-model note): this
//! module contains no EC and no ML-DSA math. It proves *which bytes must be
//! signed* — `DOMAIN_TAG || 0x00 || DOMAIN_TAG || 0x00 || u64_be(len) ||
//! jcs(envelope_document)` — so a host with its own verifier (a browser
//! WebCrypto `ECDSA`, a Node `crypto` ML-DSA build, an external HSM) can
//! check a signature over exactly those bytes. It never claims a signature is
//! valid: `verified: true` means "the expected signing input matches what this
//! envelope binds", not "the signature is good". Signing is likewise out of
//! scope — there is no private-key path through this module.
//!
//! The whole law lives in `affidavit-core::crypto_verify` (no_std, zero-dep);
//! this module is the JSON-ABI adapter: it maps core's typed refusals onto the
//! ABI's structured error surface and never panics on any input.

use crate::abi::{err, field, hex_decode, AbiError};
use affidavit_core::crypto_verify::SignatureEnvelope;
use serde_json::{json, Map, Value};

/// Typed refusal of a signing-input binding check; every variant is surfaced
/// as a structured ABI error, never a panic.
#[derive(Debug, PartialEq, Eq)]
pub enum SignatureInputError {
    /// The envelope document was refused by the core law (not JSON, wrong
    /// shape, wrong `version`, wrong array length, non-I-JSON integer).
    Envelope(affidavit_core::crypto_verify::EnvelopeError),
    /// `expected_signing_input_hex` is not valid hex (odd length or a
    /// non-hex digit), so the comparison could not be formed.
    BadExpectedHex,
}

impl SignatureInputError {
    /// Stable machine-readable ABI error code.
    pub fn code(&self) -> &'static str {
        match self {
            SignatureInputError::Envelope(e) => match e {
                affidavit_core::crypto_verify::EnvelopeError::WrongVersion => "wrong_version",
                affidavit_core::crypto_verify::EnvelopeError::NonCanonicalNumber(_) => {
                    "non_canonical_number"
                }
                affidavit_core::crypto_verify::EnvelopeError::Malformed(_)
                | affidavit_core::crypto_verify::EnvelopeError::NotYetValid(_)
                | affidavit_core::crypto_verify::EnvelopeError::Expired(_)
                | affidavit_core::crypto_verify::EnvelopeError::BufferTooSmall { .. } => {
                    "malformed"
                }
            },
            SignatureInputError::BadExpectedHex => "bad_hex",
        }
    }
}

impl core::fmt::Display for SignatureInputError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SignatureInputError::Envelope(e) => write!(f, "{e}"),
            SignatureInputError::BadExpectedHex => {
                write!(f, "expected_signing_input_hex is not valid hex")
            }
        }
    }
}

/// Lowercase-hex encoding (for echoing the recomputed pre-image).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

/// The binding check: decode `envelope_json` under the core envelope law,
/// recompute its signing pre-image, and compare it byte-for-byte with the
/// decoded `expected_signing_input_hex`.
///
/// - `Ok(true)` — the expected pre-image is exactly what this envelope binds.
/// - `Ok(false)` — the envelope decoded and canonicalized, but binds different
///   bytes than expected (a mismatched or tampered document).
/// - `Err(_)` — typed refusal: the comparison could not lawfully be formed.
///
/// This is a byte-identity check over public material (the pre-image is not
/// secret), so ordinary equality is the honest comparison.
pub fn verify_signature_input(
    envelope_json: &[u8],
    expected_signing_input_hex: &str,
) -> Result<bool, SignatureInputError> {
    check(envelope_json, expected_signing_input_hex).map(|(verified, _)| verified)
}

/// The check plus the recomputed pre-image, so hosts can see WHICH bytes must
/// be signed even on a mismatch.
fn check(
    envelope_json: &[u8],
    expected_signing_input_hex: &str,
) -> Result<(bool, Vec<u8>), SignatureInputError> {
    let expected =
        hex_decode(expected_signing_input_hex).ok_or(SignatureInputError::BadExpectedHex)?;
    let envelope =
        SignatureEnvelope::from_json(envelope_json).map_err(SignatureInputError::Envelope)?;
    let computed = envelope
        .signing_input()
        .map_err(SignatureInputError::Envelope)?;
    Ok((computed == expected, computed))
}

/// Domain-separated BLAKE3 subject digest, with the domain tag as an INPUT.
///
/// This is the rendered plane's [`affidavit_core`] signing-input law —
/// `digest(DOMAIN_TAG, [subject])` — with the constant replaced by the
/// caller's tag, so a host can derive digests for any domain, not just
/// `affidavit.crypto-trust-plane.v1`. Pre-image:
/// `tag || 0x00 || tag || 0x00 || u64_be(len) || subject` (the explicit
/// length prefix makes the boundary unambiguous under concatenation).
pub fn derive_subject_digest(domain_tag: &str, subject: &[u8]) -> [u8; 32] {
    blake3::hash(&pre_image(domain_tag, subject)).into()
}

/// The ABI op behind `{"op":"derive_subject_digest"}`. Request:
/// `{"domain_tag": "<string>", "subject": "<string>"}` or
/// `{"domain_tag": ..., "subject_hex": "<hex bytes>"}` (exactly one of the
/// two subject fields). Response adds `digest_hex` (64 lowercase hex chars)
/// and `pre_image_hex` (the exact bytes hashed, so hosts can audit the
/// domain separation without re-deriving the law).
pub(crate) fn op_derive_subject_digest(req: &Value) -> Result<Map<String, Value>, AbiError> {
    let domain_tag = field(req, "domain_tag")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`domain_tag` must be a string"))?;
    let subject: Vec<u8> = match (
        req.get("subject").and_then(Value::as_str),
        req.get("subject_hex").and_then(Value::as_str),
    ) {
        (Some(_), Some(_)) => {
            return Err(err(
                "bad_field",
                "pass exactly one of `subject` or `subject_hex`, not both",
            ))
        }
        (Some(s), None) => s.as_bytes().to_vec(),
        (None, Some(h)) => hex_decode(h).ok_or_else(|| {
            err("bad_field", "`subject_hex` must be valid lowercase or uppercase hex")
        })?,
        (None, None) => {
            return Err(err(
                "missing_field",
                "request needs one of `subject` or `subject_hex`",
            ))
        }
    };
    if domain_tag.is_empty() {
        return Err(err("bad_field", "`domain_tag` must not be empty"));
    }
    let digest = derive_subject_digest(domain_tag, &subject);
    let pre_image = pre_image(domain_tag, &subject);
    Ok(crate::abi::obj(json!({
        "digest_hex": hex_encode(&digest),
        "pre_image_hex": hex_encode(&pre_image),
    })))
}

/// The exact bytes hashed, exposed for audit.
fn pre_image(domain_tag: &str, subject: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2 * domain_tag.len() + 2 + 8 + subject.len());
    buf.extend_from_slice(domain_tag.as_bytes());
    buf.push(0x00);
    buf.extend_from_slice(domain_tag.as_bytes());
    buf.push(0x00);
    buf.extend_from_slice(&(subject.len() as u64).to_be_bytes());
    buf.extend_from_slice(subject);
    buf
}

/// The ABI op behind `{"op":"verify_signature_input"}`. Request:
/// `{"envelope_json": "<envelope document JSON>",
///   "expected_signing_input_hex": "<hex>"}`. Response adds `verified`,
/// `signing_input_hex` (the recomputed pre-image) and `canonical_bytes_len`.
pub(crate) fn op_verify_signature_input(req: &Value) -> Result<Map<String, Value>, AbiError> {
    let envelope_json = field(req, "envelope_json")?.as_str().ok_or_else(|| {
        err(
            "bad_field",
            "`envelope_json` must be a string holding the envelope document JSON",
        )
    })?;
    let expected_hex = field(req, "expected_signing_input_hex")?
        .as_str()
        .ok_or_else(|| err("bad_field", "`expected_signing_input_hex` must be a string"))?;
    let canonical_bytes_len = SignatureEnvelope::from_json(envelope_json.as_bytes())
        .map_err(SignatureInputError::Envelope)
        .and_then(|e| e.to_bytes().map_err(SignatureInputError::Envelope))
        .map(|b| b.len())
        .map_err(|e| err(e.code(), e.to_string()))?;
    let (verified, computed) =
        check(envelope_json.as_bytes(), expected_hex).map_err(|e| err(e.code(), e.to_string()))?;
    Ok(crate::abi::obj(json!({
        "verified": verified,
        "signing_input_hex": hex_encode(&computed),
        "canonical_bytes_len": canonical_bytes_len,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical KAT document env-000-ES256 (rendered plane,
    /// fixtures/crypto_trust_kat_vectors.json surfaces.envelope[0]).
    const V0_CANONICAL: &str = r#"{"algorithm":"ES256","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_a9c3bd446195e2e2","nonce":[25,107,217,46,141,94,95,54,120,72,204,59,49,154,244,42],"not_before":1700000000,"policy_epoch":1,"profile":"CLASSICAL","revocation_epoch":0,"subject_digest":[206,63,11,26,126,80,218,245,190,191,110,203,240,54,52,94,36,141,20,88,5,127,63,121,250,128,109,66,210,110,68,97],"version":"CTP-ENVELOPE-v1"}"#;
    /// KAT env-000-ES256: expected signing pre-image.
    const V0_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";

    #[test]
    fn kat_document_binds_the_rendered_pre_image() {
        assert_eq!(
            verify_signature_input(V0_CANONICAL.as_bytes(), V0_SIGNING_INPUT_HEX),
            Ok(true)
        );
        // One nibble of drift in the expectation: a mismatch, not an error.
        let mut tampered = V0_SIGNING_INPUT_HEX.to_string();
        let last = tampered.len() - 1;
        let flip = if tampered.ends_with('d') { "e" } else { "d" };
        tampered.replace_range(last.., flip);
        assert_eq!(
            verify_signature_input(V0_CANONICAL.as_bytes(), &tampered),
            Ok(false)
        );
    }

    /// KAT extracted from the rendered plane's
    /// `crypto_trust_canonical::digest_hex(DOMAIN_TAG, [subject])` law with
    /// DOMAIN_TAG := "affidavit-paid-delivery/v1" (computed against
    /// blake3 1.8.5, the pinned crate in Cargo.lock).
    const KAT_TAG: &str = "affidavit-paid-delivery/v1";
    const KAT_SUBJECT: &str = "paid-delivery:evt-001";
    const KAT_DIGEST_HEX: &str =
        "3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f";

    #[test]
    fn kat_matches_the_rust_surface_digest_law() {
        assert_eq!(
            derive_subject_digest(KAT_TAG, KAT_SUBJECT.as_bytes()),
            {
                let mut d = [0u8; 32];
                for i in 0..32 {
                    d[i] = u8::from_str_radix(&KAT_DIGEST_HEX[2 * i..2 * i + 2], 16).unwrap();
                }
                d
            }
        );
    }

    #[test]
    fn domains_are_separated() {
        let a = derive_subject_digest("affidavit-paid-delivery/v1", b"x");
        let b = derive_subject_digest("affidavit.other-domain/v1", b"x");
        assert_ne!(a, b);
        // Same tag in both slots: flipping the constant slot alone must move
        // the digest (guards against a single-slot pre-image regression).
        let pre = pre_image("affidavit-paid-delivery/v1", b"x");
        let mut single = Vec::with_capacity(pre.len());
        single.extend_from_slice(KAT_TAG.as_bytes());
        single.push(0x00);
        single.extend_from_slice(b"x");
        assert_ne!(pre, single);
    }

    #[test]
    fn deterministic_and_length_prefixed() {
        assert_eq!(
            derive_subject_digest(KAT_TAG, b"subject"),
            derive_subject_digest(KAT_TAG, b"subject")
        );
        // Boundary ambiguity under concatenation: ("ab","c") must differ
        // from ("a","bc") because parts are length-prefixed.
        assert_ne!(pre_image(KAT_TAG, b"abc"), {
            let mut v = Vec::new();
            v.extend_from_slice(KAT_TAG.as_bytes());
            v.push(0x00);
            v.extend_from_slice(b"abc");
            v
        });
        // Empty subject is a lawful input, not an error.
        assert_ne!(derive_subject_digest(KAT_TAG, b""), [0u8; 32]);
    }

    #[test]
    fn op_accepts_string_and_hex_subjects_and_refuses_bad_input() {
        use crate::abi::Res;
        let by_string = op_derive_subject_digest(&serde_json::json!({
            "domain_tag": KAT_TAG,
            "subject": KAT_SUBJECT,
        }))
        .unwrap();
        let by_hex = op_derive_subject_digest(&serde_json::json!({
            "domain_tag": KAT_TAG,
            "subject_hex": KAT_SUBJECT.as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>(),
        }))
        .unwrap();
        assert_eq!(
            by_string.get("digest_hex").and_then(Value::as_str),
            Some(KAT_DIGEST_HEX)
        );
        assert_eq!(by_string, by_hex);
        // Pre-image audit field: tag twice, 0x00 separators, be8 length.
        let pre = hex_decode(
            by_string
                .get("pre_image_hex")
                .and_then(Value::as_str)
                .unwrap(),
        )
        .unwrap();
        let tag = KAT_TAG.as_bytes();
        assert_eq!(&pre[..tag.len()], tag);
        assert_eq!(pre[tag.len()], 0x00);
        assert_eq!(&pre[tag.len() + 1..2 * tag.len() + 1], tag);
        assert_eq!(pre[2 * tag.len() + 1], 0x00);
        assert_eq!(&pre[2 * tag.len() + 2..2 * tag.len() + 10], &21u64.to_be_bytes());
        assert_eq!(&pre[2 * tag.len() + 10..], KAT_SUBJECT.as_bytes());
        // Refusals: both subject fields, neither, empty tag, bad hex.
        let r: Res<_> = op_derive_subject_digest(&serde_json::json!({
            "domain_tag": KAT_TAG, "subject": "a", "subject_hex": "61"
        }));
        assert_eq!(r.unwrap_err().code, "bad_field");
        let r: Res<_> =
            op_derive_subject_digest(&serde_json::json!({"domain_tag": KAT_TAG}));
        assert_eq!(r.unwrap_err().code, "missing_field");
        let r: Res<_> =
            op_derive_subject_digest(&serde_json::json!({"domain_tag": "", "subject": "a"}));
        assert_eq!(r.unwrap_err().code, "bad_field");
        let r: Res<_> = op_derive_subject_digest(&serde_json::json!({
            "domain_tag": KAT_TAG, "subject_hex": "zz"
        }));
        assert_eq!(r.unwrap_err().code, "bad_field");
    }

    #[test]
    fn refusals_are_typed() {        assert!(matches!(
            verify_signature_input(b"not json", V0_SIGNING_INPUT_HEX),
            Err(SignatureInputError::Envelope(
                affidavit_core::crypto_verify::EnvelopeError::Malformed(_)
            ))
        ));
        assert_eq!(
            verify_signature_input(V0_CANONICAL.as_bytes(), "zz"),
            Err(SignatureInputError::BadExpectedHex)
        );
        assert_eq!(
            verify_signature_input(V0_CANONICAL.as_bytes(), "abc"),
            Err(SignatureInputError::BadExpectedHex)
        );
        // Uppercase hex is still hex: accepted.
        assert_eq!(
            verify_signature_input(
                V0_CANONICAL.as_bytes(),
                &V0_SIGNING_INPUT_HEX.to_uppercase()
            ),
            Ok(true)
        );
    }
}
