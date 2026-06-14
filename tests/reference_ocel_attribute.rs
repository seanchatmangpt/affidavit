// Reference witness: the typed OcelAttribute builders each produce the correct
// key + value-variant (COVERAGE.md §2 — OCEL typed-attribute builder).
//
// OcelAttribute::{boolean,integer,float,string,timestamp_ns} are the ergonomic
// constructors for typed OCEL attribute values. This exercises each, asserting
// the key is carried and the value lands in the correct OcelAttributeValue variant.
//
// Failing-when-fake: if a builder put the value in the wrong variant (e.g. integer
// as a String), the match below fails; removing wasm4pm-compat fails to compile.

use wasm4pm_compat::ocel::{OcelAttribute, OcelAttributeValue as V};

#[test]
fn typed_attribute_builders_produce_correct_variants() {
    let b = OcelAttribute::boolean("flag", true);
    assert_eq!(b.key, "flag");
    assert!(matches!(b.value, V::Boolean(true)), "boolean builder → Boolean");

    let i = OcelAttribute::integer("count", 7);
    assert_eq!(i.key, "count");
    assert!(matches!(i.value, V::Integer(7)), "integer builder → Integer");

    let f = OcelAttribute::float("ratio", 0.5);
    assert!(matches!(f.value, V::Float(x) if (x - 0.5).abs() < 1e-9), "float builder → Float");

    let s = OcelAttribute::string("name", "alice");
    assert!(matches!(&s.value, V::String(x) if x == "alice"), "string builder → String");

    let t = OcelAttribute::timestamp_ns("ts", 1_000);
    assert!(matches!(t.value, V::TimestampNs(1_000)), "timestamp_ns builder → TimestampNs");
}

#[test]
fn generic_new_carries_arbitrary_value() {
    let a = OcelAttribute::new("nested", V::List(vec![V::Integer(1), V::Null]));
    assert_eq!(a.key, "nested");
    match a.value {
        V::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(items[0], V::Integer(1)));
            assert!(matches!(items[1], V::Null));
        }
        other => panic!("expected List; got {other:?}"),
    }
}
