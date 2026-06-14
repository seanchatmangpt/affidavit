// Reference witness: TypedLoopNode<Children, const ARITY> — the compile-time
// loop-arity law (COVERAGE.md §2 — arity-typed process-tree node).
//
// A process-tree Loop must have exactly 2 children (do + redo). TypedLoopNode
// encodes this as `Require<{ ARITY == 2 }>: IsTrue` — a TypedLoopNode<_, 2> is
// constructable, but TypedLoopNode<_, 3> (or any ARITY != 2) fails the where-bound
// and does NOT compile. The bound IS the loop-arity law.

use wasm4pm_compat::process_tree::TypedLoopNode;

#[test]
fn loop_node_with_arity_two_constructs() {
    // ARITY == 2: the only admissible loop arity, carrying its (do, redo) children.
    let node = TypedLoopNode::<(&str, &str), 2>::new(("do_body", "redo_body"));
    assert_eq!(node.children.0, "do_body");
    assert_eq!(node.children.1, "redo_body");
    // The fact this compiles is the witness: ARITY==2 satisfies the where-bound.
    // A TypedLoopNode::<_, 3> would fail `Require<{ARITY==2}>: IsTrue` and not build
    // — the loop-arity law holds at the type level, before any value exists.
}
