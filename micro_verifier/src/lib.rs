//! Embeddable micro-verifier for AOD Historical Witness
//! (AFFI-SPEC-26.11.19 — PR-8 / ecosystem absorption).
//!
//! Doctrine: **certify, don't decide.** This crate is a thin FFI shell over
//! [`affidavit_core::aod::verify_witness`]. It never implements cryptographic
//! algorithms, never maps Region 3, and never burns leases.
//!
//! Consumers:
//! * **C ABI** ([`ffi`]) — WASI hosts, Wasmex, Python ctypes, dirty-NIF loaders.
//! * **BEAM NIF** — documented Rustler stub under `micro_verifier/nif/` (not
//!   compiled here; hosts copy it into their Mix `native/` crate).

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod ffi;

use affidavit_core::aod::{verify_witness, AodRefusal, WitnessAttestation};

/// ABI revision exposed by [`ffi::af_aod_abi_version`].
pub const ABI_VERSION: u32 = 1;

/// Stable C layout of [`WitnessAttestation`] (152 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AfAodWitnessAttestation {
    /// Ontology hash from the header (identity only).
    pub ontology_hash: [u8; 32],
    /// Payload BLAKE3 from the CTRT shadow.
    pub payload_digest: [u8; 32],
    /// Lease epoch observed (not burned).
    pub lease_epoch: u64,
    /// Lease nullifier observed (not burned).
    pub lease_nullifier: [u8; 32],
    /// Folded receipt tip.
    pub folded_receipt: [u8; 32],
    /// Row count from the header.
    pub row_count: u64,
    /// Buffer count from the header.
    pub buffer_count: u32,
    /// Reserved; always zero on ACCEPT.
    pub _reserved: u32,
}

impl From<WitnessAttestation> for AfAodWitnessAttestation {
    fn from(att: WitnessAttestation) -> Self {
        Self {
            ontology_hash: att.ontology_hash,
            payload_digest: att.payload_digest,
            lease_epoch: att.lease_epoch,
            lease_nullifier: att.lease_nullifier,
            folded_receipt: att.folded_receipt,
            row_count: att.row_count,
            buffer_count: att.buffer_count,
            _reserved: 0,
        }
    }
}

/// Safe Historical Witness entry: envelope-only, zero Region 3.
pub fn witness_verify(raw: &[u8]) -> Result<AfAodWitnessAttestation, AodRefusal> {
    verify_witness(raw).map(AfAodWitnessAttestation::from)
}

/// Map a refusal to its stable machine code (`0xE501..=0xE50D`).
pub fn refusal_code(r: AodRefusal) -> i32 {
    i32::from(r.machine_code())
}

#[cfg(test)]
mod tests {
    use super::*;
    use affidavit_core::{seal_aod, SealParams};

    fn minimal_fossil() -> Vec<u8> {
        let ontology = [0x11u8; 32];
        seal_aod(
            &[],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 0,
                lease_epoch: 3,
                lease_nullifier: &[0x22; 32],
                folded_receipt: &[0x33; 32],
            },
        )
        .expect("seal fossil")
    }

    #[test]
    fn abi_layout_is_152_bytes() {
        assert_eq!(core::mem::size_of::<AfAodWitnessAttestation>(), 152);
        assert_eq!(core::mem::align_of::<AfAodWitnessAttestation>(), 8);
    }

    #[test]
    fn witness_accepts_minimal_fossil() {
        let raw = minimal_fossil();
        let att = witness_verify(&raw).expect("ACCEPT");
        assert_eq!(att.ontology_hash, [0x11; 32]);
        assert_eq!(att.lease_epoch, 3);
        assert_eq!(att._reserved, 0);
    }

    #[test]
    fn truncated_refuses_with_e502() {
        let err = witness_verify(&[0u8; 8]).expect_err("truncated");
        assert_eq!(refusal_code(err), 0xE502);
    }

    #[test]
    fn abi_version_is_one() {
        assert_eq!(ABI_VERSION, 1);
    }
}
