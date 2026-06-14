// Reference witness: the BPMN organizational-perspective container BpmnPool —
// a pool holding a process and its lanes (COVERAGE.md §2 — BPMN pool/lane).
//
// A BpmnPool groups a BpmnProcess with the lanes that partition its nodes (the
// organizational / resource perspective in BPMN). This witnesses pool
// construction with accessors and lane membership over a well-formed process.

use wasm4pm_compat::bpmn::{
    BpmnEdge, BpmnEvent, BpmnLane, BpmnNode, BpmnPool, BpmnProcess, BpmnTask,
};
use std::collections::HashSet;

#[test]
fn pool_holds_process_and_lanes() {
    let process = BpmnProcess::new(
        vec![
            BpmnNode::event("s", BpmnEvent::Start),
            BpmnNode::task("t", BpmnTask::new("approve")),
            BpmnNode::event("e", BpmnEvent::End),
        ],
        vec![BpmnEdge::new("s", "t"), BpmnEdge::new("t", "e")],
    );
    assert_eq!(process.validate(), Ok(()), "the pool's process is well-formed");

    let lane = BpmnLane::new("lane-ops", "Operations", ["t"]);
    let pool = BpmnPool::new("pool-1", "Sales", process, [lane]);

    assert_eq!(pool.id(), "pool-1");
    assert_eq!(pool.name(), "Sales");
    assert_eq!(pool.lanes().len(), 1, "one lane in the pool");
    assert_eq!(pool.process().nodes().len(), 3, "pool wraps the 3-node process");

    // The lane's nodes are declared in the pool's process (organizational
    // partition is consistent).
    let known: HashSet<&str> = pool.process().nodes().iter().map(|n| n.id()).collect();
    assert!(pool.lanes()[0].validate(&known).is_ok(), "lane partitions declared nodes");
}
