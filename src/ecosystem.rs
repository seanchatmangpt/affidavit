//! Cross-repository evidence federation for the Chatman ecosystem.
//!
//! This module composes already-sealed [`StandingReceipt`] values into one
//! deterministic ecosystem receipt. Composition is evidence-only: membership
//! does not transfer authority, adjacent repositories do not become trusted by
//! association, and an outer ALIVE standing means only that every explicitly
//! required evidence role reached its declared ALIVE quorum.
//!
//! A failed optional edge is preserved as topology instead of collapsing the
//! whole graph. This keeps the federation compatible with heterogeneous systems
//! where some capabilities are BLOCKED, BUILD_BROKEN, or UNSUPPORTED while the
//! exact required proof surface remains satisfied.

use crate::standing::{Standing, StandingReceipt, SubjectIdentity};
use crate::types::{AdmittedReceipt, Blake3Hash};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Stable profile for cross-repository evidence federation.
pub const ECOSYSTEM_PROFILE: &str = "affidavit/ecosystem/v1";
/// Maximum semantic claim supported by an ecosystem federation receipt.
pub const ECOSYSTEM_CLAIM_CEILING: &str =
    "VERIFIED_STANDING_COMPOSITION_ONLY_NO_TRANSITIVE_TRUTH_OR_ACTUATION_CLAIM";
/// Fixed authority ceiling: federation never manufactures actuation authority.
pub const ECOSYSTEM_AUTHORITY_CEILING: &str = "EVIDENCE_ONLY_NO_AMBIENT_ACTUATION_AUTHORITY";

/// Evidence role contributed by one repository/capability.
///
/// Roles are deliberately independent of repository names. A repository joins
/// the federation by presenting a valid standing receipt under the role it
/// actually witnessed; this avoids freezing a stale ecosystem topology into
/// Affidavit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EcosystemRole {
    /// Ontologies, schemas, policies, or other admitted semantic authority.
    SemanticAuthority,
    /// Deterministic artifact/code/configuration manufacture.
    Manufacture,
    /// Planning, scheduling, routing, or control-plane construction evidence.
    PlanningControl,
    /// Bounded world/environment execution evidence.
    WorldExecution,
    /// Machine-checked theorem/proof evidence.
    FormalProof,
    /// Runtime/program execution evidence.
    RuntimeExecution,
    /// Process/event/conformance evidence.
    ProcessEvidence,
    /// Configuration and environment identity evidence.
    Configuration,
    /// Dependency, SBOM, artifact, or supply-chain evidence.
    SupplyChain,
    /// Independent verifier/court evidence.
    Verification,
    /// Deterministic replay/capsule evidence.
    Replay,
}

/// Explicit ALIVE quorum required for one evidence role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleRequirement {
    /// Required evidence role.
    pub role: EcosystemRole,
    /// Minimum number of distinct ALIVE member receipts required.
    pub minimum_alive: usize,
}

/// One exact member receipt admitted into the federation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcosystemMember {
    /// Evidence role played by this exact receipt.
    pub role: EcosystemRole,
    /// Sealed standing receipt. Deserialization re-verifies this receipt.
    pub standing_receipt: StandingReceipt,
}

/// Per-role topology retained by the federation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleCoverage {
    /// Evidence role.
    pub role: EcosystemRole,
    /// Declared ALIVE quorum. Zero means the role is observed but optional.
    pub required_alive: usize,
    /// Total member receipts observed under this role.
    pub total: usize,
    /// ALIVE members.
    pub alive: usize,
    /// PARTIAL_ALIVE members.
    pub partial_alive: usize,
    /// UNKNOWN members.
    pub unknown: usize,
    /// BLOCKED members.
    pub blocked: usize,
    /// BUILD_BROKEN members.
    pub build_broken: usize,
    /// UNSUPPORTED members.
    pub unsupported: usize,
    /// Whether this role satisfies its declared ALIVE quorum.
    pub satisfied: bool,
}

/// Inputs admitted for ecosystem federation certification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EcosystemObservation {
    /// Exact identity of the federation subject.
    pub subject: SubjectIdentity,
    /// Commitment to the admitted observation set O* for this composition.
    pub observation_commitment: Blake3Hash,
    /// Explicit role/quorum crown criteria.
    pub requirements: Vec<RoleRequirement>,
    /// Exact sealed standing receipts to compose.
    pub members: Vec<EcosystemMember>,
    /// Optional predecessor for receipt-DAG lineage.
    pub previous_receipt: Option<Blake3Hash>,
}

/// Sealed cross-repository evidence federation receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[allow(clippy::manual_non_exhaustive)]
pub struct EcosystemReceipt {
    /// Profile identity.
    pub profile: String,
    /// Fixed semantic claim ceiling.
    pub claim_ceiling: String,
    /// Fixed authority ceiling.
    pub authority_ceiling: String,
    /// Exact federation subject.
    pub subject: SubjectIdentity,
    /// O* commitment.
    pub observation_commitment: Blake3Hash,
    /// Hash of the already-admitted source Affidavit receipt.
    pub admitted_receipt_hash: Blake3Hash,
    /// Canonically ordered role requirements.
    pub requirements: Vec<RoleRequirement>,
    /// Canonically ordered member standing receipts.
    pub members: Vec<EcosystemMember>,
    /// Canonically ordered per-role topology.
    pub coverage: Vec<RoleCoverage>,
    /// Derived federation standing.
    pub standing: Standing,
    /// Optional predecessor receipt.
    pub previous_receipt: Option<Blake3Hash>,
    /// Canonical BLAKE3 identity over every field above.
    pub receipt_hash: Blake3Hash,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal from ecosystem federation certification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EcosystemRefusal {
    /// At least one explicit crown requirement is mandatory.
    NoRequirements,
    /// A requirement with a zero quorum is vacuous and therefore refused.
    ZeroAliveRequirement(EcosystemRole),
    /// A role may appear only once in the requirement set.
    DuplicateRequirement(EcosystemRole),
    /// The same exact member cannot be counted twice under one role.
    DuplicateMember {
        /// Duplicate role.
        role: EcosystemRole,
        /// Exact member subject.
        subject: String,
        /// Exact member candidate.
        candidate: String,
    },
    /// A nested standing receipt failed its own verifier.
    MemberInvalid {
        /// Member subject.
        subject: String,
        /// Nested verifier refusal rendered without widening its claim.
        reason: String,
    },
    /// Serialized set order was not canonical.
    NonCanonicalOrder(&'static str),
    /// A required text field was empty.
    EmptyField(&'static str),
    /// A BLAKE3 commitment was malformed.
    MalformedBlake3(&'static str),
    /// Profile identity changed.
    WrongProfile,
    /// Claim ceiling was broadened/changed.
    WrongClaimCeiling,
    /// Authority ceiling was broadened/changed.
    WrongAuthorityCeiling,
    /// Stored coverage disagrees with the member topology.
    CoverageMismatch,
    /// Stored federation standing disagrees with the explicit quorum calculus.
    StandingMismatch,
    /// Canonical content hash disagrees with the stored identity.
    ReceiptHashMismatch,
    /// Canonical serialization unexpectedly failed.
    Serialization(String),
}

impl core::fmt::Display for EcosystemRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoRequirements => f.write_str("ecosystem_no_requirements"),
            Self::ZeroAliveRequirement(role) => {
                write!(f, "ecosystem_zero_alive_requirement: {role:?}")
            }
            Self::DuplicateRequirement(role) => {
                write!(f, "ecosystem_duplicate_requirement: {role:?}")
            }
            Self::DuplicateMember {
                role,
                subject,
                candidate,
            } => write!(
                f,
                "ecosystem_duplicate_member: role={role:?} subject={subject} candidate={candidate}"
            ),
            Self::MemberInvalid { subject, reason } => {
                write!(
                    f,
                    "ecosystem_member_invalid: subject={subject} reason={reason}"
                )
            }
            Self::NonCanonicalOrder(field) => {
                write!(f, "ecosystem_non_canonical_order: {field}")
            }
            Self::EmptyField(field) => write!(f, "empty_field: {field}"),
            Self::MalformedBlake3(field) => write!(f, "malformed_blake3: {field}"),
            Self::WrongProfile => f.write_str("wrong_ecosystem_profile"),
            Self::WrongClaimCeiling => f.write_str("wrong_ecosystem_claim_ceiling"),
            Self::WrongAuthorityCeiling => f.write_str("wrong_ecosystem_authority_ceiling"),
            Self::CoverageMismatch => f.write_str("ecosystem_coverage_mismatch"),
            Self::StandingMismatch => f.write_str("ecosystem_standing_mismatch"),
            Self::ReceiptHashMismatch => f.write_str("ecosystem_receipt_hash_mismatch"),
            Self::Serialization(reason) => write!(f, "ecosystem_serialization: {reason}"),
        }
    }
}

impl std::error::Error for EcosystemRefusal {}

#[derive(Serialize)]
struct EcosystemMaterial<'a> {
    profile: &'a str,
    claim_ceiling: &'a str,
    authority_ceiling: &'a str,
    subject: &'a SubjectIdentity,
    observation_commitment: &'a Blake3Hash,
    admitted_receipt_hash: &'a Blake3Hash,
    requirements: &'a [RoleRequirement],
    members: &'a [EcosystemMember],
    coverage: &'a [RoleCoverage],
    standing: Standing,
    previous_receipt: &'a Option<Blake3Hash>,
}

impl EcosystemReceipt {
    /// Re-run member verification, quorum topology, ceilings, and canonical hash.
    pub fn verify(&self) -> Result<(), EcosystemRefusal> {
        validate_fixed_fields(&self.profile, &self.claim_ceiling, &self.authority_ceiling)?;
        validate_material(
            &self.subject,
            &self.observation_commitment,
            &self.admitted_receipt_hash,
            &self.requirements,
            &self.members,
            &self.previous_receipt,
            true,
        )?;

        let expected_coverage = compute_coverage(&self.requirements, &self.members);
        if expected_coverage != self.coverage {
            return Err(EcosystemRefusal::CoverageMismatch);
        }
        let expected_standing = derive_standing(&self.members, &expected_coverage);
        if expected_standing != self.standing {
            return Err(EcosystemRefusal::StandingMismatch);
        }

        let expected_hash = compute_receipt_hash(
            &self.profile,
            &self.claim_ceiling,
            &self.authority_ceiling,
            &self.subject,
            &self.observation_commitment,
            &self.admitted_receipt_hash,
            &self.requirements,
            &self.members,
            &self.coverage,
            self.standing,
            &self.previous_receipt,
        )?;
        if expected_hash != self.receipt_hash {
            return Err(EcosystemRefusal::ReceiptHashMismatch);
        }
        Ok(())
    }
}

/// Certify a federation over exact, already-sealed repository standing receipts.
///
/// This is CONSTRUCT only. It does not execute member repositories, choose a
/// planner/policy, inherit member authority, or actuate machine state.
pub fn certify_ecosystem(
    admitted: &AdmittedReceipt,
    mut observation: EcosystemObservation,
) -> Result<EcosystemReceipt, EcosystemRefusal> {
    observation
        .requirements
        .sort_by_key(|requirement| requirement.role);
    observation.members.sort_by(compare_members);

    let admitted_receipt_hash = admitted.value.chain_hash.clone();
    validate_fixed_fields(
        ECOSYSTEM_PROFILE,
        ECOSYSTEM_CLAIM_CEILING,
        ECOSYSTEM_AUTHORITY_CEILING,
    )?;
    validate_material(
        &observation.subject,
        &observation.observation_commitment,
        &admitted_receipt_hash,
        &observation.requirements,
        &observation.members,
        &observation.previous_receipt,
        true,
    )?;

    let coverage = compute_coverage(&observation.requirements, &observation.members);
    let standing = derive_standing(&observation.members, &coverage);
    let receipt_hash = compute_receipt_hash(
        ECOSYSTEM_PROFILE,
        ECOSYSTEM_CLAIM_CEILING,
        ECOSYSTEM_AUTHORITY_CEILING,
        &observation.subject,
        &observation.observation_commitment,
        &admitted_receipt_hash,
        &observation.requirements,
        &observation.members,
        &coverage,
        standing,
        &observation.previous_receipt,
    )?;

    Ok(EcosystemReceipt {
        profile: ECOSYSTEM_PROFILE.to_string(),
        claim_ceiling: ECOSYSTEM_CLAIM_CEILING.to_string(),
        authority_ceiling: ECOSYSTEM_AUTHORITY_CEILING.to_string(),
        subject: observation.subject,
        observation_commitment: observation.observation_commitment,
        admitted_receipt_hash,
        requirements: observation.requirements,
        members: observation.members,
        coverage,
        standing,
        previous_receipt: observation.previous_receipt,
        receipt_hash,
        _seal: (),
    })
}

fn validate_fixed_fields(
    profile: &str,
    claim_ceiling: &str,
    authority_ceiling: &str,
) -> Result<(), EcosystemRefusal> {
    if profile != ECOSYSTEM_PROFILE {
        return Err(EcosystemRefusal::WrongProfile);
    }
    if claim_ceiling != ECOSYSTEM_CLAIM_CEILING {
        return Err(EcosystemRefusal::WrongClaimCeiling);
    }
    if authority_ceiling != ECOSYSTEM_AUTHORITY_CEILING {
        return Err(EcosystemRefusal::WrongAuthorityCeiling);
    }
    Ok(())
}

fn validate_material(
    subject: &SubjectIdentity,
    observation_commitment: &Blake3Hash,
    admitted_receipt_hash: &Blake3Hash,
    requirements: &[RoleRequirement],
    members: &[EcosystemMember],
    previous_receipt: &Option<Blake3Hash>,
    require_canonical_order: bool,
) -> Result<(), EcosystemRefusal> {
    require_text("subject.subject", &subject.subject)?;
    require_text("subject.base", &subject.base)?;
    require_text("subject.tree", &subject.tree)?;
    require_text("subject.candidate", &subject.candidate)?;
    require_blake3("observation_commitment", observation_commitment)?;
    require_blake3("admitted_receipt_hash", admitted_receipt_hash)?;
    if let Some(previous) = previous_receipt {
        require_blake3("previous_receipt", previous)?;
    }

    if requirements.is_empty() {
        return Err(EcosystemRefusal::NoRequirements);
    }
    if require_canonical_order {
        require_sorted_requirements(requirements)?;
        require_sorted_members(members)?;
    }

    let mut required_roles = BTreeSet::new();
    for requirement in requirements {
        if requirement.minimum_alive == 0 {
            return Err(EcosystemRefusal::ZeroAliveRequirement(requirement.role));
        }
        if !required_roles.insert(requirement.role) {
            return Err(EcosystemRefusal::DuplicateRequirement(requirement.role));
        }
    }

    let mut member_keys = BTreeSet::new();
    for member in members {
        member
            .standing_receipt
            .verify()
            .map_err(|error| EcosystemRefusal::MemberInvalid {
                subject: member.standing_receipt.subject.subject.clone(),
                reason: error.to_string(),
            })?;
        let key = (
            member.role,
            member.standing_receipt.subject.subject.clone(),
            member.standing_receipt.subject.candidate.clone(),
        );
        if !member_keys.insert(key.clone()) {
            return Err(EcosystemRefusal::DuplicateMember {
                role: key.0,
                subject: key.1,
                candidate: key.2,
            });
        }
    }

    Ok(())
}

fn require_sorted_requirements(requirements: &[RoleRequirement]) -> Result<(), EcosystemRefusal> {
    if requirements
        .windows(2)
        .any(|pair| pair[0].role > pair[1].role)
    {
        Err(EcosystemRefusal::NonCanonicalOrder("requirements"))
    } else {
        Ok(())
    }
}

fn compare_members(left: &EcosystemMember, right: &EcosystemMember) -> core::cmp::Ordering {
    left.role
        .cmp(&right.role)
        .then_with(|| {
            left.standing_receipt
                .subject
                .subject
                .cmp(&right.standing_receipt.subject.subject)
        })
        .then_with(|| {
            left.standing_receipt
                .subject
                .candidate
                .cmp(&right.standing_receipt.subject.candidate)
        })
        .then_with(|| {
            left.standing_receipt
                .receipt_hash
                .cmp(&right.standing_receipt.receipt_hash)
        })
}

fn require_sorted_members(members: &[EcosystemMember]) -> Result<(), EcosystemRefusal> {
    if members
        .windows(2)
        .any(|pair| compare_members(&pair[0], &pair[1]).is_gt())
    {
        Err(EcosystemRefusal::NonCanonicalOrder("members"))
    } else {
        Ok(())
    }
}

fn compute_coverage(
    requirements: &[RoleRequirement],
    members: &[EcosystemMember],
) -> Vec<RoleCoverage> {
    let required: BTreeMap<EcosystemRole, usize> = requirements
        .iter()
        .map(|requirement| (requirement.role, requirement.minimum_alive))
        .collect();
    let mut roles: BTreeSet<EcosystemRole> = required.keys().copied().collect();
    roles.extend(members.iter().map(|member| member.role));

    roles
        .into_iter()
        .map(|role| {
            let required_alive = required.get(&role).copied().unwrap_or(0);
            let mut coverage = RoleCoverage {
                role,
                required_alive,
                total: 0,
                alive: 0,
                partial_alive: 0,
                unknown: 0,
                blocked: 0,
                build_broken: 0,
                unsupported: 0,
                satisfied: false,
            };
            for member in members.iter().filter(|member| member.role == role) {
                coverage.total += 1;
                match member.standing_receipt.standing {
                    Standing::Alive => coverage.alive += 1,
                    Standing::PartialAlive => coverage.partial_alive += 1,
                    Standing::Unknown => coverage.unknown += 1,
                    Standing::Blocked => coverage.blocked += 1,
                    Standing::BuildBroken => coverage.build_broken += 1,
                    Standing::Unsupported => coverage.unsupported += 1,
                }
            }
            coverage.satisfied = coverage.alive >= required_alive;
            coverage
        })
        .collect()
}

fn derive_standing(members: &[EcosystemMember], coverage: &[RoleCoverage]) -> Standing {
    if members.is_empty() {
        Standing::Unknown
    } else if coverage
        .iter()
        .filter(|role| role.required_alive > 0)
        .all(|role| role.satisfied)
    {
        Standing::Alive
    } else {
        Standing::PartialAlive
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_receipt_hash(
    profile: &str,
    claim_ceiling: &str,
    authority_ceiling: &str,
    subject: &SubjectIdentity,
    observation_commitment: &Blake3Hash,
    admitted_receipt_hash: &Blake3Hash,
    requirements: &[RoleRequirement],
    members: &[EcosystemMember],
    coverage: &[RoleCoverage],
    standing: Standing,
    previous_receipt: &Option<Blake3Hash>,
) -> Result<Blake3Hash, EcosystemRefusal> {
    let material = EcosystemMaterial {
        profile,
        claim_ceiling,
        authority_ceiling,
        subject,
        observation_commitment,
        admitted_receipt_hash,
        requirements,
        members,
        coverage,
        standing,
        previous_receipt,
    };
    let canonical = serde_json::to_vec(&material)
        .map_err(|error| EcosystemRefusal::Serialization(error.to_string()))?;
    let mut bytes = Vec::with_capacity(ECOSYSTEM_PROFILE.len() + canonical.len() + 1);
    bytes.extend_from_slice(ECOSYSTEM_PROFILE.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&canonical);
    Ok(Blake3Hash::from_bytes(&bytes))
}

fn require_text(field: &'static str, value: &str) -> Result<(), EcosystemRefusal> {
    if value.trim().is_empty() {
        Err(EcosystemRefusal::EmptyField(field))
    } else {
        Ok(())
    }
}

fn require_blake3(field: &'static str, value: &Blake3Hash) -> Result<(), EcosystemRefusal> {
    let hex = value.as_hex();
    if hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(EcosystemRefusal::MalformedBlake3(field))
    }
}

impl<'de> Deserialize<'de> for EcosystemReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        struct RawEcosystemReceipt {
            profile: String,
            claim_ceiling: String,
            authority_ceiling: String,
            subject: SubjectIdentity,
            observation_commitment: Blake3Hash,
            admitted_receipt_hash: Blake3Hash,
            requirements: Vec<RoleRequirement>,
            members: Vec<EcosystemMember>,
            coverage: Vec<RoleCoverage>,
            standing: Standing,
            previous_receipt: Option<Blake3Hash>,
            receipt_hash: Blake3Hash,
        }

        let raw = RawEcosystemReceipt::deserialize(deserializer)?;
        let receipt = EcosystemReceipt {
            profile: raw.profile,
            claim_ceiling: raw.claim_ceiling,
            authority_ceiling: raw.authority_ceiling,
            subject: raw.subject,
            observation_commitment: raw.observation_commitment,
            admitted_receipt_hash: raw.admitted_receipt_hash,
            requirements: raw.requirements,
            members: raw.members,
            coverage: raw.coverage,
            standing: raw.standing,
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
    use crate::standing::{
        certify_standing, ExecutionEvidence, ReplayEvidence, StandingObservation,
        VerificationEvidence,
    };
    use wasm4pm_compat::authority::{AuthorityConstraint, AuthorityEnvelope, Capability};
    use wasm4pm_compat::receipt::Digest;
    use wasm4pm_compat::witness::RustTypestateLaw;

    fn admitted_receipt() -> AdmittedReceipt {
        let mut assembler = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "ecosystem-test",
            vec![object_ref("ecosystem", "graph")],
            b"admitted ecosystem observation",
            &mut counter,
        )
        .expect("event");
        assembler.append(event).expect("append");
        crate::admission::admit(assembler.finalize()).expect("admitted")
    }

    fn authority(subject: &str) -> AuthorityEnvelope<RustTypestateLaw> {
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
            format!("repo:{subject}"),
        )
    }

    fn member_receipt(
        admitted: &AdmittedReceipt,
        subject: &str,
        standing: Standing,
    ) -> StandingReceipt {
        let complete = standing == Standing::Alive;
        certify_standing(
            admitted,
            &authority(subject),
            StandingObservation {
                subject: SubjectIdentity {
                    subject: subject.to_string(),
                    base: format!("base-{subject}"),
                    tree: format!("tree-{subject}"),
                    candidate: format!("candidate-{subject}"),
                },
                observation_commitment: Blake3Hash::from_bytes(subject.as_bytes()),
                standing,
                execution: complete.then(|| ExecutionEvidence {
                    command: format!("execute {subject}"),
                    exit_code: 0,
                    result_commitment: Blake3Hash::from_bytes(
                        format!("execution-{subject}").as_bytes(),
                    ),
                }),
                verification: complete.then(|| VerificationEvidence {
                    command: format!("verify {subject}"),
                    exit_code: 0,
                    report_commitment: Blake3Hash::from_bytes(
                        format!("verification-{subject}").as_bytes(),
                    ),
                }),
                replay: complete.then(|| ReplayEvidence {
                    command: format!("replay {subject}"),
                    environment_commitment: Blake3Hash::from_bytes(
                        format!("environment-{subject}").as_bytes(),
                    ),
                }),
                previous_receipt: None,
            },
        )
        .expect("member standing")
    }

    fn observation(admitted: &AdmittedReceipt) -> EcosystemObservation {
        EcosystemObservation {
            subject: SubjectIdentity {
                subject: "chatman/ecosystem".to_string(),
                base: "ecosystem-base".to_string(),
                tree: "ecosystem-tree".to_string(),
                candidate: "ecosystem-candidate".to_string(),
            },
            observation_commitment: Blake3Hash::from_bytes(b"ecosystem-O-star"),
            requirements: vec![
                RoleRequirement {
                    role: EcosystemRole::SemanticAuthority,
                    minimum_alive: 1,
                },
                RoleRequirement {
                    role: EcosystemRole::Manufacture,
                    minimum_alive: 1,
                },
            ],
            members: vec![
                EcosystemMember {
                    role: EcosystemRole::SemanticAuthority,
                    standing_receipt: member_receipt(
                        admitted,
                        "seanchatmangpt/ggen-marketplace",
                        Standing::Alive,
                    ),
                },
                EcosystemMember {
                    role: EcosystemRole::Manufacture,
                    standing_receipt: member_receipt(
                        admitted,
                        "seanchatmangpt/ggen",
                        Standing::Alive,
                    ),
                },
            ],
            previous_receipt: None,
        }
    }

    #[test]
    fn required_alive_roles_crown_the_exact_federation() {
        let admitted = admitted_receipt();
        let receipt = certify_ecosystem(&admitted, observation(&admitted)).expect("ecosystem");
        assert_eq!(receipt.standing, Standing::Alive);
        assert!(receipt.coverage.iter().all(|coverage| coverage.satisfied));
        assert!(receipt.verify().is_ok());
    }

    #[test]
    fn missing_required_role_is_partial_topology_not_global_failure() {
        let admitted = admitted_receipt();
        let mut observation = observation(&admitted);
        observation
            .members
            .retain(|member| member.role != EcosystemRole::Manufacture);
        let receipt = certify_ecosystem(&admitted, observation).expect("partial ecosystem");
        assert_eq!(receipt.standing, Standing::PartialAlive);
        let manufacture = receipt
            .coverage
            .iter()
            .find(|coverage| coverage.role == EcosystemRole::Manufacture)
            .expect("manufacture coverage");
        assert_eq!(manufacture.alive, 0);
        assert!(!manufacture.satisfied);
    }

    #[test]
    fn optional_build_broken_edge_does_not_collapse_satisfied_crown() {
        let admitted = admitted_receipt();
        let mut observation = observation(&admitted);
        observation
            .requirements
            .retain(|requirement| requirement.role == EcosystemRole::SemanticAuthority);
        observation.members.push(EcosystemMember {
            role: EcosystemRole::RuntimeExecution,
            standing_receipt: member_receipt(
                &admitted,
                "seanchatmangpt/optional-runtime",
                Standing::BuildBroken,
            ),
        });
        let receipt = certify_ecosystem(&admitted, observation).expect("bounded topology");
        assert_eq!(receipt.standing, Standing::Alive);
        let runtime = receipt
            .coverage
            .iter()
            .find(|coverage| coverage.role == EcosystemRole::RuntimeExecution)
            .expect("runtime coverage");
        assert_eq!(runtime.build_broken, 1);
        assert_eq!(runtime.required_alive, 0);
        assert!(runtime.satisfied);
    }

    #[test]
    fn duplicate_exact_member_is_typed_refusal() {
        let admitted = admitted_receipt();
        let mut observation = observation(&admitted);
        observation.members.push(observation.members[0].clone());
        assert!(matches!(
            certify_ecosystem(&admitted, observation),
            Err(EcosystemRefusal::DuplicateMember { .. })
        ));
    }

    #[test]
    fn input_order_does_not_change_federation_identity() {
        let admitted = admitted_receipt();
        let mut reversed = observation(&admitted);
        reversed.requirements.reverse();
        reversed.members.reverse();
        let a = certify_ecosystem(&admitted, observation(&admitted)).expect("a");
        let b = certify_ecosystem(&admitted, reversed).expect("b");
        assert_eq!(a.receipt_hash, b.receipt_hash);
    }

    #[test]
    fn tampered_serialized_federation_is_rejected() {
        let admitted = admitted_receipt();
        let receipt = certify_ecosystem(&admitted, observation(&admitted)).expect("receipt");
        let mut json = serde_json::to_value(&receipt).expect("json");
        json["subject"]["candidate"] = serde_json::Value::String("tampered".to_string());
        assert!(serde_json::from_value::<EcosystemReceipt>(json).is_err());
    }

    #[test]
    fn zero_quorum_is_refused_instead_of_becoming_vacuous_success() {
        let admitted = admitted_receipt();
        let mut observation = observation(&admitted);
        observation.requirements[0].minimum_alive = 0;
        assert!(matches!(
            certify_ecosystem(&admitted, observation),
            Err(EcosystemRefusal::ZeroAliveRequirement(_))
        ));
    }
}
