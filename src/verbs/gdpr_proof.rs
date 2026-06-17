// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt gdpr-proof` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Generate GDPR compliance proof (data access logs, deletion confirmation)
#[verb("gdpr-proof", "receipt")]
pub fn gdpr_proof(receipts_path: String, out: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::gdpr_proof(receipts_path, out, format)
}
