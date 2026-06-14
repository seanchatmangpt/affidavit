//! Receipt diagnostics for the LSP surface — the genuine `lsp-max` integration.
//!
//! lsp-max's documented purpose includes "receipt-chain admission" and reporting
//! stage outcomes as editor diagnostics. This module realizes that integration
//! point: it maps an affidavit verify [`Verdict`] into LSP [`Diagnostic`]s, so an
//! editor driven by an lsp-max server can render each failing certify stage as a
//! red squiggle with the stage's name and detail.
//!
//! Genuine (not a stub): the returned values are `lsp_max::lsp_types::Diagnostic`
//! — remove the `lsp-max` dependency and this module does not compile
//! (failing-when-fake on the integration axis). Feed it a verdict whose stages
//! failed and the diagnostics must carry those stages (failing-when-fake on the
//! capability axis).

use crate::types::Verdict;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// The diagnostic `source` string affidavit stamps on every receipt diagnostic.
pub const DIAGNOSTIC_SOURCE: &str = "affidavit";

/// Map a verify [`Verdict`] into LSP diagnostics: one diagnostic per FAILING
/// stage (Error severity), plus — when the receipt is rejected — a summary
/// diagnostic carrying the verdict reason. An accepted verdict yields no
/// diagnostics (a clean receipt produces no squiggles), which is itself the
/// editor convention for "no problems."
///
/// # Example: see `examples/verdict_diagnostics.rs` (run: `cargo run --example verdict_diagnostics`).
/// It drives a REAL reject verdict (forged seq=5 receipt) through the verifier
/// and asserts the resulting [`DIAGNOSTIC_SOURCE`] Error diagnostic names the
/// failing `continuity` stage, then asserts an accepted verdict yields none.
pub fn verdict_to_diagnostics(verdict: &Verdict) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, outcome) in verdict.outcomes.iter().enumerate() {
        if !outcome.passed {
            // Anchor the diagnostic on the stage's ordinal line — a stable,
            // deterministic position (receipts have no source spans; the stage
            // index is the canonical anchor).
            let line = index as u32;
            let range = Range::new(Position::new(line, 0), Position::new(line, 1));
            let diag = Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some(DIAGNOSTIC_SOURCE.to_string()),
                message: format!("{}: {}", outcome.stage, outcome.detail),
                code: None,
                code_description: None,
                related_information: None,
                tags: None,
                data: None,
            };
            diagnostics.push(diag);
        }
    }

    if !verdict.accepted {
        let range = Range::new(Position::new(0, 0), Position::new(0, 1));
        let summary = Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some(DIAGNOSTIC_SOURCE.to_string()),
            message: format!("REJECT — {}", verdict.reason),
            code: None,
            code_description: None,
            related_information: None,
            tags: None,
            data: None,
        };
        diagnostics.push(summary);
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CheckOutcome, ProfileId};

    fn outcome(stage: &str, passed: bool) -> CheckOutcome {
        CheckOutcome {
            stage: stage.to_string(),
            passed,
            detail: format!("{stage} detail"),
        }
    }

    #[test]
    fn accepted_verdict_produces_no_diagnostics() {
        let verdict = Verdict {
            accepted: true,
            profile: ProfileId::CoreV1,
            outcomes: vec![outcome("decode", true), outcome("continuity", true)],
            reason: "all stages passed".to_string(),
        };
        assert!(
            verdict_to_diagnostics(&verdict).is_empty(),
            "a clean receipt yields no editor squiggles"
        );
    }

    #[test]
    fn failing_stage_becomes_an_error_diagnostic_naming_the_stage() {
        let verdict = Verdict {
            accepted: false,
            profile: ProfileId::CoreV1,
            outcomes: vec![outcome("decode", true), outcome("continuity", false)],
            reason: "continuity: seq gap".to_string(),
        };
        let diags = verdict_to_diagnostics(&verdict);

        // One stage diagnostic + one summary diagnostic.
        assert_eq!(diags.len(), 2, "one failing stage + one reject summary");
        // The stage diagnostic names the failing stage and is an Error.
        let stage_diag = &diags[0];
        assert_eq!(stage_diag.severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(stage_diag.source.as_deref(), Some(DIAGNOSTIC_SOURCE));
        assert!(
            stage_diag.message.contains("continuity"),
            "diagnostic must name the failing stage; got {}",
            stage_diag.message
        );
        // The summary carries the verdict reason.
        assert!(
            diags[1].message.contains("REJECT"),
            "summary diagnostic must mark the rejection"
        );
    }
}
