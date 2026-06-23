// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt inspect` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Detailed structural analysis of a receipt (event/object distribution)
#[verb("inspect", "receipt")]
pub fn inspect(#[arg(index = 1)] receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::inspect(receipt, format)
}
