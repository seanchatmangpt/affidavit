// Reference witness: the OCEL log structure — a collection of typed events linked
// to typed objects (COVERAGE.md §2 — OCEL log structure; complements the OCEL
// admission and relationship witnesses).
//
// An OCEL log is a set of OCELEvents and OCELObjects. Events carry an event_type
// and may link to objects via OCELRelationship. This witnesses the read surface
// returns the constructed structure.

use wasm4pm_compat::ocel::{OCELEvent, OCELObject, OCELRelationship, OCEL};

#[test]
fn ocel_log_exposes_event_and_object_sets() {
    // Two events across two object types (order + item).
    let events = vec![
        OCELEvent::new("e1".to_string(), "create"),
        OCELEvent::new("e2".to_string(), "release"),
    ];
    let objects = vec![
        OCELObject::new("ord-1".to_string(), "order"),
        OCELObject::new("itm-1".to_string(), "item"),
        OCELObject::new("itm-2".to_string(), "item"),
    ];
    let log = OCEL::new(events, objects);

    // Event and object set sizes reflect the constructed log.
    assert_eq!(log.event_set().len(), 2, "two events constructed");
    assert_eq!(log.object_set().len(), 3, "three objects constructed");
    assert_eq!(log.count_objects_of_type("order"), 1, "one order-typed object");
    assert_eq!(log.count_objects_of_type("item"), 2, "two item-typed objects");

    // Event identity.
    let evts = log.event_set();
    let e1 = evts.iter().find(|e| e.id == "e1").expect("e1 present");
    assert_eq!(e1.event_type, "create");
    let e2 = evts.iter().find(|e| e.id == "e2").expect("e2 present");
    assert_eq!(e2.event_type, "release");

    // Object identity.
    let objs = log.object_set();
    let ord = objs.iter().find(|o| o.id == "ord-1").expect("ord-1 present");
    assert_eq!(ord.object_type, "order");
    let itm = objs.iter().find(|o| o.id == "itm-1").expect("itm-1 present");
    assert_eq!(itm.object_type, "item");
}

#[test]
fn ocel_log_event_links_objects_via_relationships() {
    // e1 links to two objects; e2 has no links.
    let mut e1 = OCELEvent::new("e1".to_string(), "place_order");
    e1.relationships
        .push(OCELRelationship::new("e1".to_string(), "ord-1".to_string()));
    e1.relationships
        .push(OCELRelationship::new("e1".to_string(), "itm-1".to_string()));
    let e2 = OCELEvent::new("e2".to_string(), "ship");

    let objects = vec![
        OCELObject::new("ord-1".to_string(), "order"),
        OCELObject::new("itm-1".to_string(), "item"),
    ];
    let log = OCEL::new(vec![e1, e2], objects);

    assert_eq!(log.e2o("e1").len(), 2, "e1 links to two objects");
    assert!(log.e2o("e2").is_empty(), "e2 has no object links");
}
