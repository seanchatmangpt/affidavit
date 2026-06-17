// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt verify-compliance` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Verify receipt against compliance rules (SOC 2, GDPR, HIPAA, PCI-DSS)
#[verb("verify-compliance", "receipt")]
pub fn verify_compliance(receipt: String, framework: String, format: Option<String>) -> Result<()> {
    crate::handlers::verify_compliance(receipt, framework, format)
}
