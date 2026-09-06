// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `errc certify` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Seal a declared ERRC transformation (affidavit/errc/v1)
#[verb("certify", "errc")]
pub fn certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::errc_certify(receipt, observation, out, format)
}
