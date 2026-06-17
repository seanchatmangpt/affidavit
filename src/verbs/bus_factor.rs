// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt bus-factor` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Calculate bus factor (if person X leaves, how many repos are at risk)
#[verb("bus-factor", "receipt")]
pub fn bus_factor(receipts_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::bus_factor(receipts_path, format)
}
