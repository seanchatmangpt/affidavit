// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt stats` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// One-shot aggregate stats for a receipt (counts + DFG size + conformance)
#[verb("stats", "receipt")]
pub fn stats(receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::stats(receipt, format)
}
