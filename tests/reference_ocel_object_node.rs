// Reference witness: the full OCEL-2.0 OCELObject node — id, type, attribute
// values, and inter-object relationships (COVERAGE.md §2 — OCEL-2.0 object node).
//
// Complements the OCELType schema layer (declaration) with the OCELObject
// instance layer: an object carries its type, concrete attribute values, and
// relationships to other objects.

use wasm4pm_compat::ocel::{OCELEventAttribute, OCELObject, OCELRelationship};

#[test]
fn ocel_object_carries_type_attributes_and_relationships() {
    let mut obj = OCELObject::new("o1".to_string(), "order")
        .with_attribute(OCELEventAttribute::string("status", "open".to_string()))
        .with_attribute(OCELEventAttribute::integer("priority", 3));
    // Inter-object relationship: this order relates to item "i1".
    obj.relationships
        .push(OCELRelationship::new("o1".to_string(), "i1".to_string()));

    assert_eq!(obj.id, "o1");
    assert_eq!(obj.object_type, "order");
    assert_eq!(obj.attributes.len(), 2, "two instance attribute values");
    assert_eq!(obj.attributes[0].name, "status");
    assert_eq!(obj.relationships.len(), 1, "one inter-object relationship");
    assert_eq!(obj.relationships[0].object_id, "i1");
}

#[test]
fn bare_object_has_no_attributes_or_relationships() {
    let bare = OCELObject::new("o2".to_string(), "item");
    assert!(bare.attributes.is_empty());
    assert!(
        bare.relationships.is_empty(),
        "no relationships unless added (not fabricated)"
    );
}
