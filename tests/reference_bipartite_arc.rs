// Reference witness: the const-generic bipartite arc BipartiteArcConst<DIR, W> —
// arc direction encoded as a const-generic ArcDirectionConst parameter
// (COVERAGE.md §2 — const-generic typed arc).
//
// Unlike the value-level Arc (whose direction is a runtime field), BipartiteArcConst
// carries its direction in the TYPE: a place→transition arc and a transition→place
// arc are different types. This witnesses both directions construct, carry their
// ids + weight, and report their (type-level) direction.

use wasm4pm_compat::law::ArcDirectionConst;
use wasm4pm_compat::petri::BipartiteArcConst;

#[test]
fn bipartite_arc_encodes_direction_in_the_type() {
    let p2t = BipartiteArcConst::<{ ArcDirectionConst::PlaceToTransition }, u32>::new("p0", "t0", 1);
    assert_eq!(p2t.place_id(), "p0");
    assert_eq!(p2t.transition_id(), "t0");
    assert_eq!(p2t.weight(), 1);
    assert_eq!(p2t.direction(), ArcDirectionConst::PlaceToTransition);

    let t2p = BipartiteArcConst::<{ ArcDirectionConst::TransitionToPlace }, u32>::new("p1", "t0", 2);
    assert_eq!(t2p.direction(), ArcDirectionConst::TransitionToPlace);
    assert_eq!(t2p.weight(), 2);

    // The two arcs are DISTINCT types (direction is a const parameter), so the
    // direction is known at compile time, not just at runtime.
    assert_ne!(p2t.direction(), t2p.direction());
}
