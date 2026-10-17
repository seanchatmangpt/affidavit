//! External validation court for the PQC trust plane (FIPS 204 ML-DSA-65).
//!
//! Law (operator, 2026-10-16, amended in-session): **all cryptography comes
//! from external validation.** `ml-dsa` (RustCrypto, FIPS 204) is an
//! unconditional [dependencies] entry of affidavit-core; this court never
//! hand-rolls any algorithm — every signature operation below is the external
//! crate's, exercised through the in-crate seam:
//!
//! 1. **Envelope/pre-image seam** (`affidavit_core::crypto_verify`): the
//!    env-001-ML-DSA-65 KAT document decodes and reproduces its published
//!    signing pre-image byte-for-byte, and the pre-image is mutation-sensitive.
//! 2. **External ML-DSA-65 end-to-end** (`affidavit_core::pqc`): real
//!    `ml_dsa::SigningKey` sign -> `ExternalMlDsa65`/`PqcCourt` verify.
//!    Genuine signatures are accepted; forged, cross-key, and malformed
//!    signatures are refused with typed refusals; the empty court refuses
//!    with `MissingMlDsa65Verifier` rather than skipping.
//!
//! KAT provenance: the canonical document and pre-image hex below are the
//! fixture bytes from `fixtures/crypto_trust_kat_vectors.json`
//! (surfaces.envelope, vector env-001-ML-DSA-65) as pinned in
//! `src/crypto_verify.rs` (KAT constants). They are hardcoded literals, not
//! computed at test time; they were extracted byte-exactly from
//! `src/crypto_verify.rs` mechanically (no hand transcription) and re-diffed.

use affidavit_core::crypto_verify::{SignatureEnvelope, DOMAIN_TAG, ENVELOPE_VERSION};
use affidavit_core::pqc::{
    ExternalMlDsa65, ExternalSlhDsa128s, MlDsa65Verifier, PqcCourt, PqcProfile, PqcRefusal,
    PqcVerdict, SlhDsaVerifier, Suite,
};

// External FIPS 204 implementation — one of the two signature algorithms in play.
use ml_dsa::{KeyInit, Keypair, MlDsa65, SignatureEncoding, Signer, SigningKey};

// No coordinator dev-dependency action is required: ml-dsa is an unconditional
// [dependencies] entry (operator law, 2026-10-16) and this file needs nothing
// else.

/// Domain-separation prefix in the signing pre-image.
const DOMAIN_TAG_BYTES: &[u8] = b"affidavit.crypto-trust-plane.v1";

/// KAT vector env-001-ML-DSA-65: canonical document bytes.
const V1_CANONICAL: &str = r#"{"algorithm":"ML_DSA65","audience":"affidavit.kat","expires_at":4102444800,"generation":1,"key_id":"afk1_932a436a743d67cd","nonce":[49,148,240,225,154,244,132,33,20,7,183,98,118,211,47,50],"not_before":1700000000,"policy_epoch":1,"profile":"PQC","revocation_epoch":0,"subject_digest":[18,60,249,28,128,193,211,38,120,198,222,80,164,85,43,52,17,76,4,173,228,175,245,220,104,114,253,206,55,248,114,68],"version":"CTP-ENVELOPE-v1"}"#;

/// KAT vector env-001-ML-DSA-65: expected signing pre-image.
const V1_SIGNING_INPUT_HEX: &str = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d";

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex digit"))
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ---------------------------------------------------------------------------
// Layer 1 — in-crate envelope/pre-image seam.
// ---------------------------------------------------------------------------

/// The ML-DSA-65 KAT document decodes with the real owned decoder and
/// reproduces the published signing pre-image byte-for-byte.
#[test]
fn envelope_kat_pre_image_matches() {
    let env = SignatureEnvelope::from_json(V1_CANONICAL.as_bytes()).expect("KAT doc decodes");
    let pre = env.signing_input().expect("pre-image builds");
    assert_eq!(
        hex_encode(&pre),
        V1_SIGNING_INPUT_HEX,
        "envelope pre-image diverged from KAT fixture"
    );
}

/// The pre-image opens with the domain tag; document mutation moves the
/// pre-image (forgery sensitivity of the seam being signed).
#[test]
fn envelope_kat_pre_image_domain_tagged_and_mutation_sensitive() {
    let env = SignatureEnvelope::from_json(V1_CANONICAL.as_bytes()).expect("KAT doc decodes");
    let pre = env.signing_input().expect("pre-image builds");
    assert!(pre.starts_with(DOMAIN_TAG_BYTES), "domain tag missing");
    assert_eq!(ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
    assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");

    // Sabotage twin: one-digit subject_digest flip must change the pre-image,
    // proving the assertion pins real bytes, not a constant echo.
    let mut forged_doc = V1_CANONICAL.to_string();
    let needle = "\"subject_digest\":[";
    let at = forged_doc.find(needle).expect("field present");
    let digit_at = at + needle.len();
    assert_eq!(
        forged_doc.as_bytes()[digit_at],
        b'1',
        "expected leading 1 in KAT subject_digest"
    );
    forged_doc.replace_range(digit_at..digit_at + 1, "2");
    assert_ne!(forged_doc, V1_CANONICAL);
    let forged = SignatureEnvelope::from_json(forged_doc.as_bytes()).expect("forged doc decodes");
    let forged_pre = forged.signing_input().expect("forged pre-image builds");
    assert_ne!(
        hex_encode(&forged_pre),
        V1_SIGNING_INPUT_HEX,
        "mutation did not move the pre-image — court would be vacuous"
    );
}

// ---------------------------------------------------------------------------
// Layer 2 — external ML-DSA-65 end-to-end through the pqc seam.
// ---------------------------------------------------------------------------

/// Sign the KAT pre-image with a real external ml-dsa key and have the
/// in-crate [`ExternalMlDsa65`] adapter (which calls the external crate's
/// verifier) accept it.
#[test]
fn external_mldsa65_accepts_genuine_signature() {
    let pre_image = hex_decode(V1_SIGNING_INPUT_HEX);
    let (pk, sig) = real_sig_over(&pre_image, [7u8; 32]);
    assert!(
        ExternalMlDsa65.verify(&pre_image, &sig, &pk),
        "ExternalMlDsa65 refused a genuine FIPS 204 signature"
    );
}

/// The same genuine signature, judged by the full [`PqcCourt`] under the
/// `Pqc` profile: accepted. Bit-flipped message, forged signature, and
/// cross-key signature are all refused with the typed refusal.
#[test]
fn pqc_court_accepts_genuine_and_refuses_forgeries() {
    let pre_image = hex_decode(V1_SIGNING_INPUT_HEX);
    let court = PqcCourt::new().with_ml_dsa(&ExternalMlDsa65);
    assert!(court.has_ml_dsa());

    let (pk, sig) = real_sig_over(&pre_image, [11u8; 32]);
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::MlDsa65),
            &pre_image,
            (&sig, &pk),
            (&[], &[])
        ),
        PqcVerdict::Accept,
        "court refused a genuine FIPS 204 signature"
    );

    // Forgery 1: bit-flipped message under the same signature.
    let mut forged_msg = pre_image.clone();
    let last = forged_msg.len() - 1;
    forged_msg[last] ^= 0x01;
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::MlDsa65),
            &forged_msg,
            (&sig, &pk),
            (&[], &[])
        ),
        PqcVerdict::Refuse(PqcRefusal::MlDsa65Rejected),
        "court accepted a bit-flipped message"
    );

    // Forgery 2: garbage signature bytes (malformed input collapses to false).
    let mut forged_sig = sig.clone();
    forged_sig[0] ^= 0xFF;
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::MlDsa65),
            &pre_image,
            (&forged_sig, &pk),
            (&[], &[])
        ),
        PqcVerdict::Refuse(PqcRefusal::MlDsa65Rejected),
        "court accepted a corrupted signature"
    );

    // Forgery 3: cross-key — a different real key's pk must refuse.
    let (other_pk, _) = real_sig_over(b"other", [23u8; 32]);
    assert_ne!(other_pk, pk, "two seeds produced the same key");
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::MlDsa65),
            &pre_image,
            (&sig, &other_pk),
            (&[], &[])
        ),
        PqcVerdict::Refuse(PqcRefusal::MlDsa65Rejected),
        "court accepted a cross-key signature"
    );
}

/// The empty court (no verifier registered) refuses a Pqc-profile demand with
/// the typed MissingMlDsa65Verifier refusal — never a silent skip.
#[test]
fn empty_court_refuses_pqc_profile_without_verifier() {
    let court = PqcCourt::new();
    assert!(!court.has_ml_dsa());
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::MlDsa65),
            b"msg",
            (&[], &[]),
            (&[], &[])
        ),
        PqcVerdict::Refuse(PqcRefusal::MissingMlDsa65Verifier)
    );
}

/// Malformed pk/sig lengths collapse to `false` in the external adapter
/// without panicking (fail-closed contract of [`MlDsa65Verifier`]).
#[test]
fn external_adapter_is_fail_closed_on_malformed_inputs() {
    assert!(!ExternalMlDsa65.verify(b"msg", &[], &[]));
    assert!(!ExternalMlDsa65.verify(b"msg", &[0u8; 10], &[0u8; 10]));
    assert!(!ExternalMlDsa65.verify(b"msg", &[0u8; 3309], &[0u8; 1951]));
    assert!(!ExternalMlDsa65.verify(b"msg", &[0u8; 3310], &[0u8; 1952]));
}

/// Real sign over an arbitrary message with a fixed-seed key; returns
/// (pk, sig) wire bytes via the external crate's encodings only.
fn real_sig_over(msg: &[u8], seed: [u8; 32]) -> (Vec<u8>, Vec<u8>) {
    let sk = SigningKey::<MlDsa65>::new(&ml_dsa::Seed::from(seed));
    let vk = sk.verifying_key();
    let sig = sk.sign(msg);
    let pk: [u8; 1952] = vk.encode().into();
    let sig_bytes: [u8; 3309] = sig.to_bytes().into();
    (pk.to_vec(), sig_bytes.to_vec())
}

// ---------------------------------------------------------------------------
// Layer 3 — external SLH-DSA-SHA2-128s (FIPS 205) end-to-end.
// ---------------------------------------------------------------------------

// External FIPS 205 implementation — the second signature algorithm in play.
use slh_dsa::signature::Verifier as SlhVerifier;
use slh_dsa::{
    Sha2_128s, Signature as SlhSignature, SigningKey as SlhSigningKey,
    VerifyingKey as SlhVerifyingKey,
};

/// Fixed 32-byte seeds for the FIPS 205 key generation (slh_keygen_internal,
/// the FIPS-205 KAT constructor). Arbitrary constants; determinism is what
/// matters, secrecy of these literals does not.
const SLH_SEED: [u8; 16] = [42u8; 16];
const SLH_PRF: [u8; 16] = [43u8; 16];
const SLH_PK_SEED: [u8; 16] = [44u8; 16];

/// Real external slh-dsa sign over a fixed message with a fixed-seed key;
/// returns (pk, sig) wire bytes via the external crate's encodings only.
fn real_slh_sig_over(msg: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let sk = SlhSigningKey::<Sha2_128s>::slh_keygen_internal(&SLH_SEED, &SLH_PRF, &SLH_PK_SEED);
    let vk = sk.verifying_key();
    let sig = sk.sign(msg); // deterministic Signer impl
    let pk: [u8; 32] = vk.to_bytes().into();
    let sig_bytes: [u8; 7856] = sig.to_bytes().into();
    (pk.to_vec(), sig_bytes.to_vec())
}

/// Sign a fixed message with a real external `slh_dsa::SigningKey<Sha2_128s>`
/// from a fixed seed and have the in-crate [`ExternalSlhDsa128s`] adapter
/// (which calls the external crate's verifier) accept it; a bit-flipped
/// message must be refused.
#[test]
fn external_slh_dsa128s_accepts_genuine_signature() {
    let msg = b"affidavit slh-dsa-128s external validation message";
    let (pk, sig) = real_slh_sig_over(msg);
    assert_eq!(pk.len(), 32);
    assert_eq!(sig.len(), 7856);
    assert!(
        ExternalSlhDsa128s.verify(msg, &sig, &pk),
        "ExternalSlhDsa128s refused a genuine FIPS 205 signature"
    );

    // Sabotage twin: one flipped message bit must be refused — the acceptance
    // above pins real verification, not a constant echo.
    let mut forged_msg = *msg;
    forged_msg[0] ^= 0x01;
    assert!(
        !ExternalSlhDsa128s.verify(&forged_msg, &sig, &pk),
        "ExternalSlhDsa128s accepted a bit-flipped message"
    );

    // Cross-check that the adapter's verdict tracks the external crate's own
    // verifier on the same bytes (adapter cannot diverge from the dependency).
    let vk = SlhVerifyingKey::<Sha2_128s>::try_from(pk.as_slice()).expect("pk decodes");
    let sigma = SlhSignature::<Sha2_128s>::try_from(sig.as_slice()).expect("sig decodes");
    assert!(vk.verify(msg, &sigma).is_ok());
    assert!(vk.verify(&forged_msg, &sigma).is_err());
}

/// Real keypairs for BOTH algorithms through [`PqcCourt`] under
/// [`Suite::Dual`]: genuine halves accept; a flipped SLH half refuses with
/// the typed `SlhDsaRejected` (fail-fast after the genuine ML-DSA half).
#[test]
fn pqc_court_dual_suite_accepts_and_refuses_over_real_keys() {
    let msg = b"affidavit dual-suite external validation message";
    let (ml_pk, ml_sig) = real_sig_over(msg, [31u8; 32]);
    let (slh_pk, slh_sig) = real_slh_sig_over(msg);

    let court = PqcCourt::new()
        .with_ml_dsa(&ExternalMlDsa65)
        .with_slh_dsa(&ExternalSlhDsa128s);
    assert!(court.has_ml_dsa() && court.has_slh_dsa());

    // Genuine both halves: accept.
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::Dual),
            msg,
            (&ml_sig, &ml_pk),
            (&slh_sig, &slh_pk)
        ),
        PqcVerdict::Accept,
        "court refused a genuine dual-suite signature pair"
    );

    // Forged SLH half (bit-flipped sig) after a genuine ML-DSA half: refusal.
    let mut forged_slh = slh_sig.clone();
    forged_slh[0] ^= 0x01;
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::Dual),
            msg,
            (&ml_sig, &ml_pk),
            (&forged_slh, &slh_pk)
        ),
        PqcVerdict::Refuse(PqcRefusal::SlhDsaRejected),
        "court accepted a flipped SLH-DSA half"
    );

    // Sabotage twin for the ML-DSA half too: the refusal must be ML-typed.
    let mut forged_ml = ml_sig.clone();
    forged_ml[0] ^= 0x01;
    assert_eq!(
        court.judge(
            PqcProfile::Pqc(Suite::Dual),
            msg,
            (&forged_ml, &ml_pk),
            (&slh_sig, &slh_pk)
        ),
        PqcVerdict::Refuse(PqcRefusal::MlDsa65Rejected),
        "court accepted a flipped ML-DSA half"
    );
}
