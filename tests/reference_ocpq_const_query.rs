// Reference witness: the const-generic scoped OCPQ query OcpqQueryConst<const
// KIND: OcpqScopeKind> — the query scope kind (Open/Closed/SingleType) encoded in
// the type (COVERAGE.md §2 — const-generic OCPQ scope).
//
// Unlike the value-level OcpqQuery (runtime scope), OcpqQueryConst<KIND> carries
// its scope kind as a const parameter: an Open-scoped query and a SingleType-scoped
// query are different types. This witnesses construction under distinct scope kinds.

use wasm4pm_compat::ocpq::{ObjectScopeConst, OcpqQueryConst, OcpqScopeKind};

#[test]
fn const_scoped_queries_are_distinct_by_scope_kind() {
    let open: OcpqQueryConst<{ OcpqScopeKind::Open }> =
        OcpqQueryConst::new(ObjectScopeConst::new(["order", "item"]));
    // The scope is reachable and typed at the Open kind.
    let _ = open.scope();

    let single: OcpqQueryConst<{ OcpqScopeKind::SingleType }> =
        OcpqQueryConst::new(ObjectScopeConst::new(["order"]));
    let _ = single.scope();

    let closed: OcpqQueryConst<{ OcpqScopeKind::Closed }> =
        OcpqQueryConst::new(ObjectScopeConst::new(["order", "item", "shipment"]));
    let _ = closed.scope();

    // These are three DISTINCT types (scope kind is a const parameter) — a query's
    // scope discipline is known at compile time, not just at runtime. This file
    // compiling across all three kinds is the witness.
}
