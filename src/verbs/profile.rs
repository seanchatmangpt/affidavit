// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt profile` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Run a sustained workload for flamegraph profiling
#[verb("profile", "receipt")]
pub fn profile(receipt: Option<String>, duration: Option<u64>) -> Result<()> {
    crate::handlers::profile(receipt, duration)
}
