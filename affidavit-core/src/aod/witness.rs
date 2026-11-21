//! Historical Witness — envelope-only, zero Region 3 access
//! (AFFI-SPEC-26.11.19 §4.3 Mode B).

use super::envelope::shared_envelope_check;
use super::layout::AodShadow;
use super::refuse::AodRefusal;

/// Inert attestation: proves point-in-time envelope validity without mapping data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessAttestation {
    /// Ontology hash recorded in the header (identity only — not matched live).
    pub ontology_hash: [u8; 32],
    /// Payload BLAKE3 from the shadow (Regions 1–3).
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
}

impl WitnessAttestation {
    /// Borrow the shadow fields used to mint this attestation.
    pub fn from_shadow(
        ontology_hash: [u8; 32],
        row_count: u64,
        buffer_count: u32,
        shadow: &AodShadow,
    ) -> Self {
        Self {
            ontology_hash,
            payload_digest: shadow.payload_digest,
            lease_epoch: shadow.lease_epoch,
            lease_nullifier: shadow.lease_nullifier,
            folded_receipt: shadow.folded_receipt,
            row_count,
            buffer_count,
        }
    }
}

/// Historical witness: shared envelope only. Does **not** require a compiled
/// ontology, does **not** burn the lease, and returns no Region 3 pointers.
pub fn verify_witness(raw: &[u8]) -> Result<WitnessAttestation, AodRefusal> {
    let ok = shared_envelope_check(raw)?;
    Ok(WitnessAttestation::from_shadow(
        ok.header.ontology_hash,
        ok.header.row_count,
        ok.header.buffer_count,
        &ok.shadow,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aod::layout::{
        blake3_of, AodHeader, AodShadow, AOD_HEADER_LEN, AOD_MIN_FILE_LEN, AOD_SHADOW_LEN,
    };
    use crate::aod::pqc_seal::{compute_pqc_seal, derive_mldsa_seed, AOD_PQC_MATERIAL_LEN};

    fn minimal_signed(ontology: [u8; 32]) -> Vec<u8> {
        let mut raw = vec![0u8; AOD_HEADER_LEN];
        let header = AodHeader {
            flags: 0,
            row_count: 0,
            buffer_count: 0,
            ontology_hash: ontology,
            trailer_offset: AOD_HEADER_LEN as u64,
        };
        raw[..AOD_HEADER_LEN].copy_from_slice(&header.encode());
        let payload_digest = blake3_of(&raw[..AOD_HEADER_LEN]);
        let seed = derive_mldsa_seed(&ontology);
        let (pqc_seal, pk, sig) =
            compute_pqc_seal(&payload_digest.0, 9, &[0x55; 32], &[0x66; 32], &seed).unwrap();
        let shadow = AodShadow {
            payload_digest: payload_digest.0,
            lease_epoch: 9,
            lease_nullifier: [0x55; 32],
            folded_receipt: [0x66; 32],
            pqc_seal,
        };
        raw.extend_from_slice(&shadow.encode());
        raw.extend_from_slice(&pk);
        raw.extend_from_slice(&sig);
        assert_eq!(raw.len(), AOD_MIN_FILE_LEN);
        assert_eq!(
            raw.len(),
            AOD_HEADER_LEN + AOD_SHADOW_LEN + AOD_PQC_MATERIAL_LEN
        );
        raw
    }

    #[test]
    fn witness_has_no_data_pointers() {
        let raw = minimal_signed([0x44; 32]);
        let att = verify_witness(&raw).expect("witness");
        assert_eq!(att.ontology_hash, [0x44; 32]);
        assert_eq!(att.lease_epoch, 9);
        // Auditor isolation: attestation carries no Region 3 pointer /
        // slice into the data pages (only owned digests + counts).
        let _size = core::mem::size_of_val(&att);
        assert_eq!(
            core::mem::size_of::<WitnessAttestation>(),
            32 * 4 + 8 + 8 + 4 + 4
        );
        let _ = att;
    }

    /// `verify_witness` must not require `CompiledOntology` in its signature
    /// (auditor-isolation compile court — Mode B stands alone).
    #[test]
    fn auditor_isolation_api_has_no_ontology_param() {
        fn assert_witness_sig(_: fn(&[u8]) -> Result<WitnessAttestation, AodRefusal>) {}
        assert_witness_sig(verify_witness);
    }
}
