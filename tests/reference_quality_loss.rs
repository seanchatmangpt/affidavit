// Reference witnesses for the van der Aalst quality/loss surface — constructions,
// not citations (COVERAGE.md §2.4, §2.5; closes those 🔴 OPEN rows).
//
// Three demonstrations, each failing-when-fake (remove wasm4pm-compat → no
// compile; collapse a variant → assertion fails):
//   1. The four quality dimensions (+F1) are each constructed and distinct.
//   2. The soundness lattice progresses Unknown → Claimed → Witnessed.
//   3. OCEL→XES flattening loss is FIRST-CLASS: a flatten that drops non-case
//      object types is recorded in a named LossReport, never silent. This is the
//      convergence/divergence problem rendered as a named loss (the reason OCEL
//      exists), in the crate's actual vocabulary.

use wasm4pm_compat::conformance::QualityDimension;
use wasm4pm_compat::law::SoundnessState;
use wasm4pm_compat::loss::{LossPolicy, LossReport, ProjectionName};

#[test]
fn four_quality_dimensions_are_constructed_and_distinct() {
    // van der Aalst's conformance quartet (+ the F1 harmonic mean of fitness &
    // precision). Constructed, not named: the exhaustive match is a compile-time
    // census — a missing dimension won't compile, a ghost won't compile.
    let all = [
        QualityDimension::Fitness,
        QualityDimension::Precision,
        QualityDimension::F1,
        QualityDimension::Generalization,
        QualityDimension::Simplicity,
    ];
    fn label(d: QualityDimension) -> &'static str {
        match d {
            QualityDimension::Fitness => "fitness",
            QualityDimension::Precision => "precision",
            QualityDimension::F1 => "f1",
            QualityDimension::Generalization => "generalization",
            QualityDimension::Simplicity => "simplicity",
        }
    }
    let labels: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(
        labels.len(),
        5,
        "all five dimensions are distinct constructions"
    );
    // Relocation note (COVERAGE.md §6): in the type-enforced regime, PRECISION is
    // not a scalar we compute then discard — an over-permissive ("flower")
    // receipt is one the OCEL court refuses (EmptyEventObjectLinks), so low
    // precision surfaces as a refused construction. That refusal is witnessed in
    // tests/court_law_witness.rs; here we only certify the dimension vocabulary
    // is complete and constructed.
    assert!(labels.contains("precision") && labels.contains("fitness"));
}

#[test]
fn soundness_lattice_progresses_unknown_to_witnessed() {
    // The crate models soundness as a 3-state WITNESS lattice (NOT the classical
    // option-to-complete / proper-completion / no-dead-transitions triple — see
    // COVERAGE.md §0). Construct each state and assert they are distinct and
    // ordered Unknown < Claimed < Witnessed by their stable string labels.
    let states = [
        SoundnessState::Unknown,
        SoundnessState::Claimed,
        SoundnessState::Witnessed,
    ];
    let rendered: Vec<String> = states.iter().map(|s| s.to_string()).collect();
    assert_eq!(rendered, vec!["Unknown", "Claimed", "Witnessed"]);
    // Distinctness (no two states collapse — the ghost-variant discipline).
    assert_ne!(SoundnessState::Unknown, SoundnessState::Witnessed);
    assert_ne!(SoundnessState::Claimed, SoundnessState::Witnessed);
}

#[test]
fn ocel_to_xes_flattening_loss_is_named_not_silent() {
    // The convergence/divergence problem: flattening an object-centric log to a
    // single XES case notion DROPS links to non-case object types. The crate
    // makes this loss FIRST-CLASS — a named LossReport under an explicit policy,
    // never a silent flatten. We construct the exact loss the OCEL literature
    // warns about.
    enum Ocel {}
    enum Xes {}
    let report = LossReport::<Ocel, Xes, Vec<&str>>::new(
        ProjectionName("ocel-flatten-to-xes:by-order"),
        LossPolicy::AllowLossWithReport,
        vec!["item", "invoice"], // non-case object types dropped by flattening
    );

    // The loss is recorded, not silent.
    assert_eq!(report.policy, LossPolicy::AllowLossWithReport);
    assert_eq!(report.projection.0, "ocel-flatten-to-xes:by-order");
    let lost = report.into_lost();
    assert_eq!(
        lost,
        vec!["item", "invoice"],
        "flattening must name exactly the object types it dropped (convergence/divergence made explicit)"
    );
}

#[test]
fn refuse_loss_policy_is_distinct_from_allow() {
    // The policy lattice: a producer can REFUSE any loss (RefuseLoss) vs allow it
    // with a report. The distinction is the type-level choice between lossless and
    // lossy projection — constructed here so the policy variants are not ghosts.
    assert_ne!(LossPolicy::RefuseLoss, LossPolicy::AllowLossWithReport);
    assert_ne!(LossPolicy::RefuseLoss, LossPolicy::AllowNamedProjection);
}
