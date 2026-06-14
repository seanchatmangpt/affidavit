// Reference witness: the predictive-monitoring vocabulary and diagnostic
// severity are constructed as exhaustive censuses (COVERAGE.md §2 — prediction
// + diagnostic vocabulary).
//
//   • PredictionTarget — van der Aalst's predictive-monitoring TARGETS
//     (next-activity, outcome, remaining-time, drift, risk, compliance).
//   • PredictionHorizon — how far ahead a prediction reaches.
//   • ComplianceKind    — the compliance-checking modes.
//   • DiagnosticSeverity — the compat diagnostic severity levels.

use wasm4pm_compat::prediction::{ComplianceKind as CK, PredictionHorizon as PH, PredictionTarget as PT};
use wasm4pm_compat::diagnostic::DiagnosticSeverity as Sev;

#[test]
fn the_six_prediction_targets_are_constructed() {
    let all = [PT::NextActivity, PT::OutcomeLabel, PT::RemainingTime, PT::DriftSignal, PT::Risk, PT::ComplianceConstraint];
    // Route through the real Display impl and collect distinct names.
    let names: std::collections::BTreeSet<String> = all.iter().map(|t| t.to_string()).collect();
    assert_eq!(names.len(), 6, "six distinct predictive-monitoring target names");
    assert!(names.contains("next-activity"));
    assert!(names.contains("compliance-constraint"));
    // No-wildcard family partition: a new upstream variant breaks compilation.
    fn family(t: PT) -> &'static str {
        match t {
            PT::NextActivity | PT::OutcomeLabel => "classification",
            PT::RemainingTime => "regression",
            PT::DriftSignal => "monitoring",
            PT::Risk => "scoring",
            PT::ComplianceConstraint => "compliance",
        }
    }
    let families: std::collections::BTreeSet<&str> = all.iter().copied().map(family).collect();
    assert_eq!(families.len(), 5, "five prediction target families");
}

#[test]
fn prediction_horizons_are_constructed() {
    // Events/TimeUnits carry a count; FullCase is unit.
    let all = [PH::FullCase, PH::Events(5), PH::TimeUnits(10)];
    // No-wildcard match that EXTRACTS the payload magnitude.
    fn magnitude(h: &PH) -> usize {
        match h {
            PH::FullCase => 0,
            PH::Events(n) => *n,
            PH::TimeUnits(s) => *s as usize,
        }
    }
    let mags: Vec<usize> = all.iter().map(magnitude).collect();
    assert_eq!(mags, vec![0, 5, 10], "extracted horizon magnitudes");
    // Real Display strings (verified against the impl).
    let rendered: std::collections::BTreeSet<String> = all.iter().map(|h| h.to_string()).collect();
    assert_eq!(rendered.len(), 3, "three distinct horizon renderings");
    assert_eq!(PH::FullCase.to_string(), "full-case");
    assert_eq!(PH::Events(5).to_string(), "events(5)");
    assert_eq!(PH::TimeUnits(10).to_string(), "time(10s)");
}

#[test]
fn compliance_kinds_are_constructed() {
    let all = [CK::Monitoring, CK::Audit, CK::Certification];
    fn label(c: CK) -> &'static str {
        match c { CK::Monitoring => "monitoring", CK::Audit => "audit", CK::Certification => "certification" }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(s.len(), 3, "three compliance kinds");
}

#[test]
fn diagnostic_severities_are_constructed() {
    let all = [Sev::Error, Sev::Warning, Sev::Info];
    fn label(s: Sev) -> &'static str {
        match s { Sev::Error => "error", Sev::Warning => "warning", Sev::Info => "info" }
    }
    let set: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(set.len(), 3, "three diagnostic severities");
}
