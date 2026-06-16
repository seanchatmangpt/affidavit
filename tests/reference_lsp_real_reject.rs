// Reference witness: the FAILING editor-diagnostics path, driven by a REAL reject
// verdict (van der Aalst panel B3 fix). COVERAGE.md §5 — receipt verdict → editor
// diagnostics.
//
// The prior failing-path witness fed verdict_to_diagnostics a hand-built Verdict
// literal (accepted:false). That is hollow for the cross-product claim: it never
// shows a genuine refusal flowing from the verifier into the LSP surface. Here a
// structurally-forged receipt (seq starts at 5 — a continuity violation) is run
// through the REAL affidavit::verifier::verify, producing a genuine rejected
// Verdict, which is then rendered by affidavit::lsp::verdict_to_diagnostics.
// Failing-when-fake: if the verifier stopped catching the continuity gap, the
// verdict would be accepted and there would be no Error diagnostic to assert.

use affidavit::chain::{recompute_chain, FORMAT_VERSION};
use affidavit::lsp::{verdict_to_diagnostics, DIAGNOSTIC_SOURCE};
use affidavit::ocel::object_ref;
use affidavit::types::{Blake3Hash, OperationEvent, Receipt};
use lsp_max::lsp_types::DiagnosticSeverity;

#[test]
fn a_real_continuity_refusal_flows_into_error_diagnostics() {
    // A chain-consistent but structurally forged receipt: seq starts at 5.
    let forged_event = OperationEvent {
        id: "evt-5".to_string(),
        seq: 5, // continuity violation — stage_continuity will fail
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

    // REAL verifier — not a hand-built verdict.
    let verdict = affidavit::verifier::verify(&forged);
    assert!(
        !verdict.accepted,
        "the forged receipt is genuinely rejected by the verifier"
    );

    // The genuine reject verdict renders as editor diagnostics.
    let diags = verdict_to_diagnostics(&verdict);
    assert!(
        !diags.is_empty(),
        "a rejected verdict must surface diagnostics"
    );

    // At least one Error diagnostic from affidavit names the failing continuity stage.
    let continuity_err = diags.iter().find(|d| {
        d.severity == Some(DiagnosticSeverity::ERROR)
            && d.source.as_deref() == Some(DIAGNOSTIC_SOURCE)
            && d.message.contains("continuity")
    });
    assert!(
        continuity_err.is_some(),
        "a real continuity refusal must become an Error diagnostic naming the stage; got {diags:?}"
    );
}
