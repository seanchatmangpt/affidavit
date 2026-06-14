// Reference witness: the OCEL object-type CARDINALITY law is constructed and
// exercised against its [min,max] window (COVERAGE.md §2 — OCEL cardinality law).
//
// `ObjectTypeCardinality::admits(count)` is a real law: it accepts a count iff it
// lies in the inclusive [min_count, max_count] window (either bound may be None =
// unbounded). This witnesses the law at its boundaries and outside them.
//
// Failing-when-fake: if admits() ignored its bounds (e.g. always true), the
// out-of-range assertions below fail; removing wasm4pm-compat fails to compile.

use wasm4pm_compat::ocel::ObjectTypeCardinality;

#[test]
fn cardinality_admits_within_window_and_refuses_outside() {
    let card = ObjectTypeCardinality {
        min_count: Some(1),
        max_count: Some(3),
        ..Default::default()
    };
    // Inside / on the boundaries → admitted.
    assert!(card.admits(1), "lower boundary admitted");
    assert!(card.admits(2), "interior admitted");
    assert!(card.admits(3), "upper boundary admitted");
    // Outside → refused.
    assert!(!card.admits(0), "below min refused");
    assert!(!card.admits(4), "above max refused");
}

#[test]
fn cardinality_unbounded_sides_are_open() {
    // No min → nothing is too small; no max → nothing is too large.
    let no_min = ObjectTypeCardinality { min_count: None, max_count: Some(2), ..Default::default() };
    assert!(no_min.admits(0), "unbounded below admits zero");
    assert!(!no_min.admits(3), "still bounded above");

    let no_max = ObjectTypeCardinality { min_count: Some(5), max_count: None, ..Default::default() };
    assert!(!no_max.admits(4), "still bounded below");
    assert!(no_max.admits(1_000), "unbounded above admits large counts");

    let unbounded = ObjectTypeCardinality::default();
    assert!(unbounded.admits(0) && unbounded.admits(usize::MAX), "fully unbounded admits everything");
}

#[test]
fn cardinality_carries_lifecycle_event_types() {
    // The created_by / terminated_by fields carry the object's lifecycle-opening
    // and -closing event types — constructed here so the fields are exercised.
    let card = ObjectTypeCardinality {
        created_by: vec!["create".to_string()],
        terminated_by: vec!["release".to_string()],
        ..Default::default()
    };
    assert_eq!(card.created_by, vec!["create".to_string()]);
    assert_eq!(card.terminated_by, vec!["release".to_string()]);
}
