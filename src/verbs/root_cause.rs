// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt root-cause` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// RCA: find root cause by walking event chain backwards
#[verb("root-cause", "receipt")]
pub fn root_cause(effect_event: String, receipts_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::root_cause(effect_event, receipts_path, format)
}
