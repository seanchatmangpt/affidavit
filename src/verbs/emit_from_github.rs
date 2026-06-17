// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit-from-github` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Consume GitHub webhooks and emit events (push, PR, release, workflow)
#[verb("emit-from-github", "receipt")]
pub fn emit_from_github(repo: String, event_type: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit_from_github(repo, event_type, format)
}
