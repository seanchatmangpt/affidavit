//! The receipt chain: events, the rolling hash, and the sealed [`Receipt`].
//!
//! The borrowed [`Event`] type and [`compute_chain_hash`] are `no_std` +
//! no-`alloc`. The owned [`Receipt`]/[`ChainBuilder`] (which accumulate events)
//! live behind the `alloc` feature.

use crate::digest::{ChainHasher, Digest};

/// Format/profile tag for this chain construction. Mirrors affidavit's
/// `core/v1`; the verifier rejects receipts whose profile does not match.
pub const PROFILE: &str = "affidavit-core/v1";

/// Domain-separation tag absorbed before the first event, so an empty chain has
/// a well-defined, construction-specific hash.
const DOMAIN: &[u8] = b"affidavit-core/chain/v1";

/// A single operation-event, borrowed (zero-allocation) form.
///
/// The caller owns the backing storage for `event_id` / `event_type`; an
/// `Event` just borrows it. This is what the verifier consumes, so verification
/// never needs to allocate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Event<'a> {
    /// Monotonic sequence number; the verifier requires these be `0..n`.
    pub seq: u64,
    /// Unique-within-chain identifier (e.g. `"evt-0"`).
    pub event_id: &'a str,
    /// Operation kind (e.g. `"build"`, `"test"`); must be non-empty.
    pub event_type: &'a str,
    /// Content-address commitment for the event payload; must be non-zero.
    pub commitment: Digest,
}

impl<'a> Event<'a> {
    /// Absorb this event's *canonical* bytes into a chain-hash state.
    ///
    /// Fields are length-prefixed so distinct field boundaries can never alias
    /// (e.g. `("ab","c")` and `("a","bc")` hash differently).
    #[inline]
    pub fn absorb_into<H: ChainHasher>(&self, state: &mut H::State) {
        H::absorb(state, &self.seq.to_be_bytes());
        H::absorb(state, &(self.event_id.len() as u64).to_be_bytes());
        H::absorb(state, self.event_id.as_bytes());
        H::absorb(state, &(self.event_type.len() as u64).to_be_bytes());
        H::absorb(state, self.event_type.as_bytes());
        H::absorb(state, self.commitment.as_bytes());
    }
}

/// Fold a sequence of events into a single chain hash via a per-link rolling
/// construction: `acc_0 = H(DOMAIN)`, then `acc_i = H(acc_{i-1} || event_i)`.
///
/// Because each link mixes in the previous accumulator, **any edit to any event
/// propagates through every later link** — the property the verifier relies on.
/// Zero-allocation and `no_std`.
pub fn compute_chain_hash<H: ChainHasher>(events: &[Event<'_>]) -> Digest {
    let mut acc = {
        let mut s = H::init();
        H::absorb(&mut s, DOMAIN);
        H::finish(s)
    };
    for ev in events {
        let mut s = H::init();
        H::absorb(&mut s, acc.as_bytes());
        ev.absorb_into::<H>(&mut s);
        acc = H::finish(s);
    }
    acc
}

// ---------------------------------------------------------------------------
// Owned builder + sealed receipt (requires `alloc`)
// ---------------------------------------------------------------------------

#[cfg(feature = "alloc")]
mod owned {
    use super::{compute_chain_hash, ChainHasher, Digest, Event, PROFILE};
    use crate::verifier::{verify, Verdict};
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::marker::PhantomData;

    /// Private witness that a [`Receipt`] came through [`ChainBuilder::finalize`].
    ///
    /// It has no public constructor, so a `Receipt` literal cannot be written by
    /// hand outside this module — the compile-time seal. (See the `compile_fail`
    /// doctest in the crate root.)
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Seal(());

    /// A single operation-event in owned (heap-backed) form.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct OwnedEvent {
        /// Monotonic sequence number.
        pub seq: u64,
        /// Unique-within-chain identifier.
        pub event_id: String,
        /// Operation kind.
        pub event_type: String,
        /// Content-address commitment.
        pub commitment: Digest,
    }

    impl OwnedEvent {
        /// Borrow as a zero-copy [`Event`].
        #[inline]
        pub fn borrow(&self) -> Event<'_> {
            Event {
                seq: self.seq,
                event_id: &self.event_id,
                event_type: &self.event_type,
                commitment: self.commitment,
            }
        }
    }

    /// A **sealed, immutable** receipt.
    ///
    /// The only way to obtain one is [`ChainBuilder::finalize`]; the private
    /// `_seal` field makes struct-literal construction a compile error
    /// (`E0451`), so a `Receipt` you hold is guaranteed to have a `chain_hash`
    /// that was actually computed from its events — never hand-forged.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Receipt {
        events: Vec<OwnedEvent>,
        chain_hash: Digest,
        profile: &'static str,
        _seal: Seal,
    }

    impl Receipt {
        /// The chain's events, in order.
        #[inline]
        pub fn events(&self) -> &[OwnedEvent] {
            &self.events
        }

        /// The sealed chain hash.
        #[inline]
        pub fn chain_hash(&self) -> Digest {
            self.chain_hash
        }

        /// The profile tag (always [`PROFILE`] for receipts built in-crate).
        #[inline]
        pub fn profile(&self) -> &str {
            self.profile
        }

        /// Run the full verifier over this receipt.
        ///
        /// `H` must be the same [`ChainHasher`] used to build it, or the chain
        /// integrity stage will (correctly) report a mismatch.
        pub fn verify<H: ChainHasher>(&self) -> Verdict {
            let borrowed: Vec<Event<'_>> = self.events.iter().map(OwnedEvent::borrow).collect();
            verify::<H>(&borrowed, &self.chain_hash, self.profile)
        }
    }

    /// Typestate builder that accumulates events and seals them into a
    /// [`Receipt`]. Parameterized by the [`ChainHasher`] used for the chain hash.
    pub struct ChainBuilder<H: ChainHasher> {
        events: Vec<OwnedEvent>,
        _hasher: PhantomData<H>,
    }

    impl<H: ChainHasher> Default for ChainBuilder<H> {
        fn default() -> Self {
            Self {
                events: Vec::new(),
                _hasher: PhantomData,
            }
        }
    }

    impl<H: ChainHasher> ChainBuilder<H> {
        /// Start an empty builder.
        pub fn new() -> Self {
            Self::default()
        }

        /// Append an event. `seq` is assigned automatically as the current
        /// length, guaranteeing contiguity by construction.
        pub fn event(mut self, event_type: &str, event_id: &str, commitment: Digest) -> Self {
            let seq = self.events.len() as u64;
            self.events.push(OwnedEvent {
                seq,
                event_id: String::from(event_id),
                event_type: String::from(event_type),
                commitment,
            });
            self
        }

        /// Seal the accumulated events into an immutable [`Receipt`], computing
        /// the chain hash. This is the *only* constructor for [`Receipt`].
        pub fn finalize(self) -> Receipt {
            let chain_hash = {
                let borrowed: Vec<Event<'_>> = self.events.iter().map(OwnedEvent::borrow).collect();
                compute_chain_hash::<H>(&borrowed)
            };
            Receipt {
                events: self.events,
                chain_hash,
                profile: PROFILE,
                _seal: Seal(()),
            }
        }
    }
}

#[cfg(feature = "alloc")]
pub use owned::{ChainBuilder, OwnedEvent, Receipt, Seal};
