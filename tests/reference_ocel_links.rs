// Reference witness: the OCEL relationship/change link types' own accessor
// surface — EventObjectLink, ObjectObjectLink, ObjectChange — with qualifiers
// and timestamps (COVERAGE.md §2 — ocel link/change accessors).
//
// These are the typed edges of an OcelLog: e2o links (event→object, qualified),
// o2o links (object→object, qualified), and object changes (attribute updates,
// timestamped). This witnesses their builders + accessors.

use wasm4pm_compat::ocel::{EventObjectLink, ObjectChange, ObjectObjectLink};

#[test]
fn event_object_link_carries_ids_and_optional_qualifier() {
    let plain = EventObjectLink::new("e1", "o1");
    assert_eq!(plain.event_id(), "e1");
    assert_eq!(plain.object_id(), "o1");
    assert_eq!(plain.qualifier(), None, "no qualifier by default");

    let qualified = EventObjectLink::new("e1", "o1").qualified("input");
    assert_eq!(
        qualified.qualifier(),
        Some("input"),
        "qualified role recovered"
    );
}

#[test]
fn object_object_link_carries_source_target_qualifier() {
    let l = ObjectObjectLink::new("order-1", "item-9").qualified("contains");
    assert_eq!(l.source_id(), "order-1");
    assert_eq!(l.target_id(), "item-9");
    assert_eq!(l.qualifier(), Some("contains"));
}

#[test]
fn object_change_carries_attribute_value_and_optional_time() {
    let c = ObjectChange::new("order-1", "status", "shipped").at_ns(5_000);
    assert_eq!(c.object_id(), "order-1");
    assert_eq!(c.attribute(), "status");
    assert_eq!(c.value(), "shipped");
    assert_eq!(c.timestamp_ns(), Some(5_000), "change is timestamped");

    let untimed = ObjectChange::new("o", "a", "v");
    assert_eq!(
        untimed.timestamp_ns(),
        None,
        "no timestamp → None (not fabricated)"
    );
}
