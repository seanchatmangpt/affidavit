// Reference witness: the typed quality metric `Metric<KIND, NUM, DEN>` — binds a
// QualityMetricKind to a COMPILE-TIME-BOUNDED [0,1] value (COVERAGE.md §2 — typed
// metric carrier).
//
// Metric<KIND, NUM, DEN> carries the same `Require<{DEN>0}>` & `Require<{NUM<=DEN}>`
// where-bounds as Between01 AND tags the value with its quality dimension (KIND).
// So `Metric<Fitness, 3, 4>` is "fitness = 0.75", a distinct type from
// `Metric<Precision, 3, 4>` — a fitness value cannot be silently used as precision,
// and an out-of-range value is unconstructable.

use wasm4pm_compat::conformance::Metric;
use wasm4pm_compat::law::QualityMetricKind;

#[test]
fn typed_metrics_carry_dimension_and_bounded_value() {
    // Fitness = 3/4 = 0.75
    let fitness = Metric::<{ QualityMetricKind::Fitness }, 3, 4>::new();
    assert_eq!(fitness.num(), 3);
    assert_eq!(fitness.den(), 4);

    // Precision = 1/1 = 1.0 (boundary)
    let precision = Metric::<{ QualityMetricKind::Precision }, 1, 1>::new();
    assert_eq!(precision.num(), 1);
    assert_eq!(precision.den(), 1);

    // Simplicity = 0/1 = 0.0
    let simplicity = Metric::<{ QualityMetricKind::Simplicity }, 0, 1>::new();
    assert_eq!(simplicity.num(), 0);
    assert_eq!(simplicity.den(), 1);

    // The KIND const distinguishes the dimensions at the type level:
    // Metric<Fitness,3,4> and Metric<Precision,3,4> are DIFFERENT types even
    // though their value is identical. A fitness metric cannot be passed where a
    // precision metric is expected (mix-up is a compile error, not a runtime bug).
    // An out-of-range value (e.g. Metric<Fitness, 4, 3>) fails the where-bound and
    // cannot be written — the [0,1] law holds at the type level.
}
