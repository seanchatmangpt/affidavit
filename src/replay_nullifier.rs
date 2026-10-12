//! Replay nullifier calculation and double-spend detection for CTRT receipts.
//!
//! Evaluates keyed BLAKE3 nullifiers over wire receipts:
//! `N = BLAKE3_keyed(K_epoch, ReceiptWire)`
//!
//! Provides an epoch-scoped sliding-window bit filter combined with a dense
//! nullifier cache to detect and reject receipt replay attempts.
//!
//! # Rotation replay-amnesty fix (SK5 fix-forward, lane R31)
//!
//! Prior behavior cleared the dense cache and bit filter on every epoch
//! rotation (explicit `rotate_epoch` or capacity-triggered generation
//! advance), which allowed `check_and_admit(original_wire)` to succeed again
//! after rotation — a replay amnesty across epochs. The fix keeps a
//! rotation-stable *wire-identity* layer:
//!
//! * each wire is reduced to a 32-byte identity
//!   `BLAKE3_keyed(K_identity, wire)` where `K_identity` is derived once at
//!   construction and never rotates;
//! * every witnessed identity is tagged with the epoch generation it was
//!   witnessed in and is retained across rotations (current window in
//!   `wire_window`, retired generations in `wire_history`);
//! * generation-bound nullifiers are stored alongside
//!   (`dense_cache: HashMap<Nullifier, u64>` carries the generation per
//!   nullifier, retired nullifiers move to `nullifier_history`);
//! * re-witnessing a wire/nullifier from a *prior* generation is refused with
//!   the typed error [`ReplayNullifierError::ReplayEpochMismatch`]
//!   (machine code `0xE103`); re-witnessing within the current generation is
//!   still [`ReplayNullifierError::ReplayDetected`] (`0xE101`).

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use thiserror::Error;

/// Nullifier byte length (32 bytes).
pub const NULLIFIER_SIZE: usize = 32;

/// Key length for keyed BLAKE3 (32 bytes).
pub const NULLIFIER_KEY_SIZE: usize = 32;

/// Machine error code for [`ReplayNullifierError::ReplayDetected`].
pub const CODE_REPLAY_DETECTED: u16 = 0xE101;
/// Machine error code shared by persistence-layer failures.
pub const CODE_PERSISTENCE: u16 = 0xE102;
/// Machine error code for [`ReplayNullifierError::ReplayEpochMismatch`]:
/// a pre-rotation wire/nullifier presented after epoch rotation.
pub const CODE_REPLAY_EPOCH_MISMATCH: u16 = 0xE103;

/// Nullifier token computed for a receipt.
pub type Nullifier = [u8; NULLIFIER_SIZE];

/// Epoch secret key used to compute receipt nullifiers.
pub type EpochKey = [u8; NULLIFIER_KEY_SIZE];

/// Rotation-stable wire identity token (`BLAKE3_keyed(K_identity, wire)`).
pub type WireId = [u8; NULLIFIER_SIZE];

/// Snapshot serialization format magic (`b"AFRN"`).
const SNAPSHOT_MAGIC: &[u8; 4] = b"AFRN";
/// Snapshot serialization format version.
///
/// v2 (this version) serializes `nullifier_history` — the retired-nullifier
/// layer that enforces cross-generation refusal. v1 snapshots are REFUSED on
/// load (fail-closed): a v1 snapshot silently dropped `nullifier_history`, so
/// it cannot enforce epoch binding — see [`ReplayNullifierRegistry::load_from`].
const SNAPSHOT_VERSION: u16 = 2;
/// Fixed snapshot header size: magic(4) + version(2) + reserved(2)
/// + window_capacity(8) + generation(8) + epoch_key(32) + identity_key(32).
const SNAPSHOT_HEADER_LEN: usize = 4 + 2 + 2 + 8 + 8 + NULLIFIER_SIZE + NULLIFIER_SIZE;

/// Length of the HMAC-BLAKE3 authentication tag prepended to a sealed
/// snapshot ([`ReplayNullifierRegistry::seal`] /
/// [`ReplayNullifierRegistry::unseal`]).
pub const SNAPSHOT_MAC_LEN: usize = 32;

/// Compute the HMAC-BLAKE3 tag over a snapshot body. BLAKE3 in keyed mode is
/// a proper MAC (PRF); the key is a persistence-authentication key, distinct
/// from both the epoch key and the wire-identity key.
fn snapshot_mac(mac_key: &[u8; NULLIFIER_KEY_SIZE], body: &[u8]) -> [u8; NULLIFIER_SIZE] {
    let mut hasher = blake3::Hasher::new_keyed(mac_key);
    hasher.update(body);
    *hasher.finalize().as_bytes()
}

/// Constant-time equality for 32-byte tags.
fn tags_equal(a: &[u8; NULLIFIER_SIZE], b: &[u8; NULLIFIER_SIZE]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Errors produced during nullifier evaluation and replay checks.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReplayNullifierError {
    /// Receipt has already been witnessed within the current epoch generation.
    #[error("ReplayDetected: nullifier {0} already observed in epoch")]
    ReplayDetected(String),
    /// Invalid persistent storage or deserialization error.
    #[error("InvalidPersistenceStorage: {0}")]
    InvalidPersistenceStorage(String),
    /// Corrupt ledger state detected during nullifier evaluation (operator matrix `CorruptLedger`).
    #[error("CorruptLedger: {0}")]
    CorruptLedger(String),
    /// Pre-rotation wire or nullifier presented after epoch rotation.
    #[error("ReplayEpochMismatch: {0}")]
    ReplayEpochMismatch(String),
}

impl ReplayNullifierError {
    /// Machine error code for programmatic verification and audit mapping.
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::ReplayDetected(_) => CODE_REPLAY_DETECTED,
            Self::InvalidPersistenceStorage(_) | Self::CorruptLedger(_) => CODE_PERSISTENCE,
            Self::ReplayEpochMismatch(_) => CODE_REPLAY_EPOCH_MISMATCH,
        }
    }
}

/// Computes the keyed BLAKE3 nullifier:
/// `N = BLAKE3_keyed(K_epoch, receipt_wire)`
///
/// Note: this is the *epoch-scoped* nullifier. The generation binding that
/// closes the rotation amnesty is carried by tagging stored nullifiers and
/// wire identities with [`ReplayNullifierRegistry::epoch_generation`], not by
/// folding the generation into the digest, so this function remains stable
/// across generations of the same epoch key.
#[must_use]
pub fn compute_nullifier(epoch_key: &EpochKey, receipt_wire: &[u8]) -> Nullifier {
    let mut hasher = blake3::Hasher::new_keyed(epoch_key);
    hasher.update(receipt_wire);
    *hasher.finalize().as_bytes()
}

/// Computes the rotation-stable wire identity:
/// `Id = BLAKE3_keyed(K_identity, wire)`; `K_identity` never rotates.
#[must_use]
pub fn compute_wire_identity(identity_key: &EpochKey, receipt_wire: &[u8]) -> WireId {
    let mut hasher = blake3::Hasher::new_keyed(identity_key);
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
        let h1 = u64::from_le_bytes(nullifier[0..8].try_into().expect("32-byte slice")) as usize
            % self.num_bits;
        let h2 = u64::from_le_bytes(nullifier[8..16].try_into().expect("32-byte slice")) as usize
            % self.num_bits;
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
    /// Rotation-stable wire-identity key, derived once at construction.
    identity_key: EpochKey,
    /// Epoch generation counter; advances on every rotation, never resets.
    epoch_generation: u64,
    /// Fast bit filter screen over current-generation nullifiers.
    bit_filter: SlidingBitFilter,
    /// Exact dense cache of witnessed nullifiers in the current generation,
    /// tagged with the generation they were witnessed in.
    dense_cache: HashMap<Nullifier, u64>,
    /// Nullifiers retired by rotation, tagged with their generation.
    nullifier_history: HashMap<Nullifier, u64>,
    /// Wire identities witnessed in the current generation.
    wire_window: HashSet<WireId>,
    /// Wire identities retired by rotation, tagged with their generation.
    wire_history: HashMap<WireId, u64>,
    /// Capacity threshold before rotating / clearing sliding window.
    window_capacity: usize,
}

impl ReplayNullifierRegistry {
    /// Create a new nullifier registry with a given epoch key and window capacity.
    ///
    /// # Caller guidance
    ///
    /// `epoch_key` SHOULD be derived from entropy at session start (see
    /// [`ReplayNullifierRegistry::with_generated_key`]), not passed as a
    /// compile-time constant: a constant epoch key lets an attacker who
    /// learns it precompute nullifiers and forge cross-context replay
    /// evidence. The parameter is retained so deterministic test subjects and
    /// key-rotation stores can pin the key explicitly.
    #[must_use]
    pub fn new(epoch_key: EpochKey, window_capacity: usize) -> Self {
        let identity_key = derive_identity_key(&epoch_key);
        Self {
            epoch_key,
            identity_key,
            epoch_generation: 0,
            bit_filter: SlidingBitFilter::new(window_capacity.max(64) * 8),
            dense_cache: HashMap::with_capacity(window_capacity),
            nullifier_history: HashMap::new(),
            wire_window: HashSet::new(),
            wire_history: HashMap::new(),
            window_capacity,
        }
    }

    /// Create a registry whose epoch key is drawn from the operating-system
    /// entropy source instead of a caller-supplied constant.
    ///
    /// Available under the `crypto-trust` feature (the key derives from
    /// `rand_core::OsRng`, whose backing `getrandom` chain is wired for both
    /// native and wasm32 targets in `Cargo.toml`).
    ///
    /// # Errors
    ///
    /// Returns [`ReplayNullifierError::InvalidPersistenceStorage`] if the OS
    /// entropy source fails.
    #[cfg(feature = "crypto-trust")]
    pub fn with_generated_key(window_capacity: usize) -> Result<Self, ReplayNullifierError> {
        use rand_core::{OsRng, RngCore};

        let mut epoch_key = EpochKey::default();
        OsRng.fill_bytes(&mut epoch_key);
        if epoch_key == EpochKey::default() {
            return Err(ReplayNullifierError::CorruptLedger(
                "OS entropy returned an all-zero epoch key".to_string(),
            ));
        }
        Ok(Self::new(epoch_key, window_capacity))
    }

    /// Return reference to epoch key.
    #[must_use]
    pub const fn epoch_key(&self) -> &EpochKey {
        &self.epoch_key
    }

    /// Return reference to the rotation-stable wire-identity key.
    #[must_use]
    pub const fn identity_key(&self) -> &EpochKey {
        &self.identity_key
    }

    /// Return the current epoch generation counter.
    #[must_use]
    pub const fn epoch_generation(&self) -> u64 {
        self.epoch_generation
    }

    /// Rotate to a new epoch key.
    ///
    /// The generation counter ADVANCES (it never resets), the current window
    /// retires into the cross-epoch history, and the wire-identity layer
    /// survives so pre-rotation wires are refused with
    /// [`ReplayNullifierError::ReplayEpochMismatch`] afterwards.
    pub fn rotate_epoch(&mut self, new_epoch_key: EpochKey) {
        self.retire_current_window();
        self.epoch_generation = self.epoch_generation.saturating_add(1);
        self.epoch_key = new_epoch_key;
        self.bit_filter.clear();
        self.dense_cache.clear();
        self.wire_window.clear();
    }

    /// Retire the live window into the cross-generation history (keyed by the
    /// generation it was witnessed in) without advancing the generation.
    fn retire_current_window(&mut self) {
        let generation = self.epoch_generation;
        for (n, gen) in self.dense_cache.drain() {
            self.nullifier_history.entry(n).or_insert(gen);
        }
        for id in self.wire_window.drain() {
            self.wire_history.insert(id, generation);
        }
        self.bit_filter.clear();
    }

    /// Advance the generation on capacity pressure: retire the current window
    /// into history, derive the next epoch key, clear the live window.
    fn advance_generation(&mut self) {
        self.retire_current_window();
        self.epoch_generation = self.epoch_generation.saturating_add(1);
        let mut hasher =
            blake3::Hasher::new_derive_key("affidavit/ctrt-epoch-rotation/v1");
        hasher.update(&self.epoch_key);
        hasher.update(&self.epoch_generation.to_le_bytes());
        self.epoch_key = *hasher.finalize().as_bytes();
        self.dense_cache.clear();
        self.wire_window.clear();
    }

    /// Witness a raw wire receipt.
    ///
    /// Computes `N = BLAKE3_keyed(K_epoch, receipt_wire)`, tests for replay
    /// (within the current generation) and cross-rotation replay (a wire
    /// already witnessed in a prior generation is refused with
    /// [`ReplayNullifierError::ReplayEpochMismatch`]), and inserts into the
    /// dense cache and bit filter.
    ///
    /// # Errors
    ///
    /// * [`ReplayNullifierError::ReplayDetected`] — wire already witnessed in
    ///   the current generation.
    /// * [`ReplayNullifierError::ReplayEpochMismatch`] — wire already
    ///   witnessed in a prior generation (pre-rotation replay).
    pub fn check_and_admit(&mut self, receipt_wire: &[u8]) -> Result<Nullifier, ReplayNullifierError> {
        let wire_id = compute_wire_identity(&self.identity_key, receipt_wire);

        if self.wire_window.contains(&wire_id) {
            let nullifier = compute_nullifier(&self.epoch_key, receipt_wire);
            return Err(ReplayNullifierError::ReplayDetected(to_hex(&nullifier)));
        }
        if let Some(&witnessed_gen) = self.wire_history.get(&wire_id) {
            return Err(ReplayNullifierError::ReplayEpochMismatch(format!(
                "wire already witnessed in epoch generation {witnessed_gen} (current {})",
                self.epoch_generation
            )));
        }

        // Capacity pressure rotates the generation BEFORE the nullifier is
        // computed, so the admitted nullifier is bound to the new generation.
        if self.dense_cache.len() >= self.window_capacity {
            self.advance_generation();
        }

        let nullifier = compute_nullifier(&self.epoch_key, receipt_wire);
        self.bit_filter.set(&nullifier);
        self.dense_cache.insert(nullifier, self.epoch_generation);
        self.wire_window.insert(wire_id);
        Ok(nullifier)
    }

    /// Directly admit a precomputed nullifier, checking for replay.
    ///
    /// Nullifiers retired by rotation are refused with
    /// [`ReplayNullifierError::ReplayEpochMismatch`]; duplicates within the
    /// current generation are refused with [`ReplayNullifierError::ReplayDetected`].
    ///
    /// # Errors
    ///
    /// See above; fail-closed on any already-witnessed nullifier.
    pub fn admit_nullifier(&mut self, nullifier: Nullifier) -> Result<(), ReplayNullifierError> {
        if self.dense_cache.contains_key(&nullifier) {
            return Err(ReplayNullifierError::ReplayDetected(to_hex(&nullifier)));
        }
        if let Some(&witnessed_gen) = self.nullifier_history.get(&nullifier) {
            return Err(ReplayNullifierError::ReplayEpochMismatch(format!(
                "nullifier already witnessed in epoch generation {witnessed_gen} (current {})",
                self.epoch_generation
            )));
        }

        // Fast screen: if the bit filter says unseen, it is definitely not in
        // the current window (retired nullifiers were already checked above).
        if self.bit_filter.might_contain(&nullifier) && self.dense_cache.contains_key(&nullifier) {
            return Err(ReplayNullifierError::ReplayDetected(to_hex(&nullifier)));
        }

        if self.dense_cache.len() >= self.window_capacity {
            self.advance_generation();
        }

        self.bit_filter.set(&nullifier);
        self.dense_cache.insert(nullifier, self.epoch_generation);
        Ok(())
    }

    /// Check if a nullifier has already been admitted (current window or any
    /// retired generation) without mutating state.
    #[must_use]
    pub fn is_seen(&self, nullifier: &Nullifier) -> bool {
        self.dense_cache.contains_key(nullifier) || self.nullifier_history.contains_key(nullifier)
    }

    /// Check if a wire identity has already been witnessed in any generation
    /// without mutating state.
    #[must_use]
    pub fn wire_seen(&self, receipt_wire: &[u8]) -> bool {
        let wire_id = compute_wire_identity(&self.identity_key, receipt_wire);
        self.wire_window.contains(&wire_id) || self.wire_history.contains_key(&wire_id)
    }

    /// Count of tracked nullifiers in the current-generation dense cache.
    #[must_use]
    pub fn len(&self) -> usize {
        self.dense_cache.len()
    }

    /// Check if no nullifiers are currently tracked in the live window.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dense_cache.is_empty()
    }

    /// Count of wire identities retired by rotation (cross-epoch memory).
    #[must_use]
    pub fn wire_history_len(&self) -> usize {
        self.wire_history.len()
    }

    /// Count of nullifiers retired by rotation (cross-epoch memory).
    #[must_use]
    pub fn nullifier_history_len(&self) -> usize {
        self.nullifier_history.len()
    }

    /// Serialize the registry state to a fixed-structure byte snapshot (v2).
    ///
    /// Layout (all integers little-endian):
    /// `magic("AFRN") | version(u16) | reserved(u16) | window_capacity(u64)
    ///  | generation(u64) | epoch_key(32) | identity_key(32)
    ///  | nullifier_count(u64) | (nullifier(32) ++ generation(u64))*
    ///  | wire_window_count(u64) | wire_id(32)*
    ///  | wire_history_count(u64) | (wire_id(32) ++ generation(u64))*
    ///  | nullifier_history_count(u64) | (nullifier(32) ++ generation(u64))*`
    ///
    /// v2 serializes `nullifier_history` (v1 dropped it, which broke
    /// cross-generation refusal across process restarts — SX4). At-rest
    /// authentication is provided by [`ReplayNullifierRegistry::seal`],
    /// not by this method.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(
            SNAPSHOT_HEADER_LEN
                + 8 + self.dense_cache.len() * (NULLIFIER_SIZE + 8)
                + 8 + self.wire_window.len() * NULLIFIER_SIZE
                + 8 + self.wire_history.len() * (NULLIFIER_SIZE + 8)
                + 8 + self.nullifier_history.len() * (NULLIFIER_SIZE + 8),
        );
        out.extend_from_slice(SNAPSHOT_MAGIC);
        out.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // reserved
        out.extend_from_slice(&(self.window_capacity as u64).to_le_bytes());
        out.extend_from_slice(&self.epoch_generation.to_le_bytes());
        out.extend_from_slice(&self.epoch_key);
        out.extend_from_slice(&self.identity_key);

        out.extend_from_slice(&(self.dense_cache.len() as u64).to_le_bytes());
        for (n, gen) in &self.dense_cache {
            out.extend_from_slice(n);
            out.extend_from_slice(&gen.to_le_bytes());
        }

        out.extend_from_slice(&(self.wire_window.len() as u64).to_le_bytes());
        for id in &self.wire_window {
            out.extend_from_slice(id);
        }

        out.extend_from_slice(&(self.wire_history.len() as u64).to_le_bytes());
        for (id, gen) in &self.wire_history {
            out.extend_from_slice(id);
            out.extend_from_slice(&gen.to_le_bytes());
        }

        // v2: the retired-nullifier history (cross-generation refusal layer).
        out.extend_from_slice(&(self.nullifier_history.len() as u64).to_le_bytes());
        for (n, gen) in &self.nullifier_history {
            out.extend_from_slice(n);
            out.extend_from_slice(&gen.to_le_bytes());
        }
        out
    }

    /// Seal the snapshot with an HMAC-BLAKE3 authentication tag:
    /// `tag(32) ++ body`. The `mac_key` MUST be distinct from both the epoch
    /// key and the wire-identity key.
    #[must_use]
    pub fn seal(&self, mac_key: &[u8; NULLIFIER_KEY_SIZE]) -> Vec<u8> {
        let body = self.to_bytes();
        let mut out = Vec::with_capacity(SNAPSHOT_MAC_LEN + body.len());
        out.extend_from_slice(&snapshot_mac(mac_key, &body));
        out.extend_from_slice(&body);
        out
    }

    /// Open a [`ReplayNullifierRegistry::seal`]ed snapshot. Fail-closed:
    /// * truncated envelope (below `tag + header`) →
    ///   [`ReplayNullifierError::InvalidPersistenceStorage`]
    /// * HMAC mismatch (forged or truncated body) →
    ///   [`ReplayNullifierError::CorruptLedger`] (constant-time tag compare)
    ///
    /// # Errors
    ///
    /// Any structural defect in the sealed body surfaces as the typed error
    /// from [`ReplayNullifierRegistry::load_from`].
    pub fn unseal(
        bytes: &[u8],
        mac_key: &[u8; NULLIFIER_KEY_SIZE],
    ) -> Result<Self, ReplayNullifierError> {
        if bytes.len() < SNAPSHOT_MAC_LEN + SNAPSHOT_HEADER_LEN {
            return Err(ReplayNullifierError::InvalidPersistenceStorage(format!(
                "sealed snapshot truncated: {} bytes < tag {SNAPSHOT_MAC_LEN} + header {SNAPSHOT_HEADER_LEN}",
                bytes.len()
            )));
        }
        let (tag_bytes, body) = bytes.split_at(SNAPSHOT_MAC_LEN);
        let mut tag = [0u8; NULLIFIER_SIZE];
        tag.copy_from_slice(tag_bytes);
        let expected = snapshot_mac(mac_key, body);
        if !tags_equal(&expected, &tag) {
            return Err(ReplayNullifierError::CorruptLedger(
                "snapshot authentication failed (HMAC-BLAKE3 tag mismatch: forged or truncated)"
                    .to_string(),
            ));
        }
        Self::load_from(body)
    }

    /// Rebuild a registry from a [`ReplayNullifierRegistry::to_bytes`]
    /// snapshot. Fail-closed: ANY structural defect (wrong magic or version,
    /// truncation, misalignment, trailing bytes, duplicate records) is a
    /// typed error, never a silent skip.
    ///
    /// # Errors
    ///
    /// * [`ReplayNullifierError::CorruptLedger`] — wrong magic, unsupported
    ///   version, or duplicate nullifier/wire records.
    /// * [`ReplayNullifierError::InvalidPersistenceStorage`] — truncated,
    ///   misaligned, or trailing-byte payloads; oversized count fields;
    ///   v1 snapshots (cannot enforce epoch binding — refuse, regenerate).
    pub fn load_from(bytes: &[u8]) -> Result<Self, ReplayNullifierError> {
        // Lane R32 minimal unblock fix (disclosed): the variant constructor
        // takes String, so bind the io-error path through a &str wrapper.
        fn ERR_IO(s: &str) -> ReplayNullifierError {
            ReplayNullifierError::InvalidPersistenceStorage(s.to_string())
        }

        if bytes.len() < SNAPSHOT_HEADER_LEN {
            return Err(ERR_IO(&format!(
                "snapshot truncated: {} bytes < header {SNAPSHOT_HEADER_LEN}",
                bytes.len()
            )));
        }
        if &bytes[0..4] != SNAPSHOT_MAGIC {
            return Err(ReplayNullifierError::CorruptLedger(
                "snapshot magic mismatch (expected AFRN)".to_string(),
            ));
        }
        let version = u16::from_le_bytes(
            bytes[4..6].try_into().expect("header bounds checked"),
        );
        if version == 1 {
            // Fail-closed (SX4): v1 snapshots silently dropped
            // nullifier_history, so a reloaded registry could not enforce
            // cross-generation refusal. Refuse rather than re-admit.
            return Err(ReplayNullifierError::InvalidPersistenceStorage(
                "v1 snapshot cannot enforce epoch binding; regenerate"
                    .to_string(),
            ));
        }
        if version != SNAPSHOT_VERSION {
            return Err(ReplayNullifierError::CorruptLedger(format!(
                "unsupported snapshot version {version}"
            )));
        }

        let mut off = 8usize;
        let read_u64 = |off: &mut usize| -> Result<u64, ReplayNullifierError> {
            let end = off
                .checked_add(8)
                .ok_or_else(|| ERR_IO("offset overflow"))?;
            if end > bytes.len() {
                return Err(ERR_IO("snapshot truncated in integer field"));
            }
            let v = u64::from_le_bytes(bytes[*off..end].try_into().expect("bounds checked"));
            *off = end;
            Ok(v)
        };
        let read_key32 = |off: &mut usize| -> Result<[u8; 32], ReplayNullifierError> {
            let end = off
                .checked_add(NULLIFIER_SIZE)
                .ok_or_else(|| ERR_IO("offset overflow"))?;
            if end > bytes.len() {
                return Err(ERR_IO("snapshot truncated in key field"));
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&bytes[*off..end]);
            *off = end;
            Ok(k)
        };

        let window_capacity_u64 = read_u64(&mut off)?;
        if window_capacity_u64 == 0 || window_capacity_u64 > usize::MAX as u64 {
            return Err(ERR_IO(&format!(
                "invalid window_capacity {window_capacity_u64}"
            )));
        }
        // Rust target pointers are at most 2^61 bytes; anything above that is
        // a count-field corruption, rejected before any allocation.
        if window_capacity_u64 > (1 << 40) {
            return Err(ERR_IO(&format!(
                "implausible window_capacity {window_capacity_u64}"
            )));
        }
        let window_capacity = window_capacity_u64 as usize;

        let epoch_generation = read_u64(&mut off)?;
        let epoch_key = read_key32(&mut off)?;
        let identity_key = read_key32(&mut off)?;

        let read_records32 = |off: &mut usize,
                              count: u64,
                              tagged: bool|
         -> Result<Vec<([u8; 32], u64)>, ReplayNullifierError> {
            // Reject count fields that cannot possibly fit in the payload
            // before any allocation (bounds the u64 count to real bytes).
            let record_len: u64 = if tagged { 40 } else { 32 };
            if count > (bytes.len().saturating_sub(*off) as u64) / record_len {
                return Err(ERR_IO(&format!(
                    "snapshot truncated: record count {count} exceeds remaining bytes"
                )));
            }
            let mut records = Vec::new();
            for _ in 0..count {
                let key = read_key32(off)?;
                let gen = if tagged { read_u64(off)? } else { 0 };
                records.push((key, gen));
            }
            Ok(records)
        };

        let dense_count = read_u64(&mut off)?;
        let dense = read_records32(&mut off, dense_count, true)?;
        let window_count = read_u64(&mut off)?;
        let wire_window = read_records32(&mut off, window_count, false)?;
        let history_count = read_u64(&mut off)?;
        let wire_history = read_records32(&mut off, history_count, true)?;
        let nullifier_history_count = read_u64(&mut off)?;
        let nullifier_history = read_records32(&mut off, nullifier_history_count, true)?;

        if off != bytes.len() {
            return Err(ERR_IO(&format!(
                "snapshot has {} trailing bytes after final section",
                bytes.len() - off
            )));
        }

        let mut dense_cache = HashMap::with_capacity(dense.len());
        for (n, gen) in dense {
            if dense_cache.insert(n, gen).is_some() {
                return Err(ReplayNullifierError::CorruptLedger(format!(
                    "duplicate nullifier record {}",
                    to_hex(&n)
                )));
            }
        }
        let mut nullifier_history_map = HashMap::with_capacity(nullifier_history.len());
        for (n, gen) in nullifier_history {
            if dense_cache.contains_key(&n) {
                return Err(ReplayNullifierError::CorruptLedger(format!(
                    "nullifier {} present in both window and history",
                    to_hex(&n)
                )));
            }
            if nullifier_history_map.insert(n, gen).is_some() {
                return Err(ReplayNullifierError::CorruptLedger(
                    "duplicate nullifier-history record".to_string(),
                ));
            }
        }
        let mut wire_window_set = HashSet::with_capacity(wire_window.len());
        for (id, _gen) in wire_window {
            if !wire_window_set.insert(id) {
                return Err(ReplayNullifierError::CorruptLedger(
                    "duplicate wire-window record".to_string(),
                ));
            }
        }
        let mut wire_history_map = HashMap::with_capacity(wire_history.len());
        for (id, gen) in wire_history {
            if wire_window_set.contains(&id) {
                return Err(ReplayNullifierError::CorruptLedger(format!(
                    "wire identity {} present in both window and history",
                    to_hex(&id)
                )));
            }
            if wire_history_map.insert(id, gen).is_some() {
                return Err(ReplayNullifierError::CorruptLedger(
                    "duplicate wire-history record".to_string(),
                ));
            }
        }

        // Rebuild the bit-filter screen from the restored live window so the
        // fast path stays effective after a reload.
        let mut bit_filter = SlidingBitFilter::new(window_capacity.max(64) * 8);
        for n in dense_cache.keys() {
            bit_filter.set(n);
        }

        Ok(Self {
            epoch_key,
            identity_key,
            epoch_generation,
            bit_filter,
            dense_cache,
            nullifier_history: nullifier_history_map,
            wire_window: wire_window_set,
            wire_history: wire_history_map,
            window_capacity,
        })
    }
}

/// Hex-encode a 32-byte token for error messages.
fn to_hex(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Derive the rotation-stable wire-identity key from the construction epoch key.
fn derive_identity_key(epoch_key: &EpochKey) -> EpochKey {
    let mut hasher = blake3::Hasher::new_derive_key("affidavit/ctrt-wire-identity/v1");
    hasher.update(epoch_key);
    *hasher.finalize().as_bytes()
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
    fn test_epoch_rotation_refuses_cross_rotation_replay() {
        let key1 = [0x01; 32];
        let key2 = [0x02; 32];
        let mut registry = ReplayNullifierRegistry::new(key1, 100);

        let receipt = b"receipt-payload";
        assert!(registry.check_and_admit(receipt).is_ok());
        assert!(registry.check_and_admit(receipt).is_err());

        // Rotate epoch: the wire is retired into history at generation 0.
        registry.rotate_epoch(key2);
        assert_eq!(registry.epoch_generation(), 1);

        // SK5 falsifier fixed: the original wire MUST be refused after
        // rotation, with the typed epoch-mismatch code.
        let err = registry
            .check_and_admit(receipt)
            .expect_err("cross-rotation replay must be refused");
        assert_eq!(err.machine_code(), CODE_REPLAY_EPOCH_MISMATCH);
        assert!(matches!(
            err,
            ReplayNullifierError::ReplayEpochMismatch(_)
        ));

        // Fresh wires remain admissible in the new generation.
        assert!(registry.check_and_admit(b"receipt-payload-v2").is_ok());
    }

    #[test]
    fn test_capacity_rotation_refuses_cross_rotation_replay() {
        let key = [0xaa; 32];
        let capacity = 8;
        let mut registry = ReplayNullifierRegistry::new(key, capacity);

        let original = b"original-receipt-wire";
        assert!(registry.check_and_admit(original).is_ok());

        // Fill to capacity, then overflow to trigger a generation advance.
        for i in 1..capacity {
            let data = format!("filler_{i}");
            assert!(registry.check_and_admit(data.as_bytes()).is_ok());
        }
        assert!(registry.check_and_admit(b"overflow-trigger").is_ok());
        assert_eq!(registry.epoch_generation(), 1);

        // The pre-rotation original is refused across the capacity rotation.
        let err = registry
            .check_and_admit(original)
            .expect_err("post-eviction replay must be refused");
        assert_eq!(err.machine_code(), CODE_REPLAY_EPOCH_MISMATCH);
    }

    #[test]
    fn test_within_window_replay_still_refused_after_fix() {
        let key = [0x07; 32];
        let mut registry = ReplayNullifierRegistry::new(key, 100);
        let wire = b"same-window-wire";
        assert!(registry.check_and_admit(wire).is_ok());
        let err = registry.check_and_admit(wire).unwrap_err();
        assert_eq!(err.machine_code(), CODE_REPLAY_DETECTED);
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

        let err4 = ReplayNullifierError::ReplayEpochMismatch("gen 0 vs gen 1".to_string());
        assert_eq!(err4.machine_code(), 0xE103);
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

        let n_new = compute_nullifier(registry.epoch_key(), initial_receipt);
        assert_ne!(n_orig, n_new, "Key rotation changes nullifier for previously seen receipt");

        // And the retired wire is refused, not amnestied.
        let err = registry.check_and_admit(initial_receipt).unwrap_err();
        assert_eq!(err.machine_code(), CODE_REPLAY_EPOCH_MISMATCH);
    }

    #[test]
    fn test_retired_nullifier_admit_refused_with_epoch_mismatch() {
        let key = [0x0f; 32];
        let mut registry = ReplayNullifierRegistry::new(key, 4);

        let n = compute_nullifier(&key, b"direct-nullifier-wire");
        assert!(registry.admit_nullifier(n).is_ok());

        for i in 0..4 {
            let f: Nullifier = {
                let mut b = [0u8; 32];
                b[0] = i;
                b
            };
            assert!(registry.admit_nullifier(f).is_ok());
        }
        assert_eq!(registry.epoch_generation(), 1);
        assert!(registry.nullifier_history_len() >= 1);

        // Re-admitting the retired pre-rotation nullifier is a typed refusal.
        let err = registry.admit_nullifier(n).unwrap_err();
        assert_eq!(err.machine_code(), CODE_REPLAY_EPOCH_MISMATCH);
    }

    #[test]
    fn test_snapshot_round_trip_preserves_replay_state() {
        let key = [0x21; 32];
        let mut registry = ReplayNullifierRegistry::new(key, 8);
        let wire = b"round-trip-wire";
        assert!(registry.check_and_admit(wire).is_ok());

        let bytes = registry.to_bytes();
        let mut restored = ReplayNullifierRegistry::load_from(&bytes).expect("round trip");
        assert_eq!(restored.epoch_generation(), 0);
        assert!(restored.wire_seen(wire));
        assert!(restored.check_and_admit(wire).is_err());

        // Round-trip through a rotation, too.
        let mut rotated = ReplayNullifierRegistry::new(key, 8);
        rotated.rotate_epoch([0x22; 32]);
        assert!(rotated.check_and_admit(b"post-rotation-wire").is_ok());
        let bytes = rotated.to_bytes();
        let mut restored = ReplayNullifierRegistry::load_from(&bytes).expect("round trip");
        assert_eq!(restored.epoch_generation(), 1);
    }

    #[test]
    fn test_load_from_typed_errors() {
        let mut registry = ReplayNullifierRegistry::new([0x31; 32], 8);
        assert!(registry.check_and_admit(b"x-wire").is_ok());
        let good = registry.to_bytes();

        // Truncation -> InvalidPersistenceStorage
        for cut in [0usize, 1, 40, SNAPSHOT_HEADER_LEN - 1, good.len() / 2] {
            let err = ReplayNullifierRegistry::load_from(&good[..cut]).unwrap_err();
            assert_eq!(
                err.machine_code(),
                CODE_PERSISTENCE,
                "truncation at {cut} must be typed InvalidPersistenceStorage"
            );
            assert!(matches!(
                err,
                ReplayNullifierError::InvalidPersistenceStorage(_)
            ));
        }

        // Truncation inside the record section -> InvalidPersistenceStorage
        let err = ReplayNullifierRegistry::load_from(&good[..good.len() - 1]).unwrap_err();
        assert!(matches!(
            err,
            ReplayNullifierError::InvalidPersistenceStorage(_)
        ));

        // Bad magic -> CorruptLedger
        let mut bad_magic = good.clone();
        bad_magic[0] = b'X';
        let err = ReplayNullifierRegistry::load_from(&bad_magic).unwrap_err();
        assert!(matches!(err, ReplayNullifierError::CorruptLedger(_)));

        // Unsupported version -> CorruptLedger
        let mut bad_version = good.clone();
        bad_version[4] = 0x7F;
        let err = ReplayNullifierRegistry::load_from(&bad_version).unwrap_err();
        assert!(matches!(err, ReplayNullifierError::CorruptLedger(_)));

        // Trailing bytes -> InvalidPersistenceStorage
        let mut trailing = good.clone();
        trailing.extend_from_slice(&[0u8; 3]);
        let err = ReplayNullifierRegistry::load_from(&trailing).unwrap_err();
        assert!(matches!(
            err,
            ReplayNullifierError::InvalidPersistenceStorage(_)
        ));
    }

    #[test]
    fn test_v1_snapshot_refused_fail_closed() {
        // Hand-build a v1 (version field = 1) header over a valid v2 body
        // shape: v1 must be refused, not silently accepted with an empty
        // nullifier_history.
        let mut registry = ReplayNullifierRegistry::new([0x77; 32], 8);
        assert!(registry.check_and_admit(b"v1-wire").is_ok());
        let mut v1 = registry.to_bytes();
        v1[4] = 1; // version field: v1
        let err = ReplayNullifierRegistry::load_from(&v1).unwrap_err();
        assert!(matches!(
            err,
            ReplayNullifierError::InvalidPersistenceStorage(_)
        ));
        assert!(
            err.to_string()
                .contains("v1 snapshot cannot enforce epoch binding; regenerate"),
            "v1 refusal must name the reason, got: {err}"
        );
    }

    /// SX4-A: rotate → to_bytes/load_from → re-admit the pre-rotation wire
    /// must REFUSE with 0xE103. The retired-nullifier layer must survive the
    /// persistence round trip (v1 dropped it; v2 serializes it).
    #[test]
    fn test_sx4a_rotate_save_load_readmit_refused() {
        let mut registry = ReplayNullifierRegistry::new([0x88; 32], 8);
        let wire = b"sx4a-pre-rotation-wire";
        assert!(registry.check_and_admit(wire).is_ok());
        registry.rotate_epoch([0x89; 32]);
        assert!(registry.nullifier_history_len() >= 1);

        let bytes = registry.to_bytes();
        let mut restored = ReplayNullifierRegistry::load_from(&bytes).expect("round trip");
        assert_eq!(restored.nullifier_history_len(), registry.nullifier_history_len());
        let err = restored
            .check_and_admit(wire)
            .expect_err("cross-generation replay after reload must be refused");
        assert_eq!(err.machine_code(), CODE_REPLAY_EPOCH_MISMATCH);
    }

    /// SX4-E: a forged byte in the sealed snapshot body (here: a generation
    /// byte in the wire-history section) must surface CorruptLedger via the
    /// HMAC-BLAKE3 tag check — never silent acceptance.
    #[test]
    fn test_sx4e_forged_body_byte_is_corrupt_ledger() {
        let mac_key = [0x5Eu8; 32];
        let mut registry = ReplayNullifierRegistry::new([0x8A; 32], 8);
        assert!(registry.check_and_admit(b"sx4e-wire").is_ok());
        registry.rotate_epoch([0x8B; 32]);
        let sealed = registry.seal(&mac_key);

        // Forge a generation byte near the tail (wire-history/nullifier-history
        // region) without re-tagging.
        let mut forged = sealed.clone();
        let last = forged.len() - 1;
        forged[last] ^= 0x01;
        let err = ReplayNullifierRegistry::unseal(&forged, &mac_key).unwrap_err();
        assert!(matches!(err, ReplayNullifierError::CorruptLedger(_)));
        assert_eq!(err.machine_code(), CODE_PERSISTENCE);

        // A truncated body must also fail the authentication check.
        let err = ReplayNullifierRegistry::unseal(&sealed[..sealed.len() - 1], &mac_key)
            .unwrap_err();
        assert!(matches!(err, ReplayNullifierError::CorruptLedger(_)));

        // The honest sealed snapshot still round-trips and refuses replay.
        let restored = ReplayNullifierRegistry::unseal(&sealed, &mac_key).expect("round trip");
        assert!(restored.wire_seen(b"sx4e-wire"));
    }

    #[test]
    fn test_wire_identity_key_stable_across_rotation() {
        let mut registry = ReplayNullifierRegistry::new([0x41; 32], 8);
        let k0 = *registry.identity_key();
        registry.rotate_epoch([0x42; 32]);
        assert_eq!(k0, *registry.identity_key());
    }

    #[cfg(feature = "crypto-trust")]
    #[test]
    fn test_generated_epoch_key_is_entropy_not_constant() {
        let r1 = ReplayNullifierRegistry::with_generated_key(64).expect("entropy");
        let r2 = ReplayNullifierRegistry::with_generated_key(64).expect("entropy");
        assert_ne!(r1.epoch_key(), r2.epoch_key(), "keys must draw from entropy");
        assert_ne!(r1.epoch_key(), &[0u8; 32]);
    }

    #[cfg(feature = "crypto-trust")]
    #[test]
    fn test_generated_key_construction_refuses_replay_like_fixed_key() {
        let mut r = ReplayNullifierRegistry::with_generated_key(64).expect("entropy");
        let wire = b"entropy-construction-wire";
        assert!(r.check_and_admit(wire).is_ok());
        assert!(r.check_and_admit(wire).is_err());
    }
}
