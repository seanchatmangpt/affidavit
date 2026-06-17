// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt assemble` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Finalize the working receipt into an immutable receipt file
#[verb("assemble", "receipt")]
pub fn assemble(out: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::assemble(out, format)
}
