// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt team-velocity` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compute team productivity metrics (PR review time, code review latency, rework)
#[verb("team-velocity", "receipt")]
pub fn team_velocity(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::team_velocity(receipts_path, time_range, format)
}
