// Reference witness: the CompatDiagnostic surface — each diagnostic renders as
// "[severity] message" via Display (COVERAGE.md §2.5 — closes the HiddenFlattening
// row, the convergence/divergence silent-flatten diagnostic).
//
// CompatDiagnostic enumerates the compat-boundary defects (missing witness, raw
// exported as admitted, lossy projection without policy, HIDDEN FLATTENING, etc.).
// This witnesses each renders its severity + guidance message, with HiddenFlattening
// pointing at the LossReport remedy — the silent-flatten counterpart to the named
// LossReport path already witnessed.

use wasm4pm_compat::diagnostic::CompatDiagnostic as D;

#[test]
fn hidden_flattening_diagnostic_points_at_the_lossreport_remedy() {
    let s = D::HiddenFlattening.to_string();
    assert!(s.starts_with("[Error]"), "hidden flattening is an Error; got {s}");
    assert!(s.contains("hidden flattening"), "names the defect");
    assert!(s.contains("LossReport"), "points at the named-loss remedy; got {s}");
}

#[test]
fn compat_diagnostics_render_severity_and_message() {
    // A spread across the diagnostic taxonomy, each rendering "[severity] message".
    for d in [
        D::MissingWitness,
        D::MissingRoundTripFixture,
        D::RawEvidenceExportedAsAdmitted,
        D::LossyProjectionWithoutPolicy,
        D::HiddenFlattening,
        D::MissingRefusalPath,
        D::MissingReceiptShape,
        D::UnreachablePrimitive,
        D::MigrationRecommended,
    ] {
        let s = d.to_string();
        assert!(
            s.starts_with("[Error]") || s.starts_with("[Warning]") || s.starts_with("[Info]"),
            "every diagnostic renders a severity tag; got {s}"
        );
        assert!(s.len() > 10, "carries a guidance message, not just a tag: {s}");
    }
    // RawEvidenceExportedAsAdmitted is the T-1 (fiat-admission) hazard, named.
    assert!(D::RawEvidenceExportedAsAdmitted.to_string().to_lowercase().contains("admitted"));
}
