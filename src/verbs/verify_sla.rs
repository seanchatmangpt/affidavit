// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt verify-sla` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Verify receipt meets SLA/SLO targets (e.g., 4h MTTR, 2-person approval)
#[verb("verify-sla", "receipt")]
pub fn verify_sla(receipt: String, sla_file: String, format: Option<String>) -> Result<()> {
    crate::handlers::verify_sla(receipt, sla_file, format)
}

