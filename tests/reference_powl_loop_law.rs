// Reference witness: POWL 2.0 replaces the dynamic flat PowlNodeKind::Loop
// constructor with cyclic ChoiceGraph topology, while retaining the independent
// const-generic TypedPowlLoopNode<_, ARITY==2> arity law.
//
// The dynamic witness below is intentionally a directed cycle in a ChoiceGraph.
// Reintroducing PowlNodeKind::Loop here would contradict the published POWL 2.0
// ontology, which structurally rejects the prior POWL 1.0 flat Loop operator.

use wasm4pm_compat::powl::{PowlBuilder, PowlNodeId, TypedPowlLoopNode};

#[test]
fn dynamic_powl_loop_is_a_choice_graph_cycle() {
    // START → body → redo → body forms the cycle; body can also exit to END.
    // Successful construction witnesses the POWL 2.0 representation of cyclic
    // behavior without manufacturing a removed flat Loop variant.
    let powl = PowlBuilder::new()
        .atom("START")
        .atom("END")
        .atom("body")
        .atom("redo")
        .choice_graph(
            "loop_graph",
            &["START", "body", "redo", "END"],
            &[
                ("START", "body"),
                ("body", "redo"),
                ("redo", "body"),
                ("body", "END"),
            ],
        )
        .root("loop_graph")
        .build()
        .expect("a POWL 2.0 ChoiceGraph may contain a lawful cyclic back-edge");

    assert_eq!(powl.node_count(), 5, "four atoms plus the choice graph");
}

#[test]
fn typed_loop_node_admits_exactly_arity_two() {
    // The independent type-law surface remains exported: ARITY == 2 satisfies
    // `Require<{ARITY==2}>: IsTrue` and therefore compiles.
    let node =
        TypedPowlLoopNode::<(PowlNodeId, PowlNodeId), 2>::new((PowlNodeId(0), PowlNodeId(1)));
    assert_eq!(node.children.0, PowlNodeId(0));
    assert_eq!(node.children.1, PowlNodeId(1));
    // TypedPowlLoopNode::<_, 3> does NOT compile: Require<{3==2}> has no IsTrue impl.
    // The upstream compile-fail fixture owns the negative witness; this positive
    // case pins the exact admitted arity without confusing it with dynamic AST shape.
}
