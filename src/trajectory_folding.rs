//! Trajectory folding: associative binary operator (R_A ⊕ R_B) over 352-byte CTRT receipts.
//!
//! Monotonically folds ticks, cycles/wall time, and subject/metadata accumulators while
//! enforcing strict parent hash linkage: `B.parent_receipt_hash == A.chain_hash`.
//! Emits a folded 352-byte CTRT boundary receipt.
//!
//! ## Fold semantics (associative over contiguous chains)
//!
//! The gate `b.parent_receipt_hash == a.chain_hash` is unchanged; tamper detection is
//! preserved because any relinked/substituted interior receipt changes its chain hash,
//! which cascades to the known tip hash. To make the operator associative, the folded
//! receipt is a *summary anchored at the chain root that speaks for the chain tip*:
//!
//! - `parent_receipt_hash = a.parent_receipt_hash` (the chain root's parent)
//! - `chain_hash = b.chain_hash` (the tip's chain hash — the linkage the next fold checks)
//! - `run_id`, `timestamp_ms`, `name`, `prev_self_hash` = `b`'s (tip identity)
//! - `total_ticks`, `wall_ms` = saturating sums
//! - `outcome`, `thermal` = max (fail-closed); `budget_met` = AND; flags = OR
//! - `subject_hash`, `metadata_digest`, `positive_control` = XOR accumulators
//!   (associative combining; ordering/mandate enforcement lives in the linkage gate)
//!
//! ## Known limitations
//!
//! - **Folded receipts are summaries, not standalone records**: because `chain_hash`
//!   aliases the tip's hash, `CtrtReceipt::from_bytes` on a folded receipt returns
//!   `DigestMismatch` by design. Verify folded output against the independently known
//!   tip hash, or retain the tip receipt.
//! - **Saturating accumulation**: `total_ticks`/`wall_ms` use `saturating_add`; overflow
//!   silently clamps to `u64::MAX` with no flag (the 352-byte wire layout has no spare
//!   field). Sums are associative only below the saturation point.
//! - **Unsigned domain**: folding operates on unsigned chains only; negative/credited
//!   quantities are unrepresentable. Enforcement that folded output binds only to the
//!   original subject chain is the sealer's subject-only commitment, not this module's.
//! - Names longer than `NAME_MAX` bytes are truncated at the last UTF-8 char boundary
//!   at or below `NAME_MAX` (never mid-character).

use thiserror::Error;

/// Fixed-size constants matching the CTRT wire standard (352 bytes).
pub const COURT_RECORD_LEN: usize = 352;
pub const COURT_RECEIPT_V1_LEN: usize = 320;
pub const HEADER_LEN: usize = 16;
pub const BODY_LEN: usize = 304;
pub const TRAILER_LEN: usize = 16;
pub const NAME_LEN: usize = 100;
pub const NAME_MAX: usize = NAME_LEN - TRAILER_LEN; // 84 bytes

pub const MAGIC: [u8; 4] = *b"CTRT";
pub const VERSION: u16 = 1;

// Body-relative offsets
const B_SUBJECT: usize = 8;
const B_PARENT: usize = 40;
const B_TIMESTAMP: usize = 72;
const B_RUN_ID: usize = 80;
const B_TOTAL_TICKS: usize = 88;
const B_WALL_MS: usize = 96;
const B_THERMAL: usize = 104;
const B_OUTCOME: usize = 105;
const B_BUDGET: usize = 106;
const B_PAD: usize = 107;
const B_CONTROL: usize = 108;
const B_METADATA: usize = 140;
const B_PREV_SELF: usize = 172;
const B_NAME: usize = 204;

const BODY_PREFIX_LEN: usize = 8;
const FLAG_POSITIVE_CONTROL: u8 = 1 << 0;
const FLAG_FAIL_CLOSED: u8 = 1 << 1;
const FLAG_KNOWN_MASK: u8 = FLAG_POSITIVE_CONTROL | FLAG_FAIL_CLOSED;

/// Errors returned by trajectory folding and CTRT receipt decoding.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TrajectoryFoldError {
    #[error("invalid receipt length: expected {expected}, found {actual}")]
    InvalidLength { expected: usize, actual: usize },
    #[error("bad magic: expected CTRT, found {0:?}")]
    BadMagic([u8; 4]),
    #[error("unsupported version: found {found}, supported {supported}")]
    UnsupportedVersion { found: u16, supported: u16 },
    #[error("reserved field non-zero at offset {offset}: value {value}")]
    ReservedNonzero { offset: usize, value: u64 },
    #[error("body length mismatch: expected {expected}, found {actual}")]
    BodyLenMismatch { expected: u64, actual: u64 },
    #[error("invalid enum value for field {field}: {found}")]
    InvalidEnum { field: &'static str, found: u8 },
    #[error("control flag mismatch at offset {offset}")]
    ControlFlagMismatch { offset: usize },
    #[error("digest mismatch: computed hash does not match stored chain hash")]
    DigestMismatch,
    #[error("zero trailer violation in name field")]
    ZeroTrailerViolation,
    #[error("parent hash linkage violation: B.parent_receipt_hash ({found}) does not match BLAKE3(A) ({expected})")]
    ParentLinkageMismatch { expected: String, found: String },
    #[error("incompatible verdict: outcome or verdict status mismatch during folding")]
    IncompatibleVerdict,
    #[error("invalid name")]
    InvalidName,
}

impl TrajectoryFoldError {
    /// Return the canonical machine code for structured error classification.
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::ParentLinkageMismatch { .. } => 0xE201,
            Self::IncompatibleVerdict => 0xE202,
            Self::InvalidLength { .. } => 0xE203,
            Self::BadMagic(_) => 0xE204,
            Self::UnsupportedVersion { .. } => 0xE205,
            Self::ReservedNonzero { .. } => 0xE206,
            Self::BodyLenMismatch { .. } => 0xE207,
            Self::InvalidEnum { .. } => 0xE208,
            Self::ControlFlagMismatch { .. } => 0xE209,
            Self::DigestMismatch => 0xE20A,
            Self::ZeroTrailerViolation => 0xE20B,
            Self::InvalidName => 0xE20C,
        }
    }
}

/// A parsed 352-byte CTRT boundary receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtrtReceipt {
    pub flags: u8,
    pub subject_hash: [u8; 32],
    pub parent_receipt_hash: [u8; 32],
    pub timestamp_ms: u64,
    pub run_id: u64,
    pub total_ticks: u64,
    pub wall_ms: u64,
    pub thermal: u8,
    pub outcome: u8,
    pub budget_met: bool,
    pub positive_control: [u8; 32],
    pub metadata_digest: [u8; 32],
    pub prev_self_hash: [u8; 32],
    pub name: String,
    pub chain_hash: [u8; 32],
}

impl CtrtReceipt {
    /// Compute the chain hash: `BLAKE3(parent_receipt_hash ‖ encoded_bytes[0..304])`.
    pub fn compute_chain_hash(parent: &[u8; 32], encoded: &[u8; COURT_RECEIPT_V1_LEN]) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(parent);
        hasher.update(&encoded[..304]);
        *hasher.finalize().as_bytes()
    }

    /// Decode and verify a 352-byte CTRT receipt slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TrajectoryFoldError> {
        if bytes.len() != COURT_RECORD_LEN {
            return Err(TrajectoryFoldError::InvalidLength {
                expected: COURT_RECORD_LEN,
                actual: bytes.len(),
            });
        }

        // Header check
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        if magic != MAGIC {
            return Err(TrajectoryFoldError::BadMagic(magic));
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        if version != VERSION {
            return Err(TrajectoryFoldError::UnsupportedVersion {
                found: version,
                supported: VERSION,
            });
        }

        let flags = bytes[6];
        if flags & !FLAG_KNOWN_MASK != 0 {
            return Err(TrajectoryFoldError::ReservedNonzero {
                offset: 6,
                value: flags as u64,
            });
        }

        let reserved = bytes[7];
        if reserved != 0 {
            return Err(TrajectoryFoldError::ReservedNonzero {
                offset: 7,
                value: reserved as u64,
            });
        }

        let body_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if body_len != BODY_LEN as u64 {
            return Err(TrajectoryFoldError::BodyLenMismatch {
                expected: BODY_LEN as u64,
                actual: body_len,
            });
        }

        // Body checks
        let body = &bytes[HEADER_LEN..COURT_RECEIPT_V1_LEN];
        if body[..BODY_PREFIX_LEN].iter().any(|&b| b != 0) {
            let offset = HEADER_LEN + body[..BODY_PREFIX_LEN].iter().position(|&b| b != 0).unwrap();
            return Err(TrajectoryFoldError::ReservedNonzero {
                offset,
                value: body[offset - HEADER_LEN] as u64,
            });
        }

        if body[B_PAD] != 0 {
            return Err(TrajectoryFoldError::ReservedNonzero {
                offset: HEADER_LEN + B_PAD,
                value: body[B_PAD] as u64,
            });
        }

        // Zero trailer check in name region (NAME_MAX..NAME_LEN)
        if body[B_NAME + NAME_MAX..B_NAME + NAME_LEN].iter().any(|&b| b != 0) {
            return Err(TrajectoryFoldError::ZeroTrailerViolation);
        }

        let mut subject_hash = [0u8; 32];
        subject_hash.copy_from_slice(&body[B_SUBJECT..B_SUBJECT + 32]);

        let mut parent_receipt_hash = [0u8; 32];
        parent_receipt_hash.copy_from_slice(&body[B_PARENT..B_PARENT + 32]);

        let timestamp_ms = u64::from_le_bytes(body[B_TIMESTAMP..B_TIMESTAMP + 8].try_into().unwrap());
        let run_id = u64::from_le_bytes(body[B_RUN_ID..B_RUN_ID + 8].try_into().unwrap());
        let total_ticks = u64::from_le_bytes(body[B_TOTAL_TICKS..B_TOTAL_TICKS + 8].try_into().unwrap());
        let wall_ms = u64::from_le_bytes(body[B_WALL_MS..B_WALL_MS + 8].try_into().unwrap());
        let thermal = body[B_THERMAL];
        let outcome = body[B_OUTCOME];
        let budget_met = match body[B_BUDGET] {
            0 => false,
            1 => true,
            bad => return Err(TrajectoryFoldError::InvalidEnum { field: "budget_met", found: bad }),
        };

        let mut positive_control = [0u8; 32];
        positive_control.copy_from_slice(&body[B_CONTROL..B_CONTROL + 32]);

        let control_present = positive_control != [0u8; 32];
        let flag_present = flags & FLAG_POSITIVE_CONTROL != 0;
        if control_present != flag_present {
            return Err(TrajectoryFoldError::ControlFlagMismatch { offset: 6 });
        }

        let mut metadata_digest = [0u8; 32];
        metadata_digest.copy_from_slice(&body[B_METADATA..B_METADATA + 32]);

        let mut prev_self_hash = [0u8; 32];
        prev_self_hash.copy_from_slice(&body[B_PREV_SELF..B_PREV_SELF + 32]);

        let name_bytes = &body[B_NAME..B_NAME + NAME_MAX];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(NAME_MAX);
        let name = std::str::from_utf8(&name_bytes[..name_end])
            .map_err(|_| TrajectoryFoldError::InvalidName)?
            .to_string();

        let mut stored_chain_hash = [0u8; 32];
        stored_chain_hash.copy_from_slice(&bytes[COURT_RECEIPT_V1_LEN..COURT_RECORD_LEN]);

        let mut receipt_content = [0u8; COURT_RECEIPT_V1_LEN];
        receipt_content.copy_from_slice(&bytes[..COURT_RECEIPT_V1_LEN]);
        let computed = Self::compute_chain_hash(&parent_receipt_hash, &receipt_content);
        if computed != stored_chain_hash {
            return Err(TrajectoryFoldError::DigestMismatch);
        }

        Ok(Self {
            flags,
            subject_hash,
            parent_receipt_hash,
            timestamp_ms,
            run_id,
            total_ticks,
            wall_ms,
            thermal,
            outcome,
            budget_met,
            positive_control,
            metadata_digest,
            prev_self_hash,
            name,
            chain_hash: stored_chain_hash,
        })
    }

    /// Encode into 352-byte CTRT boundary receipt record.
    pub fn encode(&self) -> Result<[u8; COURT_RECORD_LEN], TrajectoryFoldError> {
        let name_bytes = self.name.as_bytes();
        if name_bytes.len() > NAME_MAX || name_bytes.contains(&0) {
            return Err(TrajectoryFoldError::InvalidName);
        }

        let mut out = [0u8; COURT_RECORD_LEN];
        out[0..4].copy_from_slice(&MAGIC);
        out[4..6].copy_from_slice(&VERSION.to_le_bytes());
        out[6] = self.flags;
        out[7] = 0;
        out[8..16].copy_from_slice(&(BODY_LEN as u64).to_le_bytes());

        let body = &mut out[HEADER_LEN..COURT_RECEIPT_V1_LEN];
        body[B_SUBJECT..B_SUBJECT + 32].copy_from_slice(&self.subject_hash);
        body[B_PARENT..B_PARENT + 32].copy_from_slice(&self.parent_receipt_hash);
        body[B_TIMESTAMP..B_TIMESTAMP + 8].copy_from_slice(&self.timestamp_ms.to_le_bytes());
        body[B_RUN_ID..B_RUN_ID + 8].copy_from_slice(&self.run_id.to_le_bytes());
        body[B_TOTAL_TICKS..B_TOTAL_TICKS + 8].copy_from_slice(&self.total_ticks.to_le_bytes());
        body[B_WALL_MS..B_WALL_MS + 8].copy_from_slice(&self.wall_ms.to_le_bytes());
        body[B_THERMAL] = self.thermal;
        body[B_OUTCOME] = self.outcome;
        body[B_BUDGET] = if self.budget_met { 1 } else { 0 };
        body[B_PAD] = 0;
        body[B_CONTROL..B_CONTROL + 32].copy_from_slice(&self.positive_control);
        body[B_METADATA..B_METADATA + 32].copy_from_slice(&self.metadata_digest);
        body[B_PREV_SELF..B_PREV_SELF + 32].copy_from_slice(&self.prev_self_hash);

        body[B_NAME..B_NAME + name_bytes.len()].copy_from_slice(name_bytes);

        let mut content = [0u8; COURT_RECEIPT_V1_LEN];
        content.copy_from_slice(&out[..COURT_RECEIPT_V1_LEN]);
        let chain_hash = Self::compute_chain_hash(&self.parent_receipt_hash, &content);

        out[COURT_RECEIPT_V1_LEN..COURT_RECORD_LEN].copy_from_slice(&chain_hash);

        Ok(out)
    }
}

/// Associative binary operator (R_A ⊕ R_B) for folding trajectory receipts.
///
/// 1. Checks parent hash linkage: `b.parent_receipt_hash == a.chain_hash`
///    (where `a.chain_hash == BLAKE3(A.parent || A.encoded[..304])`).
/// 2. Monotonically sums total ticks and wall_ms (saturating; see module limitations).
/// 3. XOR-accumulates subject, metadata, and positive-control digests (associative).
/// 4. Anchors at the root (`parent = a.parent`) and speaks for the tip
///    (`chain_hash = b.chain_hash`); retains `b.run_id`, `b.timestamp_ms`, `b.name`,
///    `b.prev_self_hash`, worst-case outcome/thermal, AND-ed budget, OR-ed flags.
/// 5. Emits the folded 352-byte CTRT boundary receipt.
pub fn fold_receipts(a: &CtrtReceipt, b: &CtrtReceipt) -> Result<CtrtReceipt, TrajectoryFoldError> {
    if b.parent_receipt_hash != a.chain_hash {
        return Err(TrajectoryFoldError::ParentLinkageMismatch {
            expected: hex_encode(&a.chain_hash),
            found: hex_encode(&b.parent_receipt_hash),
        });
    }

    // Monotonic accumulation of ticks and wall_ms (saturating; see module limitations)
    let folded_ticks = a.total_ticks.saturating_add(b.total_ticks);
    let folded_wall_ms = a.wall_ms.saturating_add(b.wall_ms);

    // Associative XOR accumulation over subjects, metadata, and positive control.
    // Ordering and mandate enforcement is carried by the parent-linkage gate, not
    // by these combining digests.
    let mut folded_subject = a.subject_hash;
    let mut folded_metadata = a.metadata_digest;
    let mut positive_control = a.positive_control;
    for (acc, next) in [
        (&mut folded_subject, &b.subject_hash),
        (&mut folded_metadata, &b.metadata_digest),
        (&mut positive_control, &b.positive_control),
    ] {
        for (x, y) in acc.iter_mut().zip(next.iter()) {
            *x ^= y;
        }
    }

    let flags = if (a.flags & FLAG_POSITIVE_CONTROL != 0) || (b.flags & FLAG_POSITIVE_CONTROL != 0) {
        FLAG_POSITIVE_CONTROL | FLAG_FAIL_CLOSED
    } else {
        FLAG_FAIL_CLOSED
    };

    // Outcome: worst-case / fail-closed propagation
    let folded_outcome = a.outcome.max(b.outcome);
    let folded_thermal = a.thermal.max(b.thermal);
    let folded_budget = a.budget_met && b.budget_met;

    // Tip identity: keeps the folded summary associative. Concatenating names or
    // re-deriving chain hashes here would make the operator grouping-dependent.
    let name = truncate_to_char_boundary(&b.name, NAME_MAX);

    let folded = CtrtReceipt {
        flags,
        subject_hash: folded_subject,
        parent_receipt_hash: a.parent_receipt_hash, // Chain root's parent
        timestamp_ms: b.timestamp_ms,
        run_id: b.run_id,
        total_ticks: folded_ticks,
        wall_ms: folded_wall_ms,
        thermal: folded_thermal,
        outcome: folded_outcome,
        budget_met: folded_budget,
        positive_control,
        metadata_digest: folded_metadata,
        prev_self_hash: b.prev_self_hash,
        name,
        // Speaks for the chain tip: the next fold's linkage gate checks against this.
        chain_hash: b.chain_hash,
    };

    Ok(folded)
}

/// Truncate to at most `max` bytes without splitting a UTF-8 character:
/// floors to the last char boundary at or below `max`.
fn truncate_to_char_boundary(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Helper to hex encode a byte slice.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_receipt(parent: [u8; 32], ticks: u64, wall: u64, name: &str) -> CtrtReceipt {
        let mut r = CtrtReceipt {
            flags: FLAG_FAIL_CLOSED,
            subject_hash: *blake3::hash(name.as_bytes()).as_bytes(),
            parent_receipt_hash: parent,
            timestamp_ms: 1700000000000,
            run_id: 1,
            total_ticks: ticks,
            wall_ms: wall,
            thermal: 1,
            outcome: 0, // Pass
            budget_met: true,
            positive_control: [0u8; 32],
            metadata_digest: [0x55; 32],
            prev_self_hash: [0u8; 32],
            name: name.to_string(),
            chain_hash: [0u8; 32],
        };
        let encoded = r.encode().expect("valid encode");
        r.chain_hash.copy_from_slice(&encoded[COURT_RECEIPT_V1_LEN..]);
        r
    }

    #[test]
    fn test_ctrt_receipt_encode_decode_roundtrip() {
        let r = create_test_receipt([0u8; 32], 1000, 50, "step_0");
        let wire = r.encode().expect("encode");
        assert_eq!(wire.len(), COURT_RECORD_LEN);

        let decoded = CtrtReceipt::from_bytes(&wire).expect("decode");
        assert_eq!(decoded, r);
    }

    #[test]
    fn test_fold_associative_success() {
        let r1 = create_test_receipt([0u8; 32], 100, 10, "step_1");
        let r2 = create_test_receipt(r1.chain_hash, 200, 20, "step_2");
        let r3 = create_test_receipt(r2.chain_hash, 300, 30, "step_3");

        // (r1 ⊕ r2) ⊕ r3
        let r12 = fold_receipts(&r1, &r2).expect("fold 1+2");
        assert_eq!(r12.total_ticks, 300);
        assert_eq!(r12.wall_ms, 30);
        assert_eq!(r12.parent_receipt_hash, r1.parent_receipt_hash);
        let lhs = fold_receipts(&r12, &r3).expect("fold (1+2)+3");

        // r1 ⊕ (r2 ⊕ r3)
        let r23 = fold_receipts(&r2, &r3).expect("fold 2+3");
        let rhs = fold_receipts(&r1, &r23).expect("fold 1+(2+3)");

        // Associativity over a contiguous 3-chain: both groupings must agree exactly.
        assert_eq!(lhs, rhs);
        assert_eq!(lhs.total_ticks, 600);
        assert_eq!(lhs.wall_ms, 60);
        assert_eq!(lhs.parent_receipt_hash, r1.parent_receipt_hash);
        assert_eq!(lhs.chain_hash, r3.chain_hash);
    }

    #[test]
    fn test_fold_name_multibyte_no_panic() {
        // Multibyte (emoji) names: truncation must not slice mid-character.
        let r1 = create_test_receipt([0u8; 32], 10, 1, "🚀🚀🚀🚀🚀幸");
        let r2 = create_test_receipt(r1.chain_hash, 20, 2, "幸rel");
        let folded = fold_receipts(&r1, &r2).expect("fold");
        assert!(folded.name.len() <= NAME_MAX);
        // The folded name is the tip's name, already boundary-safe.
        assert_eq!(folded.name, "幸rel");

        // Direct check of the boundary-flooring helper on a multibyte string.
        let s = "あ".repeat(60); // 180 bytes, all 3-byte chars
        let t = truncate_to_char_boundary(&s, NAME_MAX);
        assert!(t.len() <= NAME_MAX);
        assert!(s.starts_with(&t));
        assert!(t.chars().all(|c| c == 'あ'));
    }

    #[test]
    fn test_fold_parent_linkage_mismatch() {
        let r1 = create_test_receipt([0u8; 32], 100, 10, "step_1");
        let r2 = create_test_receipt([0x99; 32], 200, 20, "step_2"); // disconnected parent

        let err = fold_receipts(&r1, &r2).unwrap_err();
        match err {
            TrajectoryFoldError::ParentLinkageMismatch { .. } => {}
            other => panic!("expected ParentLinkageMismatch, got {:?}", other),
        }
    }

    #[test]
    fn test_trajectory_fold_error_machine_codes() {
        let err_parent = TrajectoryFoldError::ParentLinkageMismatch {
            expected: "aa".to_string(),
            found: "bb".to_string(),
        };
        assert_eq!(err_parent.machine_code(), 0xE201);

        let err_verdict = TrajectoryFoldError::IncompatibleVerdict;
        assert_eq!(err_verdict.machine_code(), 0xE202);
    }
}
