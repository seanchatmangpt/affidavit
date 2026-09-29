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

//! `receipt assemble-with-signature` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Assemble receipt and sign with Ed25519 or Sigstore keyless
#[rustfmt::skip]
#[verb("assemble-with-signature", "receipt")]
pub fn assemble_with_signature(
    signing_method: Option<String>,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::assemble_with_signature(
        signing_method,
        out,
        format,
    )
}
