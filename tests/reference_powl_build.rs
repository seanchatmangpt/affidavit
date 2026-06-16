// Reference witness: positive POWL construction — a valid partially-ordered
// workflow language model admits via validate() (COVERAGE.md §2 — POWL positive
// shapes, completing the model-type set; complements PowlRefusal).
//
// Builds a POWL with two atom leaves under a well-formed Choice (arity 2), and a
// Loop whose body references an existing node, then exercises validate() admit
// and node_count. Complements the PowlRefusal::InvalidChoiceArity refusal witness.

use wasm4pm_compat::powl::{Powl, PowlNode, PowlNodeId, PowlNodeKind};

#[test]
fn valid_choice_powl_admits() {
    let mut p = Powl::new();
    p.nodes
        .push(PowlNode::new(PowlNodeId(0), PowlNodeKind::Atom("a".into())));
    p.nodes
        .push(PowlNode::new(PowlNodeId(1), PowlNodeKind::Atom("b".into())));
    // A Choice with two branches satisfies the arity law.
    p.nodes.push(PowlNode::new(
        PowlNodeId(2),
        PowlNodeKind::Choice(vec![PowlNodeId(0), PowlNodeId(1)]),
    ));
    p.root = Some(PowlNodeId(2));

    assert_eq!(
        p.validate(),
        Ok(()),
        "a 2-branch Choice over two atoms admits"
    );
    assert_eq!(p.node_count(), 3, "three POWL nodes");
}

#[test]
fn valid_loop_with_existing_body_admits() {
    let mut p = Powl::new();
    p.nodes.push(PowlNode::new(
        PowlNodeId(0),
        PowlNodeKind::Atom("do".into()),
    ));
    p.nodes.push(PowlNode::new(
        PowlNodeId(1),
        PowlNodeKind::Atom("redo".into()),
    ));
    // A Loop whose body and redo reference existing nodes is valid.
    p.nodes.push(PowlNode::new(
        PowlNodeId(2),
        PowlNodeKind::Loop {
            body: PowlNodeId(0),
            redo: Some(PowlNodeId(1)),
        },
    ));
    p.root = Some(PowlNodeId(2));

    assert_eq!(
        p.validate(),
        Ok(()),
        "a Loop referencing existing body/redo admits"
    );
}
