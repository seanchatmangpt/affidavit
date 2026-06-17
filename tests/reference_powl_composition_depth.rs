#![cfg(feature = "core")]
// Reference witness: PowlComposition<Inner, const DEPTH <= MAX_POWL_DEPTH> — the
// composition-depth ceiling type law (COVERAGE.md §2 — POWL nesting ceiling).
//
// POWL compositions are bounded to MAX_POWL_DEPTH (8) nesting levels, encoded as
// `where Require<{ DEPTH <= MAX_POWL_DEPTH }>: IsTrue`. This witnesses the positive
// side: depth 0 and the boundary depth 8 both construct. The negative side (depth 9
// unrepresentable) is the trybuild fixture powl_composition_depth_nine.rs.

use wasm4pm_compat::powl::{PowlComposition, MAX_POWL_DEPTH};

#[test]
fn composition_admits_up_to_the_depth_ceiling() {
    assert_eq!(MAX_POWL_DEPTH, 8, "the documented nesting ceiling");

    let shallow = PowlComposition::<&str, 0>::new("leaf");
    assert_eq!(shallow.inner, "leaf");

    // Boundary: DEPTH == MAX_POWL_DEPTH (8) satisfies `DEPTH <= 8` — admitted.
    let at_ceiling = PowlComposition::<&str, 8>::new("deep");
    assert_eq!(at_ceiling.inner, "deep");
    // PowlComposition::<_, 9> does NOT compile — see the trybuild fixture.
}
