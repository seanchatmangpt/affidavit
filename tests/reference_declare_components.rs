// Reference witness: the DECLARE constraint components — Activity and the
// DeclareConstraint fields (template/activation/target/scope), for unary and
// binary constructions (COVERAGE.md §2 — DECLARE constraint structure).
//
// Complements the DeclareRefusal validate witnesses: here we read the constructed
// structure back — unary constraints have no target; binary ones carry one; the
// activation/scope are recoverable.

use wasm4pm_compat::declare::{Activity, DeclareConstraint, DeclareScope, DeclareTemplate};

#[test]
fn activity_carries_its_name() {
    let a = Activity::new("place_order");
    assert_eq!(a.name(), "place_order");
    assert_eq!(a.0, "place_order", "tuple field accessible too");
}

#[test]
fn unary_constraint_has_no_target() {
    let c = DeclareConstraint::unary(
        DeclareTemplate::Existence,
        Activity::new("create"),
        DeclareScope::SingleObjectScope("order".into()),
    );
    assert_eq!(c.template, DeclareTemplate::Existence);
    assert_eq!(c.activation.name(), "create");
    assert!(c.target.is_none(), "a unary constraint carries no target");
    assert!(matches!(c.scope, DeclareScope::SingleObjectScope(_)));
}

#[test]
fn binary_constraint_carries_a_target() {
    let c = DeclareConstraint::binary(
        DeclareTemplate::Response,
        Activity::new("create"),
        Activity::new("release"),
        DeclareScope::SingleObjectScope("order".into()),
    );
    assert_eq!(c.template, DeclareTemplate::Response);
    assert_eq!(c.activation.name(), "create");
    assert_eq!(c.target.as_ref().map(|t| t.name()), Some("release"), "binary carries its target");
}
