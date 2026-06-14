// Reference witness: positive Petri-net construction + accessor/marking surface
// (COVERAGE.md §2 — Petri positive shapes, complementing PetriRefusal).
//
// Builds a net with places, a visible transition, a SILENT (tau) transition, and
// arcs, then exercises: places/transitions/arcs accessors, the runtime initial
// marking's tokens_on query, and the silent/visible transition distinction.
//
// Failing-when-fake: stubbed accessors/markings fail the assertions; removing the
// dependency fails to compile.

use wasm4pm_compat::petri::{Arc, Marking, PetriNet, Place, Transition};

#[test]
fn petri_net_accessors_reflect_construction() {
    let net = PetriNet::new(
        [Place::new("p0"), Place::new("p1")],
        [Transition::new("t0", "fire"), Transition::silent("tau")],
        [
            Arc::place_to_transition("p0", "t0"),
            Arc::transition_to_place("t0", "p1"),
        ],
        Marking::new([("p0".to_string(), 1)]),
    );

    assert_eq!(net.places().len(), 2, "two places");
    assert_eq!(net.transitions().len(), 2, "two transitions");
    assert_eq!(net.arcs().len(), 2, "two arcs");

    // Runtime initial marking reads token counts; unmarked places are 0.
    let m = net.initial_marking();
    assert_eq!(m.tokens_on("p0"), 1, "p0 marked with one token");
    assert_eq!(m.tokens_on("p1"), 0, "p1 unmarked → 0");
}

#[test]
fn silent_and_visible_transitions_are_distinguished() {
    let fire = Transition::new("t0", "fire");
    let tau = Transition::silent("tau");
    assert_eq!(fire.id(), "t0");
    assert_eq!(fire.label(), "fire");
    assert!(!fire.is_silent(), "a labelled transition is visible");
    assert!(tau.is_silent(), "a tau transition is silent");
}

#[test]
fn marking_token_queries() {
    let m = Marking::new([("p0".to_string(), 2), ("p1".to_string(), 0)]);
    assert!(!m.is_empty(), "a non-empty marking");
    assert_eq!(m.tokens_on("p0"), 2);
    assert_eq!(m.tokens_on("absent"), 0, "absent place → 0 tokens");
    assert!(Marking::empty().is_empty(), "the empty marking is empty");
}
