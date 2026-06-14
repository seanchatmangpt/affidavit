// Reference witness: MultipleInstanceSpecConst<const MIN, const MAX> — the
// multi-instance multiplicity law (COVERAGE.md §2 — Petri multi-instance spec).
//
// A multi-instance spec says "between MIN and MAX copies run". Two simultaneous
// const-bound laws make incoherent multiplicities unrepresentable:
//   Require<{ MIN >= 1 }>  — at least one instance
//   Require<{ MIN <= MAX }> — lower bound does not exceed upper bound
// This witnesses the positive side (valid specs construct, min()/max() read back).
// The two negative sides are trybuild fixtures multi_instance_min_zero.rs and
// multi_instance_min_gt_max.rs (one per violated law).

use wasm4pm_compat::petri::MultipleInstanceSpecConst;

#[test]
fn valid_multiplicity_spec_constructs_and_reads_back() {
    let spec = MultipleInstanceSpecConst::<1, 5>::new();
    assert_eq!(spec.min(), 1);
    assert_eq!(spec.max(), 5);

    // Boundary: MIN == MAX (exactly N instances) satisfies both laws.
    let exact = MultipleInstanceSpecConst::<3, 3>::new();
    assert_eq!(exact.min(), 3);
    assert_eq!(exact.max(), 3);
    // MIN=0 (violates MIN>=1) and MIN>MAX (violates MIN<=MAX) do NOT compile —
    // see the two trybuild fixtures.
}
