// Reference witness: the LOG-LEVEL EventLog::validate law — propagates each
// trace's validation, so one malformed trace refuses the whole log
// (COVERAGE.md §2 — eventlog log-level validation).
//
// Distinct from the per-trace witness: EventLog::validate folds over all traces
// and returns the first refusal. An all-well-formed log admits; a log containing
// one empty trace is refused EmptyTrace.

use wasm4pm_compat::eventlog::{Event, EventLog, EventLogRefusal, Trace};

#[test]
fn log_with_all_well_formed_traces_admits() {
    let log = EventLog::from_traces([
        Trace::from_events([Event::new("a").at_ns(1), Event::new("b").at_ns(2)]),
        Trace::from_events([Event::new("c").at_ns(1)]),
    ]);
    assert_eq!(log.validate(), Ok(()), "all traces well-formed → log admits");
}

#[test]
fn one_malformed_trace_refuses_the_whole_log() {
    let log = EventLog::from_traces([
        Trace::from_events([Event::new("a").at_ns(1)]), // ok
        Trace::from_events(Vec::<Event>::new()),         // empty → EmptyTrace
    ]);
    assert_eq!(
        log.validate(),
        Err(EventLogRefusal::EmptyTrace),
        "an empty trace refuses the whole log (law propagates)"
    );
}

#[test]
fn non_monotonic_trace_refuses_the_log() {
    let log = EventLog::from_traces([
        Trace::from_events([Event::new("a").at_ns(10), Event::new("b").at_ns(5)]), // time goes backward
    ]);
    assert_eq!(log.validate(), Err(EventLogRefusal::NonMonotonicTrace));
}
