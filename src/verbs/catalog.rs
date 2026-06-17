// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt catalog` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// List and search available receipt fixtures
#[verb("catalog", "receipt")]
pub fn catalog(filter_name: Option<String>, filter_events: Option<usize>) -> Result<()> {
    crate::handlers::catalog(filter_name, filter_events)
}
