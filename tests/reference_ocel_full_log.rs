// Reference witness: a COMPLETE OCEL-2.0 log assembled from the full type surface
// (events with e2o relationships + objects with attributes), exercising every
// query together (COVERAGE.md §2 — composite OCEL-2.0 cross-product).
//
// This is a cross-product witness: a single OCEL log built from OCELEvent (with
// relationships), OCELObject (with attributes), then queried via event_set /
// object_set / count_objects_of_type / e2o / eval — proving the pieces compose
// into a coherent whole, not just construct in isolation.

use wasm4pm_compat::ocel::{
    OCELAttributeValue, OCELEvent, OCELEventAttribute, OCELObject, OCELRelationship, OCEL,
};

#[test]
fn complete_ocel_log_queries_compose() {
    // Two events: one links to two objects with qualified roles + an attribute.
    let mut e1 = OCELEvent::new("e1".to_string(), "place_order")
        .with_attribute(OCELEventAttribute::integer("amount", 100));
    e1.relationships
        .push(OCELRelationship::new("e1".to_string(), "ord-1".to_string()));
    e1.relationships
        .push(OCELRelationship::new("e1".to_string(), "itm-1".to_string()));
    let e2 = OCELEvent::new("e2".to_string(), "ship");

    let objects = vec![
        OCELObject::new("ord-1".to_string(), "order"),
        OCELObject::new("itm-1".to_string(), "item"),
        OCELObject::new("itm-2".to_string(), "item"),
    ];
    let log = OCEL::new(vec![e1, e2], objects);

    // Counts across the whole log.
    assert_eq!(log.event_set().len(), 2, "two events");
    assert_eq!(log.object_set().len(), 3, "three objects");
    assert_eq!(log.count_objects_of_type("item"), 2, "two items");
    assert_eq!(log.count_objects_of_type("order"), 1, "one order");

    // e2o links of e1 (event → objects).
    let links = log.e2o("e1");
    assert_eq!(links.len(), 2, "e1 links to two objects");
    assert!(links.iter().any(|(oid, _)| *oid == "ord-1"));
    assert!(links.iter().any(|(oid, _)| *oid == "itm-1"));
    assert!(log.e2o("e2").is_empty(), "e2 has no object links");

    // eval over e1's attributes.
    let val = log.eval("e1").expect("e1 has attributes");
    match val.get("amount") {
        Some(OCELAttributeValue::Integer(n)) => assert_eq!(*n, 100),
        other => panic!("amount should be Integer(100); got {other:?}"),
    }
}
