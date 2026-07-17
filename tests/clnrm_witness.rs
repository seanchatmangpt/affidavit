#![cfg(feature = "mutation")]
// Witness: affidavit's seal determinism (NFR-1) confirmed by clnrm's INDEPENDENT
// digest harness — the genuine `clnrm-core` integration.
//
// This is not a decoration. clnrm-core's `determinism::digest` is a SHA-256
// trace-verification harness — a DIFFERENT hash family than affidavit's BLAKE3,
// owned by a DIFFERENT crate. Using it to confirm that the same evidence yields
// the same receipt is exactly the "witness terminates outside the producer"
// discipline: an external party, with its own hash, re-checks affidavit's
// determinism claim. affidavit cannot fake clnrm's verdict.
//
// Failing-when-fake on two axes:
//   1. Remove clnrm-core → this file does not compile (integration axis).
//   2. If affidavit's seal were non-deterministic, the two canonical byte
//      streams would differ, clnrm's digests would differ, and the test fails
//      (capability axis).

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{canonical_bytes, Receipt};
use clnrm_core::determinism::digest::{generate_digest, verify_digest};

fn receipt_from(activities: &[&str]) -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for act in activities {
        let ev = build_event(
            *act,
            vec![object_ref("obj", "artifact")],
            act.as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

#[test]
fn clnrm_digest_confirms_seal_is_deterministic() {
    // Same evidence, assembled twice → must produce identical canonical bytes,
    // and therefore identical clnrm SHA-256 digests. clnrm is the external judge.
    let r1 = receipt_from(&["create", "transform", "release"]);
    let r2 = receipt_from(&["create", "transform", "release"]);

    let d1 = generate_digest(&canonical_bytes(&r1).expect("canonical bytes r1"));
    let d2 = generate_digest(&canonical_bytes(&r2).expect("canonical bytes r2"));

    assert_eq!(
        d1, d2,
        "clnrm's independent SHA-256 digest must confirm same-evidence → same-receipt (NFR-1)"
    );
    // And clnrm's own verify path agrees.
    assert!(
        verify_digest(&canonical_bytes(&r1).expect("bytes"), &d2),
        "clnrm verify_digest must accept the matching digest"
    );
}

#[test]
fn clnrm_digest_distinguishes_different_evidence() {
    // Different evidence → different canonical bytes → clnrm digests must differ.
    // (Negative control: if the digest were constant, this fails.)
    let r1 = receipt_from(&["create"]);
    let r2 = receipt_from(&["release"]);

    let d1 = generate_digest(&canonical_bytes(&r1).expect("bytes r1"));
    let d2 = generate_digest(&canonical_bytes(&r2).expect("bytes r2"));

    assert_ne!(
        d1, d2,
        "clnrm digest must distinguish different receipts (the harness is not constant)"
    );
}
