// Reference witness: the type-level causal-structure and correlation carriers
// (COVERAGE.md §2 — causality + correlation carriers).
//
// These are zero-sized, type-level markers: `CausalLink<From,To>` is a typed
// causal edge between two object-type markers; `CausalChain<const LENGTH>` is a
// chain whose length is a const parameter; `CorrelationKey<const SCHEMA>` carries
// the correlation schema as a const string. The carriers expose their const
// parameters back (length(), schema()) — witnessed here.

use wasm4pm_compat::causality::{CausalChain, CausalLink};
use wasm4pm_compat::correlation::CorrelationKey;

// Distinct object-type markers for the causal edge.
enum Order {}
enum Item {}

#[test]
fn causal_link_and_chain_are_type_level_constructible() {
    // A typed causal edge Order → Item. Zero-sized; its meaning is its type.
    let _edge: CausalLink<Order, Item> = CausalLink::new();
    // A causal chain of compile-time length 3 reports its length const.
    let chain: CausalChain<3> = CausalChain::new();
    assert_eq!(chain.length(), 3, "chain length is the const parameter");
    let longer: CausalChain<7> = CausalChain::new();
    assert_eq!(longer.length(), 7);
    // Distinct lengths are distinct types (this only compiles because LENGTH is a
    // real const parameter, not erased).
}

#[test]
fn correlation_key_carries_its_schema_const() {
    let by_case: CorrelationKey<"by-case"> = CorrelationKey::new();
    assert_eq!(
        by_case.schema(),
        "by-case",
        "correlation key reports its schema const"
    );

    let by_object: CorrelationKey<"by-object"> = CorrelationKey::new();
    assert_eq!(by_object.schema(), "by-object");
    assert_ne!(
        by_case.schema(),
        by_object.schema(),
        "distinct schemas are distinct keys"
    );
}
