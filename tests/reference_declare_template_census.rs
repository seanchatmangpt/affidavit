// Reference witness: the full DeclareTemplate vocabulary census (COVERAGE.md §2 —
// DECLARE template completeness). Previously only Existence + Response were
// witnessed; this is the exhaustive 22-variant census.
//
// DeclareTemplate is the ConDec constraint vocabulary (van der Aalst/Pesic):
// unary existence/absence, binary ordering (response/precedence/succession and
// their alternate/chain strengthenings), and negative templates. This witnesses
// EVERY variant via a no-wildcard match (a 23rd template would break compilation —
// the census cannot silently under-count) and asserts each lands in the correct
// taxonomic bucket via arity()/is_negative()/is_chain().

use std::collections::HashSet;
use wasm4pm_compat::declare::DeclareTemplate::{self, *};

// Every variant, listed once. The exhaustive match below makes this list complete
// at compile time.
const ALL: [DeclareTemplate; 22] = [
    Existence, Absence, Init, Existence2, Existence3, Absence2, Absence3,
    RespondedExistence, CoExistence, Response, Precedence, Succession,
    AlternateResponse, AlternatePrecedence, AlternateSuccession,
    ChainResponse, ChainPrecedence, ChainSuccession,
    NotSuccession, NotChainSuccession, NotCoExistence, ExclusiveChoice,
];

#[test]
fn every_declare_template_has_a_lawful_arity_and_taxonomy() {
    for t in ALL {
        // No-wildcard match: a new variant added upstream fails to compile here.
        let (expected_arity, unary) = match t {
            Existence | Absence | Init | Existence2 | Existence3 | Absence2 | Absence3 => (1, true),
            RespondedExistence | CoExistence | Response | Precedence | Succession
            | AlternateResponse | AlternatePrecedence | AlternateSuccession
            | ChainResponse | ChainPrecedence | ChainSuccession
            | NotSuccession | NotChainSuccession | NotCoExistence | ExclusiveChoice => (2, false),
        };
        assert_eq!(t.arity(), expected_arity, "{t:?} arity");
        assert_eq!(t.arity() == 1, unary, "{t:?} unary/binary classification");
    }
}

#[test]
fn negative_and_chain_taxonomies_match_the_library_contract() {
    // is_negative is the FORBIDDING family: the Absence counts + the Not-* templates.
    // (ExclusiveChoice is NOT negative — a guess to the contrary is caught here.)
    for t in [Absence, Absence2, Absence3, NotCoExistence, NotSuccession, NotChainSuccession] {
        assert!(t.is_negative(), "{t:?} is a negative (forbidding) template");
    }
    for t in [Existence, Response, Succession, ExclusiveChoice] {
        assert!(!t.is_negative(), "{t:?} is NOT negative per the library");
    }

    // Chain templates (immediate succession): the three Chain* + NotChainSuccession.
    for t in [ChainResponse, ChainPrecedence, ChainSuccession, NotChainSuccession] {
        assert!(t.is_chain(), "{t:?} is a chain template");
    }
    assert!(!Response.is_chain(), "Response is not a chain template");
}

#[test]
fn all_22_templates_are_distinct() {
    let set: HashSet<_> = ALL.iter().collect();
    assert_eq!(set.len(), 22, "no two templates collapse to the same value");
}
