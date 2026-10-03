//! Sequence-number contiguity certification over roaring bitmaps.
//!
//! Receipt chains are append-only sequences. This module certifies, with
//! compressed bitmaps, that a monotonic sequence space has **no gaps and no
//! duplicates** — the non-replay half of the standing invariant — using a
//! bounded memory footprint even for millions of receipts.

use roaring::RoaringBitmap;
use thiserror::Error;

/// Errors produced during sequence contiguity certification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SeqBitmapError {
    /// The sequence space has a gap: some index below the observed maximum
    /// was never recorded.
    #[error("sequence gap detected at {0}")]
    Gap(u32),
    /// The sequence space contains a duplicate record.
    #[error("duplicate sequence number {0}")]
    Duplicate(u32),
}

/// A compressed, append-only certifier over monotonic sequence numbers.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SeqContiguityCertifier {
    bitmap: RoaringBitmap,
}

impl SeqContiguityCertifier {
    /// Create an empty certifier.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a sequence number.
    ///
    /// # Errors
    ///
    /// Returns [`SeqBitmapError::Duplicate`] if the sequence number was
    /// already recorded.
    pub fn record(&mut self, seq: u32) -> Result<(), SeqBitmapError> {
        if !self.bitmap.insert(seq) {
            return Err(SeqBitmapError::Duplicate(seq));
        }
        Ok(())
    }

    /// Whether the sequence number has been recorded.
    #[must_use]
    pub fn has(&self, seq: u32) -> bool {
        self.bitmap.contains(seq)
    }

    /// Number of distinct recorded sequence numbers.
    #[must_use]
    pub fn count(&self) -> u64 {
        self.bitmap.len()
    }

    /// Highest recorded sequence number.
    #[must_use]
    pub fn max(&self) -> Option<u32> {
        self.bitmap.max()
    }

    /// Certify that the recorded set is exactly `{0, 1, .., max}` with no
    /// gaps.
    ///
    /// # Errors
    ///
    /// Returns [`SeqBitmapError::Gap`] naming the first missing index.
    pub fn verify_contiguous(&self) -> Result<(), SeqBitmapError> {
        let max = match self.bitmap.max() {
            Some(m) => m,
            None => return Ok(()),
        };
        let expected = u64::from(max) + 1;
        if self.bitmap.len() == expected {
            return Ok(());
        }
        // Locate the first missing index without materializing the range.
        for (position, observed) in self.bitmap.iter().enumerate() {
            if observed != position as u32 {
                return Err(SeqBitmapError::Gap(position as u32));
            }
        }
        // Cardinality mismatch above guarantees this is unreachable, but the
        // refusal must not depend on that promise.
        Err(SeqBitmapError::Gap(max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contiguous_sequences_pass() {
        let mut cert = SeqContiguityCertifier::new();
        for seq in 0..1_000u32 {
            cert.record(seq).expect("fresh seq");
        }
        assert_eq!(cert.count(), 1_000);
        assert!(cert.verify_contiguous().is_ok());
    }

    #[test]
    fn gap_is_detected_and_named() {
        let mut cert = SeqContiguityCertifier::new();
        for seq in [0u32, 1, 2, 4, 5] {
            cert.record(seq).expect("fresh seq");
        }
        assert_eq!(cert.verify_contiguous(), Err(SeqBitmapError::Gap(3)));
    }

    #[test]
    fn duplicate_is_refused() {
        let mut cert = SeqContiguityCertifier::new();
        cert.record(7).expect("first");
        assert_eq!(cert.record(7), Err(SeqBitmapError::Duplicate(7)));
    }

    #[test]
    fn membership_and_max() {
        let mut cert = SeqContiguityCertifier::new();
        cert.record(3).expect("record");
        assert!(cert.has(3));
        assert!(!cert.has(2));
        assert_eq!(cert.max(), Some(3));
    }
}
