// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
//! `receipt stats` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// One-shot aggregate stats for a receipt (counts + DFG size + conformance)
#[verb("stats", "receipt")]
pub fn stats(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::stats(receipt)
}
