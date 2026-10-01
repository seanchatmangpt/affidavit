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
//!
//! RECEIPT SIGNATURE LAW: a [`CryptoStandingReceipt`] is itself signed. The
//! certification path signs the JCS canonical pre-image of every receipt
//! field except `signature` with an ES256 attestation key (P-256, RFC 6979
//! deterministic — [`Es256SigningKey::sign`]) and records `signer_kid` (the
//! key id derived from the signing key's own fingerprint — never
//! caller-claimed) and `signer_public_key` (SEC1, carried so any holder can
//! verify the receipt offline). The custom [`serde::Deserialize`] REQUIRES
//! all three fields and VERIFIES them — kid-binding plus ES256 over the
//! reconstructed pre-image — before the receipt can exist in memory. A
//! receipt whose canonical hash an attacker recomputed around a forgery
//! (hashes are unkeyed: the hash layer alone never was authenticity) still
//! refuses as [`StandingReceiptError::BadSignature`], because the signature
//! cannot be reminted without the attestation key.
//!
//! SUBJECT BINDING LAW: [`VerificationEngine::certify_signed`] enforces that
//! the subject string it is handed digests — under
//! `BLAKE3(domain_separated(DOMAIN_TAG, [subject]))`, the same law
//! `crypto_trust_seal::subject_digest_of` applies to content addresses — to
//! exactly the envelope's `subject_digest`. A verdict over one subject can
//! never be minted as a receipt attributing it to another; the mismatch
//! refuses as [`VerifyRefusal::SubjectMismatch`] BEFORE adjudication, so a
//! mis-bound certification never consumes the presentation's nonce.

use std::cell::RefCell;

use crate::crypto_trust_canonical::{digest as domain_digest, jcs};
use crate::crypto_trust_envelope::{EnvelopeError, NonceJournal, SignatureEnvelope};
use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};
use crate::crypto_trust_keys::{
    fingerprint_public_key, AlgorithmId, CryptoProfile, InMemoryKeyRegistry, KeyId, KeyRecord,
    KeyRegistry, PublicKeyMaterial, RegistryError,
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

/// The policy a verifier enforces: the admitted audience allowlist, the
/// assurance floor, the admitted algorithm set, the revocation-staleness
/// bound, and the verifier's clock. Policy is data; the caller owns time.
#[derive(Debug, Clone)]
pub struct TrustPolicy {
    /// Audiences this verifier serves: an envelope naming any other audience
    /// is refused ([`VerifyRefusal::AudienceRefused`]) before any key is
    /// consulted. An empty allowlist refuses every presentation — closed by
    /// construction, never an accident.
    pub allowed_audiences: std::collections::BTreeSet<String>,
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
    /// every graph-admitted algorithm allowed, the trust-plane audience
    /// allowlist admitted, staleness bound `300`s, clock at 0 —
    /// set [`TrustPolicy::now`] (or use [`TrustPolicy::with_now`]) before
    /// verifying; the engine never guesses time.
    pub fn from_graph_defaults() -> Self {
        TrustPolicy {
            min_profile: CryptoProfile::Classical,
            allowed: AlgorithmId::all().iter().copied().collect(),
            allowed_audiences: ["affidavit.cli", "affidavit-notary-local", "affidavit.seal"]
                .into_iter()
                .map(str::to_string)
                .collect(),
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
    /// The envelope's audience is outside the policy's allowlist: this
    /// verifier is not a relying party of the presentation. Refused before
    /// the registry is consulted — cryptographic validity never launders a
    /// foreign audience.
    #[error("audience {0:?} refused by policy")]
    AudienceRefused(String),
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
    /// Certification was asked to mint a receipt attributing the envelope's
    /// verdict to a subject the envelope does not bind: the subject string's
    /// domain-separated digest (`BLAKE3(domain_separated(DOMAIN_TAG,
    /// [subject]))`) differs from `env.subject_digest`. Refused BEFORE
    /// adjudication, so the mis-bound presentation consumes no nonce and no
    /// journal entry.
    #[error("subject mismatch: {0}")]
    SubjectMismatch(String),
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
    ///    audience allowlisted ([`VerifyRefusal::AudienceRefused`]);
    ///    declared profile at/above the floor ([`VerifyRefusal::ProfileFloor`]);
    /// 3. registry: key known AND its record agrees with the envelope's
    ///    claims ([`VerifyRefusal::UnknownKey`]);
    /// 4. revocation: signature epoch live within the staleness grace
    ///    ([`VerifyRefusal::StaleRevocationEpoch`]), then the flat revocation
    ///    law ([`VerifyRefusal::KeyRevoked`]);
    /// 5. signature: static per-algorithm verification over the envelope's
    ///    signed bytes — signature validity GATES the journal: a decided
    ///    INVALID never touches replay state;
    /// 6. replay: ONLY a signature-valid presentation records (kid, nonce)
    ///    ([`VerifyRefusal::ReplayRejected`] on a repeat inside the window);
    ///    window-closed entries prune as a side effect of every admission, so
    ///    a long-lived verifier cannot grow the journal without bound;
    /// 7. verdict: [`CryptographicStanding::Valid`] or
    ///    [`CryptographicStanding::Invalid`] — a decided negative is `Ok`.
    ///
    /// The 5/6 order is the anti-lockout law: because the journal records
    /// only AFTER the signature decides VALID, an attacker who re-presents an
    /// observed (envelope, signature) with a flipped byte consumes nothing and
    /// cannot lock the genuine holder out, and unauthenticated
    /// garbage-signature spam cannot stuff the journal.
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

        // (2) Policy: algorithm admission, then the audience allowlist, then
        // the assurance floor. The audience is SIGNED material (envelope
        // field 12): refusing on it before the registry means a lawfully
        // registered key cannot launder a presentation for a foreign
        // subsystem through this verifier.
        if !self.policy.allowed.contains(&env.algorithm) {
            return Err(VerifyRefusal::ProfileRefused(env.algorithm));
        }
        if !self.policy.allowed_audiences.contains(&env.audience) {
            return Err(VerifyRefusal::AudienceRefused(env.audience.clone()));
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

        // (5) Signature adjudication FIRST: a failed check is a decided
        // INVALID verdict, not a refusal, and a decided INVALID must never
        // mutate replay state — an unauthenticated presentation consumes no
        // (kid, nonce) slot (anti-lockout) and no journal entry (anti-stuffing).
        let signature_holds =
            verify_signature_bytes(env.algorithm, record, &signing_input, signature)?;

        // (6) Replay defense — signature-valid presentations only: recording
        // the sight of (kid, nonce) is the admission; a repeat inside the
        // window is refused as a replay. Window-closed entries prune as a
        // side effect of every admission so the journal stays bounded without
        // relying on callers remembering to prune.
        let standing = if signature_holds {
            {
                let mut journal = self.nonces.borrow_mut();
                journal.prune(self.policy.now, NONCE_WINDOW_SECONDS);
                journal
                    .record(&kid, env.nonce, self.policy.now, NONCE_WINDOW_SECONDS)
                    .map_err(|err| VerifyRefusal::ReplayRejected(format!("{kid}: {err}")))?;
            }
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

    /// Certify a SIGNED [`CryptoStandingReceipt`]: bind the subject, verify
    /// the envelope, then mint a receipt ONLY on a
    /// [`CryptographicStanding::Valid`] verdict — and sign it.
    ///
    /// Laws, in order:
    ///
    /// 0. SUBJECT BINDING: the subject string must digest — under
    ///    `BLAKE3(domain_separated(DOMAIN_TAG, [subject]))`, the law
    ///    `crypto_trust_seal::subject_digest_of` applies to content
    ///    addresses — to exactly `env.subject_digest`; any other subject
    ///    refuses as [`VerifyRefusal::SubjectMismatch`]. Checked FIRST: a
    ///    mis-bound certification never consumes the presentation's nonce.
    /// 1. the envelope adjudicates ([`Self::verify_envelope`]); refusals
    ///    propagate, and a decided negative refuses as
    ///    [`VerifyRefusal::InvalidSignature`];
    /// 2. the mint SIGNS the receipt's canonical pre-image with `signing`
    ///    (ES256, RFC 6979) under the key id derived from that key's own
    ///    fingerprint. The receipt's authenticity rests on this attestation
    ///    key; `signer_kid` is derived, never caller-claimed.
    ///
    /// The receipt is evidence for the SA2A authorization layer; it certifies
    /// the signature, never the act. (The former unsigned `certify` is
    /// retired: an unsigned receipt cannot deserialize, so minting one would
    /// manufacture evidence that cannot load.)
    pub fn certify_signed(
        &self,
        env: &SignatureEnvelope,
        signature: &[u8],
        subject: &str,
        signing: &Es256SigningKey,
    ) -> Result<CryptoStandingReceipt, VerifyRefusal> {
        // (0) Subject binding before adjudication: refusing here never burns
        // the presentation's nonce.
        let subject_digest = domain_digest(DOMAIN_TAG, &[subject.as_bytes()]);
        if subject_digest != env.subject_digest {
            return Err(VerifyRefusal::SubjectMismatch(format!(
                "subject digests to {}, envelope binds {}",
                hex32(&subject_digest),
                hex32(&env.subject_digest)
            )));
        }
        let verdict = self.verify_envelope(env, signature)?;
        if verdict.standing != CryptographicStanding::Valid {
            return Err(VerifyRefusal::InvalidSignature(env.key_id.to_string()));
        }
        let signing_input = env.signing_input_checked()?;
        let envelope_commitment = blake3::hash(&signing_input).to_hex().to_string();
        CryptoStandingReceipt::seal_signed(
            subject,
            &envelope_commitment,
            &env.key_id.to_string(),
            env.algorithm.as_str(),
            verdict.standing,
            self.policy.now,
            signing,
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
/// The receipt is itself SIGNED (RECEIPT SIGNATURE LAW, module header):
/// `signature` is an ES256 attestation over the JCS canonical pre-image of
/// every other field, `signer_kid` derives from the attestation key's
/// fingerprint, and `signer_public_key` lets any holder verify offline. The
/// private `_seal` field prevents external struct-literal construction;
/// deserialization enforces the hash layer AND the signature layer, so a
/// tampered or forged receipt cannot load.
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
    /// Canonical BLAKE3 hash over every identity field above (JCS material).
    pub receipt_hash: String,
    /// The attestation key's registry id, derived from the signing key's own
    /// fingerprint at mint time (never caller-claimed).
    pub signer_kid: String,
    /// The signer's ES256 public key, SEC1 uncompressed (`0x04 || X || Y`) —
    /// carried so any holder can verify `signature` offline; its fingerprint
    /// MUST derive `signer_kid` (the deserialize/verify law enforces this).
    pub signer_public_key: Vec<u8>,
    /// ES256 (RFC 6979 deterministic) signature over the JCS canonical
    /// pre-image of every other field. Deserialize REQUIRES and VERIFIES it.
    pub signature: Vec<u8>,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal of the standing-receipt boundary.
#[derive(Debug, thiserror::Error)]
pub enum StandingReceiptError {
    /// The stored hash does not match the recomputed canonical hash.
    #[error("receipt hash mismatch: claimed {claimed}, recomputed {recomputed}")]
    ReceiptHashMismatch { claimed: String, recomputed: String },
    /// The receipt's signature does not verify under the carried signer key,
    /// or the carried key's fingerprint does not derive the claimed
    /// `signer_kid`. This is the variant that kills a forgery whose hash was
    /// recomputed around rewritten fields: the hash is integrity, the
    /// signature is authenticity.
    #[error("receipt signature invalid for signer {signer_kid}")]
    BadSignature { signer_kid: String },
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
    /// Seal point: computes the canonical receipt hash, SIGNS the receipt's
    /// canonical pre-image with the engine's ES256 attestation key (RFC 6979
    /// deterministic — the same key and fields always yield the same
    /// signature, so certification stays replayable byte-identically), and
    /// freezes the receipt. `pub(crate)` — only the certification path mints
    /// receipts. The signer kid is DERIVED from the signing key's
    /// fingerprint, never caller-claimed: a receipt cannot misname its
    /// signer.
    pub(crate) fn seal_signed(
        subject: &str,
        envelope_commitment: &str,
        key_id: &str,
        algorithm: &str,
        standing: CryptographicStanding,
        observed_at: u64,
        signing: &Es256SigningKey,
    ) -> Result<CryptoStandingReceipt, StandingReceiptError> {
        let signer_public_key = signing.public_key_sec1();
        let signer_kid = KeyId::from_fingerprint(&signing.key_id_fingerprint()).to_string();
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
        let preimage = signed_receipt_preimage(&SignedReceiptMaterial {
            profile: CRYPTO_STANDING_PROFILE,
            subject,
            envelope_commitment,
            key_id,
            algorithm,
            standing,
            observed_at,
            receipt_hash: &receipt_hash,
            signer_kid: &signer_kid,
            signer_public_key: &signer_public_key,
        })?;
        let signature = signing.sign(&preimage);
        Ok(CryptoStandingReceipt {
            profile: CRYPTO_STANDING_PROFILE.to_string(),
            subject: subject.to_string(),
            envelope_commitment: envelope_commitment.to_string(),
            key_id: key_id.to_string(),
            algorithm: algorithm.to_string(),
            standing,
            observed_at,
            receipt_hash,
            signer_kid,
            signer_public_key,
            signature,
            _seal: (),
        })
    }

    /// Re-audit the receipt in memory: the HASH law (every identity field
    /// matches the canonical receipt hash) AND the SIGNATURE law (the carried
    /// signer key derives `signer_kid` and its ES256 signature verifies over
    /// the canonical pre-image) — the same two-layer law
    /// [`serde::Deserialize`] enforces on the wire.
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
        self.verify_signature()
    }

    /// The signature law alone: kid-binding plus ES256 over the canonical
    /// pre-image. A malformed signature encoding is a refusal, never a panic.
    fn verify_signature(&self) -> Result<(), StandingReceiptError> {
        let preimage = signed_receipt_preimage(&SignedReceiptMaterial {
            profile: &self.profile,
            subject: &self.subject,
            envelope_commitment: &self.envelope_commitment,
            key_id: &self.key_id,
            algorithm: &self.algorithm,
            standing: self.standing,
            observed_at: self.observed_at,
            receipt_hash: &self.receipt_hash,
            signer_kid: &self.signer_kid,
            signer_public_key: &self.signer_public_key,
        })?;
        let derived = KeyId::from_fingerprint(&fingerprint_public_key(
            AlgorithmId::Es256,
            &PublicKeyMaterial::Es256Sec1(self.signer_public_key.clone()),
        ))
        .to_string();
        if derived != self.signer_kid {
            return Err(StandingReceiptError::BadSignature {
                signer_kid: self.signer_kid.clone(),
            });
        }
        let holds =
            verify_es256(&self.signer_public_key, &preimage, &self.signature).map_err(|_| {
                StandingReceiptError::BadSignature {
                    signer_kid: self.signer_kid.clone(),
                }
            })?;
        if !holds {
            return Err(StandingReceiptError::BadSignature {
                signer_kid: self.signer_kid.clone(),
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

/// The signature pre-image: every receipt field except `signature` itself
/// (identity fields, receipt hash, and the signer binding), in JCS canonical
/// form. The signer fields are INSIDE the pre-image, so a carried key cannot
/// be swapped without invalidating the signature.
#[derive(serde::Serialize)]
struct SignedReceiptMaterial<'a> {
    profile: &'a str,
    subject: &'a str,
    envelope_commitment: &'a str,
    key_id: &'a str,
    algorithm: &'a str,
    standing: CryptographicStanding,
    observed_at: u64,
    receipt_hash: &'a str,
    signer_kid: &'a str,
    signer_public_key: &'a [u8],
}

/// JCS canonical bytes of the signature pre-image.
fn signed_receipt_preimage(
    material: &SignedReceiptMaterial<'_>,
) -> Result<Vec<u8>, StandingReceiptError> {
    let value = serde_json::to_value(material)
        .map_err(|err| StandingReceiptError::Serialization(err.to_string()))?;
    let canonical =
        jcs(&value).map_err(|err| StandingReceiptError::Serialization(err.to_string()))?;
    Ok(canonical.into_bytes())
}

/// Lowercase-hex rendering of a byte slice (typed-refusal evidence).
fn hex32(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl<'de> serde::Deserialize<'de> for CryptoStandingReceipt {
    /// The wire law, in order:
    ///
    /// 1. STRUCTURAL INTEGRITY: every field is required (a receipt stripped
    ///    of `signature`/`signer_kid`/`signer_public_key` fails with a
    ///    missing-field error before any crypto runs), and the canonical
    ///    receipt hash over the identity fields must reproduce
    ///    ([`StandingReceiptError::ReceiptHashMismatch`]).
    /// 2. AUTHENTICITY: the carried signer key must derive the claimed
    ///    `signer_kid`, and its ES256 signature must verify over the
    ///    canonical pre-image of every other field
    ///    ([`StandingReceiptError::BadSignature`]). A forgery whose hash was
    ///    recomputed around rewritten fields — the hash is unkeyed, so this
    ///    is the attack the hash layer alone cannot stop — refuses here.
    ///
    /// Boundary (documented honesty): this law proves the receipt is
    /// INTERNALLY consistent and signed under its carried key; attributing
    /// the signer kid to a TRUSTED identity is the registry/engine layer's
    /// job (`certify_signed` derives the kid from the attestation key it
    /// actually holds, so minted receipts are born bound).
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
            signer_kid: String,
            signer_public_key: Vec<u8>,
            signature: Vec<u8>,
        }

        let raw = RawReceipt::deserialize(deserializer)?;
        // (1) Structural integrity: the canonical hash over the identity
        // fields must reproduce.
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
        // (2) Authenticity: kid-binding + ES256 over the signature pre-image.
        let candidate = CryptoStandingReceipt {
            profile: raw.profile,
            subject: raw.subject,
            envelope_commitment: raw.envelope_commitment,
            key_id: raw.key_id,
            algorithm: raw.algorithm,
            standing: raw.standing,
            observed_at: raw.observed_at,
            receipt_hash: raw.receipt_hash,
            signer_kid: raw.signer_kid,
            signer_public_key: raw.signer_public_key,
            signature: raw.signature,
            _seal: (),
        };
        candidate.verify_signature().map_err(D::Error::custom)?;
        Ok(candidate)
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
            // The envelope binds the fixture subject under the same
            // domain-separated law `certify_signed` enforces — a mis-bound
            // subject is exercised by exact falsifiers below, not by every
            // fixture.
            subject_digest: subject_bound("subject-a"),
            // Allowlisted (the engine's default policy): the audience law is
            // exercised by exact falsifiers below, not by every fixture.
            audience: "affidavit.cli".to_string(),
        }
    }

    /// The domain-separated digest of a subject string — the exact law
    /// `certify_signed` enforces against `env.subject_digest` (and the law
    /// `crypto_trust_seal::subject_digest_of` applies to content addresses).
    fn subject_bound(subject: &str) -> [u8; 32] {
        crate::crypto_trust_canonical::digest(DOMAIN_TAG, &[subject.as_bytes()])
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
    // The nonce journal is anti-replay STATE inside the engine: a
    // SIGNATURE-VALID adjudication records the presentation's (kid, nonce);
    // a decided INVALID records nothing. One valid presentation is therefore
    // adjudicated ONCE per engine — verify-then-certify of the same envelope
    // on one engine is lawfully a replay, and certify-then-certify is
    // refused. Fresh engines below prove the positive paths.

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
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("VALID verdict must mint a receipt");
        assert_eq!(receipt.profile, CRYPTO_STANDING_PROFILE);
        assert_eq!(receipt.standing, CryptographicStanding::Valid);
        assert_eq!(receipt.key_id, record.id.to_string());
        assert_eq!(receipt.algorithm, "ES256");
        assert_eq!(receipt.subject, "subject-a");
        assert_eq!(receipt.observed_at, NOW);
        // The receipt signature law: the signer kid derives from the
        // attestation key, the key material is carried, the signature exists.
        assert_eq!(receipt.signer_kid, record.id.to_string());
        assert_eq!(receipt.signer_public_key, signing.public_key_sec1());
        assert!(!receipt.signature.is_empty());
        receipt.verify().expect("minted receipt must re-verify");

        // Double-mint guard: one presentation, one receipt.
        match engine.certify_signed(&env, &signature, "subject-a", &signing) {
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
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint A");
        let receipt_b = engine_with(&record)
            .certify_signed(&env, &signature, "subject-a", &signing)
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
    fn flipped_byte_replay_cannot_lock_out_the_genuine_holder() {
        // The F1 exploit: the attacker observes a legitimate
        // (envelope, signature) pair and re-presents it FIRST with one flipped
        // signature byte. A decided INVALID must not consume the (kid, nonce)
        // slot — the genuine holder must still verify inside the window.
        let (signing, record) = es256_fixture(25);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [23u8; 16], 0);
        let genuine = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));

        // The forged presentation: the observed envelope, one flipped sig byte.
        let mut forged = genuine.clone();
        let last = forged.len() - 1;
        forged[last] ^= 0x01; // well-formed DER scalar flip: decides false
        let verdict = engine
            .verify_envelope(&env, &forged)
            .expect("a tampered re-presentation is DECIDED, not refused");
        assert_eq!(verdict.standing, CryptographicStanding::Invalid);

        // The forged attempt consumed nothing: the nonce is still free.
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
            None,
            "a decided INVALID must not journal the nonce"
        );

        // The genuine holder presents the true pair: still VALID (lockout
        // defeated), and the nonce is journaled exactly here.
        let verdict = engine
            .verify_envelope(&env, &genuine)
            .expect("genuine pair must verify after the forged attempt");
        assert_eq!(verdict.standing, CryptographicStanding::Valid);
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
            Some(NOW)
        );

        // And now the replay law bites a second genuine presentation.
        match engine.verify_envelope(&env, &genuine) {
            Err(VerifyRefusal::ReplayRejected(detail)) => {
                assert!(detail.contains(&record.id.to_string()));
            }
            other => panic!("expected ReplayRejected for the repeat, got {other:?}"),
        }
    }

    #[test]
    fn garbage_signature_spam_does_not_grow_the_journal() {
        // The F1 stuffing exploit: `kid` is public registry data, so every
        // presentation passes window/policy/registry/revocation. Spamming
        // fresh nonces with garbage signatures must leave the journal empty —
        // insertion happens ONLY on a signature-valid path, so the journal
        // length is unchanged (empty before, empty after: every spammed
        // (kid, nonce) pair is absent).
        let (signing, record) = es256_fixture(26);
        let engine = engine_with(&record);
        for i in 0..64u8 {
            let env = envelope(&record.id, [i; 16], 0);
            let mut signature =
                signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
            let last = signature.len() - 1;
            signature[last] ^= 0x01; // garbage that still decides, never verifies
            let verdict = engine
                .verify_envelope(&env, &signature)
                .expect("garbage signature is DECIDED, not refused");
            assert_eq!(verdict.standing, CryptographicStanding::Invalid);
            assert_eq!(
                engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
                None,
                "spam must not journal the nonce for byte {i}"
            );
        }
        // A nonce the spammer burned is still usable by the genuine holder.
        let env = envelope(&record.id, [0u8; 16], 0);
        let genuine = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let verdict = engine
            .verify_envelope(&env, &genuine)
            .expect("fresh genuine presentation after spam");
        assert_eq!(verdict.standing, CryptographicStanding::Valid);
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
    fn foreign_audience_refuses_even_under_a_lawful_key() {
        // The key is lawfully registered and the signature over the actual
        // presentation is real: the ONLY defect is the audience. A signed
        // presentation for a subsystem this verifier does not serve must
        // refuse BY VARIANT — before the registry is consulted.
        let (signing, record) = es256_fixture(22);
        let engine = engine_with(&record);
        let mut env = envelope(&record.id, [23u8; 16], 0);
        env.audience = "other-subsystem".to_string();
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.verify_envelope(&env, &signature) {
            Err(VerifyRefusal::AudienceRefused(aud)) => assert_eq!(aud, "other-subsystem"),
            other => panic!("expected AudienceRefused, got {other:?}"),
        }
    }

    #[test]
    fn allowlisted_audience_reaches_adjudication() {
        // Teeth: every default allowlist string admits a fresh, lawfully
        // signed presentation. Without this witness the refusal above could
        // be vacuous — a gate that refuses everything is not a gate.
        for audience in ["affidavit.cli", "affidavit-notary-local", "affidavit.seal"] {
            let (signing, record) = es256_fixture(23);
            let engine = engine_with(&record);
            let mut env = envelope(&record.id, [24u8; 16], 0);
            env.audience = audience.to_string();
            let signature =
                signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
            let verdict = engine
                .verify_envelope(&env, &signature)
                .expect("an allowlisted audience must be adjudicated");
            assert_eq!(verdict.standing, CryptographicStanding::Valid);
        }
    }

    #[test]
    fn empty_allowlist_refuses_every_audience() {
        // Documented law: an empty allowlist is CLOSED BY CONSTRUCTION — no
        // presentation is admissible, including audiences that are otherwise
        // lawfully served. Setting an empty set is a policy decision, never a
        // bypass.
        let (signing, record) = es256_fixture(24);
        let mut policy = TrustPolicy::from_graph_defaults().with_now(NOW);
        policy.allowed_audiences.clear();
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        let engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            NonceJournal::default(),
            policy,
        );
        for audience in [
            "affidavit.cli",
            "affidavit-notary-local",
            "affidavit.seal",
            "other-subsystem",
        ] {
            let mut env = envelope(&record.id, [25u8; 16], 0);
            env.audience = audience.to_string();
            let signature =
                signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
            match engine.verify_envelope(&env, &signature) {
                Err(VerifyRefusal::AudienceRefused(aud)) => assert_eq!(aud, audience),
                other => panic!("empty allowlist must refuse {audience}, got {other:?}"),
            }
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
        // A decided INVALID journals nothing (anti-lockout, anti-stuffing).
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
            None,
            "a decided INVALID must not consume the nonce"
        );
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
        match engine.certify_signed(&env, &signature, "subject-a", &signing) {
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
        let receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");
        let mut json = serde_json::to_value(&receipt).expect("receipt serializes");
        json["standing"] = serde_json::Value::String("INVALID".to_string());
        let outcome = serde_json::from_value::<CryptoStandingReceipt>(json);
        let err = outcome.expect_err("a flipped standing must not load");
        // The flipped standing breaks the STRUCTURAL layer first: the
        // canonical hash over the identity fields no longer reproduces, so
        // the refusal is the typed ReceiptHashMismatch rendering. (A forgery
        // that recomputes the hash to match is the BadSignature layer's
        // kill — witnessed below.)
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
        let mut receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");
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

    // -- the receipt signature law: authenticity, not just integrity ------------

    #[test]
    fn certify_refuses_a_subject_swap_before_adjudication() {
        // The envelope binds subject-a (the fixture's subject_digest law);
        // certification is asked to attribute the verdict to subject-b. The
        // mismatch refuses BY VARIANT before the envelope is adjudicated, so
        // the presentation's nonce is never consumed.
        let (signing, record) = es256_fixture(26);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [27u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        match engine.certify_signed(&env, &signature, "subject-b", &signing) {
            Err(VerifyRefusal::SubjectMismatch(detail)) => {
                assert!(
                    detail.contains(&hex32(&subject_bound("subject-b"))),
                    "the refusal must name the subject's digest: {detail}"
                );
                assert!(
                    detail.contains(&hex32(&env.subject_digest)),
                    "the refusal must name the envelope's binding: {detail}"
                );
            }
            other => panic!("expected SubjectMismatch, got {other:?}"),
        }
        // Anti-lockout composition: the mis-bound certification burned no
        // replay state.
        assert_eq!(
            engine.nonce_seen_at(&record.id.to_string(), &env.nonce),
            None,
            "a subject-mismatched certification must not consume the nonce"
        );
    }

    #[test]
    fn subject_bound_digest_matches_the_seal_law() {
        // The certification subject law and the seal module's
        // subject_digest_of law are the same domain-separated construction: a
        // content-address-shaped subject string digests identically under
        // both. This is the composition seam the journal and seal paths
        // drive.
        let address = "a".repeat(64);
        assert_eq!(
            crate::crypto_trust_canonical::digest_hex(DOMAIN_TAG, &[address.as_bytes()]),
            hex32(&subject_bound(&address))
        );
    }

    #[test]
    fn forged_receipt_with_recomputed_hash_refuses_by_bad_signature() {
        // THE FORGERY FALSIFIER: hashes are UNKEYED, so an attacker can
        // rewrite the fields and recompute the receipt hash around the
        // rewrite — the hash layer's exact hole. The signature is not
        // rebuildable: the original signature no longer covers the rewritten
        // pre-image and the attacker holds no attestation key.
        let (signing, record) = es256_fixture(28);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [29u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");

        // Flip the standing, recompute the hash honestly, keep the original
        // signature: structurally consistent, cryptographically false.
        let mut forgery = receipt.clone();
        forgery.standing = CryptographicStanding::Invalid;
        forgery.receipt_hash = receipt_material_hash(&ReceiptMaterial {
            profile: &forgery.profile,
            subject: &forgery.subject,
            envelope_commitment: &forgery.envelope_commitment,
            key_id: &forgery.key_id,
            algorithm: &forgery.algorithm,
            standing: forgery.standing,
            observed_at: forgery.observed_at,
        })
        .expect("an attacker can recompute unkeyed hashes");

        // The wire path refuses. serde erases the concrete error type at this
        // boundary, so the variant witness here is the typed rendering; the
        // variant itself is witnessed directly by the in-memory audit below.
        let json = serde_json::to_string(&forgery).expect("forgery serializes");
        let err = serde_json::from_str::<CryptoStandingReceipt>(&json)
            .expect_err("a recomputed-hash forgery must not load");
        assert!(
            err.to_string()
                .starts_with("receipt signature invalid for signer "),
            "the forged receipt must refuse as BadSignature, got {err}"
        );

        // The in-memory audit refuses BY VARIANT (the recomputed hash passes
        // the integrity layer; the signature layer is what kills it).
        match forgery.verify() {
            Err(StandingReceiptError::BadSignature { signer_kid }) => {
                assert_eq!(signer_kid, receipt.signer_kid);
            }
            other => panic!("expected BadSignature, got {other:?}"),
        }
    }

    #[test]
    fn tampered_signature_bytes_refuse_by_bad_signature() {
        // Hash-consistent, signature-false: flipping one signature byte must
        // refuse BY VARIANT on the wire — the signature layer has teeth the
        // hash layer cannot simulate.
        let (signing, record) = es256_fixture(30);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [31u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");
        let mut json = serde_json::to_value(&receipt).expect("receipt serializes");
        let first = json["signature"][0].as_u64().expect("signature byte");
        json["signature"][0] = serde_json::Value::from(first ^ 0x01);
        match serde_json::from_value::<CryptoStandingReceipt>(json) {
            Err(err) => assert!(
                err.to_string()
                    .starts_with("receipt signature invalid for signer "),
                "expected the typed BadSignature rendering, got {err}"
            ),
            Ok(_) => panic!("a tampered signature must not load"),
        }
    }

    #[test]
    fn signer_key_substitution_refuses_by_bad_signature() {
        // Swap the carried signer key (and an honest kid for it) while
        // keeping the original signature: the signature breaks — the signer
        // fields are INSIDE the pre-image, so the substituted pre-image is
        // not what was signed. Boundary (documented honesty): deserialization
        // proves internal consistency under the carried key; attributing the
        // kid to a TRUSTED identity is the registry/engine layer's job.
        let (signing, record) = es256_fixture(32);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [33u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");
        let (rogue, _rogue_record) = es256_fixture(34);
        let mut forgery = receipt.clone();
        forgery.signer_public_key = rogue.public_key_sec1();
        forgery.signer_kid = KeyId::from_fingerprint(&rogue.key_id_fingerprint()).to_string();
        // Identity fields untouched: the integrity layer passes; the
        // signature layer is the kill.
        assert_eq!(forgery.receipt_hash, receipt.receipt_hash);
        match forgery.verify() {
            Err(StandingReceiptError::BadSignature { signer_kid }) => {
                // The refusal names the CLAIMED signer: the forged kid that
                // failed its own signature law.
                assert_eq!(signer_kid, forgery.signer_kid);
            }
            other => panic!("expected BadSignature, got {other:?}"),
        }
    }

    #[test]
    fn receipt_without_signature_fields_refuses_to_load() {
        // The wire form REQUIRES the signature fields: a receipt stripped of
        // its authenticity cannot load at all (a serde missing-field error —
        // absence is structural, before any crypto runs).
        let (signing, record) = es256_fixture(35);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [36u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");
        let mut json = serde_json::to_value(&receipt)
            .expect("receipt serializes")
            .as_object_mut()
            .expect("object")
            .clone();
        json.remove("signature");
        let outcome =
            serde_json::from_value::<CryptoStandingReceipt>(serde_json::Value::Object(json));
        let err = outcome.expect_err("a signature-less receipt must not load");
        assert!(
            err.to_string().contains("signature"),
            "the refusal must name the missing signature: {err}"
        );
    }

    #[test]
    fn receipt_round_trips_byte_identically() {
        let (signing, record) = es256_fixture(20);
        let engine = engine_with(&record);
        let env = envelope(&record.id, [22u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        let receipt = engine
            .certify_signed(&env, &signature, "subject-a", &signing)
            .expect("mint");
        let json = serde_json::to_string(&receipt).expect("receipt serializes");
        let back: CryptoStandingReceipt =
            serde_json::from_str(&json).expect("honest receipt loads");
        assert_eq!(back, receipt);
        assert_eq!(serde_json::to_string(&back).expect("re-serialize"), json);
        // The signature fields ride the wire unchanged.
        assert_eq!(back.signer_kid, receipt.signer_kid);
        assert_eq!(back.signer_public_key, receipt.signer_public_key);
        assert_eq!(back.signature, receipt.signature);
        back.verify().expect("loaded receipt re-audits");
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
        // The audience allowlist is part of the constructor's contract: the
        // trust-plane relying parties, carried intact through `with_now`.
        assert_eq!(policy.allowed_audiences.len(), 3);
        for audience in ["affidavit.cli", "affidavit-notary-local", "affidavit.seal"] {
            assert!(
                policy.allowed_audiences.contains(audience),
                "{audience} must be allowlisted by the graph defaults"
            );
        }
        let timed = TrustPolicy::from_graph_defaults().with_now(NOW);
        assert_eq!(timed.now, NOW);
        assert_eq!(timed.allowed_audiences, policy.allowed_audiences);
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
        let (signing, record) = es256_fixture(21);
        let kid = record.id.to_string();

        // Phase 1 — the explicit prune still counts what it evicts: an entry
        // 400s old at the verifier clock (past the 300s window), with NO
        // verification in between.
        let mut journal = NonceJournal::default();
        journal
            .record(&kid, [1u8; 16], NOW - 400, NONCE_WINDOW_SECONDS)
            .expect("first sight admitted");
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        let mut engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            journal,
            TrustPolicy::from_graph_defaults().with_now(NOW),
        );
        assert_eq!(engine.prune_nonces(), 1, "exactly the expired entry evicts");
        assert_eq!(engine.nonce_seen_at(&kid, &[1u8; 16]), None);
        assert_eq!(engine.prune_nonces(), 0, "a second prune is a no-op");

        // Phase 2 — journal hygiene is automatic on the admitted path: every
        // signature-valid verification prunes window-closed entries as a side
        // effect of recording the fresh nonce, so a long-lived verifier that
        // never calls prune_nonces stays bounded.
        let mut journal = NonceJournal::default();
        journal
            .record(&kid, [1u8; 16], NOW - 400, NONCE_WINDOW_SECONDS)
            .expect("first sight admitted");
        let mut registry = InMemoryKeyRegistry::new();
        registry.register(record.clone()).expect("register");
        let mut engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            journal,
            TrustPolicy::from_graph_defaults().with_now(NOW),
        );
        // A live, signature-valid verification journals a fresh nonce at NOW
        // AND evicts the expired entry in the same admission.
        let env = envelope(&record.id, [2u8; 16], 0);
        let signature = signing.sign(&env.signing_input_checked().expect("canonical pre-image"));
        engine.verify_envelope(&env, &signature).expect("fresh");
        assert_eq!(
            engine.nonce_seen_at(&kid, &[1u8; 16]),
            None,
            "the expired entry must be pruned by the verify itself"
        );
        assert_eq!(
            engine.nonce_seen_at(&kid, &[2u8; 16]),
            Some(NOW),
            "the fresh nonce survives the side-effect prune"
        );
        assert_eq!(
            engine.prune_nonces(),
            0,
            "nothing left to evict: the verify already pruned"
        );
    }
}
