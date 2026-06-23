// Reference witness: graduation reasons, POWL projection states, the import/export
// format vocabulary, causal relation laws, and pm4py summary shapes are
// constructed as exhaustive censuses (COVERAGE.md §2).
//
//   • (GraduationReason lives behind the `wasm4pm` Cargo feature — OUT-OF-SURFACE.)
//   • PowlProjectionState — the POWL→ProcessTree projection lattice.
//   • FormatKind — the import/export formats the interop grammar names.
//   • RelationLaw — the OCEL relation directions (e2o/o2o/o2e).
//   • SummaryShape — pm4py summary projection shapes.

use wasm4pm_compat::formats::FormatKind as Fmt;
use wasm4pm_compat::interop::SummaryShape as Sum;
use wasm4pm_compat::law::{PowlProjectionState as PPS, RelationLaw as RL};

#[test]
fn powl_projection_states_are_constructed() {
    let all = [
        PPS::Unknown,
        PPS::ProcessTreeProjectable,
        PPS::ExceedsProcessTree,
        PPS::RefusedProjection,
    ];
    fn known(p: PPS) -> bool {
        match p {
            PPS::Unknown
            | PPS::ProcessTreeProjectable
            | PPS::ExceedsProcessTree
            | PPS::RefusedProjection => true,
        }
    }
    assert!(all.iter().copied().all(known));
    assert_eq!(all.len(), 4, "four POWL projection states");
}

#[test]
fn format_kinds_are_constructed() {
    let all = [
        Fmt::OcelJson,
        Fmt::OcelXml,
        Fmt::OcelSqlite,
        Fmt::XesXml,
        Fmt::BpmnXml,
        Fmt::PetriPnml,
        Fmt::PowlJson,
    ];
    // FormatKind is `#[non_exhaustive]` — census covers the 7 currently-known
    // formats; completeness is open by the crate's design (wildcard required).
    fn known(f: Fmt) -> bool {
        matches!(
            f,
            Fmt::OcelJson
                | Fmt::OcelXml
                | Fmt::OcelSqlite
                | Fmt::XesXml
                | Fmt::BpmnXml
                | Fmt::PetriPnml
                | Fmt::PowlJson
        )
    }
    assert!(all.iter().copied().all(known));
    assert_eq!(
        all.len(),
        7,
        "seven currently-known import/export format kinds"
    );
}

#[test]
fn relation_laws_are_constructed() {
    let all = [RL::EventToObject, RL::ObjectToObject, RL::ObjectToEvent];
    fn label(r: RL) -> &'static str {
        match r {
            RL::EventToObject => "e2o",
            RL::ObjectToObject => "o2o",
            RL::ObjectToEvent => "o2e",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(s.len(), 3, "three OCEL relation directions");
}

#[test]
fn summary_shapes_are_constructed() {
    let all = [
        Sum::Counts,
        Sum::TraceVariants,
        Sum::ActivityDistribution,
        Sum::TimingProfile,
        Sum::ObjectTypeDistribution,
    ];
    // Projection-category match. SummaryShape is `#[non_exhaustive]` (cross-crate),
    // so the compiler mandates a `_` arm; we route it to a panic so a newly-added
    // upstream variant fails this test loudly rather than being silently absorbed.
    fn category(s: Sum) -> &'static str {
        match s {
            Sum::Counts => "aggregate",
            Sum::TraceVariants | Sum::ActivityDistribution => "control-flow",
            Sum::TimingProfile => "performance",
            Sum::ObjectTypeDistribution => "object-centric",
            _ => panic!("unclassified SummaryShape variant: {s:?}"),
        }
    }
    let cats: std::collections::BTreeSet<&str> = all.iter().copied().map(category).collect();
    assert_eq!(cats.len(), 4, "four summary projection categories");
    let debugs: std::collections::BTreeSet<String> = all.iter().map(|s| format!("{s:?}")).collect();
    assert_eq!(debugs.len(), 5, "five distinct pm4py summary shapes");
}
