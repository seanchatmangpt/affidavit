//! # affidavit-core
//!
//! A **zero-dependency**, `#![no_std]`, `#![forbid(unsafe_code)]` reference
//! verifier for affidavit-style receipt chains.
//!
//! The premise: the component you *verify provenance with* should be the hardest
//! thing in the system to compromise. So this core has:
//!
//! * **No dependencies.** Nothing to audit, nothing to be broken by an upstream
//!   release. (Pointedly: the sibling `affidavit` crate currently does not
//!   compile because one published dependency, `wasm4pm-compat 26.6.13`, fails
//!   under nightly. A verifier with an empty `[dependencies]` cannot fail that
//!   way.)
//! * **No `std`, no `alloc` on the verify path.** [`verify`] runs over a
//!   borrowed `&[Event]` and allocates nothing, so it works in a WASM sandbox,
//!   an embedded HSM, or an on-chain runtime — not just on a server.
//! * **No `unsafe`.** `forbid(unsafe_code)` is enforced crate-wide.
//! * **Invalid receipts are unrepresentable.** A [`Receipt`] is only obtainable
//!   from [`ChainBuilder::finalize`]; its sealed state cannot be forged by hand
//!   (see the `compile_fail` example below).
//! * **A pluggable hash.** The chain folds through any [`ChainHasher`]; a
//!   zero-dep reference hasher [`Fnv256`] ships in-crate, and a cryptographic
//!   hash (e.g. BLAKE3) can be slotted in for a tamper-*evident* chain.
//!
//! ## Build, seal, verify
//!
//! ```
//! use affidavit_core::{ChainBuilder, Digest, Fnv256, Verdict};
//!
//! let receipt = ChainBuilder::<Fnv256>::new()
//!     .event("build", "evt-0", Digest([1u8; 32]))
//!     .event("test", "evt-1", Digest([2u8; 32]))
//!     .finalize();
//!
//! // A freshly sealed receipt always verifies.
//! assert_eq!(receipt.verify::<Fnv256>(), Verdict::Accept);
//! assert_eq!(receipt.events().len(), 2);
//! ```
//!
//! ## The seal is enforced at compile time
//!
//! You cannot construct a [`Receipt`] by hand — its witness field is private, so
//! the only `chain_hash` a `Receipt` can carry is one that was actually computed
//! from its events:
//!
//! ```compile_fail
//! use affidavit_core::{Digest, Receipt};
//! // ERROR (E0451): field `_seal` of struct `Receipt` is private.
//! let forged = Receipt {
//!     events: Vec::new(),
//!     chain_hash: Digest::ZERO,
//!     profile: "affidavit-core/v1",
//!     _seal: (),
//! };
//! ```

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod chain;
pub mod digest;
pub mod verifier;

#[cfg(feature = "alloc")]
pub mod mining;

pub use chain::{compute_chain_hash, Event, PROFILE};
pub use digest::{ChainHasher, Digest, Fnv256};
pub use verifier::{verify, RejectReason, Verdict};

#[cfg(feature = "alloc")]
pub use chain::{ChainBuilder, OwnedEvent, Receipt};

#[cfg(feature = "alloc")]
pub use mining::{DirectlyFollowsGraph, Trace};

// ---------------------------------------------------------------------------
// Tests (run under std; the lib itself is no_std in non-test builds).
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Tiny deterministic PRNG for property tests (no external dependency).
    struct XorShift64(u64);
    impl XorShift64 {
        fn new(seed: u64) -> Self {
            // Avoid the zero fixed-point.
            XorShift64(seed ^ 0x9e37_79b9_7f4a_7c15 | 1)
        }
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        fn digest(&mut self) -> Digest {
            let mut out = [0u8; 32];
            for chunk in out.chunks_mut(8) {
                chunk.copy_from_slice(&self.next().to_be_bytes());
            }
            // Guarantee non-zero so the well-formedness stage is satisfied.
            if out.iter().all(|&b| b == 0) {
                out[0] = 1;
            }
            Digest(out)
        }
    }

    fn sample_receipt() -> Receipt {
        ChainBuilder::<Fnv256>::new()
            .event("build", "evt-0", Digest([1u8; 32]))
            .event("test", "evt-1", Digest([2u8; 32]))
            .event("audit", "evt-2", Digest([3u8; 32]))
            .finalize()
    }

    #[test]
    fn sealed_receipt_accepts() {
        assert_eq!(sample_receipt().verify::<Fnv256>(), Verdict::Accept);
    }

    #[test]
    fn empty_chain_accepts() {
        let r = ChainBuilder::<Fnv256>::new().finalize();
        assert_eq!(r.verify::<Fnv256>(), Verdict::Accept);
        assert!(r.events().is_empty());
    }

    #[test]
    fn determinism_same_events_same_hash() {
        let a = sample_receipt();
        let b = sample_receipt();
        assert_eq!(a.chain_hash(), b.chain_hash());
    }

    #[test]
    fn distinct_events_distinct_hash() {
        let a = ChainBuilder::<Fnv256>::new()
            .event("build", "evt-0", Digest([1u8; 32]))
            .finalize();
        let b = ChainBuilder::<Fnv256>::new()
            .event("build", "evt-0", Digest([2u8; 32]))
            .finalize();
        assert_ne!(a.chain_hash(), b.chain_hash());
    }

    #[test]
    fn tampered_commitment_is_rejected() {
        let r = sample_receipt();
        let good = r.chain_hash();
        // Rebuild a borrowed view and flip a bit in one commitment.
        let mut events: alloc::vec::Vec<Event<'_>> =
            r.events().iter().map(OwnedEvent::borrow).collect();
        events[1].commitment.0[0] ^= 0x01;
        assert_eq!(
            verify::<Fnv256>(&events, &good, PROFILE),
            Verdict::Reject(RejectReason::ChainHashMismatch),
        );
    }

    #[test]
    fn reordered_events_are_rejected() {
        let r = sample_receipt();
        let good = r.chain_hash();
        let mut events: alloc::vec::Vec<Event<'_>> =
            r.events().iter().map(OwnedEvent::borrow).collect();
        events.swap(0, 2); // also scrambles seq, but integrity catches it first
                           // Integrity (stage 3) fires before continuity (stage 4).
        assert_eq!(
            verify::<Fnv256>(&events, &good, PROFILE),
            Verdict::Reject(RejectReason::ChainHashMismatch),
        );
    }

    #[test]
    fn wrong_profile_is_rejected() {
        let events = [Event {
            seq: 0,
            event_id: "evt-0",
            event_type: "build",
            commitment: Digest([1u8; 32]),
        }];
        let h = compute_chain_hash::<Fnv256>(&events);
        assert_eq!(
            verify::<Fnv256>(&events, &h, "core/v1"),
            Verdict::Reject(RejectReason::WrongProfile),
        );
    }

    #[test]
    fn noncontiguous_seq_is_rejected() {
        // Hash matches (so integrity passes) but seq jumps 0 -> 2.
        let events = [
            Event {
                seq: 0,
                event_id: "evt-0",
                event_type: "build",
                commitment: Digest([1u8; 32]),
            },
            Event {
                seq: 2,
                event_id: "evt-1",
                event_type: "test",
                commitment: Digest([2u8; 32]),
            },
        ];
        let h = compute_chain_hash::<Fnv256>(&events);
        assert_eq!(
            verify::<Fnv256>(&events, &h, PROFILE),
            Verdict::Reject(RejectReason::SeqNotContiguous { index: 1, found: 2 }),
        );
    }

    #[test]
    fn duplicate_event_id_is_rejected() {
        let events = [
            Event {
                seq: 0,
                event_id: "dup",
                event_type: "build",
                commitment: Digest([1u8; 32]),
            },
            Event {
                seq: 1,
                event_id: "dup",
                event_type: "test",
                commitment: Digest([2u8; 32]),
            },
        ];
        let h = compute_chain_hash::<Fnv256>(&events);
        assert_eq!(
            verify::<Fnv256>(&events, &h, PROFILE),
            Verdict::Reject(RejectReason::DuplicateEventId),
        );
    }

    #[test]
    fn empty_event_type_is_rejected() {
        let events = [Event {
            seq: 0,
            event_id: "evt-0",
            event_type: "",
            commitment: Digest([1u8; 32]),
        }];
        let h = compute_chain_hash::<Fnv256>(&events);
        assert_eq!(
            verify::<Fnv256>(&events, &h, PROFILE),
            Verdict::Reject(RejectReason::EmptyEventType { index: 0 }),
        );
    }

    #[test]
    fn zero_commitment_is_rejected() {
        let events = [Event {
            seq: 0,
            event_id: "evt-0",
            event_type: "build",
            commitment: Digest::ZERO,
        }];
        let h = compute_chain_hash::<Fnv256>(&events);
        assert_eq!(
            verify::<Fnv256>(&events, &h, PROFILE),
            Verdict::Reject(RejectReason::ZeroCommitment { index: 0 }),
        );
    }

    #[test]
    fn property_random_receipts_accept_then_tamper_rejects() {
        let types = ["build", "test", "audit", "deploy", "scan"];
        for seed in 0..500u64 {
            let mut rng = XorShift64::new(seed.wrapping_mul(0x2545_f491_4f6c_dd1d));
            let n = (rng.next() % 8) as usize; // 0..=7 events

            // Build owned backing so the borrowed Events have somewhere to point.
            let ids: alloc::vec::Vec<alloc::string::String> =
                (0..n).map(|i| alloc::format!("evt-{i}")).collect();
            let kinds: alloc::vec::Vec<&str> =
                (0..n).map(|_| types[(rng.next() % 5) as usize]).collect();
            let comms: alloc::vec::Vec<Digest> = (0..n).map(|_| rng.digest()).collect();

            let events: alloc::vec::Vec<Event<'_>> = (0..n)
                .map(|i| Event {
                    seq: i as u64,
                    event_id: &ids[i],
                    event_type: kinds[i],
                    commitment: comms[i],
                })
                .collect();

            let h = compute_chain_hash::<Fnv256>(&events);
            assert_eq!(
                verify::<Fnv256>(&events, &h, PROFILE),
                Verdict::Accept,
                "seed {seed}: honest receipt should accept",
            );

            // Tamper one commitment bit -> integrity must fail.
            if n > 0 {
                let mut tampered = events.clone();
                let k = (rng.next() as usize) % n;
                tampered[k].commitment.0[(rng.next() as usize) % 32] ^= 0x01;
                assert_eq!(
                    verify::<Fnv256>(&tampered, &h, PROFILE),
                    Verdict::Reject(RejectReason::ChainHashMismatch),
                    "seed {seed}: single-bit tamper should be detected",
                );
            }
        }
    }
}
