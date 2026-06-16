// Reference witness: PetriRefusal::ObjectTypeNotPreserved — an object-centric
// Petri net with an arc referencing an undeclared object type is refused
// (COVERAGE.md §2.6 — completes PetriRefusal's REACHABLE set: 4 of 4).
//
// ObjectCentricPetriNet::validate checks every arc's object_type against the
// declared object_types set; an arc tagged with a type not in the set violates
// object-type preservation.

use wasm4pm_compat::petri::{
    Arc, Marking, ObjectCentricPetriNet, PetriNet, PetriRefusal, Place, Transition,
};

fn net_with_object_typed_arc(arc_object_type: Option<(String, bool)>) -> PetriNet {
    let mut arc = Arc::place_to_transition("p0", "t0");
    arc.object_type = arc_object_type;
    PetriNet::new(
        [Place::new("p0")],
        [Transition::new("t0", "fire")],
        [arc],
        Marking::new([("p0".to_string(), 1)]),
    )
}

#[test]
fn arc_with_undeclared_object_type_is_refused() {
    // The arc is tagged with object type "order", but the OCPN declares none.
    let net = net_with_object_typed_arc(Some(("order".to_string(), false)));
    let ocpn = ObjectCentricPetriNet::new(net, Vec::<String>::new());
    assert_eq!(
        ocpn.validate(),
        Err(PetriRefusal::ObjectTypeNotPreserved),
        "an arc's object type must be in the declared set"
    );
}

#[test]
fn arc_with_declared_object_type_admits() {
    let net = net_with_object_typed_arc(Some(("order".to_string(), false)));
    let ocpn = ObjectCentricPetriNet::new(net, vec!["order".to_string()]);
    assert_eq!(ocpn.validate(), Ok(()), "declared object type → preserved");

    // An untyped arc (object_type None) also preserves trivially.
    let untyped = ObjectCentricPetriNet::new(net_with_object_typed_arc(None), Vec::<String>::new());
    assert_eq!(
        untyped.validate(),
        Ok(()),
        "no object-typed arcs → trivially preserved"
    );
}
