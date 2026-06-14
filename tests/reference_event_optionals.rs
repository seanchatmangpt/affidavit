// Reference witness: the eventlog Event optional-perspective surface — an event
// carries only the perspectives explicitly set; unset ones are None, not
// fabricated (COVERAGE.md §2 — Event optionals).
//
// Event::new(activity) sets only control-flow; the time/resource/lifecycle
// perspectives are added by builders (.at_ns/.by/.with_lifecycle). This witnesses
// that an event with NONE of them returns None for each (no fabricated defaults),
// and that partial sets are honoured.

use wasm4pm_compat::eventlog::Event;

#[test]
fn unset_perspectives_are_none_not_fabricated() {
    let bare = Event::new("create");
    assert_eq!(bare.activity(), "create", "control-flow always present");
    assert_eq!(bare.timestamp_ns(), None, "no time set → None");
    assert_eq!(bare.resource(), None, "no resource set → None");
    assert_eq!(bare.lifecycle(), None, "no lifecycle set → None");
}

#[test]
fn partial_perspective_sets_are_honoured() {
    // Only resource set.
    let by_alice = Event::new("approve").by("alice");
    assert_eq!(by_alice.resource(), Some("alice"));
    assert_eq!(by_alice.timestamp_ns(), None, "time still unset");
    assert_eq!(by_alice.lifecycle(), None, "lifecycle still unset");

    // Only time set.
    let at_t = Event::new("ship").at_ns(42);
    assert_eq!(at_t.timestamp_ns(), Some(42));
    assert_eq!(at_t.resource(), None);
}
