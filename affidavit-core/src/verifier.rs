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

    // Stages 5 & 6 — commitments well-formed (non-zero) and profile content
    // (non-empty event_type).
    for (index, ev) in events.iter().enumerate() {
        if ev.event_type.is_empty() {
            return Verdict::Reject(RejectReason::EmptyEventType { index });
        }
        if ev.commitment.is_zero() {
            return Verdict::Reject(RejectReason::ZeroCommitment { index });
        }
    }

    Verdict::Accept
}
