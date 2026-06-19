// Reference witness: ArtifactGrounding grounded/ungrounded distinction and
// Pm4pyShape object-centric classification (COVERAGE.md §2.5 / interop).
// ArtifactGrounding distinguishes grounded vs ungrounded claims (the basis of
// InteropRefusal::UngroundedArtifact). Pm4pyShape::is_object_centric() classifies
// shapes at the convergence/divergence boundary.

use wasm4pm_compat::interop::{ArtifactGrounding, Pm4pyShape};

#[test]
fn artifact_grounding_distinguishes_grounded_from_ungrounded() {
    let grounded = ArtifactGrounding::<()>::new(Pm4pyShape::EventLog, "ocel:fixture-1");
    assert!(
        grounded.is_grounded(),
        "an evidence-backed claim is grounded"
    );

    let ungrounded = ArtifactGrounding::<()>::new(Pm4pyShape::EventLog, "");
    assert!(
        !ungrounded.is_grounded(),
        "an empty evidence ref is ungrounded (→ UngroundedArtifact)"
    );
}

#[test]
fn pm4py_shape_classifies_object_centric_vs_flat() {
    assert!(
        Pm4pyShape::ObjectCentricLog.is_object_centric(),
        "ObjectCentricLog must be classified as object-centric"
    );
    assert!(
        !Pm4pyShape::EventLog.is_object_centric(),
        "EventLog must not be classified as object-centric"
    );
}
