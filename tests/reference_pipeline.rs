// THE WORKED WHOLE (COVERAGE.md §1/§6): the discover-then-conform collapse,
// witnessed as a real type-gated transition — NOT prose.
//
// Contribution claim (van der Aalst's terms): classical conformance is two
// artifacts — mine a model (discovery), then replay the log against it
// (conformance). Here the event log and the conformance certificate are the SAME
// object, because admission (the conformance gate) and the receipt (the log) are
// one type-enforced transition (Shape B). This file proves the fusion is real:
//
//   emit events → admit() [OCEL court + chain/continuity certify] → AdmittedReceipt
//                                                          │
//                              discover_from_admitted(&AdmittedReceipt) ──────────┘
//
// `discover_from_admitted` takes `&AdmittedReceipt`. The ONLY constructor of
// AdmittedReceipt is `admission::admit`, which runs both courts. So discovery is
// COMPILE-TIME GATED on admission: a receipt that did not pass conformance has no
// path to discovery. The receipt mined IS the certificate that proved conformance.
//
// Failing-when-fake: if admit() were a fiat cast (T-1), a forged receipt would
// reach AdmittedReceipt and then discovery — but the forged-receipt test below
// proves it is REFUSED and never yields an AdmittedReceipt to mine.

use affidavit::admission::admit;
use affidavit::chain::{recompute_chain, ChainAssembler, FORMAT_VERSION};
use affidavit::discovery::discover_from_admitted;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::{Blake3Hash, OperationEvent, Receipt};

fn honest_receipt(activities: &[&str]) -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for a in activities {
        let ev = build_event(
            *a,
            vec![object_ref("o", "artifact")],
            a.as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

#[test]
fn the_admitted_receipt_is_both_log_and_certificate() {
    // 1. Build + admit: the receipt passes the OCEL court AND the chain/continuity
    //    certify pipeline, yielding an AdmittedReceipt (the certificate).
    let receipt = honest_receipt(&["create", "transform", "release"]);
    let admitted = admit(receipt).expect("honest receipt is admitted by both courts");

    // 2. Discover FROM the admitted value: the SAME object that proved conformance
    //    is now mined as the event log. One artifact, both roles (Shape B).
    let model = discover_from_admitted(&admitted);

    // The discovered model reflects the receipt's events — discovery really ran on
    // the certificate, not on a placeholder.
    assert!(
        model.contains("create"),
        "model mined from the certificate names its activities: {model}"
    );
    assert!(
        model.contains("release"),
        "model mined from the certificate names its activities: {model}"
    );
}

#[test]
fn discovery_is_type_gated_on_admission_a_forgery_never_reaches_it() {
    // A chain-consistent but structurally-forged receipt (seq starts at 5).
    let forged_event = OperationEvent {
        id: "evt-5".to_string(),
        seq: 5, // continuity violation
        event_type: "create".to_string(),
        objects: vec![object_ref("o", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"x"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&forged_event)).expect("chain");
    // External crates cannot call Receipt::sealed (crate-private). Reach a Receipt
    // through the deserialize path, exactly as a real attacker's file would — the
    // chain_hash matches, so it survives deserialization but is structurally forged.
    let forged: Receipt = serde_json::from_value(serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [forged_event],
        "chain_hash": chain_hash,
    }))
    .expect("chain-consistent receipt deserializes");

    // admit() REFUSES it — so there is NO AdmittedReceipt to hand to discovery.
    // discover_from_admitted CANNOT be called on `forged`: it does not type-check
    // (forged is a Receipt, not an AdmittedReceipt). The refusal is the proof the
    // gate is real and not a fiat cast.
    let result = admit(forged);
    assert!(
        result.is_err(),
        "a forged receipt must be refused — there is no admitted value for discovery to consume"
    );
    // (Statically: `discover_from_admitted(&forged)` would be a compile error —
    // `&Receipt` is not `&AdmittedReceipt`. Admission is a type precondition.)
}
