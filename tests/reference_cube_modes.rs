// Reference witness: process-cube dimensions, object-centricity axis, evidence
// lifecycle modes, and process-boundary claim kinds — exhaustive censuses
// (COVERAGE.md §2 — process-cube + lifecycle + boundary vocabulary).
//
//   • CubeDimensionKind — van der Aalst's PROCESS CUBE dimensions (slice/dice axes).
//   • ObjectCentricity  — the case-centric vs object-centric axis (the OCEL reason).
//   • EvidenceMode      — the runtime mirror of the Evidence typestate lifecycle.
//
// (ProcessBoundaryKind lives behind the `strict` Cargo feature, which affidavit
// does not enable — it is OUT-OF-SURFACE for the default build, not censused
// here. Noted honestly rather than force-enabling a feature to inflate coverage.)

use wasm4pm_compat::law::{EvidenceMode as Mode, ObjectCentricity as OC};
use wasm4pm_compat::process_cube::CubeDimensionKind as Dim;

#[test]
fn process_cube_dimensions_are_constructed() {
    let all = [
        Dim::Activity,
        Dim::Resource,
        Dim::Time,
        Dim::DataAttribute,
        Dim::ObjectType,
        Dim::CaseAttribute,
    ];
    fn ok(d: Dim) -> bool {
        matches!(
            d,
            Dim::Activity
                | Dim::Resource
                | Dim::Time
                | Dim::DataAttribute
                | Dim::ObjectType
                | Dim::CaseAttribute
        )
    }
    assert!(all.iter().copied().all(ok));
    assert_eq!(all.len(), 6, "six process-cube dimensions");
}

#[test]
fn object_centricity_axis_is_constructed() {
    let all = [OC::CaseCentric, OC::ObjectCentric, OC::Mixed];
    fn label(o: OC) -> &'static str {
        match o {
            OC::CaseCentric => "case",
            OC::ObjectCentric => "object",
            OC::Mixed => "mixed",
        }
    }
    let s: std::collections::BTreeSet<&str> = all.iter().copied().map(label).collect();
    assert_eq!(
        s.len(),
        3,
        "three object-centricity modes (the OCEL convergence/divergence axis)"
    );
}

#[test]
fn evidence_lifecycle_modes_are_constructed() {
    // The runtime mirror of the Evidence<_, State, _> typestate lifecycle.
    let all = [
        Mode::Raw,
        Mode::Parsed,
        Mode::Admitted,
        Mode::Refused,
        Mode::Projected,
        Mode::Exportable,
        Mode::Witnessed,
        Mode::Receipted,
    ];
    // No-wildcard mapping: each lifecycle mode to its pipeline stage family.
    // A new upstream variant breaks compilation here (no `_` arm).
    fn stage_family(m: Mode) -> &'static str {
        match m {
            Mode::Raw | Mode::Parsed => "ingest",
            Mode::Admitted | Mode::Refused => "boundary",
            Mode::Projected => "lossy",
            Mode::Exportable => "egress",
            Mode::Witnessed | Mode::Receipted => "provenance",
        }
    }
    let families: std::collections::BTreeSet<&str> =
        all.iter().copied().map(stage_family).collect();
    assert_eq!(families.len(), 5, "five lifecycle stage families");
    // Distinctness: all 8 modes have distinct Debug renderings (no aliasing).
    let names: std::collections::BTreeSet<String> = all.iter().map(|m| format!("{m:?}")).collect();
    assert_eq!(names.len(), 8, "eight distinct evidence lifecycle modes");
}
