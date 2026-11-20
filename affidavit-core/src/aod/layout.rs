//! Physical `.aod` layout: 64B header, 32B directory entries, 64B-aligned
//! pages, 352B CTRT shadow (AFFI-SPEC-26.11.19 §4.2).

use crate::digest::{Blake3Hasher, ChainHasher, Digest};

use super::refuse::AodRefusal;

/// Non-ASCII sentinel `0x9F 0x1A 0xC4 0x01` — never a printable magic.
pub const AOD_MAGIC: [u8; 4] = [0x9F, 0x1A, 0xC4, 0x01];

/// Fixed header length (bytes).
pub const AOD_HEADER_LEN: usize = 64;

/// Directory entry length (bytes).
pub const AOD_DIR_ENTRY_LEN: usize = 32;

/// CTRT hyperdimensional shadow length (bytes).
pub const AOD_SHADOW_LEN: usize = 352;

/// Opaque PQC seal slot inside the shadow (bytes).
pub const AOD_PQC_SEAL_LEN: usize = 248;

/// Spec minimum file size: 64B header + 352B trailer (zero buffers).
pub const AOD_MIN_FILE_LEN: usize = AOD_HEADER_LEN + AOD_SHADOW_LEN; // 416

/// Domain tag for the 248-byte PQC seal commitment (not a UTF-8 on-disk label;
/// compile-time only).
pub const DOMAIN_AOD_SEAL: &[u8] = b"affidavit-core/aod/v1\0seal";

/// Round `n` up to the next multiple of 64.
#[inline]
pub const fn align64(n: usize) -> usize {
    (n + 63) & !63
}

/// Bytes of zero padding needed to reach the next 64-byte boundary.
#[inline]
pub const fn pad64(n: usize) -> usize {
    align64(n) - n
}

/// Fixed 64-byte AOD header (little-endian fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AodHeader {
    /// Format flags (v1: must be 0).
    pub flags: u32,
    /// Number of logical rows in every fixed-stride buffer.
    pub row_count: u64,
    /// Number of positional buffers in the directory.
    pub buffer_count: u32,
    /// Compiled ontology identity the writer sealed against.
    pub ontology_hash: [u8; 32],
    /// Byte offset of the 352B shadow (`file_len - 352`).
    pub trailer_offset: u64,
}

impl AodHeader {
    /// Encode into a 64-byte LE buffer.
    pub fn encode(&self) -> [u8; AOD_HEADER_LEN] {
        let mut out = [0u8; AOD_HEADER_LEN];
        out[0..4].copy_from_slice(&AOD_MAGIC);
        out[4..8].copy_from_slice(&self.flags.to_le_bytes());
        out[8..16].copy_from_slice(&self.row_count.to_le_bytes());
        out[16..20].copy_from_slice(&self.buffer_count.to_le_bytes());
        // 20..24 reserved == 0
        out[24..56].copy_from_slice(&self.ontology_hash);
        out[56..64].copy_from_slice(&self.trailer_offset.to_le_bytes());
        out
    }

    /// Decode a 64-byte header; refuses bad sentinel / non-zero reserved.
    pub fn decode(bytes: &[u8]) -> Result<Self, AodRefusal> {
        if bytes.len() < AOD_HEADER_LEN {
            return Err(AodRefusal::TruncatedFile);
        }
        if bytes[0..4] != AOD_MAGIC {
            return Err(AodRefusal::InvalidSentinel);
        }
        if bytes[20..24] != [0, 0, 0, 0] {
            return Err(AodRefusal::NonZeroReserved);
        }
        let flags = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        let row_count = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let buffer_count = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
        let mut ontology_hash = [0u8; 32];
        ontology_hash.copy_from_slice(&bytes[24..56]);
        let trailer_offset = u64::from_le_bytes(bytes[56..64].try_into().unwrap());
        Ok(Self {
            flags,
            row_count,
            buffer_count,
            ontology_hash,
            trailer_offset,
        })
    }
}

/// One positional buffer directory entry (32 bytes, no names/types).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferEntry {
    /// Absolute byte offset of the buffer start (`% 64 == 0`).
    pub byte_offset: u64,
    /// Exact byte length (fixed-stride × row_count; no trailing slack inside).
    pub byte_length: u64,
    /// Low 16 bytes of BLAKE3(page) for page-level temper detection.
    pub page_digest_lo: [u8; 16],
}

impl BufferEntry {
    /// Encode LE.
    pub fn encode(&self) -> [u8; AOD_DIR_ENTRY_LEN] {
        let mut out = [0u8; AOD_DIR_ENTRY_LEN];
        out[0..8].copy_from_slice(&self.byte_offset.to_le_bytes());
        out[8..16].copy_from_slice(&self.byte_length.to_le_bytes());
        out[16..32].copy_from_slice(&self.page_digest_lo);
        out
    }

    /// Decode LE.
    pub fn decode(bytes: &[u8]) -> Result<Self, AodRefusal> {
        if bytes.len() < AOD_DIR_ENTRY_LEN {
            return Err(AodRefusal::TruncatedFile);
        }
        let byte_offset = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let byte_length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let mut page_digest_lo = [0u8; 16];
        page_digest_lo.copy_from_slice(&bytes[16..32]);
        if !byte_offset.is_multiple_of(64) {
            return Err(AodRefusal::BoundsOrSlack);
        }
        Ok(Self {
            byte_offset,
            byte_length,
            page_digest_lo,
        })
    }
}

/// 352-byte CTRT hyperdimensional shadow (EOF trailer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AodShadow {
    /// BLAKE3 over Regions 1–3 (bytes `[0, trailer_offset)`).
    pub payload_digest: [u8; 32],
    /// Monoatomic lease epoch.
    pub lease_epoch: u64,
    /// Anti-replay nullifier burned by the live gate.
    pub lease_nullifier: [u8; 32],
    /// Folded receipt tip (`R_hist ⊕ R_rot` or genesis identity).
    pub folded_receipt: [u8; 32],
    /// Domain-separated seal commitment (248B); control plane may embed
    /// ML-DSA-65 material. Core verifies the BLAKE3 commitment shape.
    pub pqc_seal: [u8; AOD_PQC_SEAL_LEN],
}

impl AodShadow {
    /// Encode LE into 352 bytes.
    pub fn encode(&self) -> [u8; AOD_SHADOW_LEN] {
        let mut out = [0u8; AOD_SHADOW_LEN];
        out[0..32].copy_from_slice(&self.payload_digest);
        out[32..40].copy_from_slice(&self.lease_epoch.to_le_bytes());
        out[40..72].copy_from_slice(&self.lease_nullifier);
        out[72..104].copy_from_slice(&self.folded_receipt);
        out[104..352].copy_from_slice(&self.pqc_seal);
        out
    }

    /// Decode the trailing 352 bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, AodRefusal> {
        if bytes.len() < AOD_SHADOW_LEN {
            return Err(AodRefusal::TruncatedFile);
        }
        let mut payload_digest = [0u8; 32];
        payload_digest.copy_from_slice(&bytes[0..32]);
        let lease_epoch = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let mut lease_nullifier = [0u8; 32];
        lease_nullifier.copy_from_slice(&bytes[40..72]);
        let mut folded_receipt = [0u8; 32];
        folded_receipt.copy_from_slice(&bytes[72..104]);
        let mut pqc_seal = [0u8; AOD_PQC_SEAL_LEN];
        pqc_seal.copy_from_slice(&bytes[104..352]);
        Ok(Self {
            payload_digest,
            lease_epoch,
            lease_nullifier,
            folded_receipt,
            pqc_seal,
        })
    }
}

/// BLAKE3 over `bytes` → [`Digest`].
pub(crate) fn blake3_of(bytes: &[u8]) -> Digest {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, bytes);
    Blake3Hasher::finish(st)
}

/// Low 16 bytes of BLAKE3(page).
pub(crate) fn page_digest_lo(page: &[u8]) -> [u8; 16] {
    let d = blake3_of(page);
    let mut lo = [0u8; 16];
    lo.copy_from_slice(&d.0[..16]);
    lo
}

/// Domain-separated 248-byte seal commitment over shadow identity fields.
///
/// Fills `[0..32)` with BLAKE3(domain ‖ payload ‖ epoch ‖ nullifier ‖ folded)
/// and leaves the remainder zero. Control-plane ML-DSA material may overwrite
/// `[32..)` later; core envelope verification always checks `[0..32)`.
pub fn compute_seal_commitment(
    payload_digest: &[u8; 32],
    lease_epoch: u64,
    lease_nullifier: &[u8; 32],
    folded_receipt: &[u8; 32],
) -> [u8; AOD_PQC_SEAL_LEN] {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, DOMAIN_AOD_SEAL);
    Blake3Hasher::absorb(&mut st, payload_digest);
    Blake3Hasher::absorb(&mut st, &lease_epoch.to_le_bytes());
    Blake3Hasher::absorb(&mut st, lease_nullifier);
    Blake3Hasher::absorb(&mut st, folded_receipt);
    let head = Blake3Hasher::finish(st);
    let mut seal = [0u8; AOD_PQC_SEAL_LEN];
    seal[..32].copy_from_slice(&head.0);
    seal
}

/// Verify the full 248-byte seal commitment matches recomputation.
pub fn verify_seal_commitment(shadow: &AodShadow) -> Result<(), AodRefusal> {
    let expected = compute_seal_commitment(
        &shadow.payload_digest,
        shadow.lease_epoch,
        &shadow.lease_nullifier,
        &shadow.folded_receipt,
    );
    if shadow.pqc_seal != expected {
        return Err(AodRefusal::PqcSealFailure);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_round_trip() {
        let h = AodHeader {
            flags: 0,
            row_count: 7,
            buffer_count: 2,
            ontology_hash: [0xAB; 32],
            trailer_offset: 1024,
        };
        let enc = h.encode();
        assert_eq!(&enc[0..4], &AOD_MAGIC);
        assert_eq!(AodHeader::decode(&enc).unwrap(), h);
    }

    #[test]
    fn magic_is_not_printable_ascii_token() {
        // Spec bytes: 0x9F 0x1A 0xC4 0x01 — not a printable ASCII fourcc.
        assert_eq!(AOD_MAGIC, [0x9F, 0x1A, 0xC4, 0x01]);
        assert!(AOD_MAGIC.iter().any(|&b| b > 0x7F));
        assert!(AOD_MAGIC.iter().any(|&b| !b.is_ascii_graphic()));
        assert_ne!(&AOD_MAGIC, b"CTRT");
        assert_ne!(&AOD_MAGIC, b"AOD\0");
    }

    #[test]
    fn shadow_len_is_352() {
        assert_eq!(AOD_SHADOW_LEN, 352);
        assert_eq!(32 + 8 + 32 + 32 + AOD_PQC_SEAL_LEN, 352);
    }
}
