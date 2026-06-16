//! COMBINATORIAL MAXIMALISM: Feature 3.4 (Property Testing)
//!
//! Implementation of Quickcheck Arbitrary instances for the Receipt type system
//! and exhaustive property tests for decidability and tamper-detection.
//!
//! This file is a self-contained property testing suite designed to be moved
//! to `tests/property_based.rs` once validated.

use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::types::{Blake3Hash, ObjectRef, OperationEvent, Receipt, Verdict};
use affidavit::verifier::verify;
use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
use quickcheck_macros::quickcheck;

// --- Arbitrary Implementations ---

/// Wrapper for OperationEvent to implement Arbitrary (orphan rule bypass if needed).
#[derive(Clone, Debug)]
pub struct ArbitraryOperationEvent(pub OperationEvent);

impl Arbitrary for ArbitraryOperationEvent {
    fn arbitrary(g: &mut Gen) -> Self {
        let id = format!("ev-{}", u64::arbitrary(g));
        let seq = u64::arbitrary(g);
        let event_type = match u32::arbitrary(g) % 3 {
            0 => "process.exec".to_string(),
            1 => "artifact.sign".to_string(),
            _ => "admission.gate".to_string(),
        };

        let mut objects = Vec::new();
        let num_objs = u32::arbitrary(g) % 4;
        for i in 0..num_objs {
            objects.push(ObjectRef {
                id: format!("obj-{}", i),
                obj_type: "blob".to_string(),
                qualifier: if bool::arbitrary(g) {
                    Some("input".to_string())
                } else {
                    None
                },
            });
        }

        let payload: Vec<u8> = Arbitrary::arbitrary(g);
        let payload_commitment = Blake3Hash::from_bytes(&payload);

        ArbitraryOperationEvent(OperationEvent {
            id,
            seq,
            event_type,
            objects,
            payload_commitment,
        })
    }
}

/// Wrapper for Receipt to implement Arbitrary.
/// Generates structurally valid receipts by default using the ChainAssembler.
#[derive(Clone, Debug)]
pub struct ArbitraryReceipt(pub Receipt);

impl Arbitrary for ArbitraryReceipt {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut asm = ChainAssembler::new();
        // Generate a chain of 1 to 15 events to keep test time reasonable.
        let num_events = (u32::arbitrary(g) % 15) + 1;

        for i in 0..num_events {
            let mut ev = ArbitraryOperationEvent::arbitrary(g).0;
            // Force seq continuity and unique IDs for the "valid" base case
            ev.seq = i as u64;
            ev.id = format!("event-{}", i);
            asm.append(ev)
                .expect("ChainAssembler append must succeed for arbitrary events");
        }

        ArbitraryReceipt(asm.finalize())
    }
}

// --- Property Tests ---

/// Property: Decidability
/// Every receipt produced by a lawful ChainAssembler MUST be accepted by the verifier.
/// This confirms the "honest path" is always decidable and positive.
#[quickcheck]
fn prop_decidability_valid_receipts_always_accepted(arb: ArbitraryReceipt) -> bool {
    let receipt = arb.0;
    let verdict = verify(&receipt);

    if !verdict.accepted {
        eprintln!(
            "FAILURE: Valid receipt rejected! Reason: {}",
            verdict.reason
        );
        for outcome in &verdict.outcomes {
            if !outcome.passed {
                eprintln!("  Stage {} failed: {}", outcome.stage, outcome.detail);
            }
        }
    }

    verdict.accepted
}

/// Property: Tamper-Detection (Payload Commitment)
/// Mutating any payload commitment in an otherwise valid receipt MUST break chain integrity.
#[quickcheck]
fn prop_tamper_detection_commitment_flip_breaks_chain(
    arb: ArbitraryReceipt,
    event_idx: usize,
    seed: u64,
) -> TestResult {
    let mut receipt = arb.0;
    if receipt.events.is_empty() {
        return TestResult::discard();
    }

    let idx = event_idx % receipt.events.len();
    // Mutate the commitment
    receipt.events[idx].payload_commitment = Blake3Hash::from_bytes(&seed.to_le_bytes());

    let verdict = verify(&receipt);

    // We expect REJECT due to chain_integrity
    let chain_stage = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "chain_integrity")
        .unwrap();
    TestResult::from_bool(!verdict.accepted && !chain_stage.passed)
}

/// Property: Tamper-Detection (Event Type)
/// Mutating an event_type MUST break chain integrity because event_type is part of the canonical bytes.
#[quickcheck]
fn prop_tamper_detection_event_type_mutation_breaks_chain(
    arb: ArbitraryReceipt,
    event_idx: usize,
) -> TestResult {
    let mut receipt = arb.0;
    if receipt.events.is_empty() {
        return TestResult::discard();
    }

    let idx = event_idx % receipt.events.len();
    receipt.events[idx].event_type += "_mutated";

    let verdict = verify(&receipt);
    let chain_stage = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "chain_integrity")
        .unwrap();
    TestResult::from_bool(!verdict.accepted && !chain_stage.passed)
}

/// Property: Continuity Enforcement
/// Introducing a gap in sequence numbers MUST be caught by the continuity stage.
#[quickcheck]
fn prop_continuity_gap_is_rejected(arb: ArbitraryReceipt, event_idx: usize) -> TestResult {
    let mut receipt = arb.0;
    // Need at least 2 events to create a gap in the middle, or 1 to mess up the start
    if receipt.events.len() < 2 {
        return TestResult::discard();
    }

    let idx = (event_idx % (receipt.events.len() - 1)) + 1;
    receipt.events[idx].seq += 1; // Create a gap

    // Recompute chain hash so ONLY continuity fails, not chain_integrity
    receipt.chain_hash = recompute_chain(&receipt.events).unwrap();

    let verdict = verify(&receipt);
    let continuity_stage = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "continuity")
        .unwrap();

    TestResult::from_bool(!verdict.accepted && !continuity_stage.passed)
}

/// Property: Format Version Enforcement
/// Receipts with an unsupported format version MUST be rejected.
#[quickcheck]
fn prop_format_version_enforcement(arb: ArbitraryReceipt, mut version: String) -> TestResult {
    let mut receipt = arb.0;
    if version == FORMAT_VERSION || version.trim().is_empty() {
        version = "unsupported/v99".to_string();
    }

    receipt.format_version = version;
    let verdict = verify(&receipt);
    let format_stage = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "check_format")
        .unwrap();

    TestResult::from_bool(!verdict.accepted && !format_stage.passed)
}

/// Property: Id Uniqueness
/// Duplicate event IDs MUST be caught by the continuity stage.
#[quickcheck]
fn prop_duplicate_ids_are_rejected(arb: ArbitraryReceipt) -> TestResult {
    let mut receipt = arb.0;
    if receipt.events.len() < 2 {
        return TestResult::discard();
    }

    // Copy ID from event 0 to event 1
    receipt.events[1].id = receipt.events[0].id.clone();

    // Recompute chain hash so ONLY continuity fails
    receipt.chain_hash = recompute_chain(&receipt.events).unwrap();

    let verdict = verify(&receipt);
    let continuity_stage = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "continuity")
        .unwrap();

    TestResult::from_bool(!verdict.accepted && !continuity_stage.passed)
}

/// Property: Empty Event Type (CoreV1 Profile)
/// Empty event types MUST be rejected by the profile evaluation stage.
#[quickcheck]
fn prop_empty_event_type_rejected(arb: ArbitraryReceipt, event_idx: usize) -> TestResult {
    let mut receipt = arb.0;
    if receipt.events.is_empty() {
        return TestResult::discard();
    }

    let idx = event_idx % receipt.events.len();
    receipt.events[idx].event_type = "  ".to_string();

    // Recompute chain hash so ONLY profile evaluation fails
    receipt.chain_hash = recompute_chain(&receipt.events).unwrap();

    let verdict = verify(&receipt);
    let profile_stage = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "evaluate_profile")
        .unwrap();

    TestResult::from_bool(!verdict.accepted && !profile_stage.passed)
}

/// Property: Chain Determinism
/// The same receipt MUST always yield the same verdict (no side effects in verifier).
#[quickcheck]
fn prop_verdict_determinism(arb: ArbitraryReceipt) -> bool {
    let receipt = arb.0;
    let v1 = verify(&receipt);
    let v2 = verify(&receipt);
    v1 == v2
}

fn main() {
    // This main function allows running these tests as a standalone binary if desired,
    // though they are intended to be run via `cargo test`.
    println!("Running property tests...");
}
