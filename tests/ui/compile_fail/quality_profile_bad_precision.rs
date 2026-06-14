// Test: QualityProfile with an out-of-range precision dimension is unconstructable.
// Should FAIL to compile: `Require<{ PN <= PD }>: IsTrue` is unsatisfied for
// precision = 11/10 (NUM > DEN), violating the composite per-dimension NUM<=DEN law.
// All other dimensions are valid; the single bad precision pair alone breaks it.

use wasm4pm_compat::conformance::QualityProfile;

fn main() {
    // precision PN=11, PD=10 → 11/10 > 1 violates Require<{ PN <= PD }>.
    let _bad = QualityProfile::<3, 4, 11, 10, 2, 3, 1, 1, 0, 1>::new();
}
