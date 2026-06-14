//! Example: build a receipt using the type-safe EventBuilder API.
//!
//! Run: `cargo run --example event_builder`

use affidavit::chain::ChainAssembler;
use affidavit::event_builder::EventBuilder;
use affidavit::ocel::SeqCounter;
use affidavit::verifier::verify;

fn main() {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();

    let events = [
        EventBuilder::new("create")
            .object("source.rs", "artifact")
            .payload_str("fn main() {}")
            .build(&mut counter)
            .expect("build create event"),
        EventBuilder::new("transform")
            .object("source.rs", "artifact")
            .qualified_object("binary", "artifact", "output")
            .payload_str("compiled")
            .build(&mut counter)
            .expect("build transform event"),
        EventBuilder::new("release")
            .qualified_object("binary", "artifact", "output")
            .payload_str("v1.0.0")
            .build(&mut counter)
            .expect("build release event"),
    ];

    for ev in events {
        asm.append(ev).expect("append event");
    }

    let receipt = asm.finalize();
    let verdict = verify(&receipt);
    eprintln!("Events: {}", receipt.events.len());
    eprintln!("Verdict: {} — {}", if verdict.accepted { "ACCEPT" } else { "REJECT" }, verdict.reason);
    assert!(verdict.accepted, "EventBuilder receipt must be accepted");
}
