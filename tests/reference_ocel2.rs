// Reference witness: the richer OCEL 2.0 surface is CONSTRUCTED and queried
// (COVERAGE.md §2.2 — advances the OCEL-2.0 row from 🔴 to 🟢).
//
// Beyond the `OcelLog` admission shape, wasm4pm-compat exports the full OCEL 2.0
// object model: `OCEL` (log), `OCELEvent`, `OCELObject`, `OCELRelationship`.
// This builds a real OCEL 2.0 log and exercises its query surface — the
// positive (admit) side of the type, complementing the refusal witnesses.
//
// Failing-when-fake: remove wasm4pm-compat → no compile; if the query methods
// returned constants, the counts below would not reflect the constructed log.

use wasm4pm_compat::ocel::{OCELEvent, OCELObject, OCEL};

#[test]
fn ocel2_log_is_constructed_and_queryable() {
    // Two events, three objects across two object types.
    let events = vec![
        OCELEvent::new("e1".to_string(), "create"),
        OCELEvent::new("e2".to_string(), "release"),
    ];
    let objects = vec![
        OCELObject::new("o1".to_string(), "artifact"),
        OCELObject::new("o2".to_string(), "artifact"),
        OCELObject::new("agent-1".to_string(), "agent"),
    ];
    let log = OCEL::new(events, objects);

    // The query surface reflects the constructed log (not constants).
    assert_eq!(log.event_set().len(), 2, "two events constructed");
    assert_eq!(log.object_set().len(), 3, "three objects constructed");
    assert_eq!(
        log.count_objects_of_type("artifact"),
        2,
        "two artifact-typed objects"
    );
    assert_eq!(
        log.count_objects_of_type("agent"),
        1,
        "one agent-typed object"
    );
    assert_eq!(
        log.count_objects_of_type("nonexistent"),
        0,
        "an undeclared object type has zero objects"
    );
}

#[test]
fn ocel2_event_and_object_carry_their_identity() {
    let e = OCELEvent::new("e1".to_string(), "create");
    let o = OCELObject::new("o1".to_string(), "artifact");
    // Constructed values carry the identity they were built with.
    assert_eq!(e.id, "e1");
    assert_eq!(e.event_type, "create");
    assert_eq!(o.id, "o1");
    assert_eq!(o.object_type, "artifact");
}
