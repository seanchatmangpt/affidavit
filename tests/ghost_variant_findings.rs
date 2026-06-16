// REFERENCE FINDING — ghost-variant clusters in wasm4pm-compat (v32 TY-9).
//
// A reference implementation's central failure mode is a type present in the API
// but absent from any demonstration. Its DUAL — the court's own central failure —
// is a refusal variant NAMED but UNREACHABLE: a law no code path can produce.
//
// By attempting to witness every refusal law against a real violation, this
// reference DETECTED such clusters in the dependency. The following enums are
// exported, Debug/Display-able, and constructible as values — but a whole-crate
// search (`grep -rE "Err(<Enum>::|.ok_or(<Enum>::"` over `wasm4pm-compat/src`)
// finds ZERO producing code paths:
//
//   • OcpqRefusal        — 10 variants, 0 producers
//   • ConformanceRefusal — 8 variants, 0 producers
//   • PredictionRefusal  — 6 variants, 0 producers
//
// These are GHOST variants: named laws that cannot fire. This is recorded as a
// finding (reference/COVERAGE.md §7), NOT marked 🟢 — witnessing them against a
// violation is impossible because no violation reaches them. Marking them
// "covered" would be the exact certification-without-work the discipline forbids.
//
// This test pins the finding: it proves the variants are well-formed VALUES
// (materialisable, distinct, Display-stable) while the COVERAGE doc records that
// they are unreachable AS LAWS. The distinction — value-reachable vs
// law-reachable — is the finding.

use wasm4pm_compat::conformance::ConformanceRefusal;
use wasm4pm_compat::prediction::PredictionRefusal;

#[test]
fn conformance_refusal_variants_are_values_but_not_laws() {
    // Constructible and distinct as values...
    let variants = [
        ConformanceRefusal::MissingLog,
        ConformanceRefusal::MissingModel,
        ConformanceRefusal::MissingDeviationPath,
        ConformanceRefusal::FitnessUnavailable,
        ConformanceRefusal::PrecisionUnavailable,
        ConformanceRefusal::F1Unavailable,
        ConformanceRefusal::GeneralizationUnavailable,
        ConformanceRefusal::SimplicityUnavailable,
    ];
    // Display is stable (they are well-formed named values).
    assert_eq!(ConformanceRefusal::MissingLog.to_string(), "MissingLog");
    // Distinct as values.
    let set: std::collections::BTreeSet<String> = variants.iter().map(|v| v.to_string()).collect();
    assert_eq!(set.len(), 8, "all 8 variants are distinct values");
    // ...BUT the FINDING is that no wasm4pm-compat code path returns any of them.
    // That is a whole-crate property recorded in COVERAGE.md §7 with grep
    // evidence; it cannot be asserted from inside this crate, and these variants
    // are therefore NOT counted among the witnessed refusal laws.
}

#[test]
fn prediction_refusal_variants_are_values_but_not_laws() {
    let variants = [
        PredictionRefusal::MissingPrefix,
        PredictionRefusal::MissingTarget,
        PredictionRefusal::EmptyPrefix,
        PredictionRefusal::TargetUnsupported,
        PredictionRefusal::NonPrefixTrace,
        PredictionRefusal::ConstraintNotNamed,
    ];
    let set: std::collections::BTreeSet<String> =
        variants.iter().map(|v| format!("{v:?}")).collect();
    assert_eq!(
        set.len(),
        6,
        "all 6 variants are distinct values (but unreachable as laws)"
    );
}
