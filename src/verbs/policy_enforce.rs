// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt policy-enforce` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Enforce organizational policies (2-person approval, segregation of duties)
#[verb("policy-enforce", "receipt")]
pub fn policy_enforce(receipts_path: String, policy_file: String, format: Option<String>) -> Result<()> {
    crate::handlers::policy_enforce(receipts_path, policy_file, format)
}
