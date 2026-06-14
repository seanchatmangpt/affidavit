// Reference witness: the OCPQ query container (OcpqQuery + ObjectScope) — an
// object-centric process query binds object types and carries predicates
// (COVERAGE.md §2 — OCPQ query surface; positive, OcpqRefusal is a ghost §7).
//
// OcpqQuery binds an ObjectScope (the object types it queries over) and a list of
// Predicates (the query body), with optional nested sub-queries. This witnesses
// scope construction, query construction, predicate/sub-query attachment, and
// readback.

use wasm4pm_compat::ocpq::{ObjectScope, OcpqQuery, Predicate, PredicateKind};

#[test]
fn object_scope_binds_types() {
    let scope = ObjectScope::new(["order", "item"]);
    assert_eq!(scope.object_types, vec!["order".to_string(), "item".to_string()]);
    assert!(!scope.is_empty(), "a non-empty scope");
    assert!(ObjectScope::new(Vec::<String>::new()).is_empty(), "empty scope is empty");
}

#[test]
fn query_carries_scope_predicates_and_subqueries() {
    let mut q = OcpqQuery::new(ObjectScope::new(["order"]));
    assert_eq!(q.scope.object_types, vec!["order".to_string()]);
    assert!(q.predicates.is_empty(), "fresh query has no predicates");

    // Attach a predicate body and a nested sub-query.
    q.predicates.push(Predicate::new(PredicateKind::Event("place_order".into())));
    q.predicates.push(Predicate::new(PredicateKind::Cardinality { min: 1, max: 3 }));
    q.sub_queries.push(OcpqQuery::new(ObjectScope::new(["item"])));

    assert_eq!(q.predicates.len(), 2, "two predicates in the query body");
    assert_eq!(q.sub_queries.len(), 1, "one nested sub-query");
    assert_eq!(q.sub_queries[0].scope.object_types, vec!["item".to_string()]);
}
