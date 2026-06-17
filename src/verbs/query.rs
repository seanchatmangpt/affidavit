// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt query` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Query receipts by time range, event type, repo, or SPARQL expression
#[verb("query", "receipt")]
pub fn query(q: String, receipts_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::query(q, receipts_path, format)
}
