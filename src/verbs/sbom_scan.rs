// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `receipt sbom-scan` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Correlate vulnerabilities/VEX against SBOM components and propagate risk
#[verb("sbom-scan", "receipt")]
pub fn sbom_scan(
    sbom_path: String,
    advisories_path: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sbom_scan(sbom_path, advisories_path, format)
}
