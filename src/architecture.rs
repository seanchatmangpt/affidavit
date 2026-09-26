//! Deterministic enterprise-architecture qualification standing.
//!
//! This module certifies evidence bindings. It never grants consequential
//! execution authority; BRCE remains the exclusive DO boundary.

use serde::{Deserialize, Serialize};

const SCHEMA: &str = "affidavit.architecture-qualification.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchitectureStanding {
    Unknown,
    Candidate,
    Qualified,
    Refused,
    Superseded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureQualificationReceipt {
    pub schema: String,
    pub abb_digest: String,
    pub contract_digest: String,
    pub sbb_digest: String,
    pub exact_subject_digest: String,
    pub qualification_evidence_digests: Vec<String>,
    pub producer_digest: String,
    pub artifact_digests: Vec<String>,
    pub standing: ArchitectureStanding,
    pub confers_do_authority: bool,
    pub prior_receipt_digest: Option<String>,
    pub receipt_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchitectureRefusal {
    MissingExactSubject,
    MissingEvidence,
    UnknownPromotion,
    MutableOrChangedContract,
    MutableOrChangedSbb,
    CrossSubjectReuse,
    DoAuthorityForbidden,
    ReplayMismatch,
}

impl ArchitectureQualificationReceipt {
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

        let mut receipt = Self {
            schema: SCHEMA.to_string(),
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

    pub fn verify_replay(
        &self,
        expected_abb_digest: &str,
        expected_contract_digest: &str,
        expected_sbb_digest: &str,
        expected_subject_digest: &str,
    ) -> Result<(), ArchitectureRefusal> {
        if self.confers_do_authority {
            return Err(ArchitectureRefusal::DoAuthorityForbidden);
        }
        if self.abb_digest != expected_abb_digest || self.exact_subject_digest != expected_subject_digest {
            return Err(ArchitectureRefusal::CrossSubjectReuse);
        }
        if self.contract_digest != expected_contract_digest {
            return Err(ArchitectureRefusal::MutableOrChangedContract);
        }
        if self.sbb_digest != expected_sbb_digest {
            return Err(ArchitectureRefusal::MutableOrChangedSbb);
        }
        if self.receipt_digest != digest_without_receipt(self) {
            return Err(ArchitectureRefusal::ReplayMismatch);
        }
        Ok(())
    }

    pub fn supersede(
        &self,
        new_sbb_digest: impl Into<String>,
        new_subject_digest: impl Into<String>,
        evidence: Vec<String>,
    ) -> Result<Self, ArchitectureRefusal> {
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
    fn unknown_and missing_evidence_cannot acquire standing() {
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
        assert_eq!(next.prior_receipt_digest.as_deref(), Some(current.receipt_digest.as_str()));
        assert!(!next.confers_do_authority);
    }
}
