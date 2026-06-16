// Reference witness: positive BPMN construction surface (nodes, edges, kinds,
// lanes) complementing BpmnRefusal (COVERAGE.md §2 — BPMN positive shapes).
//
// Builds a well-formed process start→task→gateway→end with a lane, and exercises:
// node-kind discrimination (Task/Gateway/Event), edge source/target accessors,
// lane membership validation, and process validate() admit.

use std::collections::HashSet;
use wasm4pm_compat::bpmn::{
    BpmnEdge, BpmnEvent, BpmnGateway, BpmnLane, BpmnNode, BpmnNodeKind, BpmnProcess, BpmnTask,
};

#[test]
fn well_formed_process_validates_and_exposes_structure() {
    let nodes = vec![
        BpmnNode::event("s", BpmnEvent::Start),
        BpmnNode::task("t", BpmnTask::new("approve")),
        BpmnNode::gateway("g", BpmnGateway::Exclusive),
        BpmnNode::event("e", BpmnEvent::End),
    ];
    let edges = vec![
        BpmnEdge::new("s", "t"),
        BpmnEdge::new("t", "g"),
        BpmnEdge::new("g", "e"),
    ];
    let p = BpmnProcess::new(nodes, edges);

    assert_eq!(
        p.validate(),
        Ok(()),
        "start→task→gateway→end is well-formed"
    );
    assert_eq!(p.nodes().len(), 4);
    assert_eq!(p.edges().len(), 3);
    assert_eq!(p.edges()[0].source(), "s");
    assert_eq!(p.edges()[0].target(), "t");

    // Node-kind discrimination (control-flow element taxonomy).
    let task_node = p.nodes().iter().find(|n| n.id() == "t").unwrap();
    assert!(
        matches!(task_node.kind(), BpmnNodeKind::Task(_)),
        "t is a Task"
    );
    let gw_node = p.nodes().iter().find(|n| n.id() == "g").unwrap();
    assert!(
        matches!(gw_node.kind(), BpmnNodeKind::Gateway(_)),
        "g is a Gateway"
    );
    let start_node = p.nodes().iter().find(|n| n.id() == "s").unwrap();
    assert!(
        matches!(start_node.kind(), BpmnNodeKind::Event(_)),
        "s is an Event"
    );
}

#[test]
fn lane_over_declared_nodes_validates() {
    let lane = BpmnLane::new("lane-ops", "Operations", ["t", "g"]);
    assert_eq!(lane.id(), "lane-ops");
    assert_eq!(lane.name(), "Operations");
    assert_eq!(lane.node_ids(), &["t".to_string(), "g".to_string()]);

    let known: HashSet<&str> = ["s", "t", "g", "e"].into_iter().collect();
    assert!(
        lane.validate(&known).is_ok(),
        "a lane over declared nodes admits"
    );
}
