//! Working-backwards Design for Combinatorial Maximalism release closure.
//!
//! DfCM treats the terminal crown as the specification. Repository state is
//! evidence material. Failed alternatives remain topology rather than collapsing
//! the graph. This module certifies evidence only; it grants no actuation authority.

use crate::standing::{Standing, StandingReceipt};
use crate::types::Blake3Hash;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DFCM_PROFILE: &str = "affidavit/dfcm/v1";
pub const DFCM_CLAIM_CEILING: &str =
    "EXACT_SUBJECT_RELEASE_CLOSURE_ONLY_NO_RUNTIME_MERGE_PUBLICATION_OR_DEPLOYMENT_INFERENCE";
pub const DFCM_AUTHORITY_CEILING: &str = "EVIDENCE_ONLY_NO_SELECT_CONSTRUCT_OR_DO_AUTHORITY";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceKind {
    ImplementedSource,
    ObservedExecution,
    ExactHeadCourt,
    HostedCi,
    RuntimeStanding,
    Merge,
    Publication,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExactSubject {
    pub repository: String,
    pub candidate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EvidenceKey {
    pub kind: EvidenceKind,
    pub court: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceWitness {
    pub key: EvidenceKey,
    pub commitment: Blake3Hash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectObservation {
    pub subject: ExactSubject,
    pub standing: Standing,
    pub evidence: Vec<EvidenceWitness>,
    pub standing_receipt: Option<StandingReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectRequirement {
    pub subject: ExactSubject,
    pub required_evidence: Vec<EvidenceKey>,
    pub required_standing: Option<Standing>,
    pub require_standing_receipt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofPath {
    pub name: String,
    pub subjects: Vec<SubjectRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obligation {
    pub id: String,
    pub description: String,
    pub paths: Vec<ProofPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DfcmProfile {
    pub release: String,
    pub certifier: ExactSubject,
    pub frontier_budget: usize,
    pub obligations: Vec<Obligation>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClosureGap {
    MissingSubject {
        repository: String,
        expected_candidate: String,
    },
    ExactSubjectMoved {
        repository: String,
        expected_candidate: String,
        observed_candidates: Vec<String>,
    },
    MissingEvidence {
        subject: ExactSubject,
        evidence: EvidenceKey,
    },
    StandingMismatch {
        subject: ExactSubject,
        required: String,
        observed: String,
    },
    MissingStandingReceipt {
        subject: ExactSubject,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathEvaluation {
    pub name: String,
    pub satisfied: bool,
    pub gaps: Vec<ClosureGap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObligationEvaluation {
    pub obligation: String,
    pub satisfied: bool,
    pub paths: Vec<PathEvaluation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinimalFrontier {
    pub gaps: Vec<ClosureGap>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct DfcmReceipt {
    pub profile: String,
    pub claim_ceiling: String,
    pub authority_ceiling: String,
    pub release_profile: DfcmProfile,
    pub observations: Vec<SubjectObservation>,
    pub obligations: Vec<ObligationEvaluation>,
    pub minimal_frontiers: Vec<MinimalFrontier>,
    pub closed: bool,
    pub receipt_hash: Blake3Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DfcmRefusal {
    EmptyField(&'static str),
    NoObligations,
    ZeroFrontierBudget,
    DuplicateObligation(String),
    NoProofPaths(String),
    DuplicatePath(String, String),
    EmptyPath(String, String),
    DuplicateSubjectRequirement(ExactSubject),
    DuplicateEvidenceRequirement(ExactSubject, EvidenceKey),
    DuplicateObservation(ExactSubject),
    DuplicateEvidenceWitness(ExactSubject, EvidenceKey),
    MalformedBlake3(&'static str),
    StandingReceiptInvalid(ExactSubject, String),
    StandingReceiptSubjectMismatch(ExactSubject, ExactSubject),
    StandingReceiptStandingMismatch(ExactSubject, String, String),
    FrontierBudgetExceeded { budget: usize, required: usize },
    NonCanonicalOrder(&'static str),
    WrongProfile,
    WrongClaimCeiling,
    WrongAuthorityCeiling,
    ObligationEvaluationMismatch,
    FrontierMismatch,
    ClosureMismatch,
    ReceiptHashMismatch,
    Serialization(String),
}

impl core::fmt::Display for DfcmRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for DfcmRefusal {}

#[derive(Serialize)]
struct Material<'a> {
    profile: &'a str,
    claim_ceiling: &'a str,
    authority_ceiling: &'a str,
    release_profile: &'a DfcmProfile,
    observations: &'a [SubjectObservation],
    obligations: &'a [ObligationEvaluation],
    minimal_frontiers: &'a [MinimalFrontier],
    closed: bool,
}

pub fn certify_dfcm(
    mut release_profile: DfcmProfile,
    mut observations: Vec<SubjectObservation>,
) -> Result<DfcmReceipt, DfcmRefusal> {
    canonicalize(&mut release_profile, &mut observations);
    validate_profile(&release_profile)?;
    validate_observations(&observations)?;
    let (obligations, minimal_frontiers, closed) = evaluate(&release_profile, &observations)?;
    let receipt_hash = hash_material(
        DFCM_PROFILE,
        DFCM_CLAIM_CEILING,
        DFCM_AUTHORITY_CEILING,
        &release_profile,
        &observations,
        &obligations,
        &minimal_frontiers,
        closed,
    )?;
    Ok(DfcmReceipt {
        profile: DFCM_PROFILE.into(),
        claim_ceiling: DFCM_CLAIM_CEILING.into(),
        authority_ceiling: DFCM_AUTHORITY_CEILING.into(),
        release_profile,
        observations,
        obligations,
        minimal_frontiers,
        closed,
        receipt_hash,
    })
}

impl DfcmReceipt {
    pub fn verify(&self) -> Result<(), DfcmRefusal> {
        if self.profile != DFCM_PROFILE {
            return Err(DfcmRefusal::WrongProfile);
        }
        if self.claim_ceiling != DFCM_CLAIM_CEILING {
            return Err(DfcmRefusal::WrongClaimCeiling);
        }
        if self.authority_ceiling != DFCM_AUTHORITY_CEILING {
            return Err(DfcmRefusal::WrongAuthorityCeiling);
        }
        require_canonical(&self.release_profile, &self.observations)?;
        validate_profile(&self.release_profile)?;
        validate_observations(&self.observations)?;
        let (obligations, frontiers, closed) = evaluate(&self.release_profile, &self.observations)?;
        if obligations != self.obligations {
            return Err(DfcmRefusal::ObligationEvaluationMismatch);
        }
        if frontiers != self.minimal_frontiers {
            return Err(DfcmRefusal::FrontierMismatch);
        }
        if closed != self.closed {
            return Err(DfcmRefusal::ClosureMismatch);
        }
        let hash = hash_material(
            &self.profile,
            &self.claim_ceiling,
            &self.authority_ceiling,
            &self.release_profile,
            &self.observations,
            &self.obligations,
            &self.minimal_frontiers,
            self.closed,
        )?;
        if hash != self.receipt_hash {
            return Err(DfcmRefusal::ReceiptHashMismatch);
        }
        Ok(())
    }
}

fn evaluate(
    profile: &DfcmProfile,
    observations: &[SubjectObservation],
) -> Result<(Vec<ObligationEvaluation>, Vec<MinimalFrontier>, bool), DfcmRefusal> {
    let by_exact: BTreeMap<ExactSubject, &SubjectObservation> = observations
        .iter()
        .map(|o| (o.subject.clone(), o))
        .collect();
    let mut by_repo: BTreeMap<&str, Vec<&SubjectObservation>> = BTreeMap::new();
    for o in observations {
        by_repo.entry(&o.subject.repository).or_default().push(o);
    }

    let mut result = Vec::new();
    for obligation in &profile.obligations {
        let mut paths = Vec::new();
        for path in &obligation.paths {
            let mut gaps = BTreeSet::new();
            for req in &path.subjects {
                let Some(obs) = by_exact.get(&req.subject).copied() else {
                    let mut candidates: Vec<String> = by_repo
                        .get(req.subject.repository.as_str())
                        .map(|xs| xs.iter().map(|x| x.subject.candidate.clone()).collect())
                        .unwrap_or_default();
                    candidates.sort();
                    candidates.dedup();
                    if candidates.is_empty() {
                        gaps.insert(ClosureGap::MissingSubject {
                            repository: req.subject.repository.clone(),
                            expected_candidate: req.subject.candidate.clone(),
                        });
                    } else {
                        gaps.insert(ClosureGap::ExactSubjectMoved {
                            repository: req.subject.repository.clone(),
                            expected_candidate: req.subject.candidate.clone(),
                            observed_candidates: candidates,
                        });
                    }
                    continue;
                };
                if let Some(required) = req.required_standing {
                    if obs.standing != required {
                        gaps.insert(ClosureGap::StandingMismatch {
                            subject: req.subject.clone(),
                            required: standing_name(required).into(),
                            observed: standing_name(obs.standing).into(),
                        });
                    }
                }
                if req.require_standing_receipt && obs.standing_receipt.is_none() {
                    gaps.insert(ClosureGap::MissingStandingReceipt {
                        subject: req.subject.clone(),
                    });
                }
                for required in &req.required_evidence {
                    if !obs.evidence.iter().any(|w| &w.key == required) {
                        gaps.insert(ClosureGap::MissingEvidence {
                            subject: req.subject.clone(),
                            evidence: required.clone(),
                        });
                    }
                }
            }
            paths.push(PathEvaluation {
                name: path.name.clone(),
                satisfied: gaps.is_empty(),
                gaps: gaps.into_iter().collect(),
            });
        }
        result.push(ObligationEvaluation {
            obligation: obligation.id.clone(),
            satisfied: paths.iter().any(|p| p.satisfied),
            paths,
        });
    }
    let closed = result.iter().all(|o| o.satisfied);
    let frontiers = if closed {
        Vec::new()
    } else {
        minimal_frontiers(&result, profile.frontier_budget)?
    };
    Ok((result, frontiers, closed))
}

fn minimal_frontiers(
    obligations: &[ObligationEvaluation],
    budget: usize,
) -> Result<Vec<MinimalFrontier>, DfcmRefusal> {
    let mut frontiers = vec![BTreeSet::new()];
    for obligation in obligations.iter().filter(|o| !o.satisfied) {
        let paths: Vec<_> = obligation.paths.iter().filter(|p| !p.satisfied).collect();
        let required = frontiers.len().saturating_mul(paths.len());
        if required > budget {
            return Err(DfcmRefusal::FrontierBudgetExceeded { budget, required });
        }
        let mut next = Vec::with_capacity(required);
        for existing in &frontiers {
            for path in &paths {
                let mut combined = existing.clone();
                combined.extend(path.gaps.iter().cloned());
                next.push(combined);
            }
        }
        frontiers = minimize(next);
    }
    Ok(frontiers
        .into_iter()
        .map(|gaps| MinimalFrontier {
            gaps: gaps.into_iter().collect(),
        })
        .collect())
}

fn minimize(mut values: Vec<BTreeSet<ClosureGap>>) -> Vec<BTreeSet<ClosureGap>> {
    values.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.iter().cmp(b.iter())));
    values.dedup();
    let mut out = Vec::<BTreeSet<ClosureGap>>::new();
    'candidate: for candidate in values {
        if out.iter().any(|x| x.is_subset(&candidate)) {
            continue 'candidate;
        }
        out.push(candidate);
    }
    out
}

fn validate_profile(profile: &DfcmProfile) -> Result<(), DfcmRefusal> {
    text("release", &profile.release)?;
    exact("certifier", &profile.certifier)?;
    if profile.frontier_budget == 0 {
        return Err(DfcmRefusal::ZeroFrontierBudget);
    }
    if profile.obligations.is_empty() {
        return Err(DfcmRefusal::NoObligations);
    }
    let mut obligation_ids = BTreeSet::new();
    for o in &profile.obligations {
        text("obligation.id", &o.id)?;
        text("obligation.description", &o.description)?;
        if !obligation_ids.insert(o.id.clone()) {
            return Err(DfcmRefusal::DuplicateObligation(o.id.clone()));
        }
        if o.paths.is_empty() {
            return Err(DfcmRefusal::NoProofPaths(o.id.clone()));
        }
        let mut path_names = BTreeSet::new();
        for p in &o.paths {
            text("path.name", &p.name)?;
            if !path_names.insert(p.name.clone()) {
                return Err(DfcmRefusal::DuplicatePath(o.id.clone(), p.name.clone()));
            }
            if p.subjects.is_empty() {
                return Err(DfcmRefusal::EmptyPath(o.id.clone(), p.name.clone()));
            }
            let mut subjects = BTreeSet::new();
            for r in &p.subjects {
                exact("subject_requirement", &r.subject)?;
                if !subjects.insert(r.subject.clone()) {
                    return Err(DfcmRefusal::DuplicateSubjectRequirement(r.subject.clone()));
                }
                let mut evidence = BTreeSet::new();
                for e in &r.required_evidence {
                    text("evidence.court", &e.court)?;
                    if !evidence.insert(e.clone()) {
                        return Err(DfcmRefusal::DuplicateEvidenceRequirement(
                            r.subject.clone(),
                            e.clone(),
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_observations(observations: &[SubjectObservation]) -> Result<(), DfcmRefusal> {
    let mut subjects = BTreeSet::new();
    for o in observations {
        exact("observation.subject", &o.subject)?;
        if !subjects.insert(o.subject.clone()) {
            return Err(DfcmRefusal::DuplicateObservation(o.subject.clone()));
        }
        let mut evidence = BTreeSet::new();
        for w in &o.evidence {
            text("evidence.court", &w.key.court)?;
            hash("evidence.commitment", &w.commitment)?;
            if !evidence.insert(w.key.clone()) {
                return Err(DfcmRefusal::DuplicateEvidenceWitness(
                    o.subject.clone(),
                    w.key.clone(),
                ));
            }
        }
        if let Some(receipt) = &o.standing_receipt {
            receipt.verify().map_err(|e| {
                DfcmRefusal::StandingReceiptInvalid(o.subject.clone(), e.to_string())
            })?;
            let nested = ExactSubject {
                repository: receipt.subject.subject.clone(),
                candidate: receipt.subject.candidate.clone(),
            };
            if nested != o.subject {
                return Err(DfcmRefusal::StandingReceiptSubjectMismatch(
                    o.subject.clone(),
                    nested,
                ));
            }
            if receipt.standing != o.standing {
                return Err(DfcmRefusal::StandingReceiptStandingMismatch(
                    o.subject.clone(),
                    standing_name(o.standing).into(),
                    standing_name(receipt.standing).into(),
                ));
            }
        }
    }
    Ok(())
}

fn canonicalize(profile: &mut DfcmProfile, observations: &mut Vec<SubjectObservation>) {
    profile.obligations.sort_by(|a, b| a.id.cmp(&b.id));
    for o in &mut profile.obligations {
        o.paths.sort_by(|a, b| a.name.cmp(&b.name));
        for p in &mut o.paths {
            p.subjects.sort_by(|a, b| a.subject.cmp(&b.subject));
            for r in &mut p.subjects {
                r.required_evidence.sort();
            }
        }
    }
    observations.sort_by(|a, b| a.subject.cmp(&b.subject));
    for o in observations {
        o.evidence.sort_by(|a, b| a.key.cmp(&b.key));
    }
}

fn require_canonical(
    profile: &DfcmProfile,
    observations: &[SubjectObservation],
) -> Result<(), DfcmRefusal> {
    let mut p = profile.clone();
    let mut o = observations.to_vec();
    canonicalize(&mut p, &mut o);
    if &p != profile {
        return Err(DfcmRefusal::NonCanonicalOrder("release_profile"));
    }
    if o != observations {
        return Err(DfcmRefusal::NonCanonicalOrder("observations"));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn hash_material(
    profile: &str,
    claim_ceiling: &str,
    authority_ceiling: &str,
    release_profile: &DfcmProfile,
    observations: &[SubjectObservation],
    obligations: &[ObligationEvaluation],
    minimal_frontiers: &[MinimalFrontier],
    closed: bool,
) -> Result<Blake3Hash, DfcmRefusal> {
    let material = Material {
        profile,
        claim_ceiling,
        authority_ceiling,
        release_profile,
        observations,
        obligations,
        minimal_frontiers,
        closed,
    };
    let canonical =
        serde_json::to_vec(&material).map_err(|e| DfcmRefusal::Serialization(e.to_string()))?;
    let mut bytes = Vec::with_capacity(DFCM_PROFILE.len() + canonical.len() + 1);
    bytes.extend_from_slice(DFCM_PROFILE.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&canonical);
    Ok(Blake3Hash::from_bytes(&bytes))
}

fn text(field: &'static str, value: &str) -> Result<(), DfcmRefusal> {
    if value.trim().is_empty() {
        Err(DfcmRefusal::EmptyField(field))
    } else {
        Ok(())
    }
}
fn exact(field: &'static str, value: &ExactSubject) -> Result<(), DfcmRefusal> {
    if value.repository.trim().is_empty() || value.candidate.trim().is_empty() {
        Err(DfcmRefusal::EmptyField(field))
    } else {
        Ok(())
    }
}
fn hash(field: &'static str, value: &Blake3Hash) -> Result<(), DfcmRefusal> {
    let value = value.as_hex();
    if value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(DfcmRefusal::MalformedBlake3(field))
    }
}
fn standing_name(value: Standing) -> &'static str {
    match value {
        Standing::Unknown => "UNKNOWN",
        Standing::PartialAlive => "PARTIAL_ALIVE",
        Standing::Alive => "ALIVE",
        Standing::Blocked => "BLOCKED",
        Standing::BuildBroken => "BUILD_BROKEN",
        Standing::Unsupported => "UNSUPPORTED",
    }
}

impl<'de> Deserialize<'de> for DfcmReceipt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        #[derive(Deserialize)]
        struct Raw {
            profile: String,
            claim_ceiling: String,
            authority_ceiling: String,
            release_profile: DfcmProfile,
            observations: Vec<SubjectObservation>,
            obligations: Vec<ObligationEvaluation>,
            minimal_frontiers: Vec<MinimalFrontier>,
            closed: bool,
            receipt_hash: Blake3Hash,
        }
        let raw = Raw::deserialize(deserializer)?;
        let receipt = DfcmReceipt {
            profile: raw.profile,
            claim_ceiling: raw.claim_ceiling,
            authority_ceiling: raw.authority_ceiling,
            release_profile: raw.release_profile,
            observations: raw.observations,
            obligations: raw.obligations,
            minimal_frontiers: raw.minimal_frontiers,
            closed: raw.closed,
            receipt_hash: raw.receipt_hash,
        };
        receipt.verify().map_err(D::Error::custom)?;
        Ok(receipt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V26_9_18Subjects {
    pub affidavit: ExactSubject,
    pub ggen: ExactSubject,
    pub bcinr: ExactSubject,
    pub ash_r2rml: ExactSubject,
    pub ggen_igniter: ExactSubject,
    pub wasm4pm: ExactSubject,
    pub unrdf: ExactSubject,
}

pub fn v26_9_18_profile(s: V26_9_18Subjects) -> DfcmProfile {
    let alive = Some(Standing::Alive);
    let req = |subject: &ExactSubject, court: &str, kinds: &[EvidenceKind]| SubjectRequirement {
        subject: subject.clone(),
        required_evidence: kinds
            .iter()
            .copied()
            .map(|kind| EvidenceKey {
                kind,
                court: court.into(),
            })
            .collect(),
        required_standing: alive,
        require_standing_receipt: true,
    };
    let closure = [
        EvidenceKind::ImplementedSource,
        EvidenceKind::ObservedExecution,
        EvidenceKind::ExactHeadCourt,
        EvidenceKind::HostedCi,
    ];
    let one =
        |id: &str, description: &str, name: &str, subjects: Vec<SubjectRequirement>| Obligation {
            id: id.into(),
            description: description.into(),
            paths: vec![ProofPath {
                name: name.into(),
                subjects,
            }],
        };
    DfcmProfile {
        release: "v26.9.18".into(),
        certifier: s.affidavit,
        frontier_budget: 1,
        obligations: vec![
            one(
                "SEMANTIC_SUBJECT",
                "admitted semantic subject closes manufacture and replay identity",
                "ggen-semantic-authority",
                vec![req(&s.ggen, "ggen/v26.9.18/semantic-manufacture-closure", &closure)],
            ),
            one(
                "BOUNDED_ALLOCATION",
                "CMCA/MFW allocation is bounded, typed, deterministic, and grants no DO authority",
                "bcinr-cmca-mfw",
                vec![req(&s.bcinr, "bcinr/v26.9.18/cmca-mfw-closure", &closure)],
            ),
            one(
                "PROJECTED_MANUFACTURE",
                "semantic source, projection, and manufacturer identities stay bound and drift fails closed",
                "r2rml-plus-igniter",
                vec![
                    req(&s.ash_r2rml, "ash_r2rml/v26.9.18/projection-closure", &closure),
                    req(&s.ggen_igniter, "ggen_igniter/v26.9.18/manufacture-closure", &closure),
                ],
            ),
            one(
                "PORTABLE_GRAPHLAW",
                "content-addressed GraphLaw has equivalent bounded verdict evidence across supported hosts",
                "wasm4pm-portable-law",
                vec![req(&s.wasm4pm, "wasm4pm/v26.9.18/portable-graphlaw", &closure)],
            ),
            one(
                "BOUNDED_RUNTIME",
                "real AtomVM compute cell executes bounded exact-subject work and survives loss and drain",
                "unrdf-atomvm-cell",
                vec![req(
                    &s.unrdf,
                    "unrdf/v26.9.18/atomvm-runtime",
                    &[
                        EvidenceKind::ImplementedSource,
                        EvidenceKind::ObservedExecution,
                        EvidenceKind::ExactHeadCourt,
                        EvidenceKind::HostedCi,
                        EvidenceKind::RuntimeStanding,
                    ],
                )],
            ),
            one(
                "KNOWN_REPLAY",
                "qualified UNKNOWN experience returns through ordinary manufacture and replays as KNOWN",
                "unknown-to-known-loop",
                vec![
                    req(&s.ggen, "v26.9.18/unknown-to-known-replay", &closure[1..]),
                    req(&s.bcinr, "v26.9.18/unknown-to-known-replay", &closure[1..]),
                    req(&s.ash_r2rml, "v26.9.18/unknown-to-known-replay", &closure[1..]),
                    req(&s.ggen_igniter, "v26.9.18/unknown-to-known-replay", &closure[1..]),
                ],
            ),
        ],
    }
}
