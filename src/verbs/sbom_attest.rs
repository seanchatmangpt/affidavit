// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `receipt sbom-attest` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Emit a SLSA-flavored provenance attestation linking SBOM address to receipt
#[verb("sbom-attest", "receipt")]
pub fn sbom_attest(
    sbom_path: String,
    receipt: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sbom_attest(sbom_path, receipt, format)
}
