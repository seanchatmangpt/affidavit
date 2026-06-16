// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Aggregate module list for the rendered verb wrappers (rendered from O* by ggen).
// Consumed query column (verbs-mod.rq): modules.

//! Rendered verb modules.

pub mod assemble;
pub mod audit;
pub mod catalog;
pub mod conformance;
pub mod diagnose;
pub mod diff;
pub mod emit;
pub mod graph;
pub mod inspect;
pub mod model;
pub mod profile;
pub mod receipt_throughput;
pub mod replay;
pub mod show;
pub mod stats;
pub mod variance;
pub mod verify;
pub mod visualize;

use crate::types::Receipt;
use std::collections::HashMap;

/// `affi receipt inspect` — detailed receipt analysis using test fixtures.
///
/// Uses chicago-tdd-tools fixtures to decompose receipts into readable patterns.
/// This is the first verb enabled by 80/20 (fixture logic from chicago-tdd-tools,
/// thin glue here).
pub fn inspect_with_fixtures(receipt: &Receipt) -> String {
    let mut output = String::new();

    output.push_str("RECEIPT INSPECTION REPORT\n");
    output.push_str("=========================\n\n");

    output.push_str(&format!("Format: {}\n", receipt.format_version));
    output.push_str(&format!("Total events: {}\n", receipt.events.len()));
    output.push_str(&format!("Chain hash: {}\n\n", receipt.chain_hash));

    // Event distribution analysis
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for event in &receipt.events {
        *type_counts.entry(event.event_type.clone()).or_insert(0) += 1;
    }

    output.push_str("Event types:\n");
    for (ty, count) in &type_counts {
        output.push_str(&format!("  {}: {} events\n", ty, count));
    }

    // Object coverage
    let mut obj_types: HashMap<String, usize> = HashMap::new();
    for event in &receipt.events {
        for obj in &event.objects {
            *obj_types.entry(obj.obj_type.clone()).or_insert(0) += 1;
        }
    }

    if !obj_types.is_empty() {
        output.push_str("\nObject types:\n");
        for (ty, count) in &obj_types {
            output.push_str(&format!("  {}: {} references\n", ty, count));
        }
    }

    output.push_str("\nInspection complete.\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::*;

    #[test]
    fn inspect_generates_detailed_report() {
        let mut asm = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();

        let event1 = build_event(
            "create",
            vec![object_ref("file-1", "artifact")],
            b"initial content",
            &mut counter,
        )
        .expect("build event 1");
        asm.append(event1).expect("append event 1");

        let event2 = build_event(
            "modify",
            vec![
                object_ref("file-1", "artifact"),
                object_ref("user-42", "agent"),
            ],
            b"modified content",
            &mut counter,
        )
        .expect("build event 2");
        asm.append(event2).expect("append event 2");

        let receipt = asm.finalize();
        let report = inspect_with_fixtures(&receipt);

        assert!(report.contains("RECEIPT INSPECTION REPORT"));
        assert!(report.contains("create: 1 events"));
        assert!(report.contains("modify: 1 events"));
        assert!(report.contains("artifact: 2 references"));
        assert!(report.contains("agent: 1 references"));
    }
}
