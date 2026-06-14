// Test: FitnessConst with NUM > DEN is unconstructable.
// Should FAIL to compile: `Require<{ NUM <= DEN }>: IsTrue` is unsatisfied for
// NUM > DEN (a quality metric must be a rational in [0,1]).

use wasm4pm_compat::conformance::FitnessConst;

fn main() {
    let _bad = FitnessConst::<4, 3>::new();
}
