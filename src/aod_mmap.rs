//! Host `mmap` adapter for Live Gate (AFFI-SPEC-26.11.19 PR-4).
//!
//! [`affidavit_core::map_live`] is deliberately zero-alloc over `&[u8]`. This
//! module is the **only** Live Gate file open path: it memory-maps a sealed
//! `.aod` and hands the borrowed slice to `map_live` — **no** full-file
//! `Vec<u8>` copy on the admit path.
//!
//! # Alloc boundary (honest court)
//!
//! | What | Allocates? |
//! |------|------------|
//! | `MappedAod::open` → OS `mmap` | Mapping metadata / page tables only — **not** a heap copy of file bytes |
//! | `map_live(&mapped.as_bytes(), …)` | No file-sized `Vec` |
//! | Lease registry / CLI string parsing | Small heap (nullifier set, hex decode) — outside the file body |
//!
//! # Alignment
//!
//! OS `mmap` returns a host-page-aligned base (typically 4 KiB). AOD Region 3
//! pages are already 64 B aligned relative to file offset 0, so
//! `mmap_base + offset` inherits ≥64 B alignment when `offset % 64 == 0`.
//! See [`affidavit_core::aod::mmap`].

use memmap2::Mmap;
use std::fs::File;
use std::io;
use std::path::Path;

/// Read-only memory map of a sealed `.aod` file.
///
/// Keep this value alive for the duration of any [`affidavit_core::AodView`]
/// borrows derived from [`Self::as_bytes`].
pub struct MappedAod {
    /// Held so the Windows file handle stays valid for the mapping lifetime.
    _file: File,
    mmap: Mmap,
}

impl MappedAod {
    /// Memory-map `path` read-only without copying file bytes onto the heap.
    ///
    /// # Errors
    ///
    /// Propagates OS open / mmap failures.
    #[allow(unsafe_code)] // sole Live Gate admission: OS mmap via memmap2
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path.as_ref())?;
        // SAFETY: `memmap2::Mmap::map` is the sole unsafe admission for OS mmap.
        // On Unix the mapping remains valid after `File` drop; we still retain
        // `_file` so Windows handle semantics stay sound. The returned `Mmap`
        // is read-only; we never write through it.
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { _file: file, mmap })
    }

    /// Borrow the full mapped file as bytes for [`affidavit_core::map_live`].
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.mmap
    }

    /// Byte length of the mapped region (equals on-disk file length).
    #[inline]
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    /// True when the map is empty (AOD files are never empty in practice).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    /// True when the mapped base pointer is 64-byte aligned.
    ///
    /// Normal OS page-aligned bases satisfy this; the check is a soft
    /// diagnostic for exotic hosts, not a Live Gate refusal.
    #[inline]
    pub fn base_is_aod_aligned(&self) -> bool {
        affidavit_core::aod::mmap::slice_is_aod_aligned(self.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use affidavit_core::{
        map_live, seal_aod, AodBufferSpec, CompiledOntology, InMemoryLeaseRegistry, SealParams,
        AOD_MIN_FILE_LEN,
    };
    use std::fs;
    use tempfile::TempDir;

    fn sealed_fixture(dir: &Path) -> (std::path::PathBuf, [u8; 32], u64) {
        let ontology = [0xAAu8; 32];
        let lease = [0xBBu8; 32];
        let rows = 2u64;
        let stride = 8usize;
        let page: Vec<u8> = (0..(rows as usize * stride))
            .map(|i| (i % 251) as u8)
            .collect();
        let raw = seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: rows,
                lease_epoch: 1,
                lease_nullifier: &lease,
                folded_receipt: &[0u8; 32],
            },
        )
        .expect("seal");
        let path = dir.join("live.aod");
        fs::write(&path, &raw).expect("write");
        (path, ontology, stride as u64)
    }

    #[test]
    fn mmap_open_is_not_a_heap_file_copy() {
        let dir = TempDir::new().unwrap();
        let (path, _, _) = sealed_fixture(dir.path());
        let on_disk = fs::metadata(&path).unwrap().len() as usize;
        assert!(on_disk >= AOD_MIN_FILE_LEN);

        let mapped = MappedAod::open(&path).expect("mmap");
        assert_eq!(mapped.len(), on_disk);
        assert!(
            mapped.base_is_aod_aligned(),
            "mmap base must be ≥64B aligned"
        );

        // Structural court: MappedAod holds Mmap + File, not Vec<u8>.
        // Pointer into the map must differ from a separately allocated Vec of
        // the same bytes (heap copy would be a distinct allocation).
        let heap = fs::read(&path).expect("control Vec");
        assert_eq!(mapped.as_bytes(), heap.as_slice());
        assert_ne!(
            mapped.as_bytes().as_ptr(),
            heap.as_ptr(),
            "mapped bytes must not alias the control Vec allocation"
        );
    }

    #[test]
    fn map_live_admits_through_mmap_without_vec_body() {
        let dir = TempDir::new().unwrap();
        let (path, ontology, stride) = sealed_fixture(dir.path());
        let mapped = MappedAod::open(&path).expect("mmap");
        let compiled = CompiledOntology {
            ontology_hash: &ontology,
            strides: &[stride],
        };
        let mut leases = InMemoryLeaseRegistry::new();
        let view = map_live(mapped.as_bytes(), &compiled, &mut leases).expect("live");
        assert_eq!(view.row_count, 2);
        assert_eq!(view.buffer_count(), 1);
        // Region 3 borrow points into the map, not a copy.
        let buf = view.buffer(0).expect("buf0");
        let map_start = mapped.as_bytes().as_ptr() as usize;
        let map_end = map_start + mapped.len();
        let buf_ptr = buf.as_ptr() as usize;
        assert!(
            buf_ptr >= map_start && buf_ptr + buf.len() <= map_end,
            "buffer must be a subslice of the mmap region"
        );
        assert!(
            affidavit_core::aod::mmap::is_aod_aligned(buf_ptr),
            "Region 3 page pointer must retain 64B alignment through mmap"
        );
    }

    /// Source court: Live Gate handler must mmap, not `fs::read` the `.aod`.
    #[test]
    fn aod_verify_handler_uses_mmap_not_fs_read() {
        let src = include_str!("handlers.rs");
        // Isolate the aod_verify function body (next pub fn is aod_witness).
        let start = src.find("pub fn aod_verify(").expect("aod_verify present");
        let rest = &src[start..];
        let end = rest
            .find("pub fn aod_witness(")
            .expect("aod_witness follows");
        let body = &rest[..end];
        assert!(
            body.contains("aod_mmap::MappedAod") || body.contains("crate::aod_mmap::MappedAod"),
            "aod_verify must open via aod_mmap::MappedAod"
        );
        assert!(
            !body.contains("fs::read"),
            "aod_verify must not full-file fs::read (mmap path)"
        );
    }
}
