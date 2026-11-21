//! AFFI-SPEC-26.11.19 PR-3 — canonical cross-arch AOD determinism court.
//!
//! Law: identical logical inputs → identical `.aod` bytes → identical BLAKE3
//! digests on every LE host (x86_64, aarch64). Standing:
//!
//! * **Golden-vector drift court** (always): reseal fixed inputs; refuse if
//!   bytes or BLAKE3 diverge from
//!   `tests/fixtures/aod_cross_arch/vector_v1.{aod,blake3}`.
//! * **Dual-arch CI** (when runners admit): `.github/workflows/aod-cross-arch.yml`
//!   runs this court on `ubuntu-latest` (x86_64) and `ubuntu-24.04-arm`
//!   (aarch64), uploads digests, and asserts equality. If ARM runners are
//!   unavailable for the repo, golden-vector standing remains the honest
//!   ceiling — never invent cross-arch evidence.
//!
//! Does not change seal crypto, live mmap, or CLI verb registry counts.

use affidavit_core::{seal_aod, verify_witness, AodBufferSpec, SealParams, AOD_MAGIC};
use std::fs;
use std::path::PathBuf;

/// Logical inputs pinned in `tests/fixtures/aod_cross_arch/VECTOR.md`.
fn logical_inputs() -> ([u8; 32], [u8; 32], [u8; 32], [u8; 16], u64, u64) {
    (
        [0xAAu8; 32],
        [0xBBu8; 32],
        [0x00u8; 32],
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        2, // row_count
        1, // lease_epoch
    )
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/aod_cross_arch")
}

fn seal_canonical() -> Vec<u8> {
    let (ontology, nullifier, folded, page, row_count, lease_epoch) = logical_inputs();
    seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &ontology,
            row_count,
            lease_epoch,
            lease_nullifier: &nullifier,
            folded_receipt: &folded,
        },
    )
    .expect("canonical seal must succeed")
}

fn blake3_hex(bytes: &[u8]) -> String {
    hex::encode(blake3::hash(bytes).as_bytes())
}

/// Reseal equals the committed golden `.aod` bytes and the pinned BLAKE3.
#[test]
fn aod_cross_arch_golden_vector_refuses_digest_drift() {
    let dir = fixture_dir();
    let golden = fs::read(dir.join("vector_v1.aod")).expect("vector_v1.aod present");
    let pinned = fs::read_to_string(dir.join("vector_v1.blake3"))
        .expect("vector_v1.blake3 present")
        .trim()
        .to_owned();

    let sealed = seal_canonical();
    assert_eq!(
        sealed, golden,
        "REFUSED_AOD_CROSS_ARCH_BYTE_DRIFT: reseal must be byte-identical to golden .aod"
    );

    let digest = blake3_hex(&sealed);
    assert_eq!(
        digest, pinned,
        "REFUSED_AOD_CROSS_ARCH_DIGEST_DRIFT: BLAKE3 must match pinned vector_v1.blake3"
    );

    // Anti-vacuity: golden is a live envelope, not an empty pin.
    assert_eq!(&sealed[0..4], &AOD_MAGIC);
    assert_eq!(sealed.len(), 544);
    verify_witness(&sealed).expect("golden must witness-ACCEPT");
}

/// Same logical inputs twice in-process → identical digest (local determinism).
#[test]
fn aod_cross_arch_inprocess_replay_identical_digest() {
    let a = seal_canonical();
    let b = seal_canonical();
    assert_eq!(a, b);
    assert_eq!(blake3_hex(&a), blake3_hex(&b));
}

/// Host arch is recorded for CI artifact naming; LE hosts only (x86_64/aarch64).
#[test]
fn aod_cross_arch_host_is_little_endian() {
    // Compile-time LE gate (AOD header/directory/shadow are explicit LE).
    const _: () = assert!(cfg!(target_endian = "little"));
    // Emit standing line for CI log / artifact collectors.
    let digest = blake3_hex(&seal_canonical());
    eprintln!(
        "AOD_CROSS_ARCH_STANDING arch={} endian=little digest={}",
        std::env::consts::ARCH,
        digest
    );
}

/// Zero-text: golden `.aod` carries no UTF-8 domain tokens (PR-2 invariant preserved).
#[test]
fn aod_cross_arch_golden_remains_zero_text() {
    let golden = fs::read(fixture_dir().join("vector_v1.aod")).unwrap();
    for needle in [
        b"CTRT".as_slice(),
        b"ontology",
        b"lease_nullifier",
        b"affidavit",
        b"parquet",
        b"Arrow",
    ] {
        assert!(
            !golden.windows(needle.len()).any(|w| w == needle),
            "domain token leaked into golden .aod"
        );
    }
}
