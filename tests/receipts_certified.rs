//! Falsifier court for the OPTIONAL `certified-receipts` seam
//! ([`affidavit::receipts_certified`]). Chicago discipline: the real
//! VerificationEngine, real ES256 keys, real signatures — no doubles.
//!
//! Falsifiers pinned here:
//! 1. tampered `payload_hash_hex` breaks the subject/signature binding;
//! 2. an empty subject is refused;
//! 3. a flipped signature byte refuses;
//! 4. the honest path certifies and re-verifies end to end.

#![cfg(feature = "certified-receipts")]

use affidavit::crypto_trust_es256::Es256SigningKey;
use affidavit::crypto_trust_verify::VerifyRefusal;
use affidavit::receipts_certified::{
    build_canonical_subject, certify_paid_delivery_payload, verify_certified_paid_delivery,
};

const PAYLOAD_HASH: &str = "3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f";
const SUBJECT: &str = "subject-a";

#[test]
fn crypto_trust_certified_honest_path_certifies_and_reverifies() {
    let signing = Es256SigningKey::generate().expect("key");
    let certified =
        certify_paid_delivery_payload(PAYLOAD_HASH, SUBJECT, &signing).expect("certify");
    // The receipt names the FULL canonical subject (payload hash inside).
    assert!(certified.receipt.subject.contains(PAYLOAD_HASH));
    assert!(certified.receipt.subject.contains(SUBJECT));
    assert!(certified
        .receipt
        .subject
        .starts_with("affidavit-paid-delivery/v1|"));
    verify_certified_paid_delivery(&certified, PAYLOAD_HASH, SUBJECT).expect("reverify");
}

#[test]
fn crypto_trust_certified_tampered_payload_hash_breaks_the_binding() {
    let signing = Es256SigningKey::generate().expect("key");
    let certified =
        certify_paid_delivery_payload(PAYLOAD_HASH, SUBJECT, &signing).expect("certify");
    let tampered = "0".repeat(64);
    assert_ne!(tampered, PAYLOAD_HASH);
    match verify_certified_paid_delivery(&certified, &tampered, SUBJECT) {
        Err(VerifyRefusal::SubjectMismatch(_)) => {}
        other => panic!("expected SubjectMismatch, got {other:?}"),
    }
}

#[test]
fn crypto_trust_certified_empty_subject_is_refused() {
    let signing = Es256SigningKey::generate().expect("key");
    match certify_paid_delivery_payload(PAYLOAD_HASH, "", &signing) {
        Err(VerifyRefusal::SubjectMismatch(_)) => {}
        other => panic!("expected SubjectMismatch, got {other:?}"),
    }
}

#[test]
fn crypto_trust_certified_flipped_signature_byte_refuses() {
    let signing = Es256SigningKey::generate().expect("key");
    let mut certified =
        certify_paid_delivery_payload(PAYLOAD_HASH, SUBJECT, &signing).expect("certify");
    // Flip one hex nibble mid-signature.
    let mid = certified.signature_hex.len() / 2;
    let flipped = if &certified.signature_hex[mid..mid + 1] == "0" {
        "8"
    } else {
        "0"
    };
    certified.signature_hex.replace_range(mid..mid + 1, flipped);
    match verify_certified_paid_delivery(&certified, PAYLOAD_HASH, SUBJECT) {
        Err(VerifyRefusal::InvalidSignature(_) | VerifyRefusal::Provider(_)) => {}
        other => panic!("expected InvalidSignature, got {other:?}"),
    }
}

#[test]
fn crypto_trust_certified_canonical_subject_layout_is_pinned() {
    assert_eq!(
        build_canonical_subject("abc", "s"),
        "affidavit-paid-delivery/v1|s|abc"
    );
}
