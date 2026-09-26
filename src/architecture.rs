//! Deterministic enterprise-architecture qualification standing.
//!
//! This module certifies evidence bindings. It never grants consequential
//! execution authority; BRCE remains the exclusive DO boundary.
//!
//! Hardening invariants (v26.9.26):
//! - every digest is `algorithm:value` (non-empty, no whitespace); malformed
//!   digests are refused before a receipt exists;
//! - evidence and artifact digest sets are canonicalized (sorted, deduplicated)
//!   so duplicate delivery or reordering of the same evidence yields the same
//!   receipt digest, and a non-canonical receipt is refused on replay;
//! - replay checks the schema, the DO ceiling, the exact subject, the contract,
//!   the SBB, the standing and the self-digest;
//! - supersession requires an intact prior receipt and a genuinely different SBB,
//!   and the resulting chain link is verifiable with [`verify_chain`].
//!
//! [`verify_chain`]: ArchitectureQualificationReceipt::verify_chain

use serde::{Deserialize, Serialize};

/// Schema identifier bound into every architecture qualification receipt.
pub const ARCHITECTURE_RECEIPT_SCHEMA: &str = "affidavit.architecture-qualification.v1";

/// Standing that an architecture qualification receipt can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchitectureStanding {
    /// No admitted evidence; never certifiable.
    Unknown,
    /// Proposed but not yet qualified.
    Candidate,
    /// Qualified against the exact ABB/contract/SBB subject.
    Qualified,
    /// Qualification refused.
    Refused,
    /// Replaced by a successor SBB; chained to the prior receipt.
    Superseded,
}

/// Evidence receipt binding an ABB, contract and SBB to exact digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureQualificationReceipt {
    /// Schema identifier; must equal [`ARCHITECTURE_RECEIPT_SCHEMA`].
    pub schema: String,
    /// Architecture building block digest.
    pub abb_digest: String,
    /// Architecture contract digest.
    pub contract_digest: String,
    /// Solution building block digest.
    pub sbb_digest: String,
    /// Exact subject digest (e.g. commit or artifact identity).
    pub exact_subject_digest: String,
    /// Canonical (sorted, deduplicated) qualification evidence digests.
    pub qualification_evidence_digests: Vec<String>,
    /// Digest of the producer that emitted the evidence.
    pub producer_digest: String,
    /// Canonical (sorted, deduplicated) artifact digests.
    pub artifact_digests: Vec<String>,
    /// Standing carried by this receipt.
    pub standing: ArchitectureStanding,
    /// Always false: affidavit never confers DO authority.
    pub confers_do_authority: bool,
    /// Receipt digest of the superseded predecessor, if any.
    pub prior_receipt_digest: Option<String>,
    /// blake3 digest over the receipt with this field cleared.
    pub receipt_digest: String,
}

/// Typed refusals for architecture qualification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchitectureRefusal {
    /// Exact subject digest missing.
    MissingExactSubject,
    /// No qualification evidence.
    MissingEvidence,
    /// Attempt to certify `UNKNOWN` standing.
    UnknownPromotion,
    /// Contract digest differs from the expected one.
    MutableOrChangedContract,
    /// SBB digest differs from the expected one.
    MutableOrChangedSbb,
    /// ABB or subject digest belongs to a different subject.
    CrossSubjectReuse,
    /// Receipt claims DO authority.
    DoAuthorityForbidden,
    /// Recomputed digest or canonical form does not match.
    ReplayMismatch,
    /// A digest field is not `algorithm:value`.
    MalformedDigest {
        /// Field that carried the malformed digest.
        field: &'static str,
    },
    /// Schema identifier is not [`ARCHITECTURE_RECEIPT_SCHEMA`].
    SchemaMismatch,
    /// Supersession did not change the SBB.
    NotAReplacement,
    /// Chain link does not point at the given prior receipt.
    ChainBroken,
    /// Serialized receipt could not be parsed.
    Malformed,
}

impl core::fmt::Display for ArchitectureRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MalformedDigest { field } => write!(f, "MALFORMED_DIGEST[{field}]"),
            other => write!(f, "{other:?}"),
        }
    }
}

impl std::error::Error for ArchitectureRefusal {}

fn validate_digest(field: &'static str, value: &str) -> Result<(), ArchitectureRefusal> {
    let refused = ArchitectureRefusal::MalformedDigest { field };
    let Some((algorithm, body)) = value.split_once(':') else {
        return Err(refused);
    };
    let algorithm_ok = !algorithm.is_empty()
        && algorithm
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    let body_ok = !body.is_empty() && !body.chars().any(char::is_whitespace);
    if algorithm_ok && body_ok {
        Ok(())
    } else {
        Err(refused)
    }
}

fn canonical_set(
    field: &'static str,
    digests: Vec<String>,
) -> Result<Vec<String>, ArchitectureRefusal> {
    for digest in &digests {
        validate_digest(field, digest)?;
    }
    let mut digests = digests;
    digests.sort();
    digests.dedup();
    Ok(digests)
}

fn is_canonical(digests: &[String]) -> bool {
    digests.windows(2).all(|pair| pair[0] < pair[1])
}

impl ArchitectureQualificationReceipt {
    /// Certify a qualification receipt. Refuses malformed digests, missing
    /// subject/evidence and `UNKNOWN` promotion.
    #[allow(clippy::too_many_arguments)]
    pub fn certify(
        abb_digest: impl Into<String>,
        contract_digest: impl Into<String>,
        sbb_digest: impl Into<String>,
        exact_subject_digest: impl Into<String>,
        qualification_evidence_digests: Vec<String>,
        producer_digest: impl Into<String>,
        artifact_digests: Vec<String>,
        standing: ArchitectureStanding,
    ) -> Result<Self, ArchitectureRefusal> {
        let abb_digest = abb_digest.into();
        let contract_digest = contract_digest.into();
        let sbb_digest = sbb_digest.into();
        let exact_subject_digest = exact_subject_digest.into();
        let producer_digest = producer_digest.into();

        if exact_subject_digest.is_empty() {
            return Err(ArchitectureRefusal::MissingExactSubject);
        }
        if qualification_evidence_digests.is_empty() {
            return Err(ArchitectureRefusal::MissingEvidence);
        }
        if standing == ArchitectureStanding::Unknown {
            return Err(ArchitectureRefusal::UnknownPromotion);
        }
        validate_digest("abb_digest", &abb_digest)?;
        validate_digest("contract_digest", &contract_digest)?;
        validate_digest("sbb_digest", &sbb_digest)?;
        validate_digest("exact_subject_digest", &exact_subject_digest)?;
        validate_digest("producer_digest", &producer_digest)?;
        let qualification_evidence_digests = canonical_set(
            "qualification_evidence_digests",
            qualification_evidence_digests,
        )?;
        let artifact_digests = canonical_set("artifact_digests", artifact_digests)?;

        let mut receipt = Self {
            schema: ARCHITECTURE_RECEIPT_SCHEMA.to_string(),
            abb_digest,
            contract_digest,
            sbb_digest,
            exact_subject_digest,
            qualification_evidence_digests,
            producer_digest,
            artifact_digests,
            standing,
            confers_do_authority: false,
            prior_receipt_digest: None,
            receipt_digest: String::new(),
        };
        receipt.receipt_digest = digest_without_receipt(&receipt);
        Ok(receipt)
    }

    /// Verify the receipt's own integrity independent of any expected subject:
    /// schema, DO ceiling, standing, evidence presence, canonical form and
    /// self-digest.
    pub fn verify_integrity(&self) -> Result<(), ArchitectureRefusal> {
        if self.schema != ARCHITECTURE_RECEIPT_SCHEMA {
            return Err(ArchitectureRefusal::SchemaMismatch);
        }
        if self.confers_do_authority {
            return Err(ArchitectureRefusal::DoAuthorityForbidden);
        }
        if self.standing == ArchitectureStanding::Unknown {
            return Err(ArchitectureRefusal::UnknownPromotion);
        }
        if self.qualification_evidence_digests.is_empty() {
            return Err(ArchitectureRefusal::MissingEvidence);
        }
        if !is_canonical(&self.qualification_evidence_digests)
            || !is_canonical(&self.artifact_digests)
        {
            return Err(ArchitectureRefusal::ReplayMismatch);
        }
        if self.receipt_digest != digest_without_receipt(self) {
            return Err(ArchitectureRefusal::ReplayMismatch);
        }
        Ok(())
    }

    /// Replay the receipt against the expected exact ABB/contract/SBB/subject.
    pub fn verify_replay(
        &self,
        expected_abb_digest: &str,
        expected_contract_digest: &str,
        expected_sbb_digest: &str,
        expected_subject_digest: &str,
    ) -> Result<(), ArchitectureRefusal> {
        if self.schema != ARCHITECTURE_RECEIPT_SCHEMA {
            return Err(ArchitectureRefusal::SchemaMismatch);
        }
        if self.confers_do_authority {
            return Err(ArchitectureRefusal::DoAuthorityForbidden);
        }
        if self.abb_digest != expected_abb_digest
            || self.exact_subject_digest != expected_subject_digest
        {
            return Err(ArchitectureRefusal::CrossSubjectReuse);
        }
        if self.contract_digest != expected_contract_digest {
            return Err(ArchitectureRefusal::MutableOrChangedContract);
        }
        if self.sbb_digest != expected_sbb_digest {
            return Err(ArchitectureRefusal::MutableOrChangedSbb);
        }
        self.verify_integrity()
    }

    /// Record a successor SBB. The prior receipt must be intact and the SBB
    /// must actually change.
    pub fn supersede(
        &self,
        new_sbb_digest: impl Into<String>,
        new_subject_digest: impl Into<String>,
        evidence: Vec<String>,
    ) -> Result<Self, ArchitectureRefusal> {
        self.verify_integrity()?;
        let new_sbb_digest = new_sbb_digest.into();
        if new_sbb_digest == self.sbb_digest {
            return Err(ArchitectureRefusal::NotAReplacement);
        }
        let mut next = Self::certify(
            self.abb_digest.clone(),
            self.contract_digest.clone(),
            new_sbb_digest,
            new_subject_digest,
            evidence,
            self.producer_digest.clone(),
            self.artifact_digests.clone(),
            ArchitectureStanding::Superseded,
        )?;
        next.prior_receipt_digest = Some(self.receipt_digest.clone());
        next.receipt_digest = digest_without_receipt(&next);
        Ok(next)
    }

    /// Verify that `self` is an intact supersession link to an intact `prior`
    /// receipt of the same ABB and contract.
    pub fn verify_chain(&self, prior: &Self) -> Result<(), ArchitectureRefusal> {
        prior.verify_integrity()?;
        self.verify_integrity()?;
        if self.prior_receipt_digest.as_deref() != Some(prior.receipt_digest.as_str())
            || self.standing != ArchitectureStanding::Superseded
        {
            return Err(ArchitectureRefusal::ChainBroken);
        }
        if self.abb_digest != prior.abb_digest {
            return Err(ArchitectureRefusal::CrossSubjectReuse);
        }
        if self.contract_digest != prior.contract_digest {
            return Err(ArchitectureRefusal::MutableOrChangedContract);
        }
        if self.sbb_digest == prior.sbb_digest {
            return Err(ArchitectureRefusal::NotAReplacement);
        }
        Ok(())
    }

    /// Serialize to canonical JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("architecture receipt is serializable")
    }

    /// Parse a serialized receipt (unknown fields refused) and verify its
    /// integrity before returning it.
    pub fn from_json_verified(json: &str) -> Result<Self, ArchitectureRefusal> {
        let receipt: Self =
            serde_json::from_str(json).map_err(|_| ArchitectureRefusal::Malformed)?;
        receipt.verify_integrity()?;
        Ok(receipt)
    }
}

fn digest_without_receipt(receipt: &ArchitectureQualificationReceipt) -> String {
    let mut projection = receipt.clone();
    projection.receipt_digest.clear();
    let bytes = serde_json::to_vec(&projection).expect("architecture receipt is serializable");
    format!("blake3:{}", blake3::hash(&bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt() -> ArchitectureQualificationReceipt {
        ArchitectureQualificationReceipt::certify(
            "sha256:abb",
            "sha256:contract",
            "sha256:sbb",
            "sha256:subject",
            vec!["sha256:evidence".into()],
            "sha256:producer",
            vec!["sha256:artifact".into()],
            ArchitectureStanding::Qualified,
        )
        .unwrap()
    }

    #[test]
    fn qualification_receipt_is_deterministic_and_non_authoritative() {
        let first = receipt();
        let second = receipt();
        assert_eq!(first, second);
        assert!(!first.confers_do_authority);
        first
            .verify_replay(
                "sha256:abb",
                "sha256:contract",
                "sha256:sbb",
                "sha256:subject",
            )
            .unwrap();
    }

    #[test]
    fn changed_contract_sbb_and_cross_subject_reuse_are_refused() {
        let receipt = receipt();

        assert_eq!(
            receipt.verify_replay(
                "sha256:abb",
                "sha256:changed",
                "sha256:sbb",
                "sha256:subject"
            ),
            Err(ArchitectureRefusal::MutableOrChangedContract)
        );
        assert_eq!(
            receipt.verify_replay(
                "sha256:abb",
                "sha256:contract",
                "sha256:changed",
                "sha256:subject"
            ),
            Err(ArchitectureRefusal::MutableOrChangedSbb)
        );
        assert_eq!(
            receipt.verify_replay(
                "sha256:other",
                "sha256:contract",
                "sha256:sbb",
                "sha256:other-subject"
            ),
            Err(ArchitectureRefusal::CrossSubjectReuse)
        );
    }

    #[test]
    fn unknown_and_missing_evidence_cannot_acquire_standing() {
        assert_eq!(
            ArchitectureQualificationReceipt::certify(
                "sha256:abb",
                "sha256:contract",
                "sha256:sbb",
                "sha256:subject",
                vec!["sha256:evidence".into()],
                "sha256:producer",
                vec![],
                ArchitectureStanding::Unknown
            ),
            Err(ArchitectureRefusal::UnknownPromotion)
        );

        assert_eq!(
            ArchitectureQualificationReceipt::certify(
                "sha256:abb",
                "sha256:contract",
                "sha256:sbb",
                "sha256:subject",
                vec![],
                "sha256:producer",
                vec![],
                ArchitectureStanding::Qualified
            ),
            Err(ArchitectureRefusal::MissingEvidence)
        );
    }

    #[test]
    fn replacement_chains_supersession_to_the_prior_exact_receipt() {
        let current = receipt();
        let next = current
            .supersede(
                "sha256:sbb-b",
                "sha256:subject-b",
                vec!["sha256:evidence-b".into()],
            )
            .unwrap();

        assert_eq!(next.standing, ArchitectureStanding::Superseded);
        assert_eq!(
            next.prior_receipt_digest.as_deref(),
            Some(current.receipt_digest.as_str())
        );
        assert!(!next.confers_do_authority);
        next.verify_chain(&current).unwrap();
    }
}
