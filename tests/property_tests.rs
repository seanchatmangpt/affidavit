//! Property-style invariant tests: prove structural invariants hold across many inputs.
//!
//! These are not fuzzing tests — they use fixed representative inputs covering the
//! edge-of-domain spectrum (0, 1, 2, many, max-representative) to prove invariants
//! that must hold universally.

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::verifier::verify;

fn build_honest(n: usize) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for i in 0..n {
        let ty = ["create", "transform", "validate", "release"][i % 4];
        let ev = build_event(ty, vec![object_ref("obj", "artifact")], format!("payload-{i}").as_bytes(), &mut counter)
            .expect("build_event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

/// INVARIANT: verify(honest_receipt) == ACCEPT for all sizes 0..=50
#[test]
fn invariant_honest_receipt_always_accepts() {
    for n in [0, 1, 2, 3, 5, 10, 20, 50] {
        let receipt = build_honest(n);
        let verdict = verify(&receipt);
        assert!(verdict.accepted, "size={n}: honest receipt must ACCEPT; reason={}", verdict.reason);
    }
}

/// INVARIANT: verify is deterministic — same receipt yields identical Verdict
#[test]
fn invariant_verify_is_deterministic() {
    for n in [1, 5, 20] {
        let receipt = build_honest(n);
        let v1 = verify(&receipt);
        let v2 = verify(&receipt);
        assert_eq!(v1, v2, "size={n}: verify must be deterministic");
    }
}

/// INVARIANT: receipt.events.len() == n for honest receipts
#[test]
fn invariant_event_count_preserved() {
    for n in [0, 1, 5, 10, 100] {
        let receipt = build_honest(n);
        assert_eq!(receipt.events.len(), n, "size={n}: event count must be preserved");
    }
}

/// INVARIANT: recompute_chain([]) returns Ok (genesis state)
#[test]
fn invariant_recompute_chain_empty_ok() {
    let result = recompute_chain(&[]);
    assert!(result.is_ok(), "recompute_chain on empty slice must return Ok");
}

/// INVARIANT: seq numbers are 0-indexed monotone increasing in honest receipts
#[test]
fn invariant_seq_monotone_increasing() {
    for n in [1, 5, 10] {
        let receipt = build_honest(n);
        for (i, ev) in receipt.events.iter().enumerate() {
            assert_eq!(ev.seq, i as u64, "size={n}: event[{i}].seq must be {i}");
        }
    }
}

/// INVARIANT: adding a tampered event always causes REJECT
#[test]
fn invariant_tamper_always_rejects() {
    for n in [1, 3, 10] {
        let mut receipt = build_honest(n);
        // flip first byte of chain_hash hex
        let hex = receipt.chain_hash.as_hex();
        let mut chars: Vec<char> = hex.chars().collect();
        chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
        receipt.chain_hash = affidavit::types::Blake3Hash::from_hex(chars.into_iter().collect::<String>());
        let verdict = verify(&receipt);
        assert!(!verdict.accepted, "size={n}: tampered chain_hash must REJECT");
    }
}

/// INVARIANT: verdict.outcomes is non-empty for any non-zero-event receipt
#[test]
fn invariant_outcomes_non_empty_for_nonempty_receipt() {
    for n in [1, 5, 20] {
        let receipt = build_honest(n);
        let verdict = verify(&receipt);
        assert!(!verdict.outcomes.is_empty(), "size={n}: outcomes must be non-empty");
    }
}

/// INVARIANT: all stage outcomes have non-empty stage names
#[test]
fn invariant_all_stage_names_nonempty() {
    let receipt = build_honest(5);
    let verdict = verify(&receipt);
    for outcome in &verdict.outcomes {
        assert!(!outcome.stage.is_empty(), "stage name must not be empty; got {:?}", outcome);
    }
}

/// INVARIANT: honest receipt with N objects per event still accepts
#[test]
fn invariant_multi_object_events_accept() {
    for n_objects in [1, 3, 10] {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let objects: Vec<_> = (0..n_objects)
            .map(|i| object_ref(format!("obj-{i}"), "artifact"))
            .collect();
        let ev = build_event("create", objects, b"payload", &mut counter).expect("build_event");
        asm.append(ev).expect("append");
        let receipt = asm.finalize();
        let verdict = verify(&receipt);
        assert!(verdict.accepted, "n_objects={n_objects}: must ACCEPT; reason={}", verdict.reason);
    }
}

/// INVARIANT: recompute_chain produces the same hash for the same events
#[test]
fn invariant_recompute_chain_deterministic() {
    let receipt = build_honest(10);
    let h1 = recompute_chain(&receipt.events).expect("recompute 1");
    let h2 = recompute_chain(&receipt.events).expect("recompute 2");
    assert_eq!(h1, h2, "recompute_chain must be deterministic");
}
