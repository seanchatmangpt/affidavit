//! Core diffing logic for `affi receipt diff` (Micro-Task 1.2).
//!
//! This module provides the `DiffResult` structure and the algorithm to
//! compare two `Receipt` objects, identifying structural changes in the
//! event chain.
//!
//! Per DOD_PHASE1_INSPECTION §2.2, events are matched by their sequence number (seq).

use crate::types::{OperationEvent, Receipt};
use serde::{Deserialize, Serialize};

/// A summary entry for an event in a diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffEntry {
    /// The monotonic sequence number of the event.
    pub seq: u64,
    /// The kind of operation.
    pub event_type: String,
    /// First 12 hex characters of the payload commitment.
    pub commitment_prefix: String,
}

impl From<&OperationEvent> for DiffEntry {
    fn from(ev: &OperationEvent) -> Self {
        DiffEntry {
            seq: ev.seq,
            event_type: ev.event_type.clone(),
            commitment_prefix: ev.payload_commitment.as_hex().chars().take(12).collect(),
        }
    }
}

/// A detailed record of a modified event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifiedEntry {
    /// The sequence number of the event that was modified.
    pub seq: u64,
    /// The event entry as it existed in the original receipt.
    pub old: DiffEntry,
    /// The event entry as it exists in the new receipt.
    pub new: DiffEntry,
}

/// The result of a diff operation between two receipts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DiffResult {
    /// Events that appear in the new receipt but were not in the old one.
    pub added: Vec<DiffEntry>,
    /// Events that were in the old receipt but are missing from the new one.
    pub removed: Vec<DiffEntry>,
    /// Events present in both receipts but with differing fields.
    pub modified: Vec<ModifiedEntry>,
}

impl DiffResult {
    /// Returns true if there are no differences between the receipts.
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }
}

/// Compute the difference between two receipts based on event sequence numbers.
///
/// Per DOD_PHASE1_INSPECTION §2.2:
/// - Iterates events by seq in O(n) using sorted index.
/// - A seq present in `a` but not `b` -> `removed`
/// - A seq present in `b` but not `a` -> `added`
/// - A seq present in both but with different `event_type` OR different `payload_commitment` -> `modified`
pub fn diff_receipts(old: &Receipt, new: &Receipt) -> DiffResult {
    use std::collections::BTreeMap;

    let mut old_map = BTreeMap::new();
    for ev in &old.events {
        old_map.insert(ev.seq, ev);
    }

    let mut new_map = BTreeMap::new();
    for ev in &new.events {
        new_map.insert(ev.seq, ev);
    }

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    // Find removed and modified
    for (&seq, &old_ev) in &old_map {
        match new_map.get(&seq) {
            Some(&new_ev) => {
                if old_ev.event_type != new_ev.event_type
                    || old_ev.payload_commitment != new_ev.payload_commitment
                {
                    modified.push(ModifiedEntry {
                        seq,
                        old: DiffEntry::from(old_ev),
                        new: DiffEntry::from(new_ev),
                    });
                }
            }
            None => {
                removed.push(DiffEntry::from(old_ev));
            }
        }
    }

    // Find added
    for (&seq, &new_ev) in &new_map {
        if !old_map.contains_key(&seq) {
            added.push(DiffEntry::from(new_ev));
        }
    }

    DiffResult {
        added,
        removed,
        modified,
    }
}

/// Parse two JSON strings into Receipts and compute their difference.
pub fn diff_json_receipts(old_json: &str, new_json: &str) -> anyhow::Result<DiffResult> {
    let old: Receipt = serde_json::from_str(old_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse old receipt: {e}"))?;
    let new: Receipt = serde_json::from_str(new_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse new receipt: {e}"))?;

    Ok(diff_receipts(&old, &new))
}
