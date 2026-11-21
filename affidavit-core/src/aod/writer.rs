//! AOD seal writer — builds Regions 1–4 under `alloc`
//! (AFFI-SPEC-26.11.19 §5 `affi aod seal`).

use alloc::vec::Vec;

use crate::digest::ChainHasher;

use super::layout::{
    align64, blake3_of, pad64, page_digest_lo, AodHeader, AodShadow, BufferEntry,
    AOD_DIR_ENTRY_LEN, AOD_HEADER_LEN, AOD_SHADOW_LEN,
};
use super::pqc_seal::{compute_pqc_seal, derive_mldsa_seed};
use super::refuse::AodRefusal;

/// One buffer to seal: raw page bytes (exact fixed-stride × rows; no pad).
#[derive(Clone, Copy, Debug)]
pub struct AodBufferSpec<'a> {
    /// Exact page bytes (length must equal stride × row_count).
    pub bytes: &'a [u8],
}

/// Parameters for [`seal_aod`].
#[derive(Clone, Copy, Debug)]
pub struct SealParams<'a> {
    /// Compiled ontology hash embedded in the header.
    pub ontology_hash: &'a [u8; 32],
    /// Row count shared by every buffer.
    pub row_count: u64,
    /// Monoatomic lease epoch.
    pub lease_epoch: u64,
    /// Fresh lease nullifier (32 bytes).
    pub lease_nullifier: &'a [u8; 32],
    /// Folded receipt tip (genesis may be zeros).
    pub folded_receipt: &'a [u8; 32],
}

/// Seal raw buffers into a canonical `.aod` byte vector.
///
/// Uses a domain-derived ML-DSA-65 seed from
/// [`derive_mldsa_seed`](super::pqc_seal::derive_mldsa_seed)`(ontology_hash)`.
/// For an explicit seed (courts / control plane), use [`seal_aod_with_seed`].
///
/// Identical logical inputs → identical bytes (PR-3). Non-zero pad is never
/// written. UTF-8/ASCII labels never appear.
pub fn seal_aod(
    buffers: &[AodBufferSpec<'_>],
    params: &SealParams<'_>,
) -> Result<Vec<u8>, AodRefusal> {
    let seed = derive_mldsa_seed(params.ontology_hash);
    seal_aod_with_seed(buffers, params, &seed)
}

/// Seal raw buffers under an explicit ML-DSA-65 keygen seed.
///
/// Deterministic: same buffers + params + seed ⇒ identical `.aod` bytes.
pub fn seal_aod_with_seed(
    buffers: &[AodBufferSpec<'_>],
    params: &SealParams<'_>,
    mldsa_seed: &[u8; 32],
) -> Result<Vec<u8>, AodRefusal> {
    if buffers.len() > u32::MAX as usize {
        return Err(AodRefusal::BufferCountMismatch);
    }
    let buffer_count = buffers.len() as u32;

    // Layout: header | directory | dir-pad | pages (each padded) | shadow | pk‖sig
    let dir_bytes = buffers.len() * AOD_DIR_ENTRY_LEN;
    let dir_region = if buffer_count == 0 {
        0
    } else {
        align64(AOD_HEADER_LEN + dir_bytes) - AOD_HEADER_LEN
    };
    let data_start = AOD_HEADER_LEN + dir_region;

    let mut cursor = data_start;
    let mut entries: Vec<BufferEntry> = Vec::with_capacity(buffers.len());
    for buf in buffers {
        let len = buf.bytes.len();
        if !cursor.is_multiple_of(64) {
            return Err(AodRefusal::BoundsOrSlack);
        }
        entries.push(BufferEntry {
            byte_offset: cursor as u64,
            byte_length: len as u64,
            page_digest_lo: page_digest_lo(buf.bytes),
        });
        cursor += len + pad64(len);
    }
    let trailer_offset = cursor;

    let header = AodHeader {
        flags: 0,
        row_count: params.row_count,
        buffer_count,
        ontology_hash: *params.ontology_hash,
        trailer_offset: trailer_offset as u64,
    };

    let mut out =
        Vec::with_capacity(trailer_offset + AOD_SHADOW_LEN + super::pqc_seal::AOD_PQC_MATERIAL_LEN);
    out.extend_from_slice(&header.encode());

    if buffer_count > 0 {
        for e in &entries {
            out.extend_from_slice(&e.encode());
        }
        // Zero-pad directory to 64B.
        let pad = align64(out.len()) - out.len();
        out.resize(out.len() + pad, 0);
    }

    for buf in buffers {
        out.extend_from_slice(buf.bytes);
        let p = pad64(buf.bytes.len());
        out.resize(out.len() + p, 0);
    }
    debug_assert_eq!(out.len(), trailer_offset);

    let payload_digest = blake3_of(&out);
    let (pqc_seal, pk, sig) = compute_pqc_seal(
        &payload_digest.0,
        params.lease_epoch,
        params.lease_nullifier,
        params.folded_receipt,
        mldsa_seed,
    )?;
    let shadow = AodShadow {
        payload_digest: payload_digest.0,
        lease_epoch: params.lease_epoch,
        lease_nullifier: *params.lease_nullifier,
        folded_receipt: *params.folded_receipt,
        pqc_seal,
    };
    out.extend_from_slice(&shadow.encode());
    out.extend_from_slice(&pk);
    out.extend_from_slice(&sig);
    Ok(out)
}

/// Rotate: witness the old fossil, re-seal buffers under a fresh lease, fold
/// receipts (`R_hist ⊕ R_rot` ≈ BLAKE3(hist ‖ rot_nullifier)).
///
/// Does **not** map Region 3 for live use of `old`; callers supply the buffers
/// already extracted (or re-materialized) under a prior live lease.
pub fn rotate_aod(
    old_raw: &[u8],
    buffers: &[AodBufferSpec<'_>],
    new_lease_epoch: u64,
    new_nullifier: &[u8; 32],
) -> Result<Vec<u8>, AodRefusal> {
    let att = super::verify_witness(old_raw)?;
    // Folded tip: BLAKE3(old.folded ‖ old.nullifier ‖ new.nullifier).
    let mut st = crate::digest::Blake3Hasher::init();
    crate::digest::Blake3Hasher::absorb(&mut st, &att.folded_receipt);
    crate::digest::Blake3Hasher::absorb(&mut st, &att.lease_nullifier);
    crate::digest::Blake3Hasher::absorb(&mut st, new_nullifier);
    let folded = crate::digest::Blake3Hasher::finish(st);
    seal_aod(
        buffers,
        &SealParams {
            ontology_hash: &att.ontology_hash,
            row_count: att.row_count,
            lease_epoch: new_lease_epoch,
            lease_nullifier: new_nullifier,
            folded_receipt: &folded.0,
        },
    )
}

/// Copy Region 3 page bytes out of a sealed `.aod` for rematerialization
/// (rotate). Envelope-checked only — does **not** burn a lease or return a
/// live [`super::AodView`].
pub fn rematerialize_pages(
    raw: &[u8],
) -> Result<(super::WitnessAttestation, Vec<Vec<u8>>), AodRefusal> {
    let att = super::verify_witness(raw)?;
    let ok = super::shared_envelope_check(raw)?;
    let mut pages = Vec::with_capacity(ok.header.buffer_count as usize);
    for i in 0..ok.header.buffer_count as usize {
        let start = i * AOD_DIR_ENTRY_LEN;
        let entry = BufferEntry::decode(&ok.entries[start..start + AOD_DIR_ENTRY_LEN])?;
        let off = entry.byte_offset as usize;
        let len = entry.byte_length as usize;
        let end = off.checked_add(len).ok_or(AodRefusal::BoundsOrSlack)?;
        if end > ok.header.trailer_offset as usize {
            return Err(AodRefusal::BoundsOrSlack);
        }
        pages.push(raw[off..end].to_vec());
    }
    Ok((att, pages))
}

/// Fold two shadows' folded_receipt tips: BLAKE3(a ‖ b) as the monoid op on
/// the tip digests (trajectory fold projection for AOD fossils).
pub fn fold_aod_tips(a_raw: &[u8], b_raw: &[u8]) -> Result<[u8; 32], AodRefusal> {
    let a = super::verify_witness(a_raw)?;
    let b = super::verify_witness(b_raw)?;
    let mut st = crate::digest::Blake3Hasher::init();
    crate::digest::Blake3Hasher::absorb(&mut st, b"affidavit-core/aod/v1\0fold");
    crate::digest::Blake3Hasher::absorb(&mut st, &a.folded_receipt);
    crate::digest::Blake3Hasher::absorb(&mut st, &b.folded_receipt);
    Ok(crate::digest::Blake3Hasher::finish(st).0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aod::live::{map_live, CompiledOntology, InMemoryLeaseRegistry};
    use crate::aod::verify_witness;

    #[test]
    fn pqc_seeded_seal_positive_and_bitflip_court() {
        let ontology = [0xAAu8; 32];
        let nullifier = [0xBBu8; 32];
        let folded = [0u8; 32];
        let page = [1u8; 16];
        let seed = [0x42u8; 32];
        let params = SealParams {
            ontology_hash: &ontology,
            row_count: 2,
            lease_epoch: 1,
            lease_nullifier: &nullifier,
            folded_receipt: &folded,
        };
        let raw = seal_aod_with_seed(&[AodBufferSpec { bytes: &page }], &params, &seed)
            .expect("seeded seal");
        // Honest path ACCEPT.
        verify_witness(&raw).expect("witness accepts ML-DSA seal");
        // Same seed ⇒ identical bytes.
        let raw2 = seal_aod_with_seed(&[AodBufferSpec { bytes: &page }], &params, &seed).unwrap();
        assert_eq!(raw, raw2);
        // Bit-flip in the signature trailer → 0xE50C.
        let mut flipped = raw.clone();
        let n = flipped.len();
        flipped[n - 1] ^= 0x01;
        let err = verify_witness(&flipped).unwrap_err();
        assert_eq!(err.machine_code(), 0xE50C);
    }

    #[test]
    fn seal_live_witness_round_trip() {
        let ontology = [0xAAu8; 32];
        let nullifier = [0xBBu8; 32];
        let folded = [0u8; 32];
        // 2 rows × stride 8 = 16 bytes
        let page = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let raw = seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 2,
                lease_epoch: 1,
                lease_nullifier: &nullifier,
                folded_receipt: &folded,
            },
        )
        .expect("seal");

        // Zero-text: non-ASCII magic; no 'CTRT' fourcc. (PQC trailer is opaque
        // binary and may contain incidental ASCII bytes such as '{'.)
        assert!(!raw.windows(4).any(|w| w == b"CTRT"));
        assert!(raw[0] >= 0x80);

        let att = verify_witness(&raw).expect("witness");
        assert_eq!(att.ontology_hash, ontology);
        assert_eq!(att.lease_nullifier, nullifier);

        let compiled = CompiledOntology {
            ontology_hash: &ontology,
            strides: &[8],
        };
        let mut leases = InMemoryLeaseRegistry::new();
        let view = map_live(&raw, &compiled, &mut leases).expect("live");
        assert_eq!(view.buffer(0).unwrap(), &page[..]);

        // Replay lease → 0xE50D
        let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
        assert_eq!(err.machine_code(), 0xE50D);
    }

    #[test]
    fn determinism_same_inputs_same_bytes() {
        let ontology = [1u8; 32];
        let n = [2u8; 32];
        let f = [3u8; 32];
        let page = [9u8; 32];
        let params = SealParams {
            ontology_hash: &ontology,
            row_count: 4,
            lease_epoch: 7,
            lease_nullifier: &n,
            folded_receipt: &f,
        };
        let a = seal_aod(&[AodBufferSpec { bytes: &page }], &params).unwrap();
        let b = seal_aod(&[AodBufferSpec { bytes: &page }], &params).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn mutation_court_single_bit_flips() {
        let ontology = [0xCCu8; 32];
        let n = [0xDDu8; 32];
        let f = [0u8; 32];
        let page = [0xEEu8; 64];
        let raw = seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 1,
                lease_epoch: 1,
                lease_nullifier: &n,
                folded_receipt: &f,
            },
        )
        .unwrap();

        // Flip one bit in header, directory, page, and shadow — all must refuse
        // with an exact 0xE5xx machine code (expanded court: tests/aod_mutation_court.rs).
        for &idx in &[0usize, 70, AOD_HEADER_LEN + 80, raw.len() - 10] {
            if idx >= raw.len() {
                continue;
            }
            let mut mut_raw = raw.clone();
            mut_raw[idx] ^= 0x01;
            let err = verify_witness(&mut_raw).expect_err("bit flip must refuse");
            let code = err.machine_code();
            assert!(
                (0xE501..=0xE50D).contains(&code),
                "bit flip at {idx}: got 0x{code:04X}, want 0xE5xx"
            );
        }
    }

    #[test]
    fn zero_text_audit_no_domain_tokens() {
        let ontology = [0xABu8; 32];
        let n = [0xCDu8; 32];
        let f = [0u8; 32];
        // Deliberately include ASCII-looking payload bytes that are still
        // unlabeled ordinals — domain *tokens* must not appear as contiguous UTF-8.
        let page = b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"; // 32 bytes, stride 32 × 1 row
        let raw = seal_aod(
            &[AodBufferSpec { bytes: page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 1,
                lease_epoch: 1,
                lease_nullifier: &n,
                folded_receipt: &f,
            },
        )
        .unwrap();
        for needle in [
            b"CTRT".as_slice(),
            b"ontology",
            b"parquet",
            b"Arrow",
            b"schema",
            b"affidavit-core",
            b"lease_nullifier",
        ] {
            assert!(
                !raw.windows(needle.len()).any(|w| w == needle),
                "domain token leaked"
            );
        }
        // Magic: high bit set on first byte ⇒ not ASCII printable marker.
        assert!(raw[0] >= 0x80);
    }

    #[test]
    fn mutation_court_nonzero_padding_and_slack() {
        let ontology = [0x11u8; 32];
        let n = [0x22u8; 32];
        let f = [0u8; 32];
        // 8-byte page → 56 pad bytes after it inside the data region.
        let page = [7u8; 8];
        let raw = seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 1,
                lease_epoch: 1,
                lease_nullifier: &n,
                folded_receipt: &f,
            },
        )
        .unwrap();
        // Inject non-zero pad after the page (offset = first data page + 8).
        let mut mut_raw = raw.clone();
        let page_off = AOD_HEADER_LEN + align64(AOD_DIR_ENTRY_LEN);
        mut_raw[page_off + 8] = 0x01;
        // Envelope digest must fail closed.
        let err = verify_witness(&mut_raw).unwrap_err();
        assert_eq!(err.machine_code(), 0xE50B);

        // Slack: append a trailing byte before re-checking trailer math by
        // extending the file without updating trailer_offset.
        let mut slack = raw.clone();
        slack.push(0);
        let err = verify_witness(&slack).unwrap_err();
        assert_eq!(err.machine_code(), 0xE505);
    }

    #[test]
    fn rotate_and_fold_compose() {
        let ontology = [0x33u8; 32];
        let n1 = [0x44u8; 32];
        let n2 = [0x55u8; 32];
        let page = [9u8; 16];
        let a = seal_aod(
            &[AodBufferSpec { bytes: &page }],
            &SealParams {
                ontology_hash: &ontology,
                row_count: 2,
                lease_epoch: 1,
                lease_nullifier: &n1,
                folded_receipt: &[0u8; 32],
            },
        )
        .unwrap();
        let (_att, pages) = rematerialize_pages(&a).unwrap();
        let specs: alloc::vec::Vec<_> = pages
            .iter()
            .map(|p| AodBufferSpec {
                bytes: p.as_slice(),
            })
            .collect();
        let b = rotate_aod(&a, &specs, 2, &n2).unwrap();
        let tip = fold_aod_tips(&a, &b).unwrap();
        let tip2 = fold_aod_tips(&a, &b).unwrap();
        assert_eq!(tip, tip2);
        assert_ne!(tip, [0u8; 32]);
    }
}
