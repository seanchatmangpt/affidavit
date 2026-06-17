// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt soc2-audit` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Generate SOC 2 audit trail and compliance proof from receipts
#[verb("soc2-audit", "receipt")]
pub fn soc2_audit(
    receipts_path: String,
    soc2_type: Option<String>,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::soc2_audit(receipts_path, soc2_type, out, format)
}
