// Hand-authored verb wrapper (DX/QOL capability surface).
//! `receipt help-refs` verb — ARDPRD cross-reference map.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Print ARDPRD cross-reference map for all verbs
///
/// ARDPRD: §9 (Acceptance — what "witnessed" means for this spec)
#[verb("help-refs", "receipt")]
pub fn help_refs() -> Result<()> {
    crate::handlers::help_refs()
}
