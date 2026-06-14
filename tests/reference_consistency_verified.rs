// Reference witness: the sealed causal-consistency envelope — CausallyOrderedEvidence,
// VerifyCausalConsistency, ConsistencyVerified (COVERAGE.md §2 — causal-consistency
// sealing law). Same non-forgeability idiom as the receipt's admit().
//
// ConsistencyVerified<T> has a pub(crate) constructor — external code CANNOT mint a
// "verified" envelope by fiat. The only public door is VerifyCausalConsistency::verify,
// which an impl satisfies using the sealed ConsistencyProof. This witnesses: (a)
// CausallyOrderedEvidence wraps a payload; (b) running the public UnknownVerifier
// yields a ConsistencyVerified whose verdict is honestly Unknown (the trivial verifier
// refuses to claim Consistent it did not establish); (c) is_consistent() reflects the
// verdict. Failing-when-fake: there is NO non-verifier path to a ConsistencyVerified.

use wasm4pm_compat::causality::{
    CausalConsistency, CausallyOrderedEvidence, UnknownVerifier, VerifyCausalConsistency,
};

#[test]
fn causally_ordered_evidence_wraps_its_payload() {
    let ev = CausallyOrderedEvidence::new("trace-42");
    assert_eq!(ev.inner, "trace-42");
}

#[test]
fn the_only_path_to_a_verdict_is_running_a_verifier() {
    // UnknownVerifier is the honest floor: it returns Unknown, not a fabricated
    // Consistent. The envelope can ONLY be obtained this way (sealed constructor).
    let verified = UnknownVerifier.verify(99u64);
    assert_eq!(verified.inner, 99);
    assert_eq!(verified.verdict(), CausalConsistency::Unknown, "trivial verifier claims only Unknown");
    assert!(!verified.is_consistent(), "Unknown is not Consistent — no over-claim");
}
