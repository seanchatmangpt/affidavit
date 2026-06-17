// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt test` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// A dummy test verb for ontology validation
#[verb("test", "receipt")]
pub fn test() -> Result<()> {
    crate::handlers::test()
}
