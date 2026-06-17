// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt anomaly-detect` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Detect anomalies (unusual deploy timing, error rate spikes, latency changes)
#[verb("anomaly-detect", "receipt")]
pub fn anomaly_detect(
    receipts_path: String,
    sensitivity: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::anomaly_detect(receipts_path, sensitivity, format)
}
