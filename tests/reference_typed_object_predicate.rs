// Reference witness: TypedObjectPredicate<const KIND: ObjectPredicateKind> — the
// type-level kind-separation law for object predicates (COVERAGE.md §2 — typed
// OCPQ predicate kinds).
//
// Each ObjectPredicateKind variant produces a DISTINCT type
// `TypedObjectPredicate<{ KIND }>`. The kind() accessor returns the const-param
// kind, so an attribute-equals predicate cannot be substituted for a type-equals
// one at the type level. This witnesses the positive side: every variant
// constructs, expression() reads the stored string back, and kind() returns the
// matching const-param kind. Failing-when-fake: if the expression/kind plumbing
// is wrong, these assertions fail rather than silently passing.

use wasm4pm_compat::ocpq::{ObjectPredicateKind, TypedObjectPredicate};

#[test]
fn attribute_equals_witnesses_expression_and_kind() {
    let p = TypedObjectPredicate::<{ ObjectPredicateKind::AttributeEquals }>::new("amount > 0");
    assert_eq!(p.expression(), "amount > 0");
    assert_eq!(p.kind(), ObjectPredicateKind::AttributeEquals);
}

#[test]
fn type_equals_witnesses_expression_and_kind() {
    let p = TypedObjectPredicate::<{ ObjectPredicateKind::TypeEquals }>::new("order");
    assert_eq!(p.expression(), "order");
    assert_eq!(p.kind(), ObjectPredicateKind::TypeEquals);
}
