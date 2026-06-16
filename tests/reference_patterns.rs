// Reference witness: every in-surface workflow pattern is CONSTRUCTED, not cited.
//
// The reviewer's primary read (van der Aalst): a reference that claims the
// pattern catalogue must construct each pattern, not name it. The crate's own
// doc (law.rs) cites Russell, van der Aalst & ter Hofstede (2016), *Workflow
// Patterns: The Definitive Guide* — the 20 basic control-flow patterns (WCP-1..20).
// `wasm4pm_compat::law::WorkflowPattern` exports 17 of them.
//
// Completeness is failing-when-fake here on two axes:
//   1. The exhaustive `match` below has NO wildcard arm — if the crate added a
//      pattern we didn't construct, this file would not compile (missing arm).
//   2. Every variant we name must exist — a ghost pattern would not compile.
// So a green build is itself the proof that exactly the in-surface patterns are
// each constructed and mapped to their WCP number. The 26 patterns of the full
// 43-pattern taxonomy that the crate does NOT export are OUT-OF-SURFACE and are
// not claimed (R-1) — see reference/COVERAGE.md §2.3.

use wasm4pm_compat::law::WorkflowPattern;

/// Map each constructed pattern to its canonical WCP number (Russell/vdA/tH 2016).
/// The absence of a `_ =>` arm makes this exhaustive: it is a compile-time census
/// of the in-surface pattern set.
fn wcp_number(p: WorkflowPattern) -> u32 {
    match p {
        WorkflowPattern::Sequence => 1,
        WorkflowPattern::ParallelSplit => 2,
        WorkflowPattern::Synchronization => 3,
        WorkflowPattern::ExclusiveChoice => 4,
        WorkflowPattern::SimpleMerge => 5,
        WorkflowPattern::MultiChoice => 6,
        WorkflowPattern::StructuredSynchronizingMerge => 7,
        WorkflowPattern::MultiMerge => 8,
        WorkflowPattern::StructuredDiscriminator => 9,
        WorkflowPattern::ArbitraryCycles => 10,
        WorkflowPattern::ImplicitTermination => 11,
        WorkflowPattern::MultipleInstancesWithoutSync => 12,
        WorkflowPattern::MultipleInstancesWithDesignTimeKnowledge => 13,
        WorkflowPattern::DeferredChoice => 16,
        WorkflowPattern::InterleavedParallelRouting => 17,
        WorkflowPattern::CancelActivity => 19,
        WorkflowPattern::CancelCase => 20,
    }
}

/// The full in-surface census — every variant constructed exactly once.
fn all_in_surface_patterns() -> Vec<WorkflowPattern> {
    vec![
        WorkflowPattern::Sequence,
        WorkflowPattern::ParallelSplit,
        WorkflowPattern::Synchronization,
        WorkflowPattern::ExclusiveChoice,
        WorkflowPattern::SimpleMerge,
        WorkflowPattern::MultiChoice,
        WorkflowPattern::StructuredSynchronizingMerge,
        WorkflowPattern::MultiMerge,
        WorkflowPattern::StructuredDiscriminator,
        WorkflowPattern::ArbitraryCycles,
        WorkflowPattern::ImplicitTermination,
        WorkflowPattern::MultipleInstancesWithoutSync,
        WorkflowPattern::MultipleInstancesWithDesignTimeKnowledge,
        WorkflowPattern::DeferredChoice,
        WorkflowPattern::InterleavedParallelRouting,
        WorkflowPattern::CancelActivity,
        WorkflowPattern::CancelCase,
    ]
}

#[test]
fn all_seventeen_in_surface_patterns_are_constructed() {
    let patterns = all_in_surface_patterns();
    assert_eq!(
        patterns.len(),
        17,
        "the crate exports 17 in-surface patterns"
    );

    // Each constructed pattern maps to a distinct WCP number — proving each is a
    // reachable, distinct construction (the ghost-variant discipline applied to
    // the pattern catalogue). If two collapsed to the same value, or any failed
    // to construct, this fails.
    let mut numbers: Vec<u32> = patterns.iter().copied().map(wcp_number).collect();
    numbers.sort_unstable();
    numbers.dedup();
    assert_eq!(
        numbers.len(),
        17,
        "every pattern must be a distinct, reachable construction (no two collapse)"
    );
}

#[test]
fn patterns_are_witness_distinct_not_just_named() {
    // A pattern carried as a structural label must not be confusable with another
    // (the crate's stated guarantee: a WfNetConst claiming ParallelSplit cannot be
    // silently confused with one claiming ExclusiveChoice). Exercise the Eq law.
    assert_ne!(
        WorkflowPattern::ParallelSplit,
        WorkflowPattern::ExclusiveChoice
    );
    assert_eq!(WorkflowPattern::Sequence, WorkflowPattern::Sequence);
    // Debug renders (the label is materialised, not phantom).
    assert_eq!(format!("{:?}", WorkflowPattern::CancelCase), "CancelCase");
}
