//! Thin C ABI for AOD Historical Witness.
//!
//! The only `unsafe` in this crate lives here. Safe logic stays in
//! [`crate::witness_verify`].

#![allow(unsafe_code)]

use crate::{refusal_code, witness_verify, AfAodWitnessAttestation, ABI_VERSION};

/// ABI revision; see [`crate::ABI_VERSION`].
#[no_mangle]
pub extern "C" fn af_aod_abi_version() -> u32 {
    ABI_VERSION
}

/// Historical Witness over a raw `.aod` buffer.
///
/// # Safety
/// * `raw` must be readable for `len` bytes when non-null and `len > 0`.
/// * `out` must be a valid writable pointer to [`AfAodWitnessAttestation`].
///
/// Returns `0` on ACCEPT (and writes `out`), a positive `0xE5xx` refusal code
/// on envelope failure, or `-1` when `raw`/`out` is null or `len == 0`.
#[no_mangle]
pub unsafe extern "C" fn af_aod_witness_verify(
    raw: *const u8,
    len: usize,
    out: *mut AfAodWitnessAttestation,
) -> i32 {
    if raw.is_null() || out.is_null() || len == 0 {
        return -1;
    }
    // SAFETY: caller guarantees `raw`/`len` and `out` are valid for this call.
    let slice = unsafe { core::slice::from_raw_parts(raw, len) };
    match witness_verify(slice) {
        Ok(att) => {
            unsafe {
                *out = att;
            }
            0
        }
        Err(r) => refusal_code(r),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use affidavit_core::{seal_aod, SealParams};

    fn fossil() -> Vec<u8> {
        let ontology = [0xABu8; 32];
        seal_aod(
            &[],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 1,
                lease_epoch: 1,
                lease_nullifier: &[0xCD; 32],
                folded_receipt: &[0xEF; 32],
            },
        )
        .expect("seal fossil")
    }

    #[test]
    fn c_abi_accepts_fossil() {
        let raw = fossil();
        let mut out = AfAodWitnessAttestation {
            ontology_hash: [0; 32],
            payload_digest: [0; 32],
            lease_epoch: 0,
            lease_nullifier: [0; 32],
            folded_receipt: [0; 32],
            row_count: 0,
            buffer_count: 0,
            _reserved: 0,
        };
        let rc = unsafe { af_aod_witness_verify(raw.as_ptr(), raw.len(), &mut out) };
        assert_eq!(rc, 0);
        assert_eq!(out.ontology_hash, [0xAB; 32]);
        assert_eq!(out.lease_epoch, 1);
    }

    #[test]
    fn c_abi_null_input_is_minus_one() {
        let mut out = AfAodWitnessAttestation {
            ontology_hash: [0; 32],
            payload_digest: [0; 32],
            lease_epoch: 0,
            lease_nullifier: [0; 32],
            folded_receipt: [0; 32],
            row_count: 0,
            buffer_count: 0,
            _reserved: 0,
        };
        assert_eq!(
            unsafe { af_aod_witness_verify(core::ptr::null(), 16, &mut out) },
            -1
        );
    }
}
