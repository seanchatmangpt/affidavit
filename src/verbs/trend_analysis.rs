// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt trend-analysis` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Analyze trends (deployment velocity, test coverage, incident rate)
#[verb("trend-analysis", "receipt")]
pub fn trend_analysis(receipts_path: String, metric: String, time_range: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::trend_analysis(receipts_path, metric, time_range, format)
}
