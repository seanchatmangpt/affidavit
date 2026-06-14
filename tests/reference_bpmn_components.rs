// Reference witness: BPMN leaf component accessors — BpmnTask name, BpmnEdge
// source/target, BpmnNode id (COVERAGE.md §2 — BPMN component accessors;
// complements the process/pool/refusal witnesses).

use wasm4pm_compat::bpmn::{BpmnEdge, BpmnEvent, BpmnGateway, BpmnNode, BpmnTask};

#[test]
fn bpmn_task_carries_its_name() {
    let t = BpmnTask::new("approve_invoice");
    assert_eq!(t.name(), "approve_invoice");
}

#[test]
fn bpmn_edge_carries_source_and_target() {
    let e = BpmnEdge::new("s", "t");
    assert_eq!(e.source(), "s");
    assert_eq!(e.target(), "t");
}

#[test]
fn bpmn_node_id_is_recoverable_for_each_kind() {
    assert_eq!(BpmnNode::task("t1", BpmnTask::new("work")).id(), "t1");
    assert_eq!(BpmnNode::gateway("g1", BpmnGateway::Inclusive).id(), "g1");
    assert_eq!(BpmnNode::event("e1", BpmnEvent::End).id(), "e1");
}
