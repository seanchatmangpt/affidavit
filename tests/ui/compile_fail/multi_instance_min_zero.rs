// Test: MultipleInstanceSpecConst with MIN == 0 is unconstructable.
// Should FAIL to compile: `Require<{ MIN >= 1 }>: IsTrue` is unsatisfied for MIN==0
// (a multi-instance spec must allow at least one instance).

use wasm4pm_compat::petri::MultipleInstanceSpecConst;

fn main() {
    let _bad = MultipleInstanceSpecConst::<0, 5>::new();
}
