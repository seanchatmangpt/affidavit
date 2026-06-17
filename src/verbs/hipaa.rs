// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt hipaa` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Generate HIPAA compliance proof (access control, encryption, audit trails)
#[verb("hipaa", "receipt")]
pub fn hipaa(receipts_path: String, out: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::hipaa(receipts_path, out, format)
}
