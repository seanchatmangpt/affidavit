// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt diagnose` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Render verify outcomes as LSP-shaped diagnostics (lsp-max)
#[verb("diagnose", "receipt")]
pub fn diagnose(receipt: String) -> Result<()> {
    crate::handlers::diagnose(receipt)
}
