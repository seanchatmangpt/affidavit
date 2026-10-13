//! Golden lifecycle, strata refusal-receipt edition (L189): mint → seal →
//! ACCEPT, tamper → REJECT — the strata-seam counterpart of
//! `examples/golden_run.sh`'s genesis lifecycle.
//!
//! What is bound here (three repos, one artifact):
//!   strata valve derivation (ggen-marketplace pack, `generated/refusals.ex`)
//!     → affidavit chain (`ChainBuilder` with the cryptographic [`Sha256`]
//!       chain hasher, L33/L42 seam)
//!     → sealed receipt (`Verdict::Accept` end to end).
//!
//! Provenance: the pinned receipts were derived by the real pack derivation
//! in ggen-marketplace commit 8210f1b24 ("feat(strata-valve-pack): real
//! refusal-receipt derivation (L13, seam-29 closure)") —
//! /Users/sac/ggen-marketplace/packs/strata-valve-pack/generated/refusals.ex,
//! SHA-256 over `class_label <> 0x00 <> :erlang.term_to_binary(seed)`; the
//! cross-repo parity court over this exact fixture returned verdict
//! IDENTICAL (tests/test_strata_valve_runtime.py, seed `seam-29-l26`).
//!
//! Events are the 4 strata valve refusal classes' fixed opaque 32-byte wire
//! receipts (ground truth: strata_protocol/src/refusal.rs — no cause channel
//! on the wire); each event's commitment is `Sha256(wire receipt)`, so a
//! single-bit tamper of any refusal receipt (or its commitment in the sealed
//! envelope) propagates through the rolling chain hash and is REJECTed with
//! `ChainHashMismatch`.

use affidavit_core::{ChainBuilder, ChainHasher, Digest, Event, Sha256, Verdict};
use sha2::{Digest as ShaDigest, Sha256 as Sha2};

// ---------------------------------------------------------------------------
// Pinned fixture — byte-identical to tests/strata_refusal_seam.rs, which is
// the canonical pin. Kept literally here so this lifecycle court is
// self-contained and the two files' drift fails loudly in both.
//
// Seed: UTF-8 binary `seam-29-l26` (ETF binary_ext: 0x83 0x6d, u32 BE len).
// ---------------------------------------------------------------------------

const SEED: &[u8] = b"seam-29-l26";

fn term_to_binary(seed: &[u8]) -> Vec<u8> {
    let mut out = vec![0x83, 0x6d];
    out.extend_from_slice(&(seed.len() as u32).to_be_bytes());
    out.extend_from_slice(seed);
    out
}

/// Port of the pack's derivation:
/// `:crypto.hash(:sha256, class_label <> 0x00 <> term_to_binary(seed))`.
fn derive_receipt(class_label: &str, seed: &[u8]) -> [u8; 32] {
    let mut h = Sha2::new();
    h.update(class_label.as_bytes());
    h.update([0x00]);
    h.update(term_to_binary(seed));
    h.finalize().into()
}

/// `Strata.Valve.Refusals.ArenaFailure.refuse/1` wire form (pinned).
const ARENA_FAILURE: [u8; 32] = [
    0x68, 0x38, 0x4c, 0x70, 0x4c, 0xf1, 0xb7, 0x57, 0xdf, 0x5e, 0xf4, 0xa5, 0x9c, 0x83, 0x14, 0x60,
    0xbd, 0x27, 0x5b, 0x2b, 0x78, 0xb7, 0x72, 0x08, 0x9a, 0x49, 0xf5, 0xca, 0xd0, 0xd6, 0x0b, 0x01,
];
/// `Strata.Valve.Refusals.FuelExhausted.refuse/1` wire form (pinned).
const FUEL_EXHAUSTED: [u8; 32] = [
    0xcb, 0xc0, 0xde, 0x1a, 0x52, 0x55, 0xcc, 0x10, 0x00, 0x33, 0x34, 0x22, 0xef, 0x2a, 0xc6, 0x13,
    0xfe, 0xc4, 0xa6, 0x01, 0x62, 0x7c, 0x07, 0xb1, 0xaf, 0xa4, 0xaa, 0x22, 0xe5, 0x48, 0xe2, 0x3a,
];
/// `Strata.Valve.Refusals.MalformedFrame.refuse/1` wire form (pinned).
const MALFORMED_FRAME: [u8; 32] = [
    0x91, 0xd5, 0x81, 0x7e, 0x6a, 0x7e, 0x87, 0x53, 0xe6, 0x2b, 0x81, 0x5b, 0x4c, 0xaa, 0x54, 0x93,
    0x9b, 0xfa, 0x87, 0x6b, 0xc4, 0x8c, 0x25, 0x19, 0x74, 0x66, 0x59, 0x69, 0x43, 0x8f, 0xaf, 0x18,
];
/// `Strata.Valve.Refusals.ParentDiscontinuity.refuse/1` wire form (pinned).
const PARENT_DISCONTINUITY: [u8; 32] = [
    0x48, 0x36, 0x8e, 0x7b, 0x17, 0x6a, 0x54, 0xcd, 0x82, 0x13, 0xca, 0xad, 0xa8, 0x97, 0x75, 0x4e,
    0x2d, 0x1b, 0x2e, 0xe5, 0x36, 0xee, 0xd1, 0xb3, 0x6c, 0x73, 0x55, 0x23, 0x04, 0x77, 0x98, 0x4a,
];

/// All four classes in the pack's wire order, as (label, pinned receipt).
const PINNED: [(&str, [u8; 32]); 4] = [
    ("arena_failure", ARENA_FAILURE),
    ("fuel_exhausted", FUEL_EXHAUSTED),
    ("malformed_frame", MALFORMED_FRAME),
    ("parent_discontinuity", PARENT_DISCONTINUITY),
];

/// Mint: executable provenance — the in-test port of the pack's derivation
/// must reproduce every pinned receipt byte-for-byte.
#[test]
fn mint_reproduces_pinned_receipts() {
    for (label, pinned) in PINNED {
        assert_eq!(derive_receipt(label, SEED), pinned, "class {label}");
    }
}

/// Commitment over the exact 32-byte wire receipt, with the cryptographic
/// chain hasher (domain-separated by the chain's own absorb).
fn commitment_of(payload: &[u8; 32]) -> Digest {
    let mut state = <Sha256 as ChainHasher>::init();
    <Sha256 as ChainHasher>::absorb(&mut state, payload);
    <Sha256 as ChainHasher>::finish(state)
}

/// Seal: the 4 minted refusal receipts become 4 affidavit events via
/// `ChainBuilder` with the [`Sha256`] hasher; the envelope is finalized.
fn seal() -> affidavit_core::Receipt {
    let mut builder = ChainBuilder::<Sha256>::new();
    for (label, receipt) in PINNED {
        builder = builder.event(
            "strata_refusal",
            event_id(label),
            commitment_of(&receipt),
        );
    }
    builder.finalize()
}

fn event_id(label: &str) -> &'static str {
    match label {
        "arena_failure" => "strata.valve.refusals.arena_failure",
        "fuel_exhausted" => "strata.valve.refusals.fuel_exhausted",
        "malformed_frame" => "strata.valve.refusals.malformed_frame",
        "parent_discontinuity" => "strata.valve.refusals.parent_discontinuity",
        other => unreachable!("unknown class {other}"),
    }
}

#[test]
fn golden_lifecycle_sealed_receipt_accepts() {
    let receipt = seal();
    assert_eq!(receipt.profile(), affidavit_core::PROFILE);
    assert_eq!(receipt.events().len(), 4);
    for (ev, (label, pinned)) in receipt.events().iter().zip(PINNED) {
        assert_eq!(ev.event_type, "strata_refusal");
        assert_eq!(ev.event_id, event_id(label));
        assert_eq!(ev.commitment, commitment_of(&pinned));
    }
    // The sealed receipt verifies end to end.
    assert_eq!(receipt.verify::<Sha256>(), Verdict::Accept);
}

#[test]
fn golden_lifecycle_tampered_event_rejects() {
    let receipt = seal();
    let honest_chain_hash = receipt.chain_hash();

    // Tamper: flip one bit of one refusal receipt's wire bytes; its
    // commitment changes, so the recomputed chain hash diverges.
    let mut tampered = ARENA_FAILURE;
    tampered[7] ^= 0x01;
    let tampered_commitment = commitment_of(&tampered);
    assert_ne!(tampered_commitment, commitment_of(&ARENA_FAILURE));

    let mut events: Vec<Event<'_>> = receipt
        .events()
        .iter()
        .map(|e| Event {
            seq: e.seq,
            event_id: e.event_id.as_str(),
            event_type: e.event_type.as_str(),
            commitment: e.commitment,
        })
        .collect();
    assert_eq!(events.len(), 4);
    events[0].commitment = tampered_commitment;

    assert_eq!(
        affidavit_core::verify::<Sha256>(&events, &honest_chain_hash, receipt.profile()),
        Verdict::Reject(affidavit_core::RejectReason::ChainHashMismatch),
    );
}

#[test]
fn golden_lifecycle_each_class_tamper_is_rejected() {
    // Systematic: tampering ANY of the 4 refusal receipts must be detected.
    for class in 0..4 {
        let receipt = seal();
        let honest_chain_hash = receipt.chain_hash();
        let mut events: Vec<Event<'_>> = receipt
            .events()
            .iter()
            .map(|e| Event {
                seq: e.seq,
                event_id: e.event_id.as_str(),
                event_type: e.event_type.as_str(),
                commitment: e.commitment,
            })
            .collect();
        let mut tampered = PINNED[class].1;
        tampered[class] ^= 0x01;
        events[class].commitment = commitment_of(&tampered);
        assert_eq!(
            affidavit_core::verify::<Sha256>(&events, &honest_chain_hash, receipt.profile()),
            Verdict::Reject(affidavit_core::RejectReason::ChainHashMismatch),
            "tamper of class index {class} must be detected"
        );
    }
}

#[test]
fn golden_lifecycle_raw_wire_receipt_is_not_a_commitment_passthrough() {
    // The codec remains hash-then-commit even under the cryptographic hasher:
    // the raw 32-byte wire receipt placed directly as a commitment changes the
    // sealed chain hash, and an all-zero value is structurally rejected.
    let receipt = seal();
    let honest_chain_hash = receipt.chain_hash();

    let mut events: Vec<Event<'_>> = receipt
        .events()
        .iter()
        .map(|e| Event {
            seq: e.seq,
            event_id: e.event_id.as_str(),
            event_type: e.event_type.as_str(),
            commitment: e.commitment,
        })
        .collect();

    // Raw passthrough: FUEL_EXHAUSTED's wire bytes as the commitment itself.
    events[1].commitment = Digest(FUEL_EXHAUSTED);
    match affidavit_core::verify::<Sha256>(&events, &honest_chain_hash, receipt.profile()) {
        Verdict::Reject(affidavit_core::RejectReason::ChainHashMismatch) => {}
        other => panic!("raw wire receipt as commitment must REJECT, got {other:?}"),
    }

    // All-zero commitment is refused structurally: hash the mutated event set
    // so the chain check passes and the zero-commitment refusal is the one
    // that fires (the verifier reports chain mismatch first on a stale hash).
    events[1].commitment = Digest([0u8; 32]);
    let zero_hash = affidavit_core::compute_chain_hash::<Sha256>(&events);
    assert_eq!(
        affidavit_core::verify::<Sha256>(&events, &zero_hash, receipt.profile()),
        Verdict::Reject(affidavit_core::RejectReason::ZeroCommitment { index: 1 }),
    );
}
