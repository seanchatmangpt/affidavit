//! End-to-end demonstration of the multi-stage certify verdict.
//!
//! Builds an HONEST receipt and asserts `verify` ACCEPTS it (every stage passes),
//! then forges a seq-gap receipt (re-sealing the chain so chain_integrity still
//! passes and `continuity` is the FIRST failing stage) and asserts `verify`
//! REJECTS it, that the `continuity` outcome has `passed == false`, and that the
//! verdict `reason` names the continuity stage.
//!
//! See the doc on `verify` in src/verifier.rs.

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent};
use affidavit::verifier::verify;

fn event(id: &str, seq: u64, payload: &[u8]) -> OperationEvent {
    OperationEvent {
        id: id.to_string(),
        seq,
        event_type: "emit".to_string(),
        objects: vec![ObjectRef {
            id: format!("obj-{id}"),
            obj_type: "artifact".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(payload),
    }
}

fn main() {
    // --- HONEST receipt: assembled through the canonical seam. ---
    let mut asm = ChainAssembler::new();
    asm.append(event("e0", 0, b"payload-zero"))
        .expect("append e0");
    asm.append(event("e1", 1, b"payload-one"))
        .expect("append e1");
    let honest = asm.finalize();

    let verdict = verify(&honest);
    println!("HONEST receipt verdict: accepted={}", verdict.accepted);
    for o in &verdict.outcomes {
        println!(
            "  [{}] {} — {}",
            if o.passed { "PASS" } else { "FAIL" },
            o.stage,
            o.detail
        );
    }
    println!("  reason: {}", verdict.reason);

    assert!(
        verdict.accepted,
        "honest receipt must be accepted; reason: {}",
        verdict.reason
    );
    assert!(
        verdict.outcomes.iter().all(|o| o.passed),
        "every stage must pass for an honest receipt"
    );
    assert_eq!(verdict.reason, "all stages passed");

    // --- FORGED receipt: a seq gap (0 then 2). Re-seal the chain over the
    // tampered events so chain_integrity PASSES and `continuity` is the first
    // stage to trip — proving continuity catches gaps independently. ---
    let mut forged = honest.clone();
    forged.events[1].seq = 2; // gap: position 1 expects seq 1, finds 2
    forged.chain_hash = recompute_chain(&forged.events).expect("re-seal forged chain");

    let forged_verdict = verify(&forged);
    println!(
        "\nFORGED (seq-gap) receipt verdict: accepted={}",
        forged_verdict.accepted
    );
    for o in &forged_verdict.outcomes {
        println!(
            "  [{}] {} — {}",
            if o.passed { "PASS" } else { "FAIL" },
            o.stage,
            o.detail
        );
    }
    println!("  reason: {}", forged_verdict.reason);

    assert!(
        !forged_verdict.accepted,
        "forged seq-gap receipt must be REJECTED"
    );

    let continuity = forged_verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "continuity")
        .expect("continuity stage must be present in outcomes");
    assert!(
        !continuity.passed,
        "continuity stage must fail on a seq gap"
    );

    // chain_integrity precedes continuity, so it must still pass — proving the
    // continuity check is what caught the forgery, not the chain hash.
    let chain = forged_verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "chain_integrity")
        .expect("chain_integrity stage present");
    assert!(
        chain.passed,
        "re-sealed chain must pass; continuity is the real catch"
    );

    // The verdict reason must name the continuity stage (first failure).
    assert!(
        forged_verdict.reason.starts_with("continuity:"),
        "reason must name the continuity stage, got: {}",
        forged_verdict.reason
    );

    println!("\nAll assertions passed.");
}
