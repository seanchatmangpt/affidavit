// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `receipt sbom-blast-radius` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compute the supply-chain blast radius of a component (transitive dependents)
#[verb("sbom-blast-radius", "receipt")]
pub fn sbom_blast_radius(
    sbom_path: String,
    component: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sbom_blast_radius(sbom_path, component, format)
}
