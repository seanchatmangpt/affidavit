// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt model` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Discover a process model from a receipt's events (wasm4pm)
#[verb("model", "receipt")]
pub fn model(receipt: String) -> Result<()> {
    crate::handlers::model(receipt)
}
