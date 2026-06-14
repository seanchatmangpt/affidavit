// Test: TypedXorNode with ARITY < 2 is unconstructable (operator-node arity law).
// Should FAIL to compile: `Require<{ ARITY >= 2 }>: IsTrue` is unsatisfied for
// ARITY == 1 — a XOR choice over a single branch is meaningless; an exclusive
// choice needs at least two alternatives.

use wasm4pm_compat::process_tree::TypedXorNode;

fn main() {
    // ARITY == 1 violates the >= 2 branch law — no IsTrue impl for Require<{false}>.
    let _bad = TypedXorNode::<(&str,), 1>::new(("only",));
}
