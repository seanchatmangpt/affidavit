// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt notarize` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Notarize a receipt with external authority (timestamp + signature)
#[verb("notarize", "receipt")]
pub fn notarize(receipt: String, out: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::notarize(receipt, out, format)
}
