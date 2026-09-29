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

//! `receipt license-compliance` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Check license compliance (GPL, SSPL, commercial licenses)
#[rustfmt::skip]
#[verb("license-compliance", "receipt")]
pub fn license_compliance(
    receipts_path: String,
    license_policy: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::license_compliance(
        receipts_path,
        license_policy,
        format,
    )
}
