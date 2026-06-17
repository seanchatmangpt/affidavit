// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit-batch` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Emit multiple events from JSON array in a single command
#[verb("emit-batch", "receipt")]
pub fn emit_batch(batch_file: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit_batch(batch_file, format)
}
