//! Chain growth example: visualize rolling BLAKE3 hash evolution.
//!
//! Demonstrates that each appended event changes the chain_hash
//! (the seal is progressive — every new event reshapes the entire
//! provenance fingerprint). Removes the opacity around "what does
//! ChainAssembler do between new() and finalize()?"
//!
//! Key insight: `recompute_chain(&prefix)` re-derives the rolling hash over
//! any prefix of events, so we can show the hash at each append step without
//! finalizing a receipt mid-flight.

use affidavit::chain::{recompute_chain, ChainAssembler};
use affidavit::ocel::{build_event, object_ref, SeqCounter};

fn main() {
    let mut events = Vec::new();
    let mut counter = SeqCounter::new();
    let mut prev_hash: Option<String> = None;

    let steps = [
        ("create", b"source code v1".as_slice()),
        ("transform", b"compiled binary".as_slice()),
        ("test", b"test results: 42/42".as_slice()),
        ("sign", b"GPG signature".as_slice()),
        ("release", b"published to registry".as_slice()),
    ];

    println!("Chain hash evolution:");
    println!(
        "{:<4} {:<14} {}",
        "step", "event_type", "chain_hash (first 24 chars)"
    );
    println!("{}", "-".repeat(60));

    for (i, (ty, payload)) in steps.iter().enumerate() {
        let ev = build_event(
            *ty,
            vec![object_ref("artifact", "artifact")],
            payload,
            &mut counter,
        )
        .expect("build event");
        events.push(ev);

        // Recompute the chain over the growing prefix so far.
        let cur_hash = recompute_chain(&events).expect("recompute");
        println!("{:<4} {:<14} {}", i + 1, ty, &cur_hash.as_hex()[..24]);

        if let Some(ref prev) = prev_hash {
            assert_ne!(
                cur_hash.as_hex(),
                prev.as_str(),
                "each event must change the chain hash (BLAKE3 rolling hash)"
            );
        }
        prev_hash = Some(cur_hash.as_hex().to_string());
    }

    // Final: assemble the full receipt and verify the chain hash matches
    // what recompute_chain computes over all events.
    let mut asm = ChainAssembler::new();
    let mut counter2 = SeqCounter::new();
    for (ty, payload) in &steps {
        let ev = build_event(
            *ty,
            vec![object_ref("artifact", "artifact")],
            payload,
            &mut counter2,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    let expected = recompute_chain(&receipt.events).expect("recompute full");
    assert_eq!(
        receipt.chain_hash, expected,
        "ChainAssembler::finalize must match recompute_chain over the same events"
    );

    println!(
        "\nfinal receipt chain_hash: {}...",
        &receipt.chain_hash.as_hex()[..24]
    );
    println!("OK: rolling BLAKE3 hash changes with every appended event (ChainAssembler = recompute_chain)");
}
