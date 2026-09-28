//! Multi-object receipt: events linking multiple objects (files, agents, resources).
//!
//! Real provenance often involves multiple objects per event — a build step
//! consumes source files and produces an artifact. This shows how to build
//! receipts with multi-object events and qualified references, then verify the
//! full chain is accepted.

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, qualified_object_ref, SeqCounter};
use affidavit::verifier::verify;

fn main() {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();

    // Event 1: compile step — consumes multiple source files, produces binary.
    let compile_ev = build_event(
        "compile",
        vec![
            qualified_object_ref("main.rs", "source_file", "input"),
            qualified_object_ref("lib.rs", "source_file", "input"),
            qualified_object_ref("Cargo.toml", "manifest", "input"),
            qualified_object_ref("target/release/app", "binary", "output"),
        ],
        b"rustc --release",
        &mut counter,
    )
    .expect("compile event");
    asm.append(compile_ev).expect("append compile");

    // Event 2: test step — binary + test harness → results.
    let test_ev = build_event(
        "test",
        vec![
            object_ref("target/release/app", "binary"),
            object_ref("tests/", "test_suite"),
            qualified_object_ref("test_results.xml", "report", "output"),
        ],
        b"cargo test -- --format junit",
        &mut counter,
    )
    .expect("test event");
    asm.append(test_ev).expect("append test");

    // Event 3: publish — binary, signature, registry entry.
    let publish_ev = build_event(
        "publish",
        vec![
            object_ref("target/release/app", "binary"),
            qualified_object_ref("app.sig", "signature", "attestation"),
            qualified_object_ref("registry.io/org/app:1.0", "package", "output"),
        ],
        b"cargo publish",
        &mut counter,
    )
    .expect("publish event");
    asm.append(publish_ev).expect("append publish");

    let receipt = asm.finalize();
    println!("receipt: {} events", receipt.events.len());
    for ev in &receipt.events {
        println!(
            "  [seq {}] {} — {} object(s)",
            ev.seq,
            ev.event_type,
            ev.objects.len()
        );
        for obj in &ev.objects {
            let qual = obj
                .qualifier
                .as_deref()
                .map(|q| format!(" ({})", q))
                .unwrap_or_default();
            println!("       {}:{}{}", obj.id, obj.obj_type, qual);
        }
    }

    // Verify: multi-object receipts must pass all stages.
    let verdict = verify(&receipt);
    assert!(
        verdict.accepted,
        "multi-object receipt must accept: {}",
        verdict.reason
    );
    println!("\nverdict: ACCEPTED — {}", verdict.reason);

    let total_refs: usize = receipt.events.iter().map(|e| e.objects.len()).sum();
    println!("total object references: {total_refs}");
    println!("OK: multi-object events with qualified refs are fully supported");
}
