// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `standing certify` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Seal a standing claim over an admitted receipt (affidavit/standing/v2)
#[verb("certify", "standing")]
pub fn certify(
    receipt: String,
    observation: String,
    scope: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::standing_certify(receipt, observation, scope, out, format)
}
