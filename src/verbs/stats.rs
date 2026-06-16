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

//! `receipt stats` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// One-shot aggregate stats for a receipt (counts + DFG size + conformance)
#[verb("stats", "receipt")]
pub fn stats(format: Option<String>, #[arg(index = 1)] receipt: String) -> Result<()> {
    crate::handlers::stats(format, receipt)
}
