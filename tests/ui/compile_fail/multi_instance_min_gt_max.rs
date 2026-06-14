// Test: MultipleInstanceSpecConst with MIN > MAX is unconstructable.
// Should FAIL to compile: `Require<{ MIN <= MAX }>: IsTrue` is unsatisfied for
// MIN==5, MAX==2 (lower bound cannot exceed upper bound).

use wasm4pm_compat::petri::MultipleInstanceSpecConst;

fn main() {
    let _bad = MultipleInstanceSpecConst::<5, 2>::new();
}
