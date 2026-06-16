// Reference witness: the ocel OcelEvent / Object builders (the OcelLog-flavored
// event/object types carrying typed OcelAttribute payloads + timestamps),
// COVERAGE.md §2 — ocel event/object builder surface.
//
// Distinct from the OCEL-2.0 `OCELEvent`/`OCELObject`: these are the `OcelLog`
// node types. This witnesses id/activity/object_type/timestamp accessors and the
// with_attribute builder accumulating typed OcelAttributes.

use wasm4pm_compat::ocel::{Object, OcelAttribute, OcelAttributeValue, OcelEvent};

#[test]
fn ocel_event_builder_carries_activity_time_and_attributes() {
    let e = OcelEvent::new("e1", "place_order")
        .at_ns(1_000)
        .with_attribute(OcelAttribute::integer("amount", 42))
        .with_attribute(OcelAttribute::string("channel", "web"));

    assert_eq!(e.id(), "e1");
    assert_eq!(e.activity(), "place_order");
    assert_eq!(e.timestamp_ns(), Some(1_000), "time perspective");
    assert_eq!(e.attributes().len(), 2, "two typed attributes accumulated");
    assert!(matches!(
        e.attributes()[0].value,
        OcelAttributeValue::Integer(42)
    ));
}

#[test]
fn ocel_object_builder_carries_type_and_attributes() {
    let o = Object::new("o1", "order").with_attribute(OcelAttribute::boolean("priority", true));

    assert_eq!(o.id(), "o1");
    assert_eq!(o.object_type(), "order");
    assert_eq!(o.attributes().len(), 1);
    assert!(matches!(
        o.attributes()[0].value,
        OcelAttributeValue::Boolean(true)
    ));

    // A bare event has no timestamp/attributes (not fabricated).
    let bare = OcelEvent::new("e2", "ship");
    assert_eq!(bare.timestamp_ns(), None);
    assert!(bare.attributes().is_empty());
}
