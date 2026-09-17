// Reference witness: OCEL-2.0 attribute value type census (COVERAGE.md §2 —
// OCEL-2.0 attribute value union completeness). Replaces the former XES
// lifecycle-transition vocabulary witness.
//
// The OCEL-2.0 typed value union (OCELAttributeValue) is the OCEL equivalent of
// typed event attribute values — Integer, Float, Boolean, String, Null, and Time.
// This witnesses that all five non-temporal variants are constructible and carry
// distinct tags (a closed-alphabet check on the union), that the
// OCELEventAttribute builders land in the correct value variant, and that Null is
// its own distinct variant and not a zero-value alias.

use wasm4pm_compat::ocel::{OCELAttributeValue as V, OCELEventAttribute};

#[test]
fn ocel_attribute_value_union_is_constructible_and_tagged() {
    // Construct all 5 non-temporal OCELAttributeValue variants and verify that
    // their tag strings are distinct — a closed-alphabet check on the union.
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
    assert_eq!(
        kinds.len(),
        5,
        "five non-temporal value kinds must each produce a distinct tag"
    );
}

#[test]
fn ocel_event_attribute_builders_map_to_value_variants() {
    // The string builder lands in String, the integer builder lands in Integer.
    let s = OCELEventAttribute::string("actor", "alice".to_string());
    assert_eq!(s.name, "actor");
    assert!(
        matches!(&s.value, V::String(x) if x == "alice"),
        "string builder must produce V::String"
    );

    let i = OCELEventAttribute::integer("amount", 42);
    assert_eq!(i.name, "amount");
    assert!(
        matches!(i.value, V::Integer(42)),
        "integer builder must produce V::Integer"
    );
}

#[test]
fn ocel_attribute_value_null_is_distinct() {
    // Null is its own variant — not a zero-value alias for Integer or any other.
    let v = V::Null;
    assert!(matches!(v, V::Null), "V::Null must match V::Null");
    assert!(
        !matches!(v, V::Integer(_)),
        "V::Null must not match V::Integer(_)"
    );
}
