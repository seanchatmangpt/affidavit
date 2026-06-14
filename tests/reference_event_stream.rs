// Reference witness: the EventStream streaming surface (eventlog.rs) — the online
// counterpart to the batch EventLog (COVERAGE.md §2 — streaming events).
//
// EventStream accumulates events incrementally (push), as in online/streaming
// process mining, vs the batch Trace/EventLog. This witnesses incremental
// accumulation and the empty/len queries.

use wasm4pm_compat::eventlog::{Event, EventStream};

#[test]
fn event_stream_accumulates_incrementally() {
    let mut s = EventStream::new();
    assert!(s.is_empty(), "a fresh stream is empty");
    assert_eq!(s.len(), 0);

    s.push(Event::new("create").at_ns(1));
    s.push(Event::new("transform").at_ns(2));
    s.push(Event::new("release").at_ns(3));

    assert!(!s.is_empty(), "stream has events after pushes");
    assert_eq!(s.len(), 3, "three events streamed in");
}
