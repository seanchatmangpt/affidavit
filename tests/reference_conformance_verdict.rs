// Reference witness: the ConformanceVerdict + Deviation structure — the
// replay-diagnostic shape van der Aalst's conformance produces (COVERAGE.md §2 —
// conformance verdict + local deviation diagnostics).
//
// A Deviation is a LOCAL diagnostic: at trace `position`, activity `label`
// deviated. A ConformanceVerdict carries the four-dimension metrics plus the
// deviation list, and is_perfect() iff fitness==1.0 AND there are no deviations.
// This is the token-game's output rendered as typed structure.

use wasm4pm_compat::conformance::{ConformanceVerdict, Deviation, Fitness};

#[test]
fn perfect_verdict_requires_full_fitness_and_no_deviations() {
    // Perfect: fitness 1.0, zero deviations.
    let mut perfect = ConformanceVerdict::new();
    perfect.fitness = Fitness::new(1.0);
    assert!(perfect.deviations.is_empty());
    assert!(perfect.is_perfect(), "fitness 1.0 + no deviations ⇒ perfect alignment");
}

#[test]
fn a_deviation_breaks_perfection_and_localises_the_problem() {
    let mut v = ConformanceVerdict::new();
    v.fitness = Fitness::new(1.0);
    // A local replay diagnostic: at position 2, activity "ship" deviated.
    v.deviations.push(Deviation::new(2, "ship"));

    assert!(!v.is_perfect(), "a deviation breaks perfection even at fitness 1.0");
    assert_eq!(v.deviations.len(), 1, "the deviation is retained as a diagnostic");
}

#[test]
fn imperfect_fitness_is_not_perfect_even_without_deviations() {
    let mut v = ConformanceVerdict::new();
    v.fitness = Fitness::new(0.83);
    assert!(!v.is_perfect(), "fitness < 1.0 ⇒ not perfect");
    // A default (no-fitness) verdict is also not perfect.
    assert!(!ConformanceVerdict::new().is_perfect(), "absent fitness ⇒ not perfect");
}
