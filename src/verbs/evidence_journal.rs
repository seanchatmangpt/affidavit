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

//! `evidence journal` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Record a receipt's cryptographic standing as durable journal evidence: assemble a real receipt over the subject, seal it under the custody key (AFFI_SIGNING_KEY_PATH), adjudicate the standing, and append the hash-chained journal entry (.affi/standing-journal.jsonl)
#[rustfmt::skip]
#[verb("journal", "evidence")]
pub fn evidence_journal(
    #[arg(index = 1)]
    subject: String,
    out: Option<String>,
) -> Result<()> {
    crate::handlers::evidence_journal(
        subject,
        out,
    )
}
