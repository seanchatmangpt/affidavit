// Reference witness: recursively-nested OcelAttributeValue structures — List and
// Map nesting to arbitrary depth (COVERAGE.md §2 — OCEL attribute recursion).
//
// OcelAttributeValue::{List, Map} are recursive: a List of Maps of Lists, etc.
// This witnesses constructing and traversing a nested structure, proving the
// recursion composes (not just the flat variants).

use wasm4pm_compat::ocel::OcelAttributeValue as V;

#[test]
fn nested_list_and_map_values_compose() {
    // A map with a nested list value: {"items": [1, 2], "meta": {"flag": true}}
    let nested = V::Map(vec![
        (
            "items".to_string(),
            V::List(vec![V::Integer(1), V::Integer(2)]),
        ),
        (
            "meta".to_string(),
            V::Map(vec![("flag".to_string(), V::Boolean(true))]),
        ),
    ]);

    match &nested {
        V::Map(entries) => {
            assert_eq!(entries.len(), 2, "two top-level keys");
            // Traverse into the nested list.
            match &entries[0].1 {
                V::List(items) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(items[1], V::Integer(2)));
                }
                other => panic!("items should be a List; got {other:?}"),
            }
            // Traverse into the nested map.
            match &entries[1].1 {
                V::Map(inner) => assert!(matches!(inner[0].1, V::Boolean(true))),
                other => panic!("meta should be a Map; got {other:?}"),
            }
        }
        other => panic!("expected a Map; got {other:?}"),
    }
}

#[test]
fn deeply_nested_lists() {
    // [[[42]]] — three levels of list nesting.
    let deep = V::List(vec![V::List(vec![V::List(vec![V::Integer(42)])])]);
    let mut cur = &deep;
    for _ in 0..3 {
        cur = match cur {
            V::List(items) => &items[0],
            other => panic!("expected nested List; got {other:?}"),
        };
    }
    assert!(
        matches!(cur, V::Integer(42)),
        "innermost value reached through 3 list levels"
    );
}
