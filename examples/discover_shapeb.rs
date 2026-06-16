//! Shape-B discover-then-conform collapse: the receipt is BOTH the event log AND
//! the conformance certificate. This exercises the admission-gated discovery path
//! end-to-end: build an honest receipt, admit it through both courts, then mine the
//! *certificate* (the `AdmittedReceipt`) for its process model and quality metrics.
//!
//! The load-bearing edge is the TYPE-GATE: `discover_from_admitted` and
//! `quality_metrics_from_admitted` take `&AdmittedReceipt`, which can only be minted
//! by `affidavit::admission::admit`. A raw, un-adjudicated `Receipt` cannot reach
//! this path — discovery consumes the certificate, not arbitrary bytes. This example
//! demonstrates that gate is live (an un-admitted forged receipt is REFUSED, so no
//! `AdmittedReceipt` exists to mine).
//!
//! See the doc on `discover_from_admitted` / `quality_metrics_from_admitted` in
//! src/discovery.rs (run: `cargo run --example discover_shapeb`).

use affidavit::admission::admit;
use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::discovery::{
    conformance_metrics, discover_from_admitted, discover_process_tree, project_to_event_log,
    quality_metrics_from_admitted,
};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, OperationEvent, Receipt};

fn honest_receipt(activities: &[&str]) -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for (i, act) in activities.iter().enumerate() {
        let ev = build_event(
            *act,
            vec![object_ref(&format!("obj-{i}"), "artifact")],
            act.as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

fn main() {
    // 1. Build an honest receipt with the three canonical activities.
    let activities = ["create", "transform", "release"];
    let receipt = honest_receipt(&activities);

    // 2. Admit it: BOTH courts (OCEL structural law + chain/continuity certify)
    //    must accept before an AdmittedReceipt is minted.
    let admitted = admit(receipt).expect("an honest receipt must be admittable");

    // 3. Discover the process model FROM the admitted certificate. The discovery
    //    really ran on the certificate: the activity names must surface in the tree.
    let tree = discover_from_admitted(&admitted);
    println!("discovered process tree: {tree}");
    for act in &activities {
        assert!(
            tree.contains(act),
            "discovery on the certificate must surface activity '{act}'; got: {tree}"
        );
    }

    // 3b. discover_from_admitted mines the very value that proved conformance:
    //     mining the inner certificate value directly yields the same activity set
    //     (child order is not stable across discovery runs, so we check membership).
    let direct = discover_process_tree(&admitted.value);
    for act in &activities {
        assert!(
            direct.contains(act),
            "mining the inner certificate value must surface '{act}'; got: {direct}"
        );
    }
    assert_eq!(
        project_to_event_log(&admitted.value).traces[0].events.len(),
        activities.len(),
        "the projected event log must carry one event per receipt event"
    );

    // 4. Admission-gated quality metrics: fitness must be a real, finite,
    //    in-range token-replay number. A log replayed on its own discovered net
    //    fits well.
    let (fitness, activity_coverage, simplicity) = quality_metrics_from_admitted(&admitted);
    println!("fitness={fitness} activity_coverage={activity_coverage} simplicity={simplicity}");
    assert!(fitness.is_finite(), "fitness must be finite; got {fitness}");
    assert!(
        (0.0..=1.0).contains(&fitness),
        "fitness must be in [0,1]; got {fitness}"
    );
    assert!(
        fitness > 0.5,
        "a log replayed on its own discovered net should fit well; got {fitness}"
    );
    assert!(
        (0.0..=1.0).contains(&activity_coverage) && activity_coverage.is_finite(),
        "activity_coverage must be a finite ratio in [0,1]; got {activity_coverage}"
    );
    assert!(
        (0.0..=1.0).contains(&simplicity) && simplicity.is_finite(),
        "simplicity must be a finite number in [0,1]; got {simplicity}"
    );

    // 4b. conformance_metrics is the 2-tuple (fitness, activity_coverage) facet of
    //     the same computation — its fitness must agree with quality_metrics' fitness
    //     (both derive from the same ILP-discovered net on the same receipt).
    let (cm_fitness, cm_coverage) = conformance_metrics(&admitted.value);
    assert_eq!(
        cm_fitness, fitness,
        "conformance_metrics fitness must equal quality_metrics fitness (same receipt, same net)"
    );
    assert_eq!(
        cm_coverage, activity_coverage,
        "activity_coverage must agree across both entry points"
    );

    // 5. THE TYPE-GATE EDGE the capability exists for: a forged receipt (seq starts
    //    at 5, violating continuity) is REFUSED by admission, so no AdmittedReceipt
    //    is produced — discover_from_admitted/quality_metrics_from_admitted CANNOT
    //    be reached for it. We assert the refusal, proving the gate is live.
    let forged_event = OperationEvent {
        id: "evt-5".to_string(),
        seq: 5, // illegal: continuity requires contiguous-from-0
        event_type: "create".to_string(),
        objects: vec![object_ref("file-1", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"content"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&forged_event)).expect("recompute chain");
    // Receipts can only be minted via the sealed seam (ChainAssembler) or
    // canonical deserialization (which re-verifies the chain hash). We build the
    // forgery through deserialization with a *matching* hash, so the refusal is a
    // structural-continuity violation, not a chain-hash mismatch.
    let forged_json = serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [forged_event],
        "chain_hash": chain_hash.as_hex(),
    });
    let forged: Receipt =
        serde_json::from_value(forged_json).expect("forged receipt has a matching chain hash");

    let refusal = admit(forged);
    assert!(
        refusal.is_err(),
        "forged (continuity-violating) receipt MUST be refused — it can never become an \
         AdmittedReceipt, so discovery cannot be called on it"
    );
    println!(
        "forged receipt refused as expected: {}",
        refusal.unwrap_err()
    );

    println!("OK: Shape-B discover-then-conform collapse demonstrated end-to-end.");
}
