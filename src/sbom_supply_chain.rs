//! Supply-chain graph analytics and provenance over a canonical [`Sbom`].
//!
//! Where `crate::sbom` owns the *format × model* axis (normalizing SPDX,
//! CycloneDX and SWID into one shape), this module owns the *graph × risk ×
//! provenance* axis. Given a canonical SBOM it answers the questions a
//! Fortune-5 supply-chain security team actually asks:
//!
//! - **Dependency graph metrics** — direct/transitive dependencies and
//!   dependents, longest dependency path (depth), and cycle detection.
//! - **Blast radius / impact analysis** — if a component is compromised, the
//!   *reverse* transitive closure is everything that breaks. This is the
//!   single most important number for triaging a disclosed CVE.
//! - **Supplier concentration risk** — how much of the bill of materials a
//!   single supplier owns (a structural single-vendor-dependency signal).
//! - **Single points of failure** — chokepoint components an outsized share of
//!   the graph transitively depends on.
//! - **Provenance attestation** — a SLSA-provenance-*flavored* structural link
//!   from the SBOM's BLAKE3 content address to its primary component, builder
//!   tool and attested supplier.
//!
//! # Doctrine: certify, don't decide
//!
//! Consistent with affidavit's doctrine, everything here is *structural*. We
//! compute graph facts and attestation linkage against the SBOM as given; we
//! never decide whether the declared suppliers, tools or edges are *honest*.
//!
//! # Determinism
//!
//! Every returned `Vec` is in a documented, stable sorted order, and every
//! graph walk is guarded against cycles with a visited set. Internal indices
//! use [`BTreeMap`]/[`BTreeSet`] so iteration order is deterministic and the
//! same logical SBOM always yields byte-identical analytics.

use crate::sbom::Sbom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// An error raised while analyzing a supply-chain dependency graph.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SupplyChainError {
    /// A `bom_ref` was referenced that is not a node in the graph.
    #[error("unknown component: {0}")]
    UnknownComponent(String),
    /// The graph has no nodes (an SBOM with no components).
    #[error("empty dependency graph")]
    EmptyGraph,
}

/// A directed dependency graph derived from an [`Sbom`].
///
/// Nodes are component `bom_ref`s. A forward edge `a → b` means "a directly
/// depends on b". The struct also stores the *reverse* adjacency
/// (`b → a`, "b is depended on by a") so that blast-radius queries — which walk
/// dependents rather than dependencies — are first-class.
///
/// Both adjacency maps are [`BTreeMap`] of [`BTreeSet`], so neighbor lists are
/// de-duplicated and returned in sorted order deterministically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// All node `bom_ref`s in the graph, sorted.
    nodes: BTreeSet<String>,
    /// Forward adjacency: dependent → set of direct dependencies.
    forward: BTreeMap<String, BTreeSet<String>>,
    /// Reverse adjacency: dependency → set of direct dependents.
    reverse: BTreeMap<String, BTreeSet<String>>,
}

impl DependencyGraph {
    /// Build a dependency graph from a canonical SBOM.
    ///
    /// Every component contributes a node (so isolated components are still
    /// present), and every `depends_on` entry contributes a forward edge plus
    /// its mirrored reverse edge. Endpoints named only inside dependency edges
    /// (not catalogued as components) are still admitted as nodes, so the graph
    /// is closed under its own edges.
    pub fn from_sbom(sbom: &Sbom) -> Self {
        let mut nodes: BTreeSet<String> = BTreeSet::new();
        let mut forward: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut reverse: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        for component in &sbom.components {
            nodes.insert(component.bom_ref.clone());
        }

        for dep in &sbom.dependencies {
            nodes.insert(dep.dependent.clone());
            for target in &dep.depends_on {
                nodes.insert(target.clone());
                forward
                    .entry(dep.dependent.clone())
                    .or_default()
                    .insert(target.clone());
                reverse
                    .entry(target.clone())
                    .or_default()
                    .insert(dep.dependent.clone());
            }
        }

        DependencyGraph {
            nodes,
            forward,
            reverse,
        }
    }

    /// Number of nodes (components) in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of directed dependency edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.forward.values().map(BTreeSet::len).sum()
    }

    /// Whether `bom_ref` is a node in the graph.
    pub fn contains(&self, bom_ref: &str) -> bool {
        self.nodes.contains(bom_ref)
    }

    /// All node `bom_ref`s, sorted ascending.
    pub fn nodes(&self) -> Vec<String> {
        self.nodes.iter().cloned().collect()
    }

    /// Direct dependencies of `bom_ref` (the things it points at), sorted.
    ///
    /// Returns an empty vector for a leaf node or an unknown ref.
    pub fn direct_dependencies(&self, bom_ref: &str) -> Vec<String> {
        self.forward
            .get(bom_ref)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Direct dependents of `bom_ref` (the things that point at it), sorted.
    ///
    /// This walks the reverse edges. Returns an empty vector for a root node or
    /// an unknown ref.
    pub fn direct_dependents(&self, bom_ref: &str) -> Vec<String> {
        self.reverse
            .get(bom_ref)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Transitive dependencies: every node reachable from `bom_ref` by walking
    /// forward edges, sorted ascending and excluding `bom_ref` itself.
    ///
    /// Guarded against cycles with a visited set.
    pub fn transitive_dependencies(&self, bom_ref: &str) -> Vec<String> {
        self.closure(bom_ref, &self.forward)
    }

    /// Transitive dependents — the **blast radius**. Every node that can reach
    /// `bom_ref` by walking forward edges (equivalently, the reverse-edge
    /// closure from `bom_ref`), sorted ascending and excluding `bom_ref`.
    ///
    /// This is the set of components that break if `bom_ref` is compromised.
    /// Guarded against cycles with a visited set.
    pub fn transitive_dependents(&self, bom_ref: &str) -> Vec<String> {
        self.closure(bom_ref, &self.reverse)
    }

    /// Shared closure walk over the given adjacency map, returning every node
    /// reachable from `start` (exclusive of `start`) in sorted order.
    fn closure(&self, start: &str, adjacency: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut stack: Vec<String> = adjacency
            .get(start)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default();
        while let Some(node) = stack.pop() {
            if visited.insert(node.clone()) {
                if let Some(children) = adjacency.get(&node) {
                    stack.extend(children.iter().cloned());
                }
            }
        }
        // The start node may reappear via a cycle; never report it as its own
        // dependency/dependent.
        visited.remove(start);
        visited.into_iter().collect()
    }

    /// Longest dependency-path length from `root` following forward edges.
    ///
    /// Depth is measured in *edges*: a leaf has depth 0, a node with a single
    /// child chain `root → a → b` has depth 2. The walk is guarded against
    /// cycles with an on-path visited set, so a back-edge does not inflate or
    /// hang the computation.
    pub fn depth(&self, root: &str) -> usize {
        let mut on_path: BTreeSet<String> = BTreeSet::new();
        self.depth_inner(root, &mut on_path)
    }

    /// Recursive depth helper carrying the current root-to-node path so cycles
    /// are detected and pruned (a revisited on-path node contributes 0).
    fn depth_inner(&self, node: &str, on_path: &mut BTreeSet<String>) -> usize {
        if !on_path.insert(node.to_string()) {
            return 0;
        }
        let deepest = self
            .forward
            .get(node)
            .map(|children| {
                children
                    .iter()
                    .map(|child| 1 + self.depth_inner(child, on_path))
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        on_path.remove(node);
        deepest
    }

    /// Whether the forward dependency graph contains a directed cycle.
    ///
    /// Uses an iterative depth-first search with white/grey/black coloring; a
    /// grey-node back-edge proves a cycle. Total and allocation-bounded by the
    /// node and edge counts.
    pub fn is_cyclic(&self) -> bool {
        // 0 = white (unvisited), 1 = grey (on stack), 2 = black (done).
        let mut color: BTreeMap<String, u8> = BTreeMap::new();
        for node in &self.nodes {
            if color.get(node).copied().unwrap_or(0) != 0 {
                continue;
            }
            // Each stack frame: (node, whether we have expanded its children).
            let mut stack: Vec<(String, bool)> = vec![(node.clone(), false)];
            while let Some((current, expanded)) = stack.pop() {
                if expanded {
                    color.insert(current, 2);
                    continue;
                }
                color.insert(current.clone(), 1);
                stack.push((current.clone(), true));
                if let Some(children) = self.forward.get(&current) {
                    for child in children {
                        match color.get(child).copied().unwrap_or(0) {
                            1 => return true, // back-edge to a grey node → cycle
                            0 => stack.push((child.clone(), false)),
                            _ => {}
                        }
                    }
                }
            }
        }
        false
    }
}

/// The blast radius of a single component: everything transitively impacted if
/// that component is compromised or fails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlastRadius {
    /// The component whose impact was analyzed.
    pub component: String,
    /// Count of *direct* dependents (one edge away).
    pub directly_impacted: usize,
    /// Count of *all* transitive dependents (the full reverse closure).
    pub transitively_impacted: usize,
    /// The transitively impacted `bom_ref`s, sorted ascending.
    pub impacted: Vec<String>,
}

/// Compute the blast radius for `bom_ref` over the dependency graph.
///
/// The impacted set is the reverse transitive closure
/// ([`DependencyGraph::transitive_dependents`]); `directly_impacted` counts the
/// direct dependents only.
///
/// # Errors
///
/// Returns [`SupplyChainError::UnknownComponent`] if `bom_ref` is not a node.
pub fn blast_radius(
    graph: &DependencyGraph,
    bom_ref: &str,
) -> Result<BlastRadius, SupplyChainError> {
    if !graph.contains(bom_ref) {
        return Err(SupplyChainError::UnknownComponent(bom_ref.to_string()));
    }
    let impacted = graph.transitive_dependents(bom_ref);
    Ok(BlastRadius {
        component: bom_ref.to_string(),
        directly_impacted: graph.direct_dependents(bom_ref).len(),
        transitively_impacted: impacted.len(),
        impacted,
    })
}

/// A supplier's share of the bill of materials.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplierConcentration {
    /// Supplier name (or `"UNKNOWN"` for components with no declared supplier).
    pub supplier: String,
    /// Number of components attributed to this supplier.
    pub component_count: usize,
    /// Share of all components owned by this supplier, in `[0.0, 1.0]`.
    pub share: f64,
    /// The component `bom_ref`s attributed to this supplier, sorted ascending.
    pub components: Vec<String>,
}

/// The sentinel supplier name used for components lacking a declared supplier.
pub const UNKNOWN_SUPPLIER: &str = "UNKNOWN";

/// Group components by supplier name and compute each supplier's share.
///
/// Components whose supplier is absent (or whose supplier name is blank) are
/// bucketed under [`UNKNOWN_SUPPLIER`]. The `share` is `component_count /
/// total_components`. The result is sorted by `component_count` **descending**,
/// breaking ties by `supplier` name **ascending**, for a stable ordering.
///
/// Returns an empty vector for an SBOM with no components (no division by zero).
pub fn supplier_concentration(sbom: &Sbom) -> Vec<SupplierConcentration> {
    let total = sbom.components.len();
    if total == 0 {
        return Vec::new();
    }

    let mut buckets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for component in &sbom.components {
        let supplier = component
            .supplier
            .as_ref()
            .map(|s| s.name.trim())
            .filter(|name| !name.is_empty())
            .unwrap_or(UNKNOWN_SUPPLIER)
            .to_string();
        buckets
            .entry(supplier)
            .or_default()
            .insert(component.bom_ref.clone());
    }

    let mut out: Vec<SupplierConcentration> = buckets
        .into_iter()
        .map(|(supplier, refs)| {
            let component_count = refs.len();
            SupplierConcentration {
                supplier,
                component_count,
                share: component_count as f64 / total as f64,
                components: refs.into_iter().collect(),
            }
        })
        .collect();

    // Sort by count descending, then supplier name ascending.
    out.sort_by(|a, b| {
        b.component_count
            .cmp(&a.component_count)
            .then_with(|| a.supplier.cmp(&b.supplier))
    });
    out
}

/// Identify single points of failure: chokepoint components whose transitive
/// dependent count meets or exceeds `threshold`.
///
/// A high transitive-dependent count means a large share of the graph would
/// break if this node failed — a structural chokepoint. The result is sorted by
/// impact (transitive dependent count) **descending**, breaking ties by
/// `bom_ref` **ascending**.
///
/// A `threshold` of 0 returns every node (each trivially has `>= 0` dependents),
/// still in impact-then-name order; callers typically pass a positive value.
pub fn single_points_of_failure(
    graph: &DependencyGraph,
    _sbom: &Sbom,
    threshold: usize,
) -> Vec<String> {
    let mut scored: Vec<(usize, String)> = graph
        .nodes()
        .into_iter()
        .filter_map(|node| {
            let impact = graph.transitive_dependents(&node).len();
            if impact >= threshold {
                Some((impact, node))
            } else {
                None
            }
        })
        .collect();

    // Sort by impact descending, then bom_ref ascending.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, node)| node).collect()
}

/// A structural, SLSA-provenance-flavored attestation linking an SBOM's content
/// address to its primary subject, builder and supplier.
///
/// This is *not* a cryptographically signed in-toto statement; it is the
/// structural skeleton an affidavit receipt can certify — the content address
/// pins the SBOM, while the remaining fields name the subject (primary
/// component), the producer (first tool) and the attested supplier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceAttestation {
    /// Lowercase hex of the SBOM's BLAKE3 content address.
    pub sbom_address: String,
    /// `bom_ref` of the SBOM's primary component, if declared.
    pub primary_component: Option<String>,
    /// Name of the builder — the first tool that produced the SBOM, if any.
    pub builder: Option<String>,
    /// Attested supplier name (document-level supplier), if declared.
    pub attested_supplier: Option<String>,
    /// Number of directed dependency edges the SBOM declares.
    pub dependency_edges: usize,
    /// Reference to the affidavit receipt this attestation was generated from,
    /// if the caller supplied one (the provenance back-link).
    pub generated_from_receipt: Option<String>,
}

/// Derive a provenance attestation from an SBOM, optionally linking it back to
/// an affidavit receipt by reference.
///
/// All fields are read structurally from the SBOM: the content address from
/// [`Sbom::content_address`], the primary component and supplier from the
/// document metadata, and the builder from the first declared tool's name. The
/// dependency-edge count is the sum of every `depends_on` entry.
///
/// `receipt_ref` is carried verbatim into `generated_from_receipt`. Empty /
/// whitespace-only refs are normalized to `None`.
pub fn attest_provenance(sbom: &Sbom, receipt_ref: Option<&str>) -> ProvenanceAttestation {
    let dependency_edges = sbom.dependencies.iter().map(|d| d.depends_on.len()).sum();

    let builder = sbom
        .metadata
        .tools
        .first()
        .map(|tool| tool.name.clone())
        .filter(|name| !name.trim().is_empty());

    let attested_supplier = sbom
        .metadata
        .supplier
        .as_ref()
        .map(|s| s.name.clone())
        .filter(|name| !name.trim().is_empty());

    ProvenanceAttestation {
        sbom_address: sbom.content_address().as_hex().to_string(),
        primary_component: sbom.metadata.primary_component.clone(),
        builder,
        attested_supplier,
        dependency_edges,
        generated_from_receipt: receipt_ref
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .map(|r| r.to_string()),
    }
}

/// A rolled-up supply-chain risk report over an entire SBOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SupplyChainReport {
    /// Number of nodes (components) in the dependency graph.
    pub component_count: usize,
    /// Number of directed dependency edges.
    pub edge_count: usize,
    /// Maximum dependency depth across all nodes (longest path length).
    pub max_depth: usize,
    /// Whether the dependency graph contains a cycle.
    pub is_cyclic: bool,
    /// Number of distinct suppliers (including the `UNKNOWN` bucket if used).
    pub supplier_count: usize,
    /// Share `[0.0, 1.0]` owned by the single largest supplier (0.0 if empty).
    pub top_supplier_share: f64,
    /// Number of single points of failure at the given threshold.
    pub spof_count: usize,
}

/// Build an aggregate supply-chain report for an SBOM.
///
/// Constructs the dependency graph once, computes the maximum dependency depth
/// over all nodes, evaluates cyclicity, summarizes supplier concentration, and
/// counts single points of failure at `spof_threshold`.
pub fn build_report(sbom: &Sbom, spof_threshold: usize) -> SupplyChainReport {
    let graph = DependencyGraph::from_sbom(sbom);

    let max_depth = graph
        .nodes()
        .iter()
        .map(|node| graph.depth(node))
        .max()
        .unwrap_or(0);

    let concentration = supplier_concentration(sbom);
    let top_supplier_share = concentration.first().map(|c| c.share).unwrap_or(0.0);

    SupplyChainReport {
        component_count: graph.node_count(),
        edge_count: graph.edge_count(),
        max_depth,
        is_cyclic: graph.is_cyclic(),
        supplier_count: concentration.len(),
        top_supplier_share,
        spof_count: single_points_of_failure(&graph, sbom, spof_threshold).len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sbom::{Component, Dependency, Sbom, SbomFormat, Supplier, Tool};

    /// Attach a supplier to a freshly-built library component.
    fn supplied(bom_ref: &str, supplier: &str) -> Component {
        let mut c = Component::library(bom_ref, bom_ref, "1.0");
        c.supplier = Some(Supplier {
            name: supplier.to_string(),
            url: None,
            contact: None,
        });
        c
    }

    /// Diamond graph: A → B, A → C, B → D, C → D.
    fn diamond_sbom() -> Sbom {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.components = vec![
            supplied("A", "acme"),
            supplied("B", "acme"),
            supplied("C", "globex"),
            supplied("D", "globex"),
        ];
        sbom.dependencies = vec![
            Dependency {
                dependent: "A".to_string(),
                depends_on: vec!["B".to_string(), "C".to_string()],
            },
            Dependency {
                dependent: "B".to_string(),
                depends_on: vec!["D".to_string()],
            },
            Dependency {
                dependent: "C".to_string(),
                depends_on: vec!["D".to_string()],
            },
        ];
        sbom
    }

    /// Linear chain: A → B → C.
    fn chain_sbom() -> Sbom {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.components = vec![
            Component::library("A", "A", "1.0"),
            Component::library("B", "B", "1.0"),
            Component::library("C", "C", "1.0"),
        ];
        sbom.dependencies = vec![
            Dependency {
                dependent: "A".to_string(),
                depends_on: vec!["B".to_string()],
            },
            Dependency {
                dependent: "B".to_string(),
                depends_on: vec!["C".to_string()],
            },
        ];
        sbom
    }

    /// Cyclic graph: A → B → A.
    fn cyclic_sbom() -> Sbom {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.components = vec![
            Component::library("A", "A", "1.0"),
            Component::library("B", "B", "1.0"),
        ];
        sbom.dependencies = vec![
            Dependency {
                dependent: "A".to_string(),
                depends_on: vec!["B".to_string()],
            },
            Dependency {
                dependent: "B".to_string(),
                depends_on: vec!["A".to_string()],
            },
        ];
        sbom
    }

    #[test]
    fn graph_builds_nodes_and_edges() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        assert_eq!(g.node_count(), 4);
        // A→B, A→C, B→D, C→D
        assert_eq!(g.edge_count(), 4);
        assert!(g.contains("A"));
        assert!(g.contains("D"));
        assert!(!g.contains("Z"));
        assert_eq!(g.nodes(), vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn direct_dependencies_are_sorted() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        assert_eq!(g.direct_dependencies("A"), vec!["B", "C"]);
        assert_eq!(g.direct_dependencies("B"), vec!["D"]);
        // Leaf node has none.
        assert!(g.direct_dependencies("D").is_empty());
        // Unknown ref yields empty, not an error.
        assert!(g.direct_dependencies("Z").is_empty());
    }

    #[test]
    fn direct_dependents_walk_reverse_edges() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        // D is depended on directly by B and C.
        assert_eq!(g.direct_dependents("D"), vec!["B", "C"]);
        // A is a root; nothing depends on it.
        assert!(g.direct_dependents("A").is_empty());
    }

    #[test]
    fn transitive_dependencies_reach_the_leaf() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        assert_eq!(g.transitive_dependencies("A"), vec!["B", "C", "D"]);
        assert_eq!(g.transitive_dependencies("B"), vec!["D"]);
        assert!(g.transitive_dependencies("D").is_empty());
    }

    #[test]
    fn transitive_dependents_is_blast_radius() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        // Everything depends (transitively) on D: A, B, C.
        assert_eq!(g.transitive_dependents("D"), vec!["A", "B", "C"]);
        // Only A depends on B.
        assert_eq!(g.transitive_dependents("B"), vec!["A"]);
        // Nothing depends on the root A.
        assert!(g.transitive_dependents("A").is_empty());
    }

    #[test]
    fn depth_counts_longest_path_in_edges() {
        let chain = DependencyGraph::from_sbom(&chain_sbom());
        // A → B → C is two edges.
        assert_eq!(chain.depth("A"), 2);
        assert_eq!(chain.depth("B"), 1);
        assert_eq!(chain.depth("C"), 0);

        let diamond = DependencyGraph::from_sbom(&diamond_sbom());
        // A → B → D and A → C → D are both length 2.
        assert_eq!(diamond.depth("A"), 2);
        assert_eq!(diamond.depth("D"), 0);
    }

    #[test]
    fn acyclic_graphs_report_no_cycle() {
        assert!(!DependencyGraph::from_sbom(&diamond_sbom()).is_cyclic());
        assert!(!DependencyGraph::from_sbom(&chain_sbom()).is_cyclic());
    }

    #[test]
    fn cycle_detection_finds_a_b_a() {
        let g = DependencyGraph::from_sbom(&cyclic_sbom());
        assert!(g.is_cyclic());
    }

    #[test]
    fn depth_terminates_on_cycle() {
        // The on-path guard must keep depth finite even with A → B → A.
        let g = DependencyGraph::from_sbom(&cyclic_sbom());
        // A → B → (A pruned): two edges are counted before the back-edge to the
        // on-path node A is cut, so depth is finite (2), never divergent.
        assert_eq!(g.depth("A"), 2);
        assert_eq!(g.depth("B"), 2);
    }

    #[test]
    fn transitive_dependents_handles_cycle() {
        let g = DependencyGraph::from_sbom(&cyclic_sbom());
        // In an A↔B cycle each is a (transitive) dependent of the other,
        // but never of itself.
        assert_eq!(g.transitive_dependents("A"), vec!["B"]);
        assert_eq!(g.transitive_dependents("B"), vec!["A"]);
    }

    #[test]
    fn blast_radius_of_leaf_includes_all_ancestors() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        let br = blast_radius(&g, "D").unwrap();
        assert_eq!(br.component, "D");
        assert_eq!(br.directly_impacted, 2); // B and C
        assert_eq!(br.transitively_impacted, 3); // A, B, C
        assert_eq!(br.impacted, vec!["A", "B", "C"]);
    }

    #[test]
    fn blast_radius_rejects_unknown_component() {
        let g = DependencyGraph::from_sbom(&diamond_sbom());
        let err = blast_radius(&g, "Z").unwrap_err();
        assert_eq!(err, SupplyChainError::UnknownComponent("Z".to_string()));
    }

    #[test]
    fn supplier_concentration_groups_and_orders() {
        let conc = supplier_concentration(&diamond_sbom());
        // acme → {A, B}, globex → {C, D}; tie on count broken by name asc.
        assert_eq!(conc.len(), 2);
        assert_eq!(conc[0].supplier, "acme");
        assert_eq!(conc[0].component_count, 2);
        assert_eq!(conc[0].components, vec!["A", "B"]);
        assert_eq!(conc[1].supplier, "globex");
        // Shares sum to 1.0 since every component has a supplier.
        let total: f64 = conc.iter().map(|c| c.share).sum();
        assert!((total - 1.0).abs() < 1e-9);
        assert!((conc[0].share - 0.5).abs() < 1e-9);
    }

    #[test]
    fn supplier_concentration_buckets_unknown() {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.components = vec![
            supplied("A", "acme"),
            Component::library("B", "B", "1.0"), // no supplier
            Component::library("C", "C", "1.0"), // no supplier
        ];
        let conc = supplier_concentration(&sbom);
        // UNKNOWN has 2 → first (count desc), acme has 1.
        assert_eq!(conc[0].supplier, UNKNOWN_SUPPLIER);
        assert_eq!(conc[0].component_count, 2);
        assert_eq!(conc[1].supplier, "acme");
    }

    #[test]
    fn supplier_concentration_empty_sbom_is_empty() {
        let sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        assert!(supplier_concentration(&sbom).is_empty());
    }

    #[test]
    fn spof_detection_respects_threshold_and_order() {
        let sbom = diamond_sbom();
        let g = DependencyGraph::from_sbom(&sbom);
        // Transitive dependents: D=3, B=1, C=1, A=0.
        // Threshold 1 keeps D, B, C (A is dropped), ordered by impact desc then
        // name asc.
        let spof = single_points_of_failure(&g, &sbom, 1);
        assert_eq!(spof, vec!["D", "B", "C"]);
        // Threshold 3 keeps only D.
        assert_eq!(single_points_of_failure(&g, &sbom, 3), vec!["D"]);
        // Threshold above any impact yields nothing.
        assert!(single_points_of_failure(&g, &sbom, 4).is_empty());
    }

    #[test]
    fn attest_provenance_carries_address_builder_supplier() {
        let mut sbom = diamond_sbom();
        sbom.metadata.primary_component = Some("A".to_string());
        sbom.metadata.tools = vec![Tool {
            vendor: Some("acme".to_string()),
            name: "cdxgen".to_string(),
            version: Some("9.0".to_string()),
        }];
        sbom.metadata.supplier = Some(Supplier {
            name: "acme corp".to_string(),
            url: None,
            contact: None,
        });

        let att = attest_provenance(&sbom, Some("receipt-123"));
        assert_eq!(att.sbom_address.len(), 64); // BLAKE3 hex
        assert_eq!(att.sbom_address, sbom.content_address().as_hex());
        assert_eq!(att.primary_component, Some("A".to_string()));
        assert_eq!(att.builder, Some("cdxgen".to_string()));
        assert_eq!(att.attested_supplier, Some("acme corp".to_string()));
        // B→D, C→D, A→{B,C}: 1 + 1 + 2 = 4 edges.
        assert_eq!(att.dependency_edges, 4);
        assert_eq!(att.generated_from_receipt, Some("receipt-123".to_string()));
    }

    #[test]
    fn attest_provenance_normalizes_blank_receipt_and_missing_fields() {
        let sbom = chain_sbom();
        let att = attest_provenance(&sbom, Some("   "));
        assert_eq!(att.generated_from_receipt, None);
        assert_eq!(att.primary_component, None);
        assert_eq!(att.builder, None);
        assert_eq!(att.attested_supplier, None);
        // A→B, B→C: two edges.
        assert_eq!(att.dependency_edges, 2);
    }

    #[test]
    fn attest_provenance_address_is_deterministic() {
        let sbom = diamond_sbom();
        let a = attest_provenance(&sbom, None);
        let b = attest_provenance(&sbom, None);
        assert_eq!(a.sbom_address, b.sbom_address);
    }

    #[test]
    fn report_summarizes_diamond() {
        let sbom = diamond_sbom();
        let report = build_report(&sbom, 1);
        assert_eq!(report.component_count, 4);
        assert_eq!(report.edge_count, 4);
        assert_eq!(report.max_depth, 2);
        assert!(!report.is_cyclic);
        assert_eq!(report.supplier_count, 2);
        assert!((report.top_supplier_share - 0.5).abs() < 1e-9);
        // D, B, C have >= 1 transitive dependent.
        assert_eq!(report.spof_count, 3);
    }

    #[test]
    fn report_flags_cyclic_graph() {
        let report = build_report(&cyclic_sbom(), 1);
        assert!(report.is_cyclic);
        assert_eq!(report.component_count, 2);
        assert_eq!(report.edge_count, 2);
    }
}
