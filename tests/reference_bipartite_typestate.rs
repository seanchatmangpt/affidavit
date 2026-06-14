// Reference witness: the bipartite-arc typestate — PlaceToTransitionArc /
// TransitionToPlaceArc with PhantomData node markers, both `IsValidArc`
// (COVERAGE.md §2 — Petri bipartite-arc type law).
//
// Petri nets are bipartite: an arc connects a place to a transition or a transition
// to a place, never like-to-like. wasm4pm-compat encodes the two lawful directions
// as distinct types carrying their endpoint markers (PlaceNodeMarker /
// TransitionNodeMarker) in PhantomData, and both implement IsValidArc. This
// witnesses construction + weight read-back for both directions, and that each
// genuinely satisfies the IsValidArc bound (binding through a generic fn that
// requires it — failing-when-fake: a type without the impl would not compile here).

use wasm4pm_compat::petri::{
    IsValidArc, PlaceNodeMarker, PlaceToTransitionArc, TransitionNodeMarker, TransitionToPlaceArc,
};

// Only compiles for types that actually implement IsValidArc.
fn requires_valid_arc<A: IsValidArc>(_arc: &A) {}

#[test]
fn both_arc_directions_carry_weight_and_are_valid_arcs() {
    let p2t = PlaceToTransitionArc::<PlaceNodeMarker, TransitionNodeMarker, u32>::new(2);
    assert_eq!(p2t.weight(), 2, "place→transition arc weight");
    requires_valid_arc(&p2t); // proves IsValidArc holds

    let t2p = TransitionToPlaceArc::<TransitionNodeMarker, PlaceNodeMarker, u32>::new(5);
    assert_eq!(t2p.weight(), 5, "transition→place arc weight");
    requires_valid_arc(&t2p); // proves IsValidArc holds
}
