// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt coverage-analysis` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Analyze test coverage trends across portfolio
#[verb("coverage-analysis", "receipt")]
pub fn coverage_analysis(receipts_path: String, time_range: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::coverage_analysis(receipts_path, time_range, format)
}
