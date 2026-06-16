// Reference witness: the const-generic `ConditionCell<BITS>` law-kernel type
// (COVERAGE.md §2 — law kernel; the BITS<=8 compile-time bound).
//
// ConditionCell<BITS> carries `Require<{BITS <= 8}>: IsTrue` — a cell wider than
// 8 bits is UNCONSTRUCTABLE (it fails the where-bound). This witnesses the valid
// range (0..=8) constructs, while >8 cannot be written — the bound IS the law.

use wasm4pm_compat::law::ConditionCell;

#[test]
fn condition_cells_within_eight_bits_construct() {
    // Boundary and interior cases all construct (BITS <= 8 holds).
    let _zero = ConditionCell::<0>::new();
    let _three = ConditionCell::<3>::new();
    let _eight = ConditionCell::<8>::new(); // upper boundary
                                            // The fact this file COMPILES is the witness: every ConditionCell here has
                                            // BITS <= 8. The NEGATIVE side (BITS == 9 must NOT compile) is witnessed by the
                                            // trybuild fixture `tests/ui/compile_fail/condition_cell_nine.rs` (+ `.stderr`),
                                            // which pins the `Require<{BITS<=8}>: IsTrue` failure. The 8-bit cap is a
                                            // compile-time law, falsifiable on both sides.
}
