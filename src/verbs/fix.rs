// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `receipt fix` verb — safe structural repairs on receipt files.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Apply a safe structural repair to a receipt: quarantine a tampered file
/// or finalize a stale working receipt. Use --dry-run to preview changes.
#[verb("fix", "receipt")]
pub fn fix(
    receipt: String,
    action: Option<String>,
    dry_run: bool,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::fix_receipt(receipt, action, dry_run, format)
}
