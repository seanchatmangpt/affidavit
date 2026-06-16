// Reference witness: the OCPQ predicate vocabulary (Predicate / PredicateKind) —
// the object-centric process-query building blocks (COVERAGE.md §2 — OCPQ
// predicates; positive surface, since OcpqRefusal is a ghost cluster §7).
//
// PredicateKind enumerates the OCPQ Section-4 typed predicates: simple
// (Event/Object/Relation/Temporal), counting (Cardinality/Nested/ChildSetBound),
// and relational (E2ORelation/O2ORelation/TimeBetweenEvents). This constructs a
// Predicate for representatives across all three families and reads the kind back.

use wasm4pm_compat::ocpq::{Predicate, PredicateKind};

#[test]
fn predicates_construct_across_the_kind_families() {
    let event = Predicate::<()>::new(PredicateKind::Event("place_order".into()));
    assert!(
        matches!(event.kind, PredicateKind::Event(_)),
        "simple event predicate"
    );

    let card = Predicate::<()>::new(PredicateKind::Cardinality { min: 1, max: 5 });
    match card.kind {
        PredicateKind::Cardinality { min, max } => assert_eq!((min, max), (1, 5)),
        other => panic!("expected Cardinality; got {other:?}"),
    }

    let e2o = Predicate::<()>::new(PredicateKind::E2ORelation {
        event_var: "e".into(),
        object_var: "o".into(),
        qualifier: Some("input".into()),
    });
    match e2o.kind {
        PredicateKind::E2ORelation { qualifier, .. } => {
            assert_eq!(qualifier.as_deref(), Some("input"))
        }
        other => panic!("expected E2ORelation; got {other:?}"),
    }

    let csb = Predicate::<()>::new(PredicateKind::ChildSetBound {
        branch_label: "items".into(),
        min: 1,
        max: 10,
    });
    assert!(
        matches!(csb.kind, PredicateKind::ChildSetBound { .. }),
        "child-set-bound predicate"
    );
}

#[test]
fn simple_predicate_kinds_carry_their_expression() {
    for (k, label) in [
        (PredicateKind::Object("order".into()), "order"),
        (PredicateKind::Relation("contains".into()), "contains"),
        (PredicateKind::Temporal("before".into()), "before"),
    ] {
        let p = Predicate::<()>::new(k);
        let carried = match &p.kind {
            PredicateKind::Object(s) | PredicateKind::Relation(s) | PredicateKind::Temporal(s) => {
                s.as_str()
            }
            other => panic!("unexpected kind {other:?}"),
        };
        assert_eq!(carried, label);
    }
}
