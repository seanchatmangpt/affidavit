//! Adversarial integration suite: proves the anti-forgery property end-to-end.
//!
//! Each tamper test recomputes nothing for the forged event, so the rolling
//! BLAKE3 chain (or the format/continuity stage) must reject it. Every test
//! also demonstrates teeth: the same construction without the tamper ACCEPTs,
//! so a passing tamper test cannot be a false positive.

use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{canonical_bytes, Receipt};
use affidavit::verifier::verify;

/// Build an honest, finalized 3-event receipt via the public builder API.
fn honest_receipt() -> Receipt {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for (ty, payload) in [
        ("emit", b"alpha".as_slice()),
        ("transform", b"beta".as_slice()),
        ("release", b"gamma".as_slice()),
    ] {
        let ev = build_event(ty, vec![object_ref("o1", "artifact")], payload, &mut counter)
            .expect("build event");
        asm.append(ev).expect("append event");
    }
    asm.finalize()
}

/// Find a stage outcome by name, asserting the stage exists.
fn stage<'a>(verdict: &'a affidavit::types::Verdict, name: &str) -> &'a affidavit::types::CheckOutcome {
    verdict
        .outcomes
        .iter()
        .find(|o| o.stage == name)
        .unwrap_or_else(|| panic!("stage {name} present"))
}

#[test]
fn golden_honest_receipt_accepts() {
    let receipt = honest_receipt();
    assert_eq!(receipt.events.len(), 3);
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "honest receipt must ACCEPT: {}", verdict.reason);
    assert_eq!(verdict.reason, "all stages passed");
    assert!(verdict.outcomes.iter().all(|o| o.passed));
}

#[test]
fn tamper_commitment_rejects_at_chain_integrity() {
    let mut receipt = honest_receipt();
    // Sanity: untampered ACCEPTs (teeth).
    assert!(verify(&receipt).accepted);

    // Flip one byte of an event's payload_commitment hex without re-chaining.
    let hex = receipt.events[1].payload_commitment.as_hex();
    let mut chars: Vec<char> = hex.chars().collect();
    chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
    let forged: String = chars.into_iter().collect();
    receipt.events[1].payload_commitment = affidavit::types::Blake3Hash::from_hex(forged);

    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "tampered commitment must REJECT");
    assert!(
        !stage(&verdict, "chain_integrity").passed,
        "tamper must be caught at chain_integrity"
    );
}

#[test]
fn tamper_reorder_rejects() {
    let mut receipt = honest_receipt();
    assert!(verify(&receipt).accepted);

    // Swap two events, breaking seq monotonicity and the chain binding.
    receipt.events.swap(0, 1);

    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "reordered events must REJECT");
    let continuity_failed = !stage(&verdict, "continuity").passed;
    let chain_failed = !stage(&verdict, "chain_integrity").passed;
    assert!(
        continuity_failed || chain_failed,
        "reorder must trip continuity and/or chain_integrity"
    );
}

#[test]
fn tamper_inject_rejects_at_chain_integrity() {
    let mut receipt = honest_receipt();
    assert!(verify(&receipt).accepted);

    // Fabricate an event with a valid-looking seq/id but DON'T recompute the chain hash.
    let mut counter = SeqCounter::starting_at(receipt.events.len() as u64);
    let injected = build_event(
        "forged",
        vec![object_ref("evil", "artifact")],
        b"fabricated",
        &mut counter,
    )
    .expect("build injected event");
    receipt.events.push(injected);
    // chain_hash deliberately left stale.

    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "injected event must REJECT");
    assert!(
        !stage(&verdict, "chain_integrity").passed,
        "injection without re-chaining must be caught at chain_integrity"
    );
}

#[test]
fn wrong_version_rejects_at_check_format() {
    let mut receipt = honest_receipt();
    assert_eq!(receipt.format_version, FORMAT_VERSION);
    assert!(verify(&receipt).accepted);

    receipt.format_version = "99.9.9".to_string();

    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "wrong format_version must REJECT");
    assert!(
        !stage(&verdict, "check_format").passed,
        "wrong version must be caught at check_format"
    );
}

#[test]
fn determinism_identical_verdict_bytes() {
    let receipt = honest_receipt();
    let v1 = verify(&receipt);
    let v2 = verify(&receipt);

    let b1 = canonical_bytes(&v1).expect("serialize verdict 1");
    let b2 = canonical_bytes(&v2).expect("serialize verdict 2");
    assert_eq!(b1, b2, "verifying the same receipt must yield identical Verdict bytes");

    // Teeth: the chain itself is deterministic, so the bytes aren't trivially equal.
    assert_eq!(
        recompute_chain(&receipt.events).unwrap(),
        recompute_chain(&receipt.events).unwrap()
    );
}
