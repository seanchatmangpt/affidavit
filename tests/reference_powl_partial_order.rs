// Reference witness: the POWL partial-order surface — a PartialOrder node with
// OrderEdge precedence edges, validated acyclic (COVERAGE.md §2 — POWL partial
// order; complements the Choice/Loop POWL witnesses).
//
// POWL's distinguishing feature is partial orders: a PartialOrder node over child
// ids, with OrderEdges (from → to) giving the precedence relation. validate()
// checks the partial order is acyclic. This witnesses a valid 2-element partial
// order (a before b) admitting, and the OrderEdge from/to accessors.

use wasm4pm_compat::powl::{OrderEdge, Powl, PowlNode, PowlNodeId, PowlNodeKind};

#[test]
fn order_edge_carries_from_and_to() {
    let e = OrderEdge::new(PowlNodeId(0), PowlNodeId(1));
    assert_eq!(e.from.0, 0, "predecessor");
    assert_eq!(e.to.0, 1, "successor");
}

#[test]
fn valid_partial_order_admits() {
    let mut p = Powl::new();
    p.nodes
        .push(PowlNode::new(PowlNodeId(0), PowlNodeKind::Atom("a".into())));
    p.nodes
        .push(PowlNode::new(PowlNodeId(1), PowlNodeKind::Atom("b".into())));
    p.nodes.push(PowlNode::new(
        PowlNodeId(2),
        PowlNodeKind::PartialOrder(vec![PowlNodeId(0), PowlNodeId(1)]),
    ));
    // Precedence: a before b (acyclic).
    p.edges.push(OrderEdge::new(PowlNodeId(0), PowlNodeId(1)));
    p.root = Some(PowlNodeId(2));

    assert_eq!(
        p.validate(),
        Ok(()),
        "an acyclic partial order (a before b) admits"
    );
    assert_eq!(p.node_count(), 3);
    assert_eq!(p.edges.len(), 1, "one precedence edge");
}
