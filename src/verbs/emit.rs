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
pub fn emit(r#type: String, object: Vec<String>, payload: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit(r#type, object, payload, format)
}

