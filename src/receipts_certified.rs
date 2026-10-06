//! OPTIONAL certified-receipts upgrade for paid-delivery receipts.
//!
//! Seam contract (ggen-marketplace `affidavit-trust-plane-pack`
//! `HANDWRITTEN.md`): the paid-delivery chain stays the plain sha256 fold;
//! this seam adds signatures, never replaces it, and fails OPEN to
//! uncertified: an unsigned record still verifies fold-only.
//!
//! `certify_signed` enforces a SUBJECT BINDING LAW: the subject string is
//! digested as `BLAKE3(domain_separated(DOMAIN_TAG, [subject]))` and any
//! mismatch with `env.subject_digest` is refused. This seam binds the FULL
//! canonical string `"affidavit-paid-delivery/v1|<subject>|<payload_hash_hex>"`
//! as the certified subject, so the payload hash is inside the subject digest
//! and inside the envelope's signed bytes.

use crate::crypto_trust_canonical::digest;
use crate::crypto_trust_envelope::{NonceJournal, SignatureEnvelope, ENVELOPE_VERSION};
use crate::crypto_trust_es256::{Es256SigningKey, DOMAIN_TAG};
use crate::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CustodianIdentity, CryptoProfile, InMemoryKeyRegistry,
    KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
};
use crate::crypto_trust_lifecycle::RevocationList;
use crate::crypto_trust_verify::{
    CryptoStandingReceipt, CryptographicStanding, TrustPolicy, VerificationEngine, VerifyRefusal,
};

/// Domain prefix of the certified subject string. Versioned: a change to the
/// canonical layout is a new domain, never a silent rewrite.
pub const PAID_DELIVERY_DOMAIN: &str = "affidavit-paid-delivery/v1";

/// The allowlisted trust-plane audience this seam presents under.
pub const CERTIFIED_RECEIPT_AUDIENCE: &str = "affidavit.cli";

/// Envelope validity window in seconds.
pub const VALIDITY_WINDOW_SECONDS: u64 = 600;

/// A certified paid-delivery receipt: the sealed [`CryptoStandingReceipt`]
/// bound to the signed envelope, the ES256 (RFC 6979) signature over the
/// envelope's canonical bytes, and the verifying key material for offline
/// re-adjudication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertifiedReceiptEnvelope {
    /// The presented signature envelope (binds subject digest, key, window,
    /// nonce, audience, epochs).
    pub envelope: SignatureEnvelope,
    /// The certified standing receipt minted by
    /// [`VerificationEngine::certify_signed`] — itself signed by the same
    /// attestation key (RECEIPT SIGNATURE LAW).
    pub receipt: CryptoStandingReceipt,
    /// ES256 (RFC 6979, DER, low-s) signature over the envelope's
    /// domain-separated canonical bytes.
    pub signature_hex: String,
    /// The signer's public key, SEC1 uncompressed (`0x04 || X || Y`, 65
    /// bytes), lowercase hex. JWS/PEM are not emitted because the trust
    /// plane's admitted key-encoding law is SEC1
    /// ([`PublicKeyMaterial::Es256Sec1`]); the seam introduces no second key
    /// encoding.
    pub verifying_key_sec1_hex: String,
}

/// Certify one paid-delivery payload hash for `subject`.
///
/// Builds a [`SignatureEnvelope`] over the canonical string
/// `"affidavit-paid-delivery/v1|<subject>|<payload_hash_hex>"`, signs it with
/// `signing` (ES256, RFC 6979), then mints the signed standing receipt
/// through the existing [`VerificationEngine::certify_signed`] path over a
/// fresh engine holding that key under the graph-default policy.
///
/// Refuses (as [`VerifyRefusal`]): an empty `subject`
/// ([`VerifyRefusal::SubjectMismatch`], before any key material or nonce is
/// consumed), and any downstream engine refusal (subject mismatch, expired,
/// replay, unknown key, provider error, ...).
///
/// The returned receipt certifies WHO signed WHAT BYTES under WHICH key —
/// evidence only; it never decides authorization or honesty.
pub fn certify_paid_delivery_payload(
    payload_hash_hex: &str,
    subject: &str,
    signing: &Es256SigningKey,
) -> Result<CertifiedReceiptEnvelope, VerifyRefusal> {
    if subject.is_empty() {
        return Err(VerifyRefusal::SubjectMismatch(
            "empty subject refused: a certified receipt must name a subject".to_string(),
        ));
    }
    // AG4 whitespace law: a subject that is only whitespace — or that carries
    // leading/trailing whitespace, which round-trips ambiguously through
    // display/transport — names nobody and refuses like the empty subject.
    if subject.trim() != subject {
        return Err(VerifyRefusal::SubjectMismatch(format!(
            "subject has leading/trailing whitespace (len {}): refused",
            subject.len()
        )));
    }

    let canonical_subject = build_canonical_subject(payload_hash_hex, subject);
    let subject_digest = digest(DOMAIN_TAG, &[canonical_subject.as_bytes()]);

    let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    let key_id = KeyId::from_fingerprint(&fingerprint);

    // Fresh per-presentation nonce from the OS entropy source (crypto-trust
    // enables rand_core/getrandom). Reuse would be a lawful replay refusal on
    // any engine sharing replay state.
    let mut nonce = [0u8; 16];
    use rand_core::RngCore;
    rand_core::OsRng.fill_bytes(&mut nonce);

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let envelope = SignatureEnvelope {
        version: ENVELOPE_VERSION.to_string(),
        algorithm: AlgorithmId::Es256,
        key_id: key_id.clone(),
        profile: CryptoProfile::Classical,
        policy_epoch: 1,
        revocation_epoch: 0,
        generation: 1,
        nonce,
        not_before: now.saturating_sub(60),
        expires_at: now + VALIDITY_WINDOW_SECONDS,
        subject_digest,
        audience: CERTIFIED_RECEIPT_AUDIENCE.to_string(),
    };

    let signature = signing.sign(&envelope.signing_input_checked()?);

    // Fresh engine per certification: the subject/signature law runs on real
    // trust state (registry + policy + journal), never a bypass.
    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(KeyRecord {
            id: key_id,
            algorithm: AlgorithmId::Es256,
            fingerprint,
            custodian: CustodianIdentity {
                subject: subject.to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key: public,
            created_epoch: now.saturating_sub(60),
        })
        .map_err(|err| VerifyRefusal::Provider(err.to_string()))?;

    let engine = VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(now),
    );

    let receipt = engine.certify_signed(&envelope, &signature, &canonical_subject, signing)?;

    Ok(CertifiedReceiptEnvelope {
        envelope,
        receipt,
        signature_hex: hex_encode(&signature),
        verifying_key_sec1_hex: hex_encode(&signing.public_key_sec1()),
    })
}

/// The certified subject string:
/// `"affidavit-paid-delivery/v1|<subject>|<payload_hash_hex>"`. The payload
/// hash rides INSIDE the subject binding, so a tampered payload hash breaks
/// both the subject-binding law and the signature.
pub fn build_canonical_subject(payload_hash_hex: &str, subject: &str) -> String {
    format!("{PAID_DELIVERY_DOMAIN}|{subject}|{payload_hash_hex}")
}

/// Offline verification companion: re-derive the canonical subject from the
/// CLAIMED `payload_hash_hex`/`subject`, check the subject binding, and
/// re-adjudicate the envelope+signature against a fresh engine holding the
/// carried verifying key. A tampered hash refuses with
/// [`VerifyRefusal::SubjectMismatch`]; a flipped signature byte refuses as a
/// decided invalid.
pub fn verify_certified_paid_delivery(
    certified: &CertifiedReceiptEnvelope,
    payload_hash_hex: &str,
    subject: &str,
) -> Result<(), VerifyRefusal> {
    let canonical_subject = build_canonical_subject(payload_hash_hex, subject);
    if digest(DOMAIN_TAG, &[canonical_subject.as_bytes()]) != certified.envelope.subject_digest {
        return Err(VerifyRefusal::SubjectMismatch(
            "payload hash / subject does not match the certified binding".to_string(),
        ));
    }

    // AG4 RECEIPT-ENVELOPE LINKAGE LAW: the carried receipt is re-audited
    // (hash + signature) and must bind THIS envelope — a valid receipt
    // minted over a DIFFERENT envelope must not ride along in the swap.
    let envelope_commitment = blake3::hash(
        certified
            .envelope
            .signing_input_checked()
            .map_err(|err| VerifyRefusal::Provider(format!("envelope canonicalization: {err}")))?
            .as_slice(),
    )
    .to_hex()
    .to_string();
    if certified.receipt.envelope_commitment != envelope_commitment
        || certified.receipt.subject != canonical_subject
        || certified.receipt.key_id != certified.envelope.key_id.to_string()
        || certified.receipt.algorithm != certified.envelope.algorithm.as_str()
    {
        return Err(VerifyRefusal::Provider(
            "receipt-envelope linkage mismatch: receipt does not bind this envelope".to_string(),
        ));
    }
    certified
        .receipt
        .verify()
        .map_err(|err| VerifyRefusal::Provider(format!("receipt re-audit: {err}")))?;

    let public_bytes = decode_hex(&certified.verifying_key_sec1_hex)
        .ok_or_else(|| VerifyRefusal::Provider("verifying key is not valid hex".to_string()))?;
    let public = PublicKeyMaterial::Es256Sec1(public_bytes);
    let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
    let mut registry = InMemoryKeyRegistry::new();
    registry
        .register(KeyRecord {
            id: certified.envelope.key_id.clone(),
            algorithm: AlgorithmId::Es256,
            fingerprint,
            custodian: CustodianIdentity {
                subject: subject.to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key: public,
            created_epoch: certified.envelope.not_before,
        })
        .map_err(|e| VerifyRefusal::Provider(e.to_string()))?;
    // Adjudicate with the clock inside the envelope's own validity window;
    // the engine never guesses time, and the window was minted by the seam.
    let now = certified.envelope.not_before + 61;
    let engine = VerificationEngine::new(
        registry,
        RevocationList::default(),
        NonceJournal::default(),
        TrustPolicy::from_graph_defaults().with_now(now),
    );
    let signature = decode_hex(&certified.signature_hex)
        .ok_or_else(|| VerifyRefusal::Provider("signature is not valid hex".to_string()))?;
    let verdict = engine.verify_envelope(&certified.envelope, &signature)?;
    if verdict.standing != CryptographicStanding::Valid {
        return Err(VerifyRefusal::InvalidSignature(certified.envelope.key_id.0.clone()));
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).expect("hex digit"));
        s.push(char::from_digit((b & 0x0f) as u32, 16).expect("hex digit"));
    }
    s
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}
