// Reference witness: the XesEvent standard-attribute accessors — concept:name,
// time:timestamp, org:resource, lifecycle:transition (COVERAGE.md §2 — XES event
// perspectives), incl. parsing the lifecycle transition into the typed enum.
//
// XesEvent is a (key,value) attribute bag with convenience accessors over the XES
// standard keys. lifecycle_transition() parses the raw string into the typed
// XesLifecycleTransition; an unrecognised value is surfaced raw but not typed.

use wasm4pm_compat::xes::{XesEvent, XesLifecycleTransition};

#[test]
fn xes_event_exposes_standard_perspectives() {
    let e = XesEvent::new()
        .with("concept:name", "place_order")
        .with("time:timestamp", "2024-01-01T00:00:00Z")
        .with("org:resource", "alice")
        .with("lifecycle:transition", "complete");

    assert_eq!(e.concept_name(), Some("place_order"), "control-flow");
    assert_eq!(e.timestamp(), Some("2024-01-01T00:00:00Z"), "time");
    assert_eq!(e.resource(), Some("alice"), "resource");
    assert_eq!(
        e.lifecycle_transition(),
        Some(XesLifecycleTransition::Complete),
        "lifecycle parsed into the typed transition"
    );
    assert_eq!(
        e.lifecycle_transition_raw(),
        Some("complete"),
        "raw lifecycle string preserved"
    );
}

#[test]
fn unset_standard_attributes_are_none() {
    let bare = XesEvent::new().with("custom:key", "v");
    assert_eq!(bare.concept_name(), None);
    assert_eq!(bare.resource(), None);
    assert_eq!(bare.lifecycle_transition(), None, "no lifecycle set → None");
    assert_eq!(
        bare.attribute("custom:key"),
        Some("v"),
        "arbitrary attribute readable"
    );
    assert_eq!(bare.attributes().len(), 1, "one attribute on the bag");
}
