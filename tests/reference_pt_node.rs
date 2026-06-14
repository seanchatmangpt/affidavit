// Reference witness: the ProcessTreeNode taxonomy — Activity leaves vs Operator
// inner nodes — and ProcessTreeNodeId (COVERAGE.md §2 — process-tree node types).
//
// A process tree's arena holds two node kinds: Activity(label) leaves and
// Operator{operator, children} inner nodes referencing child ids. This witnesses
// constructing each, discriminating by variant, and reading the carried data.

use wasm4pm_compat::process_tree::{ProcessTreeNode, ProcessTreeNodeId, ProcessTreeOperator};

#[test]
fn activity_leaf_carries_its_label() {
    let leaf = ProcessTreeNode::Activity("place_order".into());
    match &leaf {
        ProcessTreeNode::Activity(label) => assert_eq!(label, "place_order"),
        _ => panic!("expected an Activity leaf"),
    }
}

#[test]
fn operator_node_carries_operator_and_children() {
    let op = ProcessTreeNode::Operator {
        operator: ProcessTreeOperator::Xor,
        children: vec![ProcessTreeNodeId(0), ProcessTreeNodeId(1)],
    };
    match &op {
        ProcessTreeNode::Operator { operator, children } => {
            assert!(matches!(operator, ProcessTreeOperator::Xor));
            assert_eq!(children.len(), 2);
            assert_eq!(children[0].0, 0, "child id is a usize index");
            assert_eq!(children[1].0, 1);
        }
        _ => panic!("expected an Operator node"),
    }
}
