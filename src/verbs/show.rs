// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt show` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Print a human-readable dump of a receipt chain
#[verb("show", "receipt")]
pub fn show(#[arg(index = 1)] receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::show(receipt, format)
}
