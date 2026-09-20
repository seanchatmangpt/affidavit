//! GALL evidence-only Chicago standing crown.
//!
//! This module certifies supplied witnesses. It never performs manufacture,
//! planning, actuation, observation, or fresh-consumer work and therefore
//! cannot manufacture missing evidence or repair a moved semantic subject.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

const GATE_COUNT: u8 = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateStatus {
    Pass,
    Open,
    Refused,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateWitness {
    pub gate: u8,
    pub name: String,
    pub status: GateStatus,
    pub subject_digest: String,
    pub evidence_digest: String,
    pub positive_observed: bool,
    pub falsifier_required: bool,
    pub falsifier_attempted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredecessorWitness {
    pub receipt_iri: String,
    pub receipt_digest: String,
    pub work_order_iri: String,
    pub graph_digest: String,
    pub repository_identity: String,
    pub head_sha: String,
    pub standing: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrownManifest {
    pub schema: String,
    pub composition_digest: String,
    /// Legacy digest-only list retained for wire compatibility. When typed
    /// witnesses are supplied, the two digest sets must correspond exactly.
    #[serde(default)]
    pub predecessor_receipts: Vec<String>,
    #[serde(default)]
    pub predecessor_witnesses: Vec<PredecessorWitness>,
    pub gates: Vec<GateWitness>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CrownStanding {
    Alive,
    PartialAlive,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrownReceipt {
    pub schema: String,
    pub composition_digest: String,
    pub predecessor_receipts: Vec<String>,
    pub predecessor_witnesses: Vec<PredecessorWitness>,
    pub gates: Vec<GateWitness>,
    pub standing: CrownStanding,
    pub evidence_ceiling: String,
    pub receipt_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrownRefusal {
    InvalidCompositionDigest,
    InvalidReceiptDigest(String),
    InvalidPredecessorField { field: &'static str, value: String },
    DuplicatePredecessorWorkOrder(String),
    PredecessorReceiptMismatch,
    PredecessorNotAlive(String),
    WrongGateSet(Vec<u8>),
    DuplicateGate(u8),
    GateSubjectMismatch { gate: u8 },
    VacuousPass { gate: u8 },
    MissingFalsifier { gate: u8 },
    Gate12NotPositive,
}

fn digest_shape(value: &str) -> bool {
    let Some((_algorithm, hex)) = value.split_once(':') else {
        return false;
    };
    !hex.is_empty() && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sha256_digest(value: &str) -> bool {
    let Some(("sha256", hex)) = value.split_once(':') else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn git_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn absolute_iri(value: &str) -> bool {
    !value.is_empty() && value.contains(':')
}

fn repository_identity(value: &str) -> bool {
    let mut parts = value.split('/');
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(owner), Some(repo), None) if !owner.is_empty() && !repo.is_empty()
    )
}

fn receipt_digest(payload: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(payload).to_hex())
}

fn canonical_receipt_bytes(receipt: &CrownReceipt) -> Vec<u8> {
    let mut unsigned = receipt.clone();
    unsigned.receipt_digest.clear();
    serde_json::to_vec(&unsigned).expect("CrownReceipt is serializable")
}

fn validate_predecessors(manifest: &CrownManifest) -> Result<(), CrownRefusal> {
    for digest in &manifest.predecessor_receipts {
        if !digest_shape(digest) {
            return Err(CrownRefusal::InvalidReceiptDigest(digest.clone()));
        }
    }

    if manifest.predecessor_witnesses.is_empty() {
        return Ok(());
    }

    let mut work_orders = HashSet::new();
    let mut witness_digests = BTreeSet::new();

    for witness in &manifest.predecessor_witnesses {
        if !absolute_iri(&witness.receipt_iri) {
            return Err(CrownRefusal::InvalidPredecessorField {
                field: "receipt_iri",
                value: witness.receipt_iri.clone(),
            });
        }
        if !digest_shape(&witness.receipt_digest) {
            return Err(CrownRefusal::InvalidReceiptDigest(
                witness.receipt_digest.clone(),
            ));
        }
        if !absolute_iri(&witness.work_order_iri) {
            return Err(CrownRefusal::InvalidPredecessorField {
                field: "work_order_iri",
                value: witness.work_order_iri.clone(),
            });
        }
        if !sha256_digest(&witness.graph_digest) {
            return Err(CrownRefusal::InvalidPredecessorField {
                field: "graph_digest",
                value: witness.graph_digest.clone(),
            });
        }
        if !repository_identity(&witness.repository_identity) {
            return Err(CrownRefusal::InvalidPredecessorField {
                field: "repository_identity",
                value: witness.repository_identity.clone(),
            });
        }
        if !git_sha(&witness.head_sha) {
            return Err(CrownRefusal::InvalidPredecessorField {
                field: "head_sha",
                value: witness.head_sha.clone(),
            });
        }
        if witness.standing != "ALIVE" {
            return Err(CrownRefusal::PredecessorNotAlive(
                witness.work_order_iri.clone(),
            ));
        }
        if !work_orders.insert(witness.work_order_iri.clone()) {
            return Err(CrownRefusal::DuplicatePredecessorWorkOrder(
                witness.work_order_iri.clone(),
            ));
        }
        witness_digests.insert(witness.receipt_digest.clone());
    }

    if !manifest.predecessor_receipts.is_empty() {
        let legacy: BTreeSet<_> = manifest.predecessor_receipts.iter().cloned().collect();
        if legacy != witness_digests {
            return Err(CrownRefusal::PredecessorReceiptMismatch);
        }
    }

    Ok(())
}

/// Certify the supplied twelve-gate witness bundle.
///
/// Success means the format and evidence relationships are certified. The
/// returned standing may still be PARTIAL_ALIVE or REFUSED; this verifier
/// does not turn an open/refused gate into execution evidence.
pub fn certify_gall_crown(manifest: &CrownManifest) -> Result<CrownReceipt, CrownRefusal> {
    if !digest_shape(&manifest.composition_digest) {
        return Err(CrownRefusal::InvalidCompositionDigest);
    }
    validate_predecessors(manifest)?;

    let mut gates = manifest.gates.clone();
    gates.sort_by_key(|gate| gate.gate);
    let numbers: Vec<u8> = gates.iter().map(|gate| gate.gate).collect();
    let expected: Vec<u8> = (1..=GATE_COUNT).collect();
    if numbers != expected {
        for pair in numbers.windows(2) {
            if pair[0] == pair[1] {
                return Err(CrownRefusal::DuplicateGate(pair[0]));
            }
        }
        return Err(CrownRefusal::WrongGateSet(numbers));
    }

    for gate in &gates {
        if gate.subject_digest != manifest.composition_digest {
            return Err(CrownRefusal::GateSubjectMismatch { gate: gate.gate });
        }
        if !digest_shape(&gate.evidence_digest) {
            return Err(CrownRefusal::InvalidReceiptDigest(
                gate.evidence_digest.clone(),
            ));
        }
        if gate.status == GateStatus::Pass && !gate.positive_observed {
            return Err(CrownRefusal::VacuousPass { gate: gate.gate });
        }
        if gate.status == GateStatus::Pass
            && gate.falsifier_required
            && !gate.falsifier_attempted
        {
            return Err(CrownRefusal::MissingFalsifier { gate: gate.gate });
        }
    }

    let gate11 = &gates[10];
    let gate12 = &gates[11];
    if gate12.status == GateStatus::Pass && !gate12.positive_observed {
        return Err(CrownRefusal::Gate12NotPositive);
    }

    let standing = if gates.iter().all(|gate| gate.status == GateStatus::Pass) {
        CrownStanding::Alive
    } else if gates.iter().any(|gate| gate.status == GateStatus::Refused) {
        CrownStanding::Refused
    } else {
        CrownStanding::PartialAlive
    };

    let evidence_ceiling = if gate11.status != GateStatus::Pass {
        "Gate 11 fresh-consumer evidence remains open; cross-repository standing cannot exceed PARTIAL_ALIVE"
            .to_string()
    } else if gate12.status != GateStatus::Pass {
        "Gate 12 positive KNOWN execution remains open; cross-repository standing cannot be ALIVE"
            .to_string()
    } else if manifest.predecessor_witnesses.is_empty() {
        "All 12 gates pass, but predecessor receipts are digest-only legacy evidence; typed cross-repository subject correspondence remains unproven"
            .to_string()
    } else {
        "All 12 supplied Chicago gate witnesses and typed predecessor subjects satisfy the structural court"
            .to_string()
    };

    let mut receipt = CrownReceipt {
        schema: "affidavit.gall.chicago-crown/v26.9.19".to_string(),
        composition_digest: manifest.composition_digest.clone(),
        predecessor_receipts: manifest.predecessor_receipts.clone(),
        predecessor_witnesses: manifest.predecessor_witnesses.clone(),
        gates,
        standing,
        evidence_ceiling,
        receipt_digest: String::new(),
    };
    receipt.receipt_digest = receipt_digest(&canonical_receipt_bytes(&receipt));
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(seed: &str) -> String {
        format!("sha256:{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn predecessor(index: usize) -> PredecessorWitness {
        PredecessorWitness {
            receipt_iri: format!("urn:gall:receipt:{index}"),
            receipt_digest: digest(&format!("receipt-{index}")),
            work_order_iri: format!("urn:gall:work-order:{index}"),
            graph_digest: format!("sha256:{}", "a".repeat(64)),
            repository_identity: format!("seanchatmangpt/repo-{index}"),
            head_sha: format!("{:040x}", index + 1),
            standing: "ALIVE".to_string(),
        }
    }

    fn manifest(gate11: GateStatus, gate12: GateStatus) -> CrownManifest {
        let composition = digest("composition");
        let predecessor_witnesses: Vec<_> = (1..=6).map(predecessor).collect();

        CrownManifest {
            schema: "gall.chicago-manifest/v26.9.19".to_string(),
            composition_digest: composition.clone(),
            predecessor_receipts: predecessor_witnesses
                .iter()
                .map(|witness| witness.receipt_digest.clone())
                .collect(),
            predecessor_witnesses,
            gates: (1..=12)
                .map(|gate| GateWitness {
                    gate,
                    name: format!("Gate {gate}"),
                    status: if gate == 11 {
                        gate11.clone()
                    } else if gate == 12 {
                        gate12.clone()
                    } else {
                        GateStatus::Pass
                    },
                    subject_digest: composition.clone(),
                    evidence_digest: digest(&format!("gate-{gate}")),
                    positive_observed: true,
                    falsifier_required: true,
                    falsifier_attempted: true,
                })
                .collect(),
        }
    }

    #[test]
    fn all_twelve_pass_with_typed_predecessors_issues_alive_receipt() {
        let receipt = certify_gall_crown(&manifest(GateStatus::Pass, GateStatus::Pass))
            .expect("valid crown");
        assert_eq!(receipt.standing, CrownStanding::Alive);
        assert_eq!(receipt.predecessor_witnesses.len(), 6);
        assert!(receipt.receipt_digest.starts_with("blake3:"));
    }

    #[test]
    fn moved_predecessor_digest_is_refused() {
        let mut witness = manifest(GateStatus::Pass, GateStatus::Pass);
        witness.predecessor_receipts[0] = digest("different");
        assert_eq!(
            certify_gall_crown(&witness),
            Err(CrownRefusal::PredecessorReceiptMismatch)
        );
    }

    #[test]
    fn duplicate_predecessor_work_order_is_refused() {
        let mut witness = manifest(GateStatus::Pass, GateStatus::Pass);
        witness.predecessor_witnesses[1].work_order_iri =
            witness.predecessor_witnesses[0].work_order_iri.clone();

        assert!(matches!(
            certify_gall_crown(&witness),
            Err(CrownRefusal::DuplicatePredecessorWorkOrder(_))
        ));
    }

    #[test]
    fn non_alive_predecessor_cannot_enter_alive_crown() {
        let mut witness = manifest(GateStatus::Pass, GateStatus::Pass);
        witness.predecessor_witnesses[0].standing = "PARTIAL_ALIVE".to_string();

        assert!(matches!(
            certify_gall_crown(&witness),
            Err(CrownRefusal::PredecessorNotAlive(_))
        ));
    }

    #[test]
    fn open_gate_11_caps_standing_at_partial_alive() {
        let receipt = certify_gall_crown(&manifest(GateStatus::Open, GateStatus::Pass))
            .expect("structurally valid partial crown");
        assert_eq!(receipt.standing, CrownStanding::PartialAlive);
        assert!(receipt.evidence_ceiling.contains("Gate 11"));
    }

    #[test]
    fn vacuous_pass_is_refused() {
        let mut witness = manifest(GateStatus::Pass, GateStatus::Pass);
        witness.gates[4].positive_observed = false;
        assert_eq!(
            certify_gall_crown(&witness),
            Err(CrownRefusal::VacuousPass { gate: 5 })
        );
    }

    #[test]
    fn missing_required_falsifier_is_refused() {
        let mut witness = manifest(GateStatus::Pass, GateStatus::Pass);
        witness.gates[7].falsifier_attempted = false;
        assert_eq!(
            certify_gall_crown(&witness),
            Err(CrownRefusal::MissingFalsifier { gate: 8 })
        );
    }

    #[test]
    fn moved_composition_subject_is_refused() {
        let mut witness = manifest(GateStatus::Pass, GateStatus::Pass);
        witness.gates[0].subject_digest = digest("moved");
        assert_eq!(
            certify_gall_crown(&witness),
            Err(CrownRefusal::GateSubjectMismatch { gate: 1 })
        );
    }
}
