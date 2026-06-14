// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
//! `receipt graph` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Discover the directly-follows graph from a receipt (wasm4pm)
///
/// ARDPRD: §3 FR-2 (chain = graph of events), wasm4pm integration (§7 Phase 1)
#[verb("graph", "receipt")]
pub fn graph(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::graph(receipt)
}
