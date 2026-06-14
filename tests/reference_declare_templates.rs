// Reference witness: the full DECLARE constraint-template vocabulary is
// CONSTRUCTED and its arity exercised (COVERAGE.md §2 — DECLARE template census).
//
// van der Aalst's declarative process mining is built on the DECLARE template
// catalogue. wasm4pm-compat exports 22 templates. This census constructs every
// one and partitions them by arity via the crate's `arity()` law.
//
// Failing-when-fake on two axes:
//   1. The exhaustive `match` (no wildcard) is a compile-time census — a missing
//      template won't compile, a ghost template won't compile.
//   2. The arity partition is asserted against the crate's own `arity()`, so a
//      mis-grouped template fails.

use wasm4pm_compat::declare::DeclareTemplate as T;

/// Every exported template, constructed exactly once.
fn all_templates() -> Vec<T> {
    vec![
        T::Existence, T::Absence, T::Init, T::Existence2, T::Existence3,
        T::Absence2, T::Absence3,
        T::RespondedExistence, T::CoExistence, T::Response, T::Precedence,
        T::Succession, T::AlternateResponse, T::AlternatePrecedence,
        T::AlternateSuccession, T::ChainResponse, T::ChainPrecedence,
        T::ChainSuccession, T::NotSuccession, T::NotChainSuccession,
        T::NotCoExistence, T::ExclusiveChoice,
    ]
}

/// Exhaustive arity classification — the no-wildcard match makes this a
/// compile-time census of the template catalogue.
fn arity_label(t: T) -> &'static str {
    match t {
        T::Existence | T::Absence | T::Init | T::Existence2 | T::Existence3
        | T::Absence2 | T::Absence3 => "unary",
        T::RespondedExistence | T::CoExistence | T::Response | T::Precedence
        | T::Succession | T::AlternateResponse | T::AlternatePrecedence
        | T::AlternateSuccession | T::ChainResponse | T::ChainPrecedence
        | T::ChainSuccession | T::NotSuccession | T::NotChainSuccession
        | T::NotCoExistence | T::ExclusiveChoice => "binary",
    }
}

#[test]
fn all_twentytwo_declare_templates_are_constructed() {
    let all = all_templates();
    assert_eq!(all.len(), 22, "the crate exports 22 DECLARE templates");

    let unary = all.iter().copied().filter(|t| arity_label(*t) == "unary").count();
    let binary = all.iter().copied().filter(|t| arity_label(*t) == "binary").count();
    assert_eq!(unary, 7, "7 unary templates");
    assert_eq!(binary, 15, "15 binary templates");
}

#[test]
fn template_arity_matches_the_crates_own_arity_law() {
    // Cross-check our classification against the crate's `arity()` method — if
    // they disagree, one of them is wrong (failing-when-fake on the arity law).
    for t in all_templates() {
        let expected = if arity_label(t) == "unary" { 1 } else { 2 };
        assert_eq!(
            t.arity(),
            expected,
            "template {t:?} arity must match its classification"
        );
    }
}
