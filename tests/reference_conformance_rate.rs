// Reference witness: ConformanceResult::conformance_rate — the trace-fitting
// ratio derivation, with the empty-log guard (COVERAGE.md §2 — derived rate).
//
// conformance_rate = fitting_traces / total_traces, guarded so an empty log
// (total == 0) yields 0.0 rather than NaN (division-by-zero). This witnesses the
// derivation at representative points and the guard.

use wasm4pm_compat::conformance::ConformanceResult;

#[test]
fn conformance_rate_is_fitting_over_total() {
    // 7 of 10 traces fit → 0.7
    let r = ConformanceResult::new(0.0, 10, 7, 3);
    assert!((r.conformance_rate() - 0.7).abs() < 1e-9, "7/10 = 0.7");

    // All fit → 1.0
    let perfect = ConformanceResult::new(0.0, 5, 5, 0);
    assert_eq!(perfect.conformance_rate(), 1.0);

    // None fit → 0.0
    let none = ConformanceResult::new(0.0, 4, 0, 4);
    assert_eq!(none.conformance_rate(), 0.0);
}

#[test]
fn empty_log_rate_is_guarded_to_zero_not_nan() {
    let empty = ConformanceResult::new(0.0, 0, 0, 0);
    let rate = empty.conformance_rate();
    assert_eq!(rate, 0.0, "empty log → 0.0, not NaN (division-by-zero guarded)");
    assert!(!rate.is_nan());
}
