// Reference witness: the OBJECT-CENTRIC DECLARE law (OcDeclareConstraint) — the
// 17th refusal enum (OcDeclareRefusal), all 3 variants fired against real
// violations, completing the refusal-enum surface (COVERAGE.md §2.6).
//
// OcDeclareConstraint extends a DeclareConstraint with object-type annotations
// and an optional synchronized flag. Its validate() enforces:
//   - EmptyObjectTypeList — at least one object type required
//   - SynchronizationRequiresMultipleTypes — synchronized needs >= 2 types
//   - ScopeMismatch — the synchronized flag must agree with the scope kind

use wasm4pm_compat::declare::{
    Activity, DeclareConstraint, DeclareScope, DeclareTemplate, OcDeclareConstraint,
    OcDeclareRefusal,
};

fn base(scope: DeclareScope) -> DeclareConstraint {
    DeclareConstraint::unary(DeclareTemplate::Existence, Activity::new("create"), scope)
}

#[test]
fn oc_declare_refuses_empty_object_type_list() {
    let c = OcDeclareConstraint::new(
        base(DeclareScope::SingleObjectScope("order".into())),
        Vec::<String>::new(),
    );
    assert_eq!(c.validate(), Err(OcDeclareRefusal::EmptyObjectTypeList));
}

#[test]
fn oc_declare_refuses_synchronization_with_single_type() {
    // synchronized but only one object type → SynchronizationRequiresMultipleTypes.
    let c = OcDeclareConstraint::synchronized(
        base(DeclareScope::SynchronizedObjectScope(vec![
            "order".into(),
            "item".into(),
        ])),
        vec!["order".into()], // only one type
    );
    assert_eq!(
        c.validate(),
        Err(OcDeclareRefusal::SynchronizationRequiresMultipleTypes)
    );
}

#[test]
fn oc_declare_refuses_scope_mismatch() {
    // synchronized=false but the scope IS a SynchronizedObjectScope → ScopeMismatch.
    let c = OcDeclareConstraint::new(
        base(DeclareScope::SynchronizedObjectScope(vec![
            "order".into(),
            "item".into(),
        ])),
        vec!["order".into(), "item".into()],
    );
    assert_eq!(c.validate(), Err(OcDeclareRefusal::ScopeMismatch));
}

#[test]
fn oc_declare_admits_well_formed_constraint() {
    let c = OcDeclareConstraint::new(
        base(DeclareScope::SingleObjectScope("order".into())),
        vec!["order".into()],
    );
    assert_eq!(
        c.validate(),
        Ok(()),
        "non-empty types, non-sync, single scope → admits"
    );
    assert!(!c.is_synchronized());
}
