// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt license-compliance` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Check license compliance (GPL, SSPL, commercial licenses)
#[verb("license-compliance", "receipt")]
pub fn license_compliance(receipts_path: String, license_policy: String, format: Option<String>) -> Result<()> {
    crate::handlers::license_compliance(receipts_path, license_policy, format)
}
