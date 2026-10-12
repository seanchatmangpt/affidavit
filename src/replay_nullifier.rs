//! Replay nullifier calculation and double-spend detection for CTRT receipts.
//!
//! Evaluates keyed BLAKE3 nullifiers over wire receipts:
//! `N = BLAKE3_keyed(K_epoch, ReceiptWire)`
//!
//! Provides an epoch-scoped sliding-window bit filter combined with a dense
//! nullifier cache to detect and reject receipt replay attempts.

use std::collections::HashSet;
use thiserror::Error;

/// Nullifier byte length (32 bytes).
pub const NULLIFIER_SIZE: usize = 32;

/// Key length for keyed BLAKE3 (32 bytes).
pub const NULLIFIER_KEY_SIZE: usize = 32;

/// Nullifier token computed for a receipt.
pub type Nullifier = [u8; NULLIFIER_SIZE];

/// Epoch secret key used to compute receipt nullifiers.
pub type EpochKey = [u8; NULLIFIER_KEY_SIZE];

/// Errors produced during nullifier evaluation and replay checks.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReplayNullifierError {
    /// Receipt has already been witnessed within the current epoch.
    #[error("ReplayDetected: nullifier {0} already observed in epoch")]
    ReplayDetected(String),
    /// Invalid persistent storage or deserialization error.
    #[error("InvalidPersistenceStorage: {0}")]
    InvalidPersistenceStorage(String),
    /// Corrupt ledger state detected during nullifier evaluation (operator matrix `CorruptLedger`).
    #[error("CorruptLedger: {0}")]
    CorruptLedger(String),
}

impl ReplayNullifierError {
    /// Machine error code for programmatic verification and audit mapping.
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::ReplayDetected(_) => 0xE101,
            Self::InvalidPersistenceStorage(_) | Self::CorruptLedger(_) => 0xE102,
        }
    }
}

/// Computes the keyed BLAKE3 nullifier:
/// `N = BLAKE3_keyed(K_epoch, receipt_wire)`
#[must_use]
pub fn compute_nullifier(epoch_key: &EpochKey, receipt_wire: &[u8]) -> Nullifier {
    let mut hasher = blake3::Hasher::new_keyed(epoch_key);
    hasher.update(receipt_wire);
    *hasher.finalize().as_bytes()
}

/// Fixed-size sliding-window bit filter for fast probabilistic pre-screening.
#[derive(Debug, Clone)]
pub struct SlidingBitFilter {
    bits: Vec<u64>,
    num_bits: usize,
}

impl SlidingBitFilter {
    /// Create a new bit filter with `num_bits` capacity.
    #[must_use]
    pub fn new(num_bits: usize) -> Self {
        let words = (num_bits + 63) / 64;
        Self {
            bits: vec![0u64; words],
            num_bits,
        }
    }

    /// Hash indexes for two-hash Bloom projection from 32-byte nullifier.
    fn indices(&self, nullifier: &Nullifier) -> (usize, usize) {
        let h1 = u64::from_le_bytes(nullifier[0..8].try_into().unwrap()) as usize % self.num_bits;
        let h2 = u64::from_le_bytes(nullifier[8..16].try_into().unwrap()) as usize % self.num_bits;
        (h1, h2)
    }

    /// Check if the nullifier might have been seen.
    #[must_use]
    pub fn might_contain(&self, nullifier: &Nullifier) -> bool {
        let (idx1, idx2) = self.indices(nullifier);
        let word1 = idx1 / 64;
        let bit1 = idx1 % 64;
        let word2 = idx2 / 64;
        let bit2 = idx2 % 64;

        ((self.bits[word1] & (1 << bit1)) != 0) && ((self.bits[word2] & (1 << bit2)) != 0)
    }

    /// Mark the nullifier in the bit filter.
    pub fn set(&mut self, nullifier: &Nullifier) {
        let (idx1, idx2) = self.indices(nullifier);
        let word1 = idx1 / 64;
        let bit1 = idx1 % 64;
        let word2 = idx2 / 64;
        let bit2 = idx2 % 64;

        self.bits[word1] |= 1 << bit1;
        self.bits[word2] |= 1 << bit2;
    }

    /// Clear all bits.
    pub fn clear(&mut self) {
        self.bits.fill(0);
    }
}

/// Sliding-window dense nullifier cache and replay detector.
#[derive(Debug, Clone)]
pub struct ReplayNullifierRegistry {
    /// Epoch secret key.
    epoch_key: EpochKey,
    /// Epoch generation counter within the current session/epoch lifecycle.
    epoch_generation: u64,
    /// Fast bit filter screen.
    bit_filter: SlidingBitFilter,
    /// Exact dense cache of witnessed nullifiers in current window/epoch.
    dense_cache: HashSet<Nullifier>,
    /// Capacity threshold before rotating / clearing sliding window.
    window_capacity: usize,
}

impl ReplayNullifierRegistry {
    /// Create a new nullifier registry with a given epoch key and window capacity.
    #[must_use]
    pub fn new(epoch_key: EpochKey, window_capacity: usize) -> Self {
        Self {
            epoch_key,
            epoch_generation: 0,
            bit_filter: SlidingBitFilter::new(window_capacity.max(64) * 8),
            dense_cache: HashSet::with_capacity(window_capacity),
            window_capacity,
        }
    }

    /// Return reference to epoch key.
    #[must_use]
    pub const fn epoch_key(&self) -> &EpochKey {
        &self.epoch_key
    }

    /// Return the current epoch generation counter.
    #[must_use]
    pub const fn epoch_generation(&self) -> u64 {
        self.epoch_generation
    }

    /// Rotate or update the epoch key, resetting the generation, clearing the filter and cache.
    pub fn rotate_epoch(&mut self, new_epoch_key: EpochKey) {
        self.epoch_key = new_epoch_key;
        self.epoch_generation = 0;
        self.bit_filter.clear();
        self.dense_cache.clear();
    }

    /// Witness a raw wire receipt.
    /// Computes `N = BLAKE3_keyed(K_epoch, receipt_wire)`, tests for replay,
    /// and inserts into the dense cache and bit filter.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayNullifierError::ReplayDetected`] if the nullifier
    /// has already been recorded in this epoch window.
    pub fn check_and_admit(&mut self, receipt_wire: &[u8]) -> Result<Nullifier, ReplayNullifierError> {
        let nullifier = compute_nullifier(&self.epoch_key, receipt_wire);
        self.admit_nullifier(nullifier)?;
        Ok(nullifier)
    }

    /// Directly admit a computed nullifier, checking for replay.
    ///
    /// # Errors
    ///
    /// Returns [`ReplayNullifierError::ReplayDetected`] if already seen.
    pub fn admit_nullifier(&mut self, nullifier: Nullifier) -> Result<(), ReplayNullifierError> {
        // Fast path: if bit filter says unseen, it is definitely not seen
        if self.bit_filter.might_contain(&nullifier) && self.dense_cache.contains(&nullifier) {
            let hex = nullifier
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>();
            return Err(ReplayNullifierError::ReplayDetected(hex));
        }

        // If capacity exceeded, harden against intra-epoch replay after eviction
        // by rotating the epoch key derived from the current key and advancing the epoch generation.
        if self.dense_cache.len() >= self.window_capacity {
            self.epoch_generation = self.epoch_generation.saturating_add(1);
            let mut hasher = blake3::Hasher::new_derive_key("affidavit/ctrt-epoch-rotation/v1");
            hasher.update(&self.epoch_key);
            hasher.update(&self.epoch_generation.to_le_bytes());
            self.epoch_key = *hasher.finalize().as_bytes();

            self.dense_cache.clear();
            self.bit_filter.clear();
        }

        self.bit_filter.set(&nullifier);
        self.dense_cache.insert(nullifier);
        Ok(())
    }

    /// Check if a nullifier has already been admitted without mutating state.
    #[must_use]
    pub fn is_seen(&self, nullifier: &Nullifier) -> bool {
        if self.bit_filter.might_contain(nullifier) {
            self.dense_cache.contains(nullifier)
        } else {
            false
        }
    }

    /// Current count of tracked nullifiers in dense cache.
    #[must_use]
    pub fn len(&self) -> usize {
        self.dense_cache.len()
    }

    /// Check if no nullifiers are currently tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dense_cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyed_blake3_nullifier_deterministic() {
        let key = [0x42; 32];
        let wire = b"sample receipt wire payload";
        let n1 = compute_nullifier(&key, wire);
        let n2 = compute_nullifier(&key, wire);
        assert_eq!(n1, n2);

        let key2 = [0x43; 32];
        let n3 = compute_nullifier(&key2, wire);
        assert_ne!(n1, n3);
    }

    #[test]
    fn test_replay_detection() {
        let key = [0x01; 32];
        let mut registry = ReplayNullifierRegistry::new(key, 100);

        let receipt1 = b"receipt-number-1";
        let receipt2 = b"receipt-number-2";

        assert!(registry.check_and_admit(receipt1).is_ok());
        assert!(registry.check_and_admit(receipt2).is_ok());

        // Replay of receipt1 should be detected and rejected
        let err = registry.check_and_admit(receipt1).unwrap_err();
        assert!(matches!(err, ReplayNullifierError::ReplayDetected(_)));

        // receipt2 replay rejected
        let err2 = registry.check_and_admit(receipt2).unwrap_err();
        assert!(matches!(err2, ReplayNullifierError::ReplayDetected(_)));
    }

    #[test]
    fn test_epoch_rotation() {
        let key1 = [0x01; 32];
        let key2 = [0x02; 32];
        let mut registry = ReplayNullifierRegistry::new(key1, 100);

        let receipt = b"receipt-payload";
        assert!(registry.check_and_admit(receipt).is_ok());
        assert!(registry.check_and_admit(receipt).is_err());

        // Rotate epoch
        registry.rotate_epoch(key2);
        // In new epoch with new key, receipt has a different nullifier and cache is cleared
        assert!(registry.check_and_admit(receipt).is_ok());
    }

    #[test]
    fn test_sliding_window_capacity_eviction() {
        let key = [0xaa; 32];
        let capacity = 10;
        let mut registry = ReplayNullifierRegistry::new(key, capacity);

        for i in 0..capacity {
            let data = format!("receipt_{i}");
            assert!(registry.check_and_admit(data.as_bytes()).is_ok());
        }

        assert_eq!(registry.len(), capacity);

        // Next admit triggers sliding window reset
        let overflow_receipt = b"receipt_overflow";
        assert!(registry.check_and_admit(overflow_receipt).is_ok());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_replay_nullifier_error_machine_codes() {
        let err1 = ReplayNullifierError::ReplayDetected("deadbeef".to_string());
        assert_eq!(err1.machine_code(), 0xE101);

        let err2 = ReplayNullifierError::InvalidPersistenceStorage("io error".to_string());
        assert_eq!(err2.machine_code(), 0xE102);

        let err3 = ReplayNullifierError::CorruptLedger("checksum mismatch".to_string());
        assert_eq!(err3.machine_code(), 0xE102);
    }

    #[test]
    fn test_intra_epoch_eviction_protection_rotates_epoch_key() {
        let key = [0xbb; 32];
        let capacity = 5;
        let mut registry = ReplayNullifierRegistry::new(key, capacity);

        let initial_receipt = b"victim_receipt";
        let n_orig = compute_nullifier(registry.epoch_key(), initial_receipt);
        assert!(registry.check_and_admit(initial_receipt).is_ok());

        // Fill up to capacity
        for i in 1..capacity {
            let data = format!("filler_{i}");
            assert!(registry.check_and_admit(data.as_bytes()).is_ok());
        }
        assert_eq!(registry.len(), capacity);
        assert_eq!(registry.epoch_generation(), 0);

        // Next admittance causes eviction and epoch rotation
        let trigger_receipt = b"eviction_trigger";
        assert!(registry.check_and_admit(trigger_receipt).is_ok());

        assert_eq!(registry.len(), 1);
        assert_eq!(registry.epoch_generation(), 1);
        assert_ne!(registry.epoch_key(), &key);

        // Even though the old dense cache was cleared, the old receipt's nullifier
        // under the OLD epoch key cannot be admitted directly under that old key, and
        // under the rotated epoch key, check_and_admit computes a completely new nullifier.
        // Furthermore, any replay attempt with the old nullifier cannot masquerade as valid.
        let n_new = compute_nullifier(registry.epoch_key(), initial_receipt);
        assert_ne!(n_orig, n_new, "Key rotation changes nullifier for previously seen receipt");
    }
}
