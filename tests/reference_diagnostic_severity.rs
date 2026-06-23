// Reference witness: the CompatDiagnostic severity CLASSIFICATION — every
// boundary DEFECT is an Error; the only advisory (MigrationRecommended) is Info
// (COVERAGE.md §2.5 — diagnostic severity law; complements the Display witness).
//
// The law: a diagnostic that names a defect (missing witness, fiat admission,
// hidden flattening, …) is Error; "migration recommended" is advisory → Info.
// This witnesses the split via the Display "[severity] …" prefix.

use wasm4pm_compat::diagnostic::CompatDiagnostic as D;

#[test]
fn defects_are_errors_advice_is_info() {
    let errors = [
        D::MissingWitness,
        D::MissingRoundTripFixture,
        D::RawEvidenceExportedAsAdmitted,
        D::LossyProjectionWithoutPolicy,
        D::HiddenFlattening,
        D::MissingRefusalPath,
        D::MissingReceiptShape,
        D::UnreachablePrimitive,
    ];
    for d in errors {
        assert!(
            d.to_string().starts_with("[Error]"),
            "defect diagnostic must be an Error: {d:?} → {}",
            d
        );
    }

    // The single advisory is Info, not Error — it is not a defect, just guidance.
    assert!(
        D::MigrationRecommended.to_string().starts_with("[Info]"),
        "migration advice is Info, not Error"
    );
}

#[test]
fn severity_counts_are_eight_errors_one_info() {
    let all = [
        D::MissingWitness,
        D::MissingRoundTripFixture,
        D::RawEvidenceExportedAsAdmitted,
        D::LossyProjectionWithoutPolicy,
        D::HiddenFlattening,
        D::MissingRefusalPath,
        D::MissingReceiptShape,
        D::UnreachablePrimitive,
        D::MigrationRecommended,
    ];
    let errors = all
        .iter()
        .filter(|d| d.to_string().starts_with("[Error]"))
        .count();
    let infos = all
        .iter()
        .filter(|d| d.to_string().starts_with("[Info]"))
        .count();
    assert_eq!(errors, 8, "eight defect diagnostics");
    assert_eq!(infos, 1, "one advisory");
}
