// Reference witness: the Trace read surface (case_id / len / is_empty / events),
// COVERAGE.md §2 — trace accessors (complements the per-trace validate law).
//
// A Trace is a case: a case_id plus an ordered list of events. This witnesses the
// accessors return the constructed case identity and event sequence, for both the
// `new(case_id, events)` and `from_events` constructors.

use wasm4pm_compat::eventlog::{Event, Trace};

#[test]
fn trace_with_explicit_case_id_exposes_identity_and_events() {
    let t = Trace::new("case-7", [Event::new("create").at_ns(1), Event::new("release").at_ns(2)]);
    assert_eq!(t.case_id(), "case-7", "explicit case id recovered");
    assert_eq!(t.len(), 2, "two events");
    assert!(!t.is_empty());
    assert_eq!(t.events()[0].activity(), "create");
    assert_eq!(t.events()[1].activity(), "release");
}

#[test]
fn empty_from_events_trace_is_empty() {
    let t = Trace::from_events(Vec::<Event>::new());
    assert!(t.is_empty(), "no events → empty");
    assert_eq!(t.len(), 0);
    assert!(t.events().is_empty());
}
