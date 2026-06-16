// Reference witness: the process-artifact shape taxonomy, conformance/receipt
// verdicts, and BPMN gateway kinds are constructed as exhaustive censuses
// (COVERAGE.md §2 — shape taxonomy + verdict vocabulary).
//
//   • ProcessShapeKind  — the 22-shape taxonomy of every process artifact the
//     court models (logs, nets, trees, POWL, DECLARE, OCPQ, alignment, …).
//   • ConformanceVerdict — token-replay verdict vocabulary.
//   • ReceiptVerdict     — the admitted/refused receipt outcome.
//   • BpmnGateway        — the BPMN gateway kinds.

use wasm4pm_compat::bpmn::BpmnGateway;
use wasm4pm_compat::law::ProcessShapeKind as Shape;
use wasm4pm_compat::receipt::{ConformanceVerdict, ReceiptRefusal, ReceiptVerdict};

#[test]
fn the_process_shape_taxonomy_is_constructed() {
    let all = [
        Shape::Event,
        Shape::Trace,
        Shape::EventLog,
        Shape::EventStream,
        Shape::XesLog,
        Shape::OcelLog,
        Shape::DirectlyFollowsGraph,
        Shape::ObjectCentricDfg,
        Shape::PetriNet,
        Shape::WorkflowNet,
        Shape::ObjectCentricPetriNet,
        Shape::ProcessTree,
        Shape::Powl,
        Shape::DeclareModel,
        Shape::ObjectCentricDeclareModel,
        Shape::LogSkeleton,
        Shape::OcpqQuery,
        Shape::Alignment,
        Shape::TokenReplay,
        Shape::ConformanceVerdict,
        Shape::PredictionProblem,
        Shape::Receipt,
    ];
    assert_eq!(
        all.len(),
        22,
        "the taxonomy enumerates 22 process-artifact shapes"
    );
    // Distinctness via Debug (each is a materialised, distinct shape label).
    let s: std::collections::BTreeSet<String> = all.iter().map(|x| format!("{x:?}")).collect();
    assert_eq!(s.len(), 22, "all 22 shapes are distinct constructions");

    // COMPILE-ENFORCED exhaustiveness (van der Aalst panel B1 fix): a no-wildcard
    // match makes the census total. If wasm4pm-compat adds a 23rd ProcessShapeKind,
    // THIS fails to compile — the hand-listed array above can drift silently, but
    // the match cannot. Every shape is classified by its modelling family, so the
    // arm is a real touch of the variant, not a throwaway `_ => ()`.
    fn family(s: Shape) -> &'static str {
        match s {
            Shape::Event
            | Shape::Trace
            | Shape::EventLog
            | Shape::EventStream
            | Shape::XesLog
            | Shape::OcelLog => "log",
            Shape::DirectlyFollowsGraph
            | Shape::ObjectCentricDfg
            | Shape::PetriNet
            | Shape::WorkflowNet
            | Shape::ObjectCentricPetriNet
            | Shape::ProcessTree
            | Shape::Powl => "model",
            Shape::DeclareModel
            | Shape::ObjectCentricDeclareModel
            | Shape::LogSkeleton
            | Shape::OcpqQuery => "declarative/query",
            Shape::Alignment | Shape::TokenReplay | Shape::ConformanceVerdict => "conformance",
            Shape::PredictionProblem => "prediction",
            Shape::Receipt => "provenance",
        }
    }
    // Every listed shape resolves to a family — and the match's exhaustiveness is
    // the real guarantee (drift → compile error).
    assert!(all.iter().copied().all(|s| !family(s).is_empty()));
}

#[test]
fn conformance_and_receipt_verdicts_are_constructed() {
    // Conformance verdict vocabulary (token replay outcomes).
    let cv = [
        ConformanceVerdict::PerfectAlignment,
        ConformanceVerdict::FitnessDeficit,
        ConformanceVerdict::DeadlockEncountered,
    ];
    fn cv_label(v: ConformanceVerdict) -> &'static str {
        match v {
            ConformanceVerdict::PerfectAlignment => "perfect",
            ConformanceVerdict::FitnessDeficit => "deficit",
            ConformanceVerdict::DeadlockEncountered => "deadlock",
        }
    }
    let s: std::collections::BTreeSet<&str> = cv.iter().copied().map(cv_label).collect();
    assert_eq!(s.len(), 3, "three conformance verdicts");

    // Receipt verdict: the admitted/refused outcome (the court's binary verdict).
    // Refused carries the named refusal law — constructed with a real one.
    fn rv_label(v: &ReceiptVerdict) -> &'static str {
        match v {
            ReceiptVerdict::Admitted => "admitted",
            ReceiptVerdict::Refused(_) => "refused",
        }
    }
    let admitted = ReceiptVerdict::Admitted;
    let refused = ReceiptVerdict::Refused(ReceiptRefusal::EmptyChain);
    assert_ne!(rv_label(&admitted), rv_label(&refused));
    assert_eq!(rv_label(&refused), "refused");
}

#[test]
fn bpmn_gateway_kinds_are_constructed() {
    let all = [
        BpmnGateway::Exclusive,
        BpmnGateway::Parallel,
        BpmnGateway::Inclusive,
        BpmnGateway::EventBased,
        BpmnGateway::Complex,
    ];
    fn ok(g: BpmnGateway) -> bool {
        matches!(
            g,
            BpmnGateway::Exclusive
                | BpmnGateway::Parallel
                | BpmnGateway::Inclusive
                | BpmnGateway::EventBased
                | BpmnGateway::Complex
        )
    }
    assert!(all.iter().copied().all(ok));
    assert_eq!(all.len(), 5, "five BPMN gateway kinds");
}
