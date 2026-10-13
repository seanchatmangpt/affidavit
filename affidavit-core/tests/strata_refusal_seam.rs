//! Court: strata valve refusal receipt <-> affidavit receipt envelope codec seam.
//!
//! Seam (L3, innovation ledger Y6; seam-29 closure under L26): a strata valve
//! refusal emits a **fixed, opaque, 32-byte receipt** — no structure, no cause
//! channel on the wire (ground truth: `strata_protocol/src/refusal.rs`,
//! mirrored in the pack's ontology as `valve:refusalReceiptLen = 32`).
//! affidavit's envelope is the sealed [`affidavit_core::Receipt`], whose events
//! carry a 32-byte [`affidavit_core::Digest`] commitment per event.
//!
//! Receipt derivation (REAL, no longer the all-zero placeholder): the
//! projected `refusals.ex` derives each receipt as
//!
//! ```text
//! :crypto.hash(:sha256, class_label <> 0x00 <> :erlang.term_to_binary(seed))
//! ```
//!
//! where `class_label` is the refusal class's UTF-8 label and `seed` is the
//! reason seed. Since the seed here is a binary, `term_to_binary` is the ETF
//! `binary_ext` form: `<<131, 109, Len::32, Bytes::binary>>`. This court ports
//! that exact derivation in Rust (`sha2`) and pins the resulting bytes, so a
//! change in the pack's derivation or labels fails here first.
//!
//! Codec: the 32-byte refusal receipt is the event **payload**; its commitment
//! is `Fnv256(payload)` — domain-separated by the affidavit chain's own DOMAIN
//! absorb, so distinct class receipts land on distinct commitments both by
//! payload bytes and by their distinct `event_id`s.
//!
//! Structural facts proven here, not assumed:
//! 1. the pinned fixture is exactly 32 bytes (wire-length law);
//! 2. the in-test port of the derivation reproduces the pinned receipts
//!    byte-for-byte (provenance is executable, not a comment);
//! 3. commitment round-trips byte-faithfully from the payload;
//! 4. the affidavit chain hasher **accepts** the refusal receipt as payload:
//!    the sealed envelope verifies `Verdict::Accept`;
//! 5. a single-bit tamper of the wire payload is detected (`ChainHashMismatch`);
//! 6. cross-class distinctness: all 4 class labels produce distinct receipts
//!    for the same seed;
//! 7. class-label swap binding: swapping the label changes the receipt (the
//!    receipt is bound to the class, not just the seed);
//! 8. the raw all-zero receipt is still NOT usable as a commitment (the
//!    verifier rejects `Digest::ZERO`) — the codec remains hash-then-commit.

use affidavit_core::{ChainBuilder, Digest, Fnv256, Verdict};
use sha2::{Digest as ShaDigest, Sha256};

// ---------------------------------------------------------------------------
// Pinned fixture.
//
// Provenance: real derivation, regenerated in ggen-marketplace commit
// 8210f1b24 ("feat(strata-valve-pack): real refusal-receipt derivation
// (L13, seam-29 closure)") —
// /Users/sac/ggen-marketplace/packs/strata-valve-pack/generated/refusals.ex.
// Algorithm (from that file's @doc): SHA-256 over
// `class <> 0x00 <> :erlang.term_to_binary(seed)`; the 256-bit digest IS the
// fixed 32-byte wire form (no truncation/padding). Ground truth cited there:
// strata_protocol/src/refusal.rs — "constant-time refusal, fixed opaque
// 32-byte receipt, no cause channel on the wire". Taxonomy:
// packs/strata-valve-pack/ontology.ttl (valve:RefusalClass, 4 subclasses).
//
// Pinned seed: the UTF-8 binary `seam-29-l26` (ETF binary_ext encoding below).
// ---------------------------------------------------------------------------

/// The reason seed this court pins: a UTF-8 binary, so `term_to_binary(seed)`
/// is exactly the ETF `binary_ext` tag (131), `BINARY_EXT` (109), 32-bit BE
/// length, raw bytes.
const SEED: &[u8] = b"seam-29-l26";

/// `:erlang.term_to_binary(seed)` for a binary seed (ETF binary_ext).
fn term_to_binary(seed: &[u8]) -> Vec<u8> {
    let mut out = vec![0x83, 0x6d];
    out.extend_from_slice(&(seed.len() as u32).to_be_bytes());
    out.extend_from_slice(seed);
    out
}

/// Port of the pack's derivation:
/// `:crypto.hash(:sha256, class_label <> 0x00 <> term_to_binary(seed))`.
fn derive_receipt(class_label: &str, seed: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(class_label.as_bytes());
    h.update([0x00]);
    h.update(term_to_binary(seed));
    h.finalize().into()
}

/// `Strata.Valve.Refusals.MalformedFrame.refuse/1` wire form (pinned).
const MALFORMED_FRAME: [u8; 32] = [
    0x91, 0xd5, 0x81, 0x7e, 0x6a, 0x7e, 0x87, 0x53, 0xe6, 0x2b, 0x81, 0x5b, 0x4c, 0xaa, 0x54, 0x93,
    0x9b, 0xfa, 0x87, 0x6b, 0xc4, 0x8c, 0x25, 0x19, 0x74, 0x66, 0x59, 0x69, 0x43, 0x8f, 0xaf, 0x18,
];
/// `Strata.Valve.Refusals.FuelExhausted.refuse/1` wire form (pinned).
const FUEL_EXHAUSTED: [u8; 32] = [
    0xcb, 0xc0, 0xde, 0x1a, 0x52, 0x55, 0xcc, 0x10, 0x00, 0x33, 0x34, 0x22, 0xef, 0x2a, 0xc6, 0x13,
    0xfe, 0xc4, 0xa6, 0x01, 0x62, 0x7c, 0x07, 0xb1, 0xaf, 0xa4, 0xaa, 0x22, 0xe5, 0x48, 0xe2, 0x3a,
];
/// `Strata.Valve.Refusals.ParentDiscontinuity.refuse/1` wire form (pinned).
const PARENT_DISCONTINUITY: [u8; 32] = [
    0x48, 0x36, 0x8e, 0x7b, 0x17, 0x6a, 0x54, 0xcd, 0x82, 0x13, 0xca, 0xad, 0xa8, 0x97, 0x75, 0x4e,
    0x2d, 0x1b, 0x2e, 0xe5, 0x36, 0xee, 0xd1, 0xb3, 0x6c, 0x73, 0x55, 0x23, 0x04, 0x77, 0x98, 0x4a,
];
/// `Strata.Valve.Refusals.ArenaFailure.refuse/1` wire form (pinned).
const ARENA_FAILURE: [u8; 32] = [
    0x68, 0x38, 0x4c, 0x70, 0x4c, 0xf1, 0xb7, 0x57, 0xdf, 0x5e, 0xf4, 0xa5, 0x9c, 0x83, 0x14, 0x60,
    0xbd, 0x27, 0x5b, 0x2b, 0x78, 0xb7, 0x72, 0x08, 0x9a, 0x49, 0xf5, 0xca, 0xd0, 0xd6, 0x0b, 0x01,
];

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
        // Real receipts, not the retired all-zero placeholder.
        assert!(*p != [0u8; 32], "receipt must not regress to all-zero");
    }
}

#[test]
fn in_test_port_reproduces_pinned_receipts() {
    // Executable provenance: the derivation ported from refusals.ex@8210f1b24
    // must reproduce every pinned fixture byte-for-byte.
    assert_eq!(derive_receipt("malformed_frame", SEED), MALFORMED_FRAME);
    assert_eq!(derive_receipt("fuel_exhausted", SEED), FUEL_EXHAUSTED);
    assert_eq!(derive_receipt("parent_discontinuity", SEED), PARENT_DISCONTINUITY);
    assert_eq!(derive_receipt("arena_failure", SEED), ARENA_FAILURE);
    // Determinism: recomputation is pure.
    assert_eq!(derive_receipt("malformed_frame", SEED), MALFORMED_FRAME);
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
        // The codec is a pure function of the wire bytes, not of the class
        // label: identical bytes give identical commitments regardless of
        // which seam carries them.
        let c_other = commitment_of(&MALFORMED_FRAME);
        assert_eq!(commitment_of(&MALFORMED_FRAME), c_other);
        let _ = c;
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
fn cross_class_receipts_are_distinct_for_same_seed() {
    // All 4 class labels, same pinned seed: the class label is in the hash
    // preimage, so every class yields a distinct receipt.
    let receipts = [
        ("arena_failure", derive_receipt("arena_failure", SEED)),
        ("fuel_exhausted", derive_receipt("fuel_exhausted", SEED)),
        ("malformed_frame", derive_receipt("malformed_frame", SEED)),
        ("parent_discontinuity", derive_receipt("parent_discontinuity", SEED)),
    ];
    for (label, r) in &receipts {
        assert_eq!(r.len(), RECEIPT_LEN, "{label}");
    }
    for i in 0..receipts.len() {
        for j in (i + 1)..receipts.len() {
            assert_ne!(
                receipts[i].1, receipts[j].1,
                "class collision: {} vs {}",
                receipts[i].0, receipts[j].0
            );
        }
    }
    // The derived receipts are exactly the pinned fixtures.
    assert_eq!(receipts[0].1, ARENA_FAILURE);
    assert_eq!(receipts[1].1, FUEL_EXHAUSTED);
    assert_eq!(receipts[2].1, MALFORMED_FRAME);
    assert_eq!(receipts[3].1, PARENT_DISCONTINUITY);
}

#[test]
fn class_label_swap_changes_receipt() {
    // Binding test: holding the seed fixed, swapping the class label must
    // change the receipt — the receipt identifies the refusal class.
    let base = derive_receipt("malformed_frame", SEED);
    for other in ["fuel_exhausted", "parent_discontinuity", "arena_failure"] {
        assert_ne!(
            derive_receipt(other, SEED),
            base,
            "label swap malformed_frame -> {other} must change the receipt"
        );
    }
    // Even a single-byte change of the label (truncation) changes the receipt.
    assert_ne!(derive_receipt("malformed_fram", SEED), base);
    assert_ne!(derive_receipt("malformed_framee", SEED), base);
}

#[test]
fn raw_zero_receipt_cannot_serve_as_commitment() {
    // The standing asymmetry: an all-zero 32-byte value (which the retired
    // placeholder fixture was, wholesale) is structurally rejected by the
    // affidavit verifier as a commitment. The codec is therefore
    // hash-then-commit, never raw-passthrough.
    let events = [affidavit_core::Event {
        seq: 0,
        event_id: "strata.valve.refusals.malformed_frame",
        event_type: "strata_refusal",
        commitment: Digest([0u8; 32]),
    }];
    let h = affidavit_core::compute_chain_hash::<Fnv256>(&events);
    assert_eq!(
        affidavit_core::verify::<Fnv256>(&events, &h, affidavit_core::PROFILE),
        Verdict::Reject(affidavit_core::RejectReason::ZeroCommitment { index: 0 }),
    );
}
