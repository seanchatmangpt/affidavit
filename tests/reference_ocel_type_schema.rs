// Reference witness: the OCEL-2.0 TYPE-DECLARATION schema layer — OCELType and
// OCELTypeAttribute (COVERAGE.md §2 — OCEL-2.0 type schema).
//
// OCEL 2.0 declares object/event TYPES with typed attribute schemas (name +
// value-type), distinct from the per-instance attribute values. This witnesses
// constructing a type declaration with its attribute schema and reading it back.

use wasm4pm_compat::ocel::{OCELType, OCELTypeAttribute};

#[test]
fn ocel_type_declares_attribute_schema() {
    let order_type = OCELType {
        name: "order".to_string(),
        attributes: vec![
            OCELTypeAttribute { name: "total".to_string(), value_type: "float".to_string() },
            OCELTypeAttribute { name: "priority".to_string(), value_type: "boolean".to_string() },
        ],
    };

    assert_eq!(order_type.name, "order", "the declared object type");
    assert_eq!(order_type.attributes.len(), 2, "two attribute declarations");
    assert_eq!(order_type.attributes[0].name, "total");
    assert_eq!(order_type.attributes[0].value_type, "float", "attribute's declared value type");
    assert_eq!(order_type.attributes[1].value_type, "boolean");
}

#[test]
fn type_with_no_attributes_is_valid() {
    let bare = OCELType { name: "tag".to_string(), attributes: vec![] };
    assert_eq!(bare.name, "tag");
    assert!(bare.attributes.is_empty(), "a type may declare no attributes");
}
