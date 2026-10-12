//! Strict 352-byte CTRT (Court Receipt Token) binary wire format.
//!
//! Provides zero-copy compatible, fixed-length canonical serialization and
//! validation for Sovereign Court receipts as specified in
//! STRATA-SPEC-26.10.12-AFFIDAVIT-SOVEREIGN.
//!
//! Layout (strictly 352 bytes):
//! - Magic: `*b"CTRT"` (0..4)
//! - Version: `1 u16 LE` (4..6)
//! - parent_receipt_hash: `[u8; 32]` (6..38)
//! - court_id: `[u8; 32]` (38..70)
//! - subject_merkle_root: `[u8; 32]` (70..102)
//! - compiler_binary_digest: `[u8; 32]` (102..134)
//! - execution_ticks: `u64 LE` (134..142)
//! - elapsed_cycles: `u64 LE` (142..150)
//! - verdict_status: `u8` (150)
//! - payload & metadata: `[u8; 201]` (151..352)

use thiserror::Error;

/// Exact byte length of a CTRT wire token.
pub const CTRT_WIRE_SIZE: usize = 352;

/// Magic identifier bytes for CTRT wire receipts: `b"CTRT"`.
pub const CTRT_MAGIC: [u8; 4] = *b"CTRT";

/// Canonical wire version 1.
pub const CTRT_VERSION_1: u16 = 1;

/// Payload & metadata field size in bytes.
pub const CTRT_PAYLOAD_SIZE: usize = 201;

/// Errors arising from CTRT wire decoding or validation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WireCtrtError {
    /// Provided slice does not match the exact 352-byte length requirement.
    #[error("invalid wire length: expected {CTRT_WIRE_SIZE} bytes, found {0}")]
    InvalidLength(usize),

    /// Magic bytes did not match `CTRT`.
    #[error("invalid magic: expected {:?}, found {:?}", CTRT_MAGIC, .0)]
    InvalidMagic([u8; 4]),

    /// Unsupported wire version.
    #[error("unsupported version: expected {CTRT_VERSION_1}, found {0}")]
    UnsupportedVersion(u16),

    /// Verdict status byte is not recognized.
    #[error("invalid verdict status byte: {0}")]
    InvalidVerdictStatus(u8),
}

impl WireCtrtError {
    /// Return the canonical numeric machine code corresponding to this error.
    ///
    /// - `InvalidLength` => `0xE001`
    /// - `InvalidMagic` => `0xE002`
    /// - `UnsupportedVersion` => `0xE003`
    /// - `InvalidVerdictStatus` => `0xE004`
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::InvalidLength(_) => 0xE001,
            Self::InvalidMagic(_) => 0xE002,
            Self::UnsupportedVersion(_) => 0xE003,
            Self::InvalidVerdictStatus(_) => 0xE004,
        }
    }
}

/// Constant-time branchless byte slice equality check using bitwise accumulator.
///
/// Returns `1` if slices are equal in length and content, `0` otherwise.
#[inline]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> u8 {
    if a.len() != b.len() {
        return 0;
    }
    let mut diff: u8 = 0;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    // diff is 0 iff equal. (diff as u16 | -(diff as i16) as u16) >> 15 gives 1 if diff != 0, 0 if diff == 0.
    // Or in u8:
    let diff_nz = ((diff | diff.wrapping_neg()) >> 7) & 1;
    diff_nz ^ 1
}

/// Constant-time branchless check whether magic bytes match [`CTRT_MAGIC`].
///
/// Returns `1` if valid, `0` otherwise.
#[inline]
pub fn ct_validate_magic(magic: &[u8; 4]) -> u8 {
    constant_time_eq(magic, &CTRT_MAGIC)
}

/// Constant-time branchless check whether version matches [`CTRT_VERSION_1`].
///
/// Returns `1` if valid, `0` otherwise.
#[inline]
pub fn ct_validate_version(version: u16) -> u8 {
    let diff = version ^ CTRT_VERSION_1;
    let diff_nz = ((diff | diff.wrapping_neg()) >> 15) as u8 & 1;
    diff_nz ^ 1
}

/// Constant-time branchless check whether a verdict status byte is valid (0..=3).
///
/// Returns `1` if valid, `0` otherwise.
#[inline]
pub fn ct_validate_verdict_status(status: u8) -> u8 {
    // Valid values are 0, 1, 2, 3 (i.e. status <= 3, meaning high 6 bits are 0).
    // (status >> 2) must be 0.
    let high_bits = status >> 2;
    let high_nz = ((high_bits | high_bits.wrapping_neg()) >> 7) & 1;
    high_nz ^ 1
}

/// Known verdict status codes for CTRT wire receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum VerdictStatus {
    /// Refused / Failed.
    Refused = 0,
    /// Accepted / Certified.
    Accepted = 1,
    /// Pending verification / provisional.
    Pending = 2,
    /// Inconclusive / Undecidable.
    Inconclusive = 3,
}

impl VerdictStatus {
    /// Convert a raw status byte into a [`VerdictStatus`].
    pub fn from_u8(value: u8) -> Result<Self, WireCtrtError> {
        if ct_validate_verdict_status(value) == 0 {
            return Err(WireCtrtError::InvalidVerdictStatus(value));
        }
        match value {
            0 => Ok(Self::Refused),
            1 => Ok(Self::Accepted),
            2 => Ok(Self::Pending),
            3 => Ok(Self::Inconclusive),
            _ => unreachable!(),
        }
    }

    /// Convert into raw status byte.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }
}

/// A decoded CTRT (Court Receipt Token) structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireCtrt {
    /// Wire format version (must be 1).
    pub version: u16,
    /// 32-byte cryptographic hash of the parent receipt.
    pub parent_receipt_hash: [u8; 32],
    /// 32-byte identifier of the issuing court.
    pub court_id: [u8; 32],
    /// 32-byte root of the subject Merkle tree.
    pub subject_merkle_root: [u8; 32],
    /// 32-byte hash / digest of the executing compiler binary.
    pub compiler_binary_digest: [u8; 32],
    /// Execution ticks elapsed during verification/compilation.
    pub execution_ticks: u64,
    /// CPU / hardware cycles elapsed.
    pub elapsed_cycles: u64,
    /// Verdict status code.
    pub verdict_status: VerdictStatus,
    /// Fixed 201-byte payload and metadata envelope.
    pub payload: [u8; CTRT_PAYLOAD_SIZE],
}

impl WireCtrt {
    /// Create a new `WireCtrt` instance with default version 1.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent_receipt_hash: [u8; 32],
        court_id: [u8; 32],
        subject_merkle_root: [u8; 32],
        compiler_binary_digest: [u8; 32],
        execution_ticks: u64,
        elapsed_cycles: u64,
        verdict_status: VerdictStatus,
        payload: [u8; CTRT_PAYLOAD_SIZE],
    ) -> Self {
        Self {
            version: CTRT_VERSION_1,
            parent_receipt_hash,
            court_id,
            subject_merkle_root,
            compiler_binary_digest,
            execution_ticks,
            elapsed_cycles,
            verdict_status,
            payload,
        }
    }

    /// Serialize this receipt into an exact 352-byte array.
    #[must_use]
    pub fn serialize(&self) -> [u8; CTRT_WIRE_SIZE] {
        let mut out = [0u8; CTRT_WIRE_SIZE];

        out[0..4].copy_from_slice(&CTRT_MAGIC);
        out[4..6].copy_from_slice(&self.version.to_le_bytes());
        out[6..38].copy_from_slice(&self.parent_receipt_hash);
        out[38..70].copy_from_slice(&self.court_id);
        out[70..102].copy_from_slice(&self.subject_merkle_root);
        out[102..134].copy_from_slice(&self.compiler_binary_digest);
        out[134..142].copy_from_slice(&self.execution_ticks.to_le_bytes());
        out[142..150].copy_from_slice(&self.elapsed_cycles.to_le_bytes());
        out[150] = self.verdict_status.to_u8();
        out[151..352].copy_from_slice(&self.payload);

        out
    }

    /// Parse and validate a 352-byte slice into a `WireCtrt`.
    pub fn deserialize(bytes: &[u8]) -> Result<Self, WireCtrtError> {
        if bytes.len() != CTRT_WIRE_SIZE {
            return Err(WireCtrtError::InvalidLength(bytes.len()));
        }

        // Branchless law: all payload-byte checks accumulate into a single
        // error-sink mask with one return; no short-circuit if/else on
        // payload bytes in the decode hot path.
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        let magic_ok = ct_validate_magic(&magic);

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        let version_ok = ct_validate_version(version);

        let mut parent_receipt_hash = [0u8; 32];
        parent_receipt_hash.copy_from_slice(&bytes[6..38]);

        let mut court_id = [0u8; 32];
        court_id.copy_from_slice(&bytes[38..70]);

        let mut subject_merkle_root = [0u8; 32];
        subject_merkle_root.copy_from_slice(&bytes[70..102]);

        let mut compiler_binary_digest = [0u8; 32];
        compiler_binary_digest.copy_from_slice(&bytes[102..134]);

        let execution_ticks = u64::from_le_bytes(
            bytes[134..142]
                .try_into()
                .expect("slice length matches 8 bytes"),
        );

        let elapsed_cycles = u64::from_le_bytes(
            bytes[142..150]
                .try_into()
                .expect("slice length matches 8 bytes"),
        );

        let verdict_raw = bytes[150];
        let verdict_ok = ct_validate_verdict_status(verdict_raw);

        // Arithmetic-select the verdict variant (no branch on payload bytes);
        // when verdict_ok == 1 this equals VerdictStatus::from_u8(verdict_raw).
        const VERDICT_TABLE: [VerdictStatus; 4] = [
            VerdictStatus::Refused,
            VerdictStatus::Accepted,
            VerdictStatus::Pending,
            VerdictStatus::Inconclusive,
        ];
        let verdict_status = VERDICT_TABLE[(verdict_raw & 0x03) as usize];

        let mut payload = [0u8; CTRT_PAYLOAD_SIZE];
        payload.copy_from_slice(&bytes[151..352]);

        // Single error-sink: bitwise OR of error masks, one return point.
        let error_sink = (magic_ok ^ 1) | (version_ok ^ 1) | (verdict_ok ^ 1);
        if error_sink == 0 {
            Ok(Self {
                version,
                parent_receipt_hash,
                court_id,
                subject_merkle_root,
                compiler_binary_digest,
                execution_ticks,
                elapsed_cycles,
                verdict_status,
                payload,
            })
        } else {
            // Priority select on the accumulated mask (post-hot-path only).
            Err(if magic_ok == 0 {
                WireCtrtError::InvalidMagic(magic)
            } else if version_ok == 0 {
                WireCtrtError::UnsupportedVersion(version)
            } else {
                WireCtrtError::InvalidVerdictStatus(verdict_raw)
            })
        }
    }

    /// Compute the canonical BLAKE3 commitment of this wire receipt.
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let serialized = self.serialize();
        *blake3::hash(&serialized).as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ctrt() -> WireCtrt {
        WireCtrt::new(
            [0x11; 32],
            [0x22; 32],
            [0x33; 32],
            [0x44; 32],
            100_000,
            4_200_000,
            VerdictStatus::Accepted,
            [0x7a; CTRT_PAYLOAD_SIZE],
        )
    }

    #[test]
    fn test_ctrt_roundtrip() {
        let original = sample_ctrt();
        let bytes = original.serialize();
        assert_eq!(bytes.len(), CTRT_WIRE_SIZE);

        let decoded = WireCtrt::deserialize(&bytes).expect("deserialization succeeds");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_ctrt_exact_length_rejection() {
        let bytes = sample_ctrt().serialize();
        assert!(matches!(
            WireCtrt::deserialize(&bytes[..351]),
            Err(WireCtrtError::InvalidLength(351))
        ));

        let mut over = bytes.to_vec();
        over.push(0x00);
        assert!(matches!(
            WireCtrt::deserialize(&over),
            Err(WireCtrtError::InvalidLength(353))
        ));
    }

    #[test]
    fn test_ctrt_magic_rejection() {
        let mut bytes = sample_ctrt().serialize();
        bytes[0] = b'X';
        assert!(matches!(
            WireCtrt::deserialize(&bytes),
            Err(WireCtrtError::InvalidMagic(_))
        ));
    }

    #[test]
    fn test_ctrt_version_rejection() {
        let mut bytes = sample_ctrt().serialize();
        bytes[4] = 2; // version 2
        assert!(matches!(
            WireCtrt::deserialize(&bytes),
            Err(WireCtrtError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn test_ctrt_verdict_rejection() {
        let mut bytes = sample_ctrt().serialize();
        bytes[150] = 99; // invalid status
        assert!(matches!(
            WireCtrt::deserialize(&bytes),
            Err(WireCtrtError::InvalidVerdictStatus(99))
        ));
    }

    #[test]
    fn test_full_352_byte_tamper_matrix() {
        let original = sample_ctrt();
        let serialized = original.serialize();
        assert_eq!(serialized.len(), CTRT_WIRE_SIZE);

        // Every single byte modified out of 352 must either:
        // 1. Fail deserialization directly (for header fields with strict validation: magic, version, verdict_status), OR
        // 2. Result in a deserialized record that differs from the original (tamper detection), AND produce a different blake3 digest.
        for byte_idx in 0..CTRT_WIRE_SIZE {
            let mut tampered = serialized;
            tampered[byte_idx] ^= 0x01; // flip 1 bit

            match WireCtrt::deserialize(&tampered) {
                Ok(decoded) => {
                    // If it parses, it MUST NOT match original
                    assert_ne!(
                        decoded, original,
                        "tampered byte at offset {byte_idx} silently matched original"
                    );
                    assert_ne!(
                        decoded.digest(),
                        original.digest(),
                        "tampered byte at offset {byte_idx} resulted in duplicate digest"
                    );
                }
                Err(err) => {
                    // Expected error for strict constraints
                    match byte_idx {
                        0..4 => assert!(matches!(err, WireCtrtError::InvalidMagic(_))),
                        4..6 => assert!(matches!(err, WireCtrtError::UnsupportedVersion(_))),
                        150 => assert!(matches!(err, WireCtrtError::InvalidVerdictStatus(_))),
                        _ => panic!("unexpected deserialization error at offset {byte_idx}: {err}"),
                    }
                }
            }
        }
    }

    #[test]
    fn test_strict_tamper_matrix_all_fail_deserialization_or_digest() {
        let original = sample_ctrt();
        let orig_digest = original.digest();
        let serialized = original.serialize();

        for i in 0..CTRT_WIRE_SIZE {
            let mut mutated = serialized;
            mutated[i] ^= 0xff; // bitwise inversion

            let res = WireCtrt::deserialize(&mutated);
            if let Ok(deserialized) = res {
                assert_ne!(
                    deserialized.digest(),
                    orig_digest,
                    "offset {i} failed to perturb digest"
                );
            }
        }
    }

    #[test]
    fn test_wire_ctrt_error_machine_codes() {
        assert_eq!(WireCtrtError::InvalidLength(100).machine_code(), 0xE001);
        assert_eq!(WireCtrtError::InvalidMagic(*b"ABCD").machine_code(), 0xE002);
        assert_eq!(WireCtrtError::UnsupportedVersion(2).machine_code(), 0xE003);
        assert_eq!(WireCtrtError::InvalidVerdictStatus(42).machine_code(), 0xE004);
    }

    #[test]
    fn test_constant_time_validation_helpers() {
        // magic validation
        assert_eq!(ct_validate_magic(b"CTRT"), 1);
        assert_eq!(ct_validate_magic(b"CTRX"), 0);
        assert_eq!(ct_validate_magic(b"XTRT"), 0);

        // version validation
        assert_eq!(ct_validate_version(1), 1);
        assert_eq!(ct_validate_version(0), 0);
        assert_eq!(ct_validate_version(2), 0);
        assert_eq!(ct_validate_version(u16::MAX), 0);

        // verdict status validation (0..=3 valid)
        for valid in 0..=3 {
            assert_eq!(ct_validate_verdict_status(valid), 1);
        }
        for invalid in 4..=255 {
            assert_eq!(ct_validate_verdict_status(invalid), 0);
        }

        // constant_time_eq general helper
        assert_eq!(constant_time_eq(b"hello", b"hello"), 1);
        assert_eq!(constant_time_eq(b"hello", b"world"), 0);
        assert_eq!(constant_time_eq(b"hello", b"hell"), 0);
    }
}

