// Reference witness: the ReceiptChainConst<N> const-generic surface — try_new
// over a fixed-size [ReceiptEnvelope; N] array (COVERAGE.md §2 — receipt chain).
//
// Unlike trybuild compile-fail witnesses, the const-generic refusals here are
// REACHABLE at runtime: N=0 is still a constructible type, so the EmptyChain and
// BrokenChainLink refusals fire on real (not faked) malformed input. This is the
// const-generic sibling of tests/reference_receipt_chain.rs (the runtime Vec
// version); see also tests/reference_multi_instance_spec.rs for the convention.

use wasm4pm_compat::receipt::{
    Digest, ReceiptChainConst, ReceiptEnvelope, ReceiptRefusal, ReplayHint,
};

fn envelope(subject: &str) -> ReceiptEnvelope {
    ReceiptEnvelope::new(subject, "witness", Digest::new("d"), ReplayHint::new("h"))
}

#[test]
fn valid_two_link_chain_exposes_arity_root_tip_iter() {
    let a = envelope("root");
    let b = envelope("step");
    let chain =
        ReceiptChainConst::try_new("run-001", [a, b]).expect("two well-shaped links seed a chain");

    assert_eq!(chain.arity(), 2, "const arity reflects the array length");
    assert_eq!(chain.root().subject, "root", "root is the first link");
    assert_eq!(chain.tip().subject, "step", "tip is the last link");
    assert_eq!(chain.iter().count(), 2, "iter walks every link");
}

#[test]
fn empty_chain_is_refused_at_zero_arity() {
    // Reachable refusal: N == 0 is a constructible type; no trybuild needed.
    let empty: [ReceiptEnvelope; 0] = [];
    let result = ReceiptChainConst::try_new("run-x", empty);
    assert_eq!(
        result,
        Err(ReceiptRefusal::EmptyChain),
        "a zero-arity chain is refused by name"
    );
}

#[test]
fn ill_shaped_link_is_refused_by_index() {
    // Reachable refusal: a real envelope with an empty subject is NOT well-shaped,
    // so the chain refuses at that link's zero-based index. Failing-when-fake — the
    // refusal only fires because the malformed envelope is genuinely ill-shaped.
    let broken = ReceiptEnvelope::new("", "witness", Digest::new("d"), ReplayHint::new("h"));
    assert!(
        !broken.is_well_shaped(),
        "the empty-subject envelope is genuinely ill-shaped"
    );

    let result = ReceiptChainConst::try_new("run-x", [broken]);
    assert_eq!(
        result,
        Err(ReceiptRefusal::BrokenChainLink(0)),
        "the first ill-shaped link is refused at index 0",
    );
}
