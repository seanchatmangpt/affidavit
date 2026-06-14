// Reference witness: QualityProfile<FN,FD, PN,PD, F1N,F1D, GN,GD, SN,SD> — the
// composite five-dimension conformance quality law (COVERAGE.md §2 — composite
// per-dimension NUM<=DEN law).
//
// A QualityProfile encodes all five conformance dimensions (fitness, precision,
// f1, generalization, simplicity) as rational constants in the type. Per the
// composite law, each dimension independently obeys `Require<{ N <= D }>` and
// `Require<{ D > 0 }>`, so an out-of-range rational (e.g. NUM > DEN) is
// unconstructable. This witnesses the positive side: a valid profile constructs
// and every dimension's num()/den() reads back the correct const params,
// proving all five dimensions wire to the right (N,D) pair in the declared order.
//
// The negative side (one dimension violating NUM<=DEN) is the trybuild fixture
// tests/ui/compile_fail/quality_profile_bad_precision.rs.

use wasm4pm_compat::conformance::QualityProfile;

#[test]
fn valid_quality_profile_constructs_and_reads_back_every_dimension() {
    // fitness=3/4, precision=7/10, f1=2/3, generalization=1/1, simplicity=0/1.
    let p = QualityProfile::<3, 4, 7, 10, 2, 3, 1, 1, 0, 1>::new();

    // Each dimension wires to its correct (NUM, DEN) const pair in declared order.
    assert_eq!(p.fitness.num(), 3);
    assert_eq!(p.fitness.den(), 4);

    assert_eq!(p.precision.num(), 7);
    assert_eq!(p.precision.den(), 10);

    assert_eq!(p.f1.num(), 2);
    assert_eq!(p.f1.den(), 3);

    assert_eq!(p.generalization.num(), 1);
    assert_eq!(p.generalization.den(), 1);

    assert_eq!(p.simplicity.num(), 0);
    assert_eq!(p.simplicity.den(), 1);

    // default() routes through new() with the same const params.
    let d = QualityProfile::<3, 4, 7, 10, 2, 3, 1, 1, 0, 1>::default();
    assert_eq!(d.fitness.num(), 3);
    assert_eq!(d.simplicity.den(), 1);

    // A profile with any dimension NUM > DEN does NOT compile —
    // see trybuild fixture quality_profile_bad_precision.rs.
}
