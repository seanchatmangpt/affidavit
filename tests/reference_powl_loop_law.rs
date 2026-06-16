// Reference witness: the POWL loop node — both the dynamic PowlNodeKind::Loop and
// the const-generic TypedPowlLoopNode<_, ARITY==2> type law (COVERAGE.md §2 — POWL
// binary-loop law).
//
// POWL loops are binary by definition: a body and an optional redo. wasm4pm-compat
// encodes this two ways: PowlNodeKind::Loop { body, redo } at the value level, and
// TypedPowlLoopNode<Children, const ARITY> with `where Require<{ARITY==2}>: IsTrue`
// at the type level. This witnesses both. The ARITY!=2 rejection is a *compile*
// error (unrepresentable), noted below; this test pins the positive (ARITY==2)
// construction plus the dynamic node shape.

use wasm4pm_compat::powl::{PowlNodeId, PowlNodeKind, TypedPowlLoopNode};

#[test]
fn dynamic_powl_loop_carries_body_and_optional_redo() {
    let with_redo = PowlNodeKind::Loop {
        body: PowlNodeId(0),
        redo: Some(PowlNodeId(1)),
    };
    match with_redo {
        PowlNodeKind::Loop { body, redo } => {
            assert_eq!(body, PowlNodeId(0));
            assert_eq!(redo, Some(PowlNodeId(1)), "redo present");
        }
        other => panic!("expected Loop; got {other:?}"),
    }

    let no_redo = PowlNodeKind::Loop {
        body: PowlNodeId(2),
        redo: None,
    };
    match no_redo {
        PowlNodeKind::Loop { redo: None, .. } => {}
        other => panic!("expected redo-less Loop; got {other:?}"),
    }
}

#[test]
fn typed_loop_node_admits_exactly_arity_two() {
    // ARITY == 2 satisfies `Require<{ARITY==2}>: IsTrue` — this compiles.
    let node =
        TypedPowlLoopNode::<(PowlNodeId, PowlNodeId), 2>::new((PowlNodeId(0), PowlNodeId(1)));
    assert_eq!(node.children.0, PowlNodeId(0));
    assert_eq!(node.children.1, PowlNodeId(1));
    // TypedPowlLoopNode::<_, 3> does NOT compile: Require<{3==2}> has no IsTrue impl.
    // That arity law is enforced at the type level (a compile-fail fixture would
    // witness rejection); this positive case pins the admitted arity.
}
