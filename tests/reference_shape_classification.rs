// Reference witness: the Pm4pyShape object-centric classification — exactly one
// of the seven shapes (ObjectCentricLog) is object-centric; the rest are flat
// (COVERAGE.md §2 — shape classification; the basis of FlatClaimOverObjectCentric).
//
// is_object_centric() is the predicate the interop convergence/divergence guard
// rests on. This witnesses the full classification across all seven Pm4pyShapes.

use wasm4pm_compat::interop::Pm4pyShape;

#[test]
fn exactly_object_centric_log_is_object_centric() {
    assert!(Pm4pyShape::ObjectCentricLog.is_object_centric(), "OCEL is object-centric");

    // All other shapes are flat (case-centric / not object-centric).
    for flat in [
        Pm4pyShape::EventLog,
        Pm4pyShape::PetriNet,
        Pm4pyShape::ProcessTree,
        Pm4pyShape::Bpmn,
        Pm4pyShape::DirectlyFollowsGraph,
        Pm4pyShape::Declare,
    ] {
        assert!(!flat.is_object_centric(), "{flat:?} is flat, not object-centric");
    }
}

#[test]
fn exactly_one_of_seven_shapes_is_object_centric() {
    let all = [
        Pm4pyShape::EventLog, Pm4pyShape::ObjectCentricLog, Pm4pyShape::PetriNet,
        Pm4pyShape::ProcessTree, Pm4pyShape::Bpmn, Pm4pyShape::DirectlyFollowsGraph,
        Pm4pyShape::Declare,
    ];
    let oc_count = all.iter().filter(|s| s.is_object_centric()).count();
    assert_eq!(oc_count, 1, "exactly one of the seven shapes is object-centric");
}
