// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt attest` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Create a signed attestation for a receipt (SLSA provenance)
#[verb("attest", "receipt")]
pub fn attest(
    receipt: String,
    attestation_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::attest(receipt, attestation_type, out, format)
}
