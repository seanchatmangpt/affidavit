// Reference witness: the conformance metric newtypes enforce the [0,1] bound at
// construction (reject NaN/±∞/out-of-range), and ConformanceResult carries a
// verdict (COVERAGE.md §2 — conformance metric carriers + verdict container).
//
// Each metric newtype (Fitness/Precision/F1/Generalization/Simplicity) validates
// its f64 to be finite and in [0,1] via `new(v) -> Option`. ConformanceResult is
// the verdict container an engine populates. This witnesses the bound law (admit
// in-range, refuse out-of-range/NaN) and the builder's metric accumulation.

use wasm4pm_compat::conformance::{
    ConformanceResult, F1, Fitness, Generalization, Precision, Simplicity,
};

#[test]
fn metric_newtypes_enforce_zero_one_bound() {
    // In-range values are admitted and recoverable.
    assert_eq!(Fitness::new(0.0).map(|m| m.get()), Some(0.0), "lower bound admitted");
    assert_eq!(Fitness::new(1.0).map(|m| m.get()), Some(1.0), "upper bound admitted");
    assert_eq!(Precision::new(0.5).map(|m| m.get()), Some(0.5), "interior admitted");

    // Out-of-range and non-finite are REFUSED (None) — the bound is a real law.
    assert_eq!(Fitness::new(1.5), None, "above 1.0 refused");
    assert_eq!(Precision::new(-0.1), None, "below 0.0 refused");
    assert_eq!(F1::new(f64::NAN), None, "NaN refused");
    assert_eq!(Generalization::new(f64::INFINITY), None, "+inf refused");
    assert_eq!(Simplicity::new(f64::NEG_INFINITY), None, "-inf refused");
}

#[test]
fn conformance_result_accumulates_a_verdict() {
    // fitness + 7 of 10 traces fitting; then layer precision/generalization/simplicity.
    let r = ConformanceResult::new(0.83, 10, 7, 3)
        .with_precision(0.9)
        .with_generalization(0.8)
        .with_simplicity(0.7);

    assert_eq!(r.fitness, 0.83);
    assert_eq!(r.precision, Some(0.9));
    assert_eq!(r.generalization, Some(0.8));
    assert_eq!(r.simplicity, Some(0.7));
    assert_eq!(r.total_traces, 10);
    assert_eq!(r.fitting_traces, 7);
    assert_eq!(r.deviating_traces, 3);
    // conformance_rate is derived from the trace counts (a real computation, in [0,1]).
    let rate = r.conformance_rate();
    assert!((0.0..=1.0).contains(&rate), "conformance_rate in [0,1]; got {rate}");
}
