//! Proof-carrying DECLARE gate — behavioural templates must ACCEPT before a
//! DFG seal subject is minted. A temper that violates Response / Precedence /
//! NotCoExistence refuses typed and never reaches the subject digester.
//!
//! Run: `cargo run --example proof_carrying_declare`

use affidavit::admission::admit;
use affidavit::chain::ChainAssembler;
use affidavit::mining::ocel_declare_court::{admit_declare, REFUSED_DECLARE_VIOLATION};
use affidavit::mining::ocel_flatten_loss_court::fixture_ocel;
use affidavit::mining::ocel_proof_carrying::proof_carrying_dfg_conformance;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use wasm4pm_compat::declare::DeclareTemplate;

fn order_log(acts: &[&str]) -> wasm4pm_compat::ocel::OCEL {
    let mut asm = ChainAssembler::new();
    let mut counter = SeqCounter::new();
    for act in acts {
        asm.append(
            build_event(
                *act,
                vec![object_ref("o1", "order")],
                act.as_bytes(),
                &mut counter,
            )
            .expect("event"),
        )
        .expect("append");
    }
    let admitted = admit(asm.finalize()).expect("admit");
    fixture_ocel(&admitted.value).expect("fixture ocel")
}

fn hex32(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Gate: every DECLARE constraint ACCEPTS, then mint the DFG subject.
fn proof_carrying_after_declare(
    ocel: &wasm4pm_compat::ocel::OCEL,
    constraints: &[(DeclareTemplate, &str, Option<&str>)],
    normative: &[(String, String)],
) -> Result<[u8; 32], String> {
    for (tpl, act, tgt) in constraints {
        admit_declare(ocel, "o1", *tpl, act, *tgt, "order").map_err(|e| e.to_string())?;
    }
    let pc = proof_carrying_dfg_conformance(ocel, "order", "log:declare-demo", normative)
        .map_err(|e| e.to_string())?;
    Ok(pc.subject_digest)
}

fn main() {
    println!("=== proof-carrying DECLARE gate → DFG subject ===\n");

    let honest = order_log(&["create", "approve", "ship", "deliver"]);
    let constraints = [
        (DeclareTemplate::Init, "create", None),
        (DeclareTemplate::Response, "create", Some("approve")),
        (DeclareTemplate::Precedence, "ship", Some("deliver")),
        (DeclareTemplate::Succession, "ship", Some("deliver")),
        (DeclareTemplate::Absence, "CANCEL", None),
        (DeclareTemplate::NotCoExistence, "create", Some("CANCEL")),
    ];
    let normative = vec![
        ("create".into(), "approve".into()),
        ("approve".into(), "ship".into()),
        ("ship".into(), "deliver".into()),
    ];

    let subject = proof_carrying_after_declare(&honest, &constraints, &normative)
        .expect("satisfying log must mint");
    println!("DECLARE ACCEPT × {} templates", constraints.len());
    println!("subject_digest = {}", hex32(&subject));
    println!();

    // Temper: drop approve → Response refuses; subject never minted.
    let tempered = order_log(&["create", "ship", "deliver"]);
    let err = proof_carrying_after_declare(&tempered, &constraints, &normative)
        .expect_err("missing approve must refuse at DECLARE gate");
    println!("temper (no approve): {err}");
    assert!(
        err.contains(REFUSED_DECLARE_VIOLATION),
        "must be DECLARE violation, not a silent subject"
    );
    println!("falsifier: DECLARE temper blocks subject mint ✓");
}
