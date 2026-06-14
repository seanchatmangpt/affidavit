// Reference witness: XesRefusal::LiftingLoss is REACHABLE (van der Aalst panel M3
// correction — it was wrongly listed as a ghost). COVERAGE.md §7 / §2.
//
// The §7 ghost census grepped xes.rs (the DECLARING file) and missed the real
// producer in a different module: interop.rs's `XesToOcedProjection::project`
// returns Err(XesRefusal::LiftingLoss) under LossPolicy::RefuseLoss — the lifting
// law at the XES→OCED boundary (refuse rather than silently drop object-perspective
// evidence). This witnesses the refusal fires, and that a permissive policy instead
// yields a LossReport. Failing-when-fake: if the projection stopped refusing under
// RefuseLoss, the first assert fails.

use wasm4pm_compat::loss::{LossPolicy, Project};
use wasm4pm_compat::interop::XesToOcedProjection;
use wasm4pm_compat::xes::XesRefusal;

#[test]
fn lifting_under_refuse_loss_refuses_with_lifting_loss() {
    let proj = XesToOcedProjection::new("order");
    let result = proj.project(LossPolicy::RefuseLoss);
    assert_eq!(
        result.err(),
        Some(XesRefusal::LiftingLoss),
        "lifting XES→OCED under RefuseLoss must refuse rather than drop evidence"
    );
}

#[test]
fn lifting_under_a_permissive_policy_yields_a_report_not_a_refusal() {
    let proj = XesToOcedProjection::new("order");
    let report = proj.project(LossPolicy::AllowLossWithReport);
    assert!(report.is_ok(), "a permissive policy produces a LossReport, not a refusal");
}
