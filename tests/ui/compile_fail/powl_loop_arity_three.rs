// Test: TypedPowlLoopNode with ARITY != 2 is unconstructable (POWL binary-loop law).
// This file should FAIL to compile: `Require<{ ARITY == 2 }>: IsTrue` is unsatisfied
// for ARITY == 3, so the type does not exist for arity 3.

use wasm4pm_compat::powl::{PowlNodeId, TypedPowlLoopNode};

fn main() {
    // ARITY == 3 violates the binary-loop law — no IsTrue impl for Require<{false}>.
    let _bad = TypedPowlLoopNode::<(PowlNodeId, PowlNodeId, PowlNodeId), 3>::new((
        PowlNodeId(0),
        PowlNodeId(1),
        PowlNodeId(2),
    ));
}
