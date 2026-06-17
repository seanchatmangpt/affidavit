// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt dora-metrics` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compute DORA 4 Key Metrics (deployment frequency, lead time, MTTR, change failure rate)
#[verb("dora-metrics", "receipt")]
pub fn dora_metrics(receipts_path: String, time_range: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::dora_metrics(receipts_path, time_range, format)
}
