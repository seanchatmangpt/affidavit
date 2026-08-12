// Reference witness: the WITNESS FAMILY taxonomy (TY-5) — concrete witness
// markers carry a stable KEY, a FAMILY classification, a TITLE, and a YEAR, all
// at the type level (COVERAGE.md §2 — witness markers + family gating).
//
// `Witness` markers are how `Admission<T, W>` and `Evidence<_, _, W>` are made
// family-typed. v26.8.7 intentionally exposes marker metadata through the
// `Witness` trait, so the trait import is part of the API witness rather than an
// incidental implementation detail.

use wasm4pm_compat::witness::{Ocel20, Pm4pyApiGrammar, PowlPaper, WitnessFamily};
use wasm4pm_compat::Witness;

#[test]
fn standard_family_markers_carry_correct_metadata() {
    assert_eq!(Ocel20::KEY, "ocel-2.0");
    assert_eq!(
        Ocel20::FAMILY,
        WitnessFamily::Standard,
        "OCEL 2.0 is a Standard"
    );
    assert_eq!(Ocel20::TITLE, "OCEL 2.0");
    assert_eq!(Ocel20::YEAR, Some(2023));
}

#[test]
fn family_classification_distinguishes_marker_kinds() {
    // Paper-family vs ApiGrammar-family vs Standard-family are distinct.
    assert_eq!(PowlPaper::FAMILY, WitnessFamily::Paper, "POWL is a Paper");
    assert_eq!(
        Pm4pyApiGrammar::FAMILY,
        WitnessFamily::ApiGrammar,
        "pm4py is an ApiGrammar"
    );
    assert_ne!(Ocel20::FAMILY, PowlPaper::FAMILY, "Standard != Paper");
    assert_ne!(
        PowlPaper::FAMILY,
        Pm4pyApiGrammar::FAMILY,
        "Paper != ApiGrammar"
    );

    // Keys are distinct — markers are not interchangeable.
    assert_ne!(Ocel20::KEY, PowlPaper::KEY);
}
