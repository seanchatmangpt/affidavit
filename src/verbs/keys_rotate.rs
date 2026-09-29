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

//! `keys rotate` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Rotate a registered ES256 key to a freshly generated ES256 successor: the successor signs the rotation record and its public record joins the store
#[rustfmt::skip]
#[verb("rotate", "keys")]
pub fn keys_rotate(
    #[arg(index = 1)]
    kid: String,
    store: Option<String>,
    out: Option<String>,
) -> Result<()> {
    crate::handlers::keys_rotate(
        kid,
        store,
        out,
    )
}
