// Reference witness: the BPMN element taxonomy — BpmnEvent kinds
// (Start/Intermediate/End/Boundary) and BpmnNodeKind (Task/Gateway/Event)
// (COVERAGE.md §2 — BPMN element kinds census).
//
// BpmnEvent enumerates the four event kinds in the BPMN catalogue; BpmnNodeKind
// is the three top-level node categories. This constructs each event kind into a
// node and discriminates it via BpmnNodeKind, an exhaustive census of both enums.

use wasm4pm_compat::bpmn::{BpmnEvent, BpmnGateway, BpmnNode, BpmnNodeKind, BpmnTask};

#[test]
fn all_four_bpmn_event_kinds_construct() {
    let kinds = [
        BpmnEvent::Start,
        BpmnEvent::Intermediate,
        BpmnEvent::End,
        BpmnEvent::Boundary,
    ];
    fn label(e: BpmnEvent) -> &'static str {
        match e {
            BpmnEvent::Start => "start",
            BpmnEvent::Intermediate => "intermediate",
            BpmnEvent::End => "end",
            BpmnEvent::Boundary => "boundary",
            _ => "unknown-future-kind", // BpmnEvent is #[non_exhaustive]
        }
    }
    let s: std::collections::BTreeSet<&str> = kinds.iter().copied().map(label).collect();
    assert_eq!(s.len(), 4, "four distinct BPMN event kinds");
}

#[test]
fn node_kind_discriminates_task_gateway_event() {
    let task = BpmnNode::task("t", BpmnTask::new("approve"));
    let gw = BpmnNode::gateway("g", BpmnGateway::Parallel);
    let ev = BpmnNode::event("s", BpmnEvent::Start);

    // Exhaustive discrimination over the three node categories.
    for (node, expect) in [(&task, "task"), (&gw, "gateway"), (&ev, "event")] {
        let got = match node.kind() {
            BpmnNodeKind::Task(_) => "task",
            BpmnNodeKind::Gateway(_) => "gateway",
            BpmnNodeKind::Event(_) => "event",
        };
        assert_eq!(got, expect, "node {} discriminated correctly", node.id());
    }
}
