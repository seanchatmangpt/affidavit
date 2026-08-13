// Reference witness: positive POWL 2.0 construction — connected ChoiceGraph
// structures admit through validate() (COVERAGE.md §2 — POWL positive shapes).
//
// POWL 2.0 replaces the legacy flat Choice and Loop dynamic AST variants with
// one directed ChoiceGraph operator. Branching is represented by multiple arcs;
// looping is represented by a lawful cycle. Every graph node must lie on a path
// from the declared Start boundary to the End boundary.

use wasm4pm_compat::powl::{ChoiceGraphEdge, Powl, PowlNode, PowlNodeId, PowlNodeKind};

fn node(id: usize, kind: PowlNodeKind) -> PowlNode {
    PowlNode::new(PowlNodeId(id), kind)
}

#[test]
fn connected_choice_graph_with_two_branches_admits() {
    let mut p = Powl::new();
    p.nodes.extend([
        node(0, PowlNodeKind::Start),
        node(1, PowlNodeKind::Atom("a".into())),
        node(2, PowlNodeKind::Atom("b".into())),
        node(3, PowlNodeKind::End),
    ]);
    p.nodes.push(node(
        4,
        PowlNodeKind::ChoiceGraph {
            nodes: vec![PowlNodeId(0), PowlNodeId(1), PowlNodeId(2), PowlNodeId(3)],
            edges: vec![
                ChoiceGraphEdge::new(PowlNodeId(0), PowlNodeId(1)),
                ChoiceGraphEdge::new(PowlNodeId(0), PowlNodeId(2)),
                ChoiceGraphEdge::new(PowlNodeId(1), PowlNodeId(3)),
                ChoiceGraphEdge::new(PowlNodeId(2), PowlNodeId(3)),
            ],
        },
    ));
    p.root = Some(PowlNodeId(4));

    assert_eq!(p.validate(), Ok(()));
    assert_eq!(
        p.node_count(),
        5,
        "four graph nodes plus the graph operator"
    );
}

#[test]
fn connected_choice_graph_cycle_admits_as_powl2_loop_shape() {
    let mut p = Powl::new();
    p.nodes.extend([
        node(0, PowlNodeKind::Start),
        node(1, PowlNodeKind::Atom("do".into())),
        node(2, PowlNodeKind::Atom("redo".into())),
        node(3, PowlNodeKind::End),
    ]);
    p.nodes.push(node(
        4,
        PowlNodeKind::ChoiceGraph {
            nodes: vec![PowlNodeId(0), PowlNodeId(1), PowlNodeId(2), PowlNodeId(3)],
            edges: vec![
                ChoiceGraphEdge::new(PowlNodeId(0), PowlNodeId(1)),
                ChoiceGraphEdge::new(PowlNodeId(1), PowlNodeId(2)),
                ChoiceGraphEdge::new(PowlNodeId(2), PowlNodeId(1)),
                ChoiceGraphEdge::new(PowlNodeId(1), PowlNodeId(3)),
            ],
        },
    ));
    p.root = Some(PowlNodeId(4));

    assert_eq!(
        p.validate(),
        Ok(()),
        "cycles are lawful when every ChoiceGraph node remains on a start-to-end path"
    );
}
