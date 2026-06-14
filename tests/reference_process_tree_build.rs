// Reference witness: positive ProcessTree construction — a nested well-formed
// tree admits, with node_count/root accessors (COVERAGE.md §2 — ProcessTree
// positive shapes, complementing ProcessTreeRefusal).
//
// Builds Sequence( a, Xor(b, c) ) — a nested operator tree — and exercises:
// admit_shape() admit, node_count, root presence, and operator-arity validity.

use wasm4pm_compat::process_tree::{
    ProcessTree, ProcessTreeNode, ProcessTreeNodeId, ProcessTreeOperator,
};

#[test]
fn nested_well_formed_tree_admits_and_exposes_structure() {
    // Arena: 0=a, 1=b, 2=c, 3=Xor(b,c), 4=Sequence(a, Xor)
    let mut t = ProcessTree::new();
    t.nodes.push(ProcessTreeNode::Activity("a".into())); // 0
    t.nodes.push(ProcessTreeNode::Activity("b".into())); // 1
    t.nodes.push(ProcessTreeNode::Activity("c".into())); // 2
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Xor,
        children: vec![ProcessTreeNodeId(1), ProcessTreeNodeId(2)],
    }); // 3
    t.nodes.push(ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Sequence,
        children: vec![ProcessTreeNodeId(0), ProcessTreeNodeId(3)],
    }); // 4
    t.root = Some(ProcessTreeNodeId(4));

    assert_eq!(t.admit_shape(), Ok(()), "Sequence(a, Xor(b,c)) is structurally valid");
    assert_eq!(t.node_count(), 5, "five nodes in the arena");
    assert_eq!(t.root, Some(ProcessTreeNodeId(4)), "root is the outer Sequence");

    // The root is the Sequence operator with two children.
    match &t.nodes[4] {
        ProcessTreeNode::Operator { operator, children } => {
            assert!(matches!(operator, ProcessTreeOperator::Sequence));
            assert_eq!(children.len(), 2, "Sequence has arity 2");
        }
        _ => panic!("root should be an operator"),
    }
}

#[test]
fn empty_tree_is_vacuously_admissible() {
    // An empty tree (no nodes, no root) has nothing to violate.
    let t = ProcessTree::new();
    assert_eq!(t.node_count(), 0);
    assert_eq!(t.admit_shape(), Ok(()), "the empty tree admits (no node violates a law)");
}
