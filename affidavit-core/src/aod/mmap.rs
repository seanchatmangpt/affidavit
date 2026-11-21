//! Zero-copy Live Gate borrow contract (AFFI-SPEC-26.11.19 PR-4).
//!
//! Real OS `mmap` lives in the host `affidavit` crate (`src/aod_mmap.rs`).
//! This module stays **`no_std`**: it only documents and checks the pointer
//! alignment that Region 3 pages already require on disk.
//!
//! # Design split
//!
//! | Layer | Responsibility |
//! |-------|----------------|
//! | `affidavit-core::map_live(&[u8])` | Ontology / stride / lease; **zero heap file copy** |
//! | `affidavit::aod_mmap::MappedAod` | Host `mmap` → `&[u8]` for `map_live` |
//!
//! # Alignment
//!
//! AOD pages and directory padding are **64-byte** aligned relative to file
//! offset 0 ([`crate::aod::align64`]). An OS `mmap` base is typically
//! host-page aligned (4 KiB+). Therefore buffer pointers formed as
//! `mmap_base + offset` inherit ≥64 B alignment whenever `offset % 64 == 0`.
//!
//! When a host **demands** alignment before admit, call
//! [`demand_aod_aligned`] — that is the sole emitter of
//! [`AodRefusal::UnalignedInput`] (`0xE501`).

use super::refuse::AodRefusal;

/// Byte alignment of AOD Region 3 pages and directory padding.
pub const AOD_MMAP_ALIGN: usize = 64;

/// True when `addr` has at least [`AOD_MMAP_ALIGN`] alignment.
#[inline]
pub const fn is_aod_aligned(addr: usize) -> bool {
    addr.is_multiple_of(AOD_MMAP_ALIGN)
}

/// True when the first byte of `bytes` is [`AOD_MMAP_ALIGN`]-aligned.
///
/// Empty slices are treated as aligned (no pointer to check).
#[inline]
pub fn slice_is_aod_aligned(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return true;
    }
    is_aod_aligned(bytes.as_ptr() as usize)
}

/// Refuse [`AodRefusal::UnalignedInput`] (`0xE501`) when `bytes` is non-empty
/// and its base pointer is not [`AOD_MMAP_ALIGN`]-aligned.
///
/// Soft diagnostics use [`slice_is_aod_aligned`]; this helper is the hard gate
/// for hosts that demand alignment before Live Gate / Avec admit.
#[inline]
pub fn demand_aod_aligned(bytes: &[u8]) -> Result<(), AodRefusal> {
    if slice_is_aod_aligned(bytes) {
        Ok(())
    } else {
        Err(AodRefusal::UnalignedInput)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn align_constant_matches_layout_pad() {
        assert_eq!(AOD_MMAP_ALIGN, 64);
        assert!(is_aod_aligned(0));
        assert!(is_aod_aligned(64));
        assert!(!is_aod_aligned(1));
        assert!(!is_aod_aligned(63));
    }

    #[test]
    fn stack_array_may_or_may_not_be_aligned() {
        // Just exercise the helper; alignment of a local `[u8; N]` is
        // unspecified beyond 1. Empty slice is always OK.
        assert!(slice_is_aod_aligned(&[]));
        assert!(demand_aod_aligned(&[]).is_ok());
    }

    #[test]
    fn demand_refuses_deliberately_misaligned_slice() {
        let storage = [0u8; 128];
        let base = storage.as_ptr() as usize;
        // Pick a start offset so the subslice base is not 64-byte aligned.
        let start = if base.is_multiple_of(AOD_MMAP_ALIGN) {
            1
        } else {
            0
        };
        let mis = &storage[start..start + 16];
        assert!(!slice_is_aod_aligned(mis));
        let err = demand_aod_aligned(mis).unwrap_err();
        assert_eq!(err, AodRefusal::UnalignedInput);
        assert_eq!(err.machine_code(), 0xE501);
    }
}
