//! Capability register row #75 — streaming online/offline context-confusion
//! court.
//!
//! Compat tags evidence as [`OnlineEvidence`] or [`OfflineEvidence`]. Offline
//! conformance admission must refuse online-tagged evidence — a live window
//! is not a finished log. This court is the type/runtime gate that makes the
//! mislabel loud.
//!
//! Falsifier: online-vs-offline mislabeled evidence flips admission.

use wasm4pm_compat::streaming::{
    ContextualEvidence, OfflineAnalysisContext, OfflineEvidence, OnlineEvidence,
    OnlineMonitoringContext,
};

use crate::mining::MiningError;

/// Typed refusal when online evidence is presented to an offline court.
pub const REFUSED_STREAMING_CONTEXT_CONFUSION: &str = "REFUSED_STREAMING_CONTEXT_CONFUSION";

/// Subject matter for offline OCEL conformance admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineConformanceSubject {
    /// Opaque log reference (must be non-empty).
    pub log_ref: String,
}

/// Admit offline-tagged evidence for conformance work.
pub fn admit_offline_conformance_subject(
    evidence: &OfflineEvidence<OfflineConformanceSubject>,
) -> Result<&OfflineConformanceSubject, MiningError> {
    let subject = &evidence.inner;
    if subject.log_ref.trim().is_empty() {
        return Err(MiningError::Alignment(
            "REFUSED_STREAMING_SUBJECT_UNGROUNDED: empty log_ref".into(),
        ));
    }
    Ok(subject)
}

/// Runtime confusion detector: an online-tagged subject handed to the offline
/// court is refused typed. (The type system already separates the aliases;
/// this path exists so a caller that erased the tag via a common carrier can
/// still be refused by name.)
pub fn refuse_online_as_offline(
    _online: &OnlineEvidence<OfflineConformanceSubject>,
) -> Result<(), MiningError> {
    Err(MiningError::Alignment(format!(
        "{REFUSED_STREAMING_CONTEXT_CONFUSION}: online evidence presented to offline conformance court"
    )))
}

/// Re-tag helper used only by the falsifier mutation test: build offline from
/// the same inner bytes an online carrier held.
pub fn offline_from_inner(
    inner: OfflineConformanceSubject,
) -> OfflineEvidence<OfflineConformanceSubject> {
    ContextualEvidence::<_, OfflineAnalysisContext>::offline(inner)
}

/// Build online-tagged evidence (falsifier input).
pub fn online_from_inner(
    inner: OfflineConformanceSubject,
) -> OnlineEvidence<OfflineConformanceSubject> {
    ContextualEvidence::<_, OnlineMonitoringContext>::online(inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_evidence_admits() {
        let ev = offline_from_inner(OfflineConformanceSubject {
            log_ref: "log:fixture".into(),
        });
        let s = admit_offline_conformance_subject(&ev).expect("offline must admit");
        assert_eq!(s.log_ref, "log:fixture");
    }

    #[test]
    fn online_presented_to_offline_court_refuses() {
        let online = online_from_inner(OfflineConformanceSubject {
            log_ref: "log:fixture".into(),
        });
        let err = refuse_online_as_offline(&online).expect_err("online must refuse");
        assert!(err
            .to_string()
            .contains(REFUSED_STREAMING_CONTEXT_CONFUSION));
    }

    #[test]
    fn mislabel_mutation_flips_admission() {
        let subject = OfflineConformanceSubject {
            log_ref: "log:fixture".into(),
        };
        // Honest: offline admits.
        assert!(admit_offline_conformance_subject(&offline_from_inner(subject.clone())).is_ok());
        // Mutation: same payload tagged online → refuse at the offline court.
        assert!(refuse_online_as_offline(&online_from_inner(subject)).is_err());
    }
}
