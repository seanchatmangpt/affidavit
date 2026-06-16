// Reference witness: the POWL-2.0 ChoiceGraph + StandaloneChoiceGraphNode — the
// directed choice-graph that replaces flat XOR/loop in POWL 2.0
// (COVERAGE.md §2 — choice graph).
//
// A ChoiceGraph is a directed graph over decision nodes with a Start (▷) and End
// (□). This witnesses construction, the node taxonomy (Start/End/Activity/SubModel),
// and the successors/predecessors graph queries.

use wasm4pm_compat::powl::{ChoiceGraph, StandaloneChoiceGraphNode as Node};

#[test]
fn choice_graph_node_taxonomy_and_graph_queries() {
    // ▷ → a → □ : start, an activity, end.
    let nodes = vec![
        Node::Start,
        Node::Activity("approve".to_string()),
        Node::End,
    ];
    let edges = vec![(0, 1), (1, 2)]; // start→activity→end
    let cg = ChoiceGraph::new(nodes, edges);

    // Graph queries reflect the constructed edges.
    assert_eq!(
        cg.successors(0),
        vec![1],
        "start's successor is the activity"
    );
    assert_eq!(cg.successors(1), vec![2], "activity's successor is end");
    assert_eq!(
        cg.predecessors(2),
        vec![1],
        "end's predecessor is the activity"
    );
    assert!(cg.successors(2).is_empty(), "end has no successors");
}

#[test]
fn choice_graph_node_variants_construct() {
    // The four node kinds: Start, End, Activity(label), SubModel(id).
    let _start = Node::Start;
    let _end = Node::End;
    let act = Node::Activity("ship".to_string());
    let sub = Node::SubModel(7);
    assert!(matches!(act, Node::Activity(ref s) if s == "ship"));
    assert!(matches!(sub, Node::SubModel(7)));
}
