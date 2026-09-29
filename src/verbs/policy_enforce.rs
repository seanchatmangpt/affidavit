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

//! `receipt policy-enforce` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Enforce organizational policies (2-person approval, segregation of duties)
#[rustfmt::skip]
#[verb("policy-enforce", "receipt")]
pub fn policy_enforce(
    receipts_path: String,
    policy_file: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::policy_enforce(
        receipts_path,
        policy_file,
        format,
    )
}
