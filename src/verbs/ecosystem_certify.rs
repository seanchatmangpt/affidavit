// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `ecosystem certify` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Federate sealed member standing receipts into one quorum-scored receipt
#[verb("certify", "ecosystem")]
pub fn certify(
    receipt: String,
    observation: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::ecosystem_certify(receipt, observation, out, format)
}
