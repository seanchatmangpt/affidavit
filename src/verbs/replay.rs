// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
//! `receipt replay` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Replay a receipt's event sequence step by step in lawful seq order
///
/// ARDPRD: FR-1 (Receipt emission), §7 Phase 1 (event trace as provenance record)
#[verb("replay", "receipt")]
pub fn replay(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::replay(receipt)
}
