// Reference witness: TypedRelationPredicate<const KIND: RelationPredicateKind> —
// the type-level kind-separation law for relation predicates (COVERAGE.md §2 —
// typed OCPQ predicate kinds).
//
// Each RelationPredicateKind variant (E2O, O2O, TimeBetweenEvents) produces a
// DISTINCT type `TypedRelationPredicate<{ KIND }>`. The kind() accessor returns
// the const-param kind, so an E2O link cannot be substituted for an O2O link at
// the type level. This witnesses the positive side: every variant constructs,
// expression() reads the stored string back, and kind() returns the matching
// const-param kind. Failing-when-fake: if the expression/kind plumbing is wrong,
// these assertions fail rather than silently passing.

use wasm4pm_compat::ocpq::{RelationPredicateKind, TypedRelationPredicate};

#[test]
fn e2o_witnesses_expression_and_kind() {
    let p = TypedRelationPredicate::<{ RelationPredicateKind::E2O }>::new("e1 -> o1 [order]");
    assert_eq!(p.expression(), "e1 -> o1 [order]");
    assert_eq!(p.kind(), RelationPredicateKind::E2O);
}

#[test]
fn o2o_witnesses_expression_and_kind() {
    let p = TypedRelationPredicate::<{ RelationPredicateKind::O2O }>::new("o1 -> o2");
    assert_eq!(p.expression(), "o1 -> o2");
    assert_eq!(p.kind(), RelationPredicateKind::O2O);
}

#[test]
fn time_between_events_witnesses_expression_and_kind() {
    let p =
        TypedRelationPredicate::<{ RelationPredicateKind::TimeBetweenEvents }>::new("TBE(e1,e2,0,3600000)");
    assert_eq!(p.expression(), "TBE(e1,e2,0,3600000)");
    assert_eq!(p.kind(), RelationPredicateKind::TimeBetweenEvents);
}
