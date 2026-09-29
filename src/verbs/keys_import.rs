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

//! `keys import` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Import an externally-held public key (hex) into the key store: fingerprint it and register it under a custodian (origin Imported)
#[rustfmt::skip]
#[verb("import", "keys")]
pub fn keys_import(
    #[arg(index = 1)]
    algorithm: String,
    #[arg(index = 2)]
    public_key_hex: String,
    #[arg(index = 3)]
    custodian: String,
    out: Option<String>,
) -> Result<()> {
    crate::handlers::keys_import(
        algorithm,
        public_key_hex,
        custodian,
        out,
    )
}
