//! Sealed receipts: cryptographic trust plane envelope bound to an affidavit Receipt (PQ-SEAL-v1 shape conservation).
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//! Consumed query columns (seal.rq): digest_alg, envelope_version, domain_tag.
//!
//! Shape conservation: the retired mock seal module pinned the `PQ-SEAL-v1`
//! wire name; the name survives and the crypto behind it is now real. A
//! [`SealedReceipt`] couples two independent bindings over one provenance
//! object:
//!
//! 1. **The base receipt's own law is untouched.** `affidavit`'s `Receipt` is
//!    a BLAKE3 hash chain whose `Deserialize` recomputes the chain hash, so a
//!    tampered base cannot even become a `Receipt` value — the derived
//!    `Deserialize` on [`SealedReceipt`] inherits that door-slam for free.
//! 2. **The seal binds WHO attests it.** The envelope's `subject_digest` must
//!    equal the domain-separated BLAKE3 digest of the receipt's content
//!    address ([`subject_digest_of`]); [`seal_receipt`] refuses a mismatch and
//!    [`verify_sealed`] re-checks the binding before spending any verifier
//!    effort.
//!
//! House law: certify-don't-decide. A [`CryptographicVerdict`] with standing
//! `Valid` is evidence that a named key signed these envelope bytes — never a
//! statement that the attested act is authorized. Refusals are typed values.

use crate::chain::content_address;
use crate::crypto_trust_canonical::digest;
use crate::crypto_trust_envelope::SignatureEnvelope;
use crate::crypto_trust_verify::{CryptographicVerdict, VerificationEngine, VerifyRefusal};
use crate::types::Receipt;

/// Wire format name (conservation: pinned by the retired mock seal module —
/// the shape survives, the crypto is now real).
pub const SEALED_RECEIPT_FORMAT: &str = "PQ-SEAL-v1";

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag). Mixed into the
/// subject digest so a receipt attested in one domain cannot be replayed as a
/// seal in another.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Signed-envelope version law (ctp:policy-v1 ctp:envelopeVersion). The seal
/// carries the envelope as-is; version admission remains the envelope module's
/// law (`from_bytes`).
pub const ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";

/// Digest algorithm over the domain-separated subject pre-image
/// (ctp:canon-JCS ctp:digestAlgorithm).
pub const DIGEST_ALGORITHM: &str = "BLAKE3";

/// A provenance receipt bound to the signature envelope that attests it.
///
/// Serde law of the composite: deriving `Deserialize` composes the base
/// receipt's chain-recomputing deserializer with the envelope's structural
/// deserializer, so tampered base bytes fail deserialization before any seal
/// check can run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SealedReceipt {
    /// The chain-sealed provenance receipt (content-addressed; never
    /// constructible outside `chain::ChainAssembler::finalize`).
    pub base: Receipt,
    /// The RFC-SA2A-007-errata signature envelope attesting the base.
    pub envelope: SignatureEnvelope,
    /// The envelope signature over `envelope.signing_input()`.
    pub signature: Vec<u8>,
}

/// Typed refusal of a seal operation; refusal-as-value, never a panic.
#[derive(Debug, thiserror::Error)]
pub enum SealError {
    /// The envelope's `subject_digest` does not bind the receipt it was
    /// offered with — the seal would attest a different object.
    #[error("envelope subject digest does not bind this receipt")]
    SubjectMismatch,
    /// The base receipt's content address could not be computed.
    #[error("chain content address: {0}")]
    Chain(String),
    /// Envelope verification refused adjudication (unknown key, expired
    /// window, replay, policy refusal — the carried payload is the typed
    /// verifier refusal's text).
    #[error("verification: {0}")]
    Verification(String),
    /// Seal-wire (de)serialization failed; on the deserialize side this is
    /// where a tampered base surfaces (the chain law refuses inside
    /// `Receipt`'s deserializer).
    #[error("serialization: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for SealError {
    fn from(err: serde_json::Error) -> Self {
        SealError::Serialization(err.to_string())
    }
}

/// The domain-separated subject digest binding `receipt`:
/// `digest(DOMAIN_TAG, [content_address(receipt).as_hex()])`. Deterministic:
/// the same receipt bytes always yield the same subject digest, and a receipt
/// differing by a single event byte diverges.
///
/// The digest is over the receipt's canonical content address — itself a
/// BLAKE3 of the receipt's sorted-key canonical JSON — under the trust-plane
/// domain tag; [`DIGEST_ALGORITHM`] names the algorithm of this outer digest.
pub fn subject_digest_of(receipt: &Receipt) -> Result<[u8; 32], SealError> {
    let address = content_address(receipt).map_err(|err| SealError::Chain(err.to_string()))?;
    Ok(digest(DOMAIN_TAG, &[address.as_hex().as_bytes()]))
}

/// Seal a provenance receipt: attach `envelope` + `signature` as the
/// attestation of `receipt`. Refuses
/// [`SealError::SubjectMismatch`] when `envelope.subject_digest` does not
/// equal [`subject_digest_of`]`(receipt)` — the seal must bind exactly the
/// object it is offered with, never a near-miss.
///
/// The signature's own validity is NOT checked here: sealing records an
/// attestation, [`verify_sealed`] adjudicates it. This keeps the sealing path
/// total over well-bound inputs and leaves every cryptographic decision to the
/// verification engine.
pub fn seal_receipt(
    receipt: &Receipt,
    envelope: SignatureEnvelope,
    signature: Vec<u8>,
) -> Result<SealedReceipt, SealError> {
    if envelope.subject_digest != subject_digest_of(receipt)? {
        return Err(SealError::SubjectMismatch);
    }
    Ok(SealedReceipt {
        base: receipt.clone(),
        envelope,
        signature,
    })
}

/// Adjudicate a sealed receipt: re-derive the subject digest from
/// `sealed.base` (never trust the envelope's claim alone), refuse
/// [`SealError::SubjectMismatch`] on a broken binding, then run the
/// verification engine over the envelope. The engine's typed refusals
/// (unknown key, expired window, replay, stale epoch, policy) map to
/// [`SealError::Verification`]; a decided signature failure is NOT a refusal —
/// it returns `Ok` with [`crate::crypto_trust_verify::CryptographicStanding::Invalid`].
pub fn verify_sealed(
    sealed: &SealedReceipt,
    engine: &VerificationEngine,
) -> Result<CryptographicVerdict, SealError> {
    if sealed.envelope.subject_digest != subject_digest_of(&sealed.base)? {
        return Err(SealError::SubjectMismatch);
    }
    engine
        .verify_envelope(&sealed.envelope, &sealed.signature)
        .map_err(|err: VerifyRefusal| SealError::Verification(err.to_string()))
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::chain::ChainAssembler;
    use crate::crypto_trust_envelope::NonceJournal;
    use crate::crypto_trust_es256::Es256SigningKey;
    use crate::crypto_trust_keys::{
        fingerprint_public_key, AlgorithmId, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry,
        KeyId, KeyOrigin, KeyRecord, KeyRegistry, PublicKeyMaterial,
    };
    use crate::crypto_trust_lifecycle::RevocationList;
    use crate::crypto_trust_verify::{CryptographicStanding, TrustPolicy};
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::verifier::verify;

    /// A real one-event receipt from the canonical assembler — never a
    /// hand-built struct (external construction is unconstructable anyway).
    fn real_receipt(payload: &[u8]) -> Receipt {
        let mut assembler = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "attest.op",
            vec![object_ref("receipt-under-test", "artifact")],
            payload,
            &mut counter,
        )
        .expect("event is well-formed");
        assembler.append(event).expect("append admitted");
        assembler.finalize()
    }

    /// A real ES256 signing key and its registry record (the same fixture
    /// shape the verification module's court uses).
    fn es256_fixture(tag: u8) -> (Es256SigningKey, KeyRecord) {
        let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid fixture scalar");
        let public = PublicKeyMaterial::Es256Sec1(signing.public_key_sec1());
        let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &public);
        let record = KeyRecord {
            id: KeyId::from_fingerprint(&fingerprint),
            algorithm: AlgorithmId::Es256,
            fingerprint,
            custodian: CustodianIdentity {
                subject: "subject-a".to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key: public,
            created_epoch: 1_700_000_000,
        };
        (signing, record)
    }

    /// An in-window ES256 envelope bound to `subject` (all lawfully-formed
    /// fields; every integer stays inside JCS's 2^53 so the pre-image is the
    /// real domain-separated document, never the empty fallback). The
    /// signature is applied by the caller over its pre-image.
    fn envelope_bound_to(
        subject: [u8; 32],
        nonce_tag: u8,
        kid: crate::crypto_trust_keys::KeyId,
    ) -> SignatureEnvelope {
        SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::Es256,
            key_id: kid,
            profile: CryptoProfile::Classical,
            policy_epoch: 1,
            revocation_epoch: 0,
            generation: 1,
            nonce: [nonce_tag; 16],
            not_before: 0,
            expires_at: 4_102_444_800, // 2100-01-01T00:00:00Z, JCS-safe
            subject_digest: subject,
            audience: "affidavit.seal".to_string(),
        }
    }

    /// The engine holding exactly the fixture key: window live, nothing
    /// revoked, fresh journal, graph-default policy at the court instant.
    fn engine_with(record: &KeyRecord) -> VerificationEngine {
        let mut registry = InMemoryKeyRegistry::new();
        registry
            .register(record.clone())
            .expect("register fixture key");
        VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(1_700_000_500),
        )
    }

    /// Seal `receipt` under a real key: correct subject digest, signed
    /// envelope pre-image.
    fn seal_real(
        receipt: &Receipt,
        signing: &Es256SigningKey,
        kid: crate::crypto_trust_keys::KeyId,
    ) -> Result<SealedReceipt, SealError> {
        let subject = subject_digest_of(receipt)?;
        let envelope = envelope_bound_to(subject, 0x5E, kid);
        let signature = signing.sign(&envelope.signing_input());
        seal_receipt(receipt, envelope, signature)
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(SEALED_RECEIPT_FORMAT, "PQ-SEAL-v1");
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
        assert_eq!(DIGEST_ALGORITHM, "BLAKE3");
    }

    #[test]
    fn subject_digest_is_deterministic_and_diverges() {
        let a = real_receipt(b"payload-a");
        let a2 = real_receipt(b"payload-a");
        let b = real_receipt(b"payload-b");
        let da = subject_digest_of(&a).expect("digest a");
        assert_eq!(da, subject_digest_of(&a2).expect("digest a2"));
        assert_ne!(da, subject_digest_of(&b).expect("digest b"));
        // Domain separation: the same content address under a different
        // domain tag diverges from the seal's subject digest.
        let address = content_address(&a).expect("content address");
        assert_ne!(da, digest("other.domain", &[address.as_hex().as_bytes()]));
    }

    #[test]
    fn seal_and_verify_round_trips_to_valid() {
        // Teeth first: the base receipt itself must certify before the seal
        // is allowed to attest anything.
        let receipt = real_receipt(b"round-trip");
        assert!(verify(&receipt).accepted, "base receipt must certify");

        let (signing, record) = es256_fixture(1);
        let sealed = seal_real(&receipt, &signing, record.id.clone()).expect("lawful seal");

        assert_eq!(SEALED_RECEIPT_FORMAT, "PQ-SEAL-v1");
        assert_eq!(
            sealed.envelope.subject_digest,
            subject_digest_of(&receipt).expect("subject digest")
        );

        let engine = engine_with(&record);
        let verdict = verify_sealed(&sealed, &engine).expect("verify sealed");
        assert_eq!(verdict.standing, CryptographicStanding::Valid);
        assert_eq!(verdict.subject_digest, sealed.envelope.subject_digest);
    }

    #[test]
    fn wrong_subject_digest_refused_by_variant() {
        let receipt = real_receipt(b"binding-test");
        let other = real_receipt(b"a-different-receipt");
        let (signing, record) = es256_fixture(2);
        let engine = engine_with(&record);

        // Envelope bound to `other`, offered with `receipt`: refused at seal.
        let subject_other = subject_digest_of(&other).expect("digest other");
        let envelope = envelope_bound_to(subject_other, 0x5F, record.id.clone());
        let signature = signing.sign(&envelope.signing_input());
        match seal_receipt(&receipt, envelope, signature) {
            Err(SealError::SubjectMismatch) => {}
            other => panic!("expected SubjectMismatch, got {other:?}"),
        }

        // The same mismatch cannot slip through verify_sealed either: a
        // sealed value whose envelope binds `other` re-pointed at a
        // different base is refused before any verifier effort is spent.
        let sealed_other =
            seal_real(&other, &signing, record.id.clone()).expect("lawful seal of other");
        let rebound = SealedReceipt {
            base: receipt,
            envelope: sealed_other.envelope,
            signature: sealed_other.signature,
        };
        match verify_sealed(&rebound, &engine) {
            Err(SealError::SubjectMismatch) => {}
            other => panic!("expected SubjectMismatch from verify_sealed, got {other:?}"),
        }
    }

    #[test]
    fn tampered_base_bytes_fail_deserialization() {
        let receipt = real_receipt(b"tamper-base");
        let (signing, record) = es256_fixture(3);
        let sealed = seal_real(&receipt, &signing, record.id.clone()).expect("lawful seal");

        let json = serde_json::to_string(&sealed).expect("sealed serializes");
        // Flip the last character of the base's first payload commitment in
        // the wire form; the seal envelope and signature are untouched.
        let marker = sealed.base.events[0]
            .payload_commitment
            .as_hex()
            .to_string();
        let flipped = format!(
            "{}{}",
            &marker[..marker.len() - 1],
            if marker.ends_with('0') { "1" } else { "0" }
        );
        let tampered = json.replace(&marker, &flipped);
        assert_ne!(json, tampered, "tamper must change the wire bytes");

        // The base receipt's own law fires: chain recomputation refuses, so
        // the composite cannot deserialize at all — mapped into the typed
        // Serialization variant by the seal's From impl.
        let outcome: Result<SealedReceipt, _> = serde_json::from_str(&tampered);
        let err = outcome.expect_err("tampered base must refuse to deserialize");
        let mapped: SealError = err.into();
        assert!(matches!(mapped, SealError::Serialization(_)));
        // And the honest wire form still loads.
        let honest: Result<SealedReceipt, _> = serde_json::from_str(&json);
        assert!(honest.is_ok(), "untampered wire form must deserialize");
    }

    #[test]
    fn tampered_signature_is_a_decided_invalid_verdict() {
        let receipt = real_receipt(b"tamper-sig");
        let (signing, record) = es256_fixture(4);
        let mut sealed = seal_real(&receipt, &signing, record.id.clone()).expect("lawful seal");
        assert!(!sealed.signature.is_empty());
        let last = sealed.signature.len() - 1;
        sealed.signature[last] ^= 0x01;

        let engine = engine_with(&record);
        // A failed signature is a DECIDED negative: Ok verdict, standing
        // Invalid — a refusal would mean "could not adjudicate".
        let verdict = verify_sealed(&sealed, &engine).expect("adjudication happens");
        assert_eq!(verdict.standing, CryptographicStanding::Invalid);
    }

    #[test]
    fn serde_round_trip_preserves_the_seal() {
        let receipt = real_receipt(b"round-trip-wire");
        let (signing, record) = es256_fixture(5);
        let sealed = seal_real(&receipt, &signing, record.id.clone()).expect("lawful seal");
        let bytes = serde_json::to_vec(&sealed).expect("serializes");
        let back: SealedReceipt =
            serde_json::from_slice(&bytes).expect("deserializes (chain re-verified)");
        assert_eq!(back.base, sealed.base);
        assert_eq!(back.envelope, sealed.envelope);
        assert_eq!(back.signature, sealed.signature);
    }
}
