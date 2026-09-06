//! Formal ERRC reconstruction receipts.
//!
//! ERRC (Eliminate, Reduce, Raise, Create) is a typed, evidence-bound
//! transformation calculus here, not an optimization heuristic. The certifier
//! checks declared directional relations over commensurable
//! `(target, metric, unit)` coordinates, requires an explicit preservation
//! fence, binds exact source/subject/replay identity, and seals the result. It
//! does not choose a transformation and has no actuation authority.

use crate::standing::{ReplayEvidence, SubjectIdentity};
use crate::types::{AdmittedReceipt, Blake3Hash};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

/// Stable formal ERRC profile.
pub const ERRC_PROFILE: &str = "affidavit/errc/v1";
/// Exact archaeological source repository.
pub const ERRC_SOURCE_REPOSITORY: &str = "seanchatmangpt/ggen-legacy";
/// Exact source commit admitted for this reconstruction.
pub const ERRC_SOURCE_COMMIT: &str = "60d38265b8d1d94c43f04ca6bdb8537184e510a8";
/// Exact source artifact carrying the admitted ERRC routing mechanics.
pub const ERRC_SOURCE_ARTIFACT: &str = "scripts/ci_errc.py";
/// Maximum semantic claim supported by this receipt.
pub const ERRC_CLAIM_CEILING: &str =
    "DECLARED_DIRECTIONAL_TRANSFORMATION_ONLY_NO_CAUSAL_OR_OPTIMALITY_CLAIM";

/// One of four pairwise-disjoint directional relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrcQuadrant {
    /// `baseline > 0 && candidate == 0`.
    Eliminate,
    /// `baseline > candidate && candidate > 0`.
    Reduce,
    /// `candidate > baseline && baseline > 0`.
    Raise,
    /// `baseline == 0 && candidate > 0`.
    Create,
}

/// Exact integer measurement over one declared metric/unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrcMeasure {
    /// Metric name.
    pub metric: String,
    /// Explicit integer unit shared by baseline and candidate.
    pub unit: String,
    /// Baseline magnitude.
    pub baseline: u64,
    /// Candidate magnitude.
    pub candidate: u64,
}

/// One evidence-bound ERRC claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrcClaim {
    /// Stable claim identifier.
    pub id: String,
    /// System surface/capability being transformed.
    pub target: String,
    /// Declared directional class.
    pub quadrant: ErrcQuadrant,
    /// Before/after measurement.
    pub measure: ErrcMeasure,
    /// BLAKE3 commitment to measurement evidence.
    pub evidence_commitment: Blake3Hash,
}

/// Explicit Chesterton-fence invariant preserved across reconstruction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreservedInvariant {
    /// Stable invariant id.
    pub id: String,
    /// Human-readable invariant statement.
    pub statement: String,
    /// BLAKE3 commitment to the witnessing evidence/specification.
    pub evidence_commitment: Blake3Hash,
}

/// Immutable source coordinates of the archaeological implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrcSource {
    /// Repository coordinate.
    pub repository: String,
    /// Exact immutable commit.
    pub commit: String,
    /// Source artifact path.
    pub artifact: String,
}

/// Descriptive cardinalities only; never a cross-unit score.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuadrantCounts {
    /// Eliminate claims.
    pub eliminate: u64,
    /// Reduce claims.
    pub reduce: u64,
    /// Raise claims.
    pub raise: u64,
    /// Create claims.
    pub create: u64,
}

/// Admitted input to formal ERRC certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrcObservation {
    /// Exact subject identity.
    pub subject: SubjectIdentity,
    /// Commitment to admitted observation set O*.
    pub observation_commitment: Blake3Hash,
    /// Directional claims; certification canonicalizes by id.
    pub claims: Vec<ErrcClaim>,
    /// Non-empty preservation fence; certification canonicalizes by id.
    pub preserved_invariants: Vec<PreservedInvariant>,
    /// Deterministic replay recipe/capsule commitment.
    pub replay: ReplayEvidence,
    /// Optional predecessor for receipt-DAG lineage.
    pub previous_receipt: Option<Blake3Hash>,
}

/// Sealed ERRC receipt. Deserialization re-runs every structural law and hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::manual_non_exhaustive)]
pub struct ErrcReceipt {
    /// Profile identity.
    pub profile: String,
    /// Exact ggen-legacy lineage.
    pub source: ErrcSource,
    /// Fixed semantic claim ceiling.
    pub claim_ceiling: String,
    /// Exact subject identity.
    pub subject: SubjectIdentity,
    /// O* commitment.
    pub observation_commitment: Blake3Hash,
    /// Hash of the already-admitted source Affidavit receipt.
    pub admitted_receipt_hash: Blake3Hash,
    /// Canonically ordered claims.
    pub claims: Vec<ErrcClaim>,
    /// Canonically ordered preservation fence.
    pub preserved_invariants: Vec<PreservedInvariant>,
    /// Descriptive counts only.
    pub quadrant_counts: QuadrantCounts,
    /// Replay evidence.
    pub replay: ReplayEvidence,
    /// Optional predecessor receipt.
    pub previous_receipt: Option<Blake3Hash>,
    /// Canonical content hash.
    pub receipt_hash: Blake3Hash,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal at the formal ERRC admission boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrcRefusal {
    /// Required text was empty.
    EmptyField(&'static str),
    /// BLAKE3 commitment was malformed.
    MalformedBlake3(&'static str),
    /// At least one directional claim is required.
    NoClaims,
    /// At least one preservation invariant is required.
    NoPreservationFence,
    /// Claim ids must be unique.
    DuplicateClaimId(String),
    /// Invariant ids must be unique.
    DuplicateInvariantId(String),
    /// A factor coordinate may have exactly one directional classification.
    DuplicateFactorCoordinate {
        /// Target coordinate.
        target: String,
        /// Metric coordinate.
        metric: String,
        /// Unit coordinate.
        unit: String,
    },
    /// The declared ERRC class contradicts its exact before/after values.
    DirectionViolation {
        /// Claim identifier.
        claim_id: String,
        /// Declared class.
        quadrant: ErrcQuadrant,
        /// Baseline value.
        baseline: u64,
        /// Candidate value.
        candidate: u64,
    },
    /// Serialized set order was not canonical.
    NonCanonicalOrder(&'static str),
    /// Profile identity changed.
    WrongProfile,
    /// Archaeological source identity changed.
    WrongSource,
    /// Claim ceiling was broadened/changed.
    WrongClaimCeiling,
    /// Stored receipt material/hash disagrees.
    ReceiptHashMismatch,
    /// Canonical serialization failed.
    Serialization(String),
}

impl core::fmt::Display for ErrcRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "empty_field: {field}"),
            Self::MalformedBlake3(field) => write!(f, "malformed_blake3: {field}"),
            Self::NoClaims => f.write_str("errc_no_claims"),
            Self::NoPreservationFence => f.write_str("errc_no_preservation_fence"),
            Self::DuplicateClaimId(id) => write!(f, "errc_duplicate_claim_id: {id}"),
            Self::DuplicateInvariantId(id) => write!(f, "errc_duplicate_invariant_id: {id}"),
            Self::DuplicateFactorCoordinate { target, metric, unit } => write!(
                f,
                "errc_duplicate_factor_coordinate: target={target} metric={metric} unit={unit}"
            ),
            Self::DirectionViolation { claim_id, quadrant, baseline, candidate } => write!(
                f,
                "errc_direction_violation: claim={claim_id} quadrant={quadrant:?} baseline={baseline} candidate={candidate}"
            ),
            Self::NonCanonicalOrder(field) => write!(f, "errc_non_canonical_order: {field}"),
            Self::WrongProfile => f.write_str("wrong_errc_profile"),
            Self::WrongSource => f.write_str("wrong_errc_source"),
            Self::WrongClaimCeiling => f.write_str("wrong_errc_claim_ceiling"),
            Self::ReceiptHashMismatch => f.write_str("errc_receipt_hash_mismatch"),
            Self::Serialization(reason) => write!(f, "errc_serialization: {reason}"),
        }
    }
}

impl std::error::Error for ErrcRefusal {}

#[derive(Serialize)]
struct ErrcMaterial<'a> {
    profile: &'a str,
    source: &'a ErrcSource,
    claim_ceiling: &'a str,
    subject: &'a SubjectIdentity,
    observation_commitment: &'a Blake3Hash,
    admitted_receipt_hash: &'a Blake3Hash,
    claims: &'a [ErrcClaim],
    preserved_invariants: &'a [PreservedInvariant],
    quadrant_counts: QuadrantCounts,
    replay: &'a ReplayEvidence,
    previous_receipt: &'a Option<Blake3Hash>,
}

impl ErrcReceipt {
    /// Re-run profile, source, directional, preservation, replay, and hash laws.
    pub fn verify(&self) -> Result<(), ErrcRefusal> {
        validate_fixed_fields(&self.profile, &self.source, &self.claim_ceiling)?;
        validate_material(
            &self.subject,
            &self.observation_commitment,
            &self.admitted_receipt_hash,
            &self.claims,
            &self.preserved_invariants,
            &self.replay,
            &self.previous_receipt,
            true,
        )?;
        if count_quadrants(&self.claims) != self.quadrant_counts {
            return Err(ErrcRefusal::ReceiptHashMismatch);
        }
        let expected = compute_receipt_hash(
            &self.profile,
            &self.source,
            &self.claim_ceiling,
            &self.subject,
            &self.observation_commitment,
            &self.admitted_receipt_hash,
            &self.claims,
            &self.preserved_invariants,
            self.quadrant_counts,
            &self.replay,
            &self.previous_receipt,
        )?;
        if expected != self.receipt_hash {
            return Err(ErrcRefusal::ReceiptHashMismatch);
        }
        Ok(())
    }
}

/// Certify a declared ERRC transformation over an already-admitted receipt.
///
/// This is CONSTRUCT only. It does not select policy, infer causality/utility,
/// claim optimality, or actuate machine state.
pub fn certify_errc(
    admitted: &AdmittedReceipt,
    mut observation: ErrcObservation,
) -> Result<ErrcReceipt, ErrcRefusal> {
    observation.claims.sort_by(|a, b| a.id.cmp(&b.id));
    observation
        .preserved_invariants
        .sort_by(|a, b| a.id.cmp(&b.id));

    let source = ErrcSource {
        repository: ERRC_SOURCE_REPOSITORY.to_string(),
        commit: ERRC_SOURCE_COMMIT.to_string(),
        artifact: ERRC_SOURCE_ARTIFACT.to_string(),
    };
    let admitted_receipt_hash = admitted.value.chain_hash.clone();
    validate_fixed_fields(ERRC_PROFILE, &source, ERRC_CLAIM_CEILING)?;
    validate_material(
        &observation.subject,
        &observation.observation_commitment,
        &admitted_receipt_hash,
        &observation.claims,
        &observation.preserved_invariants,
        &observation.replay,
        &observation.previous_receipt,
        true,
    )?;
    let quadrant_counts = count_quadrants(&observation.claims);
    let receipt_hash = compute_receipt_hash(
        ERRC_PROFILE,
        &source,
        ERRC_CLAIM_CEILING,
        &observation.subject,
        &observation.observation_commitment,
        &admitted_receipt_hash,
        &observation.claims,
        &observation.preserved_invariants,
        quadrant_counts,
        &observation.replay,
        &observation.previous_receipt,
    )?;

    Ok(ErrcReceipt {
        profile: ERRC_PROFILE.to_string(),
        source,
        claim_ceiling: ERRC_CLAIM_CEILING.to_string(),
        subject: observation.subject,
        observation_commitment: observation.observation_commitment,
        admitted_receipt_hash,
        claims: observation.claims,
        preserved_invariants: observation.preserved_invariants,
        quadrant_counts,
        replay: observation.replay,
        previous_receipt: observation.previous_receipt,
        receipt_hash,
        _seal: (),
    })
}

fn validate_fixed_fields(
    profile: &str,
    source: &ErrcSource,
    claim_ceiling: &str,
) -> Result<(), ErrcRefusal> {
    if profile != ERRC_PROFILE {
        return Err(ErrcRefusal::WrongProfile);
    }
    if source.repository != ERRC_SOURCE_REPOSITORY
        || source.commit != ERRC_SOURCE_COMMIT
        || source.artifact != ERRC_SOURCE_ARTIFACT
    {
        return Err(ErrcRefusal::WrongSource);
    }
    if claim_ceiling != ERRC_CLAIM_CEILING {
        return Err(ErrcRefusal::WrongClaimCeiling);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_material(
    subject: &SubjectIdentity,
    observation_commitment: &Blake3Hash,
    admitted_receipt_hash: &Blake3Hash,
    claims: &[ErrcClaim],
    preserved_invariants: &[PreservedInvariant],
    replay: &ReplayEvidence,
    previous_receipt: &Option<Blake3Hash>,
    require_canonical_order: bool,
) -> Result<(), ErrcRefusal> {
    require_text("subject.subject", &subject.subject)?;
    require_text("subject.base", &subject.base)?;
    require_text("subject.tree", &subject.tree)?;
    require_text("subject.candidate", &subject.candidate)?;
    require_blake3("observation_commitment", observation_commitment)?;
    require_blake3("admitted_receipt_hash", admitted_receipt_hash)?;
    if claims.is_empty() {
        return Err(ErrcRefusal::NoClaims);
    }
    if preserved_invariants.is_empty() {
        return Err(ErrcRefusal::NoPreservationFence);
    }
    if require_canonical_order {
        require_sorted_claims(claims)?;
        require_sorted_invariants(preserved_invariants)?;
    }

    let mut claim_ids = BTreeSet::new();
    let mut coordinates = BTreeSet::new();
    for claim in claims {
        require_text("claim.id", &claim.id)?;
        require_text("claim.target", &claim.target)?;
        require_text("claim.measure.metric", &claim.measure.metric)?;
        require_text("claim.measure.unit", &claim.measure.unit)?;
        require_blake3("claim.evidence_commitment", &claim.evidence_commitment)?;
        if !claim_ids.insert(claim.id.clone()) {
            return Err(ErrcRefusal::DuplicateClaimId(claim.id.clone()));
        }
        let coordinate = (
            claim.target.clone(),
            claim.measure.metric.clone(),
            claim.measure.unit.clone(),
        );
        if !coordinates.insert(coordinate.clone()) {
            return Err(ErrcRefusal::DuplicateFactorCoordinate {
                target: coordinate.0,
                metric: coordinate.1,
                unit: coordinate.2,
            });
        }
        if !direction_holds(claim) {
            return Err(ErrcRefusal::DirectionViolation {
                claim_id: claim.id.clone(),
                quadrant: claim.quadrant,
                baseline: claim.measure.baseline,
                candidate: claim.measure.candidate,
            });
        }
    }

    let mut invariant_ids = BTreeSet::new();
    for invariant in preserved_invariants {
        require_text("invariant.id", &invariant.id)?;
        require_text("invariant.statement", &invariant.statement)?;
        require_blake3(
            "invariant.evidence_commitment",
            &invariant.evidence_commitment,
        )?;
        if !invariant_ids.insert(invariant.id.clone()) {
            return Err(ErrcRefusal::DuplicateInvariantId(invariant.id.clone()));
        }
    }
    require_text("replay.command", &replay.command)?;
    require_blake3(
        "replay.environment_commitment",
        &replay.environment_commitment,
    )?;
    if let Some(previous) = previous_receipt {
        require_blake3("previous_receipt", previous)?;
    }
    Ok(())
}

fn direction_holds(claim: &ErrcClaim) -> bool {
    let b = claim.measure.baseline;
    let c = claim.measure.candidate;
    match claim.quadrant {
        ErrcQuadrant::Eliminate => b > 0 && c == 0,
        ErrcQuadrant::Reduce => b > c && c > 0,
        ErrcQuadrant::Raise => c > b && b > 0,
        ErrcQuadrant::Create => b == 0 && c > 0,
    }
}

fn require_sorted_claims(claims: &[ErrcClaim]) -> Result<(), ErrcRefusal> {
    if claims
        .windows(2)
        .any(|p| p[0].id.as_str() > p[1].id.as_str())
    {
        Err(ErrcRefusal::NonCanonicalOrder("claims"))
    } else {
        Ok(())
    }
}

fn require_sorted_invariants(invariants: &[PreservedInvariant]) -> Result<(), ErrcRefusal> {
    if invariants
        .windows(2)
        .any(|p| p[0].id.as_str() > p[1].id.as_str())
    {
        Err(ErrcRefusal::NonCanonicalOrder("preserved_invariants"))
    } else {
        Ok(())
    }
}

fn count_quadrants(claims: &[ErrcClaim]) -> QuadrantCounts {
    let mut counts = QuadrantCounts::default();
    for claim in claims {
        match claim.quadrant {
            ErrcQuadrant::Eliminate => counts.eliminate += 1,
            ErrcQuadrant::Reduce => counts.reduce += 1,
            ErrcQuadrant::Raise => counts.raise += 1,
            ErrcQuadrant::Create => counts.create += 1,
        }
    }
    counts
}

#[allow(clippy::too_many_arguments)]
fn compute_receipt_hash(
    profile: &str,
    source: &ErrcSource,
    claim_ceiling: &str,
    subject: &SubjectIdentity,
    observation_commitment: &Blake3Hash,
    admitted_receipt_hash: &Blake3Hash,
    claims: &[ErrcClaim],
    preserved_invariants: &[PreservedInvariant],
    quadrant_counts: QuadrantCounts,
    replay: &ReplayEvidence,
    previous_receipt: &Option<Blake3Hash>,
) -> Result<Blake3Hash, ErrcRefusal> {
    let material = ErrcMaterial {
        profile,
        source,
        claim_ceiling,
        subject,
        observation_commitment,
        admitted_receipt_hash,
        claims,
        preserved_invariants,
        quadrant_counts,
        replay,
        previous_receipt,
    };
    let canonical =
        serde_json::to_vec(&material).map_err(|e| ErrcRefusal::Serialization(e.to_string()))?;
    let mut bytes = Vec::with_capacity(profile.len() + canonical.len() + 1);
    bytes.extend_from_slice(profile.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&canonical);
    Ok(Blake3Hash::from_bytes(&bytes))
}

fn require_text(field: &'static str, value: &str) -> Result<(), ErrcRefusal> {
    if value.trim().is_empty() {
        Err(ErrcRefusal::EmptyField(field))
    } else {
        Ok(())
    }
}

fn require_blake3(field: &'static str, value: &Blake3Hash) -> Result<(), ErrcRefusal> {
    let hex = value.as_hex();
    // Lowercase only. `is_ascii_hexdigit` also accepts A-F, which would let
    // the same digest appear as two distinct strings and therefore hash to
    // two distinct receipt identities — a canonicalisation hole under ADR-5.
    // Every digest this crate produces is lowercase (`blake3::Hash::to_hex`),
    // so this narrows admission to what is already canonical.
    if hex.len() == 64
        && hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        Ok(())
    } else {
        Err(ErrcRefusal::MalformedBlake3(field))
    }
}

impl<'de> Deserialize<'de> for ErrcReceipt {
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
            subject: SubjectIdentity,
            observation_commitment: Blake3Hash,
            admitted_receipt_hash: Blake3Hash,
            claims: Vec<ErrcClaim>,
            preserved_invariants: Vec<PreservedInvariant>,
            quadrant_counts: QuadrantCounts,
            replay: ReplayEvidence,
            previous_receipt: Option<Blake3Hash>,
            receipt_hash: Blake3Hash,
        }
        let raw = Raw::deserialize(deserializer)?;
        let receipt = Self {
            profile: raw.profile,
            source: raw.source,
            claim_ceiling: raw.claim_ceiling,
            subject: raw.subject,
            observation_commitment: raw.observation_commitment,
            admitted_receipt_hash: raw.admitted_receipt_hash,
            claims: raw.claims,
            preserved_invariants: raw.preserved_invariants,
            quadrant_counts: raw.quadrant_counts,
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

    fn admitted_receipt() -> AdmittedReceipt {
        let mut assembler = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "errc-test",
            vec![object_ref("repo", "git")],
            b"admitted-errc-observation",
            &mut counter,
        )
        .expect("event");
        assembler.append(event).expect("append");
        crate::admission::admit(assembler.finalize()).expect("admitted")
    }

    fn claim(id: &str, target: &str, quadrant: ErrcQuadrant, b: u64, c: u64) -> ErrcClaim {
        ErrcClaim {
            id: id.to_string(),
            target: target.to_string(),
            quadrant,
            measure: ErrcMeasure {
                metric: "surface_count".to_string(),
                unit: "count".to_string(),
                baseline: b,
                candidate: c,
            },
            evidence_commitment: Blake3Hash::from_bytes(id.as_bytes()),
        }
    }

    fn observation() -> ErrcObservation {
        ErrcObservation {
            subject: SubjectIdentity {
                subject: "seanchatmangpt/affidavit".to_string(),
                base: "383b41dc9c587f8a8a90fe280b9621ec55701a2c".to_string(),
                tree: "tree-id".to_string(),
                candidate: "candidate-head".to_string(),
            },
            observation_commitment: Blake3Hash::from_bytes(b"O-star"),
            claims: vec![
                claim("raise-receipts", "receipts", ErrcQuadrant::Raise, 1, 2),
                claim("eliminate-null", "cli-null", ErrcQuadrant::Eliminate, 1, 0),
                claim("create-court", "errc-court", ErrcQuadrant::Create, 0, 1),
                claim(
                    "reduce-stubs",
                    "dependency-stubs",
                    ErrcQuadrant::Reduce,
                    3,
                    2,
                ),
            ],
            preserved_invariants: vec![PreservedInvariant {
                id: "certify-dont-decide".to_string(),
                statement: "Affidavit certifies evidence and does not choose policy.".to_string(),
                evidence_commitment: Blake3Hash::from_bytes(b"doctrine"),
            }],
            replay: ReplayEvidence {
                command: "python3 scripts/ci_errc.py --changed-file src/errc.rs".to_string(),
                environment_commitment: Blake3Hash::from_bytes(b"python-stdlib"),
            },
            previous_receipt: None,
        }
    }

    #[test]
    fn all_four_disjoint_relations_certify() {
        let receipt = certify_errc(&admitted_receipt(), observation()).expect("valid ERRC");
        assert_eq!(
            receipt.quadrant_counts,
            QuadrantCounts {
                eliminate: 1,
                reduce: 1,
                raise: 1,
                create: 1
            }
        );
        assert!(receipt.verify().is_ok());
    }

    #[test]
    fn certification_canonicalizes_claim_set_order() {
        let admitted = admitted_receipt();
        let mut reversed = observation();
        reversed.claims.reverse();
        let a = certify_errc(&admitted, observation()).expect("a");
        let b = certify_errc(&admitted, reversed).expect("b");
        assert_eq!(a.receipt_hash, b.receipt_hash);
    }

    #[test]
    fn eliminate_must_reach_zero() {
        let admitted = admitted_receipt();
        let mut invalid = observation();
        invalid.claims[1].measure.candidate = 1;
        assert!(matches!(
            certify_errc(&admitted, invalid),
            Err(ErrcRefusal::DirectionViolation {
                quadrant: ErrcQuadrant::Eliminate,
                ..
            })
        ));
    }

    #[test]
    fn reduce_must_remain_present() {
        let admitted = admitted_receipt();
        let mut invalid = observation();
        invalid.claims[3].measure.candidate = 0;
        assert!(matches!(
            certify_errc(&admitted, invalid),
            Err(ErrcRefusal::DirectionViolation {
                quadrant: ErrcQuadrant::Reduce,
                ..
            })
        ));
    }

    #[test]
    fn raise_requires_an_existing_factor() {
        let admitted = admitted_receipt();
        let mut invalid = observation();
        invalid.claims[0].measure.baseline = 0;
        assert!(matches!(
            certify_errc(&admitted, invalid),
            Err(ErrcRefusal::DirectionViolation {
                quadrant: ErrcQuadrant::Raise,
                ..
            })
        ));
    }

    #[test]
    fn duplicate_claim_id_is_typed() {
        let admitted = admitted_receipt();
        let mut invalid = observation();
        let mut duplicate = invalid.claims[0].clone();
        duplicate.target = "other-target".to_string();
        duplicate.measure.metric = "other-metric".to_string();
        invalid.claims.push(duplicate);
        assert!(matches!(
            certify_errc(&admitted, invalid),
            Err(ErrcRefusal::DuplicateClaimId(_))
        ));
    }

    #[test]
    fn duplicate_factor_coordinate_is_refused() {
        let admitted = admitted_receipt();
        let mut invalid = observation();
        let mut duplicate = invalid.claims[0].clone();
        duplicate.id = "other-id".to_string();
        invalid.claims.push(duplicate);
        assert!(matches!(
            certify_errc(&admitted, invalid),
            Err(ErrcRefusal::DuplicateFactorCoordinate { .. })
        ));
    }

    #[test]
    fn preservation_fence_is_mandatory() {
        let admitted = admitted_receipt();
        let mut invalid = observation();
        invalid.preserved_invariants.clear();
        assert_eq!(
            certify_errc(&admitted, invalid).unwrap_err(),
            ErrcRefusal::NoPreservationFence
        );
    }

    #[test]
    fn tampered_serialized_receipt_is_rejected() {
        let admitted = admitted_receipt();
        let receipt = certify_errc(&admitted, observation()).expect("receipt");
        let mut json = serde_json::to_value(&receipt).expect("json");
        json["claims"][0]["measure"]["candidate"] = serde_json::Value::from(99_u64);
        assert!(serde_json::from_value::<ErrcReceipt>(json).is_err());
    }

    #[test]
    fn source_lineage_is_exact() {
        let admitted = admitted_receipt();
        let receipt = certify_errc(&admitted, observation()).expect("receipt");
        assert_eq!(receipt.source.repository, ERRC_SOURCE_REPOSITORY);
        assert_eq!(receipt.source.commit, ERRC_SOURCE_COMMIT);
        assert_eq!(receipt.source.artifact, ERRC_SOURCE_ARTIFACT);
    }
}
