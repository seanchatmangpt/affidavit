// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper rendered from O* by ggen. The pack is authoritative for the
// CLI *interface* only; the body delegates to a stable consumer-implemented
// handler. There is NO logic slot here — business logic lives behind the seam in
// `crate::handlers::*`, which is hand-written (a missing impl is a compile error).
//
// Consumed query columns (verb-signatures.rq): noun_name, verb_name, verb_about,
// return_type, handler_name, args.

//! `dossier seal` verb (rendered).

#[cfg(feature = "ctdd")]
use clap_noun_verb::Result;
#[cfg(feature = "ctdd")]
use clap_noun_verb_macros::verb;
/// Seal a directory of 352-byte court records into a signed sealed-dossier wire file
// [coordinator] ctdd self-gate (template cannot emit gates)
#[cfg(feature = "ctdd")]
#[rustfmt::skip]
#[verb("seal", "dossier")]
pub fn dossier_seal(
    #[arg(index = 1)]
    dir: String,
    output: String,
) -> Result<()> {
    crate::handlers::dossier_seal(dir, output)
}
