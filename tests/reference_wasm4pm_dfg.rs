// Reference witness: the wasm4pm DFG node/edge structures — DFG, DFGNode,
// DirectlyFollowsRelation (COVERAGE.md §2 — wasm4pm discovered-DFG model).
//
// These are the wasm4pm (engine) DFG types affidavit's `graph`/`model` verbs
// build. This witnesses constructing a DFG with nodes (activity + frequency) and
// directly-follows relations (from/to/frequency), and the start/end activity maps.

use wasm4pm::models::{DFGNode, DirectlyFollowsRelation, DFG};

#[test]
fn dfg_node_and_relation_carry_frequencies() {
    let node = DFGNode { id: "a".to_string(), label: "create".to_string(), frequency: 5 };
    assert_eq!(node.label, "create");
    assert_eq!(node.frequency, 5, "activity occurrence count");

    let rel = DirectlyFollowsRelation { from: "a".to_string(), to: "b".to_string(), frequency: 3 };
    assert_eq!(rel.from, "a");
    assert_eq!(rel.to, "b");
    assert_eq!(rel.frequency, 3, "directly-follows count");
}

#[test]
fn dfg_assembles_nodes_edges_and_endpoints() {
    let mut dfg = DFG::new();
    dfg.nodes.push(DFGNode { id: "a".to_string(), label: "create".to_string(), frequency: 2 });
    dfg.nodes.push(DFGNode { id: "b".to_string(), label: "release".to_string(), frequency: 2 });
    dfg.edges.push(DirectlyFollowsRelation { from: "a".to_string(), to: "b".to_string(), frequency: 2 });
    dfg.start_activities.insert("a".to_string(), 2);
    dfg.end_activities.insert("b".to_string(), 2);

    assert_eq!(dfg.nodes.len(), 2);
    assert_eq!(dfg.edges.len(), 1);
    assert_eq!(dfg.start_activities.get("a"), Some(&2), "start activity frequency");
    assert_eq!(dfg.end_activities.get("b"), Some(&2), "end activity frequency");
}
