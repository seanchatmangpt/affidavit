// Reference witness: the const-generic bounded-fraction law `Between01<NUM,DEN>`
// (COVERAGE.md §2 — law kernel; the compile-time [0,1] metric bound).
//
// Between01<NUM, DEN> carries a where-bound `Require<{DEN > 0}>: IsTrue` AND
// `Require<{NUM <= DEN}>: IsTrue`. So a fraction outside [0,1] or with a zero
// denominator is UNCONSTRUCTABLE — it does not compile. The law is enforced at
// the type level, before any value exists.
//
// This witnesses the positive side: valid fractions (0/1, 3/4, 1/1) construct and
// report their num/den. The negative side (e.g. Between01<4,3> = 1.33, or
// Between01<1,0>) is a COMPILE error by the where-bound — it cannot be written in
// this file at all, which is the law itself (an out-of-range metric has no
// representation). A trybuild compile-fail fixture is intentionally NOT added
// here because its expected stderr is rustc-version-fragile; the where-bound in
// wasm4pm-compat/src/law.rs is the authoritative, self-documenting witness.

use wasm4pm_compat::law::Between01;

#[test]
fn valid_fractions_construct_and_report_num_den() {
    let zero = Between01::<0, 1>::new(); // 0.0
    assert_eq!(zero.num(), 0);
    assert_eq!(zero.den(), 1);

    let three_quarters = Between01::<3, 4>::new(); // 0.75
    assert_eq!(three_quarters.num(), 3);
    assert_eq!(three_quarters.den(), 4);

    let one = Between01::<1, 1>::new(); // 1.0 (boundary)
    assert_eq!(one.num(), 1);
    assert_eq!(one.den(), 1);

    // The fact that this file COMPILES is the witness: every Between01 written
    // here satisfies NUM <= DEN and DEN > 0. An out-of-range fraction would fail
    // the `Require<...>: IsTrue` where-bound and this file would not build.
}
