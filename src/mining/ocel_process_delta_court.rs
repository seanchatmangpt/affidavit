//! Capability register row #93 — ProcessDelta unresolved-evidence court.
//!
//! `ProcessDeltaKind::TelemetryGap` (and other unresolved kinds) must never be
//! promoted into a conformance verdict. This court:
//! 1. Requires `admit_flat()` (non-empty `process_ref`).
//! 2. Refuses unresolved kinds with `REFUSED_UNRESOLVED_PROCESS_DELTA` —
//!    they are evidence gaps, not Conformant/NonConformant outcomes.
//! 3. Only resolved kinds (`Conformant`, `UnexpectedTransition`, …) may become
//!    a [`ProcessDeltaVerdict`].
//!
//! Falsifier: promoting `TelemetryGap` as Conformant flips the court to
//! typed refusal.

use wasm4pm_compat::process_delta::{ProcessDelta, ProcessDeltaKind, ProcessDeltaRefusal};

use crate::mining::MiningError;

/// Typed refusal: unresolved evidence was treated as a conformance verdict.
pub const REFUSED_UNRESOLVED_PROCESS_DELTA: &str = "REFUSED_UNRESOLVED_PROCESS_DELTA";

/// Typed refusal: the delta claim failed structure-only `admit_flat`.
pub const REFUSED_PROCESS_DELTA_UNGROUNDED: &str = "REFUSED_PROCESS_DELTA_UNGROUNDED";

/// A process-delta that has cleared both grounding and resolved-evidence law.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessDeltaVerdict {
    /// The resolved kind (never an unresolved evidence class).
    pub kind: ProcessDeltaKind,
    /// Process reference the claim is grounded against.
    pub process_ref: String,
}

fn ungrounded_msg(r: ProcessDeltaRefusal) -> String {
    let law = match r {
        ProcessDeltaRefusal::UngroundedDelta => "UngroundedDelta",
        ProcessDeltaRefusal::AmbiguousCorrelation => "AmbiguousCorrelation",
        _ => "Unspecified",
    };
    format!("{REFUSED_PROCESS_DELTA_UNGROUNDED}: {law}")
}

/// Admit a process delta as a conformance-capable verdict, or refuse typed.
///
/// Unresolved kinds (`TelemetryGap`, `IncompleteTrace`, `CorrelationAmbiguity`)
/// are refused even when structurally grounded — absence of evidence is never
/// a Conformant cell.
pub fn admit_process_delta_verdict<T>(
    delta: &ProcessDelta<T>,
) -> Result<ProcessDeltaVerdict, MiningError> {
    if let Err(r) = delta.admit_flat() {
        return Err(MiningError::Alignment(ungrounded_msg(r)));
    }
    if delta.kind.is_unresolved() {
        return Err(MiningError::Alignment(format!(
            "{REFUSED_UNRESOLVED_PROCESS_DELTA}: {:?} is unresolved evidence, not a conformance verdict",
            delta.kind
        )));
    }
    Ok(ProcessDeltaVerdict {
        kind: delta.kind,
        process_ref: delta.process_ref.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformant_delta_admits() {
        let d = ProcessDelta::<()>::new(ProcessDeltaKind::Conformant, "case:42");
        let v = admit_process_delta_verdict(&d).expect("conformant must admit");
        assert_eq!(v.kind, ProcessDeltaKind::Conformant);
        assert_eq!(v.process_ref, "case:42");
    }

    #[test]
    fn telemetry_gap_never_becomes_a_conformance_verdict() {
        let d = ProcessDelta::<()>::new(ProcessDeltaKind::TelemetryGap, "case:42");
        let err = admit_process_delta_verdict(&d).expect_err("TelemetryGap must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains(REFUSED_UNRESOLVED_PROCESS_DELTA),
            "expected unresolved refusal, got {msg}"
        );
    }

    #[test]
    fn promoting_gap_as_conformant_is_the_falsifier_mutation() {
        // Honest path: gap refuses.
        let gap = ProcessDelta::<()>::new(ProcessDeltaKind::TelemetryGap, "case:42");
        assert!(admit_process_delta_verdict(&gap).is_err());
        // Mutation: rewrite kind to Conformant without resolving evidence —
        // that *would* accept. The court exists so callers cannot skip the
        // unresolved check; this documents the flip.
        let mutated = ProcessDelta::<()>::new(ProcessDeltaKind::Conformant, "case:42");
        assert!(admit_process_delta_verdict(&mutated).is_ok());
    }

    #[test]
    fn empty_process_ref_refuses_before_kind_check() {
        let d = ProcessDelta::<()>::new(ProcessDeltaKind::Conformant, "");
        let err = admit_process_delta_verdict(&d).expect_err("empty ref must refuse");
        assert!(err.to_string().contains(REFUSED_PROCESS_DELTA_UNGROUNDED));
    }
}
