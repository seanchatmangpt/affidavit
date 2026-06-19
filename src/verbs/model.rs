// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI interface only;
// the body delegates to the stable consumer-implemented handler.

//! `receipt model` verb — discover a process model and optionally check
//! breed-stage conformance against the wasm4pm-cognition registry.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Discover a process model from a receipt's events (wasm4pm).
/// Pass --breed to check conformance against a breed's declared stage sequence.
#[verb("model", "receipt")]
pub fn model(
    receipt: String,
    breed: Option<String>,
    registry: Option<String>,
) -> Result<()> {
    crate::handlers::model(receipt, breed, registry)
}
