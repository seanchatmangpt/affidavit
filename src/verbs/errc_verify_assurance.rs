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

//! `errc verify-assurance` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Re-run the claim-assurance law, optionally binding to the exact parent ERRC receipt
#[rustfmt::skip]
#[verb("verify-assurance", "errc")]
pub fn errc_verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::errc_verify_assurance(
        receipt,
        parent,
        format,
    )
}
