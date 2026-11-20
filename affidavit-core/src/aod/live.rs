//! Live Gate — ontology match, fixed-stride layout, lease burn
//! (AFFI-SPEC-26.11.19 §4.3 Mode A).

use super::envelope::shared_envelope_check;
use super::layout::{align64, page_digest_lo, BufferEntry, AOD_DIR_ENTRY_LEN, AOD_HEADER_LEN};
use super::refuse::AodRefusal;

/// Compiled ontology binding: identity hash + per-slot fixed byte strides.
///
/// Closed-domain symbols live in the application binary (ggen); the on-disk
/// artifact only carries this hash and positional buffers.
#[derive(Clone, Copy, Debug)]
pub struct CompiledOntology<'a> {
    /// Expected `header.ontology_hash`.
    pub ontology_hash: &'a [u8; 32],
    /// Fixed byte length **per row** for each positional buffer slot.
    pub strides: &'a [u64],
}

impl<'a> CompiledOntology<'a> {
    /// Expected absolute byte length for slot `i` given `row_count`.
    #[inline]
    pub fn expected_byte_length(&self, slot: usize, row_count: u64) -> Option<u64> {
        self.strides.get(slot).map(|s| s.saturating_mul(row_count))
    }
}

/// Monoatomic lease registry — burn-on-admit.
///
/// The root crate plugs a persistent nullifier store; tests use
/// [`InMemoryLeaseRegistry`].
pub trait LeaseRegistry {
    /// Refuse if `nullifier` was already burned or is outside the epoch window.
    fn burn(&mut self, lease_epoch: u64, nullifier: &[u8; 32]) -> Result<(), AodRefusal>;
}

/// Zero-copy live view over Region 3 buffers.
#[derive(Clone, Copy, Debug)]
pub struct AodView<'a> {
    /// Ontology hash that admitted this view.
    pub ontology_hash: &'a [u8; 32],
    /// Row count.
    pub row_count: u64,
    /// Contiguous raw file (for advanced consumers).
    pub raw: &'a [u8],
    /// Directory entries slice.
    pub entries: &'a [u8],
    /// Lease epoch that was burned.
    pub lease_epoch: u64,
    /// Nullifier that was burned.
    pub lease_nullifier: &'a [u8; 32],
}

impl<'a> AodView<'a> {
    /// Number of buffers.
    pub fn buffer_count(&self) -> usize {
        self.entries.len() / AOD_DIR_ENTRY_LEN
    }

    /// Borrow buffer slot `i` as a byte slice (Region 3).
    pub fn buffer(&self, i: usize) -> Result<&'a [u8], AodRefusal> {
        let start = i
            .checked_mul(AOD_DIR_ENTRY_LEN)
            .ok_or(AodRefusal::BoundsOrSlack)?;
        let end = start
            .checked_add(AOD_DIR_ENTRY_LEN)
            .ok_or(AodRefusal::BoundsOrSlack)?;
        if end > self.entries.len() {
            return Err(AodRefusal::BoundsOrSlack);
        }
        let entry = BufferEntry::decode(&self.entries[start..end])?;
        let off = entry.byte_offset as usize;
        let len = entry.byte_length as usize;
        let end = off.checked_add(len).ok_or(AodRefusal::BoundsOrSlack)?;
        if end > self.raw.len() {
            return Err(AodRefusal::BoundsOrSlack);
        }
        Ok(&self.raw[off..end])
    }
}

/// Live gate: envelope + ontology + contiguous fixed-stride layout + lease burn.
pub fn map_live<'a, R: LeaseRegistry>(
    raw: &'a [u8],
    ontology: &CompiledOntology<'_>,
    leases: &mut R,
) -> Result<AodView<'a>, AodRefusal> {
    let ok = shared_envelope_check(raw)?;

    if &ok.header.ontology_hash != ontology.ontology_hash {
        return Err(AodRefusal::UnknownOntology);
    }
    if ok.header.buffer_count as usize != ontology.strides.len() {
        return Err(AodRefusal::BufferCountMismatch);
    }

    let trailer = ok.header.trailer_offset as usize;
    let mut cursor = if ok.header.buffer_count == 0 {
        AOD_HEADER_LEN
    } else {
        let dir_end = AOD_HEADER_LEN + ok.header.buffer_count as usize * AOD_DIR_ENTRY_LEN;
        align64(dir_end)
    };

    for i in 0..ok.header.buffer_count as usize {
        let start = i * AOD_DIR_ENTRY_LEN;
        let entry = BufferEntry::decode(&ok.entries[start..start + AOD_DIR_ENTRY_LEN])?;
        if entry.byte_offset as usize != cursor {
            return Err(AodRefusal::BoundsOrSlack);
        }
        let expected = ontology
            .expected_byte_length(i, ok.header.row_count)
            .ok_or(AodRefusal::BufferCountMismatch)?;
        if entry.byte_length != expected {
            return Err(AodRefusal::FixedStrideViolation);
        }
        let off = entry.byte_offset as usize;
        let len = entry.byte_length as usize;
        let end = off.checked_add(len).ok_or(AodRefusal::BoundsOrSlack)?;
        if end > trailer {
            return Err(AodRefusal::BoundsOrSlack);
        }
        let page = &raw[off..end];
        if page_digest_lo(page) != entry.page_digest_lo {
            return Err(AodRefusal::DigestMismatch);
        }
        // Page must be followed by zero pad to 64B (except last may abut trailer).
        let padded_end = align64(end);
        if padded_end > trailer {
            return Err(AodRefusal::BoundsOrSlack);
        }
        if raw[end..padded_end].iter().any(|&b| b != 0) {
            return Err(AodRefusal::NonZeroPadding);
        }
        cursor = padded_end;
    }
    if cursor != trailer {
        return Err(AodRefusal::BoundsOrSlack);
    }

    // Final step: burn monoatomic lease.
    leases.burn(ok.shadow.lease_epoch, &ok.shadow.lease_nullifier)?;

    Ok(AodView {
        ontology_hash: {
            // Re-borrow from raw header bytes (stable for 'a).
            let h = &raw[24..56];
            // SAFETY-free: transmute via try_into on a reborrowed array reference.
            // We can't return &ok.header.ontology_hash (local). Point into raw.
            <&[u8; 32]>::try_from(h).map_err(|_| AodRefusal::BoundsOrSlack)?
        },
        row_count: ok.header.row_count,
        raw,
        entries: ok.entries,
        lease_epoch: ok.shadow.lease_epoch,
        lease_nullifier: {
            let off = trailer + 40;
            <&[u8; 32]>::try_from(&raw[off..off + 32]).map_err(|_| AodRefusal::BoundsOrSlack)?
        },
    })
}

/// In-memory lease registry for courts / tests (`alloc`).
#[cfg(feature = "alloc")]
pub mod mem {
    use super::*;
    use alloc::collections::BTreeSet;

    /// Burns nullifiers into a sorted set (deterministic).
    #[derive(Clone, Debug, Default)]
    pub struct InMemoryLeaseRegistry {
        burned: BTreeSet<[u8; 32]>,
    }

    impl InMemoryLeaseRegistry {
        /// Empty registry.
        pub fn new() -> Self {
            Self {
                burned: BTreeSet::new(),
            }
        }
    }

    impl LeaseRegistry for InMemoryLeaseRegistry {
        fn burn(&mut self, _lease_epoch: u64, nullifier: &[u8; 32]) -> Result<(), AodRefusal> {
            if !self.burned.insert(*nullifier) {
                return Err(AodRefusal::LeaseSpentOrExpired);
            }
            Ok(())
        }
    }
}

#[cfg(feature = "alloc")]
pub use mem::InMemoryLeaseRegistry;
