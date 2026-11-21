//! Arrow-on-Disk acceptance court (AFFI-SPEC-26.11.19).
//!
//! Proves sealed Region-3 pages are consumable as fixed-stride **avec** rows
//! after Live Gate admit — the JTBD bar that `.aod` "works in Arrow" (avec
//! semantics), not merely that seal/verify exit 0.
//!
//! Path: `seal_aod` → `map_live` → [`AvecView::from_aod_buffer`] → row bytes.
//! Historical witness remains envelope-only (no Region 3).

use affidavit_core::{
    ffor_pack, ffor_packed_bytes, map_live, seal_aod, verify_witness, AodBufferSpec, AvecView,
    CompiledOntology, FforView, InMemoryLeaseRegistry, SealParams,
};

#[test]
fn sealed_page_admits_as_avec_rows_after_live_gate() {
    let ontology = [0xAAu8; 32];
    let nullifier = [0xBBu8; 32];
    let stride = 8usize;
    let rows = 4u64;
    // Arrow-avec style: fixed-width little-endian i64-ish cells (8 bytes).
    let mut page = vec![0u8; stride * rows as usize];
    for r in 0..rows as usize {
        let v = (r as u64 + 1).to_le_bytes();
        page[r * stride..(r + 1) * stride].copy_from_slice(&v);
    }

    let raw = seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count: rows,
            lease_epoch: 1,
            lease_nullifier: &nullifier,
            folded_receipt: &[0u8; 32],
        },
    )
    .expect("seal");

    // Witness never maps Region 3.
    let att = verify_witness(&raw).expect("witness");
    assert_eq!(att.row_count, rows);
    assert_eq!(att.buffer_count, 1);

    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[stride as u64],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let view = map_live(&raw, &compiled, &mut leases).expect("live");

    let avec = AvecView::from_aod_buffer(&view, 0, stride).expect("avec");
    assert_eq!(avec.row_count(), rows as usize);
    assert_eq!(avec.stride(), stride);
    for r in 0..rows as usize {
        let row = avec.row(r).expect("row");
        let got = u64::from_le_bytes(row.try_into().unwrap());
        assert_eq!(got, r as u64 + 1, "row {r}");
    }
}

#[test]
fn avec_refuses_stride_mismatch() {
    let ontology = [0x11u8; 32];
    let page = [0u8; 16]; // 2×8
    let raw = seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count: 2,
            lease_epoch: 1,
            lease_nullifier: &[0x22; 32],
            folded_receipt: &[0u8; 32],
        },
    )
    .unwrap();
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[8],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let view = map_live(&raw, &compiled, &mut leases).unwrap();
    // Wrong stride for the admitted page length.
    let err = AvecView::from_aod_buffer(&view, 0, 7).unwrap_err();
    assert_eq!(err.machine_code(), 0xE509);
}

#[test]
fn sealed_ffor_page_admits_and_unpacks_after_live_gate() {
    // Byte-aligned FFOR (bit_width=8) so Live Gate fixed-stride admits the page
    // as stride=1 × row_count, then FforView consumes Region-3 as packed deltas.
    let ontology = [0xCCu8; 32];
    let values = [10u64, 11, 14, 18];
    let base = 10u64;
    let bit_width = 8u8;
    let n = values.len();
    let nbytes = ffor_packed_bytes(n, bit_width).expect("pack size");
    let mut page = vec![0u8; nbytes];
    ffor_pack(&values, base, bit_width, &mut page).expect("ffor_pack");

    let raw = seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count: n as u64,
            lease_epoch: 1,
            lease_nullifier: &[0xDD; 32],
            folded_receipt: &[0u8; 32],
        },
    )
    .expect("seal");

    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[1], // 1 byte/row at bit_width 8
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let view = map_live(&raw, &compiled, &mut leases).expect("live");
    let packed = view.buffer(0).expect("buf");
    let ffor = FforView::try_new(packed, base, bit_width, n).expect("ffor");
    let mut out = [0u64; 4];
    ffor.unpack_into(&mut out).expect("unpack");
    assert_eq!(out, values);
}
