// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt find-blast-radius` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Find all downstream repos/services affected by a change
#[verb("find-blast-radius", "receipt")]
pub fn find_blast_radius(
    change_event: String,
    receipts_path: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::find_blast_radius(change_event, receipts_path, format)
}
