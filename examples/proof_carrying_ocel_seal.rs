//! Proof-carrying seal/v2 round-trip over an OCEL DFG subject.
//!
//! 1. Mint the BLAKE3 conformance subject (row 10 + proof-carrying module).
//! 2. Sign `M = subject` under FIPS 204 context `affidavit-core/seal/v2`.
//! 3. Verify ACCEPT; empty-ctx verify must REFUSE (context-string court).
//! 4. Temper the normative model → new subject → old signature must not verify.
//!
//! Run: `cargo run --features crypto-trust --example proof_carrying_ocel_seal`

use affidavit::admission::admit;
use affidavit::chain::ChainAssembler;
use affidavit::crypto_trust_pqc::ml_dsa65_from_seed;
use affidavit::deontic::sign_with_context_raw;
use affidavit::mining::ocel_flatten_loss_court::fixture_ocel;
use affidavit::mining::ocel_proof_carrying::proof_carrying_dfg_conformance;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use ml_dsa::{EncodedVerifyingKey, MlDsa65, Signature, VerifyingKey};

/// Exact byte string from `affidavit_core::sealed::DOMAIN_SEAL` (seal/v2).
const SEAL_V2_CTX: &[u8] = b"affidavit-core/seal/v2";

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

fn verify_with_ctx(subject: &[u8; 32], ctx: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    let Ok(encoded) = EncodedVerifyingKey::<MlDsa65>::try_from(pk) else {
        return false;
    };
    let vk = VerifyingKey::<MlDsa65>::decode(&encoded);
    let Ok(parsed) = Signature::<MlDsa65>::try_from(sig) else {
        return false;
    };
    vk.verify_with_context(subject, ctx, &parsed)
}

fn main() {
    println!("=== proof-carrying OCEL subject → seal/v2 ===\n");
    println!("ctx = {:?}", std::str::from_utf8(SEAL_V2_CTX).unwrap());

    let ocel = order_log(&["create", "pay", "ship"]);
    let normative = vec![
        ("create".into(), "pay".into()),
        ("pay".into(), "ship".into()),
    ];
    let pc = proof_carrying_dfg_conformance(&ocel, "order", "log:seal-demo", &normative)
        .expect("mint subject");
    println!("subject = {}", hex32(&pc.subject_digest));
    println!(
        "scores: fitness={:.3} precision={:.3}",
        pc.scores.fitness, pc.scores.precision
    );

    let seed = [0xA5u8; 32];
    let kp = ml_dsa65_from_seed(&seed);
    let sig = sign_with_context_raw(&seed, &pc.subject_digest, SEAL_V2_CTX)
        .expect("sign under seal/v2 ctx");

    assert!(
        verify_with_ctx(&pc.subject_digest, SEAL_V2_CTX, &sig, &kp.public),
        "honest seal/v2 must verify"
    );
    println!("seal/v2 verify ACCEPT ✓");

    assert!(
        !verify_with_ctx(&pc.subject_digest, &[], &sig, &kp.public),
        "COURT-VACUITY: ctx-bound sig must fail empty-ctx verify"
    );
    println!("empty-ctx verify REFUSE ✓ (context-string court)");

    let tempered_model = vec![
        ("create".into(), "pay".into()),
        ("pay".into(), "CANCEL".into()),
    ];
    let pc2 = proof_carrying_dfg_conformance(&ocel, "order", "log:seal-demo", &tempered_model)
        .expect("temper still grounded");
    assert_ne!(pc.subject_digest, pc2.subject_digest);
    assert!(
        !verify_with_ctx(&pc2.subject_digest, SEAL_V2_CTX, &sig, &kp.public),
        "COURT-VACUITY: old seal must not verify over tempered subject"
    );
    println!(
        "tempered subject = {} — old seal does not verify ✓",
        hex32(&pc2.subject_digest)
    );
}
