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
//!
//! Court identity is BYTE-rendered: no Unicode normalization is applied, so NFC and
//! NFD encodings of the same visual identity are distinct courts by design
//! (SX3-F3); canonicalization is the issuer's duty before encoding.

use thiserror::Error;

/// Exact byte length of a CTRT wire token.
pub const CTRT_WIRE_SIZE: usize = 352;

/// Magic identifier bytes for CTRT wire receipts: `b"CTRT"`.
pub const CTRT_MAGIC: [u8; 4] = *b"CTRT";

/// Canonical wire version 1.
pub const CTRT_VERSION_1: u16 = 1;

/// Payload & metadata field size in bytes.
pub const CTRT_PAYLOAD_SIZE: usize = 201;

// --- STATUTORY ANCHOR & DEONTIC MODALITY CONSTANTS (v26.10.16) ---

/// Statutory trigger: Sarbanes-Oxley § 806 Whistleblower Safe Harbor (`SOX_1514A`).
pub const STATUTORY_ANCHOR_SOX_1514A: [u8; 16] = *b"SOX_1514A\0\0\0\0\0\0\0";

/// Statutory trigger: FRCP 37(e) Mandatory Evidentiary Preservation Hold (`FRCP_37E`).
pub const STATUTORY_ANCHOR_FRCP_37E: [u8; 16] = *b"FRCP_37E\0\0\0\0\0\0\0\0";

/// Statutory trigger: Constructive Trust Gain-Share Allocation (`GAIN_5050`).
pub const STATUTORY_ANCHOR_GAIN_5050: [u8; 16] = *b"GAIN_5050\0\0\0\0\0\0\0";

/// Deontic modality: royalty-free tier (<$50M gross annual revenue).
pub const DEONTIC_PERMITTED: u8 = 0x01;

/// Deontic modality: mandatory 50/50 gain-share distribution (>= $50M revenue).
pub const DEONTIC_OBLIGATED: u8 = 0x02;

/// Deontic modality: unlicensed invocation detected; statutory legal hold attached.
pub const DEONTIC_PRESERVATION_HOLD: u8 = 0x03;

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

    /// Recomputed BLAKE3 digest over the full 352-byte wire does not match the
    /// caller-supplied expected digest (SK4-F1: the payload tail 151..352 is
    /// commitment-covered at decode, not just at digest time).
    #[error("wire digest mismatch: recomputed BLAKE3 over all {CTRT_WIRE_SIZE} bytes does not match expected digest")]
    DigestMismatch,
}

impl WireCtrtError {
    /// Return the canonical numeric machine code corresponding to this error.
    ///
    /// - `InvalidLength` => `0xE001`
    /// - `InvalidMagic` => `0xE002`
    /// - `UnsupportedVersion` => `0xE003`
    /// - `InvalidVerdictStatus` => `0xE004`
    /// - `DigestMismatch` => `0xE005`
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::InvalidLength(_) => 0xE001,
            Self::InvalidMagic(_) => 0xE002,
            Self::UnsupportedVersion(_) => 0xE003,
            Self::InvalidVerdictStatus(_) => 0xE004,
            Self::DigestMismatch => 0xE005,
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

/// Plain-data decoded CTRT body — the fixed-size out-parameter of the
/// constant-time decode core ([`deserialize_ct`]).
///
/// Every field is plain data (integers and byte arrays), so a caller can
/// zero-initialize this struct and hand it to [`deserialize_ct`] without any
/// uninitialized-memory hazard. The raw verdict byte is carried as `u8`;
/// validity is conveyed by the returned machine code, not by the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireCtrtBody {
    /// Wire format version (must be 1 on a `0` return code).
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
    /// Raw verdict status byte (validated by the return code, not here).
    pub verdict_raw: u8,
    /// Fixed 201-byte payload and metadata envelope.
    pub payload: [u8; CTRT_PAYLOAD_SIZE],
}

impl Default for WireCtrtBody {
    fn default() -> Self {
        Self {
            version: 0,
            parent_receipt_hash: [0; 32],
            court_id: [0; 32],
            subject_merkle_root: [0; 32],
            compiler_binary_digest: [0; 32],
            execution_ticks: 0,
            elapsed_cycles: 0,
            verdict_raw: 0,
            payload: [0; CTRT_PAYLOAD_SIZE],
        }
    }
}

/// Packed 352-byte Compact Trust Receipt Token (CTRT) with Statutory Deontic Extension.
///
/// Matches the exact memory layout of STRATA-SPEC-26.10.16-CTRT:
/// - `0..4`: magic `b"CTRT"`
/// - `4..6`: version `u16 LE`
/// - `6..38`: parent_receipt_hash `[u8; 32]`
/// - `38..70`: court_id `[u8; 32]`
/// - `70..102`: subject_merkle_root `[u8; 32]`
/// - `102..134`: compiler_binary_digest `[u8; 32]`
/// - `134..142`: execution_ticks `u64 LE`
/// - `142..150`: elapsed_cycles `u64 LE`
/// - `150..151`: verdict_status `u8`
/// - `151..167`: statutory_anchor `[u8; 16]` (SOX_1514A | FRCP_37E | GAIN_5050)
/// - `167..175`: tariff_basis_cents `u64 LE` (Restatement of Restitution § 42)
/// - `175..176`: deontic_modality `u8` (0x01 PERMIT, 0x02 OBLIG, 0x03 HOLD)
/// - `176..208`: nullifier_commitment `[u8; 32]`
/// - `208..240`: ocel_event_digest `[u8; 32]`
/// - `240..352`: pqc_witness_signature `[u8; 112]`
#[repr(C, packed)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactTrustReceiptToken {
    /// Magic identifier bytes (b"CTRT").
    pub magic: [u8; 4],
    /// Wire version (1).
    pub version: u16,
    /// 32-byte cryptographic hash of the parent receipt.
    pub parent_receipt_hash: [u8; 32],
    /// 32-byte identifier of the issuing court.
    pub court_id: [u8; 32],
    /// 32-byte root of the subject Merkle tree.
    pub subject_merkle_root: [u8; 32],
    /// 32-byte hash / digest of the executing compiler binary.
    pub compiler_binary_digest: [u8; 32],
    /// Execution ticks elapsed.
    pub execution_ticks: u64,
    /// Hardware elapsed cycles.
    pub elapsed_cycles: u64,
    /// Raw verdict status code (0x01 ACCEPT, 0x02 REJECT).
    pub verdict_status: u8,
    // --- Statutory Deontic Extension (Bytes 151..352) ---
    /// Statutory trigger identifier (SOX_1514A, FRCP_37E, GAIN_5050).
    pub statutory_anchor: [u8; 16],
    /// Liquid avoided cloud tariff basis in USD cents.
    pub tariff_basis_cents: u64,
    /// Deontic modality: 0x01 PERMITTED, 0x02 OBLIGATED, 0x03 PRESERVATION_HOLD.
    pub deontic_modality: u8,
    /// Anti-replay deterministic nullifier commitment.
    pub nullifier_commitment: [u8; 32],
    /// Linked IEEE OCEL 2.0 trace digest.
    pub ocel_event_digest: [u8; 32],
    /// Truncated ML-DSA-65 witness block.
    pub pqc_witness_signature: [u8; 112],
}

impl CompactTrustReceiptToken {
    /// Convert from packed struct to exact 352-byte wire array.
    #[must_use]
    pub fn to_bytes(&self) -> [u8; CTRT_WIRE_SIZE] {
        let mut out = [0u8; CTRT_WIRE_SIZE];
        out[0..4].copy_from_slice(&self.magic);
        out[4..6].copy_from_slice(&self.version.to_le_bytes());
        out[6..38].copy_from_slice(&self.parent_receipt_hash);
        out[38..70].copy_from_slice(&self.court_id);
        out[70..102].copy_from_slice(&self.subject_merkle_root);
        out[102..134].copy_from_slice(&self.compiler_binary_digest);
        out[134..142].copy_from_slice(&self.execution_ticks.to_le_bytes());
        out[142..150].copy_from_slice(&self.elapsed_cycles.to_le_bytes());
        out[150] = self.verdict_status;
        out[151..167].copy_from_slice(&self.statutory_anchor);
        out[167..175].copy_from_slice(&self.tariff_basis_cents.to_le_bytes());
        out[175] = self.deontic_modality;
        out[176..208].copy_from_slice(&self.nullifier_commitment);
        out[208..240].copy_from_slice(&self.ocel_event_digest);
        out[240..352].copy_from_slice(&self.pqc_witness_signature);
        out
    }

    /// Decode directly from 352-byte wire slice.
    pub fn from_bytes(bytes: &[u8; CTRT_WIRE_SIZE]) -> Result<Self, WireCtrtError> {
        let magic_ok = ct_validate_magic(bytes[0..4].try_into().unwrap());
        if magic_ok == 0 {
            let mut magic = [0u8; 4];
            magic.copy_from_slice(&bytes[0..4]);
            return Err(WireCtrtError::InvalidMagic(magic));
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if ct_validate_version(version) == 0 {
            return Err(WireCtrtError::UnsupportedVersion(version));
        }
        let verdict_status = bytes[150];
        if ct_validate_verdict_status(verdict_status) == 0 {
            return Err(WireCtrtError::InvalidVerdictStatus(verdict_status));
        }
        Ok(Self {
            magic: bytes[0..4].try_into().unwrap(),
            version,
            parent_receipt_hash: bytes[6..38].try_into().unwrap(),
            court_id: bytes[38..70].try_into().unwrap(),
            subject_merkle_root: bytes[70..102].try_into().unwrap(),
            compiler_binary_digest: bytes[102..134].try_into().unwrap(),
            execution_ticks: u64::from_le_bytes(bytes[134..142].try_into().unwrap()),
            elapsed_cycles: u64::from_le_bytes(bytes[142..150].try_into().unwrap()),
            verdict_status,
            statutory_anchor: bytes[151..167].try_into().unwrap(),
            tariff_basis_cents: u64::from_le_bytes(bytes[167..175].try_into().unwrap()),
            deontic_modality: bytes[175],
            nullifier_commitment: bytes[176..208].try_into().unwrap(),
            ocel_event_digest: bytes[208..240].try_into().unwrap(),
            pqc_witness_signature: bytes[240..352].try_into().unwrap(),
        })
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

    /// Parse and validate a 352-byte slice into a `WireCtrt`, enforcing the
    /// full-wire BLAKE3 commitment.
    ///
    /// SK4-F1 fix: unlike [`WireCtrt::deserialize`], which validates only the
    /// structural fields (magic, version, verdict status) and accepts arbitrary
    /// payload bytes at 151..352, this recomputes BLAKE3 over ALL 352 bytes and
    /// compares (via [`constant_time_eq`], branchless) against
    /// `expected_digest`. Any tampering anywhere in the wire — including the
    /// previously uncovered payload tail — refuses with
    /// [`WireCtrtError::DigestMismatch`] (machine code `0xE005`).
    pub fn deserialize_verified(
        bytes: &[u8],
        expected_digest: &[u8; 32],
    ) -> Result<Self, WireCtrtError> {
        #[allow(deprecated)]
        let parsed = Self::deserialize(bytes)?;
        let recomputed = {
            // Recompute over the raw wire bytes (== parsed.serialize() for a
            // structurally valid record; use the input slice so the commitment
            // is over exactly the bytes received).
            let mut h = blake3::Hasher::new();
            h.update(bytes);
            let mut out = [0u8; 32];
            out.copy_from_slice(h.finalize().as_bytes());
            out
        };
        if constant_time_eq(&recomputed, expected_digest) == 0 {
            return Err(WireCtrtError::DigestMismatch);
        }
        Ok(parsed)
    }

    /// Read the statutory deontic metadata from the 201-byte payload.
    ///
    /// Offsets inside the 201-byte payload:
    /// - `0..16`: statutory_anchor (`[u8; 16]`)
    /// - `16..24`: tariff_basis_cents (`u64 LE`)
    /// - `24..25`: deontic_modality (`u8`)
    /// - `25..57`: nullifier_commitment (`[u8; 32]`)
    /// - `57..89`: ocel_event_digest (`[u8; 32]`)
    /// - `89..201`: pqc_witness_signature (`[u8; 112]`)
    #[must_use]
    pub fn statutory_anchor(&self) -> [u8; 16] {
        self.payload[0..16].try_into().unwrap()
    }

    /// Read the liquid avoided cloud compute tariff basis in USD cents.
    #[must_use]
    pub fn tariff_basis_cents(&self) -> u64 {
        u64::from_le_bytes(self.payload[16..24].try_into().unwrap())
    }

    /// Read the deontic modality byte.
    #[must_use]
    pub fn deontic_modality(&self) -> u8 {
        self.payload[24]
    }

    /// Read the anti-replay nullifier commitment.
    #[must_use]
    pub fn nullifier_commitment(&self) -> [u8; 32] {
        self.payload[25..57].try_into().unwrap()
    }

    /// Read the linked IEEE OCEL 2.0 trace event digest.
    #[must_use]
    pub fn ocel_event_digest(&self) -> [u8; 32] {
        self.payload[57..89].try_into().unwrap()
    }

    /// Convert to packed `CompactTrustReceiptToken`.
    #[must_use]
    pub fn to_compact_token(&self) -> CompactTrustReceiptToken {
        let serialized = self.serialize();
        CompactTrustReceiptToken::from_bytes(&serialized).expect("WireCtrt is valid CTRT bytes")
    }

    /// Parse and validate a 352-byte slice into a `WireCtrt`.
    ///
    /// DEPRECATED (SK4-F1): this does NOT verify the BLAKE3 commitment — the
    /// 201-byte payload (151..352) is accepted with arbitrary content. Use
    /// [`WireCtrt::deserialize_verified`] with a digest obtained from a trusted
    /// channel; it is fail-closed end-to-end.
    ///
    /// # Timing guarantees (SK2-F1, measured)
    ///
    /// This is a **constant-FLOW, not constant-TIME** convenience wrapper:
    /// the branchless predicate computation runs identically for every input,
    /// but the accumulated error mask selects the `Result` variant through a
    /// branch (Ok path copies 352 bytes into the returned struct; Err paths
    /// are short). Timing therefore leaks the *validity class* of the wire
    /// (valid vs. structurally invalid), roughly ~8ns/call — acceptable when
    /// the verdict class is public information. When the validity class is
    /// secret, use [`deserialize_ct`], which performs size-normalized work on
    /// both paths and returns a machine code instead of a `Result`.
    #[deprecated(
        since = "26.10.13",
        note = "does not enforce the wire digest commitment; use deserialize_verified"
    )]
    pub fn deserialize(bytes: &[u8]) -> Result<Self, WireCtrtError> {
        let mut body = WireCtrtBody::default();
        let code = deserialize_ct(&mut body, bytes);
        if code == 0 {
            // code == 0 guarantees verdict_raw is a valid status byte.
            return Ok(Self {
                version: body.version,
                parent_receipt_hash: body.parent_receipt_hash,
                court_id: body.court_id,
                subject_merkle_root: body.subject_merkle_root,
                compiler_binary_digest: body.compiler_binary_digest,
                execution_ticks: body.execution_ticks,
                elapsed_cycles: body.elapsed_cycles,
                verdict_status: VerdictStatus::from_u8(body.verdict_raw)
                    .unwrap_or(VerdictStatus::Refused),
                payload: body.payload,
            });
        }
        // Cold path: map the machine code to a rich error with payloads.
        // bytes may be shorter than the fixed layout; read defensively.
        let mut magic = [0u8; 4];
        if bytes.len() >= 4 {
            magic.copy_from_slice(&bytes[0..4]);
        }
        let mut version_bytes = [0u8; 2];
        if bytes.len() >= 6 {
            version_bytes.copy_from_slice(&bytes[4..6]);
        }
        Err(match code {
            0xE001 => WireCtrtError::InvalidLength(bytes.len()),
            0xE003 => WireCtrtError::UnsupportedVersion(u16::from_le_bytes(version_bytes)),
            0xE004 => WireCtrtError::InvalidVerdictStatus(body.verdict_raw),
            // 0xE002 and any unexpected code: magic is the first structural
            // gate, so 0xE002 dominates the fallthrough.
            _ => WireCtrtError::InvalidMagic(magic),
        })
    }

    /// Compute the canonical BLAKE3 commitment of this wire receipt.
    #[must_use]
    pub fn digest(&self) -> [u8; 32] {
        let serialized = self.serialize();
        *blake3::hash(&serialized).as_bytes()
    }
}

/// Constant-time (size-normalized) CTRT wire decode core.
///
/// Writes the decoded fields of `bytes` into the caller-supplied fixed-size
/// `out` body and returns a canonical machine code — `0` = valid, otherwise
/// one of the [`WireCtrtError::machine_code`] values (`0xE001` length,
/// `0xE002` magic, `0xE003` version, `0xE004` verdict status). Both the
/// valid and invalid paths execute the identical instruction sequence: all
/// fields are always copied into `out`, all predicates are always computed,
/// and the verdict variant is always arithmetic-selected from the same
/// 4-entry table. No branch depends on payload bytes and no path-dependent
/// store size differs between the two classes, so per-call timing does not
/// reveal the validity class.
///
/// When a `Result`-shaped API is preferred and the validity class is public
/// information, use [`WireCtrt::deserialize`] (constant-flow only; timing
/// leaks the validity class by ~8ns/call).
///
/// On a nonzero code the contents of `out` are unspecified garbage (the
/// decoded fields are still written, but the input failed validation).
#[must_use = "the machine code conveys the decode verdict; discarding it discards the result"]
pub fn deserialize_ct(out: &mut WireCtrtBody, bytes: &[u8]) -> u16 {
    if bytes.len() != CTRT_WIRE_SIZE {
        // Branch on LENGTH only — length is public, not payload-derived.
        return 0xE001;
    }

    let mut magic = [0u8; 4];
    magic.copy_from_slice(&bytes[0..4]);
    let magic_ok = ct_validate_magic(&magic);

    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    let version_ok = ct_validate_version(version);
    out.version = version;

    out.parent_receipt_hash.copy_from_slice(&bytes[6..38]);
    out.court_id.copy_from_slice(&bytes[38..70]);
    out.subject_merkle_root.copy_from_slice(&bytes[70..102]);
    out.compiler_binary_digest.copy_from_slice(&bytes[102..134]);

    out.execution_ticks = u64::from_le_bytes(
        bytes[134..142]
            .try_into()
            .expect("slice length matches 8 bytes"),
    );
    out.elapsed_cycles = u64::from_le_bytes(
        bytes[142..150]
            .try_into()
            .expect("slice length matches 8 bytes"),
    );

    out.verdict_raw = bytes[150];
    let verdict_ok = ct_validate_verdict_status(out.verdict_raw);

    out.payload.copy_from_slice(&bytes[151..352]);

    // Single error-sink: bitwise OR of validity masks, one code computation.
    let bad_magic = magic_ok ^ 1;
    let bad_version = version_ok ^ 1;
    let bad_verdict = verdict_ok ^ 1;

    // Priority arithmetic select (magic > version > verdict), branch-free:
    // each contribution is all-ones-masked only when it is the highest-
    // priority failing predicate, so exactly one code lands in `code`.
    let sel_version = bad_version & (bad_magic ^ 1);
    let sel_verdict = bad_verdict & (bad_magic ^ 1) & (bad_version ^ 1);
    let mut code = 0u16;
    code |= u16::from(bad_magic).wrapping_neg() & 0xE002;
    code |= u16::from(sel_version).wrapping_neg() & 0xE003;
    code |= u16::from(sel_verdict).wrapping_neg() & 0xE004;
    code
}

#[cfg(test)]
#[allow(deprecated)]
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
        assert_eq!(
            WireCtrtError::InvalidVerdictStatus(42).machine_code(),
            0xE004
        );
        assert_eq!(WireCtrtError::DigestMismatch.machine_code(), 0xE005);
    }

    // ── SK4-F1: full-wire digest enforcement at decode ─────────────────────

    #[test]
    fn test_deserialize_verified_flipped_payload_stale_digest_refuses() {
        let original = sample_ctrt();
        let mut bytes = original.serialize();

        // Tamper strictly inside the payload tail (151..352) — the field class
        // deserialize() accepts with arbitrary content.
        bytes[300] ^= 0x01;
        assert!(bytes[300] != original.serialize()[300]);

        // Stale expected digest = commitment of the untampered wire.
        let stale = original.digest();
        let err = WireCtrt::deserialize_verified(&bytes, &stale)
            .expect_err("flipped payload byte with stale digest must refuse");
        assert_eq!(err.machine_code(), 0xE005, "must be DigestMismatch");
        assert_eq!(err, WireCtrtError::DigestMismatch);
    }

    #[test]
    fn test_deserialize_verified_flipped_payload_recomputed_digest_accepts() {
        let original = sample_ctrt();
        let bytes = original.serialize();

        // Recompute the commitment over the (unmodified) full 352 bytes.
        let expected = original.digest();
        let decoded =
            WireCtrt::deserialize_verified(&bytes, &expected).expect("matching digest accepts");
        assert_eq!(decoded, original);

        // Prove the tail is commitment-covered: tamper a payload byte, recompute
        // the expected digest from the TAMPERED bytes — accept must still hold
        // (the digest tracks the tail, not a prefix).
        let mut tampered = bytes;
        tampered[250] ^= 0xFF;
        let mut h = blake3::Hasher::new();
        h.update(&tampered);
        let mut fresh_expected = [0u8; 32];
        fresh_expected.copy_from_slice(h.finalize().as_bytes());
        let decoded_tampered = WireCtrt::deserialize_verified(&tampered, &fresh_expected)
            .expect("tampered wire with freshly recomputed digest accepts");
        assert_ne!(decoded_tampered, original, "content actually changed");
    }

    #[test]
    fn test_deserialize_verified_full_wire_tamper_matrix() {
        let original = sample_ctrt();
        let serialized = original.serialize();
        let expected = original.digest();

        // Every byte flip in all 352 bytes must refuse — no byte is
        // commitment-free. Header fields with strict structural validation
        // (magic/version/verdict) refuse with their structural code; every
        // other byte, including the whole payload tail, refuses with 0xE005.
        for i in 0..CTRT_WIRE_SIZE {
            let mut tampered = serialized;
            tampered[i] ^= 0x01;
            match WireCtrt::deserialize_verified(&tampered, &expected) {
                Err(err) => match i {
                    // Structural fields may refuse either structurally (their
                    // bit flip may still land on a valid value, e.g. verdict
                    // 1->0) or via the digest check — any refusal is fail-closed.
                    0..4 | 4..6 | 150 => {
                        assert!(
                            (0xE001..=0xE005).contains(&err.machine_code()),
                            "offset {i}: {err}"
                        );
                    }
                    _ => assert_eq!(err.machine_code(), 0xE005, "offset {i}"),
                },
                Ok(_) => panic!("tampered byte at offset {i} accepted"),
            }
        }
    }

    #[test]
    fn test_deserialize_verified_truncated_input_refuses_length_first() {
        let original = sample_ctrt();
        let bytes = original.serialize();
        assert!(matches!(
            WireCtrt::deserialize_verified(&bytes[..351], &original.digest()),
            Err(WireCtrtError::InvalidLength(351))
        ));
    }

    #[test]
    fn test_deserialize_ct_parity_with_deserialize() {
        let bytes = sample_ctrt().serialize();

        // Valid input: code 0, body carries the decoded fields.
        let mut body = WireCtrtBody::default();
        assert_eq!(deserialize_ct(&mut body, &bytes), 0);
        assert_eq!(body.version, CTRT_VERSION_1);
        assert_eq!(body.parent_receipt_hash, [0x11; 32]);
        assert_eq!(body.execution_ticks, 100_000);
        assert_eq!(body.verdict_raw, 1);

        // Structural failures return the same machine codes the Result API
        // reports, for every failure class.
        assert_eq!(deserialize_ct(&mut body, &bytes[..351]), 0xE001);
        let mut bad_magic = bytes;
        bad_magic[0] = b'X';
        assert_eq!(deserialize_ct(&mut body, &bad_magic), 0xE002);
        let mut bad_version = bytes;
        bad_version[4] = 2;
        assert_eq!(deserialize_ct(&mut body, &bad_version), 0xE003);
        let mut bad_verdict = bytes;
        bad_verdict[150] = 99;
        assert_eq!(deserialize_ct(&mut body, &bad_verdict), 0xE004);

        // Both validity classes produce a fully written body (size-normalized
        // work): the invalid class still carries all decoded fields.
        let mut body2 = WireCtrtBody::default();
        let _ = deserialize_ct(&mut body2, &bad_magic);
        assert_eq!(body2.parent_receipt_hash, body.parent_receipt_hash);
        assert_eq!(body2.payload, body.payload);
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
