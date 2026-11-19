//! FNOX-SJ-PROV-3 courts: offline SLSA / in-toto envelope verification.
//!
//! The valid envelope is NOT forged here: it is the output of fnox PROV-1
//! sealing (`in_toto_gcp.seal_statement(build_slsa_statement(..))`) stored at
//! `tests/fixtures/slsa_provenance_sealed.json` as
//! `{"sealed": <envelope>, "public_key_sec1_hex": "<uncompressed SEC1 hex>"}`.
//! A missing fixture fails loudly; it is never skipped.

use affidavit::slsa_provenance::{
    sec1_from_hex, verify_sealed_envelope, SlsaError, SLSA_PREDICATE_TYPE,
};
use serde_json::Value;

fn load() -> (Value, Vec<u8>) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/slsa_provenance_sealed.json"
    );
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("fixture missing ({path}): produce via fnox PROV-1 sealing: {e}"));
    let v: Value = serde_json::from_str(&raw).expect("fixture is JSON");
    let key = sec1_from_hex(v["public_key_sec1_hex"].as_str().expect("public_key_sec1_hex"))
        .expect("hex key");
    (v["sealed"].clone(), key)
}

fn flip_char(s: &str, idx: usize) -> String {
    let mut b = s.as_bytes().to_vec();
    b[idx] = if b[idx] == b'A' { b'B' } else { b'A' };
    String::from_utf8(b).unwrap()
}

#[test]
fn slsa_envelope_valid_accepted() {
    let (sealed, key) = load();
    let ok = verify_sealed_envelope(&sealed, &key, SLSA_PREDICATE_TYPE)
        .expect("valid envelope must verify offline");
    assert_eq!(ok.predicate_type, SLSA_PREDICATE_TYPE);
    assert_eq!(ok.digest, sealed["digest"].as_str().unwrap());
}

#[test]
fn slsa_envelope_payload_byte_flip_rejected() {
    let (mut sealed, key) = load();
    let d = sealed["statement"]["subject"][0]["digest"]["sha256"]
        .as_str()
        .unwrap()
        .to_string();
    sealed["statement"]["subject"][0]["digest"]["sha256"] = Value::String(flip_char(&d, 0));
    let r = verify_sealed_envelope(&sealed, &key, SLSA_PREDICATE_TYPE);
    assert!(matches!(r, Err(SlsaError::DigestMismatch { .. })), "got {r:?}");
}

#[test]
fn slsa_envelope_signature_byte_flip_rejected() {
    let (mut sealed, key) = load();
    let sig = sealed["kms_signature"].as_str().unwrap().to_string();
    sealed["kms_signature"] = Value::String(flip_char(&sig, 10));
    let r = verify_sealed_envelope(&sealed, &key, SLSA_PREDICATE_TYPE);
    assert!(r.is_err(), "tampered signature accepted: {r:?}");
}

#[test]
fn slsa_envelope_wrong_predicate_rejected() {
    let (sealed, key) = load();
    let r = verify_sealed_envelope(&sealed, &key, "https://example.com/not-slsa/v1");
    assert!(matches!(r, Err(SlsaError::WrongPredicate { .. })), "got {r:?}");
}
