// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt graph` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Discover the directly-follows graph from a receipt (wasm4pm)
#[verb("graph", "receipt")]
pub fn graph(receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::graph(receipt, format)
}
