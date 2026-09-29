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

//! `envelope verify` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Adjudicate a PQ-SEAL-v1 sealed receipt against the registered keys and print the standing VERDICT JSON (exit 0=VALID, 2=decided otherwise)
#[rustfmt::skip]
#[verb("verify", "envelope")]
pub fn envelope_verify(
    #[arg(index = 1)]
    sealed_file: String,
    store: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::envelope_verify(
        sealed_file,
        store,
        format,
    )
}
