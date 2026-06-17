// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `receipt sbom-ntia` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Certify an SBOM against the NTIA minimum elements (EO 14028)
#[verb("sbom-ntia", "receipt")]
pub fn sbom_ntia(sbom_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::sbom_ntia(sbom_path, format)
}
