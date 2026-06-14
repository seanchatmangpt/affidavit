// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
// Same shape ggen would render: thin #[verb], delegates to crate::handlers.
//! `receipt model` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Discover a process model from a receipt's events (wasm4pm)
#[verb("model", "receipt")]
pub fn model(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::model(receipt)
}
