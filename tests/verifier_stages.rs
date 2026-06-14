//! Per-stage unit tests for the 7-stage certify pipeline.
//! Each test targets one specific stage, proving the stage is active
//! and catches exactly the violation it's designed for.

use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, Receipt};
use affidavit::verifier::verify;

fn stage<'a>(verdict: &'a affidavit::types::Verdict, name: &str) -> &'a affidavit::types::CheckOutcome {
    verdict.outcomes.iter().find(|o| o.stage == name)
        .unwrap_or_else(|| panic!("stage {name} not found in outcomes"))
}

fn honest_1event() -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut c = SeqCounter::new();
    let ev = build_event("create", vec![object_ref("o", "artifact")], b"payload", &mut c).unwrap();
    asm.append(ev).unwrap();
    asm.finalize()
}

// ── Stage 2: check_format ───────────────────────────────────────────────────

#[test]
fn stage_check_format_catches_wrong_version() {
    let mut receipt = honest_1event();
    assert!(verify(&receipt).accepted, "honest receipt must accept first");
    receipt.format_version = "bad/v0".to_string();
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "wrong version must REJECT");
    assert!(
        !stage(&verdict, "check_format").passed,
        "wrong version must fail check_format"
    );
}

#[test]
fn stage_check_format_passes_on_correct_version() {
    let receipt = honest_1event();
    assert_eq!(receipt.format_version, FORMAT_VERSION);
    let verdict = verify(&receipt);
    assert!(stage(&verdict, "check_format").passed, "correct version must pass check_format");
}

// ── Stage 3: chain_integrity ────────────────────────────────────────────────

#[test]
fn stage_chain_integrity_catches_tampered_chain_hash() {
    let mut receipt = honest_1event();
    receipt.chain_hash = Blake3Hash::from_hex(
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "tampered chain_hash must REJECT");
    assert!(
        !stage(&verdict, "chain_integrity").passed,
        "tampered chain_hash must fail chain_integrity"
    );
}

#[test]
fn stage_chain_integrity_catches_tampered_event_commitment() {
    let mut receipt = honest_1event();
    // Flip first hex char of the commitment
    let hex = receipt.events[0].payload_commitment.as_hex();
    let mut chars: Vec<char> = hex.chars().collect();
    chars[0] = if chars[0] == 'a' { 'b' } else { 'a' };
    let forged: String = chars.into_iter().collect();
    receipt.events[0].payload_commitment = Blake3Hash::from_hex(forged);
    // Do NOT recompute chain_hash, so chain_integrity fires
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "tampered commitment must REJECT");
    assert!(
        !stage(&verdict, "chain_integrity").passed,
        "tampered commitment must fail chain_integrity"
    );
}

// ── Stage 4: continuity ─────────────────────────────────────────────────────

#[test]
fn stage_continuity_catches_gap_at_start() {
    // Build a 1-event receipt with seq=0, then change seq to 1 and recompute chain.
    // The chain becomes consistent but seq doesn't start at 0 -- continuity must catch it.
    let mut receipt = honest_1event();
    receipt.events[0].seq = 1; // seq should be 0
    receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "seq not starting at 0 must REJECT");
    assert!(
        !stage(&verdict, "continuity").passed,
        "seq not starting at 0 must fail continuity"
    );
}

#[test]
fn stage_continuity_catches_duplicate_id() {
    let mut asm = ChainAssembler::new();
    let mut c = SeqCounter::new();
    // Build two events, manually giving them the same ID by mutating after build
    let ev0 = build_event("op", vec![object_ref("o", "artifact")], b"p0", &mut c).unwrap();
    let mut ev1 = build_event("op", vec![object_ref("o", "artifact")], b"p1", &mut c).unwrap();
    ev1.id = ev0.id.clone(); // duplicate id
    asm.append(ev0).unwrap();
    asm.append(ev1).unwrap();
    let mut receipt = asm.finalize();
    // Recompute chain to ensure chain_integrity passes, isolating continuity
    receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");

    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "duplicate id must REJECT");
    assert!(
        !stage(&verdict, "continuity").passed,
        "duplicate id must fail continuity"
    );
}

#[test]
fn stage_continuity_catches_seq_gap() {
    // Build a 2-event receipt with seq 0, then set seq[1] = 2 (gap)
    let mut asm = ChainAssembler::new();
    let mut c = SeqCounter::new();
    let ev0 = build_event("op", vec![object_ref("o", "artifact")], b"p0", &mut c).unwrap();
    let mut ev1 = build_event("op", vec![object_ref("o", "artifact")], b"p1", &mut c).unwrap();
    ev1.seq = 2; // gap: expected 1
    asm.append(ev0).unwrap();
    asm.append(ev1).unwrap();
    let mut receipt = asm.finalize();
    // Recompute chain so chain_integrity passes, isolating continuity
    receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");

    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "seq gap must REJECT");
    assert!(
        !stage(&verdict, "continuity").passed,
        "seq gap must fail continuity"
    );
}

// ── Stage 5: verify_commitments ─────────────────────────────────────────────

/// Blake3Hash::from_bytes always produces a valid 64-char lowercase hex digest,
/// so verify_commitments always passes for receipts built through the public API.
/// This test documents that invariant.
#[test]
fn stage_verify_commitments_passes_on_honest_receipts() {
    let receipt = honest_1event();
    let verdict = verify(&receipt);
    assert!(
        stage(&verdict, "verify_commitments").passed,
        "valid hex always passes verify_commitments"
    );
}

// ── Stage 6: evaluate_profile ───────────────────────────────────────────────

#[test]
fn stage_evaluate_profile_catches_empty_event_type() {
    let mut receipt = honest_1event();
    receipt.events[0].event_type = "".to_string();
    // Recompute chain so chain_integrity passes, isolating evaluate_profile
    receipt.chain_hash = recompute_chain(&receipt.events).expect("recompute");
    let verdict = verify(&receipt);
    assert!(!verdict.accepted, "empty event_type must REJECT");
    assert!(
        !stage(&verdict, "evaluate_profile").passed,
        "empty event_type must fail evaluate_profile"
    );
}

// ── All stages ───────────────────────────────────────────────────────────────

#[test]
fn all_stages_pass_on_honest_receipt() {
    let receipt = honest_1event();
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "honest receipt must be accepted: {}", verdict.reason);
    for outcome in &verdict.outcomes {
        assert!(
            outcome.passed,
            "stage {} must pass on honest receipt (detail: {})",
            outcome.stage, outcome.detail
        );
    }
}
