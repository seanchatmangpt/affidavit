// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `errc verify` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Re-run the ERRC directional and preservation laws over a sealed receipt
#[verb("verify", "errc")]
pub fn verify(receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::errc_verify(receipt, format)
}
