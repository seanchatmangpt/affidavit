// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt conformance` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compute fitness, activity coverage, and simplicity metrics
#[verb("conformance", "receipt")]
pub fn conformance(#[arg(index = 1)] receipt: String) -> Result<()> {
    crate::handlers::conformance(receipt)
}
