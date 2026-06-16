//! Feature 2.3: Predict Logic (Maximalist Implementation)
//!
//! This module implements next-activity prediction and confidence scoring
//! based on a discovered Directly-Follows Graph (DFG). It handles multiple
//! candidate branches with frequency-based weighting, fulfilling the
//! requirements of the COMBINATORIAL MAXIMALISM mandate.
//!
//! # Logic
//! 1. Project the admitted receipt into a wasm4pm EventLog.
//! 2. Discover a Directly-Follows Graph (DFG) from the log (frequency-weighted).
//! 3. Identify the "current state" (the last activity in the receipt).
//! 4. Find all activities that followed this state in the DFG.
//! 5. Calculate confidence scores based on the relative frequencies of these transitions.
//! 6. Return a ranked report of top-K predictions.

use crate::types::AdmittedReceipt;
use crate::discovery::{project_to_event_log, ACTIVITY_KEY};
use wasm4pm::ilp_discovery::discover_optimized_dfg_from_log;
use wasm4pm::models::DFG;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// A single predicted next activity with its associated confidence score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityPrediction {
    /// The predicted next activity label (matches an `event_type` from the receipt).
    pub activity: String,
    /// Confidence in [0.0, 1.0]; sum over all predictions for a state is ≤ 1.0.
    pub confidence: f64,
}

/// The final report produced by the prediction engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PredictionReport {
    /// Top-K ranked next-activity predictions in descending confidence order.
    pub predictions: Vec<ActivityPrediction>,
    /// Number of events in the receipt used as context for prediction.
    pub context_length: usize,
    /// Identifier of the underlying model used for prediction.
    pub model_type: String,
}

/// Errors that can occur during prediction.
#[derive(Debug, thiserror::Error)]
pub enum PredictionError {
    /// The requested top-K value was invalid (e.g., 0).
    #[error("invalid top-k: {0} (must be at least 1)")]
    InvalidTopK(usize),
    /// An error occurred in the underlying wasm4pm discovery engine.
    #[error("wasm4pm discovery failed: {0}")]
    Wasm4pm(String),
}

/// Predict the top-K most likely next activities for the receipt's current trace,
/// using a separate model receipt as the source of learned behavior.
///
/// # Arguments
/// * `model` - The receipt containing the process behavior to learn from.
/// * `current_trace` - The partial receipt we want to predict the next activity for.
/// * `top_k` - Maximum number of predictions to return.
pub fn predict_next_with_model(
    model: &AdmittedReceipt,
    current_trace: &AdmittedReceipt,
    top_k: usize,
) -> Result<PredictionReport, PredictionError> {
    if top_k == 0 {
        return Err(PredictionError::InvalidTopK(0));
    }

    let model_receipt = &model.value;
    let trace_receipt = &current_trace.value;
    let context_length = trace_receipt.events.len();

    // 1. Project model to EventLog and discover DFG
    let log = project_to_event_log(model_receipt);
    let dfg = discover_optimized_dfg_from_log(&log, ACTIVITY_KEY, 1.0, 1.0);

    // 2. Identify candidates from the trace's current state
    let candidates = if trace_receipt.events.is_empty() {
        let total_starts: usize = dfg.start_activities.values().sum();
        if total_starts > 0 {
            dfg.start_activities.iter()
                .map(|(id, &freq)| {
                    let label = find_label(&dfg, id);
                    (label, freq as f64 / total_starts as f64)
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    } else {
        let last_activity = &trace_receipt.events.last().unwrap().event_type;
        let node_ids: Vec<&String> = dfg.nodes.iter()
            .filter(|n| &n.label == last_activity)
            .map(|n| &n.id)
            .collect();

        let mut next_freqs: HashMap<String, f64> = HashMap::new();
        let mut total_node_freq = 0.0;

        for &node_id in &node_ids {
            if let Some(node) = dfg.nodes.iter().find(|n| &n.id == node_id) {
                total_node_freq += node.frequency as f64;
            }
            for edge in &dfg.edges {
                if &edge.from == node_id {
                    let target_label = find_label(&dfg, &edge.to);
                    *next_freqs.entry(target_label).or_insert(0.0) += edge.frequency as f64;
                }
            }
        }

        if total_node_freq > 0.0 {
            next_freqs.into_iter()
                .map(|(label, freq)| (label, freq / total_node_freq))
                .collect()
        } else {
            vec![]
        }
    };

    // 3. Assemble and rank
    let mut predictions: Vec<ActivityPrediction> = candidates.into_iter()
        .map(|(activity, confidence)| ActivityPrediction { activity, confidence })
        .collect();

    predictions.sort_by(|a, b| {
        b.confidence.partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.activity.cmp(&b.activity))
    });

    predictions.truncate(top_k);

    Ok(PredictionReport {
        predictions,
        context_length,
        model_type: "Directly-Follows Graph (DFG)".to_string(),
    })
}

/// Predict the top-K most likely next activities for the receipt's current trace.
///
/// This is a convenience wrapper that uses the receipt itself as the model.
pub fn predict_next(
    admitted: &AdmittedReceipt,
    top_k: usize,
) -> Result<PredictionReport, PredictionError> {
    predict_next_with_model(admitted, admitted, top_k)
}

/// Helper to map a DFG node ID back to its activity label.
fn find_label(dfg: &DFG, id: &str) -> String {
    dfg.nodes.iter()
        .find(|n| n.id == id)
        .map(|n| n.label.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::admission::admit;

    fn test_receipt(activities: &[&str]) -> AdmittedReceipt {
        let mut asm = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for (i, &act) in activities.iter().enumerate() {
            let ev = build_event(
                act,
                vec![object_ref(&format!("obj-{}", i), "artifact")],
                act.as_bytes(),
                &mut counter,
            ).unwrap();
            asm.append(ev).unwrap();
        }
        admit(asm.finalize()).expect("test receipt must be admittable")
    }

    #[test]
    fn predicts_sequential_activity_with_full_confidence() {
        let model = test_receipt(&["A", "B", "C"]);
        let prefix = test_receipt(&["A"]);
        
        let report = predict_next_with_model(&model, &prefix, 5).unwrap();
        assert_eq!(report.predictions.len(), 1);
        assert_eq!(report.predictions[0].activity, "B");
        assert_eq!(report.predictions[0].confidence, 1.0);
    }

    #[test]
    fn handles_multiple_branches_with_weighting() {
        // Trace: A, B, A, C, A, B.
        // A is followed by B twice and C once.
        // Total occurrences of A = 3.
        let model = test_receipt(&["A", "B", "A", "C", "A", "B"]);
        let prefix = test_receipt(&["A", "B", "A", "C", "A"]); // ends in A
        
        let report = predict_next_with_model(&model, &prefix, 5).unwrap();
        
        // Expected: B with 2/3, C with 1/3
        assert_eq!(report.predictions.len(), 2);
        
        let b = report.predictions.iter().find(|p| p.activity == "B").unwrap();
        let c = report.predictions.iter().find(|p| p.activity == "C").unwrap();
        
        assert!((b.confidence - 0.6666).abs() < 0.001);
        assert!((c.confidence - 0.3333).abs() < 0.001);
        
        // Ranking: B should be first
        assert_eq!(report.predictions[0].activity, "B");
    }

    #[test]
    fn returns_empty_predictions_for_unknown_state() {
        let model = test_receipt(&["A", "B"]);
        let prefix = test_receipt(&["C"]); // C is not in model
        
        let report = predict_next_with_model(&model, &prefix, 5).unwrap();
        assert!(report.predictions.is_empty());
    }

    #[test]
    fn respects_top_k_limit() {
        let model = test_receipt(&["A", "B", "A", "C", "A", "D"]);
        let prefix = test_receipt(&["A"]);
        
        let report = predict_next_with_model(&model, &prefix, 2).unwrap();
        assert_eq!(report.predictions.len(), 2);
    }
}
