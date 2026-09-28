// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `errc assure` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Seal a one-witness-per-claim assurance ledger over a sealed ERRC receipt
#[verb("assure", "errc")]
pub fn assure(
    parent: String,
    witnesses: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::errc_assure(parent, witnesses, out, format)
}
