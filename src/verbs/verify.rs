// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt verify` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Run the certify pipeline over a receipt and print the verdict
#[verb("verify", "receipt")]
pub fn verify(
    #[arg(index = 1)] receipt: String,
    format: Option<String>,
    profile: Option<String>,
    strict: Option<bool>,
) -> Result<()> {
    crate::handlers::verify(receipt, format, profile, strict)
}
