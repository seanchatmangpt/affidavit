//! Example: the fiat-admission defense — the only honest path to `AdmittedReceipt`.
//!
//! Demonstrates `affidavit::admission::admit` end-to-end: an HONEST receipt
//! (events with object links, contiguous seq from 0) is admitted (Ok), while a
//! FORGED receipt (chain-consistent but seq starting at 5 — a continuity
//! violation) is REFUSED by name with `AffidavitRefusal::StructuralLawViolation`.
//! The refusal firing IS the capability: a receipt that did not pass the court
//! has no path to the `Admitted` type.
//!
//! See the doc on `admit` / `AffidavitRefusal` in src/admission.rs.

use affidavit::admission::{admit, AffidavitRefusal};
use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{AdmittedReceipt, Blake3Hash, OperationEvent, Receipt};

fn main() {
    // --- HONEST receipt: events with object links, contiguous seq from 0. ---
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for activity in ["create", "transform", "release"] {
        let event = build_event(
            activity,
            vec![object_ref("file-1", "artifact")],
            activity.as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(event).expect("append event");
    }
    let honest: Receipt = asm.finalize();

    let admitted: Result<AdmittedReceipt, AffidavitRefusal> = admit(honest.clone());
    assert!(
        admitted.is_ok(),
        "honest receipt MUST be admitted by both courts, got {:?}",
        admitted.err()
    );
    // The admitted carrier holds the very receipt that passed the courts.
    assert_eq!(
        admitted.unwrap().into_inner(),
        honest,
        "the AdmittedReceipt must carry the same receipt that was admitted"
    );

    // --- FORGED receipt: chain-consistent but seq starts at 5 (continuity
    // violation). Built via the deserialize path exactly as an attacker's file
    // would be — Receipt::sealed is crate-private. The chain_hash is recomputed
    // to MATCH, so this survives deserialization: it is a structural-law forgery,
    // not a chain-hash mismatch. ---
    let forged_event = OperationEvent {
        id: "evt-5".to_string(),
        seq: 5, // illegal: continuity requires contiguous-from-0
        event_type: "create".to_string(),
        objects: vec![object_ref("file-1", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"content"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&forged_event)).expect("recompute chain");
    let forged: Receipt = serde_json::from_value(serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [forged_event],
        "chain_hash": chain_hash,
    }))
    .expect("chain-consistent forged receipt deserializes");

    // The capability: admit() must REFUSE the forgery by name. If this did not
    // fire, the gate would be a fiat cast and this example would fail.
    let result = admit(forged);
    assert!(
        result.is_err(),
        "forged (continuity-violating) receipt MUST be refused — the gate is not a fiat cast"
    );
    match result.unwrap_err() {
        AffidavitRefusal::StructuralLawViolation { stage, reason } => {
            assert_eq!(
                stage, "continuity",
                "refusal must name the continuity stage that caught the forgery"
            );
            println!("forgery refused by name: structural_law_violation[{stage}]: {reason}");
        }
        other => panic!("expected continuity StructuralLawViolation, got {other:?}"),
    }

    println!("admission gate holds: honest admitted, forgery refused.");
}
