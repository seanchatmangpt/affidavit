//! Zero-String Sealed Arrow-on-Disk (AOD) substrate — AFFI-SPEC-26.11.19.
//!
//! Persisted `.aod` artifacts carry **no** UTF-8/ASCII domain tokens: raw
//! 64-byte-aligned buffers bound to a compiled ontology hash and sealed by
//! a 352-byte CTRT hyperdimensional shadow. Dual-mode verification severs
//! operational authority from auditability:
//!
//! * **Live Gate** ([`map_live`]): requires [`CompiledOntology`], burns the
//!   monoatomic lease nullifier, returns [`AodView`] over Region 3. Operates
//!   over borrowed `&[u8]` (zero file-sized heap copy); host crates mmap via
//!   [`mmap`] contract + an OS adapter.
//! * **Historical Witness** ([`verify_witness`]): shared envelope only; returns
//!   inert [`WitnessAttestation`] with **zero** pointers into Region 3.
//!
//! Numeric refusals occupy `0xE501..=0xE50D` (see [`AodRefusal`]).

mod envelope;
mod layout;
mod live;
/// Pointer-alignment contract for host mmap adapters (no_std; no OS mmap here).
pub mod mmap;
mod refuse;
mod witness;
#[cfg(feature = "alloc")]
mod writer;

pub use envelope::{shared_envelope_check, EnvelopeOk};
pub use layout::{
    align64, pad64, AodHeader, AodShadow, BufferEntry, AOD_DIR_ENTRY_LEN, AOD_HEADER_LEN,
    AOD_MAGIC, AOD_MIN_FILE_LEN, AOD_PQC_SEAL_LEN, AOD_SHADOW_LEN, DOMAIN_AOD_SEAL,
};
#[cfg(feature = "alloc")]
pub use live::InMemoryLeaseRegistry;
pub use live::{map_live, AodView, CompiledOntology, LeaseRegistry};
pub use mmap::{is_aod_aligned, slice_is_aod_aligned, AOD_MMAP_ALIGN};
pub use refuse::AodRefusal;
pub use witness::{verify_witness, WitnessAttestation};
#[cfg(feature = "alloc")]
pub use writer::{
    fold_aod_tips, rematerialize_pages, rotate_aod, seal_aod, AodBufferSpec, SealParams,
};
