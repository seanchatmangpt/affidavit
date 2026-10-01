// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper rendered from O* by ggen. The pack is authoritative for the
// CLI *interface* only; the body delegates to a stable consumer-implemented
// handler. There is NO logic slot here — business logic lives behind the seam in
// `crate::handlers::*`, which is hand-written (a missing impl is a compile error).
//
// Consumed query columns (verb-signatures.rq): noun_name, verb_name, verb_about,
// return_type, handler_name, args.

//! `evidence heads` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Audit the standing journal (full chain re-verified from genesis) and report the RFC 9162 tree head re-derived from the journal entries alone; with AFFI_SIGNING_KEY_PATH custody the head is published signed and self-verified
#[rustfmt::skip]
#[verb("heads", "evidence")]
pub fn evidence_heads(
    journal_file: Option<String>,
) -> Result<()> {
    crate::handlers::evidence_heads(
        journal_file,
    )
}
