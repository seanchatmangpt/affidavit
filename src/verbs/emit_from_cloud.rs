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

//! `receipt emit-from-cloud` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Consume cloud platform events (AWS CloudTrail, GCP Audit Logs, Azure Activity)
#[rustfmt::skip]
#[verb("emit-from-cloud", "receipt")]
pub fn emit_from_cloud(
    provider: String,
    resource_type: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::emit_from_cloud(
        provider,
        resource_type,
        format,
    )
}
