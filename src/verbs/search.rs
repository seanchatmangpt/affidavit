// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt search` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Full-text search over receipt payloads (grep-like)
#[verb("search", "receipt")]
pub fn search(pattern: String, receipts_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::search(pattern, receipts_path, format)
}
