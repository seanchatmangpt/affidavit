// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
//! `receipt mutate` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// `affi receipt mutate` — show the effect of tampering with a receipt field.
///
/// Mutates the first event's type to "tampered" and shows the resulting
/// chain hash divergence. Demonstrates tamper-evidence: the mutated receipt
/// would REJECT on verify. (ARDPRD §2, NFR-2)
#[verb("mutate", "receipt")]
pub fn mutate(
    #[arg(index = 1)]
    receipt: String,
) -> Result<()> {
    crate::handlers::mutate(receipt)
}
