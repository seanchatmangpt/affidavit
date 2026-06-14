// Reference witness: the zero-cost typed name wrappers (ids module —
// ObjectTypeName<K> / EventTypeName<K>), COVERAGE.md §2 (id wrappers).
//
// These are `Cow`-backed, log-kind-parameterised name types: ObjectTypeName<K>
// labels an object type, EventTypeName<K> labels an event type, both tagged by a
// log marker K so names from different logs (or different roles) cannot be
// silently mixed. Witnesses from_static / from_owned construction and as_str
// recovery.

use wasm4pm_compat::ids::{EventTypeName, ObjectTypeName};

// A log marker — distinguishes names belonging to this log at the type level.
enum MyLog {}

#[test]
fn object_type_names_construct_and_recover_label() {
    let from_static = ObjectTypeName::<MyLog>::from_static("order");
    assert_eq!(from_static.as_str(), "order", "static label recovered");

    let from_owned = ObjectTypeName::<MyLog>::from_owned(String::from("item"));
    assert_eq!(from_owned.as_str(), "item", "owned label recovered");
}

#[test]
fn event_type_names_construct_and_recover_label() {
    let place = EventTypeName::<MyLog>::from_static("place_order");
    assert_eq!(place.as_str(), "place_order");

    let ship = EventTypeName::<MyLog>::from_owned(String::from("ship_item"));
    assert_eq!(ship.as_str(), "ship_item");

    // ObjectTypeName and EventTypeName are DISTINCT types: an object-type label
    // cannot be used where an event-type label is expected (control-flow vs
    // object taxonomy kept separate at the type level). They only share `as_str`.
    let obj = ObjectTypeName::<MyLog>::from_static("order");
    assert_ne!(obj.as_str(), place.as_str(), "distinct labels");
}
