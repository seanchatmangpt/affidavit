// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt orphaned-code` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Find repos with no commits in N days (orphaned/unmaintained)
#[verb("orphaned-code", "receipt")]
pub fn orphaned_code(receipts_path: String, days: Option<u32>, format: Option<String>) -> Result<()> {
    crate::handlers::orphaned_code(receipts_path, days, format)
}
