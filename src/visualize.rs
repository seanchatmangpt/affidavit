//! Logic for building a graph representation of a receipt and exporting it
//! to Graphviz DOT or JSON formats.
//!
//! Per DOD_PHASE1_INSPECTION §3.2, the graph is a Directly-Follows Graph (DFG)
//! where nodes represent distinct event types and edges represent transitions.

use crate::types::Receipt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A node in the receipt graph, representing a distinct event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique identifier for the node (the event type).
    pub id: String,
    /// Human-readable label for the node.
    pub label: String,
    /// Number of times this event type appears in the receipt.
    pub event_count: usize,
}

/// A directed edge in the receipt graph, representing a transition between event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// ID of the source node.
    pub from: String,
    /// ID of the target node.
    pub to: String,
    /// Number of times this transition occurs in the receipt.
    pub weight: usize,
}

/// A graph representation of a receipt's process flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptGraph {
    /// The nodes in the graph.
    pub nodes: Vec<GraphNode>,
    /// The edges in the graph.
    pub edges: Vec<GraphEdge>,
}

/// Build a graph from a receipt where nodes are event types and edges are transitions.
///
/// This implements a Directly-Follows Graph (DFG) summary as required by
/// ARDPRD and the Phase 1 inspection criteria.
pub fn build_graph(receipt: &Receipt) -> ReceiptGraph {
    let mut nodes: BTreeMap<String, GraphNode> = BTreeMap::new();
    let mut edges: BTreeMap<(String, String), GraphEdge> = BTreeMap::new();

    // 1. Build nodes from distinct event types
    for event in &receipt.events {
        let node = nodes.entry(event.event_type.clone()).or_insert_with(|| GraphNode {
            id: event.event_type.clone(),
            label: event.event_type.clone(),
            event_count: 0,
        });
        node.event_count += 1;
    }

    // 2. Build edges from consecutive event pairs
    for i in 0..receipt.events.len().saturating_sub(1) {
        let from_type = receipt.events[i].event_type.clone();
        let to_type = receipt.events[i + 1].event_type.clone();

        let edge = edges.entry((from_type.clone(), to_type.clone())).or_insert_with(|| GraphEdge {
            from: from_type,
            to: to_type,
            weight: 0,
        });
        edge.weight += 1;
    }

    ReceiptGraph {
        nodes: nodes.into_values().collect(),
        edges: edges.into_values().collect(),
    }
}

/// Export the graph as DOT (Graphviz) format.
///
/// The output follows the specific formatting rules defined in DOD_PHASE1_INSPECTION.md.
pub fn to_dot(graph: &ReceiptGraph) -> String {
    let mut dot = String::from("digraph receipt {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [fontname=\"Courier\"];\n");

    for node in &graph.nodes {
        let escaped_id = node.id.replace('\"', "\\\"");
        let escaped_label = node.label.replace('\"', "\\\"");
        dot.push_str(&format!(
            "  \"{}\" [label=\"{} ({})\"];\n",
            escaped_id, escaped_label, node.event_count
        ));
    }

    for edge in &graph.edges {
        let escaped_from = edge.from.replace('\"', "\\\"");
        let escaped_to = edge.to.replace('\"', "\\\"");
        dot.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
            escaped_from, escaped_to, edge.weight
        ));
    }

    dot.push_str("}\n");
    dot
}

/// Export the graph as JSON format.
pub fn to_json(graph: &ReceiptGraph) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(graph)?)
}
