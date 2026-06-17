// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt dependency-matrix` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Build dependency matrix across all repos (who uses what library versions)
#[verb("dependency-matrix", "receipt")]
pub fn dependency_matrix(receipts_path: String, output_matrix: Option<String>, format: Option<String>) -> Result<()> {
    crate::handlers::dependency_matrix(receipts_path, output_matrix, format)
}
