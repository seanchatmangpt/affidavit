//! Capability: verdict -> editor diagnostics (the lsp-max integration).
//!
//! End-to-end witness, not a smoke test: a structurally forged receipt (seq
//! starts at 5 — a continuity violation) is run through the REAL
//! `affidavit::verifier::verify`, producing a genuine rejected `Verdict`, which
//! is then rendered by `affidavit::lsp::verdict_to_diagnostics` into LSP
//! diagnostics. We assert there is >=1 Error diagnostic from affidavit naming
//! the failing `continuity` stage. Then a clean accepted verdict is asserted to
//! yield EMPTY diagnostics (no squiggles).
//!
//! Failing-when-fake: if the verifier stopped catching the continuity gap the
//! verdict would be accepted and there would be no Error diagnostic to assert.
//!
//! See the doc on `verdict_to_diagnostics` / `DIAGNOSTIC_SOURCE` in src/lsp.rs.
//! Modeled on tests/reference_lsp_real_reject.rs.

use affidavit::chain::{recompute_chain, FORMAT_VERSION};
use affidavit::lsp::{verdict_to_diagnostics, DIAGNOSTIC_SOURCE};
use affidavit::ocel::object_ref;
use affidavit::types::{
    Blake3Hash, CheckOutcome, OperationEvent, ProfileId, Receipt, Verdict,
};
use lsp_types::DiagnosticSeverity;

fn main() {
    // ---- 1. REAL reject path: forged (seq=5) receipt -> verifier -> diagnostics.
    let forged_event = OperationEvent {
        id: "evt-5".to_string(),
        seq: 5, // continuity violation
        event_type: "create".to_string(),
        objects: vec![object_ref("o", "artifact")],
        payload_commitment: Blake3Hash::from_bytes(b"x"),
    };
    let chain_hash = recompute_chain(std::slice::from_ref(&forged_event)).expect("chain");
    let forged: Receipt = serde_json::from_value(serde_json::json!({
        "format_version": FORMAT_VERSION,
        "events": [forged_event],
        "chain_hash": chain_hash,
    }))
    .expect("chain-consistent receipt deserializes");

    let verdict = affidavit::verifier::verify(&forged);
    assert!(
        !verdict.accepted,
        "the forged receipt must be genuinely rejected by the verifier"
    );

    let diags = verdict_to_diagnostics(&verdict);
    assert!(
        !diags.is_empty(),
        "a rejected verdict must surface >=1 diagnostic"
    );

    let continuity_err = diags.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.source.as_deref() == Some(DIAGNOSTIC_SOURCE)
            && d.message.contains("continuity")
    });
    assert!(
        continuity_err.is_some(),
        "a real continuity refusal must become an Error diagnostic from '{DIAGNOSTIC_SOURCE}' \
         naming the failing stage; got {diags:?}"
    );
    println!(
        "reject verdict -> {} diagnostic(s); continuity Error: {}",
        diags.len(),
        continuity_err.unwrap().message
    );

    // ---- 2. Clean accepted verdict -> EMPTY diagnostics (no squiggles).
    let clean = Verdict {
        accepted: true,
        profile: ProfileId::CoreV1,
        outcomes: vec![
            CheckOutcome {
                stage: "decode".to_string(),
                passed: true,
                detail: "ok".to_string(),
            },
            CheckOutcome {
                stage: "continuity".to_string(),
                passed: true,
                detail: "ok".to_string(),
            },
        ],
        reason: "all stages passed".to_string(),
    };
    let clean_diags = verdict_to_diagnostics(&clean);
    assert!(
        clean_diags.is_empty(),
        "an accepted verdict must yield NO diagnostics; got {clean_diags:?}"
    );
    println!("accepted verdict -> 0 diagnostics (clean editor)");

    println!("OK: verdict -> editor diagnostics covered end-to-end");
}
