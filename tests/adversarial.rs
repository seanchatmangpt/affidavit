//! Adversarial integration suite: proves the anti-forgery property end-to-end.
//!
//! Each tamper test recomputes nothing for the forged event, so the rolling
//! BLAKE3 chain (or the format/continuity stage) must reject it. Every test
//! also demonstrates teeth: the same construction without the tamper ACCEPTs,
//! so a passing tamper test cannot be a false positive.

use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::ocel::{build_event, object_ref, qualified_object_ref, SeqCounter};
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

// ── NEW TESTS ────────────────────────────────────────────────────────────────

/// A zero-event receipt (genesis state) must not panic under verify().
#[test]
fn zero_event_receipt_is_handled_without_panic() {
    let receipt = ChainAssembler::new().finalize();
    assert_eq!(receipt.events.len(), 0);
    let verdict = verify(&receipt);
    let _ = verdict;
}

/// A single-event receipt built via the public API must be accepted.
#[test]
fn single_event_receipt_accepts() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    let ev = build_event("create", vec![object_ref("o1", "artifact")], b"payload", &mut counter)
        .expect("build event");
    asm.append(ev).expect("append");
    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "single-event receipt must ACCEPT: {}", verdict.reason);
    assert!(verdict.outcomes.iter().all(|o| o.passed));
}

/// Changing event_type to "" must be rejected at evaluate_profile.
#[test]
fn tamper_event_type_rejects_at_evaluate_profile() {
    let mut receipt = honest_receipt();
    assert!(verify(&receipt).accepted);
    receipt.events[0].event_type = "".to_string();
    receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "empty event_type must REJECT");
    assert!(
        !stage(&verdict, "evaluate_profile").passed,
        "empty event_type must be caught at evaluate_profile"
    );
}

/// Replacing chain_hash with zeros must be caught at chain_integrity.
#[test]
fn tamper_chain_hash_directly_rejects() {
    let mut receipt = honest_receipt();
    assert!(verify(&receipt).accepted);
    receipt.chain_hash = affidavit::types::Blake3Hash::from_hex(
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "zeroed chain_hash must REJECT");
    assert!(
        !stage(&verdict, "chain_integrity").passed,
        "zeroed chain_hash must be caught at chain_integrity"
    );
}

/// Setting two events to the same id must be caught at continuity (duplicate id check).
/// (A single empty id in isolation is not detected by the current CoreV1 verifier stages;
/// the continuity stage detects *duplicate* ids. This test uses duplicate empty ids to
/// exercise that path.)
#[test]
fn tamper_event_id_rejects() {
    let mut receipt = honest_receipt();
    assert!(verify(&receipt).accepted);
    // Set both events[0] and events[1] to "" — continuity catches the duplicate.
    receipt.events[0].id = "".to_string();
    receipt.events[1].id = "".to_string();
    receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "duplicate empty event id must REJECT");
    assert!(
        !stage(&verdict, "continuity").passed,
        "duplicate empty id must be caught at continuity"
    );
}

/// A receipt where one event has 5 different qualified object refs must be accepted.
#[test]
fn receipt_with_many_objects_per_event_accepts() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    let objects = vec![
        qualified_object_ref("file1.txt", "artifact", "source"),
        qualified_object_ref("file2.txt", "artifact", "source"),
        qualified_object_ref("file3.txt", "artifact", "output"),
        qualified_object_ref("schema.json", "config", "input"),
        qualified_object_ref("manifest.toml", "config", "input"),
    ];
    let ev = build_event("bundle", objects, b"multi-object payload", &mut counter)
        .expect("build event");
    asm.append(ev).expect("append");
    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "receipt with many objects must ACCEPT: {}", verdict.reason);
}

/// A 3-event receipt where every event has the same event_type must be accepted.
#[test]
fn all_same_event_type_accepts() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for i in 0..3 {
        let ev = build_event(
            "create",
            vec![object_ref(format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "all-same event_type must ACCEPT: {}", verdict.reason);
}

/// A receipt with a Unicode event_type must be accepted.
#[test]
fn unicode_in_event_type_accepts() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for ty in ["créer", "操作"] {
        let ev = build_event(ty, vec![object_ref("u1", "artifact")], b"unicode", &mut counter)
            .expect("build event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "unicode event_type must ACCEPT: {}", verdict.reason);
}

/// A receipt with an event whose id is 1000 characters long must be accepted.
#[test]
fn very_long_event_id_accepts() {
    use affidavit::types::{Blake3Hash, OperationEvent};
    let mut asm = ChainAssembler::new();
    let long_id = "x".repeat(1000);
    let ev = OperationEvent {
        id: long_id,
        seq: 0,
        event_type: "create".to_string(),
        objects: vec![object_ref("o1", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"long-id-payload"),
    };
    asm.append(ev).expect("append");
    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "very long event id must ACCEPT: {}", verdict.reason);
}

/// Swapping payload commitments of two events (without re-chaining) must be caught at chain_integrity.
#[test]
fn tamper_swap_commitments_rejects() {
    let mut receipt = honest_receipt();
    assert!(verify(&receipt).accepted);
    let c0 = receipt.events[0].payload_commitment.clone();
    let c1 = receipt.events[1].payload_commitment.clone();
    assert_ne!(c0, c1, "test setup: commitments must differ");
    receipt.events[0].payload_commitment = c1;
    receipt.events[1].payload_commitment = c0;
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "swapped commitments must REJECT");
    assert!(
        !stage(&verdict, "chain_integrity").passed,
        "swapped commitments must be caught at chain_integrity"
    );
}

/// A 50-event receipt must be accepted and verify must be deterministic.
#[test]
fn large_receipt_accepts_and_is_deterministic() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for i in 0u64..50 {
        let ev = build_event(
            "step",
            vec![object_ref(format!("obj-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    assert_eq!(receipt.events.len(), 50);
    let v1 = verify(&receipt);
    let v2 = verify(&receipt);
    assert!(v1.accepted, "50-event receipt must ACCEPT: {}", v1.reason);
    assert_eq!(v1.reason, v2.reason, "verify must be deterministic over reason");
    assert_eq!(v1, v2, "verify must be fully deterministic");
}

/// A receipt with qualified_object_ref must be accepted.
#[test]
fn receipt_with_qualified_object_refs_accepts() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    let ev = build_event(
        "compile",
        vec![qualified_object_ref("file.txt", "artifact", "source")],
        b"source payload",
        &mut counter,
    )
    .expect("build event");
    asm.append(ev).expect("append");
    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "qualified object ref receipt must ACCEPT: {}", verdict.reason);
}
