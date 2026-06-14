// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
// Same shape ggen would render: thin #[verb], delegates to crate::handlers.
//! `receipt inspect` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Detailed structural analysis of a receipt (event/object distribution)
///
/// ARDPRD: FR-4 (Inspection), §9 (witnessed surface — failing-when-fake)
#[verb("inspect", "receipt")]
pub fn inspect(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::inspect(receipt)
}
