//! Conformance report example: emit → assemble → admit → quality_metrics.
//!
//! Demonstrates the full discover-then-conform pipeline:
//! 1. Build a 4-event receipt (create, transform, validate, release)
//! 2. Run it through the admission gate (admit())
//! 3. Compute fitness (token replay) + activity_coverage + simplicity (Occam)
//!
//! The admitted receipt is the ONLY path to quality_metrics_from_admitted —
//! the type gate proves discovery is gated on passing the OCEL court.

use affidavit::admission::admit;
use affidavit::chain::ChainAssembler;
use affidavit::discovery::quality_metrics_from_admitted;
use affidavit::ocel::{build_event, object_ref, SeqCounter};

fn main() {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let activities = ["create", "transform", "validate", "release"];
    for act in &activities {
        let ev = build_event(
            *act,
            vec![object_ref("artifact-1", "artifact")],
            act.as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    let receipt = asm.finalize();
    println!(
        "built receipt: {} events, chain_hash = {}...",
        receipt.events.len(),
        &receipt.chain_hash.as_hex()[..12]
    );

    let admitted = admit(receipt).expect("honest receipt must be admitted");
    println!("admitted: receipt passed OCEL court + chain verifier");

    let (fitness, activity_coverage, simplicity) = quality_metrics_from_admitted(&admitted);
    println!("conformance metrics:");
    println!("  fitness (token replay):    {fitness:.4}");
    println!("  activity_coverage:         {activity_coverage:.4}");
    println!("  simplicity (Occam):        {simplicity:.4}");

    // Invariant: fitness on a receipt-derived log is always 1.0 (every trace fits the model)
    assert!(
        fitness >= 0.99,
        "fitness from a valid receipt should be ~1.0 (perfect replay)"
    );
    assert!(activity_coverage > 0.0, "activity_coverage must be positive");
    assert!(
        simplicity > 0.0 && simplicity <= 1.0,
        "simplicity is in (0, 1]"
    );

    println!("OK: conformance pipeline produces real quality metrics from an admitted receipt");
}
