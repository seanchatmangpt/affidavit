// Reference witness: the multiperspective EVIDENCE CARRIER tags a value with
// process-mining perspective markers at the type level (COVERAGE.md §2 —
// multiperspective carrier; complements the ProcessPerspective enum census).
//
// MultiPerspectiveEvidence<T, Perspectives> carries an inner value indexed by a
// zero-sized perspective tag (ControlFlow/Data/Resource/Time, or a combination).
// This witnesses that a value can be tagged with a single perspective and with a
// PerspectiveCombination, and that the inner value is recoverable — the perspective
// is a type-level annotation, zero runtime cost.

use wasm4pm_compat::multiperspective::{
    ControlFlowPerspective, DataPerspective, MultiPerspectiveEvidence, PerspectiveCombination,
    ResourcePerspective, TimePerspective,
};

#[test]
fn a_value_can_be_tagged_with_a_single_perspective() {
    let cf: MultiPerspectiveEvidence<&str, ControlFlowPerspective> =
        MultiPerspectiveEvidence::new("activity-trace");
    assert_eq!(
        cf.inner, "activity-trace",
        "inner value recoverable under a control-flow tag"
    );

    let time: MultiPerspectiveEvidence<u64, TimePerspective> = MultiPerspectiveEvidence::new(1_000);
    assert_eq!(
        time.inner, 1_000,
        "inner value recoverable under a time tag"
    );
}

#[test]
fn perspectives_compose_at_the_type_level() {
    // A value tagged with the combination of two perspectives (e.g. resource×data
    // — "who recorded what attribute"). The combination is a distinct phantom type;
    // the inner value is unchanged (zero runtime cost).
    type ResourceData = PerspectiveCombination<ResourcePerspective, DataPerspective>;
    let combined: MultiPerspectiveEvidence<i32, ResourceData> = MultiPerspectiveEvidence::new(42);
    assert_eq!(
        combined.inner, 42,
        "combined-perspective carrier holds its value"
    );

    // The four single perspectives are distinct carrier types — this only compiles
    // because each marker is a real, distinct type (zero-sized).
    let _cf: MultiPerspectiveEvidence<(), ControlFlowPerspective> =
        MultiPerspectiveEvidence::new(());
    let _d: MultiPerspectiveEvidence<(), DataPerspective> = MultiPerspectiveEvidence::new(());
    let _r: MultiPerspectiveEvidence<(), ResourcePerspective> = MultiPerspectiveEvidence::new(());
    let _t: MultiPerspectiveEvidence<(), TimePerspective> = MultiPerspectiveEvidence::new(());
}
