// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt timeline` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Render event timeline across receipts with causality swimlanes
#[verb("timeline", "receipt")]
pub fn timeline(receipts_path: String, start_time: Option<String>, end_time: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::timeline(receipts_path, start_time, end_time, format)
}
