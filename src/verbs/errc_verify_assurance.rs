// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `errc verify-assurance` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Re-run the claim-assurance law, optionally binding to the exact parent ERRC receipt
#[verb("verify-assurance", "errc")]
pub fn verify_assurance(
    receipt: String,
    parent: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::errc_verify_assurance(receipt, parent, format)
}
