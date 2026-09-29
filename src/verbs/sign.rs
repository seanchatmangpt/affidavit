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

//! `receipt sign` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Sign a receipt or attestation with your key
#[rustfmt::skip]
#[verb("sign", "receipt")]
pub fn sign(
    #[arg(index = 1)]
    receipt: String,
    key_path: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sign(
        receipt,
        key_path,
        out,
        format,
    )
}
