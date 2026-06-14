// Test: ConditionCell with BITS > 8 is unconstructable (the "Need9 means split" law).
// Should FAIL to compile: `Require<{ BITS <= 8 }>: IsTrue` is unsatisfied for BITS==9
// — at most 8 primary condition bits; a 9-bit cell must be split into two cells.

use wasm4pm_compat::law::ConditionCell;

fn main() {
    // BITS == 9 violates the <= 8 cap — no IsTrue impl for Require<{false}>.
    let _too_wide = ConditionCell::<9>::new();
}
