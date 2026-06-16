//! verdict_to_diagnostics() — moved from src/lsp.rs.
//! Fulfills the core "Diagnostics -> real-time tamper/forgery detection" requirement.

use lsp_max::lsp_types_max::{Diagnostic, DiagnosticSeverity, Position, Range};

/// The source string used for affidavit diagnostics.
pub const DIAGNOSTIC_SOURCE: &str = "affidavit";

/// Converts a [`crate::types::Verdict`] into a list of LSP [`Diagnostic`]s.
pub fn verdict_to_diagnostics(verdict: &crate::types::Verdict) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for outcome in &verdict.outcomes {
        if !outcome.passed {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 10,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some(DIAGNOSTIC_SOURCE.to_string()),
                message: format!("{}: {}", outcome.stage, outcome.detail),
                related_information: None,
                tags: None,
                data: None,
            });
        }
    }

    diagnostics
}
