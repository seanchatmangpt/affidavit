//! Bounded-memory replay screening (cuckoo filter).
//!
//! A cuckoo filter is a *deletable* probabilistic membership structure: unlike
//! a Bloom filter it supports removal, which is what makes revocable leases
//! and expiring standing tokens representable.
//!
//! ## Doctrine: Certify, Don't Decide — and the standing ceiling
//!
//! A probabilistic filter can produce false positives ("seen" for an unseen
//! id), never false negatives. This module therefore has an explicit standing
//! ceiling: it is a **screen**, not a court. A negative is decisive (the id
//! was never inserted); a positive must be confirmed against the exact
//! receipt store before any consequential refusal.

use cuckoofilter::CuckooFilter;
use thiserror::Error;

/// Errors produced by replay filter operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReplayFilterError {
    /// The filter is at capacity; the witness could not be recorded.
    #[error("replay filter at capacity: {0} entries")]
    Full(usize),
}

/// A bounded-memory screen for already-witnessed receipt identifiers.
pub struct ReplayFilter {
    filter: CuckooFilter<std::collections::hash_map::DefaultHasher>,
    capacity: usize,
}

impl ReplayFilter {
    /// Create a filter sized for `capacity` identifiers.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            filter: CuckooFilter::with_capacity(capacity),
            capacity,
        }
    }

    /// Record a witness id. Returns `true` if the id is new (now recorded),
    /// `false` if the filter has already seen it (probable replay).
    ///
    /// # Errors
    ///
    /// Returns [`ReplayFilterError::Full`] if the filter cannot admit the
    /// entry; the caller must treat the id as unrecorded.
    pub fn witness(&mut self, id: &[u8; 32]) -> Result<bool, ReplayFilterError> {
        if self.filter.contains(id) {
            return Ok(false);
        }
        self.filter
            .add(id)
            .map_err(|_| ReplayFilterError::Full(self.len()))?;
        Ok(true)
    }

    /// Whether the id has probably been witnessed.
    ///
    /// A `false` here is exact; a `true` is probabilistic and must be
    /// confirmed upstream before acting on it.
    #[must_use]
    pub fn probably_seen(&self, id: &[u8; 32]) -> bool {
        self.filter.contains(id)
    }

    /// Retract a witness id (lease expiry / standing revocation).
    ///
    /// Returns `true` if the id was present and removed.
    pub fn retract(&mut self, id: &[u8; 32]) -> bool {
        self.filter.delete(id)
    }

    /// Number of entries currently recorded.
    #[must_use]
    pub fn len(&self) -> usize {
        self.filter.len()
    }

    /// The capacity the filter was sized for.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Whether no entries are recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(seed: u8) -> [u8; 32] {
        let mut i = [0u8; 32];
        i[0] = seed;
        i
    }

    #[test]
    fn fresh_ids_admit_and_repeat_is_replay() {
        let mut rf = ReplayFilter::new(1_000);
        assert!(rf.witness(&id(1)).expect("fresh"));
        assert!(!rf.witness(&id(1)).expect("replay"));
        assert!(rf.probably_seen(&id(1)));
        assert!(!rf.probably_seen(&id(2)));
    }

    #[test]
    fn retraction_allows_re_witnessing() {
        let mut rf = ReplayFilter::new(1_000);
        rf.witness(&id(5)).expect("witness");
        assert!(rf.retract(&id(5)), "retraction of present id succeeds");
        assert!(!rf.probably_seen(&id(5)));
        assert!(rf.witness(&id(5)).expect("re-witness after expiry"));
    }

    #[test]
    fn negative_screens_are_exact() {
        let mut rf = ReplayFilter::new(1_000);
        for s in 0..64u8 {
            rf.witness(&id(s)).expect("witness");
        }
        // Un-inserted ids are never reported unseen-as-seen from the other
        // side: a negative must be exact.
        assert!(!rf.probably_seen(&id(200)));
    }
}
