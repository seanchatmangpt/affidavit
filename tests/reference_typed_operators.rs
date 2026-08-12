// Reference witness: the complete arity-typed process-tree operator surface.
//
// wasm4pm-compat v26.8.7 exposes four arity-typed structural nodes:
// XOR, AND (parallel), and SEQ require ARITY >= 2; LOOP requires ARITY == 2.
// Inclusive OR is not part of the canonical closed process-tree operator set and
// therefore must not be manufactured as a local compatibility fiction.

use wasm4pm_compat::process_tree::{TypedAndNode, TypedLoopNode, TypedSeqNode, TypedXorNode};

#[test]
fn typed_process_tree_operators_enforce_their_arity_laws() {
    let xor = TypedXorNode::<(&str, &str), 2>::new(("a", "b"));
    assert_eq!(xor.children.0, "a");

    let and = TypedAndNode::<(u8, u8, u8), 3>::new((1, 2, 3));
    assert_eq!(and.children.2, 3);

    let seq = TypedSeqNode::<(&str, &str), 2>::new(("first", "second"));
    assert_eq!(seq.children.1, "second");

    let loop_node = TypedLoopNode::<(&str, &str), 2>::new(("do", "redo"));
    assert_eq!(loop_node.children, ("do", "redo"));

    // This file compiling is the positive type-law witness. Upstream owns the
    // compile-fail negative witnesses for XOR/AND/SEQ arity < 2 and LOOP arity != 2.
}
