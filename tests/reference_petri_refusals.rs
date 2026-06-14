// Reference witness: two more reachable PetriRefusal variants — UnsafeNet and
// InvalidInstanceBounds (COVERAGE.md §2.6 — extends PetriRefusal beyond
// MissingFinalMarking).
//
// - InitialFinalMarkingPair::validate → UnsafeNet when a place holds a token in
//   BOTH the initial and final marking (a token that never leaves = unsafe).
// - MultipleInstanceSpec::validate → InvalidInstanceBounds when min == 0 or
//   min > max (degenerate multi-instance bounds).

use wasm4pm_compat::petri::{
    InitialFinalMarkingPair, InstanceCreationKind, Marking, MultipleInstanceSpec, PetriRefusal,
};

#[test]
fn token_in_both_initial_and_final_is_unsafe() {
    // Place "p" marked in initial AND final → UnsafeNet.
    let pair = InitialFinalMarkingPair::new(
        Marking::new([("p".to_string(), 1)]),
        Marking::new([("p".to_string(), 1)]),
    );
    assert_eq!(pair.validate(), Err(PetriRefusal::UnsafeNet));

    // Disjoint initial/final places → safe.
    let safe = InitialFinalMarkingPair::new(
        Marking::new([("i".to_string(), 1)]),
        Marking::new([("o".to_string(), 1)]),
    );
    assert_eq!(safe.validate(), Ok(()), "disjoint initial/final markings are safe");
}

#[test]
fn degenerate_instance_bounds_are_invalid() {
    // min == 0 → invalid.
    let zero = MultipleInstanceSpec::new(0, Some(3), None, InstanceCreationKind::Static);
    assert_eq!(zero.validate(), Err(PetriRefusal::InvalidInstanceBounds));

    // min > max → invalid.
    let inverted = MultipleInstanceSpec::new(5, Some(2), None, InstanceCreationKind::Dynamic);
    assert_eq!(inverted.validate(), Err(PetriRefusal::InvalidInstanceBounds));

    // 1..=3 → valid.
    let ok = MultipleInstanceSpec::new(1, Some(3), None, InstanceCreationKind::Static);
    assert_eq!(ok.validate(), Ok(()), "1..=3 is a valid instance bound");
}
