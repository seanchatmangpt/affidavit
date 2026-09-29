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

//! `envelope export` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Export the attestation envelope of a PQ-SEAL-v1 sealed document: json = the CTP-ENVELOPE-v1 document (default), sa2a = the SA2A-C2-APPROVAL-v1 approval (JCS-canonical); unknown formats are refused
#[rustfmt::skip]
#[verb("export", "envelope")]
pub fn envelope_export(
    #[arg(index = 1)]
    sealed_file: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::envelope_export(sealed_file, format)
}
