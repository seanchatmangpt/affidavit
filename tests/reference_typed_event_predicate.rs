// Reference witness: TypedEventPredicate<const KIND: EventPredicateKind> — the
// type-level kind-separation law for event predicates (COVERAGE.md §2 — typed
// OCPQ predicate kinds).
//
// Each EventPredicateKind variant produces a DISTINCT type
// `TypedEventPredicate<{ KIND }>`. The kind() accessor returns the const-param
// kind, so a value of one kind cannot masquerade as another at the type level.
// This witnesses the positive side: every variant constructs, expression() reads
// the stored string back, and kind() returns the matching const-param kind.
// Failing-when-fake: if the expression/kind plumbing is wrong, these assertions
// fail rather than silently passing.

use wasm4pm_compat::ocpq::{EventPredicateKind, TypedEventPredicate};

#[test]
fn activity_equals_witnesses_expression_and_kind() {
    let p = TypedEventPredicate::<{ EventPredicateKind::ActivityEquals }>::new("approve");
    assert_eq!(p.expression(), "approve");
    assert_eq!(p.kind(), EventPredicateKind::ActivityEquals);
}

#[test]
fn attribute_equals_witnesses_expression_and_kind() {
    let p = TypedEventPredicate::<{ EventPredicateKind::AttributeEquals }>::new("cost = 10");
    assert_eq!(p.expression(), "cost = 10");
    assert_eq!(p.kind(), EventPredicateKind::AttributeEquals);
}

#[test]
fn timestamp_in_range_witnesses_expression_and_kind() {
    let p = TypedEventPredicate::<{ EventPredicateKind::TimestampInRange }>::new("[0, 3600000]");
    assert_eq!(p.expression(), "[0, 3600000]");
    assert_eq!(p.kind(), EventPredicateKind::TimestampInRange);
}
