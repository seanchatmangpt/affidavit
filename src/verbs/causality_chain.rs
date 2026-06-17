// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt causality-chain` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Trace causal event chain (X caused Y caused Z)
#[verb("causality-chain", "receipt")]
pub fn causality_chain(
    start_event: String,
    receipts_path: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::causality_chain(start_event, receipts_path, format)
}
