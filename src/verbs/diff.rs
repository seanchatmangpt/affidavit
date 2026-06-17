// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt diff` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compare two receipts and print their differences
#[verb("diff", "receipt")]
pub fn diff(receipt_a: String, receipt_b: String, format: Option<String>) -> Result<()> {
    crate::handlers::diff(receipt_a, receipt_b, format)
}
