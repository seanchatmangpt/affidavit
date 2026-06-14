// Reference witness: PowlRefusal::InvalidLoop — a POWL Loop whose body references
// a node not in the arena is refused (COVERAGE.md §2.6 — completes the REACHABLE
// PowlRefusal set: 4 of 4 reachable variants now witnessed).
//
// A Loop { body, redo } must reference existing node ids. A body pointing at a
// non-existent id is an InvalidLoop. (The remaining 4 PowlRefusal variants —
// InvalidChoice, LoopMissingDoBody, IrreducibleProjection, LanguageMismatch —
// have NO Err producer in powl.rs and are ghost variants, §7.)

use wasm4pm_compat::powl::{Powl, PowlNode, PowlNodeId, PowlNodeKind, PowlRefusal};

#[test]
fn loop_with_missing_body_is_invalid() {
    let mut p = Powl::new();
    p.nodes.push(PowlNode::new(PowlNodeId(0), PowlNodeKind::Atom("a".into())));
    // Loop body references id 99, which is not in the arena.
    p.nodes.push(PowlNode::new(
        PowlNodeId(1),
        PowlNodeKind::Loop { body: PowlNodeId(99), redo: None },
    ));
    p.root = Some(PowlNodeId(1));
    assert_eq!(p.validate(), Err(PowlRefusal::InvalidLoop));
}

#[test]
fn loop_with_missing_redo_is_invalid() {
    let mut p = Powl::new();
    p.nodes.push(PowlNode::new(PowlNodeId(0), PowlNodeKind::Atom("a".into())));
    // Valid body (0), but redo references a missing id.
    p.nodes.push(PowlNode::new(
        PowlNodeId(1),
        PowlNodeKind::Loop { body: PowlNodeId(0), redo: Some(PowlNodeId(88)) },
    ));
    p.root = Some(PowlNodeId(1));
    assert_eq!(p.validate(), Err(PowlRefusal::InvalidLoop));
}
