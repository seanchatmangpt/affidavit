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

//! `keys generate` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Generate a real ES256 signing key and append its public record to the key store (the secret never touches disk)
#[rustfmt::skip]
#[verb("generate", "keys")]
pub fn keys_generate(
    #[arg(index = 1)]
    algorithm: String,
    #[arg(index = 2)]
    custodian: String,
    out: Option<String>,
) -> Result<()> {
    crate::handlers::keys_generate(
        algorithm,
        custodian,
        out,
    )
}
