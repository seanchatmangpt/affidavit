// Reference witness: the named OCEL→XES flattening PROJECTION and the artifact
// grounding positive surface (COVERAGE.md §2.5 / interop).
//
// OcelToXesProjection is the concrete convergence/divergence projection: it
// flattens an object-centric log to a single XES case notion, parameterised by
// the chosen case_type (the choice that *causes* convergence/divergence). It
// carries a stable PROJECTION_NAME. ArtifactGrounding distinguishes grounded vs
// ungrounded claims (the basis of the InteropRefusal::UngroundedArtifact law).

use wasm4pm_compat::interop::{ArtifactGrounding, OcelToXesProjection, Pm4pyShape};

#[test]
fn named_flatten_projection_carries_its_case_notion() {
    let proj = OcelToXesProjection::new("order");
    assert_eq!(
        proj.case_type(),
        "order",
        "the projection records the chosen case type"
    );
    assert_eq!(
        OcelToXesProjection::PROJECTION_NAME.as_str(),
        "ocel-flatten-to-xes:by-case-type",
        "the flatten projection has a stable name"
    );
    // A different case notion is a different flattening choice (different convergence
    // /divergence behaviour) — same projection name, different parameter.
    let proj2 = OcelToXesProjection::new("item");
    assert_ne!(proj.case_type(), proj2.case_type());
}

#[test]
fn artifact_grounding_distinguishes_grounded_from_ungrounded() {
    let grounded = ArtifactGrounding::<()>::new(Pm4pyShape::EventLog, "xes:fixture-1");
    assert!(
        grounded.is_grounded(),
        "an evidence-backed claim is grounded"
    );

    let ungrounded = ArtifactGrounding::<()>::new(Pm4pyShape::EventLog, "");
    assert!(
        !ungrounded.is_grounded(),
        "an empty evidence ref is ungrounded (→ UngroundedArtifact)"
    );

    // The object-centric vs flat classification (basis of FlatClaimOverObjectCentric).
    assert!(Pm4pyShape::ObjectCentricLog.is_object_centric());
    assert!(!Pm4pyShape::EventLog.is_object_centric());
}
