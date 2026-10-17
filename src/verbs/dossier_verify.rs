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

//! `dossier verify` verb (rendered).

#[cfg(feature = "ctdd")]
use clap_noun_verb::Result;
#[cfg(feature = "ctdd")]
use clap_noun_verb_macros::verb;
/// Verify a sealed-dossier wire file against the local ML-DSA-65 key store
// [coordinator] ctdd self-gate (template cannot emit gates)
#[cfg(feature = "ctdd")]
#[rustfmt::skip]
#[verb("verify", "dossier")]
pub fn dossier_verify(
    #[arg(index = 1)]
    file: String,
) -> Result<()> {
    crate::handlers::dossier_verify(file)
}
