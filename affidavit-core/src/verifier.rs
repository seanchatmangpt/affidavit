//! The verifier: a total, allocation-free function from a borrowed chain to a
//! [`Verdict`].
//!
//! It condenses affidavit's 7-stage pipeline to the structural stages that are
//! meaningful in a pure core (no I/O, no JSON), in the same fail-fast order:
//!
//! | affidavit stage        | here |
//! |------------------------|------|
//! | 2 `check_format`       | profile tag matches [`PROFILE`] |
//! | 3 `chain_integrity`    | recomputed [`compute_chain_hash`] equals the stored hash |
//! | 4 `continuity`         | `seq` is `0..n`; `event_id`s are unique |
//! | 5 `verify_commitments` | every commitment is non-zero (well-formed) |
//! | 6 `evaluate_profile`   | every `event_type` is non-empty |
//!
//! Doctrine: *certify, don't decide.* The verifier reports whether the chain
//! conforms to the format; it never judges whether the recorded work was honest.

use crate::chain::{compute_chain_hash, Event, PROFILE};
use crate::digest::ChainHasher;

/// Why a receipt was rejected. The first failing stage wins (fail-fast).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RejectReason {
    /// The profile tag did not equal [`PROFILE`].
    WrongProfile,
    /// The recomputed chain hash did not match the stored one (tampering).
    ChainHashMismatch,
    /// `seq` was not contiguous from 0: event at `index` had `found`.
    SeqNotContiguous {
        /// Position in the event slice.
        index: usize,
        /// The out-of-place `seq` value found there.
        found: u64,
    },
    /// Two events shared an `event_id`.
    DuplicateEventId,
    /// An event had an empty `event_type`.
    EmptyEventType {
        /// Position of the offending event.
        index: usize,
    },
    /// An event's commitment was the all-zero (unset) digest.
    ZeroCommitment {
        /// Position of the offending event.
        index: usize,
    },
}

/// The verifier's verdict: `Accept`, or `Reject` with the first failure reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// Every stage passed.
    Accept,
    /// A stage failed; see the reason.
    Reject(RejectReason),
}

impl Verdict {
    /// True iff this is [`Verdict::Accept`].
    #[inline]
    pub fn is_accept(&self) -> bool {
        matches!(self, Verdict::Accept)
    }

    /// The rejection reason, if any.
    #[inline]
    pub fn reason(&self) -> Option<RejectReason> {
        match self {
            Verdict::Reject(r) => Some(*r),
            Verdict::Accept => None,
        }
    }
}

/// Verify a borrowed event slice + its expected `chain_hash` against `profile`.
///
/// `H` must be the [`ChainHasher`] that produced `chain_hash`. Allocation-free
/// and `no_std`: suitable for verifying receipts inside a WASM sandbox, an
/// embedded HSM, or an on-chain runtime.
pub fn verify<H: ChainHasher>(
    events: &[Event<'_>],
    chain_hash: &crate::digest::Digest,
    profile: &str,
) -> Verdict {
    // Stage 2 — format/profile.
    if profile != PROFILE {
        return Verdict::Reject(RejectReason::WrongProfile);
    }

    // Stage 3 — chain integrity (recompute the rolling hash; must match).
    let recomputed = compute_chain_hash::<H>(events);
    if &recomputed != chain_hash {
        return Verdict::Reject(RejectReason::ChainHashMismatch);
    }

    // Stage 4 — continuity: seq contiguous from 0.
    for (index, ev) in events.iter().enumerate() {
        if ev.seq != index as u64 {
            return Verdict::Reject(RejectReason::SeqNotContiguous {
                index,
                found: ev.seq,
            });
        }
    }
    // Stage 4 (cont.) — event_id uniqueness. O(n^2) but allocation-free, which
    // is the right trade for a no-alloc core verifying typically-small chains.
    for i in 0..events.len() {
        for j in (i + 1)..events.len() {
            if events[i].event_id == events[j].event_id {
                return Verdict::Reject(RejectReason::DuplicateEventId);
            }
        }
    }

    // Stage 5 — commitments well-formed (non-zero).
    for (index, ev) in events.iter().enumerate() {
        if ev.commitment.is_zero() {
            return Verdict::Reject(RejectReason::ZeroCommitment { index });
        }
    }

    // Stage 6 — profile content: every event_type is non-empty.
    for (index, ev) in events.iter().enumerate() {
        if ev.event_type.is_empty() {
            return Verdict::Reject(RejectReason::EmptyEventType { index });
        }
    }

    Verdict::Accept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::{Digest, Fnv256};

    fn ev<'a>(seq: u64, id: &'a str, ty: &'a str, commitment: Digest) -> Event<'a> {
        Event {
            seq,
            event_id: id,
            event_type: ty,
            commitment,
        }
    }

    fn hash(events: &[Event<'_>]) -> Digest {
        crate::chain::compute_chain_hash::<Fnv256>(events)
    }

    fn accept(events: &[Event<'_>]) -> Verdict {
        verify::<Fnv256>(events, &hash(events), PROFILE)
    }

    #[test]
    fn well_formed_chain_accepts() {
        let events = [
            ev(0, "evt-0", "build", Digest([1u8; 32])),
            ev(1, "evt-1", "test", Digest([2u8; 32])),
        ];
        assert_eq!(accept(&events), Verdict::Accept);
    }

    #[test]
    fn empty_chain_accepts() {
        assert_eq!(accept(&[]), Verdict::Accept);
    }

    #[test]
    fn wrong_profile_rejects_even_with_bad_hash() {
        // Stage 2 fires before stage 3: profile wins even when the hash is
        // also wrong.
        let events = [ev(0, "evt-0", "build", Digest([1u8; 32]))];
        let wrong_hash = crate::digest::Digest([0xFF; 32]);
        assert_eq!(
            verify::<Fnv256>(&events, &wrong_hash, "core/v0"),
            Verdict::Reject(RejectReason::WrongProfile),
        );
    }

    #[test]
    fn hash_mismatch_rejects_before_continuity() {
        // Stage 3 fires before stage 4: a chain with both a bad hash and a
        // broken seq reports the hash first.
        let events = [
            ev(0, "evt-0", "build", Digest([1u8; 32])),
            ev(7, "evt-1", "test", Digest([2u8; 32])),
        ];
        let bad_hash = crate::digest::Digest([0xEE; 32]);
        assert_eq!(
            verify::<Fnv256>(&events, &bad_hash, PROFILE),
            Verdict::Reject(RejectReason::ChainHashMismatch),
        );
    }

    #[test]
    fn seq_continuity_fires_before_duplicate_ids() {
        // Stage 4a before stage 4b: bad seq AND duplicate ids -> seq reason.
        let events = [
            ev(0, "dup", "build", Digest([1u8; 32])),
            ev(5, "dup", "test", Digest([2u8; 32])),
        ];
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::SeqNotContiguous { index: 1, found: 5 }),
        );
    }

    #[test]
    fn duplicate_event_id_rejects_non_adjacent_pair() {
        // Uniqueness must survive adjacency: dups at positions 0 and 2.
        let events = [
            ev(0, "dup", "build", Digest([1u8; 32])),
            ev(1, "mid", "test", Digest([2u8; 32])),
            ev(2, "dup", "audit", Digest([3u8; 32])),
        ];
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::DuplicateEventId),
        );
    }

    #[test]
    fn duplicate_event_id_fires_before_commitment_stage() {
        // Stage 4b before stage 5: duplicate ids AND a zero commitment.
        let events = [
            ev(0, "dup", "build", Digest([1u8; 32])),
            ev(1, "dup", "test", Digest::ZERO),
        ];
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::DuplicateEventId),
        );
    }

    #[test]
    fn zero_commitment_fires_before_empty_event_type() {
        // Stage 5 before stage 6, per the documented stage order.
        let events = [ev(0, "evt-0", "", Digest::ZERO)];
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::ZeroCommitment { index: 0 }),
        );
    }

    #[test]
    fn zero_commitment_reports_first_offending_index() {
        let events = [
            ev(0, "evt-0", "build", Digest([1u8; 32])),
            ev(1, "evt-1", "test", Digest::ZERO),
        ];
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::ZeroCommitment { index: 1 }),
        );
    }

    #[test]
    fn empty_event_type_reports_first_offending_index() {
        let events = [
            ev(0, "evt-0", "build", Digest([1u8; 32])),
            ev(1, "evt-1", "", Digest([2u8; 32])),
        ];
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::EmptyEventType { index: 1 }),
        );
    }

    #[test]
    fn event_id_uniqueness_spans_beyond_adjacent_pairs() {
        // The O(n^2) uniqueness sweep compares every pair, not just neighbors:
        // the collision is at positions 0 and 11 (maximally far apart).
        let ids = [
            "dup", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "dup",
        ];
        let events: alloc::vec::Vec<Event<'_>> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| ev(i as u64, id, "build", Digest([i as u8 + 1; 32])))
            .collect();
        assert_eq!(
            verify::<Fnv256>(&events, &hash(&events), PROFILE),
            Verdict::Reject(RejectReason::DuplicateEventId),
        );
    }

    #[test]
    fn uniqueness_court_is_not_vacuous_same_chain_unique_ids_accepts() {
        // Positive control for the court above: identical chain with the first
        // id made unique must Accept.
        let ids = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "dup"];
        let events: alloc::vec::Vec<Event<'_>> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| ev(i as u64, id, "build", Digest([i as u8 + 1; 32])))
            .collect();
        assert_eq!(accept(&events), Verdict::Accept);
    }
}
