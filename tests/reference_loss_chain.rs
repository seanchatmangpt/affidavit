// Reference witness: the loss-tracking surface beyond the single flatten case —
// LossChain accumulation and the lossless/lossy distinction (COVERAGE.md §2.5).
//
// A LossChain accumulates NamedLoss steps; it is lossless iff empty. A LossReport
// summarises to a NamedLoss carrying its projection + category, and reports
// is_lossless from its dropped-items set. This witnesses that loss is FIRST-CLASS
// and tracked, never silent.

use wasm4pm_compat::loss::{LossChain, LossPolicy, LossReport, ProjectionName};

#[test]
fn loss_chain_tracks_lossless_then_lossy() {
    let mut chain = LossChain::new();
    assert!(chain.is_empty(), "fresh chain is empty");
    assert!(chain.is_lossless(), "an empty loss chain is lossless");
    assert_eq!(chain.len(), 0);

    // Summarise a lossy OCEL→XES flatten into a NamedLoss and record it.
    enum Ocel {}
    enum Xes {}
    let report = LossReport::<Ocel, Xes, Vec<&str>>::new(
        ProjectionName("ocel-flatten-to-xes:by-order"),
        LossPolicy::AllowLossWithReport,
        vec!["item", "invoice"],
    );
    let step = report.summary("DroppedObjectTypeLinks");
    assert_eq!(step.projection().as_str(), "ocel-flatten-to-xes:by-order");
    assert_eq!(step.category(), "DroppedObjectTypeLinks");

    chain.push(step);
    assert!(!chain.is_empty(), "chain now has a recorded loss");
    assert!(!chain.is_lossless(), "a chain with a loss step is NOT lossless");
    assert_eq!(chain.len(), 1);
    assert_eq!(chain.steps().len(), 1, "the loss step is retained, not silent");
}

#[test]
fn loss_report_reports_lossless_vs_lossy() {
    enum A {}
    enum B {}
    let lossless = LossReport::<A, B, Vec<&str>>::new(
        ProjectionName("identity"),
        LossPolicy::RefuseLoss,
        vec![],
    );
    assert!(lossless.is_lossless(), "no dropped items → lossless");

    let lossy = LossReport::<A, B, Vec<&str>>::new(
        ProjectionName("drop"),
        LossPolicy::AllowLossWithReport,
        vec!["x"],
    );
    assert!(!lossy.is_lossless(), "dropped items → lossy");
}
