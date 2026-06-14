// Reference witness: the Evidence typestate LIFECYCLE transitions are exercised
// (COVERAGE.md §2.1 — closes the intermediate-states row Parsed/Projected/
// Exportable/Receipted).
//
// The lifecycle is Raw ──parse──▶ Parsed ──admit──▶ Admitted ──▶ {Projected |
// Exportable | Receipted}. Each transition is a distinct type; an illegal one
// does not compile. This witnesses:
//   - Raw → Parsed (into_parsed), value recoverable
//   - Admitted → Projected / Exportable / Receipted (post-admission transitions)
// using a real AdmittedReceipt minted by admission::admit (the only path).

use affidavit::admission::admit;
use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::Receipt;
use wasm4pm_compat::evidence::Evidence;
use wasm4pm_compat::state::Raw;
use wasm4pm_compat::witness::Ocel20;

fn honest_receipt() -> Receipt {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    let ev = build_event("create", vec![object_ref("o", "artifact")], b"x", &mut counter)
        .expect("event");
    asm.append(ev).expect("append");
    asm.finalize()
}

#[test]
fn raw_evidence_parses_and_value_is_recoverable() {
    // Raw is the only freely-available constructor; into_parsed advances the state.
    let raw: Evidence<String, Raw, Ocel20> = Evidence::raw("boundary-input".to_string());
    let parsed = raw.into_parsed();
    assert_eq!(parsed.value, "boundary-input", "value survives Raw → Parsed");
}

#[test]
fn admitted_receipt_transitions_to_projected_exportable_receipted() {
    // Each post-admission transition consumes a fresh AdmittedReceipt (the only
    // constructor of Admitted is admission::admit, which runs both courts).
    let projected = admit(honest_receipt()).expect("admitted").into_projected();
    // into_inner recovers the receipt after projection.
    assert_eq!(projected.value.events.len(), 1, "Admitted → Projected, value intact");

    let exportable = admit(honest_receipt()).expect("admitted").into_exportable();
    assert_eq!(exportable.value.events.len(), 1, "Admitted → Exportable, value intact");

    let receipted = admit(honest_receipt()).expect("admitted").into_receipted();
    assert_eq!(receipted.value.events.len(), 1, "Admitted → Receipted, value intact");
}
