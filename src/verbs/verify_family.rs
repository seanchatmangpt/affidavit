// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt verify-family` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Verify multiple receipts from same repo/org are consistent
#[verb("verify-family", "receipt")]
pub fn verify_family(receipts_dir: String, format: Option<String>) -> Result<()> {
    crate::handlers::verify_family(receipts_dir, format)
}
