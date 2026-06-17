// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit-from-security` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Consume security events (Snyk, SonarQube, Trivy, GitHub Advanced Security)
#[verb("emit-from-security", "receipt")]
pub fn emit_from_security(provider: String, vuln_type: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit_from_security(provider, vuln_type, format)
}
