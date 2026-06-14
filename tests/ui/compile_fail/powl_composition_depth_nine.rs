// Test: PowlComposition with DEPTH > MAX_POWL_DEPTH (8) is unconstructable.
// This file should FAIL to compile: `Require<{ DEPTH <= 8 }>: IsTrue` is unsatisfied
// for DEPTH == 9, so the type does not exist past the nesting ceiling.

use wasm4pm_compat::powl::PowlComposition;

fn main() {
    // DEPTH == 9 exceeds MAX_POWL_DEPTH — no IsTrue impl for Require<{false}>.
    let _too_deep = PowlComposition::<&str, 9>::new("over the ceiling");
}
