# DX/QOL 1000x — Concrete Code Templates

**Purpose:** Copy-paste-ready code for Phase 1 features  
**Status:** Ready to implement; test in sandbox first

---

## Template 1: Receipt Inspection Handler

**File:** `src/handlers.rs` (NEW FUNCTION)

```rust
use std::collections::{HashMap, BTreeMap};
use crate::types::{Receipt, OperationEvent};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InspectionReport {
    /// Count of each event type
    pub event_types: BTreeMap<String, usize>,
    /// Count of each object type
    pub object_types: BTreeMap<String, usize>,
    /// Total number of events
    pub event_count: usize,
    /// Total number of unique objects
    pub object_count: usize,
    /// Chain hash (last hash in chain)
    pub chain_hash: String,
    /// Verifier verdict for this receipt
    pub verdict_accepted: bool,
    /// Average payload commitment length (bytes)
    pub avg_commitment_len: usize,
}

/// Inspect a receipt and return a detailed analysis report.
///
/// # 80/20 Note
/// 80%: chicago-tdd-tools fixture structure + test patterns
/// 20%: This ~60-line function that aggregates and reports
pub fn inspect(receipt: &Receipt) -> InspectionReport {
    let mut event_types: BTreeMap<String, usize> = BTreeMap::new();
    let mut object_types: BTreeMap<String, usize> = BTreeMap::new();
    let mut object_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut commitment_lengths = Vec::new();

    // Aggregate event and object statistics
    for event in &receipt.events {
        *event_types.entry(event.event_type.clone()).or_insert(0) += 1;

        for obj_ref in &event.objects {
            *object_types.entry(obj_ref.obj_type.clone()).or_insert(0) += 1;
            object_ids.insert(obj_ref.id.clone());
        }

        commitment_lengths.push(event.payload_commitment.len());
    }

    let avg_commitment_len = if !commitment_lengths.is_empty() {
        commitment_lengths.iter().sum::<usize>() / commitment_lengths.len()
    } else {
        0
    };

    // Verify receipt to get verdict
    let verdict = crate::verifier::verify(receipt);

    InspectionReport {
        event_types,
        object_types,
        event_count: receipt.events.len(),
        object_count: object_ids.len(),
        chain_hash: receipt.chain_hash.clone(),
        verdict_accepted: verdict.accepted,
        avg_commitment_len,
    }
}

/// Pretty-print an inspection report
pub fn format_inspection_report(report: &InspectionReport) -> String {
    let mut output = String::new();

    output.push_str("=== Receipt Inspection Report ===\n");
    output.push_str(&format!("Events: {}\n", report.event_count));
    output.push_str(&format!("Objects: {}\n", report.object_count));
    output.push_str(&format!("Chain Hash: {}\n", report.chain_hash));
    output.push_str(&format!("Status: {}\n", if report.verdict_accepted { "✓ Valid" } else { "✗ Invalid" }));

    output.push_str("\nEvent Types:\n");
    for (event_type, count) in &report.event_types {
        output.push_str(&format!("  {}: {} events\n", event_type, count));
    }

    output.push_str("\nObject Types:\n");
    for (obj_type, count) in &report.object_types {
        output.push_str(&format!("  {}: {} references\n", obj_type, count));
    }

    output.push_str(&format!("\nAvg Commitment Size: {} bytes\n", report.avg_commitment_len));
    output
}
```

**Usage in CLI:**
```rust
// In src/handlers.rs or delegated from src/verbs/inspect.rs
pub fn handle_inspect(receipt_path: &str, format: Option<&str>) -> anyhow::Result<String> {
    let receipt = crate::chain::deserialize_receipt(receipt_path)?;
    let report = inspect(&receipt);

    match format {
        Some("json") => Ok(serde_json::to_string_pretty(&report)?),
        _ => Ok(format_inspection_report(&report)),
    }
}
```

---

## Template 2: Receipt Diff Handler

**File:** `src/handlers.rs` (NEW FUNCTION)

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiffResult {
    /// Events only in the first receipt
    pub added: Vec<OperationEvent>,
    /// Events only in the second receipt
    pub removed: Vec<OperationEvent>,
    /// Events present in both but with different commitment
    pub modified: Vec<(OperationEvent, OperationEvent)>,
    /// Total change count
    pub change_count: usize,
}

/// Compare two receipts and return their differences.
///
/// Uses a simple O(n) algorithm:
/// 1. Iterate through both event lists by seq number
/// 2. If seq matches, compare event_type + commitment
/// 3. If commitment differs, it's modified
/// 4. If seq only in first, it's added; only in second, it's removed
pub fn diff_receipts(a: &Receipt, b: &Receipt) -> DiffResult {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    let a_by_seq: BTreeMap<u32, &OperationEvent> = a.events.iter().map(|e| (e.seq, e)).collect();
    let b_by_seq: BTreeMap<u32, &OperationEvent> = b.events.iter().map(|e| (e.seq, e)).collect();

    let all_seqs: std::collections::HashSet<u32> =
        a_by_seq.keys().chain(b_by_seq.keys()).copied().collect();

    for seq in all_seqs {
        match (a_by_seq.get(&seq), b_by_seq.get(&seq)) {
            (Some(&a_event), Some(&b_event)) => {
                // Both have this seq; check if modified
                if a_event.event_type != b_event.event_type
                    || a_event.payload_commitment != b_event.payload_commitment
                {
                    modified.push((a_event.clone(), b_event.clone()));
                }
            }
            (Some(&a_event), None) => {
                // Only in a (added relative to b)
                added.push(a_event.clone());
            }
            (None, Some(&b_event)) => {
                // Only in b (removed relative to a)
                removed.push(b_event.clone());
            }
            (None, None) => {} // Impossible
        }
    }

    let change_count = added.len() + removed.len() + modified.len();

    DiffResult {
        added,
        removed,
        modified,
        change_count,
    }
}

pub fn format_diff_report(result: &DiffResult) -> String {
    let mut output = String::new();

    output.push_str("=== Receipt Diff ===\n");
    output.push_str(&format!("Changes: {}\n\n", result.change_count));

    if !result.added.is_empty() {
        output.push_str(&format!("+ Added ({}) events:\n", result.added.len()));
        for event in &result.added {
            output.push_str(&format!("  seq {}: {} {}\n", event.seq, event.event_type, &event.id[..8]));
        }
    }

    if !result.removed.is_empty() {
        output.push_str(&format!("\n- Removed ({}) events:\n", result.removed.len()));
        for event in &result.removed {
            output.push_str(&format!("  seq {}: {} {}\n", event.seq, event.event_type, &event.id[..8]));
        }
    }

    if !result.modified.is_empty() {
        output.push_str(&format!("\n~ Modified ({}) events:\n", result.modified.len()));
        for (before, after) in &result.modified {
            output.push_str(&format!(
                "  seq {}: {} → {} (commitment changed)\n",
                before.seq, before.event_type, after.event_type
            ));
        }
    }

    output
}
```

---

## Template 3: Receipt Visualization Handler

**File:** `src/graph_builder.rs` (NEW MODULE)

```rust
use std::collections::{BTreeMap, BTreeSet};
use serde::{Serialize, Deserialize};
use crate::types::{Receipt, OperationEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    id: String,
    label: String,
    #[serde(rename = "type")]
    node_type: String, // "event" or "object"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    source: String,
    target: String,
    label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl ReceiptGraph {
    /// Build a graph from a receipt where:
    /// - Nodes: events (as circles) and objects (as boxes)
    /// - Edges: events → events (control flow), events → objects (reference)
    pub fn from_receipt(receipt: &Receipt) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut seen_objects = BTreeSet::new();

        // Add event nodes
        for event in &receipt.events {
            nodes.push(GraphNode {
                id: event.id.clone(),
                label: format!("{} (seq {})", event.event_type, event.seq),
                node_type: "event".to_string(),
            });
        }

        // Add object nodes (deduplicated)
        for event in &receipt.events {
            for obj_ref in &event.objects {
                if !seen_objects.contains(&obj_ref.id) {
                    nodes.push(GraphNode {
                        id: obj_ref.id.clone(),
                        label: format!("{} ({})", obj_ref.id, obj_ref.obj_type),
                        node_type: "object".to_string(),
                    });
                    seen_objects.insert(obj_ref.id.clone());
                }
            }
        }

        // Add edges: event → next event (control flow)
        for i in 0..receipt.events.len().saturating_sub(1) {
            let current = &receipt.events[i];
            let next = &receipt.events[i + 1];

            edges.push(GraphEdge {
                source: current.id.clone(),
                target: next.id.clone(),
                label: "→".to_string(),
            });
        }

        // Add edges: event → object (reference)
        for event in &receipt.events {
            for obj_ref in &event.objects {
                edges.push(GraphEdge {
                    source: event.id.clone(),
                    target: obj_ref.id.clone(),
                    label: format!("uses ({})", obj_ref.qualifier.as_deref().unwrap_or("—")),
                });
            }
        }

        ReceiptGraph { nodes, edges }
    }

    /// Export graph as DOT (graphviz) format
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph Receipt {\n");
        dot.push_str("  rankdir=LR;\n");

        // Style event nodes as circles, object nodes as boxes
        for node in &self.nodes {
            let shape = if node.node_type == "event" { "ellipse" } else { "box" };
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", shape={}];\n",
                node.id, node.label, shape
            ));
        }

        // Add edges
        for edge in &self.edges {
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                edge.source, edge.target, edge.label
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Export graph as JSON
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::chain::ChainAssembler;

    #[test]
    fn graph_3_event_linear() {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();

        let ev1 = build_event("emit", vec![object_ref("order", "Order")], b"p1", &mut counter).unwrap();
        let ev2 = build_event("assemble", vec![object_ref("order", "Order")], b"p2", &mut counter).unwrap();
        let ev3 = build_event("verify", vec![object_ref("order", "Order")], b"p3", &mut counter).unwrap();

        asm.append(ev1).unwrap();
        asm.append(ev2).unwrap();
        asm.append(ev3).unwrap();

        let receipt = asm.finalize();
        let graph = ReceiptGraph::from_receipt(&receipt);

        assert_eq!(graph.nodes.len(), 4); // 3 events + 1 object
        assert_eq!(graph.edges.len(), 5); // 2 control flow + 3 event→object
    }
}
```

**Usage in Handler:**
```rust
// In src/handlers.rs
pub fn visualize(receipt: &Receipt, format: &str) -> anyhow::Result<String> {
    use crate::graph_builder::ReceiptGraph;

    let graph = ReceiptGraph::from_receipt(receipt);

    match format {
        "dot" => Ok(graph.to_dot()),
        "json" => graph.to_json(),
        _ => Err(anyhow::anyhow!("Unknown format: {}", format)),
    }
}
```

---

## Template 4: Ontology Extension (affi-cli.ttl)

**File:** `ontology/affi-cli.ttl` (ADD NEW VERBS)

```turtle
# Receipt noun — existing (MODIFY to add verbs)

@prefix affi: <http://affidavit.io/cli#> .
@prefix cnv: <http://clap-noun-verb.io/ontology#> .

# Receipt noun with new verbs
affi:receipt a cnv:Noun ;
    rdfs:label "receipt"@en ;
    cnv:hasVerbs affi:emit, affi:assemble, affi:verify, affi:show,
                 affi:inspect, affi:diff, affi:visualize, affi:catalog .

# ============= INSPECTION VERBS =============

affi:inspect a cnv:Verb ;
    rdfs:label "inspect a receipt"@en ;
    rdfs:comment "Analyze receipt structure: event types, object coverage, statistics"@en ;
    cnv:belongsToNoun affi:receipt ;
    cnv:hasVerbName "inspect"@en ;
    cnv:hasArguments affi:receipt_path, affi:format_flag .

affi:receipt_path a cnv:Argument ;
    rdfs:label "path"@en ;
    rdfs:comment "Path to receipt JSON file"@en ;
    cnv:hasArgumentName "receipt-path" ;
    cnv:isRequired true ;
    cnv:hasValueType xsd:string .

affi:format_flag a cnv:Argument ;
    rdfs:label "format"@en ;
    rdfs:comment "Output format (plain, json)"@en ;
    cnv:hasArgumentName "format" ;
    cnv:isRequired false ;
    cnv:isFlag true ;
    cnv:hasShortName "f" ;
    cnv:hasValueType xsd:string ;
    cnv:hasDefaultValue "plain" .

affi:diff a cnv:Verb ;
    rdfs:label "compare two receipts"@en ;
    rdfs:comment "Show additions, removals, and modifications between receipts"@en ;
    cnv:belongsToNoun affi:receipt ;
    cnv:hasVerbName "diff"@en ;
    cnv:hasArguments affi:receipt_a, affi:receipt_b .

affi:receipt_a a cnv:Argument ;
    cnv:hasArgumentName "receipt-a" ;
    cnv:isRequired true .

affi:receipt_b a cnv:Argument ;
    cnv:hasArgumentName "receipt-b" ;
    cnv:isRequired true .

affi:visualize a cnv:Verb ;
    rdfs:label "visualize receipt as graph"@en ;
    cnv:belongsToNoun affi:receipt ;
    cnv:hasVerbName "visualize"@en ;
    cnv:hasArguments affi:receipt_path, affi:format_flag .

affi:catalog a cnv:Verb ;
    rdfs:label "list receipt fixtures"@en ;
    cnv:belongsToNoun affi:receipt ;
    cnv:hasVerbName "catalog"@en ;
    cnv:hasArguments affi:filter_key, affi:filter_value .

affi:filter_key a cnv:Argument ;
    cnv:hasArgumentName "filter-key" ;
    cnv:isRequired false .

affi:filter_value a cnv:Argument ;
    cnv:hasArgumentName "filter-value" ;
    cnv:isRequired false .
```

---

## Template 5: Test Template (E2E Inspection)

**File:** `tests/e2e_inspection.rs` (NEW)

```rust
use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::handlers::*;

#[test]
fn test_inspect_simple_3_event() {
    // Create a simple 3-event receipt
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    let ev1 = build_event("emit", vec![object_ref("order-1", "Order")], b"payload1", &mut counter)
        .expect("build event");
    let ev2 = build_event("assemble", vec![object_ref("order-1", "Order")], b"payload2", &mut counter)
        .expect("build event");
    let ev3 = build_event("verify", vec![object_ref("order-1", "Order")], b"payload3", &mut counter)
        .expect("build event");

    asm.append(ev1).unwrap();
    asm.append(ev2).unwrap();
    asm.append(ev3).unwrap();

    let receipt = asm.finalize();

    // Test inspection
    let report = inspect(&receipt);

    // Assertions
    assert_eq!(report.event_count, 3);
    assert_eq!(report.object_count, 1);
    assert_eq!(report.event_types.get("emit"), Some(&1));
    assert_eq!(report.event_types.get("assemble"), Some(&1));
    assert_eq!(report.event_types.get("verify"), Some(&1));
    assert_eq!(report.object_types.get("Order"), Some(&3)); // Same object in 3 events
    assert!(report.verdict_accepted);
    assert!(!report.chain_hash.is_empty());
}

#[test]
fn test_diff_simple_vs_complex() {
    // Receipt A: 2 events
    let mut asm_a = ChainAssembler::new();
    let mut counter_a = SeqCounter::new();
    let ev1 = build_event("emit", vec![object_ref("obj", "Item")], b"data1", &mut counter_a).unwrap();
    let ev2 = build_event("verify", vec![object_ref("obj", "Item")], b"data2", &mut counter_a).unwrap();
    asm_a.append(ev1).unwrap();
    asm_a.append(ev2).unwrap();
    let receipt_a = asm_a.finalize();

    // Receipt B: 3 events (insert assemble in middle)
    let mut asm_b = ChainAssembler::new();
    let mut counter_b = SeqCounter::new();
    let ev1 = build_event("emit", vec![object_ref("obj", "Item")], b"data1", &mut counter_b).unwrap();
    let ev2 = build_event("assemble", vec![object_ref("obj", "Item")], b"data_new", &mut counter_b).unwrap();
    let ev3 = build_event("verify", vec![object_ref("obj", "Item")], b"data2", &mut counter_b).unwrap();
    asm_b.append(ev1).unwrap();
    asm_b.append(ev2).unwrap();
    asm_b.append(ev3).unwrap();
    let receipt_b = asm_b.finalize();

    // Diff A → B should show "assemble" as added
    let diff = diff_receipts(&receipt_a, &receipt_b);
    assert!(diff.added.iter().any(|e| e.event_type == "assemble"));
    assert_eq!(diff.change_count, 1);
}

#[test]
fn test_visualize_produces_valid_json() {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    for i in 0..3 {
        let event = build_event(
            &format!("op{}", i),
            vec![object_ref("obj", "artifact")],
            format!("payload{}", i).as_bytes(),
            &mut counter,
        ).unwrap();
        asm.append(event).unwrap();
    }

    let receipt = asm.finalize();
    let graph_json = visualize(&receipt, "json").expect("visualize");

    // Parse as JSON to verify validity
    let graph: serde_json::Value = serde_json::from_str(&graph_json).expect("parse json");
    assert!(graph.get("nodes").is_some());
    assert!(graph.get("edges").is_some());

    let nodes = graph["nodes"].as_array().expect("nodes array");
    assert!(!nodes.is_empty());
}

#[test]
fn test_catalog_returns_fixtures() {
    let fixtures = catalog("event_count", "3").expect("catalog");
    assert!(!fixtures.is_empty(), "Fixture catalog should have entries");

    // At least one fixture should have 3 events
    assert!(fixtures.iter().any(|f| f.event_count == 3));
}

#[test]
fn test_inspection_workflow_e2e() {
    // Create multiple receipts with different patterns
    let receipts = vec![
        build_linear_receipt(3),    // linear: emit → assemble → verify
        build_linear_receipt(5),    // linear: 5 events
        build_branch_receipt(2, 2), // branching: 2 parallel streams
    ];

    for receipt in receipts {
        // 1. Inspect
        let report = inspect(&receipt);
        assert!(!report.event_types.is_empty());
        assert!(!report.object_types.is_empty());

        // 2. Visualize
        let graph = visualize(&receipt, "json").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&graph).unwrap();
        assert!(parsed.get("nodes").is_some());

        // 3. Verify verdict
        let verdict = affidavit::verifier::verify(&receipt);
        assert_eq!(report.verdict_accepted, verdict.accepted);
    }
}

// Helper: build linear receipt (emit → assemble → ... → verify)
fn build_linear_receipt(count: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    for i in 0..count {
        let event = build_event(
            &["emit", "assemble", "verify"][i % 3],
            vec![object_ref("obj", "Item")],
            format!("data{}", i).as_bytes(),
            &mut counter,
        ).unwrap();
        asm.append(event).unwrap();
    }

    asm.finalize()
}

// Helper: build branching receipt (two parallel object streams)
fn build_branch_receipt(stream1_len: usize, stream2_len: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    for i in 0..stream1_len.max(stream2_len) {
        if i < stream1_len {
            let event = build_event(
                "op",
                vec![object_ref("obj1", "Item")],
                format!("obj1_data{}", i).as_bytes(),
                &mut counter,
            ).unwrap();
            asm.append(event).unwrap();
        }
        if i < stream2_len {
            let event = build_event(
                "op",
                vec![object_ref("obj2", "Item")],
                format!("obj2_data{}", i).as_bytes(),
                &mut counter,
            ).unwrap();
            asm.append(event).unwrap();
        }
    }

    asm.finalize()
}
```

---

## Summary: Phase 1 Code To Implement

| File | Status | Lines | Role |
|------|--------|-------|------|
| `src/handlers.rs` | Modify | +150 | Add inspect(), diff_receipts(), visualize() |
| `src/graph_builder.rs` | New | 150 | Graph model + DOT/JSON export |
| `ontology/affi-cli.ttl` | Modify | +40 | Add inspect, diff, visualize, catalog verbs |
| `tests/e2e_inspection.rs` | New | 200 | E2E tests for 4 features |
| `src/verbs/inspect.rs` | New (generated) | 20 | Auto-generated verb wrapper |
| `src/verbs/diff.rs` | New (generated) | 20 | Auto-generated verb wrapper |
| `src/verbs/visualize.rs` | New (generated) | 20 | Auto-generated verb wrapper |
| `src/verbs/catalog.rs` | New (generated) | 20 | Auto-generated verb wrapper |

**Total Phase 1:** ~620 lines of code, 2.5 hours work

All code above is ready to copy-paste and test. Each handler is independent and can be integrated one-by-one.

