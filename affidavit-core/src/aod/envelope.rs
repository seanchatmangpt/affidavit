//! Shared envelope check — O(1) framing + BLAKE3 + seal commitment
//! (AFFI-SPEC-26.11.19 §4.3). Zero heap allocation.

use super::layout::{
    blake3_of, verify_seal_commitment, AodHeader, AodShadow, BufferEntry, AOD_DIR_ENTRY_LEN,
    AOD_HEADER_LEN, AOD_MIN_FILE_LEN, AOD_SHADOW_LEN,
};
use super::refuse::AodRefusal;

/// Borrowed view of a file that passed the shared envelope check.
#[derive(Clone, Copy, Debug)]
pub struct EnvelopeOk<'a> {
    /// Decoded header.
    pub header: AodHeader,
    /// Decoded shadow.
    pub shadow: AodShadow,
    /// Full file bytes (Regions 1–4).
    pub raw: &'a [u8],
    /// Directory entries (may be empty).
    pub entries: &'a [u8],
}

/// Shared envelope check (live and witness).
///
/// Checks alignment demand (length ≥ 416), magic, reserved, trailer offset,
/// zero-padding of the directory pad region, BLAKE3(payload) == shadow digest,
/// and the domain-separated seal commitment.
pub fn shared_envelope_check(raw: &[u8]) -> Result<EnvelopeOk<'_>, AodRefusal> {
    if raw.len() < AOD_MIN_FILE_LEN {
        return Err(AodRefusal::TruncatedFile);
    }
    let header = AodHeader::decode(raw)?;
    let file_len = raw.len() as u64;
    if header.trailer_offset != file_len - AOD_SHADOW_LEN as u64 {
        return Err(AodRefusal::InvalidTrailerOffset);
    }
    if header.trailer_offset < AOD_HEADER_LEN as u64 {
        return Err(AodRefusal::InvalidTrailerOffset);
    }
    if header.flags != 0 {
        // v1: unknown flags refuse closed (reuse reserved band semantics).
        return Err(AodRefusal::NonZeroReserved);
    }

    let dir_bytes = (header.buffer_count as usize).saturating_mul(AOD_DIR_ENTRY_LEN);
    let dir_end = AOD_HEADER_LEN + dir_bytes;
    // Directory region padded to 64B when non-empty; empty ⇒ no directory.
    let data_start = if header.buffer_count == 0 {
        AOD_HEADER_LEN
    } else {
        let padded = super::layout::align64(dir_end);
        // Pad bytes must be zero.
        if padded > header.trailer_offset as usize {
            return Err(AodRefusal::BoundsOrSlack);
        }
        if raw[dir_end..padded].iter().any(|&b| b != 0) {
            return Err(AodRefusal::NonZeroPadding);
        }
        padded
    };

    if data_start > header.trailer_offset as usize {
        return Err(AodRefusal::BoundsOrSlack);
    }

    let shadow_off = header.trailer_offset as usize;
    let shadow = AodShadow::decode(&raw[shadow_off..])?;

    let payload = &raw[..shadow_off];
    let digest = blake3_of(payload);
    if digest.0 != shadow.payload_digest {
        return Err(AodRefusal::DigestMismatch);
    }
    verify_seal_commitment(&shadow)?;

    let entries = if header.buffer_count == 0 {
        &raw[AOD_HEADER_LEN..AOD_HEADER_LEN]
    } else {
        &raw[AOD_HEADER_LEN..dir_end]
    };
    // Validate each directory entry is well-formed (offset % 64).
    for i in 0..header.buffer_count as usize {
        let start = i * AOD_DIR_ENTRY_LEN;
        let _ = BufferEntry::decode(&entries[start..start + AOD_DIR_ENTRY_LEN])?;
    }

    Ok(EnvelopeOk {
        header,
        shadow,
        raw,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aod::layout::{compute_seal_commitment, AOD_MAGIC};

    fn minimal_unsigned_empty() -> [u8; AOD_MIN_FILE_LEN] {
        let mut raw = [0u8; AOD_MIN_FILE_LEN];
        let header = AodHeader {
            flags: 0,
            row_count: 0,
            buffer_count: 0,
            ontology_hash: [0x11; 32],
            trailer_offset: AOD_HEADER_LEN as u64,
        };
        raw[..AOD_HEADER_LEN].copy_from_slice(&header.encode());
        let payload_digest = blake3_of(&raw[..AOD_HEADER_LEN]);
        let mut shadow = AodShadow {
            payload_digest: payload_digest.0,
            lease_epoch: 1,
            lease_nullifier: [0x22; 32],
            folded_receipt: [0x33; 32],
            pqc_seal: [0u8; super::super::layout::AOD_PQC_SEAL_LEN],
        };
        shadow.pqc_seal = compute_seal_commitment(
            &shadow.payload_digest,
            shadow.lease_epoch,
            &shadow.lease_nullifier,
            &shadow.folded_receipt,
        );
        raw[AOD_HEADER_LEN..].copy_from_slice(&shadow.encode());
        assert_eq!(&raw[0..4], &AOD_MAGIC);
        raw
    }

    #[test]
    fn empty_file_envelope_accepts() {
        let raw = minimal_unsigned_empty();
        let ok = shared_envelope_check(&raw).expect("envelope");
        assert_eq!(ok.header.buffer_count, 0);
        assert_eq!(ok.shadow.lease_epoch, 1);
    }

    #[test]
    fn truncated_refuses_e502() {
        let err = shared_envelope_check(&[0u8; 100]).unwrap_err();
        assert_eq!(err, AodRefusal::TruncatedFile);
        assert_eq!(err.machine_code(), 0xE502);
    }

    #[test]
    fn bad_magic_refuses_e503() {
        let mut raw = minimal_unsigned_empty();
        raw[0] = 0x00;
        let err = shared_envelope_check(&raw).unwrap_err();
        assert_eq!(err.machine_code(), 0xE503);
    }

    #[test]
    fn digest_tamper_refuses_e50b() {
        let mut raw = minimal_unsigned_empty();
        raw[24] ^= 0xFF; // flip ontology_hash byte inside payload
        let err = shared_envelope_check(&raw).unwrap_err();
        assert_eq!(err.machine_code(), 0xE50B);
    }
}
