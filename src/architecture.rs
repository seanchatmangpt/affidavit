//! Deterministic enterprise-architecture qualification standing.
//!
//! This module certifies evidence bindings. It never grants consequential
//! execution authority; BRCE remains the exclusive DO boundary.
//!
//! Hardening invariants (v26.9.26, schema v2):
//! - every digest is `algorithm:value` (lowercase algorithm, printable-ASCII
//!   body with no whitespace or invisible characters). Malformed digests are
//!   refused by [`certify`] *and* re-checked by [`verify_integrity`], so a
//!   resealed receipt carrying a malformed or empty digest is refused on
//!   replay and on [`from_json_verified`];
//! - evidence and artifact digest sets are canonicalized (sorted, deduplicated)
//!   so duplicate delivery or reordering of the same evidence yields the same
//!   receipt digest, and a non-canonical receipt is refused on replay;
//! - standing and chain fields must agree: `SUPERSEDED` exists only as a
//!   retirement record carrying both `prior_receipt_digest` (the retired
//!   receipt) and `superseded_by_receipt_digest` (its successor); `CANDIDATE`
//!   and `REFUSED` carry no chain fields; `QUALIFIED` may carry only
//!   `prior_receipt_digest` (it is then the successor of that receipt);
//! - supersession ([`supersede`]) yields a QUALIFIED successor for the new SBB
//!   and a SUPERSEDED retirement record for the replaced SBB, so a consumer
//!   selecting `QUALIFIED` by ABB gets the current SBB, never the stale one;
//! - [`ArchitectureStandingLedger`] admits receipts, refuses orphan or forked
//!   chain links, and answers "current qualified SBB by ABB" as
//!   machine-readable JSON.
//!
//! Integrity is not authenticity: `receipt_digest` is an unkeyed blake3
//! self-digest, so anyone may recompute it after editing fields. Forged
//! evidence is refused by re-deriving evidence digests from the observed
//! evidence bytes ([`QualificationEvidence`], [`verify_evidence`]); producer
//! authentication (signatures/keys) is out of scope for this module.
//!
//! [`certify`]: ArchitectureQualificationReceipt::certify
//! [`verify_integrity`]: ArchitectureQualificationReceipt::verify_integrity
//! [`from_json_verified`]: ArchitectureQualificationReceipt::from_json_verified
//! [`supersede`]: ArchitectureQualificationReceipt::supersede
//! [`verify_evidence`]: ArchitectureQualificationReceipt::verify_evidence

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema identifier bound into every architecture qualification receipt.
pub const ARCHITECTURE_RECEIPT_SCHEMA: &str = "affidavit.architecture-qualification.v2";

/// Schema identifier of the machine-readable standing query answer.
pub const ARCHITECTURE_QUERY_SCHEMA: &str = "affidavit.architecture-standing-query.v1";

/// Standing that an architecture qualification receipt can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    /// Retirement record of a receipt whose SBB was replaced by a successor
    /// SBB; chained to both the retired receipt and its successor.
    Superseded,
}

/// Independent evidence producer family (charter DoD item 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceSource {
    /// autofde-lab qualification runs.
    AutofdeLab,
    /// xaas conformance/process observations.
    Xaas,
    /// Runtime/process observations of the deployed SBB.
    Runtime,
}

/// One piece of independent qualification evidence, bound to the exact bytes
/// that were observed. Its [`binding_digest`] is what a receipt records.
///
/// [`binding_digest`]: QualificationEvidence::binding_digest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationEvidence {
    /// Producer family.
    pub source: EvidenceSource,
    /// Digest of the producer that emitted the evidence.
    pub producer_digest: String,
    /// `blake3:` digest of the observed evidence bytes.
    pub content_digest: String,
}

impl QualificationEvidence {
    /// Observe evidence bytes: the content digest is derived from the bytes,
    /// never asserted by the caller.
    pub fn observe(
        source: EvidenceSource,
        producer_digest: impl Into<String>,
        bytes: &[u8],
    ) -> Result<Self, ArchitectureRefusal> {
        let producer_digest = producer_digest.into();
        validate_digest("evidence.producer_digest", &producer_digest)?;
        if bytes.is_empty() {
            return Err(ArchitectureRefusal::MissingEvidence);
        }
        Ok(Self {
            source,
            producer_digest,
            content_digest: blake3_digest(bytes),
        })
    }

    /// Digest binding source, producer and content; recorded in receipts.
    pub fn binding_digest(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("evidence is serializable");
        blake3_digest(&bytes)
    }

    /// Refuse unless `bytes` are exactly the observed evidence.
    pub fn verify_bytes(&self, bytes: &[u8]) -> Result<(), ArchitectureRefusal> {
        validate_digest("evidence.producer_digest", &self.producer_digest)?;
        if bytes.is_empty() || self.content_digest != blake3_digest(bytes) {
            return Err(ArchitectureRefusal::ForgedEvidence);
        }
        Ok(())
    }
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
    /// For a QUALIFIED successor: the receipt it replaces. For a SUPERSEDED
    /// retirement record: the receipt being retired.
    pub prior_receipt_digest: Option<String>,
    /// For a SUPERSEDED retirement record: the successor receipt.
    pub superseded_by_receipt_digest: Option<String>,
    /// blake3 digest over the receipt with this field cleared.
    pub receipt_digest: String,
}

/// Result of replacing an SBB: the QUALIFIED successor and the SUPERSEDED
/// retirement record of the replaced receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Supersession {
    /// SUPERSEDED record retiring the prior receipt.
    pub retired: ArchitectureQualificationReceipt,
    /// QUALIFIED receipt for the replacement SBB.
    pub successor: ArchitectureQualificationReceipt,
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
    /// Standing and chain fields disagree (e.g. SUPERSEDED without a chain,
    /// or a chain field on a CANDIDATE/REFUSED receipt).
    StandingChainMismatch,
    /// Only a QUALIFIED receipt can be superseded.
    NotSupersedable,
    /// The receipt was already superseded by a different successor.
    AlreadySuperseded,
    /// A chain link references a receipt absent from the ledger.
    OrphanChainLink,
    /// Evidence digests do not match the observed evidence bytes.
    ForgedEvidence,
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

fn blake3_digest(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn split_digest(value: &str) -> Option<(&str, &str)> {
    let (algorithm, body) = value.split_once(':')?;
    let algorithm_ok = !algorithm.is_empty()
        && algorithm
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    // Printable ASCII only: rejects whitespace, control characters and
    // invisible Unicode such as U+200B.
    let body_ok = !body.is_empty() && body.bytes().all(|b| b.is_ascii_graphic());
    (algorithm_ok && body_ok).then_some((algorithm, body))
}

fn validate_digest(field: &'static str, value: &str) -> Result<(), ArchitectureRefusal> {
    split_digest(value)
        .map(|_| ())
        .ok_or(ArchitectureRefusal::MalformedDigest { field })
}

/// A receipt link must be a well-formed `blake3:<64 lowercase hex>` digest.
fn validate_receipt_link(field: &'static str, value: &str) -> Result<(), ArchitectureRefusal> {
    let ok = value.strip_prefix("blake3:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    });
    if ok {
        Ok(())
    } else {
        Err(ArchitectureRefusal::MalformedDigest { field })
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
    /// subject/evidence, `UNKNOWN` promotion and direct `SUPERSEDED`
    /// certification (supersession only arises from [`supersede`]).
    ///
    /// [`supersede`]: ArchitectureQualificationReceipt::supersede
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
        match standing {
            ArchitectureStanding::Unknown => return Err(ArchitectureRefusal::UnknownPromotion),
            ArchitectureStanding::Superseded => {
                return Err(ArchitectureRefusal::StandingChainMismatch)
            }
            _ => {}
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
            superseded_by_receipt_digest: None,
            receipt_digest: String::new(),
        };
        receipt.reseal();
        Ok(receipt)
    }

    /// Certify from observed evidence items: the recorded evidence digests
    /// are their [`QualificationEvidence::binding_digest`]s.
    #[allow(clippy::too_many_arguments)]
    pub fn certify_from_evidence(
        abb_digest: impl Into<String>,
        contract_digest: impl Into<String>,
        sbb_digest: impl Into<String>,
        exact_subject_digest: impl Into<String>,
        evidence: &[QualificationEvidence],
        producer_digest: impl Into<String>,
        artifact_digests: Vec<String>,
        standing: ArchitectureStanding,
    ) -> Result<Self, ArchitectureRefusal> {
        for item in evidence {
            validate_digest("evidence.producer_digest", &item.producer_digest)?;
            validate_receipt_link("evidence.content_digest", &item.content_digest)?;
        }
        Self::certify(
            abb_digest,
            contract_digest,
            sbb_digest,
            exact_subject_digest,
            evidence
                .iter()
                .map(QualificationEvidence::binding_digest)
                .collect(),
            producer_digest,
            artifact_digests,
            standing,
        )
    }

    fn reseal(&mut self) {
        self.receipt_digest = digest_without_receipt(self);
    }

    /// Verify the receipt's own integrity independent of any expected subject:
    /// schema, DO ceiling, standing, digest well-formedness, standing/chain
    /// agreement, evidence presence, canonical form and self-digest.
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
        if self.exact_subject_digest.is_empty() {
            return Err(ArchitectureRefusal::MissingExactSubject);
        }
        if self.qualification_evidence_digests.is_empty() {
            return Err(ArchitectureRefusal::MissingEvidence);
        }
        validate_digest("abb_digest", &self.abb_digest)?;
        validate_digest("contract_digest", &self.contract_digest)?;
        validate_digest("sbb_digest", &self.sbb_digest)?;
        validate_digest("exact_subject_digest", &self.exact_subject_digest)?;
        validate_digest("producer_digest", &self.producer_digest)?;
        for digest in &self.qualification_evidence_digests {
            validate_digest("qualification_evidence_digests", digest)?;
        }
        for digest in &self.artifact_digests {
            validate_digest("artifact_digests", digest)?;
        }
        if let Some(prior) = &self.prior_receipt_digest {
            validate_receipt_link("prior_receipt_digest", prior)?;
        }
        if let Some(successor) = &self.superseded_by_receipt_digest {
            validate_receipt_link("superseded_by_receipt_digest", successor)?;
        }
        let chain_ok = match self.standing {
            ArchitectureStanding::Superseded => match (
                &self.prior_receipt_digest,
                &self.superseded_by_receipt_digest,
            ) {
                (Some(prior), Some(successor)) => prior != successor,
                _ => false,
            },
            ArchitectureStanding::Qualified => self.superseded_by_receipt_digest.is_none(),
            _ => self.prior_receipt_digest.is_none() && self.superseded_by_receipt_digest.is_none(),
        };
        if !chain_ok {
            return Err(ArchitectureRefusal::StandingChainMismatch);
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
    /// Integrity only: see the module doc on authenticity.
    pub fn verify_replay(
        &self,
        expected_abb_digest: &str,
        expected_contract_digest: &str,
        expected_sbb_digest: &str,
        expected_subject_digest: &str,
    ) -> Result<(), ArchitectureRefusal> {
        self.verify_integrity()?;
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
        Ok(())
    }

    /// Re-derive the evidence set from observed evidence bytes. Refuses with
    /// [`ArchitectureRefusal::ForgedEvidence`] when any bytes do not match
    /// their content digest, or when the recorded evidence set differs from
    /// the set derived from the observations.
    pub fn verify_evidence(
        &self,
        observed: &[(QualificationEvidence, &[u8])],
    ) -> Result<(), ArchitectureRefusal> {
        self.verify_integrity()?;
        for (item, bytes) in observed {
            item.verify_bytes(bytes)?;
        }
        let mut derived: Vec<String> = observed
            .iter()
            .map(|(item, _)| item.binding_digest())
            .collect();
        derived.sort();
        derived.dedup();
        if derived != self.qualification_evidence_digests {
            return Err(ArchitectureRefusal::ForgedEvidence);
        }
        Ok(())
    }

    /// Replace this receipt's SBB. The prior receipt must be intact and
    /// QUALIFIED, and the SBB must actually change (a different digest body,
    /// not only a different algorithm label). Returns the QUALIFIED successor
    /// and the SUPERSEDED retirement record of `self`.
    pub fn supersede(
        &self,
        new_sbb_digest: impl Into<String>,
        new_subject_digest: impl Into<String>,
        evidence: Vec<String>,
    ) -> Result<Supersession, ArchitectureRefusal> {
        self.verify_integrity()?;
        if self.standing != ArchitectureStanding::Qualified {
            return Err(ArchitectureRefusal::NotSupersedable);
        }
        let new_sbb_digest = new_sbb_digest.into();
        if same_sbb(&new_sbb_digest, &self.sbb_digest) {
            return Err(ArchitectureRefusal::NotAReplacement);
        }
        let mut successor = Self::certify(
            self.abb_digest.clone(),
            self.contract_digest.clone(),
            new_sbb_digest,
            new_subject_digest,
            evidence,
            self.producer_digest.clone(),
            self.artifact_digests.clone(),
            ArchitectureStanding::Qualified,
        )?;
        successor.prior_receipt_digest = Some(self.receipt_digest.clone());
        successor.reseal();

        let mut retired = self.clone();
        retired.standing = ArchitectureStanding::Superseded;
        retired.prior_receipt_digest = Some(self.receipt_digest.clone());
        retired.superseded_by_receipt_digest = Some(successor.receipt_digest.clone());
        retired.reseal();

        Ok(Supersession { retired, successor })
    }

    /// Verify that `self` is an intact QUALIFIED successor link to an intact
    /// QUALIFIED `prior` receipt of the same ABB and contract.
    pub fn verify_chain(&self, prior: &Self) -> Result<(), ArchitectureRefusal> {
        prior.verify_integrity()?;
        self.verify_integrity()?;
        if self.prior_receipt_digest.as_deref() != Some(prior.receipt_digest.as_str())
            || self.standing != ArchitectureStanding::Qualified
        {
            return Err(ArchitectureRefusal::ChainBroken);
        }
        if prior.standing != ArchitectureStanding::Qualified {
            return Err(ArchitectureRefusal::NotSupersedable);
        }
        if self.abb_digest != prior.abb_digest {
            return Err(ArchitectureRefusal::CrossSubjectReuse);
        }
        if self.contract_digest != prior.contract_digest {
            return Err(ArchitectureRefusal::MutableOrChangedContract);
        }
        if same_sbb(&self.sbb_digest, &prior.sbb_digest) {
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

impl Supersession {
    /// Verify both halves against the intact prior receipt: the successor is
    /// a valid chain link, and the retirement record retires exactly `prior`
    /// (identical content, SUPERSEDED, pointing at prior and successor).
    pub fn verify(
        &self,
        prior: &ArchitectureQualificationReceipt,
    ) -> Result<(), ArchitectureRefusal> {
        self.successor.verify_chain(prior)?;
        self.retired.verify_integrity()?;
        let r = &self.retired;
        if r.standing != ArchitectureStanding::Superseded
            || r.prior_receipt_digest.as_deref() != Some(prior.receipt_digest.as_str())
            || r.superseded_by_receipt_digest.as_deref()
                != Some(self.successor.receipt_digest.as_str())
        {
            return Err(ArchitectureRefusal::ChainBroken);
        }
        let same_content = r.abb_digest == prior.abb_digest
            && r.contract_digest == prior.contract_digest
            && r.sbb_digest == prior.sbb_digest
            && r.exact_subject_digest == prior.exact_subject_digest
            && r.qualification_evidence_digests == prior.qualification_evidence_digests
            && r.producer_digest == prior.producer_digest
            && r.artifact_digests == prior.artifact_digests;
        if !same_content {
            return Err(ArchitectureRefusal::ChainBroken);
        }
        Ok(())
    }
}

fn same_sbb(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    match (split_digest(a), split_digest(b)) {
        (Some((_, body_a)), Some((_, body_b))) => body_a == body_b,
        _ => false,
    }
}

fn digest_without_receipt(receipt: &ArchitectureQualificationReceipt) -> String {
    let mut projection = receipt.clone();
    projection.receipt_digest.clear();
    let bytes = serde_json::to_vec(&projection).expect("architecture receipt is serializable");
    blake3_digest(&bytes)
}

/// Admitted architecture receipts with chain enforcement and standing queries
/// (charter DoD item 8). Every admitted receipt is integrity-checked; chain
/// links must reference receipts already admitted; a receipt is retired at
/// most once.
#[derive(Debug, Clone, Default)]
pub struct ArchitectureStandingLedger {
    receipts: BTreeMap<String, ArchitectureQualificationReceipt>,
    /// retired receipt digest -> successor receipt digest
    retired_by: BTreeMap<String, String>,
}

impl ArchitectureStandingLedger {
    /// Empty ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of admitted receipts.
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// True when no receipt is admitted.
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    /// Admit an unchained receipt (QUALIFIED/CANDIDATE/REFUSED with no chain
    /// fields). Chained receipts must arrive through [`admit_supersession`].
    /// Re-admitting an identical receipt is idempotent.
    ///
    /// [`admit_supersession`]: ArchitectureStandingLedger::admit_supersession
    pub fn admit(
        &mut self,
        receipt: ArchitectureQualificationReceipt,
    ) -> Result<(), ArchitectureRefusal> {
        receipt.verify_integrity()?;
        if receipt.prior_receipt_digest.is_some() {
            let prior = receipt.prior_receipt_digest.as_deref().unwrap_or_default();
            return Err(if self.receipts.contains_key(prior) {
                ArchitectureRefusal::ChainBroken
            } else {
                ArchitectureRefusal::OrphanChainLink
            });
        }
        self.receipts
            .insert(receipt.receipt_digest.clone(), receipt);
        Ok(())
    }

    /// Admit a supersession: the prior receipt must already be admitted and
    /// not yet retired, and both halves must verify against it. Nothing is
    /// admitted unless everything verifies.
    pub fn admit_supersession(
        &mut self,
        supersession: Supersession,
    ) -> Result<(), ArchitectureRefusal> {
        let prior_digest = supersession
            .successor
            .prior_receipt_digest
            .clone()
            .ok_or(ArchitectureRefusal::ChainBroken)?;
        let prior = self
            .receipts
            .get(&prior_digest)
            .ok_or(ArchitectureRefusal::OrphanChainLink)?;
        supersession.verify(prior)?;
        if let Some(existing) = self.retired_by.get(&prior_digest) {
            if existing == &supersession.successor.receipt_digest {
                return Ok(());
            }
            return Err(ArchitectureRefusal::AlreadySuperseded);
        }
        self.retired_by
            .insert(prior_digest, supersession.successor.receipt_digest.clone());
        let Supersession { retired, successor } = supersession;
        self.receipts
            .insert(successor.receipt_digest.clone(), successor);
        self.receipts
            .insert(retired.receipt_digest.clone(), retired);
        Ok(())
    }

    /// Effective standing of an admitted receipt: SUPERSEDED once retired,
    /// otherwise its own standing. `None` when not admitted.
    pub fn standing_of(&self, receipt_digest: &str) -> Option<ArchitectureStanding> {
        let receipt = self.receipts.get(receipt_digest)?;
        if self.retired_by.contains_key(receipt_digest) {
            Some(ArchitectureStanding::Superseded)
        } else {
            Some(receipt.standing)
        }
    }

    /// Receipts currently QUALIFIED for `abb_digest` (not retired, not
    /// retirement records), ordered by receipt digest.
    pub fn current_qualified(&self, abb_digest: &str) -> Vec<&ArchitectureQualificationReceipt> {
        self.receipts
            .values()
            .filter(|r| {
                r.abb_digest == abb_digest
                    && r.standing == ArchitectureStanding::Qualified
                    && !self.retired_by.contains_key(&r.receipt_digest)
            })
            .collect()
    }

    /// Machine-readable answer to "which SBBs currently stand QUALIFIED for
    /// this ABB, and which were superseded".
    pub fn query_json(&self, abb_digest: &str) -> String {
        let current: Vec<serde_json::Value> = self
            .current_qualified(abb_digest)
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "sbb_digest": r.sbb_digest,
                    "contract_digest": r.contract_digest,
                    "exact_subject_digest": r.exact_subject_digest,
                    "receipt_digest": r.receipt_digest,
                    "prior_receipt_digest": r.prior_receipt_digest,
                })
            })
            .collect();
        let superseded: Vec<serde_json::Value> = self
            .receipts
            .values()
            .filter(|r| {
                r.abb_digest == abb_digest && r.standing == ArchitectureStanding::Superseded
            })
            .map(|r| {
                serde_json::json!({
                    "sbb_digest": r.sbb_digest,
                    "retired_receipt_digest": r.prior_receipt_digest,
                    "superseded_by_receipt_digest": r.superseded_by_receipt_digest,
                    "record_digest": r.receipt_digest,
                })
            })
            .collect();
        serde_json::json!({
            "schema": ARCHITECTURE_QUERY_SCHEMA,
            "abb_digest": abb_digest,
            "confers_do_authority": false,
            "current_qualified": current,
            "superseded": superseded,
        })
        .to_string()
    }
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
    fn replacement_qualifies_successor_and_supersedes_the_prior() {
        let current = receipt();
        let s = current
            .supersede(
                "sha256:sbb-b",
                "sha256:subject-b",
                vec!["sha256:evidence-b".into()],
            )
            .unwrap();

        assert_eq!(s.successor.standing, ArchitectureStanding::Qualified);
        assert_eq!(s.successor.sbb_digest, "sha256:sbb-b");
        assert_eq!(
            s.successor.prior_receipt_digest.as_deref(),
            Some(current.receipt_digest.as_str())
        );
        assert_eq!(s.retired.standing, ArchitectureStanding::Superseded);
        assert_eq!(s.retired.sbb_digest, "sha256:sbb");
        assert_eq!(
            s.retired.superseded_by_receipt_digest.as_deref(),
            Some(s.successor.receipt_digest.as_str())
        );
        assert!(!s.successor.confers_do_authority && !s.retired.confers_do_authority);
        s.verify(&current).unwrap();
    }
}
