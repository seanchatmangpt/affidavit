// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt security-debt` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Analyze security debt (unpatched vulns, CVE age, remediation lag)
#[verb("security-debt", "receipt")]
pub fn security_debt(
    receipts_path: String,
    time_range: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::security_debt(receipts_path, time_range, format)
}
