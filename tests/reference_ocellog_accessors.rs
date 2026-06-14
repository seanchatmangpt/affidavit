// Reference witness: the OcelLog positive ACCESSOR surface — objects/events/
// e2o/o2o/changes are read back as constructed (COVERAGE.md §2.2; complements the
// OcelLog::validate refusal witnesses).
//
// Builds an OcelLog with objects, events, event-object links, object-object links,
// and an object change, then exercises every accessor — confirming the log holds
// and returns the full object-centric structure, not just what validate() checks.

use wasm4pm_compat::ocel::{
    EventObjectLink, Object, ObjectChange, ObjectObjectLink, OcelEvent, OcelLog,
};

#[test]
fn ocel_log_accessors_return_constructed_structure() {
    let log = OcelLog::new(
        vec![Object::new("o1", "order"), Object::new("o2", "item")],
        vec![OcelEvent::new("e1", "create")],
        vec![EventObjectLink::new("e1", "o1")],
        vec![ObjectObjectLink::new("o1", "o2")],
        vec![ObjectChange::new("o1", "status", "shipped")],
    );

    assert_eq!(log.objects().len(), 2, "two objects");
    assert_eq!(log.events().len(), 1, "one event");
    assert_eq!(log.event_object_links().len(), 1, "one e2o link");
    assert_eq!(log.object_object_links().len(), 1, "one o2o link");
    assert_eq!(log.object_changes().len(), 1, "one object change");

    // The log is also link-consistent + non-empty → validate admits it.
    assert_eq!(log.validate(), Ok(()), "a fully-structured log validates");
}
