// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt variance` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Measure control-flow surprise (anomaly score) and its computation cost
#[verb("variance", "receipt")]
pub fn variance(receipt: Option<String>, iterations: Option<u32>) -> Result<()> {
    crate::handlers::variance(receipt, iterations)
}
