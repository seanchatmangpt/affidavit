// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt assemble-and-notarize` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Assemble receipt and obtain timestamp notarization from external authority
#[verb("assemble-and-notarize", "receipt")]
pub fn assemble_and_notarize(notary_provider: Option<String>, out: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::assemble_and_notarize(notary_provider, out, format)
}
