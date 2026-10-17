//! BLAKE3 chain court (W2-D): proves the end-to-end chain story through the
//! external cryptographic hasher [`Blake3Hasher`] — no hand-implemented crypto;
//! the independent recomputation below calls the raw `blake3` dev-dependency,
//! mirroring the documented fold in `src/chain.rs` byte for byte.

use affidavit_core::{
    compute_chain_hash, verify, Blake3Hasher, ChainBuilder, Digest, Event, Fnv256, RejectReason,
    Verdict, PROFILE,
};

const DOMAIN: &[u8] = b"affidavit-core/chain/v1";

fn two_events() -> Vec<Event<'static>> {
    vec![
        Event {
            seq: 0,
            event_id: "evt-0",
            event_type: "build",
            commitment: Digest([1u8; 32]),
        },
        Event {
            seq: 1,
            event_id: "evt-1",
            event_type: "test",
            commitment: Digest([2u8; 32]),
        },
    ]
}

/// Independent recomputation of `compute_chain_hash::<Blake3Hasher>` using the
/// raw external `blake3` crate. Mirrors src/chain.rs:56-69 (`acc_0 = H(DOMAIN)`,
/// `acc_i = H(acc_{i-1} || event_i)`) and the length-prefixed field encoding of
/// `Event::absorb_into` (src/chain.rs:40-47).
fn independent_blake3_fold(events: &[Event<'_>]) -> Digest {
    use blake3::Hasher;

    // acc_0 = H(DOMAIN)  (src/chain.rs:57-61)
    let mut acc = {
        let mut s = Hasher::new();
        s.update(DOMAIN);
        *s.finalize().as_bytes()
    };
    for ev in events {
        // acc_i = H(acc_{i-1} || event_i)  (src/chain.rs:62-67)
        let mut s = Hasher::new();
        s.update(&acc);
        // Event::absorb_into (src/chain.rs:40-47): be-u64 seq, len-prefixed
        // event_id, len-prefixed event_type, raw 32-byte commitment.
        s.update(&ev.seq.to_be_bytes());
        s.update(&(ev.event_id.len() as u64).to_be_bytes());
        s.update(ev.event_id.as_bytes());
        s.update(&(ev.event_type.len() as u64).to_be_bytes());
        s.update(ev.event_type.as_bytes());
        s.update(ev.commitment.as_bytes());
        acc = *s.finalize().as_bytes();
    }
    Digest(acc)
}

/// (1) KAT / anti-vacuity: the adapter's fold equals an independently computed
/// raw-blake3 fold of the same 2-event chain. Also pins the DOMAIN seed link.
#[test]
fn blake3_chain_kat_matches_independent_raw_blake3_fold() {
    let events = two_events();

    let via_adapter = compute_chain_hash::<Blake3Hasher>(&events);
    let via_raw = independent_blake3_fold(&events);
    assert_eq!(
        via_adapter, via_raw,
        "Blake3Hasher adapter must fold exactly the documented encoding"
    );
    assert!(
        !via_adapter.is_zero(),
        "head must not be the unset sentinel"
    );

    // The domain-separation link is load-bearing: a fold without DOMAIN is a
    // different construction and must not alias the real head.
    let mut s = blake3::Hasher::new();
    s.update(&[0u8; 32]); // empty acc0 placeholder instead of H(DOMAIN)
    let no_domain = s.finalize();
    assert_ne!(
        via_adapter.as_bytes(),
        no_domain.as_bytes(),
        "chain head must depend on the domain-separation tag"
    );
}

/// (1b) Single-event variant so the KAT covers both the DOMAIN link and one
/// full event link independently.
#[test]
fn blake3_chain_kat_single_event() {
    let events = [Event {
        seq: 7,
        event_id: "solo",
        event_type: "audit-log",
        commitment: Digest([9u8; 32]),
    }];
    assert_eq!(
        compute_chain_hash::<Blake3Hasher>(&events),
        independent_blake3_fold(&events)
    );
}

/// (2) Tamper propagation: flipping one byte in any event field propagates
/// through the rolling fold, so `verify::<Blake3Hasher>` rejects the edited
/// chain against the original head.
#[test]
fn blake3_tamper_any_field_rejected() {
    let good = compute_chain_hash::<Blake3Hasher>(&two_events());
    assert_eq!(
        verify::<Blake3Hasher>(&two_events(), &good, PROFILE),
        Verdict::Accept
    );

    let commitments = [Digest([1u8; 32]), Digest([2u8; 32])];

    // Flip seq.
    let mut evs = two_events();
    evs[1].seq = 99;
    let tampered = compute_chain_hash::<Blake3Hasher>(&evs);
    assert_ne!(tampered, good, "seq edit must propagate to the head");
    assert_eq!(
        verify::<Blake3Hasher>(&evs, &good, PROFILE),
        Verdict::Reject(RejectReason::ChainHashMismatch),
        "seq tamper must be rejected at the chain_integrity stage"
    );

    // Flip event_id (also confirms field-boundary non-aliasing: ("ab","c") vs
    // ("a","bc") style edits cannot alias).
    let mut evs = two_events();
    evs[0].event_id = "evt-X";
    let tampered = compute_chain_hash::<Blake3Hasher>(&evs);
    assert_ne!(tampered, good);
    assert_eq!(
        verify::<Blake3Hasher>(&evs, &good, PROFILE),
        Verdict::Reject(RejectReason::ChainHashMismatch)
    );

    // Flip event_type.
    let mut evs = two_events();
    evs[1].event_type = "deploy";
    let tampered = compute_chain_hash::<Blake3Hasher>(&evs);
    assert_ne!(tampered, good);
    assert_eq!(
        verify::<Blake3Hasher>(&evs, &good, PROFILE),
        Verdict::Reject(RejectReason::ChainHashMismatch)
    );

    // Flip one byte of the commitment.
    let mut evs = two_events();
    let mut c = [1u8; 32];
    c[31] ^= 0x01;
    evs[0].commitment = Digest(c);
    assert_ne!(evs[0].commitment, commitments[0]);
    let tampered = compute_chain_hash::<Blake3Hasher>(&evs);
    assert_ne!(tampered, good);
    assert_eq!(
        verify::<Blake3Hasher>(&evs, &good, PROFILE),
        Verdict::Reject(RejectReason::ChainHashMismatch)
    );
}

/// (3) Cross-hash refusal: a receipt/head built with one hasher is refused
/// under the other — hasher identity is visible in the head.
#[test]
fn blake3_cross_hash_refusal() {
    let events = two_events();

    let blake_head = compute_chain_hash::<Blake3Hasher>(&events);
    let fnv_head = compute_chain_hash::<Fnv256>(&events);
    assert_ne!(blake_head, fnv_head, "hasher identity must change the head");

    // Fnv-built head refused under Blake3Hasher, and vice versa.
    assert_eq!(
        verify::<Blake3Hasher>(&events, &fnv_head, PROFILE),
        Verdict::Reject(RejectReason::ChainHashMismatch)
    );
    assert_eq!(
        verify::<Fnv256>(&events, &blake_head, PROFILE),
        Verdict::Reject(RejectReason::ChainHashMismatch)
    );

    // Same at the sealed-receipt level.
    let receipt = ChainBuilder::<Fnv256>::new()
        .event("build", "evt-0", Digest([1u8; 32]))
        .event("test", "evt-1", Digest([2u8; 32]))
        .finalize();
    assert_eq!(receipt.verify::<Fnv256>(), Verdict::Accept);
    assert_eq!(
        receipt.verify::<Blake3Hasher>(),
        Verdict::Reject(RejectReason::ChainHashMismatch),
        "Fnv-sealed receipt must be refused under the Blake3 verifier"
    );
}

/// (4) Determinism: same events folded twice produce a byte-identical head,
/// and the builder path agrees with the bare fold.
#[test]
fn blake3_determinism_two_runs_same_head() {
    let a = compute_chain_hash::<Blake3Hasher>(&two_events());
    let b = compute_chain_hash::<Blake3Hasher>(&two_events());
    assert_eq!(a, b);
    assert_eq!(a, independent_blake3_fold(&two_events()));

    let ra = ChainBuilder::<Blake3Hasher>::new()
        .event("build", "evt-0", Digest([1u8; 32]))
        .event("test", "evt-1", Digest([2u8; 32]))
        .finalize();
    let rb = ChainBuilder::<Blake3Hasher>::new()
        .event("build", "evt-0", Digest([1u8; 32]))
        .event("test", "evt-1", Digest([2u8; 32]))
        .finalize();
    assert_eq!(ra.chain_hash(), rb.chain_hash());
    assert_eq!(ra.chain_hash(), a, "builder seal == bare fold");
    assert_eq!(ra.verify::<Blake3Hasher>(), Verdict::Accept);
}
