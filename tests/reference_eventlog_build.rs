// Reference witness: positive EventLog construction carrying the control-flow,
// resource, and time perspectives, with accessors + counts + validation
// (COVERAGE.md §2 — eventlog Event/Trace/EventLog positive surface).
//
// Complements the EventLogRefusal witnesses (which exercise the REFUSAL side):
// here we build a well-formed multiperspective log and read it back. Each Event
// carries activity (control-flow), timestamp (time), resource (resource), and a
// lifecycle transition — van der Aalst's perspectives on a single event.
//
// Failing-when-fake: if a builder dropped a perspective, the accessor assertion
// fails; if counts were stubbed, trace_count/event_count fail; remove the dep → no compile.

use wasm4pm_compat::eventlog::{Event, EventLog, Trace};

#[test]
fn event_carries_all_perspectives() {
    let e = Event::new("place_order")
        .at_ns(1_000)
        .by("alice")
        .with_lifecycle("complete");
    assert_eq!(e.activity(), "place_order", "control-flow perspective");
    assert_eq!(e.timestamp_ns(), Some(1_000), "time perspective");
    assert_eq!(e.resource(), Some("alice"), "resource perspective");
    assert_eq!(e.lifecycle(), Some("complete"), "lifecycle transition");
}

#[test]
fn well_formed_log_validates_and_counts_correctly() {
    let t1 = Trace::new(
        "case-1",
        [
            Event::new("create").at_ns(1),
            Event::new("transform").at_ns(2),
            Event::new("release").at_ns(3),
        ],
    );
    let t2 = Trace::from_events([
        Event::new("create").at_ns(1),
        Event::new("release").at_ns(2),
    ]);
    let log = EventLog::from_traces([t1, t2]);

    assert_eq!(log.trace_count(), 2, "two cases");
    assert_eq!(log.event_count(), 5, "3 + 2 events");
    // The per-trace validation law accepts a non-empty, monotonic trace.
    assert!(
        log.traces()[0].validate().is_ok(),
        "monotonic 3-event trace admits"
    );
    assert!(
        log.traces()[1].validate().is_ok(),
        "monotonic 2-event trace admits"
    );
}
