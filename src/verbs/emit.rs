// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Append one operation-event to the working receipt
#[verb("emit", "receipt")]
pub fn emit(r#type: String, object: String, payload: String, format: Option<String>) -> Result<()> {
    let objects = object
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();
    crate::handlers::emit(r#type, objects, payload, format)
}
