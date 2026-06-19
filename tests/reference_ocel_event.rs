// Reference witness: the OCEL event identity surface and typed event-attribute
// builders (COVERAGE.md §2 — OCEL event perspectives).
//
// OCELEvent carries an (id, event_type) identity pair constructed via
// OCELEvent::new. OCELEventAttribute is a (name, value) pair where the value
// lands in the typed OCELAttributeValue union; the string/integer builders map
// to the correct variant and preserve the name field verbatim.

use wasm4pm_compat::ocel::{OCELAttributeValue as V, OCELEvent, OCELEventAttribute};

#[test]
fn ocel_event_carries_id_and_type() {
    let e = OCELEvent::new("evt-1".to_string(), "place_order");

    assert_eq!(e.id, "evt-1", "event identity");
    assert_eq!(e.event_type, "place_order", "event type label");
}

#[test]
fn ocel_event_attribute_typed_values() {
    let s = OCELEventAttribute::string("actor", "alice".to_string());
    assert_eq!(s.name, "actor");
    assert!(
        matches!(&s.value, V::String(x) if x == "alice"),
        "string builder → String variant"
    );

    let i = OCELEventAttribute::integer("amount", 42);
    assert_eq!(i.name, "amount");
    assert!(
        matches!(i.value, V::Integer(42)),
        "integer builder → Integer variant"
    );

    let b_val = V::Boolean(true);
    assert!(
        matches!(b_val, V::Boolean(true)),
        "boolean value variant is constructible directly"
    );
}
