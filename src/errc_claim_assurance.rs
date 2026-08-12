//! Claim-assurance refinement for formal ERRC receipts.
//!
//! `affidavit/errc/v1` proves that each declared before/after measurement lies
//! in the claimed ERRC relation. This module adds the stronger evidence-shape
//! required by the mature `ggen-legacy` claims register: every widened claim
//! must have a named verifier, exact evidence locator, observed result,
//! cryptographic evidence commitment, explicit exclusions, and a claim ceiling.
//!
//! The relation between an [`ErrcReceipt`] and its witnesses is a finite
//! bijection by claim id. A missing witness, an extra witness, or two witnesses
//! for one claim is refused. This certifies evidence completeness and binding;
//! it does not establish causal truth, production readiness, compliance,
//! utility, optimality, or actuation authority.

use crate::errc::{ErrcReceipt, ErrcRefusal, ErrcSource, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY};
use crate::types::Blake3Hash;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

/// Stable claim-assurance refinement profile.
pub const ERRC_CLAIM_ASSURANCE_PROFILE: &str = "affidavit/errc-claim-assurance/v1";
/// Exact mature ggen-legacy claims-register source artifact.
pub const ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT: &str = "governance/claims-register.md";
/// Maximum semantic claim supported by this refinement receipt.
pub const ERRC_CLAIM_ASSURANCE_CEILING: &str =
    "CLAIM_WITNESS_COMPLETENESS_AND_BINDING_ONLY_NO_TRUTH_CAUSALITY_PRODUCTION_COMPLIANCE_OPTIMALITY_OR_ACTUATION_CLAIM";

/// Complete witness required for one widened ERRC claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrcClaimWitness {
    /// Existing claim id in the parent ERRC receipt.
    pub claim_id: String,
    /// Exact verifier command, verifier id, or deterministic verifier coordinate.
    pub verifier: String,
    /// Exact evidence path/URI/artifact coordinate inspected by the verifier.
    pub evidence_locator: String,
    /// Observed verifier result. This is evidence, not an inferred conclusion.
    pub observed_result: String,
    /// BLAKE3 commitment to the evidence bytes/result carrier.
    pub evidence_commitment: Blake3Hash,
    /// Explicit statements of what this evidence does not establish.
    pub exclusions: Vec<String>,
}

/// Sealed one-to-one assurance ledger over a parent [`ErrcReceipt`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::manual_non_exhaustive)]
pub struct ErrcClaimAssuranceReceipt {
    /// Stable refinement profile.
    pub profile: String,
    /// Exact archaeological source of the widened-claim completeness rule.
    pub source: ErrcSource,
    /// Fixed semantic claim ceiling.
    pub claim_ceiling: String,
    /// Exact BLAKE3 identity of the parent ERRC receipt.
    pub errc_receipt_hash: Blake3Hash,
    /// Canonical claim-id set copied from the verified parent.
    pub claim_ids: Vec<String>,
    /// Canonically ordered, exactly-one-per-claim evidence witnesses.
    pub witnesses: Vec<ErrcClaimWitness>,
    /// Canonical content hash of this assurance receipt.
    pub receipt_hash: Blake3Hash,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal at the claim-assurance admission boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrcClaimAssuranceRefusal {
    /// Parent ERRC receipt did not verify.
    ParentInvalid(ErrcRefusal),
    /// Required witness text was empty.
    EmptyWitnessField {
        /// Claim id owning the malformed witness.
        claim_id: String,
        /// Field name.
        field: &'static str,
    },
    /// Evidence commitment is not a canonical BLAKE3 hex digest.
    MalformedEvidenceCommitment(String),
    /// A claim has no explicit exclusions.
    NoExclusions(String),
    /// An exclusion statement was empty.
    EmptyExclusion(String),
    /// A claim repeats the same exclusion.
    DuplicateExclusion {
        /// Claim id.
        claim_id: String,
        /// Repeated exclusion text.
        exclusion: String,
    },
    /// Parent contains no claims to assure.
    NoClaims,
    /// A parent claim has no witness.
    MissingWitness(String),
    /// A witness refers to no parent claim.
    UnexpectedWitness(String),
    /// More than one witness exists for the same claim.
    DuplicateWitness(String),
    /// Serialized set order is not canonical.
    NonCanonicalOrder(&'static str),
    /// Stable profile changed.
    WrongProfile,
    /// Pinned ggen-legacy source changed.
    WrongSource,
    /// Semantic claim ceiling changed or broadened.
    WrongClaimCeiling,
    /// Receipt was checked against a different valid parent ERRC receipt.
    ParentHashMismatch,
    /// Stored claim-id set does not equal the verified parent's claim set.
    ClaimSetMismatch,
    /// Stored content hash does not match canonical material.
    ReceiptHashMismatch,
    /// Canonical serialization failed.
    Serialization(String),
}

impl core::fmt::Display for ErrcClaimAssuranceRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ParentInvalid(reason) => write!(f, "errc_claim_parent_invalid: {reason}"),
            Self::EmptyWitnessField { claim_id, field } => {
                write!(f, "errc_claim_empty_witness_field: claim={claim_id} field={field}")
            }
            Self::MalformedEvidenceCommitment(id) => {
                write!(f, "errc_claim_malformed_evidence_commitment: {id}")
            }
            Self::NoExclusions(id) => write!(f, "errc_claim_no_exclusions: {id}"),
            Self::EmptyExclusion(id) => write!(f, "errc_claim_empty_exclusion: {id}"),
            Self::DuplicateExclusion { claim_id, exclusion } => write!(
                f,
                "errc_claim_duplicate_exclusion: claim={claim_id} exclusion={exclusion}"
            ),
            Self::NoClaims => f.write_str("errc_claim_assurance_no_claims"),
            Self::MissingWitness(id) => write!(f, "errc_claim_missing_witness: {id}"),
            Self::UnexpectedWitness(id) => write!(f, "errc_claim_unexpected_witness: {id}"),
            Self::DuplicateWitness(id) => write!(f, "errc_claim_duplicate_witness: {id}"),
            Self::NonCanonicalOrder(field) => {
                write!(f, "errc_claim_non_canonical_order: {field}")
            }
            Self::WrongProfile => f.write_str("wrong_errc_claim_assurance_profile"),
            Self::WrongSource => f.write_str("wrong_errc_claim_assurance_source"),
            Self::WrongClaimCeiling => f.write_str("wrong_errc_claim_assurance_ceiling"),
            Self::ParentHashMismatch => f.write_str("errc_claim_parent_hash_mismatch"),
            Self::ClaimSetMismatch => f.write_str("errc_claim_set_mismatch"),
            Self::ReceiptHashMismatch => f.write_str("errc_claim_assurance_hash_mismatch"),
            Self::Serialization(reason) => write!(f, "errc_claim_assurance_serialization: {reason}"),
        }
    }
}

impl std::error::Error for ErrcClaimAssuranceRefusal {}

#[derive(Serialize)]
struct AssuranceMaterial<'a> {
    profile: &'a str,
    source: &'a ErrcSource,
    claim_ceiling: &'a str,
    errc_receipt_hash: &'a Blake3Hash,
    claim_ids: &'a [String],
    witnesses: &'a [ErrcClaimWitness],
}

impl ErrcClaimAssuranceReceipt {
    /// Verify canonical structure and content identity without loading the parent.
    ///
    /// Use [`Self::verify_against`] when the parent receipt is available; that
    /// additionally proves parent identity and exact claim-set correspondence.
    pub fn verify(&self) -> Result<(), ErrcClaimAssuranceRefusal> {
        validate_fixed_fields(&self.profile, &self.source, &self.claim_ceiling)?;
        require_blake3(&self.errc_receipt_hash, "parent")?;
        validate_bijection(&self.claim_ids, &self.witnesses, true)?;
        let expected = compute_receipt_hash(
            &self.profile,
            &self.source,
            &self.claim_ceiling,
            &self.errc_receipt_hash,
            &self.claim_ids,
            &self.witnesses,
        )?;
        if expected != self.receipt_hash {
            return Err(ErrcClaimAssuranceRefusal::ReceiptHashMismatch);
        }
        Ok(())
    }

    /// Verify this assurance receipt against the exact parent ERRC receipt.
    pub fn verify_against(
        &self,
        parent: &ErrcReceipt,
    ) -> Result<(), ErrcClaimAssuranceRefusal> {
        self.verify()?;
        parent
            .verify()
            .map_err(ErrcClaimAssuranceRefusal::ParentInvalid)?;
        if self.errc_receipt_hash != parent.receipt_hash {
            return Err(ErrcClaimAssuranceRefusal::ParentHashMismatch);
        }
        let mut expected_ids: Vec<String> = parent.claims.iter().map(|c| c.id.clone()).collect();
        expected_ids.sort();
        if expected_ids != self.claim_ids {
            return Err(ErrcClaimAssuranceRefusal::ClaimSetMismatch);
        }
        Ok(())
    }
}

/// Certify a complete one-to-one witness ledger for every claim in `parent`.
///
/// This refinement is still CONSTRUCT only. It proves evidence-shape
/// completeness and cryptographic binding, not truth, causality, utility,
/// production readiness, compliance, optimality, or actuation authority.
pub fn certify_errc_claim_assurance(
    parent: &ErrcReceipt,
    mut witnesses: Vec<ErrcClaimWitness>,
) -> Result<ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal> {
    parent
        .verify()
        .map_err(ErrcClaimAssuranceRefusal::ParentInvalid)?;

    for witness in &mut witnesses {
        witness.exclusions.sort();
    }
    witnesses.sort_by(|a, b| a.claim_id.cmp(&b.claim_id));

    let mut claim_ids: Vec<String> = parent.claims.iter().map(|c| c.id.clone()).collect();
    claim_ids.sort();
    validate_bijection(&claim_ids, &witnesses, true)?;

    let source = ErrcSource {
        repository: ERRC_SOURCE_REPOSITORY.to_string(),
        commit: ERRC_SOURCE_COMMIT.to_string(),
        artifact: ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT.to_string(),
    };
    validate_fixed_fields(
        ERRC_CLAIM_ASSURANCE_PROFILE,
        &source,
        ERRC_CLAIM_ASSURANCE_CEILING,
    )?;
    let receipt_hash = compute_receipt_hash(
        ERRC_CLAIM_ASSURANCE_PROFILE,
        &source,
        ERRC_CLAIM_ASSURANCE_CEILING,
        &parent.receipt_hash,
        &claim_ids,
        &witnesses,
    )?;

    Ok(ErrcClaimAssuranceReceipt {
        profile: ERRC_CLAIM_ASSURANCE_PROFILE.to_string(),
        source,
        claim_ceiling: ERRC_CLAIM_ASSURANCE_CEILING.to_string(),
        errc_receipt_hash: parent.receipt_hash.clone(),
        claim_ids,
        witnesses,
        receipt_hash,
        _seal: (),
    })
}

fn validate_fixed_fields(
    profile: &str,
    source: &ErrcSource,
    claim_ceiling: &str,
) -> Result<(), ErrcClaimAssuranceRefusal> {
    if profile != ERRC_CLAIM_ASSURANCE_PROFILE {
        return Err(ErrcClaimAssuranceRefusal::WrongProfile);
    }
    if source.repository != ERRC_SOURCE_REPOSITORY
        || source.commit != ERRC_SOURCE_COMMIT
        || source.artifact != ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT
    {
        return Err(ErrcClaimAssuranceRefusal::WrongSource);
    }
    if claim_ceiling != ERRC_CLAIM_ASSURANCE_CEILING {
        return Err(ErrcClaimAssuranceRefusal::WrongClaimCeiling);
    }
    Ok(())
}

fn validate_bijection(
    claim_ids: &[String],
    witnesses: &[ErrcClaimWitness],
    require_canonical_order: bool,
) -> Result<(), ErrcClaimAssuranceRefusal> {
    if claim_ids.is_empty() {
        return Err(ErrcClaimAssuranceRefusal::NoClaims);
    }
    if require_canonical_order {
        if claim_ids.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(ErrcClaimAssuranceRefusal::NonCanonicalOrder("claim_ids"));
        }
        if witnesses
            .windows(2)
            .any(|pair| pair[0].claim_id > pair[1].claim_id)
        {
            return Err(ErrcClaimAssuranceRefusal::NonCanonicalOrder("witnesses"));
        }
    }

    let expected: BTreeSet<&str> = claim_ids.iter().map(String::as_str).collect();
    let mut observed = BTreeSet::new();
    for witness in witnesses {
        require_witness_text(witness, "claim_id", &witness.claim_id)?;
        require_witness_text(witness, "verifier", &witness.verifier)?;
        require_witness_text(witness, "evidence_locator", &witness.evidence_locator)?;
        require_witness_text(witness, "observed_result", &witness.observed_result)?;
        require_blake3(&witness.evidence_commitment, &witness.claim_id)?;

        if !observed.insert(witness.claim_id.as_str()) {
            return Err(ErrcClaimAssuranceRefusal::DuplicateWitness(
                witness.claim_id.clone(),
            ));
        }
        if !expected.contains(witness.claim_id.as_str()) {
            return Err(ErrcClaimAssuranceRefusal::UnexpectedWitness(
                witness.claim_id.clone(),
            ));
        }
        if witness.exclusions.is_empty() {
            return Err(ErrcClaimAssuranceRefusal::NoExclusions(
                witness.claim_id.clone(),
            ));
        }
        if require_canonical_order
            && witness
                .exclusions
                .windows(2)
                .any(|pair| pair[0] > pair[1])
        {
            return Err(ErrcClaimAssuranceRefusal::NonCanonicalOrder(
                "witness.exclusions",
            ));
        }
        let mut exclusions = BTreeSet::new();
        for exclusion in &witness.exclusions {
            if exclusion.trim().is_empty() {
                return Err(ErrcClaimAssuranceRefusal::EmptyExclusion(
                    witness.claim_id.clone(),
                ));
            }
            if !exclusions.insert(exclusion.as_str()) {
                return Err(ErrcClaimAssuranceRefusal::DuplicateExclusion {
                    claim_id: witness.claim_id.clone(),
                    exclusion: exclusion.clone(),
                });
            }
        }
    }

    for claim_id in claim_ids {
        if !observed.contains(claim_id.as_str()) {
            return Err(ErrcClaimAssuranceRefusal::MissingWitness(claim_id.clone()));
        }
    }
    Ok(())
}

fn require_witness_text(
    witness: &ErrcClaimWitness,
    field: &'static str,
    value: &str,
) -> Result<(), ErrcClaimAssuranceRefusal> {
    if value.trim().is_empty() {
        Err(ErrcClaimAssuranceRefusal::EmptyWitnessField {
            claim_id: witness.claim_id.clone(),
            field,
        })
    } else {
        Ok(())
    }
}

fn require_blake3(
    hash: &Blake3Hash,
    coordinate: &str,
) -> Result<(), ErrcClaimAssuranceRefusal> {
    let hex = hash.as_hex();
    if hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ErrcClaimAssuranceRefusal::MalformedEvidenceCommitment(
            coordinate.to_string(),
        ))
    }
}

fn compute_receipt_hash(
    profile: &str,
    source: &ErrcSource,
    claim_ceiling: &str,
    errc_receipt_hash: &Blake3Hash,
    claim_ids: &[String],
    witnesses: &[ErrcClaimWitness],
) -> Result<Blake3Hash, ErrcClaimAssuranceRefusal> {
    let material = AssuranceMaterial {
        profile,
        source,
        claim_ceiling,
        errc_receipt_hash,
        claim_ids,
        witnesses,
    };
    let canonical = serde_json::to_vec(&material)
        .map_err(|error| ErrcClaimAssuranceRefusal::Serialization(error.to_string()))?;
    let mut bytes = Vec::with_capacity(profile.len() + canonical.len() + 1);
    bytes.extend_from_slice(profile.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&canonical);
    Ok(Blake3Hash::from_bytes(&bytes))
}

impl<'de> Deserialize<'de> for ErrcClaimAssuranceReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct Raw {
            profile: String,
            source: ErrcSource,
            claim_ceiling: String,
            errc_receipt_hash: Blake3Hash,
            claim_ids: Vec<String>,
            witnesses: Vec<ErrcClaimWitness>,
            receipt_hash: Blake3Hash,
        }

        let raw = Raw::deserialize(deserializer)?;
        let receipt = Self {
            profile: raw.profile,
            source: raw.source,
            claim_ceiling: raw.claim_ceiling,
            errc_receipt_hash: raw.errc_receipt_hash,
            claim_ids: raw.claim_ids,
            witnesses: raw.witnesses,
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
    use crate::errc::{
        certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, PreservedInvariant,
    };
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::standing::{ReplayEvidence, SubjectIdentity};

    fn parent_receipt(candidate: &str) -> ErrcReceipt {
        let mut assembler = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "errc-assurance-test",
            vec![object_ref("repo", "git")],
            b"claim-assurance-observation",
            &mut counter,
        )
        .expect("event");
        assembler.append(event).expect("append");
        let admitted = crate::admission::admit(assembler.finalize()).expect("admitted");

        let claims = vec![
            ErrcClaim {
                id: "eliminate-stub".to_string(),
                target: "dependency-stub".to_string(),
                quadrant: ErrcQuadrant::Eliminate,
                measure: ErrcMeasure {
                    metric: "stub_count".to_string(),
                    unit: "count".to_string(),
                    baseline: 1,
                    candidate: 0,
                },
                evidence_commitment: Blake3Hash::from_bytes(b"stub-evidence"),
            },
            ErrcClaim {
                id: "raise-replay".to_string(),
                target: "replay-identity".to_string(),
                quadrant: ErrcQuadrant::Raise,
                measure: ErrcMeasure {
                    metric: "bound_coordinates".to_string(),
                    unit: "count".to_string(),
                    baseline: 1,
                    candidate: 4,
                },
                evidence_commitment: Blake3Hash::from_bytes(b"replay-evidence"),
            },
        ];

        certify_errc(
            &admitted,
            ErrcObservation {
                subject: SubjectIdentity {
                    subject: "seanchatmangpt/affidavit".to_string(),
                    base: "base-sha".to_string(),
                    tree: "tree-sha".to_string(),
                    candidate: candidate.to_string(),
                },
                observation_commitment: Blake3Hash::from_bytes(candidate.as_bytes()),
                claims,
                preserved_invariants: vec![PreservedInvariant {
                    id: "certify-dont-decide".to_string(),
                    statement: "No evidence object gains ambient decision authority.".to_string(),
                    evidence_commitment: Blake3Hash::from_bytes(b"fence"),
                }],
                replay: ReplayEvidence {
                    command: "cargo test errc_claim_assurance --lib".to_string(),
                    environment_commitment: Blake3Hash::from_bytes(b"toolchain"),
                },
                previous_receipt: None,
            },
        )
        .expect("parent ERRC receipt")
    }

    fn witness(claim_id: &str) -> ErrcClaimWitness {
        ErrcClaimWitness {
            claim_id: claim_id.to_string(),
            verifier: format!("verifier:{claim_id}"),
            evidence_locator: format!("evidence/{claim_id}.json"),
            observed_result: "exit=0; relation=observed".to_string(),
            evidence_commitment: Blake3Hash::from_bytes(claim_id.as_bytes()),
            exclusions: vec![
                "does not establish causality".to_string(),
                "does not establish production readiness".to_string(),
            ],
        }
    }

    #[test]
    fn exact_one_to_one_claim_witness_bijection_certifies() {
        let parent = parent_receipt("candidate-a");
        let receipt = certify_errc_claim_assurance(
            &parent,
            vec![witness("raise-replay"), witness("eliminate-stub")],
        )
        .expect("complete witness ledger");
        assert!(receipt.verify_against(&parent).is_ok());
        assert_eq!(receipt.claim_ids.len(), receipt.witnesses.len());
    }

    #[test]
    fn missing_witness_is_refused() {
        let parent = parent_receipt("candidate-a");
        assert_eq!(
            certify_errc_claim_assurance(&parent, vec![witness("eliminate-stub")]).unwrap_err(),
            ErrcClaimAssuranceRefusal::MissingWitness("raise-replay".to_string())
        );
    }

    #[test]
    fn unexpected_witness_is_refused() {
        let parent = parent_receipt("candidate-a");
        assert_eq!(
            certify_errc_claim_assurance(
                &parent,
                vec![
                    witness("eliminate-stub"),
                    witness("raise-replay"),
                    witness("invented-claim"),
                ],
            )
            .unwrap_err(),
            ErrcClaimAssuranceRefusal::UnexpectedWitness("invented-claim".to_string())
        );
    }

    #[test]
    fn exclusions_are_mandatory() {
        let parent = parent_receipt("candidate-a");
        let mut incomplete = witness("eliminate-stub");
        incomplete.exclusions.clear();
        assert_eq!(
            certify_errc_claim_assurance(
                &parent,
                vec![incomplete, witness("raise-replay")],
            )
            .unwrap_err(),
            ErrcClaimAssuranceRefusal::NoExclusions("eliminate-stub".to_string())
        );
    }

    #[test]
    fn canonicalization_makes_input_order_irrelevant() {
        let parent = parent_receipt("candidate-a");
        let a = certify_errc_claim_assurance(
            &parent,
            vec![witness("eliminate-stub"), witness("raise-replay")],
        )
        .expect("a");
        let b = certify_errc_claim_assurance(
            &parent,
            vec![witness("raise-replay"), witness("eliminate-stub")],
        )
        .expect("b");
        assert_eq!(a.receipt_hash, b.receipt_hash);
    }

    #[test]
    fn assurance_is_bound_to_the_exact_parent_receipt() {
        let parent_a = parent_receipt("candidate-a");
        let parent_b = parent_receipt("candidate-b");
        let receipt = certify_errc_claim_assurance(
            &parent_a,
            vec![witness("eliminate-stub"), witness("raise-replay")],
        )
        .expect("receipt");
        assert_eq!(
            receipt.verify_against(&parent_b).unwrap_err(),
            ErrcClaimAssuranceRefusal::ParentHashMismatch
        );
    }

    #[test]
    fn tampered_serialized_assurance_is_rejected() {
        let parent = parent_receipt("candidate-a");
        let receipt = certify_errc_claim_assurance(
            &parent,
            vec![witness("eliminate-stub"), witness("raise-replay")],
        )
        .expect("receipt");
        let mut json = serde_json::to_value(&receipt).expect("json");
        json["witnesses"][0]["observed_result"] =
            serde_json::Value::String("fabricated".to_string());
        assert!(serde_json::from_value::<ErrcClaimAssuranceReceipt>(json).is_err());
    }

    #[test]
    fn ggen_claims_register_lineage_is_exact() {
        let parent = parent_receipt("candidate-a");
        let receipt = certify_errc_claim_assurance(
            &parent,
            vec![witness("eliminate-stub"), witness("raise-replay")],
        )
        .expect("receipt");
        assert_eq!(receipt.source.repository, ERRC_SOURCE_REPOSITORY);
        assert_eq!(receipt.source.commit, ERRC_SOURCE_COMMIT);
        assert_eq!(
            receipt.source.artifact,
            ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT
        );
        assert_eq!(receipt.claim_ceiling, ERRC_CLAIM_ASSURANCE_CEILING);
    }
}
