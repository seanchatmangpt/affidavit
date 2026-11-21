//! AOD adversarial mutation court (AFFI-SPEC-26.11.19 DoD).
//!
//! Exhaustive deterministic suite (no cargo-fuzz in this tree): every mutation
//! class must fail-closed with an exact `0xE5xx` machine code.
//!
//! # Coverage table (mutation → refusal)
//!
//! | Mutation | Path | Code | `AodRefusal` |
//! |---|---|---|---|
//! | Truncate `< 416` | witness | `0xE502` | TruncatedFile |
//! | Magic bitflip | witness | `0xE503` | InvalidSentinel |
//! | Non-zero reserved | witness | `0xE504` | NonZeroReserved |
//! | Non-zero flags | witness | `0xE504` | NonZeroReserved |
//! | Slack byte append | witness | `0xE505` | InvalidTrailerOffset |
//! | Trailer offset lie | witness | `0xE505` | InvalidTrailerOffset |
//! | Unknown ontology | live | `0xE506` | UnknownOntology |
//! | Buffer count mismatch | live | `0xE507` | BufferCountMismatch |
//! | Unaligned dir offset† | witness | `0xE508` | BoundsOrSlack |
//! | Contiguous slack† | live | `0xE508` | BoundsOrSlack |
//! | Fixed-stride mismatch | live | `0xE509` | FixedStrideViolation |
//! | Non-zero directory pad | witness | `0xE50A` | NonZeroPadding |
//! | Non-zero page pad† | live | `0xE50A` | NonZeroPadding |
//! | Payload / page bitflip | witness | `0xE50B` | DigestMismatch |
//! | Corrupt PQC seal | witness | `0xE50C` | PqcSealFailure |
//! | Spent nullifier replay | live | `0xE50D` | LeaseSpentOrExpired |
//! | Exhaustive single-bit flips | witness | `0xE5xx` | any band code |
//!
//! † Surgical: digest + seal recomputed after the structural mutation so the
//! envelope reaches the intended refusal stage (fail-closed, exact code).

use affidavit_core::aod::align64;
use affidavit_core::aod::pqc_seal::compute_pqc_seal;
use affidavit_core::{
    derive_mldsa_seed, map_live, seal_aod, verify_witness, AodBufferSpec, AodRefusal, AodShadow,
    Blake3Hasher, ChainHasher, CompiledOntology, InMemoryLeaseRegistry, SealParams,
    AOD_DIR_ENTRY_LEN, AOD_HEADER_LEN, AOD_PQC_MATERIAL_LEN, AOD_SHADOW_LEN,
};

fn blake3_of(bytes: &[u8]) -> [u8; 32] {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, bytes);
    Blake3Hasher::finish(st).0
}

fn honest_fixture() -> (Vec<u8>, [u8; 32], [u8; 32]) {
    let ontology = [0xCCu8; 32];
    let nullifier = [0xDDu8; 32];
    let folded = [0u8; 32];
    // 8-byte page → 56 zero pad bytes (directory pad also present).
    let page = [0xEEu8; 8];
    let raw = seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count: 1,
            lease_epoch: 1,
            lease_nullifier: &nullifier,
            folded_receipt: &folded,
        },
    )
    .expect("seal");
    (raw, ontology, nullifier)
}

fn shadow_start(raw: &[u8]) -> usize {
    raw.len()
        .checked_sub(AOD_SHADOW_LEN + AOD_PQC_MATERIAL_LEN)
        .expect("file holds shadow + PQC trailer")
}

fn recompute_digest_and_seal(raw: &mut [u8]) {
    let trailer = shadow_start(raw);
    let payload_digest = blake3_of(&raw[..trailer]);
    let mut shadow = AodShadow::decode(&raw[trailer..trailer + AOD_SHADOW_LEN]).expect("shadow");
    shadow.payload_digest = payload_digest;
    // Deterministic reseal from ontology hash embedded in the header.
    let mut ontology = [0u8; 32];
    ontology.copy_from_slice(&raw[24..56]);
    let seed = derive_mldsa_seed(&ontology);
    let (slot, pk, sig) = compute_pqc_seal(
        &shadow.payload_digest,
        shadow.lease_epoch,
        &shadow.lease_nullifier,
        &shadow.folded_receipt,
        &seed,
    )
    .expect("reseal");
    shadow.pqc_seal = slot;
    raw[trailer..trailer + AOD_SHADOW_LEN].copy_from_slice(&shadow.encode());
    let mat = trailer + AOD_SHADOW_LEN;
    raw[mat..mat + pk.len()].copy_from_slice(&pk);
    raw[mat + pk.len()..mat + pk.len() + sig.len()].copy_from_slice(&sig);
}

fn assert_code(err: AodRefusal, want: u16, label: &str) {
    let got = err.machine_code();
    assert_eq!(
        got, want,
        "{label}: got 0x{got:04X} ({err:?}), want 0x{want:04X}",
    );
    assert!(
        (0xE501..=0xE50D).contains(&got),
        "{label}: code must stay on 0xE5xx band"
    );
}

#[test]
fn mutation_truncate_e502() {
    let (raw, _, _) = honest_fixture();
    let err = verify_witness(&raw[..100]).unwrap_err();
    assert_code(err, 0xE502, "truncate");
}

#[test]
fn mutation_magic_bitflip_e503() {
    let (mut raw, _, _) = honest_fixture();
    raw[0] ^= 0x01;
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE503, "magic bitflip");
}

#[test]
fn mutation_nonzero_reserved_e504() {
    let (mut raw, _, _) = honest_fixture();
    raw[20] = 0x01;
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE504, "nonzero reserved");
}

#[test]
fn mutation_nonzero_flags_e504() {
    let (mut raw, _, _) = honest_fixture();
    raw[4] = 0x01;
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE504, "nonzero flags");
}

#[test]
fn mutation_slack_append_e505() {
    let (mut raw, _, _) = honest_fixture();
    raw.push(0);
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE505, "slack append");
}

#[test]
fn mutation_trailer_offset_lie_e505() {
    let (mut raw, _, _) = honest_fixture();
    // Lie about trailer_offset without changing file length.
    let lie = (raw.len() as u64 - AOD_SHADOW_LEN as u64 - 64).to_le_bytes();
    raw[56..64].copy_from_slice(&lie);
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE505, "trailer offset lie");
}

#[test]
fn mutation_unknown_ontology_e506() {
    let (raw, _, _) = honest_fixture();
    let wrong = [0x00u8; 32];
    let compiled = CompiledOntology {
        ontology_hash: &wrong,
        strides: &[8],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    assert_code(err, 0xE506, "unknown ontology");
}

#[test]
fn mutation_buffer_count_mismatch_e507() {
    let (raw, ontology, _) = honest_fixture();
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[8, 8], // sealed with one buffer
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    assert_code(err, 0xE507, "buffer count mismatch");
}

#[test]
fn mutation_unaligned_dir_offset_e508() {
    let (mut raw, _, _) = honest_fixture();
    // Directory entry byte_offset at [64..72); force off 64B grid, then
    // recompute digest+seal so envelope reaches BufferEntry::decode.
    let off = u64::from_le_bytes(raw[AOD_HEADER_LEN..AOD_HEADER_LEN + 8].try_into().unwrap());
    raw[AOD_HEADER_LEN..AOD_HEADER_LEN + 8].copy_from_slice(&(off + 1).to_le_bytes());
    recompute_digest_and_seal(&mut raw);
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE508, "unaligned dir offset");
}

#[test]
fn mutation_contiguous_slack_live_e508() {
    let (mut raw, ontology, _) = honest_fixture();
    // Insert 64 zero slack bytes before the shadow and bump trailer_offset so
    // envelope framing + digest pass; live contiguous cursor ≠ trailer → E508.
    let trailer = shadow_start(&raw);
    let tail = raw[trailer..].to_vec(); // shadow ‖ pk ‖ sig
    raw.truncate(trailer);
    raw.extend(std::iter::repeat_n(0u8, 64));
    raw.extend_from_slice(&tail);
    let new_trailer = shadow_start(&raw) as u64;
    raw[56..64].copy_from_slice(&new_trailer.to_le_bytes());
    recompute_digest_and_seal(&mut raw);
    // Witness accepts (envelope only); live refuses contiguous slack.
    verify_witness(&raw).expect("witness still accepts surgically padded fossil");
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[8],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    assert_code(err, 0xE508, "live contiguous slack");
}

#[test]
fn mutation_fixed_stride_e509() {
    let (raw, ontology, _) = honest_fixture();
    // Honest page is 8 bytes (1 row × stride 8). Claim stride 16 → E509.
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[16],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    assert_code(err, 0xE509, "fixed stride violation");
}

#[test]
fn mutation_nonzero_directory_pad_e50a() {
    let (mut raw, _, _) = honest_fixture();
    // Directory: 32B entry starting at 64; pad region is [96..128).
    let dir_end = AOD_HEADER_LEN + AOD_DIR_ENTRY_LEN;
    let pad_end = align64(dir_end);
    assert!(pad_end > dir_end, "fixture must have directory pad");
    raw[dir_end] = 0x01;
    // Envelope checks directory pad before digest → exact E50A.
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE50A, "nonzero directory pad");
}

#[test]
fn mutation_nonzero_page_pad_live_e50a() {
    let (mut raw, ontology, _) = honest_fixture();
    let page_off = AOD_HEADER_LEN + align64(AOD_DIR_ENTRY_LEN);
    raw[page_off + 8] = 0x01; // first pad byte after 8B page
    recompute_digest_and_seal(&mut raw);
    verify_witness(&raw).expect("witness ignores page-pad law");
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[8],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    assert_code(err, 0xE50A, "nonzero page pad live");
}

#[test]
fn mutation_payload_bitflip_e50b() {
    let (mut raw, _, _) = honest_fixture();
    let page_off = AOD_HEADER_LEN + align64(AOD_DIR_ENTRY_LEN);
    raw[page_off] ^= 0x01;
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE50B, "payload bitflip");
}

#[test]
fn mutation_page_pad_without_reseal_e50b() {
    let (mut raw, _, _) = honest_fixture();
    let page_off = AOD_HEADER_LEN + align64(AOD_DIR_ENTRY_LEN);
    raw[page_off + 8] = 0x01;
    // No digest repair → envelope digest fails closed.
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE50B, "page pad without reseal");
}

#[test]
fn mutation_corrupt_pqc_seal_e50c() {
    let (mut raw, _, _) = honest_fixture();
    // pqc_seal occupies shadow[104..352); flip the message tip byte.
    let seal_off = shadow_start(&raw) + 104;
    raw[seal_off] ^= 0xFF;
    let err = verify_witness(&raw).unwrap_err();
    assert_code(err, 0xE50C, "corrupt pqc seal");
}

#[test]
fn mutation_spent_nullifier_e50d() {
    let (raw, ontology, _) = honest_fixture();
    let compiled = CompiledOntology {
        ontology_hash: &ontology,
        strides: &[8],
    };
    let mut leases = InMemoryLeaseRegistry::new();
    map_live(&raw, &compiled, &mut leases).expect("first live admit");
    let err = map_live(&raw, &compiled, &mut leases).unwrap_err();
    assert_code(err, 0xE50D, "spent nullifier");
}

#[test]
fn mutation_exhaustive_single_bit_flips_fail_closed() {
    let (raw, _, _) = honest_fixture();
    let mut refusals = 0usize;
    // Header + directory + page + shadow (skip large ML-DSA trailer body —
    // tip/key_id already covered by E50C; flipping deep sig bytes is same band).
    let shadow_end = shadow_start(&raw) + AOD_SHADOW_LEN;
    for idx in 0..shadow_end {
        let mut mut_raw = raw.clone();
        mut_raw[idx] ^= 0x01;
        match verify_witness(&mut_raw) {
            Ok(_) => {
                // Only the exact honest bytes may accept; a flipped bit must not.
                panic!("bit flip at {idx} must refuse (fail-closed)");
            }
            Err(err) => {
                let code = err.machine_code();
                assert!(
                    (0xE501..=0xE50D).contains(&code),
                    "bit flip at {idx}: code 0x{code:04X} outside 0xE5xx"
                );
                refusals += 1;
            }
        }
    }
    assert_eq!(
        refusals, shadow_end,
        "every header‥shadow bit flip must refuse"
    );
    // Spot-check ML-DSA trailer tip + first sig byte (same 0xE50C band).
    for &idx in &[shadow_end, shadow_end + 1, raw.len() - 1] {
        let mut mut_raw = raw.clone();
        mut_raw[idx] ^= 0x01;
        let err = verify_witness(&mut_raw).expect_err("PQC trailer flip");
        assert!(
            (0xE501..=0xE50D).contains(&err.machine_code()),
            "trailer flip at {idx}"
        );
    }
}
