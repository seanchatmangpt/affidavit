// Reference witness: const-generic quality-metric aliases (COVERAGE.md §2 —
// conformance quality dimensions as compile-time rationals).
//
// Each alias is a QualityMetricKind-tagged specialization of the underlying
// generic `Metric<KIND, NUM, DEN>`. Two simultaneous const-bound laws make an
// out-of-range metric unrepresentable:
//   Require<{ DEN > 0 }>   — denominator is nonzero (a real rational)
//   Require<{ NUM <= DEN }> — the value lies in [0,1]
// This witnesses the positive side: valid rationals construct, and num()/den()
// read back the const params (proving each alias forwards NUM/DEN to the
// correctly-tagged Metric). The NUM>DEN negative side is the trybuild fixture
// ui/compile_fail/fitness_const_num_gt_den.rs.

use wasm4pm_compat::conformance::{
    F1Const, FitnessConst, GeneralizationConst, PrecisionConst, SimplicityConst,
};

#[test]
fn quality_const_aliases_construct_and_read_back() {
    let fitness = FitnessConst::<3, 4>::new();
    assert_eq!(fitness.num(), 3);
    assert_eq!(fitness.den(), 4);

    // Boundary: NUM == DEN (value 1.0) satisfies both laws.
    let precision = PrecisionConst::<1, 1>::new();
    assert_eq!(precision.num(), 1);
    assert_eq!(precision.den(), 1);

    let f1 = F1Const::<2, 3>::new();
    assert_eq!(f1.num(), 2);
    assert_eq!(f1.den(), 3);

    // Boundary: NUM == 0 (value 0.0) with DEN > 0 satisfies both laws.
    let generalization = GeneralizationConst::<0, 1>::new();
    assert_eq!(generalization.num(), 0);
    assert_eq!(generalization.den(), 1);

    let simplicity = SimplicityConst::<5, 5>::new();
    assert_eq!(simplicity.num(), 5);
    assert_eq!(simplicity.den(), 5);
    // NUM > DEN (e.g. FitnessConst::<4,3>) does NOT compile — see the trybuild
    // fixture fitness_const_num_gt_den.rs.
}
