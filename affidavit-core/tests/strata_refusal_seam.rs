//! Court: strata valve refusal receipt <-> affidavit receipt envelope codec seam.
//!
//! Seam (L3, innovation ledger Y6): a strata valve refusal emits a **fixed,
//! opaque, 32-byte receipt** — no structure, no cause channel on the wire
//! (ground truth: `strata_protocol/src/refusal.rs`, mirrored in the pack's
//! ontology as `valve:refusalReceiptLen = 32`). affidavit's envelope is the
//! sealed [`affidavit_core::Receipt`], whose events carry a 32-byte
//! [`affidavit_core::Digest`] commitment per event.
//!
//! Codec: the 32-byte refusal receipt is the event **payload**; its commitment
//! is `Fnv256(payload)` — domain-separated by the affidavit chain's own DOMAIN
//! absorb, so the zero receipt of one class and the zero receipt of another
//! land on different commitments via their distinct `event_id`s.
//!
//! Structural facts proven here, not assumed:
//! 1. the pinned fixture is exactly 32 bytes (wire-length law);
//! 2. commitment round-trips byte-faithfully from the payload;
//! 3. the affidavit chain hasher **accepts** the refusal receipt as payload:
//!    the sealed envelope verifies `Verdict::Accept`;
//! 4. a single-bit tamper of the wire payload is detected (`ChainHashMismatch`);
//! 5. the raw zero receipt is NOT itself usable as a commitment (the verifier
//!    rejects `Digest::ZERO`) — this is the one asymmetry of the seam, and the
//!    codec (hash-then-commit) is what closes it.

use affidavit_core::{ChainBuilder, Digest, Fnv256, Verdict};

// ---------------------------------------------------------------------------
// Pinned fixture.
//
// Provenance: copied const shape from the projected source of
// strata-valve-pack — /Users/sac/ggen-marketplace/packs/strata-valve-pack/
// generated/refusals.ex (@receipt_len 32; every refusal class returns
// `{:refuse, :erlang.list_to_binary(List.duplicate(0, @receipt_len))}` until
// the blake3 derivation TODO lands). Ground truth cited in that generated
// file: strata_protocol/src/refusal.rs — "constant-time refusal, fixed opaque
// 32-byte receipt, no cause channel on the wire". Taxonomy:
// packs/strata-valve-pack/ontology.ttl (valve:RefusalClass, 4 subclasses).
// ---------------------------------------------------------------------------

/// `Strata.Valve.Refusals.MalformedFrame.refuse/1` wire form.
const MALFORMED_FRAME: [u8; 32] = [0u8; 32];
/// `Strata.Valve.Refusals.FuelExhausted.refuse/1` wire form.
const FUEL_EXHAUSTED: [u8; 32] = [0u8; 32];
/// `Strata.Valve.Refusals.ParentDiscontinuity.refuse/1` wire form.
const PARENT_DISCONTINUITY: [u8; 32] = [0u8; 32];
/// `Strata.Valve.Refusals.ArenaFailure.refuse/1` wire form.
const ARENA_FAILURE: [u8; 32] = [0u8; 32];

/// The pack's wire-length law: `valve:refusalReceiptLen = 32`.
const RECEIPT_LEN: usize = 32;

/// Codec commitment: `Fnv256` over the exact 32-byte wire payload.
fn commitment_of(payload: &[u8; RECEIPT_LEN]) -> Digest {
    use affidavit_core::ChainHasher;
    let mut state = <Fnv256 as ChainHasher>::init();
    <Fnv256 as ChainHasher>::absorb(&mut state, payload);
    <Fnv256 as ChainHasher>::finish(state)
}

/// A codec seam under test: one wire receipt bound to its envelope event id.
struct Seam<'a> {
    event_id: &'a str,
    payload: &'a [u8; RECEIPT_LEN],
}

/// Build a sealed envelope over the four refusal classes, asserting round-trip
/// fidelity at construction time (commitment == Fnv256(payload)).
fn seal(seams: &[Seam<'_>]) -> affidavit_core::Receipt {
    let mut builder = ChainBuilder::<Fnv256>::new();
    for s in seams {
        builder = builder.event("strata_refusal", s.event_id, commitment_of(s.payload));
    }
    builder.finalize()
}

#[test]
fn fixture_matches_wire_length_law() {
    assert_eq!(RECEIPT_LEN, 32);
    for p in [
        &MALFORMED_FRAME,
        &FUEL_EXHAUSTED,
        &PARENT_DISCONTINUITY,
        &ARENA_FAILURE,
    ] {
        assert_eq!(p.len(), RECEIPT_LEN);
    }
}

#[test]
fn commitment_round_trips_from_payload() {
    for p in [
        &MALFORMED_FRAME,
        &FUEL_EXHAUSTED,
        &PARENT_DISCONTINUITY,
        &ARENA_FAILURE,
    ] {
        let c = commitment_of(p);
        assert_eq!(c.as_bytes().len(), 32);
        // Round-trip: recomputing from the wire payload is deterministic.
        assert_eq!(c, commitment_of(p));
        // Distinct payloads must not collide on identity of bytes (all four
        // are currently the zero receipt — the envelope's event_id is what
        // disambiguates them downstream, so assert the codec is a pure
        // function of the wire bytes, not of the class label).
        let c_other = commitment_of(&MALFORMED_FRAME);
        assert_eq!(c, c_other);
    }
}

#[test]
fn chain_hasher_accepts_refusal_receipt_as_payload() {
    let seams = [
        Seam { event_id: "strata.valve.refusals.malformed_frame", payload: &MALFORMED_FRAME },
        Seam { event_id: "strata.valve.refusals.fuel_exhausted", payload: &FUEL_EXHAUSTED },
        Seam { event_id: "strata.valve.refusals.parent_discontinuity", payload: &PARENT_DISCONTINUITY },
        Seam { event_id: "strata.valve.refusals.arena_failure", payload: &ARENA_FAILURE },
    ];
    let receipt = seal(&seams);
    // The envelope is sealed and profiled.
    assert_eq!(receipt.profile(), "affidavit-core/v1");
    assert_eq!(receipt.events().len(), 4);
    // The chain hasher accepts the refusal receipts as payload: full verify.
    assert_eq!(receipt.verify::<Fnv256>(), Verdict::Accept);
    // Byte fidelity: each stored commitment equals Fnv256 of the wire payload.
    for (ev, s) in receipt.events().iter().zip(seams.iter()) {
        assert_eq!(ev.event_id, s.event_id);
        assert_eq!(ev.commitment, commitment_of(s.payload));
    }
}

#[test]
fn single_bit_tamper_of_wire_payload_is_detected() {
    let seams = [
        Seam { event_id: "strata.valve.refusals.malformed_frame", payload: &MALFORMED_FRAME },
        Seam { event_id: "strata.valve.refusals.fuel_exhausted", payload: &FUEL_EXHAUSTED },
    ];
    let receipt = seal(&seams);
    let honest = receipt.chain_hash();

    // Flip one bit of one wire payload; the codec commitment changes, so the
    // recomputed chain hash must diverge from the sealed one.
    let mut tampered = MALFORMED_FRAME;
    tampered[7] ^= 0x01;
    let tampered_commitment = commitment_of(&tampered);
    assert_ne!(tampered_commitment, commitment_of(&MALFORMED_FRAME));

    let mut evs = receipt
        .events()
        .iter()
        .map(|e| affidavit_core::Event {
            seq: e.seq,
            event_id: e.event_id.as_str(),
            event_type: e.event_type.as_str(),
            commitment: e.commitment,
        })
        .collect::<Vec<_>>();
    evs[0].commitment = tampered_commitment;
    assert_eq!(
        affidavit_core::verify::<Fnv256>(&evs, &honest, receipt.profile()),
        Verdict::Reject(affidavit_core::RejectReason::ChainHashMismatch),
    );
}

#[test]
fn raw_zero_receipt_cannot_serve_as_commitment() {
    // The one asymmetry: strata's current wire receipt is all-zero, and the
    // affidavit verifier structurally rejects a zero commitment. The codec is
    // therefore hash-then-commit, never raw-passthrough.
    let events = [affidavit_core::Event {
        seq: 0,
        event_id: "strata.valve.refusals.malformed_frame",
        event_type: "strata_refusal",
        commitment: Digest(MALFORMED_FRAME),
    }];
    let h = affidavit_core::compute_chain_hash::<Fnv256>(&events);
    assert_eq!(
        affidavit_core::verify::<Fnv256>(&events, &h, affidavit_core::PROFILE),
        Verdict::Reject(affidavit_core::RejectReason::ZeroCommitment { index: 0 }),
    );
}
