// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt sign` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Sign a receipt or attestation with your key
#[verb("sign", "receipt")]
pub fn sign(
    receipt: String,
    key_path: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sign(receipt, key_path, out, format)
}
