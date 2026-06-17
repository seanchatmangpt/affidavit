// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt explain-incident` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Explain root cause of incident (trace from error to code change to PR)
#[verb("explain-incident", "receipt")]
pub fn explain_incident(incident_desc: String, receipts_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::explain_incident(incident_desc, receipts_path, format)
}
