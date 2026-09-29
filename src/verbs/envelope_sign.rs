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

//! `envelope sign` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Seal a receipt under an ES256 key: build the CTP envelope, sign its canonical pre-image, write the PQ-SEAL-v1 document
#[rustfmt::skip]
#[verb("sign", "envelope")]
pub fn envelope_sign(
    #[arg(index = 1)]
    receipt: String,
    #[arg(index = 2)]
    key_file: String,
    out: Option<String>,
) -> Result<()> {
    crate::handlers::envelope_sign(
        receipt,
        key_file,
        out,
    )
}
