// Reference witness: the OCEL-2.0 event attribute type (OCELEventAttribute) and
// its value union (OCELAttributeValue), COVERAGE.md §2 — OCEL-2.0 attributes.
//
// Distinct from the OcelLog-flavored OcelAttribute: OCELEventAttribute is the
// OCEL-2.0 event attribute (name + OCELAttributeValue). Its string/integer
// builders land in the correct value variant, and the value union covers
// Integer/Float/Boolean/Time/String/Null. (OCELObjectAttribute additionally
// carries a chrono timestamp and is noted, not constructed here.)

use wasm4pm_compat::ocel::{OCELAttributeValue as V, OCELEventAttribute};

#[test]
fn ocel2_event_attribute_builders_map_to_value_variants() {
    let s = OCELEventAttribute::string("actor", "alice".to_string());
    assert_eq!(s.name, "actor");
    assert!(
        matches!(&s.value, V::String(x) if x == "alice"),
        "string builder → String"
    );

    let i = OCELEventAttribute::integer("amount", 42);
    assert_eq!(i.name, "amount");
    assert!(
        matches!(i.value, V::Integer(42)),
        "integer builder → Integer"
    );
}

#[test]
fn ocel2_attribute_value_union_is_constructible() {
    // The value union covers all OCEL-2.0 attribute value kinds (Time is
    // chrono-backed and elsewhere; the non-temporal ones are constructed here).
    let values = [
        V::Integer(7),
        V::Float(1.5),
        V::Boolean(true),
        V::String("x".to_string()),
        V::Null,
    ];
    fn tag(v: &V) -> &'static str {
        match v {
            V::Integer(_) => "int",
            V::Float(_) => "float",
            V::Boolean(_) => "bool",
            V::String(_) => "string",
            V::Null => "null",
            V::Time(_) => "time",
        }
    }
    let kinds: std::collections::BTreeSet<&str> = values.iter().map(tag).collect();
    assert_eq!(kinds.len(), 5, "five non-temporal value kinds constructed");
}
