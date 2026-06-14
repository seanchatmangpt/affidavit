// Reference witness: van der Aalst's four process-mining PERSPECTIVES and the
// quality-metric vocabulary are constructed as exhaustive censuses
// (COVERAGE.md §2 — multiperspective + quality vocabulary).
//
// `ProcessPerspective` is the canonical four-perspective decomposition of
// process mining (control-flow, data, resource, time). `QualityMetricKind` is
// the conformance-metric vocabulary. Both are covered as constructions, with
// no-wildcard matches making them compile-time censuses.

use wasm4pm_compat::multiperspective::ProcessPerspective;
use wasm4pm_compat::law::QualityMetricKind;

#[test]
fn the_four_process_perspectives_are_constructed() {
    let all = [
        ProcessPerspective::ControlFlow,
        ProcessPerspective::Data,
        ProcessPerspective::Resource,
        ProcessPerspective::Time,
    ];
    // Exhaustive census: a missing/ghost perspective won't compile.
    fn label(p: ProcessPerspective) -> &'static str {
        match p {
            ProcessPerspective::ControlFlow => "control-flow",
            ProcessPerspective::Data => "data",
            ProcessPerspective::Resource => "resource",
            ProcessPerspective::Time => "time",
        }
    }
    let labels: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(labels.len(), 4, "all four perspectives are distinct constructions");
}

#[test]
fn the_quality_metric_vocabulary_is_constructed() {
    let all = [
        QualityMetricKind::Fitness,
        QualityMetricKind::Precision,
        QualityMetricKind::F1,
        QualityMetricKind::Generalization,
        QualityMetricKind::Simplicity,
    ];
    fn label(k: QualityMetricKind) -> &'static str {
        match k {
            QualityMetricKind::Fitness => "fitness",
            QualityMetricKind::Precision => "precision",
            QualityMetricKind::F1 => "f1",
            QualityMetricKind::Generalization => "generalization",
            QualityMetricKind::Simplicity => "simplicity",
        }
    }
    let labels: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(labels.len(), 5, "all five quality-metric kinds are distinct constructions");
}
