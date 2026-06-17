// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `receipt sbom-compliance` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Assess an SBOM against NTIA/EO14028/SLSA/in-toto/CISA/C-SCRM frameworks
#[verb("sbom-compliance", "receipt")]
pub fn sbom_compliance(
    sbom_path: String,
    framework: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sbom_compliance(sbom_path, framework, format)
}
