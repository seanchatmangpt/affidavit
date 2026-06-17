// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt receipt-throughput` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Measure end-to-end latency of the emit -> assemble -> verify pipeline
#[verb("receipt-throughput", "receipt")]
pub fn receipt_throughput(iterations: Option<u32>) -> Result<()> {
    crate::handlers::receipt_throughput(iterations)
}
