// Reference witness: the arity-typed XOR/AND/SEQ/OR operator nodes — each
// enforces ARITY >= 2 at compile time (COVERAGE.md §2 — typed operator arity law;
// complements TypedLoopNode's ARITY == 2).
//
// XOR, AND (parallel), SEQ (sequence), and OR operators all require at least two
// children. TypedXorNode<_, ARITY> etc. encode this as `Require<{ARITY >= 2}>:
// IsTrue` — arity 2/3/… construct, but ARITY < 2 fails the where-bound and is
// unconstructable. The bound IS the minimum-arity law.

use wasm4pm_compat::process_tree::{TypedAndNode, TypedOrNode, TypedSeqNode, TypedXorNode};

#[test]
fn typed_operators_admit_arity_two_and_above() {
    let xor = TypedXorNode::<(&str, &str), 2>::new(("a", "b"));
    assert_eq!(xor.children.0, "a");

    let and = TypedAndNode::<(u8, u8, u8), 3>::new((1, 2, 3));
    assert_eq!(and.children.2, 3);

    let seq = TypedSeqNode::<(&str, &str), 2>::new(("first", "second"));
    assert_eq!(seq.children.1, "second");

    let or = TypedOrNode::<(bool, bool), 2>::new((true, false));
    assert!(or.children.0);

    // This file COMPILING is the witness: every operator here has ARITY >= 2.
    // A TypedXorNode::<_, 1> (or 0) would fail `Require<{ARITY >= 2}>: IsTrue` and
    // not build — XOR/AND/SEQ/OR cannot be unary, enforced at the type level.
}
