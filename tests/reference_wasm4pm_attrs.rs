// Reference witness: the wasm4pm engine-side AttributeValue union + Event/Trace
// attribute bags (COVERAGE.md §2 — wasm4pm log attribute model).
//
// wasm4pm Event/Trace are HashMap<String, AttributeValue> bags. AttributeValue
// covers String/Int/Float/Date/Boolean/List/Container (recursive). This witnesses
// constructing an Event with each value kind and reading them back.

use std::collections::HashMap;
use wasm4pm::models::{AttributeValue as V, Event};

#[test]
fn event_attribute_bag_holds_each_value_kind() {
    let mut attrs: HashMap<String, V> = HashMap::new();
    attrs.insert("concept:name".to_string(), V::String("create".to_string()));
    attrs.insert("count".to_string(), V::Int(7));
    attrs.insert("score".to_string(), V::Float(0.5));
    attrs.insert("when".to_string(), V::Date("2024-01-01".to_string()));
    attrs.insert("ok".to_string(), V::Boolean(true));
    attrs.insert(
        "tags".to_string(),
        V::List(vec![V::String("a".to_string()), V::String("b".to_string())]),
    );
    attrs.insert(
        "meta".to_string(),
        V::Container(HashMap::from([("k".to_string(), V::Int(1))])),
    );
    let e = Event { attributes: attrs };

    assert!(matches!(e.attributes.get("concept:name"), Some(V::String(s)) if s == "create"));
    assert!(matches!(e.attributes.get("count"), Some(V::Int(7))));
    assert!(matches!(e.attributes.get("ok"), Some(V::Boolean(true))));
    // Recursive variants.
    match e.attributes.get("tags") {
        Some(V::List(items)) => assert_eq!(items.len(), 2),
        other => panic!("tags should be a List; got {other:?}"),
    }
    match e.attributes.get("meta") {
        Some(V::Container(m)) => assert!(matches!(m.get("k"), Some(V::Int(1)))),
        other => panic!("meta should be a Container; got {other:?}"),
    }
}
