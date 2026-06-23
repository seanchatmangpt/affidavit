// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper. The pack is authoritative for the CLI *interface* only;
// the body delegates to a stable consumer-implemented handler.

//! `affi doctor` verb — environment and receipt-store health checks.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Run environment and receipt-store health checks; optionally apply safe automatic fixes
#[verb("doctor", "affi")]
pub fn doctor(receipts: Option<String>, fix: bool) -> Result<()> {
    crate::handlers::doctor(receipts, fix)
}
