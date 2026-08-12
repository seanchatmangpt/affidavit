// Reference witness: the reachable POWL 2.0 dynamic refusal is
// PowlRefusal::ChoiceGraphDisconnected.
//
// The legacy flat PowlNodeKind::Loop no longer exists in v26.8.7. Dynamic loop
// behavior is a cycle in ChoiceGraph; malformed graph topology is refused when
// a declared graph node is not on a connected Start→End path or an edge names a
// node outside the graph's declared node set.

use wasm4pm_compat::powl::{
    ChoiceGraphEdge, Powl, PowlNode, PowlNodeId, PowlNodeKind, PowlRefusal,
};

fn node(id: usize, kind: PowlNodeKind) -> PowlNode {
    PowlNode::new(PowlNodeId(id), kind)
}

#[test]
fn choice_graph_without_path_to_end_is_refused() {
    let mut p = Powl::new();
    p.nodes.extend([
        node(0, PowlNodeKind::Start),
        node(1, PowlNodeKind::Atom("body".into())),
        node(2, PowlNodeKind::End),
    ]);
    p.nodes.push(node(
        3,
        PowlNodeKind::ChoiceGraph {
            nodes: vec![PowlNodeId(0), PowlNodeId(1), PowlNodeId(2)],
            edges: vec![ChoiceGraphEdge::new(PowlNodeId(0), PowlNodeId(1))],
        },
    ));
    p.root = Some(PowlNodeId(3));

    assert_eq!(p.validate(), Err(PowlRefusal::ChoiceGraphDisconnected));
}

#[test]
fn choice_graph_edge_outside_declared_node_set_is_refused() {
    let mut p = Powl::new();
    p.nodes.extend([
        node(0, PowlNodeKind::Start),
        node(1, PowlNodeKind::End),
    ]);
    p.nodes.push(node(
        2,
        PowlNodeKind::ChoiceGraph {
            nodes: vec![PowlNodeId(0), PowlNodeId(1)],
            edges: vec![ChoiceGraphEdge::new(PowlNodeId(0), PowlNodeId(99))],
        },
    ));
    p.root = Some(PowlNodeId(2));

    assert_eq!(p.validate(), Err(PowlRefusal::ChoiceGraphDisconnected));
}
