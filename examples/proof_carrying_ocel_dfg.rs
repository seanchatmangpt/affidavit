//! Proof-carrying OCEL DFG conformance — mint a seal/v2 subject from grounded
//! fitness/precision, then show the temper falsifier that moves the digest.
//!
//! Pipeline:
//!   receipt → fixture OCEL → `admit_flat` DFG court → BLAKE3 subject digest
//!
//! The subject is the 32-byte message `M` a consumer seals under FIPS 204
//! context `affidavit-core/seal/v2` (see `proof_carrying_ocel_seal` for the
//! signing round-trip under `--features crypto-trust`).
//!
//! Run: `cargo run --example proof_carrying_ocel_dfg`

use affidavit::admission::admit;
use affidavit::chain::ChainAssembler;
use affidavit::mining::ocel_flatten_loss_court::fixture_ocel;
use affidavit::mining::ocel_proof_carrying::{
    proof_carrying_dfg_conformance, DOMAIN_OCEL_CONFORMANCE_SUBJECT,
};
use affidavit::ocel::{build_event, object_ref, SeqCounter};

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

fn main() {
    println!("=== proof-carrying OCEL DFG subject ===");
    println!(
        "subject domain: {}",
        std::str::from_utf8(DOMAIN_OCEL_CONFORMANCE_SUBJECT).unwrap()
    );
    println!("seal/v2 context (consumer): affidavit-core/seal/v2");
    println!();

    let ocel = order_log(&["create", "pay", "ship"]);
    let normative = vec![
        ("create".into(), "pay".into()),
        ("pay".into(), "ship".into()),
    ];

    let pc = proof_carrying_dfg_conformance(&ocel, "order", "log:demo", &normative)
        .expect("grounded claim must mint a subject");
    assert_ne!(pc.subject_digest, [0u8; 32], "degenerate subject refused");

    println!(
        "honest: fitness={:.3} precision={:.3}",
        pc.scores.fitness, pc.scores.precision
    );
    println!("subject_digest = {}", hex32(&pc.subject_digest));
    println!("  (pass this 32-byte M to sign_deterministic under seal/v2 ctx)");
    println!();

    // Temper: rewrite the normative model so fitness/precision move.
    let tempered = vec![
        ("create".into(), "pay".into()),
        ("pay".into(), "CANCEL".into()),
    ];
    let pc2 = proof_carrying_dfg_conformance(&ocel, "order", "log:demo", &tempered)
        .expect("still grounded, but scores change");
    println!(
        "tempered model: fitness={:.3} precision={:.3}",
        pc2.scores.fitness, pc2.scores.precision
    );
    println!("subject_digest = {}", hex32(&pc2.subject_digest));
    assert_ne!(
        pc.subject_digest, pc2.subject_digest,
        "COURT-VACUITY: temper must move the seal subject"
    );
    println!("falsifier: temper moved the subject digest ✓");
    println!();

    // Ungrounded claim never yields a seal subject.
    let err = proof_carrying_dfg_conformance(&ocel, "order", "", &normative)
        .expect_err("empty log_ref must refuse before mint");
    println!("ungrounded refusal: {err}");
    assert!(err
        .to_string()
        .contains("REFUSED_OCEL_CONFORMANCE_UNGROUNDED"));
    println!("ungrounded claim never mints a subject ✓");
}
