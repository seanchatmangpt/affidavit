// Reference witness: object-lifecycle, causal-consistency, and temporal-order
// vocabularies are constructed as exhaustive censuses
// (COVERAGE.md §2 — lifecycle/causality/temporal vocabulary).
//
// These are the object-centric and causal surfaces of the process-mining model:
//   • ObjectLifecyclePhase — the lawful phases an object passes through.
//   • CausalConsistency    — the cross-object causality verdict lattice.
//   • TemporalOrder        — pairwise temporal relations (Lamport-style).
//
// (XesLifecycleTransition coverage lives in reference_xes_lifecycle_vocab.rs,
//  which exercises the as_str↔parse round-trip more strongly.)
//
// Each no-wildcard match is a compile-time census: missing/ghost variant → no compile.

use wasm4pm_compat::object_lifecycle::ObjectLifecyclePhase as Phase;
use wasm4pm_compat::causality::CausalConsistency as CC;
use wasm4pm_compat::temporal::TemporalOrder as TO;

#[test]
fn object_lifecycle_phases_are_constructed() {
    let all = [Phase::Created, Phase::Active, Phase::Modified, Phase::Archived, Phase::Deleted];
    // No-wildcard match: each phase maps to a lifecycle stage family. A 23rd
    // variant would break compilation here.
    fn family(p: Phase) -> &'static str {
        match p {
            Phase::Created => "pre-active",
            Phase::Active | Phase::Modified => "live",
            Phase::Archived | Phase::Deleted => "terminal",
        }
    }
    assert!(all.iter().copied().map(family).all(|f| !f.is_empty()));
    // Distinctness via Display: five distinct rendered phase names.
    let names: std::collections::BTreeSet<String> =
        all.iter().map(|p| format!("{p}")).collect();
    assert_eq!(names.len(), 5, "five distinct lifecycle phases (Display)");
}

#[test]
fn causal_consistency_verdicts_are_constructed() {
    let all = [CC::Consistent, CC::HasCycles, CC::HasContradictions, CC::Unknown];
    fn label(c: CC) -> &'static str {
        match c {
            CC::Consistent => "consistent",
            CC::HasCycles => "has-cycles",
            CC::HasContradictions => "has-contradictions",
            CC::Unknown => "unknown",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(s.len(), 4, "four causal-consistency verdicts");
}

#[test]
fn temporal_orders_are_constructed() {
    let all = [TO::Before, TO::After, TO::Concurrent, TO::Unknown];
    fn label(t: TO) -> &'static str {
        match t {
            TO::Before => "before",
            TO::After => "after",
            TO::Concurrent => "concurrent",
            TO::Unknown => "unknown",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(s.len(), 4, "four temporal relations");
}
