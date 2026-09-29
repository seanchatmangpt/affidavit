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

//! `keys revoke` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Revoke a registered key: append a tamper-evident revocation entry to the checksummed sidecar beside the key store
#[rustfmt::skip]
#[verb("revoke", "keys")]
pub fn keys_revoke(
    #[arg(index = 1)]
    kid: String,
    #[arg(index = 2)]
    reason: String,
    store: Option<String>,
) -> Result<()> {
    crate::handlers::keys_revoke(
        kid,
        reason,
        store,
    )
}
