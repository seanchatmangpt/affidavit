// Reference witness: ConformanceResult builder CLAMPING — with_precision/
// with_generalization/with_simplicity coerce out-of-range and NaN inputs into
// [0,1] (clamp_finite), never storing an out-of-range or NaN metric
// (COVERAGE.md §2 — conformance builder clamp law).
//
// The builder accepts a raw f64 (a metric an engine produced) and stores it
// clamped to [0,1], with NaN coerced to the low bound (0.0) — so a stored metric
// is always a valid [0,1] number, never NaN/±inf/out-of-range.

use wasm4pm_compat::conformance::ConformanceResult;

#[test]
fn builder_clamps_out_of_range_into_unit_interval() {
    let r = ConformanceResult::new(0.5, 1, 1, 0)
        .with_precision(1.7)        // > 1 → clamped to 1.0
        .with_generalization(-0.3)  // < 0 → clamped to 0.0
        .with_simplicity(0.4);      // in range → kept
    assert_eq!(r.precision, Some(1.0), "above-1 precision clamped to 1.0");
    assert_eq!(r.generalization, Some(0.0), "below-0 generalization clamped to 0.0");
    assert_eq!(r.simplicity, Some(0.4), "in-range simplicity kept");
}

#[test]
fn builder_coerces_nan_to_low_bound() {
    let r = ConformanceResult::new(0.5, 1, 1, 0).with_precision(f64::NAN);
    let p = r.precision.expect("precision set");
    assert!(!p.is_nan(), "NaN must not be stored");
    assert_eq!(p, 0.0, "NaN coerced to the conservative low bound");

    // ±inf also coerced into range.
    let inf = ConformanceResult::new(0.5, 1, 1, 0).with_generalization(f64::INFINITY);
    assert_eq!(inf.generalization, Some(1.0), "+inf clamped to upper bound");
}
