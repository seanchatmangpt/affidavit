//! Deterministic content-addressing (NFR-1): a Receipt's canonical bytes are a
//! pure function of its logical value, and BLAKE3 hex round-trips losslessly.
//!
//! Demonstrates the contract of `canonical_bytes`, `Blake3Hash`, and `ProfileId::as_str`.
//! See the docs on `canonical_bytes`, `Blake3Hash`, and `ProfileId` in src/types.rs.
//! Run: `cargo run --example receipt_determinism`

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{canonical_bytes, Blake3Hash, ProfileId};

fn make_receipt(event_type: &str, payload: &[u8]) -> affidavit::types::Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let event = build_event(
        event_type,
        vec![object_ref("obj-1", "artifact")],
        payload,
        &mut counter,
    )
    .expect("build event");
    asm.append(event).expect("append");
    asm.finalize()
}

fn main() {
    // --- Determinism (NFR-1): same logical value -> byte-identical canonical bytes ---
    let receipt = make_receipt("seed", b"payload-bytes");
    let bytes_a = canonical_bytes(&receipt).expect("canonical bytes a");
    let bytes_b = canonical_bytes(&receipt).expect("canonical bytes b");
    assert_eq!(
        bytes_a, bytes_b,
        "canonical_bytes must be deterministic: two computations of the same receipt diverged"
    );

    // --- BLAKE3 hex round-trip: as_hex() -> from_hex() preserves the digest ---
    let hash = Blake3Hash::from_bytes(&bytes_a);
    let hex = hash.as_hex();
    let rehydrated = Blake3Hash::from_hex(hex);
    assert_eq!(
        hash, rehydrated,
        "Blake3Hash hex must round-trip: from_hex(as_hex(h)) != h"
    );
    // from_bytes is itself deterministic over identical input bytes.
    assert_eq!(
        hash,
        Blake3Hash::from_bytes(&bytes_b),
        "Blake3Hash::from_bytes must be deterministic over identical bytes"
    );

    // --- NEGATIVE: a DIFFERENT receipt yields DIFFERENT canonical bytes (and hash) ---
    let other = make_receipt("breed", b"payload-bytes");
    let other_bytes = canonical_bytes(&other).expect("canonical bytes other");
    assert_ne!(
        bytes_a, other_bytes,
        "distinct receipts must produce distinct canonical bytes — content-addressing collision"
    );
    assert_ne!(
        Blake3Hash::from_bytes(&bytes_a),
        Blake3Hash::from_bytes(&other_bytes),
        "distinct receipts must produce distinct content addresses"
    );

    // --- ProfileId::as_str: stable string identifier ---
    assert_eq!(ProfileId::CoreV1.as_str(), "core/v1");

    println!(
        "OK: deterministic canonical_bytes ({} bytes), hash {} round-trips, distinct receipt differs, profile {}",
        bytes_a.len(),
        hash.as_hex(),
        ProfileId::CoreV1.as_str()
    );
}
