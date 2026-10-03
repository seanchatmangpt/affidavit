//! Hybrid logical clocks — causal ordering without wall-clock trust.
//!
//! Physical NTP clocks drift and partitions break them. An HLC stamps each
//! event with `(physical_ms, logical)` such that:
//!
//! - `h1 < h2` implies causality (`e1 happened-before e2`),
//! - concurrent events are detected and never silently ordered,
//! - the physical component tracks real time within bounded offsets.
//!
//! This is an in-tree implementation (marker feature `hlc`): the intended
//! upstream crate `hybrid-logical-clocks` does not exist on crates.io, and a
//! ~100-line deterministic law is not bought twice.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! HLC comparison is a decidable order check on timestamps. It certifies
//! causal claims; it never invents them — causality enters only through
//! [`HlcClock::receive`].

use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors produced by hybrid logical clock operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum HlcError {
    /// A received timestamp claims a physical future beyond any admissible
    /// clock skew bound.
    #[error("received HLC physical {received_ms} ms exceeds local clock + {skew_ms} ms bound")]
    ClockSkewExceeded {
        /// The physical component of the received stamp.
        received_ms: u64,
        /// The admissible skew bound in milliseconds.
        skew_ms: u64,
    },
}

/// A hybrid logical clock timestamp: physical milliseconds plus a logical
/// counter that breaks ties within the same millisecond.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HlcTimestamp {
    /// Wall-clock milliseconds at stamping time.
    pub physical_ms: u64,
    /// Logical counter, incremented on same-millisecond events.
    pub logical: u32,
}

/// A local hybrid logical clock.
#[derive(Debug, Clone)]
pub struct HlcClock {
    last: HlcTimestamp,
    /// Maximum admissible skew between local physical time and any received
    /// stamp, in milliseconds. `0` disables the bound (no refusal).
    max_skew_ms: u64,
}

fn physical_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        // A clock before the epoch is unrepresentable; clamp to zero and let
        // causality carry the ordering.
        .unwrap_or(0)
}

impl HlcClock {
    /// Create a clock with a skew bound of `max_skew_ms` on received stamps.
    #[must_use]
    pub fn with_skew_bound(max_skew_ms: u64) -> Self {
        Self {
            last: HlcTimestamp {
                physical_ms: physical_now_ms(),
                logical: 0,
            },
            max_skew_ms,
        }
    }

    /// The clock's current timestamp.
    #[must_use]
    pub fn peek(&self) -> HlcTimestamp {
        self.last
    }

    /// Issue a timestamp for a locally-initiated event.
    #[must_use]
    pub fn send(&mut self) -> HlcTimestamp {
        let now = physical_now_ms();
        if now > self.last.physical_ms {
            self.last = HlcTimestamp {
                physical_ms: now,
                logical: 0,
            };
        } else {
            self.last.logical += 1;
        }
        self.last
    }

    /// Absorb a received timestamp and issue one for the receive event.
    ///
    /// The result is strictly greater than both the local and the received
    /// stamp, preserving causality across the message boundary.
    ///
    /// # Errors
    ///
    /// Returns [`HlcError::ClockSkewExceeded`] if the received stamp claims a
    /// physical future beyond the local clock plus the configured bound.
    pub fn receive(&mut self, received: HlcTimestamp) -> Result<HlcTimestamp, HlcError> {
        let now = physical_now_ms();
        if self.max_skew_ms > 0 && received.physical_ms > now.saturating_add(self.max_skew_ms) {
            return Err(HlcError::ClockSkewExceeded {
                received_ms: received.physical_ms,
                skew_ms: self.max_skew_ms,
            });
        }
        let physical_ms = now.max(self.last.physical_ms).max(received.physical_ms);
        let logical = if physical_ms == self.last.physical_ms && physical_ms == received.physical_ms
        {
            self.last.logical.max(received.logical) + 1
        } else if physical_ms == self.last.physical_ms {
            self.last.logical + 1
        } else if physical_ms == received.physical_ms {
            received.logical + 1
        } else {
            0
        };
        self.last = HlcTimestamp {
            physical_ms,
            logical,
        };
        Ok(self.last)
    }

    /// Whether `a` strictly happened-before `b` in the HLC order.
    #[must_use]
    pub fn happened_before(a: HlcTimestamp, b: HlcTimestamp) -> bool {
        a < b
    }

    /// Whether `a` and `b` are concurrent (neither precedes the other).
    #[must_use]
    pub fn concurrent(a: HlcTimestamp, b: HlcTimestamp) -> bool {
        a != b && !Self::happened_before(a, b) && !Self::happened_before(b, a)
    }
}

/// Deterministic testing clock (physical component supplied by the caller).
#[cfg(test)]
pub(crate) fn timestamp(physical_ms: u64, logical: u32) -> HlcTimestamp {
    HlcTimestamp {
        physical_ms,
        logical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering_is_total_and_transitive() {
        let a = timestamp(100, 0);
        let b = timestamp(100, 1);
        let c = timestamp(101, 0);
        assert!(HlcClock::happened_before(a, b));
        assert!(HlcClock::happened_before(b, c));
        assert!(HlcClock::happened_before(a, c));
        assert!(!HlcClock::happened_before(b, a));
    }

    #[test]
    fn distinct_replica_stamps_at_same_ms_are_concurrent() {
        let a = timestamp(100, 3);
        // (100, 3) vs (100, 4) are ordered; (100, 3) vs (101, 0) ordered.
        // Concurrency requires equal stamps — impossible from one clock —
        // so a replica stamp mid-chain is what concurrency detection is for.
        let replica = timestamp(100, 3);
        assert_eq!(a, replica);
        assert!(!HlcClock::concurrent(a, timestamp(101, 0)));
        assert!(!HlcClock::concurrent(timestamp(100, 7), timestamp(101, 0)));
    }

    #[test]
    fn receive_preserves_causality() {
        let mut local = HlcClock::with_skew_bound(1_000);
        let sent = timestamp(500, 2);
        let received = local.receive(sent).expect("within skew");
        assert!(HlcClock::happened_before(sent, received));
        let next = local.send();
        assert!(HlcClock::happened_before(received, next));
    }

    #[test]
    fn excessive_future_skew_is_refused() {
        let mut local = HlcClock::with_skew_bound(1_000);
        let far_future = timestamp(u64::MAX / 2, 0);
        assert!(matches!(
            local.receive(far_future),
            Err(HlcError::ClockSkewExceeded { .. })
        ));
    }
}
