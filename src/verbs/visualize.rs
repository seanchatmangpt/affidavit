// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt visualize` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Visualize receipt as graph (DOT or JSON)
#[verb("visualize", "receipt")]
pub fn visualize(format: String, receipt: String) -> Result<()> {
    crate::handlers::visualize(format, receipt)
}
