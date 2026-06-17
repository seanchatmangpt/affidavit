// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit-from-cloud` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Consume cloud platform events (AWS CloudTrail, GCP Audit Logs, Azure Activity)
#[verb("emit-from-cloud", "receipt")]
pub fn emit_from_cloud(provider: String, resource_type: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit_from_cloud(provider, resource_type, format)
}
