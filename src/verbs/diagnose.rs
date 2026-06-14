// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
// Same shape ggen would render: thin #[verb], delegates to crate::handlers.
//! `receipt diagnose` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Render verify outcomes as LSP-shaped diagnostics (lsp-max)
///
/// ARDPRD: §6 (stdout hazard), §9 (witnessed LSP surface via lsp-max)
#[verb("diagnose", "receipt")]
pub fn diagnose(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::diagnose(receipt)
}
