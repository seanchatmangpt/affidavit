// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt emit-from-cicd` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Consume CI/CD platform events (GitHub Actions, CircleCI, GitLab CI, Jenkins)
#[verb("emit-from-cicd", "receipt")]
pub fn emit_from_cicd(provider: String, job_status: String, format: Option<String>) -> Result<()> {
    crate::handlers::emit_from_cicd(provider, job_status, format)
}
