// Reference witness: the XesLog / XesTrace accessor surface — name, declared
// extensions, traces, and per-trace events (COVERAGE.md §2 — XES log structure;
// complements the XesRefusal validate witnesses).
//
// A XesLog is a named, extension-declaring collection of XesTraces. This
// witnesses the read surface returns the constructed structure.

use wasm4pm_compat::xes::{XesEvent, XesExtension, XesLog, XesTrace};

#[test]
fn xes_log_exposes_name_extensions_and_traces() {
    let log = XesLog::new(
        "sales-log",
        [XesExtension::new("Concept", "concept", "uri")],
        [
            XesTrace::new("case-1", [XesEvent::new().with("concept:name", "create")]),
            XesTrace::new(
                "case-2",
                [
                    XesEvent::new().with("concept:name", "create"),
                    XesEvent::new().with("concept:name", "release"),
                ],
            ),
        ],
    );

    assert_eq!(log.name(), "sales-log");
    assert_eq!(log.extensions().len(), 1, "one declared extension");
    assert_eq!(log.extensions()[0].prefix(), "concept");
    assert_eq!(log.traces().len(), 2, "two traces");

    // Per-trace read surface.
    assert_eq!(log.traces()[0].name(), "case-1");
    assert_eq!(log.traces()[1].len(), 2, "case-2 has two events");
    assert_eq!(log.traces()[1].events()[1].concept_name(), Some("release"));
}
