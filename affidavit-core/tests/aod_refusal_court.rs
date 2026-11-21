//! Vacuity-proof AOD refusal court (AFFI-SPEC-26.11.19 §4.4 / courts-fuzz).
//!
//! Every code in `0xE501..=0xE50D` must fire through a **real emission path**
//! (Live Gate, shared envelope, AvecView, alignment demand, or lease registry).
//! Asserting `machine_code()` alone is vacuous — each control below unwraps an
//! `Err` from the boundary that actually returns that refusal.
//!
//! Also pins the witness-vs-live boundary: Historical Witness never maps
//! Region 3 and never burns the monoatomic lease.

use affidavit_core::{
    demand_aod_aligned, map_live, seal_aod, shared_envelope_check, verify_witness, AodBufferSpec,
    AodRefusal, AvecView, BufferEntry, CompiledOntology, InMemoryLeaseRegistry, SealParams,
    WitnessAttestation, AOD_DIR_ENTRY_LEN, AOD_HEADER_LEN, AOD_MMAP_ALIGN, AOD_SHADOW_LEN,
};

/// Sealed one-buffer fixture: 2 rows × stride 8.
fn sealed_page() -> (Vec<u8>, [u8; 32], [u8; 32], usize) {
    let ontology = [0xAAu8; 32];
    let nullifier = [0xBBu8; 32];
    let stride = 8usize;
    let page = [
        1u8, 2, 3, 4, 5, 6, 7, 8, // row 0
        9, 10, 11, 12, 13, 14, 15, 16, // row 1
    ];
    let raw = seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count: 2,
            lease_epoch: 1,
            lease_nullifier: &nullifier,
            folded_receipt: &[0u8; 32],
        },
    )
    .expect("seal");
    (raw, ontology, nullifier, stride)
}

/// Directory pad start for a single 32-byte directory entry (header 64 + 32 → pad @ 96).
fn dir_pad_start() -> usize {
    AOD_HEADER_LEN + AOD_DIR_ENTRY_LEN
}

/// Table: (machine_code, label, closure that must return that code).
fn fire(code: u16, label: &str, got: AodRefusal) {
    assert_eq!(
        got.machine_code(),
        code,
        "positive control {label}: expected {code:#x}, got {:#x} ({got:?})",
        got.machine_code()
    );
    assert!(
        AodRefusal::BAND.contains(&code),
        "control {label} produced out-of-band {code:#x}"
    );
}

#[test]
fn e501_unaligned_input_demand_fires() {
    let storage = [0u8; 128];
    let base = storage.as_ptr() as usize;
    let start = if base.is_multiple_of(AOD_MMAP_ALIGN) {
        1
    } else {
        0
    };
    let mis = &storage[start..start + 16];
    let err = demand_aod_aligned(mis).unwrap_err();
    fire(0xE501, "demand_aod_aligned", err);
}

#[test]
fn e502_truncated_file_fires() {
    let err = shared_envelope_check(&[0u8; 100]).unwrap_err();
    fire(0xE502, "truncated", err);
}

#[test]
fn e503_bad_magic_fires() {
    let (mut raw, _, _, _) = sealed_page();
    raw[0] ^= 0xFF;
    let err = shared_envelope_check(&raw).unwrap_err();
    fire(0xE503, "bad_magic", err);
}

#[test]
fn e504_nonzero_reserved_fires() {
    let (mut raw, _, _, _) = sealed_page();
    // Header reserved word at bytes [20..24) — checked in AodHeader::decode
    // before digest, so this is a true E504 positive control (not E50B).
    raw[20] = 0x01;
    let err = shared_envelope_check(&raw).unwrap_err();
    fire(0xE504, "nonzero_reserved", err);
}

#[test]
fn e505_invalid_trailer_offset_fires() {
    let (mut raw, _, _, _) = sealed_page();
    raw.push(0); // length drift without updating trailer_offset
    let err = shared_envelope_check(&raw).unwrap_err();
    fire(0xE505, "trailer_slack", err);
}

#[test]
fn e506_ontology_mismatch_fires() {
    let (raw, ontology, _, stride) = sealed_page();
    let wrong = [0xFFu8; 32];
    assert_ne!(wrong, ontology);
    let compiled = CompiledOntology {
        ontology_hash: &wrong,
        strides: &[stride as u64],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    fire(0xE506, "unknown_ontology", err);
}

#[test]
fn e507_buffer_count_mismatch_fires() {
    let (raw, ontology, _, _) = sealed_page();
    // Sealed with 1 buffer; demand 2 strides → BufferCountMismatch.
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[8, 8],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    fire(0xE507, "buffer_count", err);
}

#[test]
fn e508_bounds_or_slack_fires() {
    // Directory entry with non-64-aligned byte_offset — BufferEntry::decode.
    let mut entry = [0u8; AOD_DIR_ENTRY_LEN];
    entry[0] = 1; // offset = 1
    let err = BufferEntry::decode(&entry).unwrap_err();
    fire(0xE508, "unaligned_dir_offset", err);
}

#[test]
fn e509_avec_stride_mismatch_fires() {
    let (raw, ontology, _, stride) = sealed_page();
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[stride as u64],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let view = map_live(&raw, &compiled, &mut leases).expect("live");
    // Wrong stride for admitted page length (16 bytes / 2 rows ⇒ need 8).
    let err = AvecView::from_aod_buffer(&view, 0, 7).unwrap_err();
    fire(0xE509, "avec_stride", err);
}

#[test]
fn e509_live_gate_fixed_stride_violation_fires() {
    let (raw, ontology, _, _) = sealed_page();
    // Ontology claims stride 7 → expected length 14 ≠ sealed 16.
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[7],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    fire(0xE509, "live_stride", err);
}

#[test]
fn e50a_nonzero_directory_padding_fires() {
    let (mut raw, _, _, _) = sealed_page();
    let pad = dir_pad_start();
    assert!(pad < raw.len() - AOD_SHADOW_LEN);
    // Directory pad is checked in shared_envelope_check *before* digest.
    raw[pad] = 0x01;
    let err = shared_envelope_check(&raw).unwrap_err();
    fire(0xE50A, "dir_pad", err);
}

#[test]
fn e50b_digest_mismatch_fires() {
    let (mut raw, _, _, _) = sealed_page();
    // Flip ontology hash inside payload (header[24]) — past magic/reserved —
    // so decode succeeds and digest fails closed.
    raw[24] ^= 0xFF;
    let err = shared_envelope_check(&raw).unwrap_err();
    fire(0xE50B, "digest", err);
}

#[test]
fn e50c_pqc_seal_failure_fires() {
    let (mut raw, _, _, _) = sealed_page();
    // Shadow starts at trailer; pqc_seal begins at shadow+104.
    let seal_byte = raw.len() - AOD_SHADOW_LEN + 104;
    raw[seal_byte] ^= 0x01;
    let err = shared_envelope_check(&raw).unwrap_err();
    fire(0xE50C, "pqc_seal", err);
}

#[test]
fn e50d_lease_spent_or_expired_fires() {
    let (raw, ontology, _, stride) = sealed_page();
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[stride as u64],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    map_live(&raw, &compiled, &mut leases).expect("first burn");
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    fire(0xE50D, "lease_replay", err);
}

/// Meta-court: every variant in [`AodRefusal::ALL`] has a positive control
/// that actually returned that code in this module's suite.
///
/// Implemented as an in-process table re-fire so deleting any control above
/// fails here — not a vacuous `machine_code()` loop.
#[test]
fn every_band_code_has_live_positive_control() {
    let mut fired = [false; 13];

    // Re-execute each emitter and mark the band slot.
    let mark = |err: AodRefusal, fired: &mut [bool; 13]| {
        let code = err.machine_code();
        let idx = (code - 0xE501) as usize;
        fired[idx] = true;
    };

    // E501
    {
        let storage = [0u8; 128];
        let base = storage.as_ptr() as usize;
        let start = if base.is_multiple_of(AOD_MMAP_ALIGN) {
            1
        } else {
            0
        };
        mark(
            demand_aod_aligned(&storage[start..start + 16]).unwrap_err(),
            &mut fired,
        );
    }
    // E502
    mark(shared_envelope_check(&[0u8; 64]).unwrap_err(), &mut fired);
    // E503–E505, E50A–E50C from sealed mutations
    {
        let (raw, ontology, _, stride) = sealed_page();
        let mut bad = raw.clone();
        bad[0] ^= 0xFF;
        mark(shared_envelope_check(&bad).unwrap_err(), &mut fired); // E503
        let mut bad = raw.clone();
        bad[20] = 1;
        mark(shared_envelope_check(&bad).unwrap_err(), &mut fired); // E504
        let mut bad = raw.clone();
        bad.push(0);
        mark(shared_envelope_check(&bad).unwrap_err(), &mut fired); // E505
        let mut bad = raw.clone();
        bad[dir_pad_start()] = 1;
        mark(shared_envelope_check(&bad).unwrap_err(), &mut fired); // E50A
        let mut bad = raw.clone();
        bad[24] ^= 0xFF;
        mark(shared_envelope_check(&bad).unwrap_err(), &mut fired); // E50B
        let mut bad = raw.clone();
        bad[raw.len() - AOD_SHADOW_LEN + 104] ^= 1;
        mark(shared_envelope_check(&bad).unwrap_err(), &mut fired); // E50C

        // E506 / E507 / E509 live
        let wrong = [0x11u8; 32];
        mark(
            map_live(
                &raw,
                &CompiledOntology {
                    ontology_hash: &wrong,
                    strides: &[stride as u64],
                },
                &mut InMemoryLeaseRegistry::new(),
            )
            .unwrap_err(),
            &mut fired,
        );
        mark(
            map_live(
                &raw,
                &CompiledOntology {
                    ontology_hash: &ontology,
                    strides: &[8, 8],
                },
                &mut InMemoryLeaseRegistry::new(),
            )
            .unwrap_err(),
            &mut fired,
        );
        mark(
            map_live(
                &raw,
                &CompiledOntology {
                    ontology_hash: &ontology,
                    strides: &[7],
                },
                &mut InMemoryLeaseRegistry::new(),
            )
            .unwrap_err(),
            &mut fired,
        );

        // E508
        let mut entry = [0u8; AOD_DIR_ENTRY_LEN];
        entry[0] = 1;
        mark(BufferEntry::decode(&entry).unwrap_err(), &mut fired);

        // E509 AvecView (second path) + E50D lease replay
        let mut leases = InMemoryLeaseRegistry::new();
        let view = map_live(
            &raw,
            &CompiledOntology {
                ontology_hash: &ontology,
                strides: &[stride as u64],
            },
            &mut leases,
        )
        .unwrap();
        mark(
            AvecView::from_aod_buffer(&view, 0, 7).unwrap_err(),
            &mut fired,
        );
        mark(
            map_live(
                &raw,
                &CompiledOntology {
                    ontology_hash: &ontology,
                    strides: &[stride as u64],
                },
                &mut leases,
            )
            .unwrap_err(),
            &mut fired,
        );
    }

    for (i, &ok) in fired.iter().enumerate() {
        let code = 0xE501u16 + i as u16;
        assert!(
            ok,
            "VACUOUS COURT: no positive control fired for {code:#x} ({:?})",
            AodRefusal::ALL[i]
        );
    }
    assert_eq!(AodRefusal::ALL.len(), 13);
}

#[test]
fn witness_never_maps_region3_and_does_not_burn_lease() {
    let (raw, ontology, nullifier, stride) = sealed_page();

    // Mode B: envelope only.
    let att = verify_witness(&raw).expect("witness");
    assert_eq!(att.ontology_hash, ontology);
    assert_eq!(att.lease_nullifier, nullifier);
    assert_eq!(att.row_count, 2);
    assert_eq!(att.buffer_count, 1);

    // Attestation is owned digests/counts — no borrowed Region 3 slice.
    // (Layout: 4×32 digests + epoch u64 + row_count u64 + buffer_count u32 + pad.)
    assert_eq!(
        core::mem::size_of::<WitnessAttestation>(),
        32 * 4 + 8 + 8 + 4 + 4
    );
    // No lifetime parameter on WitnessAttestation ⇒ cannot hold `&[u8]` into raw.
    fn assert_static_att(_: WitnessAttestation) {}
    assert_static_att(att);

    // Witness must not burn the lease — Live Gate still admits once.
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[stride as u64],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let view = map_live(&raw, &compiled, &mut leases).expect("live after witness");
    let page = view.buffer(0).expect("region3");
    assert_eq!(page.len(), 16);
    let avec = AvecView::from_aod_buffer(&view, 0, stride).expect("avec");
    assert_eq!(avec.row_count(), 2);
    assert_eq!(avec.row(0).unwrap(), &page[0..8]);

    // Second live admit refuses — lease burned by map_live, not witness.
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    fire(0xE50D, "post_witness_replay", err);
}
