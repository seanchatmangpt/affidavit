//! Receipt-bound standing certification for the Chatman ecosystem.
//!
//! This module is deliberately a **certifier, not an actuator**. It takes an
//! already-admitted Affidavit receipt, a structurally valid authority envelope,
//! exact subject identity, and observed execution/verification/replay evidence,
//! then seals those facts into a deterministic standing receipt.
//!
//! The key law is intentionally asymmetric: callers may record UNKNOWN,
//! PARTIAL_ALIVE, BLOCKED, BUILD_BROKEN, or UNSUPPORTED with incomplete evidence,
//! but ALIVE is unconstructable unless successful execution, successful
//! verification, and a replay recipe are all present. The certifier does not
//! infer that the evidence is truthful; it makes the claim's required structure
//! explicit, content-addressed, replay-linked, and tamper evident.

use crate::types::{AdmittedReceipt, Blake3Hash};
use serde::{Deserialize, Deserializer, Serialize};
use wasm4pm_compat::authority::{AuthorityConstraint, AuthorityEnvelope, AuthorityRefusal};
use wasm4pm_compat::witness::Witness;

/// Stable profile tag for ecosystem standing receipts.
pub const STANDING_PROFILE: &str = "affidavit/standing/v2";

/// Scoped standing over the exact admitted subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Standing {
    /// No execution standing has been established.
    Unknown,
    /// Some required evidence exists, but the ALIVE crown is incomplete.
    PartialAlive,
    /// Successful execution + verification + replay are all bound to the receipt.
    Alive,
    /// Execution could not proceed because a required capability/authority/input was absent.
    Blocked,
    /// The admitted subject was executed far enough to establish a build failure.
    BuildBroken,
    /// The requested edge is outside the admitted capability/ontology boundary.
    Unsupported,
}

/// Exact identity of the subject whose standing is being certified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectIdentity {
    /// Canonical subject name, for example `seanchatmangpt/affidavit`.
    pub subject: String,
    /// Frozen base identity (normally an exact Git commit SHA).
    pub base: String,
    /// Frozen source-tree identity.
    pub tree: String,
    /// Exact candidate/head identity that was executed.
    pub candidate: String,
}

/// Observed execution evidence. `exit_code` is data, never inferred from text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionEvidence {
    /// Exact command or execution contract that ran.
    pub command: String,
    /// Observed process exit code.
    pub exit_code: i32,
    /// Commitment to the execution result/log/artifact set.
    pub result_commitment: Blake3Hash,
}

/// Independent verification evidence for the executed subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationEvidence {
    /// Exact verifier command or behavioral proof contract.
    pub command: String,
    /// Observed verifier exit code.
    pub exit_code: i32,
    /// Commitment to the verifier report/log/artifact set.
    pub report_commitment: Blake3Hash,
}

/// Replay information sufficient to identify how the evidence can be reproduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayEvidence {
    /// Replay command or deterministic replay entry point.
    pub command: String,
    /// Commitment to the toolchain/config/environment capsule.
    pub environment_commitment: Blake3Hash,
}

/// Canonical projection of a validated wasm4pm-compat authority envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityBinding {
    /// Stable witness key naming the authority family.
    pub witness_key: String,
    /// Capability name.
    pub capability: String,
    /// Capability bundle/version digest pin.
    pub capability_digest: String,
    /// Bounded scope carried by the envelope.
    pub scope: String,
    /// Canonical names of declared structural authority constraints.
    pub constraints: Vec<String>,
    /// Declared data-minimization note, if any.
    pub data_minimization_note: String,
    /// Declared fairness-attestation reference, if any.
    pub fairness_attestation_ref: String,
}

/// Inputs admitted for standing certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingObservation {
    /// Exact subject identity.
    pub subject: SubjectIdentity,
    /// Commitment to the admitted observation set O* used for this claim.
    pub observation_commitment: Blake3Hash,
    /// Requested standing. ALIVE has the strongest evidence law.
    pub standing: Standing,
    /// Observed execution, when execution occurred.
    pub execution: Option<ExecutionEvidence>,
    /// Independent verification, when verification occurred.
    pub verification: Option<VerificationEvidence>,
    /// Replay recipe/capsule identity, when replay is available.
    pub replay: Option<ReplayEvidence>,
    /// Previous standing receipt hash for receipt-DAG lineage.
    pub previous_receipt: Option<Blake3Hash>,
}

/// A sealed, deterministic ecosystem standing receipt.
///
/// The private `_seal` field prevents external struct-literal construction.
/// Deserialization re-runs all structural laws and the receipt hash, so loading
/// JSON cannot bypass certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::manual_non_exhaustive)]
pub struct StandingReceipt {
    /// Receipt profile. Always [`STANDING_PROFILE`] when certified here.
    pub profile: String,
    /// Exact subject identity.
    pub subject: SubjectIdentity,
    /// Commitment to O*.
    pub observation_commitment: Blake3Hash,
    /// BLAKE3 chain hash of the already-admitted source Affidavit receipt.
    pub admitted_receipt_hash: Blake3Hash,
    /// Structurally validated authority binding.
    pub authority: AuthorityBinding,
    /// Scoped standing.
    pub standing: Standing,
    /// Observed execution evidence.
    pub execution: Option<ExecutionEvidence>,
    /// Independent verification evidence.
    pub verification: Option<VerificationEvidence>,
    /// Replay evidence.
    pub replay: Option<ReplayEvidence>,
    /// Previous standing receipt hash, if this extends an earlier receipt.
    pub previous_receipt: Option<Blake3Hash>,
    /// Canonical BLAKE3 hash over every field above.
    pub receipt_hash: Blake3Hash,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal from the standing-certification boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandingRefusal {
    /// wasm4pm-compat refused the declared authority envelope structurally.
    AuthorityEnvelopeRejected(Vec<AuthorityRefusal>),
    /// A future authority constraint is not yet in this profile's canonical projection.
    UnsupportedAuthorityConstraint,
    /// A required identity/text field was empty.
    EmptyField(&'static str),
    /// A claimed BLAKE3 commitment was not exactly 64 lowercase hex digits.
    MalformedBlake3(&'static str),
    /// ALIVE was requested without observed execution.
    AliveMissingExecution,
    /// ALIVE execution did not exit successfully.
    AliveExecutionFailed(i32),
    /// ALIVE was requested without independent verification.
    AliveMissingVerification,
    /// ALIVE verification did not exit successfully.
    AliveVerificationFailed(i32),
    /// ALIVE was requested without replay evidence.
    AliveMissingReplay,
    /// Receipt profile did not match this certifier.
    WrongProfile,
    /// Canonical receipt hash did not match the stored hash.
    ReceiptHashMismatch,
    /// Canonical serialization unexpectedly failed.
    Serialization(String),
}

impl core::fmt::Display for StandingRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AuthorityEnvelopeRejected(reasons) => {
                write!(f, "authority_envelope_rejected: {reasons:?}")
            }
            Self::UnsupportedAuthorityConstraint => f.write_str("unsupported_authority_constraint"),
            Self::EmptyField(field) => write!(f, "empty_field: {field}"),
            Self::MalformedBlake3(field) => write!(f, "malformed_blake3: {field}"),
            Self::AliveMissingExecution => f.write_str("alive_missing_execution"),
            Self::AliveExecutionFailed(code) => write!(f, "alive_execution_failed: {code}"),
            Self::AliveMissingVerification => f.write_str("alive_missing_verification"),
            Self::AliveVerificationFailed(code) => {
                write!(f, "alive_verification_failed: {code}")
            }
            Self::AliveMissingReplay => f.write_str("alive_missing_replay"),
            Self::WrongProfile => f.write_str("wrong_standing_profile"),
            Self::ReceiptHashMismatch => f.write_str("standing_receipt_hash_mismatch"),
            Self::Serialization(reason) => write!(f, "standing_serialization: {reason}"),
        }
    }
}

impl std::error::Error for StandingRefusal {}

#[derive(Serialize)]
struct StandingMaterial<'a> {
    profile: &'a str,
    subject: &'a SubjectIdentity,
    observation_commitment: &'a Blake3Hash,
    admitted_receipt_hash: &'a Blake3Hash,
    authority: &'a AuthorityBinding,
    standing: Standing,
    execution: &'a Option<ExecutionEvidence>,
    verification: &'a Option<VerificationEvidence>,
    replay: &'a Option<ReplayEvidence>,
    previous_receipt: &'a Option<Blake3Hash>,
}

impl StandingReceipt {
    /// Re-run the receipt's structural laws and canonical hash.
    pub fn verify(&self) -> Result<(), StandingRefusal> {
        validate_material(
            &self.profile,
            &self.subject,
            &self.observation_commitment,
            &self.admitted_receipt_hash,
            &self.authority,
            self.standing,
            &self.execution,
            &self.verification,
            &self.replay,
            &self.previous_receipt,
        )?;

        let recomputed = compute_receipt_hash(
            &self.profile,
            &self.subject,
            &self.observation_commitment,
            &self.admitted_receipt_hash,
            &self.authority,
            self.standing,
            &self.execution,
            &self.verification,
            &self.replay,
            &self.previous_receipt,
        )?;
        if recomputed != self.receipt_hash {
            return Err(StandingRefusal::ReceiptHashMismatch);
        }
        Ok(())
    }
}

/// Certify a standing claim over an already-admitted Affidavit receipt.
///
/// This function performs no actuation. `admitted` proves the source receipt
/// crossed Affidavit's existing admission court; `authority` is structurally
/// validated by wasm4pm-compat before it is projected; and the ALIVE law is
/// enforced before the new receipt is sealed.
pub fn certify_standing<W: Witness>(
    admitted: &AdmittedReceipt,
    authority: &AuthorityEnvelope<W>,
    observation: StandingObservation,
) -> Result<StandingReceipt, StandingRefusal> {
    authority
        .validate()
        .map_err(StandingRefusal::AuthorityEnvelopeRejected)?;
    let authority = project_authority(authority)?;
    let admitted_receipt_hash = admitted.value.chain_hash.clone();

    validate_material(
        STANDING_PROFILE,
        &observation.subject,
        &observation.observation_commitment,
        &admitted_receipt_hash,
        &authority,
        observation.standing,
        &observation.execution,
        &observation.verification,
        &observation.replay,
        &observation.previous_receipt,
    )?;

    let receipt_hash = compute_receipt_hash(
        STANDING_PROFILE,
        &observation.subject,
        &observation.observation_commitment,
        &admitted_receipt_hash,
        &authority,
        observation.standing,
        &observation.execution,
        &observation.verification,
        &observation.replay,
        &observation.previous_receipt,
    )?;

    Ok(StandingReceipt {
        profile: STANDING_PROFILE.to_string(),
        subject: observation.subject,
        observation_commitment: observation.observation_commitment,
        admitted_receipt_hash,
        authority,
        standing: observation.standing,
        execution: observation.execution,
        verification: observation.verification,
        replay: observation.replay,
        previous_receipt: observation.previous_receipt,
        receipt_hash,
        _seal: (),
    })
}

fn project_authority<W: Witness>(
    authority: &AuthorityEnvelope<W>,
) -> Result<AuthorityBinding, StandingRefusal> {
    let constraints = authority
        .constraints
        .iter()
        .map(|constraint| {
            let name = match constraint {
                AuthorityConstraint::RequiresWitness => "RequiresWitness",
                AuthorityConstraint::RequiresDigestPin => "RequiresDigestPin",
                AuthorityConstraint::RequiresBoundedScope => "RequiresBoundedScope",
                AuthorityConstraint::RequiresExpiry => "RequiresExpiry",
                AuthorityConstraint::RequiresDataMinimization => "RequiresDataMinimization",
                AuthorityConstraint::RequiresFairnessAttestation => "RequiresFairnessAttestation",
                _ => return Err(StandingRefusal::UnsupportedAuthorityConstraint),
            };
            Ok(name.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AuthorityBinding {
        witness_key: W::KEY.to_string(),
        capability: authority.capability.name.clone(),
        capability_digest: authority.capability.digest.0.clone(),
        scope: authority.scope.clone(),
        constraints,
        data_minimization_note: authority.data_minimization_note.clone(),
        fairness_attestation_ref: authority.fairness_attestation_ref.clone(),
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_material(
    profile: &str,
    subject: &SubjectIdentity,
    observation_commitment: &Blake3Hash,
    admitted_receipt_hash: &Blake3Hash,
    authority: &AuthorityBinding,
    standing: Standing,
    execution: &Option<ExecutionEvidence>,
    verification: &Option<VerificationEvidence>,
    replay: &Option<ReplayEvidence>,
    previous_receipt: &Option<Blake3Hash>,
) -> Result<(), StandingRefusal> {
    if profile != STANDING_PROFILE {
        return Err(StandingRefusal::WrongProfile);
    }

    require_text("subject.subject", &subject.subject)?;
    require_text("subject.base", &subject.base)?;
    require_text("subject.tree", &subject.tree)?;
    require_text("subject.candidate", &subject.candidate)?;
    require_blake3("observation_commitment", observation_commitment)?;
    require_blake3("admitted_receipt_hash", admitted_receipt_hash)?;

    require_text("authority.witness_key", &authority.witness_key)?;
    require_text("authority.capability", &authority.capability)?;
    require_text("authority.capability_digest", &authority.capability_digest)?;
    require_text("authority.scope", &authority.scope)?;
    if authority.constraints.is_empty() {
        return Err(StandingRefusal::EmptyField("authority.constraints"));
    }

    if let Some(execution) = execution {
        require_text("execution.command", &execution.command)?;
        require_blake3("execution.result_commitment", &execution.result_commitment)?;
    }
    if let Some(verification) = verification {
        require_text("verification.command", &verification.command)?;
        require_blake3(
            "verification.report_commitment",
            &verification.report_commitment,
        )?;
    }
    if let Some(replay) = replay {
        require_text("replay.command", &replay.command)?;
        require_blake3(
            "replay.environment_commitment",
            &replay.environment_commitment,
        )?;
    }
    if let Some(previous) = previous_receipt {
        require_blake3("previous_receipt", previous)?;
    }

    if standing == Standing::Alive {
        let execution = execution
            .as_ref()
            .ok_or(StandingRefusal::AliveMissingExecution)?;
        if execution.exit_code != 0 {
            return Err(StandingRefusal::AliveExecutionFailed(execution.exit_code));
        }

        let verification = verification
            .as_ref()
            .ok_or(StandingRefusal::AliveMissingVerification)?;
        if verification.exit_code != 0 {
            return Err(StandingRefusal::AliveVerificationFailed(
                verification.exit_code,
            ));
        }
        if replay.is_none() {
            return Err(StandingRefusal::AliveMissingReplay);
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn compute_receipt_hash(
    profile: &str,
    subject: &SubjectIdentity,
    observation_commitment: &Blake3Hash,
    admitted_receipt_hash: &Blake3Hash,
    authority: &AuthorityBinding,
    standing: Standing,
    execution: &Option<ExecutionEvidence>,
    verification: &Option<VerificationEvidence>,
    replay: &Option<ReplayEvidence>,
    previous_receipt: &Option<Blake3Hash>,
) -> Result<Blake3Hash, StandingRefusal> {
    let material = StandingMaterial {
        profile,
        subject,
        observation_commitment,
        admitted_receipt_hash,
        authority,
        standing,
        execution,
        verification,
        replay,
        previous_receipt,
    };
    let canonical = serde_json::to_vec(&material)
        .map_err(|err| StandingRefusal::Serialization(err.to_string()))?;
    let mut bytes = Vec::with_capacity(STANDING_PROFILE.len() + canonical.len() + 1);
    bytes.extend_from_slice(STANDING_PROFILE.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&canonical);
    Ok(Blake3Hash::from_bytes(&bytes))
}

fn require_text(field: &'static str, value: &str) -> Result<(), StandingRefusal> {
    if value.trim().is_empty() {
        Err(StandingRefusal::EmptyField(field))
    } else {
        Ok(())
    }
}

fn require_blake3(field: &'static str, value: &Blake3Hash) -> Result<(), StandingRefusal> {
    let hex = value.as_hex();
    // Lowercase only. `is_ascii_hexdigit` also accepts A-F, which would let
    // the same digest appear as two distinct strings and therefore hash to
    // two distinct receipt identities — a canonicalisation hole under ADR-5.
    // Every digest this crate produces is lowercase (`blake3::Hash::to_hex`),
    // so this narrows admission to what is already canonical.
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(StandingRefusal::MalformedBlake3(field))
    }
}

impl<'de> Deserialize<'de> for StandingReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct RawStandingReceipt {
            profile: String,
            subject: SubjectIdentity,
            observation_commitment: Blake3Hash,
            admitted_receipt_hash: Blake3Hash,
            authority: AuthorityBinding,
            standing: Standing,
            execution: Option<ExecutionEvidence>,
            verification: Option<VerificationEvidence>,
            replay: Option<ReplayEvidence>,
            previous_receipt: Option<Blake3Hash>,
            receipt_hash: Blake3Hash,
        }

        let raw = RawStandingReceipt::deserialize(deserializer)?;
        let receipt = StandingReceipt {
            profile: raw.profile,
            subject: raw.subject,
            observation_commitment: raw.observation_commitment,
            admitted_receipt_hash: raw.admitted_receipt_hash,
            authority: raw.authority,
            standing: raw.standing,
            execution: raw.execution,
            verification: raw.verification,
            replay: raw.replay,
            previous_receipt: raw.previous_receipt,
            receipt_hash: raw.receipt_hash,
            _seal: (),
        };
        receipt.verify().map_err(D::Error::custom)?;
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use wasm4pm_compat::authority::{AuthorityEnvelope, Capability};
    use wasm4pm_compat::receipt::Digest;
    use wasm4pm_compat::witness::RustTypestateLaw;

    fn admitted_receipt() -> AdmittedReceipt {
        let mut assembler = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "test",
            vec![object_ref("repo", "git")],
            b"exact-head execution",
            &mut counter,
        )
        .expect("event");
        assembler.append(event).expect("append");
        crate::admission::admit(assembler.finalize()).expect("admitted")
    }

    fn authority(scope: &str) -> AuthorityEnvelope<RustTypestateLaw> {
        let capability = Capability::<RustTypestateLaw>::new(
            "affidavit.certify-standing",
            Digest::new("blake3:affidavit-standing-v2"),
        );
        AuthorityEnvelope::new(
            capability,
            vec![
                AuthorityConstraint::RequiresWitness,
                AuthorityConstraint::RequiresDigestPin,
                AuthorityConstraint::RequiresBoundedScope,
            ],
            scope,
        )
    }

    fn alive_observation() -> StandingObservation {
        StandingObservation {
            subject: SubjectIdentity {
                subject: "seanchatmangpt/affidavit".to_string(),
                base: "383b41dc9c587f8a8a90fe280b9621ec55701a2c".to_string(),
                tree: "cb64a2a465c73519067803df46a0471e088aa39a".to_string(),
                candidate: "candidate-head".to_string(),
            },
            observation_commitment: Blake3Hash::from_bytes(b"O-star"),
            standing: Standing::Alive,
            execution: Some(ExecutionEvidence {
                command: "cargo test --all-targets".to_string(),
                exit_code: 0,
                result_commitment: Blake3Hash::from_bytes(b"execution-log"),
            }),
            verification: Some(VerificationEvidence {
                command: "cargo clippy --all-targets -- -D warnings".to_string(),
                exit_code: 0,
                report_commitment: Blake3Hash::from_bytes(b"verification-log"),
            }),
            replay: Some(ReplayEvidence {
                command: "cargo test --all-targets".to_string(),
                environment_commitment: Blake3Hash::from_bytes(b"nightly-toolchain"),
            }),
            previous_receipt: None,
        }
    }

    #[test]
    fn alive_requires_execution_verification_and_replay() {
        let admitted = admitted_receipt();
        let authority = authority("repo:seanchatmangpt/affidavit");
        let receipt = certify_standing(&admitted, &authority, alive_observation())
            .expect("complete ALIVE evidence should certify");
        assert_eq!(receipt.standing, Standing::Alive);
        assert!(receipt.verify().is_ok());
    }

    #[test]
    fn alive_without_execution_is_refused_by_name() {
        let admitted = admitted_receipt();
        let authority = authority("repo:seanchatmangpt/affidavit");
        let mut observation = alive_observation();
        observation.execution = None;
        assert_eq!(
            certify_standing(&admitted, &authority, observation).unwrap_err(),
            StandingRefusal::AliveMissingExecution
        );
    }

    #[test]
    fn failed_verifier_cannot_crown_alive() {
        let admitted = admitted_receipt();
        let authority = authority("repo:seanchatmangpt/affidavit");
        let mut observation = alive_observation();
        observation.verification.as_mut().unwrap().exit_code = 1;
        assert_eq!(
            certify_standing(&admitted, &authority, observation).unwrap_err(),
            StandingRefusal::AliveVerificationFailed(1)
        );
    }

    #[test]
    fn unbounded_authority_is_refused_before_certification() {
        let admitted = admitted_receipt();
        let authority = authority("");
        let error = certify_standing(&admitted, &authority, alive_observation()).unwrap_err();
        assert_eq!(
            error,
            StandingRefusal::AuthorityEnvelopeRejected(vec![AuthorityRefusal::UnboundedScope])
        );
    }

    #[test]
    fn same_admitted_inputs_produce_same_receipt_hash() {
        let admitted = admitted_receipt();
        let authority = authority("repo:seanchatmangpt/affidavit");
        let a = certify_standing(&admitted, &authority, alive_observation()).unwrap();
        let b = certify_standing(&admitted, &authority, alive_observation()).unwrap();
        assert_eq!(a.receipt_hash, b.receipt_hash);
    }

    #[test]
    fn previous_receipt_changes_receipt_identity() {
        let admitted = admitted_receipt();
        let authority = authority("repo:seanchatmangpt/affidavit");
        let first = certify_standing(&admitted, &authority, alive_observation()).unwrap();
        let mut next = alive_observation();
        next.previous_receipt = Some(first.receipt_hash.clone());
        let second = certify_standing(&admitted, &authority, next).unwrap();
        assert_ne!(first.receipt_hash, second.receipt_hash);
        assert_eq!(second.previous_receipt, Some(first.receipt_hash));
    }

    #[test]
    fn deserialization_rejects_tampered_subject() {
        let admitted = admitted_receipt();
        let authority = authority("repo:seanchatmangpt/affidavit");
        let receipt = certify_standing(&admitted, &authority, alive_observation()).unwrap();
        let mut json = serde_json::to_value(&receipt).unwrap();
        json["subject"]["candidate"] = serde_json::Value::String("tampered".to_string());
        assert!(serde_json::from_value::<StandingReceipt>(json).is_err());
    }

    /// Digests must be canonically lowercase.
    ///
    /// `is_ascii_hexdigit` accepts `A-F`, so before v26.9.6 the same BLAKE3
    /// digest could be written two ways — and because the digest string is
    /// hashed into the receipt identity, the two spellings produced two
    /// different receipt hashes for the same evidence. That is a
    /// canonicalisation hole in a format whose whole value is that identical
    /// content has identical identity (ADR-5).
    ///
    /// No receipt this crate has ever produced is affected: `Blake3Hash` is
    /// built from `blake3::Hash::to_hex`, which is lowercase.
    #[test]
    fn an_uppercase_digest_is_refused_as_non_canonical() {
        let admitted = admitted_receipt();
        let mut observation = alive_observation();
        let upper = observation.observation_commitment.as_hex().to_uppercase();
        assert_ne!(
            upper,
            observation.observation_commitment.as_hex(),
            "the fixture digest must contain a-f for this test to mean anything"
        );
        observation.observation_commitment = Blake3Hash::from_hex(upper);

        assert!(
            matches!(
                certify_standing(&admitted, &authority("repo:affidavit"), observation),
                Err(StandingRefusal::MalformedBlake3(_))
            ),
            "an uppercase digest is not canonical and must be refused by name"
        );
    }

    /// The lowercase form of the same digest is admitted, so the rule is a
    /// canonicalisation constraint and not an accidental rejection of hex.
    #[test]
    fn the_lowercase_form_of_the_same_digest_is_admitted() {
        let admitted = admitted_receipt();
        let observation = alive_observation();
        assert!(
            certify_standing(&admitted, &authority("repo:affidavit"), observation).is_ok(),
            "canonical lowercase digests must still certify"
        );
    }
}
