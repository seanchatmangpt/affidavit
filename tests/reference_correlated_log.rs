// Reference witness: the cross-log CorrelatedLog<A, B, const SCHEMA> carrier — two
// logs correlated under a named schema (COVERAGE.md §2 — cross-log correlation).
//
// CorrelatedLog pairs two log types (A, B) under a const SCHEMA string (by-case,
// by-object, …) — the type-level record that two logs were correlated by a
// specific strategy. This witnesses construction under distinct schemas and the
// schema() const accessor.

use wasm4pm_compat::correlation::CorrelatedLog;

// Two distinct log marker types.
enum OrderLog {}
enum ShipmentLog {}

#[test]
fn correlated_log_carries_its_schema_const() {
    let by_object: CorrelatedLog<OrderLog, ShipmentLog, "by-object"> = CorrelatedLog::new();
    assert_eq!(
        by_object.schema(),
        "by-object",
        "correlation schema recovered"
    );

    let by_case: CorrelatedLog<OrderLog, ShipmentLog, "by-case"> = CorrelatedLog::new();
    assert_eq!(by_case.schema(), "by-case");

    // Distinct schemas → distinct correlation types.
    assert_ne!(by_object.schema(), by_case.schema());
}
