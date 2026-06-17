// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt assemble-with-signature` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Assemble receipt and sign with Ed25519 or Sigstore keyless
#[verb("assemble-with-signature", "receipt")]
pub fn assemble_with_signature(signing_method: Option<String>, out: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::assemble_with_signature(signing_method, out, format)
}
