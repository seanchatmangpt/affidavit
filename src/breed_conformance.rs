//! Breed-aware conformance checking for receipt chains.
//!
//! Given a breed entry from the wasm4pm-cognition registry and a receipt,
//! checks whether the receipt's event sequence conforms to the breed's
//! declared stages. Produces a structured deviation report.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single entry from the breed registry (registry.json).
/// Only the fields we need for conformance are required; the rest are
/// captured in `extra` to avoid breaking on schema evolution.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreedEntry {
    pub breed_id: String,
    pub breed_name: String,
    #[serde(default)]
    pub status: String,
    /// Optional explicit stage list. When absent we synthesise a minimal
    /// single-stage model named after the breed_id itself.
    #[serde(default)]
    pub stages: Vec<String>,
    /// All other fields (historical_ancestor, standing, …) collected here.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl BreedEntry {
    /// The declared stage sequence for this breed.
    ///
    /// If the registry entry has an explicit `stages` list, return it;
    /// otherwise return a synthetic single-stage list `[<breed_id>]`.
    pub fn declared_stages(&self) -> Vec<String> {
        if !self.stages.is_empty() {
            self.stages.clone()
        } else {
            vec![self.breed_id.clone()]
        }
    }
}

/// Deviation details for a single receipt under breed conformance checking.
#[derive(Debug, Clone, Serialize)]
pub struct ReceiptDeviation {
    /// The receipt's chain_hash (serves as receipt identifier).
    pub receipt_id: String,
    /// Stages declared by the breed but absent from the receipt events.
    pub missing: Vec<String>,
    /// Event types present in the receipt but not in the breed's stage list.
    pub extra: Vec<String>,
    /// Pairs `(expected, actual)` where the sequence order diverges.
    pub order_violations: Vec<(String, String)>,
}

/// Top-level conformance report.
#[derive(Debug, Clone, Serialize)]
pub struct BreedConformanceReport {
    pub breed_id: String,
    pub breed_name: String,
    pub declared_stages: Vec<String>,
    pub total_receipts: usize,
    pub conformant_count: usize,
    pub deviations: Vec<ReceiptDeviation>,
}

/// Check whether the event sequence of `receipt` conforms to `breed`'s
/// declared stages and return a [`ReceiptDeviation`].
pub fn check_receipt(receipt: &crate::types::Receipt, breed: &BreedEntry) -> ReceiptDeviation {
    let declared = breed.declared_stages();
    let actual: Vec<String> = receipt
        .events
        .iter()
        .map(|e| e.event_type.clone())
        .collect();

    // Missing: declared stages not present anywhere in actual sequence.
    let missing: Vec<String> = declared
        .iter()
        .filter(|s| !actual.contains(s))
        .cloned()
        .collect();

    // Extra: actual event types not in declared stages.
    let mut seen_extra = std::collections::BTreeSet::new();
    let extra: Vec<String> = actual
        .iter()
        .filter(|s| {
            if !declared.contains(s) && seen_extra.insert((*s).clone()) {
                true
            } else {
                false
            }
        })
        .cloned()
        .collect();

    // Order violations: compare the sub-sequence of actual events that match
    // declared stage names against the declared order.
    let actual_matching: Vec<&String> = actual.iter().filter(|s| declared.contains(s)).collect();

    // Build expected order from declared stages, filtering to only those present.
    let expected_order: Vec<&String> = declared.iter().filter(|s| actual.contains(s)).collect();

    let order_violations: Vec<(String, String)> = actual_matching
        .iter()
        .zip(expected_order.iter())
        .filter(|(a, e)| a != e)
        .map(|(a, e)| ((*e).clone(), (*a).clone()))
        .collect();

    ReceiptDeviation {
        receipt_id: receipt.chain_hash.as_hex().to_string(),
        missing,
        extra,
        order_violations,
    }
}

/// Run breed conformance over a slice of receipts and produce the full report.
pub fn run_conformance(
    receipts: &[crate::types::Receipt],
    breed: &BreedEntry,
) -> BreedConformanceReport {
    let declared_stages = breed.declared_stages();
    let mut deviations = Vec::new();
    let mut conformant_count = 0usize;

    for receipt in receipts {
        let dev = check_receipt(receipt, breed);
        let is_conformant =
            dev.missing.is_empty() && dev.extra.is_empty() && dev.order_violations.is_empty();
        if is_conformant {
            conformant_count += 1;
        } else {
            deviations.push(dev);
        }
    }

    BreedConformanceReport {
        breed_id: breed.breed_id.clone(),
        breed_name: breed.breed_name.clone(),
        declared_stages,
        total_receipts: receipts.len(),
        conformant_count,
        deviations,
    }
}

/// Load the breed registry from a JSON file path.
///
/// The registry is an array of breed entries (possibly with duplicate breed_ids;
/// we return the first match for the requested breed_id).
pub fn load_registry(path: &str) -> anyhow::Result<Vec<BreedEntry>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("failed to read registry at {path}: {e}"))?;
    let entries: Vec<BreedEntry> = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse registry JSON at {path}: {e}"))?;
    Ok(entries)
}

/// The embedded fallback registry — a minimal set of well-known breeds with
/// their canonical stage sequences. Used when `--registry` is not supplied.
pub fn embedded_registry() -> Vec<BreedEntry> {
    // We ship a small subset of the wasm4pm-cognition registry inline so the
    // feature works without an external file.  The canonical registry is at
    // `crates/wasm4pm-cognition/breeds/registry.json` in the wasm4pm workspace.
    let raw = include_str!("../fixtures/embedded_registry.json");
    serde_json::from_str(raw).unwrap_or_default()
}

/// Find a breed entry by breed_id in the given registry (first match wins).
pub fn find_breed<'a>(registry: &'a [BreedEntry], breed_id: &str) -> Option<&'a BreedEntry> {
    registry.iter().find(|e| e.breed_id == breed_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};

    fn make_receipt(activities: &[&str]) -> crate::types::Receipt {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for (i, act) in activities.iter().enumerate() {
            let ev = build_event(
                *act,
                vec![object_ref(&format!("obj-{i}"), "artifact")],
                act.as_bytes(),
                &mut counter,
            )
            .expect("build event");
            asm.append(ev).expect("append");
        }
        asm.finalize()
    }

    fn breed(id: &str, stages: &[&str]) -> BreedEntry {
        BreedEntry {
            breed_id: id.to_string(),
            breed_name: id.to_string(),
            status: "PARTIAL_ALIVE".to_string(),
            stages: stages.iter().map(|s| s.to_string()).collect(),
            extra: Default::default(),
        }
    }

    #[test]
    fn conformant_receipt_produces_no_deviations() {
        let b = breed("mycin", &["intake", "rule_eval", "conclusion"]);
        let receipt = make_receipt(&["intake", "rule_eval", "conclusion"]);
        let report = run_conformance(&[receipt], &b);

        assert_eq!(report.conformant_count, 1, "should be conformant");
        assert!(report.deviations.is_empty(), "no deviations expected");
    }

    #[test]
    fn missing_stage_is_reported() {
        let b = breed("mycin", &["intake", "rule_eval", "conclusion"]);
        // missing "rule_eval"
        let receipt = make_receipt(&["intake", "conclusion"]);
        let report = run_conformance(&[receipt], &b);

        assert_eq!(report.conformant_count, 0);
        assert_eq!(report.deviations.len(), 1);
        assert!(
            report.deviations[0]
                .missing
                .contains(&"rule_eval".to_string()),
            "rule_eval should be in missing"
        );
    }

    #[test]
    fn extra_stage_is_reported() {
        let b = breed("mycin", &["intake", "conclusion"]);
        // extra "rule_eval"
        let receipt = make_receipt(&["intake", "rule_eval", "conclusion"]);
        let report = run_conformance(&[receipt], &b);

        assert_eq!(report.conformant_count, 0);
        assert!(
            report.deviations[0]
                .extra
                .contains(&"rule_eval".to_string()),
            "rule_eval should be in extra"
        );
    }

    #[test]
    fn order_violation_is_reported() {
        let b = breed("mycin", &["intake", "rule_eval", "conclusion"]);
        // wrong order: conclusion before rule_eval
        let receipt = make_receipt(&["intake", "conclusion", "rule_eval"]);
        let report = run_conformance(&[receipt], &b);

        assert_eq!(report.conformant_count, 0);
        assert!(
            !report.deviations[0].order_violations.is_empty(),
            "order violations expected"
        );
    }

    #[test]
    fn empty_stages_uses_breed_id_as_sole_stage() {
        let b = breed("dempster_shafer", &[]);
        assert_eq!(b.declared_stages(), vec!["dempster_shafer"]);
    }
}
