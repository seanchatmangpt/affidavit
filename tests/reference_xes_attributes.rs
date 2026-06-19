// Reference witness: the OCEL-2.0 type schema declaration and typed attribute
// builder surface (COVERAGE.md §2 — OCEL type schema and attribute builders).
//
// OCELType declares an object/event type with a named list of OCELTypeAttribute
// schema entries (name + value_type string). OcelAttribute::{string,integer,boolean}
// are ergonomic builders that carry the key and land the value in the correct
// OcelAttributeValue variant. This witnesses both layers and their round-trips.

use wasm4pm_compat::ocel::{OcelAttribute, OcelAttributeValue as V, OCELType, OCELTypeAttribute};

#[test]
fn ocel_type_declares_attribute_schema() {
    let order_type = OCELType {
        name: "order".to_string(),
        attributes: vec![
            OCELTypeAttribute {
                name: "total".to_string(),
                value_type: "float".to_string(),
            },
            OCELTypeAttribute {
                name: "priority".to_string(),
                value_type: "boolean".to_string(),
            },
        ],
    };

    assert_eq!(order_type.name, "order", "the declared object type name");
    assert_eq!(order_type.attributes.len(), 2, "two attribute declarations");
    assert_eq!(order_type.attributes[0].name, "total");
    assert_eq!(
        order_type.attributes[0].value_type,
        "float",
        "first attribute's declared value type"
    );
    assert_eq!(order_type.attributes[1].name, "priority");
    assert_eq!(order_type.attributes[1].value_type, "boolean");
}

#[test]
fn ocel_attribute_builders_round_trip() {
    let s = OcelAttribute::string("actor", "alice");
    assert_eq!(s.key, "actor");
    assert!(
        matches!(&s.value, V::String(x) if x == "alice"),
        "string builder → String variant"
    );

    let i = OcelAttribute::integer("count", 3);
    assert_eq!(i.key, "count");
    assert!(
        matches!(i.value, V::Integer(3)),
        "integer builder → Integer variant"
    );

    let b = OcelAttribute::boolean("active", true);
    assert_eq!(b.key, "active");
    assert!(
        matches!(b.value, V::Boolean(true)),
        "boolean builder → Boolean variant"
    );
}
