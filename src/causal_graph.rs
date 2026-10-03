//! Causal DAG certification (petgraph): cycle refusal + topological order.
//!
//! Cross-repo work-order graphs must be strict DAGs before execution can be
//! scheduled. This module certifies acyclicity with Tarjan's SCC algorithm
//! (naming the offending component) and emits a topological execution order
//! for admissible graphs.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Acyclicity is a decidable structural property. The graph certifies
//! declared dependency order; it never decides whether the dependencies are
//! *sensible*.

use petgraph::algo::{tarjan_scc, toposort};
use petgraph::graphmap::DiGraphMap;
use std::fmt::Display;
use std::hash::Hash;
use thiserror::Error;

/// Errors produced by causal graph certification.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CausalError {
    /// The graph contains a cycle. Names one node of the offending
    /// strongly-connected component.
    #[error("causal cycle detected involving: {0}")]
    CycleDetected(String),
    /// The graph references an unknown node.
    #[error("unknown node: {0}")]
    UnknownNode(String),
}

/// A causal dependency graph over copyable, hashable node ids.
#[derive(Debug, Clone)]
pub struct CausalGraph<K>
where
    K: Copy + Eq + Ord + Hash + Display,
{
    graph: DiGraphMap<K, ()>,
}

impl<K> Default for CausalGraph<K>
where
    K: Copy + Eq + Ord + Hash + Display,
{
    fn default() -> Self {
        Self {
            graph: DiGraphMap::new(),
        }
    }
}

impl<K> CausalGraph<K>
where
    K: Copy + Eq + Ord + Hash + Display,
{
    /// An empty causal graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a causal dependency: `from` must precede `to`.
    pub fn add_dependency(&mut self, from: K, to: K) {
        self.graph.add_edge(from, to, ());
    }

    /// Whether the node is present in the graph.
    #[must_use]
    pub fn contains(&self, node: K) -> bool {
        self.graph.contains_node(node)
    }

    /// Number of registered dependencies.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Certify the graph is a strict DAG.
    ///
    /// # Errors
    ///
    /// Returns [`CausalError::CycleDetected`] naming one node of the first
    /// strongly-connected component of size > 1 (or a self-loop).
    pub fn verify_acyclic(&self) -> Result<(), CausalError> {
        for component in tarjan_scc(&self.graph) {
            let is_cycle = component.len() > 1
                || self
                    .graph
                    .neighbors_directed(component[0], petgraph::Direction::Outgoing)
                    .any(|neighbor| neighbor == component[0]);
            if is_cycle {
                return Err(CausalError::CycleDetected(component[0].to_string()));
            }
        }
        Ok(())
    }

    /// Certify acyclicity and emit a topological execution order.
    ///
    /// # Errors
    ///
    /// Returns [`CausalError::CycleDetected`] on any cycle.
    pub fn topological_order(&self) -> Result<Vec<K>, CausalError> {
        self.verify_acyclic()?;
        toposort(&self.graph, None)
            .map_err(|cycle| CausalError::CycleDetected(cycle.node_id().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dag_admits_and_orders() {
        let mut g = CausalGraph::new();
        // resolve -> construct -> actuate -> receipt
        g.add_dependency("resolve", "construct");
        g.add_dependency("construct", "actuate");
        g.add_dependency("actuate", "receipt");
        g.add_dependency("resolve", "actuate");
        assert!(g.verify_acyclic().is_ok());
        let order = g.topological_order().expect("order");
        let names: Vec<String> = order.iter().map(ToString::to_string).collect();
        let pos = |n: &str| names.iter().position(|x| x == n).expect("node");
        assert!(pos("resolve") < pos("construct"));
        assert!(pos("construct") < pos("actuate"));
        assert!(pos("actuate") < pos("receipt"));
    }

    #[test]
    fn two_node_cycle_is_refused() {
        let mut g = CausalGraph::new();
        g.add_dependency(1u32, 2);
        g.add_dependency(2, 1);
        assert!(matches!(
            g.verify_acyclic(),
            Err(CausalError::CycleDetected(_))
        ));
    }

    #[test]
    fn self_loop_is_refused() {
        let mut g = CausalGraph::new();
        g.add_dependency("self", "self");
        assert!(matches!(
            g.verify_acyclic(),
            Err(CausalError::CycleDetected(_))
        ));
    }

    #[test]
    fn empty_graph_trivially_admits() {
        let g: CausalGraph<u8> = CausalGraph::new();
        assert!(g.verify_acyclic().is_ok());
        assert_eq!(g.topological_order().expect("order"), Vec::<u8>::new());
    }
}
