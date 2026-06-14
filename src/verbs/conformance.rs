// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
//! `receipt conformance` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compute fitness (wasm4pm token replay), activity_coverage (NOT van der Aalst precision), and simplicity (Occam)
#[verb("conformance", "receipt")]
pub fn conformance(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::conformance(receipt)
}
