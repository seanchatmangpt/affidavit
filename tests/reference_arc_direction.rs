// Reference witness: the Petri Arc direction surface — Arc::direction reports
// PlaceToTransition vs TransitionToPlace, and object_type accessor
// (COVERAGE.md §2 — Petri arc direction/bipartite law).
//
// A Petri net is bipartite: arcs go place→transition or transition→place.
// Arc::direction classifies each; object_type() surfaces an object-centric arc's
// type tag (None for plain arcs).

use wasm4pm_compat::petri::{Arc, ArcDirection};

#[test]
fn arc_direction_classifies_bipartite_edges() {
    let p2t = Arc::place_to_transition("p0", "t0");
    assert_eq!(p2t.direction(), ArcDirection::PlaceToTransition);

    let t2p = Arc::transition_to_place("t0", "p1");
    assert_eq!(t2p.direction(), ArcDirection::TransitionToPlace);

    assert_ne!(p2t.direction(), t2p.direction(), "the two directions are distinct");
}

#[test]
fn plain_arc_has_no_object_type() {
    let arc = Arc::place_to_transition("p", "t");
    assert_eq!(arc.object_type(), None, "a plain arc carries no object type");

    let mut typed = Arc::place_to_transition("p", "t");
    typed.object_type = Some(("order".to_string(), false));
    assert_eq!(typed.object_type(), Some("order"), "an object-typed arc reports its type");
}
