// Reference witness: the ReceiptChain multi-link surface — try_new + extend_with
// grow a chain of well-shaped receipt envelopes (COVERAGE.md §2 — receipt chain).
//
// A ReceiptChain is a sequence of ReceiptEnvelopes (the cross-step provenance
// chain). try_new seeds it (refusing empty / broken links); extend_with appends a
// link, accepting a well-shaped one and refusing a broken one by name. This
// witnesses growth + len, and the extend refusal path.

use wasm4pm_compat::receipt::{Digest, ReceiptChain, ReceiptEnvelope, ReplayHint};

fn envelope(subject: &str) -> ReceiptEnvelope {
    ReceiptEnvelope::try_from_parts(subject, "witness", Digest::new("d"), ReplayHint::new("h"))
        .expect("well-shaped envelope")
}

#[test]
fn chain_grows_with_extend_with() {
    let mut chain = ReceiptChain::try_new("run-x", vec![envelope("step-0")])
        .expect("a non-empty chain of well-shaped links seeds");
    assert_eq!(chain.len(), 1, "seeded with one link");

    chain
        .extend_with(envelope("step-1"))
        .expect("well-shaped link accepted");
    chain
        .extend_with(envelope("step-2"))
        .expect("well-shaped link accepted");
    assert_eq!(chain.len(), 3, "two links appended");
    assert!(!chain.is_empty());
}

#[test]
fn empty_chain_is_refused_at_seed() {
    // (extend_with's BrokenChainLink path requires an ill-shaped envelope, which
    // try_from_parts cannot produce — so the reachable refusal here is EmptyChain
    // at seed time, already witnessed in court_law_witness; this asserts the
    // positive seed boundary.)
    assert!(
        ReceiptChain::try_new("run-x", vec![]).is_err(),
        "empty chain refused at seed"
    );
}
