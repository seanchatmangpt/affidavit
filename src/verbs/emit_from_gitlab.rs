// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit-from-gitlab` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Consume GitLab webhooks and emit events
#[verb("emit-from-gitlab", "receipt")]
pub fn emit_from_gitlab(repo: String, event_type: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit_from_gitlab(repo, event_type, format)
}
