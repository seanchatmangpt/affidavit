// Reference witness: two more PowlRefusal variants fired against real malformed
// POWL — ChoiceGraphDisconnected and CyclicPartialOrder (COVERAGE.md §2.6 —
// extends PowlRefusal beyond InvalidChoiceArity).
//
// - A POWL 2.0 ChoiceGraph with fewer than 2 nodes is disconnected.
// - A PartialOrder with a back-edge (a→b and b→a) is cyclic — partial orders
//   must be acyclic.

use wasm4pm_compat::powl::{OrderEdge, Powl, PowlNode, PowlNodeId, PowlNodeKind, PowlRefusal};

#[test]
fn choice_graph_with_one_node_is_disconnected() {
    let mut p = Powl::new();
    p.nodes.push(PowlNode::new(
        PowlNodeId(0),
        PowlNodeKind::ChoiceGraph {
            nodes: vec![PowlNodeId(0)],
            edges: vec![],
        }, // < 2 nodes
    ));
    p.root = Some(PowlNodeId(0));
    assert_eq!(p.validate(), Err(PowlRefusal::ChoiceGraphDisconnected));
}

#[test]
fn cyclic_partial_order_is_refused() {
    let mut p = Powl::new();
    p.nodes
        .push(PowlNode::new(PowlNodeId(0), PowlNodeKind::Atom("a".into())));
    p.nodes
        .push(PowlNode::new(PowlNodeId(1), PowlNodeKind::Atom("b".into())));
    p.nodes.push(PowlNode::new(
        PowlNodeId(2),
        PowlNodeKind::PartialOrder(vec![PowlNodeId(0), PowlNodeId(1)]),
    ));
    // A cycle: a before b AND b before a.
    p.edges.push(OrderEdge::new(PowlNodeId(0), PowlNodeId(1)));
    p.edges.push(OrderEdge::new(PowlNodeId(1), PowlNodeId(0)));
    p.root = Some(PowlNodeId(2));
    assert_eq!(p.validate(), Err(PowlRefusal::CyclicPartialOrder));
}
