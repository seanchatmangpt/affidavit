// Reference witness: the OCEL 2.0 RELATIONSHIP query layer (e2o / o2o) is wired
// and exercised against constructed links (COVERAGE.md §2.2 — closes the
// OCELRelationship query row).
//
// Beyond constructing OCEL events/objects, this builds real event→object (e2o)
// and object→object (o2o) relationships and queries them back, proving the
// query methods read the constructed link structure (not constants).

use wasm4pm_compat::ocel::{OCELEvent, OCELObject, OCELRelationship, OCEL};

fn rel(object_id: &str, qualifier: &str) -> OCELRelationship {
    OCELRelationship {
        object_id: object_id.to_string(),
        qualifier: qualifier.to_string(),
    }
}

#[test]
fn e2o_returns_constructed_event_object_links() {
    let mut e = OCELEvent::new("e1".to_string(), "create");
    e.relationships.push(rel("o1", "input"));
    e.relationships.push(rel("o2", "output"));
    let log = OCEL::new(
        vec![e],
        vec![
            OCELObject::new("o1".to_string(), "artifact"),
            OCELObject::new("o2".to_string(), "artifact"),
        ],
    );

    let links = log.e2o("e1");
    assert_eq!(links.len(), 2, "both event-object links returned");
    assert!(links.contains(&("o1", "input")), "input link present");
    assert!(links.contains(&("o2", "output")), "output link present");
    // An unknown event has no links (not a panic, not a constant).
    assert!(log.e2o("nonexistent").is_empty());
}

#[test]
fn o2o_returns_constructed_object_object_links() {
    let mut o1 = OCELObject::new("o1".to_string(), "artifact");
    o1.relationships.push(rel("o2", "derived-from"));
    let log = OCEL::new(
        vec![],
        vec![o1, OCELObject::new("o2".to_string(), "artifact")],
    );

    let links = log.o2o("o1");
    assert_eq!(
        links,
        vec![("o2", "derived-from")],
        "object-object link returned with its qualifier"
    );
    assert!(
        log.o2o("o2").is_empty(),
        "an object with no o2o links returns none"
    );
}
