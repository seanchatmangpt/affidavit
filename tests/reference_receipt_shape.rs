// Reference witness: the receipt-shape positive surface — Digest / ReplayHint
// newtypes and a well-shaped ReceiptEnvelope (COVERAGE.md §2 — receipt shape;
// complements the ReceiptRefusal witnesses).
//
// A ReceiptEnvelope is well-shaped iff it has a non-empty subject, witness,
// digest, and replay hint (try_from_parts enforces this). Digest and ReplayHint
// are transparent string newtypes. This witnesses the positive (well-shaped) case.

use wasm4pm_compat::receipt::{Digest, ReceiptEnvelope, ReplayHint};

#[test]
fn digest_and_replay_hint_carry_their_strings() {
    let d = Digest::new("blake3:abc");
    assert_eq!(d.0, "blake3:abc");
    let h = ReplayHint::new("run-7@step-3");
    assert_eq!(h.0, "run-7@step-3");
}

#[test]
fn well_formed_envelope_is_well_shaped() {
    let env = ReceiptEnvelope::try_from_parts(
        "subject-1",
        "witness-A",
        Digest::new("d1"),
        ReplayHint::new("h1"),
    )
    .expect("complete parts → well-shaped envelope");
    assert!(env.is_well_shaped(), "an envelope with all four parts is well-shaped");
}
