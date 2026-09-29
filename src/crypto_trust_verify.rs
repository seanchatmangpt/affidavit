//! Verification engine: envelope + key state -> cryptographic standing for the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//! Consumed query columns (verify.rq): min_profile, default_profile, staleness,
//! envelope_version, domain_tag, standings.
//!
//! AUTHORITY BOUNDARY (certify-don't-decide): this engine answers exactly one
//! question — WHO signed WHAT BYTES under WHICH key state. A
//! [`CryptographicStanding::Valid`] verdict is a fact about bytes, keys, and
//! revocation state; it is NEVER a statement that the signed act is
//! authorized. Authorization belongs to the SA2A admission layer, which
//! consumes [`CryptoStandingReceipt`]s as evidence. This module performs no
//! actuation and grants no authority; every refusal is a typed value
//! ([`VerifyRefusal`]), never a panic, and a decided-negative signature check
//! is a verdict ([`CryptographicStanding::Invalid`]), not a refusal —
//! refusals are reserved for states that PREVENT adjudication.

use std::cell::RefCell;

use crate::crypto_trust_canonical::jcs;
use crate::crypto_trust_envelope::{EnvelopeError, NonceJournal, SignatureEnvelope};
use crate::crypto_trust_es256::verify_es256;
use crate::crypto_trust_keys::{
    AlgorithmId, CryptoProfile, InMemoryKeyRegistry, KeyId, KeyRecord, KeyRegistry,
    PublicKeyMaterial, RegistryError,
};
use crate::crypto_trust_lifecycle::{LifecycleRefusal, RevocationList, NONCE_WINDOW_SECONDS};
use crate::crypto_trust_pqc::{
    hybrid_verify, ml_dsa65_verify, slh_dsa128s_verify, HybridSignature,
};

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag).
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";
/// Minimum assurance profile floor (ctp:policy-v1 ctp:minProfile).
pub const MIN_PROFILE: &str = "CLASSICAL";
/// Default assurance profile for newly sealed envelopes (ctp:policy-v1 ctp:defaultProfile).
pub const DEFAULT_PROFILE: &str = "HYBRID";
/// Maximum seconds a signature's revocation epoch may lag the current epoch
/// (ctp:policy-v1 ctp:maxRevocationStalenessSeconds).
pub const MAX_REVOCATION_STALENESS_SECONDS: u64 = 300;
/// Signed-envelope version law (ctp:policy-v1 ctp:envelopeVersion).
pub const ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";
/// Stable profile tag for cryptographic standing receipts.
pub const CRYPTO_STANDING_PROFILE: &str = "affidavit/crypto-standing/v1";

/// Closed result vocabulary of envelope verification, rendered from the
/// `ctp:CryptographicStanding` individuals in graph order
/// (`ctp:standingOrder` 0..7). The serde wire form is SCREAMING_SNAKE_CASE
/// and equals `ctp:standingName`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CryptographicStanding {
    /// VALID — graph order 0.
    Valid,
    /// INVALID — graph order 1.
    Invalid,
    /// EXPIRED — graph order 2.
    Expired,
    /// REVOKED — graph order 3.
    Revoked,
    /// REPLAY_REJECTED — graph order 4.
    ReplayRejected,
    /// UNKNOWN_KEY — graph order 5.
    UnknownKey,
    /// PROFILE_REFUSED — graph order 6.
    ProfileRefused,
    /// MALFORMED — graph order 7.
    Malformed,
}

impl CryptographicStanding {
    /// SCREAMING_SNAKE_CASE standing value as declared in the graph
    /// (`ctp:standingName`); identical to the serde wire form.
    pub fn as_str(self) -> &'static str {
        match self {
            CryptographicStanding::Valid => "VALID",
            CryptographicStanding::Invalid => "INVALID",
            CryptographicStanding::Expired => "EXPIRED",
            CryptographicStanding::Revoked => "REVOKED",
            CryptographicStanding::ReplayRejected => "REPLAY_REJECTED",
            CryptographicStanding::UnknownKey => "UNKNOWN_KEY",
            CryptographicStanding::ProfileRefused => "PROFILE_REFUSED",
            CryptographicStanding::Malformed => "MALFORMED",
        }
    }

    /// Every standing in graph order (`ctp:standingOrder` ascending).
    pub fn all() -> &'static [CryptographicStanding] {
        &[
            CryptographicStanding::Valid,
            CryptographicStanding::Invalid,
            CryptographicStanding::Expired,
            CryptographicStanding::Revoked,
            CryptographicStanding::ReplayRejected,
            CryptographicStanding::UnknownKey,
            CryptographicStanding::ProfileRefused,
            CryptographicStanding::Malformed,
        ]
    }
}

/// The policy a verifier enforces: the assurance floor, the admitted
/// algorithm set, the revocation-staleness bound, and the verifier's clock.
/// Policy is data; the caller owns time.
#[derive(Debug, Clone)]
pub struct TrustPolicy {
    /// Signatures under a profile below this floor are refused
    /// ([`VerifyRefusal::ProfileFloor`]).
    pub min_profile: CryptoProfile,
    /// Algorithms whose envelopes are adjudicated at all; anything else is
    /// refused ([`VerifyRefusal::ProfileRefused`]).
    pub allowed: std::collections::BTreeSet<AlgorithmId>,
    /// Revocation-staleness bound in seconds (graph: 300).
    pub max_revocation_staleness_seconds: u64,
    /// The verifier's clock, in seconds since the trust-plane epoch. Set per
    /// verification batch; the engine never reads wall-clock time.
    pub now: u64,
}

impl TrustPolicy {
    /// Policy rendered from the graph defaults: floor `CLASSICAL`,
    /// every graph-admitted algorithm allowed, staleness bound `300`s,
    /// clock at 0 — set [`TrustPolicy::now`] (or use [`TrustPolicy::with_now`])
    /// before verifying; the engine never guesses time.
    pub fn from_graph_defaults() -> Self {
        TrustPolicy {
            min_profile: CryptoProfile::Classical,
            allowed: AlgorithmId::all().iter().copied().collect(),
            max_revocation_staleness_seconds: MAX_REVOCATION_STALENESS_SECONDS,
            now: 0,
        }
    }

    /// The policy with the verifier clock moved to `now`.
    pub fn with_now(mut self, now: u64) -> Self {
        self.now = now;
        self
    }
}

/// Typed refusal of the verification engine. Refusals are data: every variant
/// names the state that PREVENTED adjudication (as opposed to a decided
/// negative, which is [`CryptographicStanding::Invalid`]).
#[derive(Debug, thiserror::Error)]
pub enum VerifyRefusal {
    /// The envelope could not be parsed, or its signed bytes cannot be
    /// reconstructed (non-canonicalizable document).
    #[error("malformed envelope: {0}")]
    MalformedEnvelope(String),
    /// The envelope names a key absent from the registry, the registered
    /// key's algorithm disagrees with the envelope, or the key material on
    /// record does not serve the envelope's algorithm.
    #[error("unknown key {0}")]
    UnknownKey(String),
    /// The key carries a revocation record and the envelope proves awareness
    /// of the current revocation epoch — the revocation governs regardless.
    #[error("key {0} is revoked")]
    KeyRevoked(String),
    /// The signature names a revocation epoch older than the current one and
    /// the key's revocation is past the staleness grace.
    #[error("signature under stale revocation epoch for {0}")]
    StaleRevocationEpoch(String),
    /// The envelope's validity window ended before the verifier's clock.
    #[error("envelope expired at {0}")]
    Expired(u64),
    /// The envelope's validity window starts after the verifier's clock.
    #[error("envelope not yet valid at {0}")]
    NotYetValid(u64),
    /// The (kid, nonce) pair was already journaled inside the replay window.
    #[error("nonce replay for key {0}")]
    ReplayRejected(String),
    /// The envelope's algorithm is outside the policy's allowed set.
    #[error("algorithm {0:?} refused by policy")]
    ProfileRefused(crate::crypto_trust_keys::AlgorithmId),
    /// The envelope's declared assurance profile is below the policy floor.
    #[error("profile violation: {0:?} below policy floor")]
    ProfileFloor(CryptoProfile),
    /// A verification provider refused its inputs (malformed key/signature
    /// encodings) while adjudicating; the signature is NOT decided.
    #[error("provider error: {0}")]
    Provider(String),
    /// Certification was asked to mint over a presentation whose signature
    /// already DECIDED as [`CryptographicStanding::Invalid`] — a decided
    /// negative never becomes a receipt.
    #[error("signature did not verify: decided invalid for key {0}")]
    InvalidSignature(String),
}

impl From<EnvelopeError> for VerifyRefusal {
    fn from(err: EnvelopeError) -> Self {
        VerifyRefusal::MalformedEnvelope(err.to_string())
    }
}

/// A decided outcome of envelope verification: a standing over an exact key
/// and subject digest. Note the asymmetry with [`VerifyRefusal`]: a failed
/// signature check is a DECIDED negative
/// ([`CryptographicStanding::Invalid`]) returned `Ok`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CryptographicVerdict {
    /// The decided cryptographic standing (`Valid` or `Invalid` on this
    /// path; the full closed vocabulary is graph-owned and used plane-wide).
    pub standing: CryptographicStanding,
    /// The registry key the envelope named, when adjudication reached the
    /// registry.
    pub key_id: Option<KeyId>,
    /// The subject digest bound inside the signed bytes.
    pub subject_digest: [u8; 32],
}

/// The verification engine: key registry + revocation list + nonce journal +
/// policy. Certification only — it never actuates and never decides
/// authorization.
///
/// The nonce journal sits behind a `RefCell` because recording a sight of
/// (kid, nonce) is a state transition, while verification is a shared-ref
/// read of everything else: one engine verifies from `&self` and the journal
/// still advances. The engine is single-threaded by construction.
#[derive(Debug)]
pub struct VerificationEngine {
    registry: InMemoryKeyRegistry,
    revocations: RevocationList,
    nonces: RefCell<NonceJournal>,
    policy: TrustPolicy,
}

impl VerificationEngine {
    /// Assemble an engine over the given trust state.
    pub fn new(
        registry: InMemoryKeyRegistry,
        revocations: RevocationList,
        nonces: NonceJournal,
        policy: TrustPolicy,
    ) -> Self {
        VerificationEngine {
            registry,
            revocations,
            nonces: RefCell::new(nonces),
            policy,
        }
    }

    /// The key registry in force.
    pub fn registry(&self) -> &InMemoryKeyRegistry {
        &self.registry
    }

    /// The revocation list in force (doubles as the revocation-epoch clock).
    pub fn revocations(&self) -> &RevocationList {
        &self.revocations
    }

    /// The verifier clock in force.
    pub fn policy(&self) -> &TrustPolicy {
        &self.policy
    }

    /// The instant `(kid, nonce)` was last journaled, if present.
    pub fn nonce_seen_at(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> {
        self.nonces.borrow().seen(kid, nonce)
    }

    /// Admit a key record into the registry (refuses duplicates by id and by
    /// fingerprint).
    pub fn register_key(&mut self, record: KeyRecord) -> Result<(), RegistryError> {
        self.registry.register(record)
    }

    /// Revoke `kid` at `at` for `reason`; advances the revocation epoch to
    /// `max(revoked_at) + 1` (the latest revocation governs).
    pub fn revoke_key(&mut self, kid: &str, at: u64, reason: String) {
        self.revocations.revoke(kid, at, reason);
    }

    /// Evict journal entries whose replay window closed against the
    /// verifier's clock; returns the number evicted.
    pub fn prune_nonces(&mut self) -> usize {
        self.nonces
            .borrow_mut()
            .prune(self.policy.now, NONCE_WINDOW_SECONDS)
    }

    /// THE LAW — verify one envelope against key state, in order:
    ///
    /// 0. the signed bytes must reconstruct
    ///    ([`VerifyRefusal::MalformedEnvelope`] otherwise);
    /// 1. window: `now` inside the envelope's validity window
    ///    ([`VerifyRefusal::NotYetValid`] / [`VerifyRefusal::Expired`]);
    /// 2. policy: algorithm allowed ([`VerifyRefusal::ProfileRefused`]);
    ///    declared profile at/above the floor ([`VerifyRefusal::ProfileFloor`]);
    /// 3. registry: key known AND its record agrees with the envelope's
    ///    claims ([`VerifyRefusal::UnknownKey`]);
    /// 4. revocation: signature epoch live within the staleness grace
    ///    ([`VerifyRefusal::StaleRevocationEpoch`]), then the flat revocation
    ///    law ([`VerifyRefusal::KeyRevoked`]);
    /// 5. replay: (kid, nonce) not already journaled inside the window
    ///    ([`VerifyRefusal::ReplayRejected`]);
    /// 6. signature: static per-algorithm verification over the envelope's
    ///    signed bytes;
    /// 7. verdict: [`CryptographicStanding::Valid`] or
    ///    [`CryptographicStanding::Invalid`] — a decided negative is `Ok`.
    pub fn verify_envelope(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
    ) -> Result<CryptographicVerdict, VerifyRefusal> {
        // (0) The signed bytes must reconstruct; a document that cannot
        // canonicalize cannot be adjudicated at all.
        let signing_input = env.signing_input_checked()?;

        // (1) Validity window (the envelope module owns the boundary law).
        env.window_live(self.policy.now).map_err(|err| match err {
            EnvelopeError::NotYetValid(nb) => VerifyRefusal::NotYetValid(nb),
            EnvelopeError::Expired(ex) => VerifyRefusal::Expired(ex),
            other => VerifyRefusal::MalformedEnvelope(other.to_string()),
        })?;

        // (2) Policy: algorithm admission, then assurance floor.
        if !self.policy.allowed.contains(&env.algorithm) {
            return Err(VerifyRefusal::ProfileRefused(env.algorithm));
        }
        if env.profile < self.policy.min_profile {
            return Err(VerifyRefusal::ProfileFloor(env.profile));
        }

        // (3) Registry: the key must be known, and the record's algorithm
        // must agree with the envelope's claim (a key does not change
        // algorithm between registration and use).
        let kid = env.key_id.to_string();
        let Some(record) = self.registry.lookup(&env.key_id) else {
            return Err(VerifyRefusal::UnknownKey(kid));
        };
        if record.algorithm != env.algorithm {
            return Err(VerifyRefusal::UnknownKey(kid));
        }

        // (4) Revocation: first the epoch-freshness law (a signature naming a
        // stale epoch past the grace window is refused with the revocation
        // evidence), then the flat revocation law (a revoked key refuses even
        // when the envelope proves awareness of the current epoch).
        if let Err(LifecycleRefusal::Revoked(ref revoked_kid, revoked_at, ref reason)) = self
            .revocations
            .signature_epoch_live(&kid, env.revocation_epoch, self.policy.now)
        {
            return Err(VerifyRefusal::StaleRevocationEpoch(format!(
                "{revoked_kid} revoked at {revoked_at}: {reason}"
            )));
        }
        if self.revocations.is_revoked(&kid) {
            return Err(VerifyRefusal::KeyRevoked(kid));
        }

        // (5) Replay defense: recording the sight of (kid, nonce) is the
        // admission; a repeat inside the window is refused as a replay.
        self.nonces
            .borrow_mut()
            .record(&kid, env.nonce, self.policy.now, NONCE_WINDOW_SECONDS)
            .map_err(|err| VerifyRefusal::ReplayRejected(format!("{kid}: {err}")))?;

        // (6)+(7) Signature adjudication: a failed check is a decided
        // INVALID verdict, not a refusal.
        let signature_holds =
            verify_signature_bytes(env.algorithm, record, &signing_input, signature)?;
        let standing = if signature_holds {
            CryptographicStanding::Valid
        } else {
            CryptographicStanding::Invalid
        };
        Ok(CryptographicVerdict {
            standing,
            key_id: Some(env.key_id.clone()),
            subject_digest: env.subject_digest,
        })
    }

    /// Certify a [`CryptoStandingReceipt`]: verify the envelope, then mint a
    /// sealed receipt ONLY on a [`CryptographicStanding::Valid`] verdict.
    /// Every other outcome refuses — verification refusals propagate, and a
    /// decided negative refuses as [`VerifyRefusal::InvalidSignature`]. The
    /// receipt is evidence for the SA2A authorization layer; it certifies the
    /// signature, never the act.
    pub fn certify(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
        subject: &str,
    ) -> Result<CryptoStandingReceipt, VerifyRefusal> {
        let verdict = self.verify_envelope(env, signature)?;
        if verdict.standing != CryptographicStanding::Valid {
            return Err(VerifyRefusal::InvalidSignature(env.key_id.to_string()));
        }
        let signing_input = env.signing_input_checked()?;
        let envelope_commitment = blake3::hash(&signing_input).to_hex().to_string();
        CryptoStandingReceipt::seal(
            subject,
            &envelope_commitment,
            &env.key_id.to_string(),
            env.algorithm.as_str(),
            verdict.standing,
            self.policy.now,
        )
        .map_err(|err| VerifyRefusal::Provider(err.to_string()))
    }
}

/// Static per-algorithm signature adjudication over the envelope's signed
/// bytes. Dispatch is on [`AlgorithmId`] (no trait objects); the registered
/// public-key material must match the envelope's algorithm family exactly —
/// a mismatch is [`VerifyRefusal::UnknownKey`], not a failed signature.
///
/// Hybrid wire form: a hybrid signature travels as the JSON serialization of
/// its two component signatures ([`HybridSignature`]); the bytes are parsed
/// here and BOTH halves must verify under their respective keys.
fn verify_signature_bytes(
    algorithm: AlgorithmId,
    record: &KeyRecord,
    signing_input: &[u8],
    signature: &[u8],
) -> Result<bool, VerifyRefusal> {
    match (&record.public_key, algorithm) {
        (PublicKeyMaterial::Es256Sec1(pk), AlgorithmId::Es256) => {
            verify_es256(pk, signing_input, signature)
                .map_err(|err| VerifyRefusal::Provider(err.to_string()))
        }
        (PublicKeyMaterial::MlDsa65(pk), AlgorithmId::MlDsa65) => {
            ml_dsa65_verify(pk, signing_input, signature)
                .map_err(|err| VerifyRefusal::Provider(err.to_string()))
        }
        (PublicKeyMaterial::SlhDsa128s(pk), AlgorithmId::SlhDsa128s) => {
            slh_dsa128s_verify(pk, signing_input, signature)
                .map_err(|err| VerifyRefusal::Provider(err.to_string()))
        }
        (
            PublicKeyMaterial::Hybrid {
                es256: es256_pk,
                mldsa65: mldsa65_pk,
            },
            AlgorithmId::HybridEs256MlDsa65,
        ) => {
            let hybrid: HybridSignature = serde_json::from_slice(signature).map_err(|err| {
                VerifyRefusal::Provider(format!("hybrid signature decode: {err}"))
            })?;
            hybrid_verify(es256_pk, mldsa65_pk, signing_input, &hybrid)
                .map_err(|err| VerifyRefusal::Provider(err.to_string()))
        }
        // The registered material and the envelope's algorithm disagree: the
        // verifier cannot adjudicate under a key it does not have.
        _ => Err(VerifyRefusal::UnknownKey(record.id.to_string())),
    }
}

/// A sealed cryptographic standing receipt: the receipt-facing projection of
/// a [`CryptographicStanding::Valid`] verdict. Certifies WHO signed WHAT
/// BYTES under WHICH key — evidence only; authorization is decided elsewhere.
///
/// The private `_seal` field prevents external struct-literal construction;
/// deserialization recomputes [`CryptoStandingReceipt::receipt_hash`] over
/// the canonical (JCS) form of every other field and refuses mismatches, so
/// a tampered receipt cannot load.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[allow(clippy::manual_non_exhaustive)]
pub struct CryptoStandingReceipt {
    /// Receipt profile. Always [`CRYPTO_STANDING_PROFILE`] when minted here.
    pub profile: String,
    /// The subject the signed act is attributed to (custodian claim).
    pub subject: String,
    /// Lowercase-hex BLAKE3 commitment over the envelope's signed bytes.
    pub envelope_commitment: String,
    /// The registry key that verified.
    pub key_id: String,
    /// The verifying algorithm's canonical name (e.g. `ES256`).
    pub algorithm: String,
    /// The decided cryptographic standing (VALID for minted receipts).
    pub standing: CryptographicStanding,
    /// Verifier-clock second the standing was decided at.
    pub observed_at: u64,
    /// Canonical BLAKE3 hash over every field above (JCS material).
    pub receipt_hash: String,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal of the standing-receipt boundary.
#[derive(Debug, thiserror::Error)]
pub enum StandingReceiptError {
    /// The stored hash does not match the recomputed canonical hash.
    #[error("receipt hash mismatch: claimed {claimed}, recomputed {recomputed}")]
    ReceiptHashMismatch { claimed: String, recomputed: String },
    /// Canonical serialization of the receipt material failed.
    #[error("serialization: {0}")]
    Serialization(String),
}

/// The hash material: every receipt field except `receipt_hash` itself.
#[derive(serde::Serialize)]
struct ReceiptMaterial<'a> {
    profile: &'a str,
    subject: &'a str,
    envelope_commitment: &'a str,
    key_id: &'a str,
    algorithm: &'a str,
    standing: CryptographicStanding,
    observed_at: u64,
}

impl CryptoStandingReceipt {
    /// Seal point: computes the canonical receipt hash and freezes the
    /// receipt. `pub(crate)` — only the certification path mints receipts.
    pub(crate) fn seal(
        subject: &str,
        envelope_commitment: &str,
        key_id: &str,
        algorithm: &str,
        standing: CryptographicStanding,
        observed_at: u64,
    ) -> Result<CryptoStandingReceipt, StandingReceiptError> {
        let material = ReceiptMaterial {
            profile: CRYPTO_STANDING_PROFILE,
            subject,
            envelope_commitment,
            key_id,
            algorithm,
            standing,
            observed_at,
        };
        let receipt_hash = receipt_material_hash(&material)?;
        Ok(CryptoStandingReceipt {
            profile: CRYPTO_STANDING_PROFILE.to_string(),
            subject: subject.to_string(),
            envelope_commitment: envelope_commitment.to_string(),
            key_id: key_id.to_string(),
            algorithm: algorithm.to_string(),
            standing,
            observed_at,
            receipt_hash,
            _seal: (),
        })
    }

    /// Recompute the canonical receipt hash and compare — the same law
    /// [`serde::Deserialize`] enforces; exposed for holders that re-audit a
    /// receipt in memory.
    pub fn verify(&self) -> Result<(), StandingReceiptError> {
        let material = ReceiptMaterial {
            profile: &self.profile,
            subject: &self.subject,
            envelope_commitment: &self.envelope_commitment,
            key_id: &self.key_id,
            algorithm: &self.algorithm,
            standing: self.standing,
            observed_at: self.observed_at,
        };
        let recomputed = receipt_material_hash(&material)?;
        if recomputed != self.receipt_hash {
            return Err(StandingReceiptError::ReceiptHashMismatch {
                claimed: self.receipt_hash.clone(),
                recomputed,
            });
        }
        Ok(())
    }
}

/// BLAKE3 over the JCS canonical form of the receipt material, lowercase hex.
fn receipt_material_hash(material: &ReceiptMaterial<'_>) -> Result<String, StandingReceiptError> {
    let value = serde_json::to_value(material)
        .map_err(|err| StandingReceiptError::Serialization(err.to_string()))?;
    let canonical =
        jcs(&value).map_err(|err| StandingReceiptError::Serialization(err.to_string()))?;
    Ok(blake3::hash(canonical.as_bytes()).to_hex().to_string())
}

impl<'de> serde::Deserialize<'de> for CryptoStandingReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(serde::Deserialize)]
        struct RawReceipt {
            profile: String,
            subject: String,
            envelope_commitment: String,
            key_id: String,
            algorithm: String,
            standing: CryptographicStanding,
            observed_at: u64,
            receipt_hash: String,
        }

        let raw = RawReceipt::deserialize(deserializer)?;
        let material = ReceiptMaterial {
            profile: &raw.profile,
            subject: &raw.subject,
            envelope_commitment: &raw.envelope_commitment,
            key_id: &raw.key_id,
            algorithm: &raw.algorithm,
            standing: raw.standing,
            observed_at: raw.observed_at,
        };
        let recomputed = receipt_material_hash(&material).map_err(D::Error::custom)?;
        if recomputed != raw.receipt_hash {
            return Err(D::Error::custom(
                StandingReceiptError::ReceiptHashMismatch {
                    claimed: raw.receipt_hash,
                    recomputed,
                },
            ));
        }
        Ok(CryptoStandingReceipt {
            profile: raw.profile,
            subject: raw.subject,
            envelope_commitment: raw.envelope_commitment,
            key_id: raw.key_id,
            algorithm: raw.algorithm,
            standing: raw.standing,
            observed_at: raw.observed_at,
            receipt_hash: raw.receipt_hash,
            _seal: (),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_es256::Es256SigningKey;
    use crate::crypto_trust_keys::{fingerprint_public_key, CustodianIdentity, KeyOrigin};

    /// The verifier clock used across the court.
    const NOW: u64 = 1_700_000_500;

    /// Builds a real ES256 signing key and its registry record.
    fn es256_fixture(tag: u8) -> (Es256SigningKey, KeyRecord) {
        let signing = Es256SigningKey::from_seed(&[tag; 32]).expect("valid scalar seed");
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
            created_epoch: NOW - 1_000,
        };
        (signing, record)
    }

    /// A fresh, in-window envelope naming `kid`.
    fn envelope(kid: &KeyId, nonce: [u8; 16], revocation_epoch: u64) -> SignatureEnvelope {
        SignatureEnvelope {
            version: crate::crypto_trust_envelope::ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::Es256,
            key_id: kid.clone(),
            profile: CryptoProfile::Classical,
            policy_epoch: 1,
            revocation_epoch,
            generation: 1,
            nonce,
            not_before: NOW - 1_000,
            expires_at: NOW + 1_000,
            subject_digest: [0x22; 32],
            audience: "affidavit.verifier".to_string(),
        }
    }

    /// An engine holding `record` under the graph-default policy, clock at NOW.
    fn engine_with(record: &KeyRecord) -> VerificationEngine {
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(NOW),
        )
    }

    // -- teeth -----------------------------------------------------------------
    //
    // The nonce journal is anti-replay STATE inside the engine: an
    // adjudication records the presentation's (kid, nonce). One presentation
    // is therefore adjudicated ONCE per engine — verify-then-certify of the
    // same envelope on one engine is lawfully a replay, and certify-then-
    // certify is refused. Fresh engines below prove the positive paths.

    #[test]
    fn valid_signature_verifies_and_certify_mints_a_sealed_receipt() {
        let (signing, record) = es256_fixture(1);
        let env = envelope(&record.id, [1u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));

        let verdict = engine_with(&record)
            .verify_envelope(&env, &signature)
            .expect("fresh signature must verify");
        assert_eq!(verdict.standing, CryptographicStanding::Valid);
        assert_eq!(verdict.subject_digest, env.subject_digest);
        assert_eq!(verdict.key_id, Some(record.id.clone()));

        let engine = engine_with(&record);
        let receipt = engine
            .certify(&env, &signature, "subject-a")
            .expect("VALID verdict must mint a receipt");
        assert_eq!(receipt.profile, CRYPTO_STANDING_PROFILE);
        assert_eq!(receipt.standing, CryptographicStanding::Valid);
        assert_eq!(receipt.key_id, record.id.to_string());
        assert_eq!(receipt.algorithm, "ES256");
        assert_eq!(receipt.subject, "subject-a");
        assert_eq!(receipt.observed_at, NOW);
        receipt.verify().expect("minted receipt must re-verify");

        // Double-mint guard: one presentation, one receipt.
        match engine.certify(&env, &signature, "subject-a") {
            Err(VerifyRefusal::ReplayRejected(_)) => {}
            other => {
                panic!("a second certify of one presentation must replay-refuse, got {other:?}")
            }
        }
    }

    #[test]
    fn certification_is_deterministic_across_engines() {
        let (signing, record) = es256_fixture(9);
        let env = envelope(&record.id, [9u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt_a = engine_with(&record)
            .certify(&env, &signature, "subject-a")
            .expect("mint A");
        let receipt_b = engine_with(&record)
            .certify(&env, &signature, "subject-a")
            .expect("mint B");
        assert_eq!(receipt_a.receipt_hash, receipt_b.receipt_hash);
        assert_eq!(receipt_a, receipt_b);
    }

    // -- every refusal witnessed by exact variant -------------------------------

    #[test]
    fn unknown_key_refuses_by_name() {
        let (_signing, record) = es256_fixture(2);
        // The engine holds only a DIFFERENT key; the envelope names the
        // fixture key, which the registry has never seen.
        let (_, other_record) = es256_fixture(3);
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(other_record).expect("register other");
        let engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(NOW),
        );
        let env = envelope(&record.id, [2u8; 16], 0);
        let signature = vec![0u8; 8]; // never reached: the key is unknown
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::UnknownKey(kid)) => assert_eq!(kid, record.id.to_string()),
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn record_algorithm_disagreement_refuses_unknown_key() {
        let (_signing, record) = es256_fixture(4);
        let engine = engine_with(&record);
        // Same kid, but the envelope claims ML-DSA-65 while the record is ES256.
        let mut env = envelope(&record.id, [4u8; 16], 0);
        env.algorithm = AlgorithmId::MlDsa65;
        env.profile = CryptoProfile::Pqc;
        match engine.verify_envelope(&env, b"irrelevant") {
            Err(VerifyRefusal::UnknownKey(kid)) => assert_eq!(kid, record.id.to_string()),
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn key_material_serving_another_family_refuses_unknown_key() {
        // A record whose material is ML-DSA-65 while it claims ES256: the
        // dispatch has no (material, algorithm) arm for that pair.
        let material = PublicKeyMaterial::MlDsa65(vec![7u8; 1952]);
        let fingerprint = fingerprint_public_key(AlgorithmId::Es256, &material);
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
            public_key: material,
            created_epoch: NOW - 1_000,
        };
        let engine = engine_with(&record);
        let env = envelope(&record.id, [5u8; 16], 0);
        match engine.verify_envelope(&env, b"irrelevant") {
            Err(VerifyRefusal::UnknownKey(kid)) => assert_eq!(kid, record.id.to_string()),
            other => panic!("expected UnknownKey, got {other:?}"),
        }
    }

    #[test]
    fn revoked_key_refuses_even_when_the_epoch_is_current() {
        let (signing, record) = es256_fixture(5);
        let kid = record.id.to_string();
        let mut engine = engine_with(&record);
        engine.revoke_key(&kid, NOW, "compromised".to_string());
        // The envelope names the CURRENT epoch (NOW + 1): it proves awareness
        // of the revocation — the flat revocation law governs anyway.
        let env = envelope(&record.id, [6u8; 16], engine.revocations().current_epoch());
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::KeyRevoked(refused)) => assert_eq!(refused, kid),
            other => panic!("expected KeyRevoked, got {other:?}"),
        }
    }

    #[test]
    fn stale_revocation_epoch_refuses_past_grace() {
        let (signing, record) = es256_fixture(6);
        let kid = record.id.to_string();
        let mut engine = engine_with(&record);
        // Revoked 301s ago: one second past the 300s staleness grace.
        engine.revoke_key(
            &kid,
            NOW - MAX_REVOCATION_STALENESS_SECONDS - 1,
            "compromised".to_string(),
        );
        let env = envelope(&record.id, [7u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::StaleRevocationEpoch(detail)) => {
                assert!(
                    detail.contains(&kid),
                    "evidence must name the kid: {detail}"
                );
                assert!(
                    detail.contains("compromised"),
                    "evidence must carry the reason: {detail}"
                );
            }
            other => panic!("expected StaleRevocationEpoch, got {other:?}"),
        }
    }

    #[test]
    fn revocation_at_the_grace_boundary_still_refuses_flat() {
        let (signing, record) = es256_fixture(16);
        let kid = record.id.to_string();
        let mut engine = engine_with(&record);
        // Exactly at the bound (300s): the epoch-freshness law still admits
        // (the revocation may not have propagated yet), so the flat
        // revocation law is what refuses — a revoked key is a revoked key.
        engine.revoke_key(
            &kid,
            NOW - MAX_REVOCATION_STALENESS_SECONDS,
            "compromised".to_string(),
        );
        let env = envelope(&record.id, [17u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::KeyRevoked(refused)) => assert_eq!(refused, kid),
            other => panic!("expected KeyRevoked, got {other:?}"),
        }
    }

    #[test]
    fn expired_envelope_refuses() {
        let (signing, record) = es256_fixture(7);
        let engine = engine_with(&record);
        let mut env = envelope(&record.id, [8u8; 16], 0);
        env.expires_at = NOW - 1;
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::Expired(at)) => assert_eq!(at, NOW - 1),
            other => panic!("expected Expired, got {other:?}"),
        }
    }

    #[test]
    fn not_yet_valid_envelope_refuses() {
        let (signing, record) = es256_fixture(8);
        let engine = engine_with(&record);
        let mut env = envelope(&record.id, [10u8; 16], 0);
        env.not_before = NOW + 1;
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::NotYetValid(nb)) => assert_eq!(nb, NOW + 1),
            other => panic!("expected NotYetValid, got {other:?}"),
        }
    }

    #[test]
    fn nonce_replay_refuses_the_second_presentation() {
        let (signing, record) = es256_fixture(10);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [11u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let first = engine
            .verify_envelope(&env, &signature)
            .expect("first is fresh");
        assert_eq!(first.standing, CryptographicStanding::Valid);
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::ReplayRejected(detail)) => {
                assert!(detail.contains(&record.id.to_string()));
            }
            other => panic!("expected ReplayRejected, got {other:?}"),
        }
        // The journal saw the nonce exactly once (the refused repeat did not
        // restamp it).
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
            Some(NOW)
        );
    }

    #[test]
    fn algorithm_outside_allowed_set_refuses() {
        let (signing, record) = es256_fixture(11);
        let mut policy = TrustPolicy::from_graph_defaults().with_now(NOW);
        policy.allowed = std::collections::BTreeSet::from([AlgorithmId::MlDsa65]);
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        let engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            policy,
        );
        let env = envelope(&record.id, [12u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::ProfileRefused(AlgorithmId::Es256)) => {}
            other => panic!("expected ProfileRefused(ES256), got {other:?}"),
        }
    }

    #[test]
    fn profile_below_floor_refuses() {
        let (signing, record) = es256_fixture(12);
        let mut policy = TrustPolicy::from_graph_defaults().with_now(NOW);
        policy.min_profile = CryptoProfile::Pqc; // ES256 is CLASSICAL: below the floor
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        let engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            policy,
        );
        let env = envelope(&record.id, [13u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::ProfileFloor(CryptoProfile::Classical)) => {}
            other => panic!("expected ProfileFloor(Classical), got {other:?}"),
        }
    }

    #[test]
    fn malformed_envelope_refuses_inside_the_law() {
        let (_signing, record) = es256_fixture(13);
        let engine = engine_with(&record);
        // policy_epoch beyond 2^53: the document cannot canonicalize (JCS
        // refuses non-I-JSON integers), so the signed bytes cannot be
        // reconstructed and nothing downstream can be adjudicated. No
        // signature is pre-computable for such a document; bytes are never
        // reached.
        let mut env = envelope(&record.id, [14u8; 16], 0);
        env.policy_epoch = 9_007_199_254_740_993;
        match engine.verify_envelope(&env, b"irrelevant") {
            Err(VerifyRefusal::MalformedEnvelope(_)) => {}
            other => panic!("expected MalformedEnvelope, got {other:?}"),
        }
    }

    #[test]
    fn corrupted_bytes_refuse_through_the_typed_from() {
        let (_signing, record) = es256_fixture(14);
        let env = envelope(&record.id, [15u8; 16], 0);
        let mut bytes = serde_json::to_vec(&env).expect("envelope serializes");
        bytes[10] ^= 0xFF; // corrupt the transport form
        match SignatureEnvelope::from_bytes(&bytes) {
            Err(err) => {
                let refusal: VerifyRefusal = err.into();
                assert!(matches!(refusal, VerifyRefusal::MalformedEnvelope(_)));
            }
            Ok(_) => panic!("corrupted bytes must not parse"),
        }
    }

    // -- decided negatives are verdicts, not refusals ----------------------------

    #[test]
    fn tampered_signature_is_a_decided_invalid_verdict() {
        let (signing, record) = es256_fixture(15);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [18u8; 16], 0);
        let mut signature =
            signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let last = signature.len() - 1;
        signature[last] ^= 0x01; // well-formed DER scalar flip: verifies false
        let verdict = engine
            .verify_envelope(&env, &signature)
            .expect("a tampered signature is DECIDED, not refused");
        assert_eq!(verdict.standing, CryptographicStanding::Invalid);
        assert_eq!(verdict.key_id, Some(record.id.clone()));
    }

    #[test]
    fn certify_refuses_a_decided_negative() {
        let (signing, record) = es256_fixture(17);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [19u8; 16], 0);
        let mut signature =
            signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let last = signature.len() - 1;
        signature[last] ^= 0x01;
        match engine.certify(&env, &signature, "subject-a") {
            Err(VerifyRefusal::InvalidSignature(kid)) => {
                assert_eq!(kid, record.id.to_string());
            }
            other => panic!("expected InvalidSignature, got {other:?}"),
        }
    }

    // -- the receipt seal law -----------------------------------------------------

    #[test]
    fn tampered_standing_refuses_to_deserialize() {
        let (signing, record) = es256_fixture(18);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [20u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine.certify(&env, &signature, "subject-a").expect("mint");
        let mut json = serde_json::to_value(&receipt).expect("receipt serializes");
        json["standing"] = serde_json::Value::String("INVALID".to_string());
        let outcome = serde_json::from_value::<CryptoStandingReceipt>(json);
        let err = outcome.expect_err("a flipped standing must not load");
        assert!(
            err.to_string()
                .starts_with("receipt hash mismatch: claimed "),
            "the refusal must be the typed ReceiptHashMismatch rendering: {err}"
        );
    }

    #[test]
    fn in_memory_tamper_is_caught_by_variant() {
        let (signing, record) = es256_fixture(19);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [21u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let mut receipt = engine.certify(&env, &signature, "subject-a").expect("mint");
        let honest_hash = receipt.receipt_hash.clone();
        receipt.subject = "tampered-subject".to_string();
        match receipt.verify() {
            Err(StandingReceiptError::ReceiptHashMismatch {
                claimed,
                recomputed,
            }) => {
                assert_eq!(claimed, honest_hash);
                assert_ne!(recomputed, honest_hash);
            }
            other => panic!("expected ReceiptHashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn receipt_round_trips_byte_identically() {
        let (signing, record) = es256_fixture(20);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [22u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine.certify(&env, &signature, "subject-a").expect("mint");
        let json = serde_json::to_string(&receipt).expect("receipt serializes");
        let back: CryptoStandingReceipt =
            serde_json::from_str(&json).expect("honest receipt loads");
        assert_eq!(back, receipt);
        assert_eq!(serde_json::to_string(&back).expect("re-serialize"), json);
        assert_eq!(back.standing, CryptographicStanding::Valid);
        assert_eq!(
            back.envelope_commitment.len(),
            64,
            "commitment is lowercase-hex BLAKE3"
        );
    }

    // -- graph-projection teeth ----------------------------------------------------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(MIN_PROFILE, "CLASSICAL");
        assert_eq!(DEFAULT_PROFILE, "HYBRID");
        assert_eq!(MAX_REVOCATION_STALENESS_SECONDS, 300);
        assert_eq!(ENVELOPE_VERSION, "CTP-ENVELOPE-v1");
        // Cross-module graph consistency: every lane rendered the same facts.
        assert_eq!(
            MAX_REVOCATION_STALENESS_SECONDS,
            crate::crypto_trust_lifecycle::MAX_REVOCATION_STALENESS_SECONDS
        );
        assert_eq!(
            crate::crypto_trust_envelope::ENVELOPE_VERSION,
            ENVELOPE_VERSION
        );
    }

    #[test]
    fn from_graph_defaults_matches_the_graph() {
        let policy = TrustPolicy::from_graph_defaults();
        assert_eq!(policy.min_profile, CryptoProfile::Classical);
        assert_eq!(policy.max_revocation_staleness_seconds, 300);
        assert_eq!(policy.now, 0);
        assert_eq!(policy.allowed.len(), AlgorithmId::all().len());
        for alg in AlgorithmId::all() {
            assert!(policy.allowed.contains(alg), "{alg:?} must be allowed");
        }
        let timed = TrustPolicy::from_graph_defaults().with_now(NOW);
        assert_eq!(timed.now, NOW);
    }

    #[test]
    fn standings_are_the_closed_graph_vocabulary_in_order() {
        const EXPECTED: [&str; 8] = [
            "VALID",
            "INVALID",
            "EXPIRED",
            "REVOKED",
            "REPLAY_REJECTED",
            "UNKNOWN_KEY",
            "PROFILE_REFUSED",
            "MALFORMED",
        ];
        let all = CryptographicStanding::all();
        assert_eq!(all.len(), EXPECTED.len());
        for (standing, expected) in all.iter().zip(EXPECTED.iter()) {
            assert_eq!(&standing.as_str(), expected);
        }
    }

    #[test]
    fn standing_serde_round_trips_screaming_snake_for_all_variants() {
        for standing in CryptographicStanding::all() {
            let json = serde_json::to_string(standing).expect("serialize standing");
            assert_eq!(json, format!("\"{}\"", standing.as_str()));
            let back: CryptographicStanding =
                serde_json::from_str(&json).expect("deserialize standing");
            assert_eq!(&back, standing);
        }
        // Spot-check the multi-word variant: the wire form is SCREAMING_SNAKE.
        assert_eq!(
            serde_json::to_string(&CryptographicStanding::ReplayRejected).expect("serialize"),
            "\"REPLAY_REJECTED\""
        );
    }

    // -- journal hygiene -------------------------------------------------------------

    #[test]
    fn prune_nonces_evicts_expired_entries_and_counts() {
        let (_signing, record) = es256_fixture(21);
        let mut journal = NonceJournal::default();
        // An entry that will be 400s old at the verifier clock (past the 300s
        // window), recorded directly before the engine takes the journal.
        journal
            .record(
                &record.id.to_string(),
                [1u8; 16],
                NOW - 400,
                NONCE_WINDOW_SECONDS,
            )
            .expect("first sight admitted");
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        let mut engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            journal,
            TrustPolicy::from_graph_defaults().with_now(NOW),
        );
        // A live verification journals a fresh nonce at NOW.
        let env = envelope(&record.id, [2u8; 16], 0);
        let signature = _signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        engine.verify_envelope(&env, &signature).expect("fresh");
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &[2u8; 16]),
            Some(NOW)
        );

        assert_eq!(engine.prune_nonces(), 1, "exactly the expired entry evicts");
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &[1u8; 16]),
            None
        );
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &[2u8; 16]),
            Some(NOW)
        );
        assert_eq!(engine.prune_nonces(), 0, "a second prune is a no-op");
    }
}
