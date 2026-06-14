// Reference witness: the OCEDO formal query `eval(event_id)` returns an event's
// attribute valuation, exercised against constructed attributes
// (COVERAGE.md §2.2 — OCEL eval/attribute layer).
//
// `OCEL::eval(e)` is the L = (E, O, eval, oaval) formal layer: it maps an event
// to its {name → value} attribute valuation. This builds an event carrying a
// string and an integer attribute and queries them back, confirming eval reads
// the constructed attribute set (not constants) and returns None for unknown ids.

use wasm4pm_compat::ocel::{OCEL, OCELAttributeValue, OCELEvent, OCELEventAttribute};

#[test]
fn eval_returns_constructed_event_attribute_valuation() {
    let e = OCELEvent::new("e1".to_string(), "create")
        .with_attribute(OCELEventAttribute::string("actor", "alice".to_string()))
        .with_attribute(OCELEventAttribute::integer("amount", 42));
    let log = OCEL::new(vec![e], vec![]);

    let map = log.eval("e1").expect("e1 has an attribute valuation");
    assert_eq!(map.len(), 2, "two attributes constructed");

    match map.get("actor") {
        Some(OCELAttributeValue::String(s)) => assert_eq!(s, "alice"),
        other => panic!("actor should be String(alice); got {other:?}"),
    }
    match map.get("amount") {
        Some(OCELAttributeValue::Integer(n)) => assert_eq!(*n, 42),
        other => panic!("amount should be Integer(42); got {other:?}"),
    }
}

#[test]
fn eval_returns_none_for_unknown_event() {
    let log = OCEL::new(vec![OCELEvent::new("e1".to_string(), "create")], vec![]);
    assert!(log.eval("nonexistent").is_none(), "unknown event id → None, not a fabricated map");
}
