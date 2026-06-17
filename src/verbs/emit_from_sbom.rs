// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `receipt emit-from-sbom` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Ingest an SPDX/CycloneDX SBOM and emit OCEL component/dependency events
#[verb("emit-from-sbom", "receipt")]
pub fn emit_from_sbom(sbom_path: String, format: Option<String>) -> Result<()> {
    crate::handlers::sbom_emit(sbom_path, format)
}
