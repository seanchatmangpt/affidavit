//! Build a BLAKE3 receipt chain, finalize it, and demonstrate tamper-evidence.
//!
//! End-to-end: assemble 3 events via `ChainAssembler`, `finalize()` to a
//! `Receipt`, then recompute the chain from the receipt's events and assert it
//! reproduces `receipt.chain_hash`. The edge the chain exists for: mutating one
//! event's `payload_commitment` and proving `recompute_chain` now DIFFERS.
//!
//! See the doc on `ChainAssembler` and `recompute_chain` in src/chain.rs.

use affidavit::chain::{deserialize_receipt, recompute_chain, serialize_receipt, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, Receipt};

fn main() {
    // --- Build a 3-event chain via the append-only assembler ---
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();

    assert!(asm.is_empty(), "fresh assembler must hold no events");

    asm.append(
        build_event(
            "artifact.seed",
            vec![object_ref("a1", "artifact")],
            b"seed payload",
            &mut counter,
        )
        .expect("event 0 well-formed"),
    )
    .expect("append event 0");

    asm.append(
        build_event(
            "artifact.transform",
            vec![object_ref("a1", "artifact")],
            b"transform payload",
            &mut counter,
        )
        .expect("event 1 well-formed"),
    )
    .expect("append event 1");

    asm.append(
        build_event(
            "artifact.release",
            vec![object_ref("a1", "artifact")],
            b"release payload",
            &mut counter,
        )
        .expect("event 2 well-formed"),
    )
    .expect("append event 2");

    assert_eq!(asm.len(), 3, "exactly three events appended");
    assert!(!asm.is_empty());

    // --- Finalize into an immutable Receipt carrying the final chain hash ---
    let receipt: Receipt = asm.finalize();
    println!("chain_hash = {}", receipt.chain_hash.as_hex());

    // --- Reproducibility: re-derive the chain from the receipt's events alone ---
    let recomputed = recompute_chain(&receipt.events).expect("recompute over honest events");
    assert_eq!(
        recomputed, receipt.chain_hash,
        "chain must be reproducible from event bytes alone"
    );
    println!("reproduced chain_hash from events: MATCH");

    // --- EDGE: tamper-evidence. Mutate one event's payload_commitment. ---
    let mut tampered = receipt.events.clone();
    tampered[0].payload_commitment = Blake3Hash::from_bytes(b"forged seed payload");

    let forged = recompute_chain(&tampered).expect("recompute over tampered events");
    assert_ne!(
        forged, receipt.chain_hash,
        "tampering an event's commitment MUST break the chain hash"
    );
    println!("tampered chain_hash = {}", forged.as_hex());
    println!("tamper detected: forged chain hash differs from sealed receipt");

    // --- from_events: rebuild an assembler from existing events and re-finalize ---
    // Rehydrating the receipt's own events must reproduce the identical chain hash
    // (re-finalizing is deterministic over the same event sequence).
    let rebuilt = ChainAssembler::from_events(receipt.events.clone())
        .expect("honest events rehydrate into an assembler");
    assert_eq!(rebuilt.events(), receipt.events.as_slice(), "events() returns the held sequence");
    let refinalized = rebuilt.finalize();
    assert_eq!(
        refinalized.chain_hash, receipt.chain_hash,
        "re-finalizing the same events reproduces the chain hash (determinism)"
    );

    // --- serialize/deserialize round-trip: canonical bytes reload to an equal receipt ---
    let bytes = serialize_receipt(&receipt).expect("canonical serialization");
    let reloaded = deserialize_receipt(&bytes).expect("canonical bytes reload");
    assert_eq!(reloaded.chain_hash, receipt.chain_hash, "round-trip preserves the chain hash");
    assert_eq!(reloaded.events.len(), receipt.events.len(), "round-trip preserves all events");
    // EDGE (verified against the runtime contract): the `Receipt` Deserialize impl
    // RE-VERIFIES the chain hash at decode time, so deserialize_receipt REJECTS a
    // receipt whose stated chain_hash does not match a recompute over its events.
    let mut corrupt: serde_json::Value = serde_json::from_slice(&bytes).expect("valid json");
    corrupt["chain_hash"] = serde_json::json!("0".repeat(64));
    let corrupt_bytes = serde_json::to_vec(&corrupt).expect("reserialize");
    let err = deserialize_receipt(&corrupt_bytes)
        .expect_err("a chain-hash mismatch must be refused at deserialization");
    // The error names the mismatch (claimed vs recomputed) — tamper-evidence at the
    // decode boundary, not merely at verify/admit time.
    assert!(
        format!("{err}").contains("chain hash mismatch"),
        "deserialize must reject with a chain-hash-mismatch error; got: {err}"
    );
    println!("from_events re-finalize MATCH; serialize/deserialize round-trip OK; corrupt hash refused at decode: {err}");
}
