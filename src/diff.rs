//! Core diffing logic for `affi receipt diff` (Micro-Task 1.2).
//!
//! This module provides the `DiffResult` structure and the algorithm to
//! compare two `Receipt` objects, identifying structural changes in the
//! event chain.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::{Receipt, OperationEvent};

/// The result of a diff operation between two receipts.
///
/// Encapsulates all additions, removals, and modifications found in the
/// second receipt relative to the first.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffResult {
    /// Events that appear in the new receipt but were not in the old one.
    pub added: Vec<OperationEvent>,
    /// Events that were in the old receipt but are missing from the new one.
    pub removed: Vec<OperationEvent>,
    /// Events present in both receipts but with differing fields (e.g. payload commitment).
    pub modified: Vec<ModifiedEvent>,
}

/// A detailed record of a modified event.
///
/// Contains both the original and updated state of the event to allow
/// for granular inspection of changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifiedEvent {
    /// The stable ID of the event that was modified.
    pub id: String,
    /// The event as it existed in the original receipt.
    pub old: OperationEvent,
    /// The event as it exists in the new receipt.
    pub new: OperationEvent,
}

impl DiffResult {
    /// Returns true if there are no differences between the receipts.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }
}

/// Compute the difference between two receipts based on event IDs.
///
/// This algorithm performs a set-based comparison of event IDs to find additions
/// and removals, and a field-wise comparison for events present in both.
/// It respects the unique ID of each event as the matching key.
pub fn diff_receipts(old: &Receipt, new: &Receipt) -> DiffResult {
    let old_map: HashMap<String, &OperationEvent> = old.events.iter()
        .map(|e| (e.id.clone(), e))
        .collect();
    
    let new_map: HashMap<String, &OperationEvent> = new.events.iter()
        .map(|e| (e.id.clone(), e))
        .collect();

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();

    // Scan the new receipt for additions and modifications.
    for event in &new.events {
        match old_map.get(&event.id) {
            Some(old_event) => {
                if *old_event != event {
                    modified.push(ModifiedEvent {
                        id: event.id.clone(),
                        old: (*old_event).clone(),
                        new: event.clone(),
                    });
                }
            }
            None => {
                added.push(event.clone());
            }
        }
    }

    // Scan the old receipt for removals.
    for event in &old.events {
        if !new_map.contains_key(&event.id) {
            removed.push(event.clone());
        }
    }

    DiffResult {
        added,
        removed,
        modified,
    }
}

/// Parse two JSON strings into Receipts and compute their difference.
///
/// Useful for CLI entry points where receipts are provided as raw JSON text.
/// This utilizes the custom deserializer in `crate::types::Receipt` which
/// automatically re-verifies the BLAKE3 chain hash during parsing.
pub fn diff_json_receipts(old_json: &str, new_json: &str) -> anyhow::Result<DiffResult> {
    let old: Receipt = serde_json::from_str(old_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse old receipt: {e}"))?;
    let new: Receipt = serde_json::from_str(new_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse new receipt: {e}"))?;
    
    Ok(diff_receipts(&old, &new))
}
