//! COMBINATORIAL MAXIMALISM: Feature 2.1 (Model Logic)
//!
//! Implement the full conversion from Receipt events to OCEL and subsequently 
//! to a Petri Net model using wasm4pm-compat patterns. No placeholders.
//!
//! This module provides the core process mining logic for Affidavit, bridging
//! the gap between the immutable event chain (Receipt) and the discovered 
//! process model (Petri Net). It leverages the object-centric nature of 
//! OCEL to discover causal dependencies that are more nuanced than simple 
//! linear sequences.

use std::collections::{HashMap, HashSet};
use crate::types::{Receipt, AdmittedReceipt};
use wasm4pm_compat::ocel::{OCEL, OCELEvent, OCELObject, OCELRelationship};
use wasm4pm_compat::petri::{Arc, Marking, PetriNet, Place, Transition};
use thiserror::Error;

/// Errors raised during the process mining lifecycle.
#[derive(Debug, Error)]
pub enum MiningError {
    /// Failed to mine a Petri net using the Heuristic Inductive Miner.
    #[error("wasm4pm HIM failed: {0}")]
    Him(String),
    
    /// The receipt was not admitted or failed admission checks.
    #[error("admission refused: {0}")]
    Admission(String),
    
    /// The conversion from Receipt to OCEL format failed.
    #[error("OCEL conversion failed: {0}")]
    OcelConversion(String),
}

/// Convert an Affidavit [`Receipt`] into an [`OCEL`] 2.0 log.
///
/// This projection preserves the object-centric relationships defined in the 
/// receipt, mapping every [`crate::types::OperationEvent`] to an [`OCELEvent`] 
/// and ensuring all referenced objects are declared in the log.
pub fn receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> {
    let mut objects_map = HashMap::new();
    let mut ocel_events = Vec::new();

    for ev in &receipt.events {
        let mut ocel_ev = OCELEvent::new(ev.id.clone(), ev.event_type.clone());
        
        for obj_ref in &ev.objects {
            // Ensure the object exists in the object set.
            objects_map.entry(obj_ref.id.clone()).or_insert_with(|| {
                OCELObject::new(obj_ref.id.clone(), obj_ref.obj_type.clone())
            });

            // Add the relationship between this event and the object.
            // We preserve the qualifier (role) if present in the receipt.
            let qualifier = obj_ref.qualifier.clone().unwrap_or_else(|| "unspecified".to_string());
            ocel_ev.relationships.push(OCELRelationship {
                object_id: obj_ref.id.clone(),
                qualifier,
            });
        }
        ocel_events.push(ocel_ev);
    }

    let ocel_objects: Vec<_> = objects_map.into_values().collect();
    
    // Construct the OCEL log. wasm4pm-compat handles the internal indexing.
    Ok(OCEL::new(ocel_events, ocel_objects))
}

/// Discover a Petri Net model from an admitted receipt using a maximalist 
/// Heuristic Inductive Miner (HIM) pattern.
///
/// The discovery follows these steps:
/// 1. **Transition Discovery**: Every unique `event_type` in the log becomes 
///     a visible transition in the Petri net.
/// 2. **Causal Discovery**: Analyzes the log for "directly-follows" relations. 
///    It uses both a global sequence perspective and an object-centric 
///    perspective (tracing individual object lifecycles).
/// 3. **Place Synthesis**: Creates places to represent the observed causality:
///    - A `start` place feeding all potential initial activities.
///    - Causal places `p_a_b` for every observed transition from `a` to `b`.
///    - An `end` place collecting all terminal activities.
/// 4. **Initial Marking**: Seeds the `start` place with a single token.
pub fn discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> {
    // Stage 1: Projection to OCEL
    let ocel = receipt_to_ocel(&admitted.value)?;
    
    let mut activities = HashSet::new();
    let mut transitions = Vec::new();
    let mut places = Vec::new();
    let mut arcs = Vec::new();

    // Stage 2: Transition Discovery (AC-1.9)
    // We iterate through the event set and create one transition per activity type.
    for ev in ocel.event_set() {
        if activities.insert(ev.event_type.clone()) {
            transitions.push(Transition::new(ev.event_type.clone(), ev.event_type.clone()));
        }
    }
    // Ensure deterministic output by sorting transitions.
    transitions.sort_by(|a, b| a.id().cmp(b.id()));

    // Stage 3: Node/Transition Discovery Logic (Handover & Causality)
    let mut object_traces: HashMap<String, Vec<String>> = HashMap::new();
    let mut global_trace: Vec<String> = Vec::new();

    // Trace discovery: populate traces from the OCEL log.
    for ev in ocel.event_set() {
        global_trace.push(ev.event_type.clone());
        for (obj_id, _) in ocel.e2o(&ev.id) {
            object_traces.entry(obj_id.to_string()).or_default().push(ev.event_type.clone());
        }
    }

    let mut causal_pairs = HashSet::new();
    let mut starts = HashSet::new();
    let mut ends = HashSet::new();

    // Extract causal relations from individual object lifecycles.
    for trace in object_traces.values() {
        if let Some(first) = trace.first() { starts.insert(first.clone()); }
        if let Some(last) = trace.last() { ends.insert(last.clone()); }
        for window in trace.windows(2) {
            causal_pairs.insert((window[0].clone(), window[1].clone()));
        }
    }

    // Fallback/Augmentation: use global trace to ensure the net is fully connected 
    // even if some events lack object associations.
    if let Some(first) = global_trace.first() { starts.insert(first.clone()); }
    if let Some(last) = global_trace.last() { ends.insert(last.clone()); }
    for window in global_trace.windows(2) {
        causal_pairs.insert((window[0].clone(), window[1].clone()));
    }

    // Stage 4: Synthesis of Places and Arcs.
    
    // Start Place -> Initial Transitions
    places.push(Place::new("start"));
    for start_act in sorted_set(starts) {
        arcs.push(Arc::place_to_transition("start", start_act));
    }

    // Intermediate Causal Places
    for (a, b) in sorted_pairs(causal_pairs) {
        let place_id = format!("p_{}_{}", a, b);
        places.push(Place::new(place_id.clone()));
        arcs.push(Arc::transition_to_place(a, place_id.clone()));
        arcs.push(Arc::place_to_transition(place_id, b));
    }

    // Final Transitions -> End Place
    places.push(Place::new("end"));
    for end_act in sorted_set(ends) {
        arcs.push(Arc::transition_to_place(end_act, "end"));
    }

    // Stage 5: Initial Marking
    let marking = Marking::new([("start".to_string(), 1)]);

    // Construct the final Petri net.
    Ok(PetriNet::new(places, transitions, arcs, marking))
}

/// Helper to sort a set of strings for deterministic processing.
fn sorted_set(set: HashSet<String>) -> Vec<String> {
    let mut v: Vec<_> = set.into_iter().collect();
    v.sort();
    v
}

/// Helper to sort a set of activity pairs for deterministic processing.
fn sorted_pairs(set: HashSet<(String, String)>) -> Vec<(String, String)> {
    let mut v: Vec<_> = set.into_iter().collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::chain::ChainAssembler;
    use crate::admission::admit;

    #[test]
    fn test_maximalist_discovery_linear() {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();

        let ev1 = build_event("create", vec![object_ref("o1", "artifact")], b"1", &mut counter).unwrap();
        let ev2 = build_event("update", vec![object_ref("o1", "artifact")], b"2", &mut counter).unwrap();
        let ev3 = build_event("delete", vec![object_ref("o1", "artifact")], b"3", &mut counter).unwrap();

        asm.append(ev1).unwrap();
        asm.append(ev2).unwrap();
        asm.append(ev3).unwrap();

        let receipt = asm.finalize();
        let admitted = admit(receipt).expect("admission must pass");

        let net = discover_him(&admitted).expect("discovery must pass");

        assert_eq!(net.transitions().len(), 3, "three activities -> three transitions");
        assert!(net.places().iter().any(|p| p.id() == "start"));
        assert!(net.places().iter().any(|p| p.id() == "end"));
        assert!(net.places().iter().any(|p| p.id() == "p_create_update"));
        assert!(net.places().iter().any(|p| p.id() == "p_update_delete"));
    }
}
