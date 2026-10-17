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

//! `dossier inspect` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Inspect a CTT-DOSSIER-v1 dossier: recompute the Merkle root, optionally verify the ML-DSA-65 signature
#[rustfmt::skip]
#[verb("inspect", "dossier")]
pub fn dossier_inspect(
    #[arg(index = 1)]
    path: String,
    verify_pqc_signature: Option<String>,
    assert_merkle_root: bool,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::dossier_inspect(
        path,
        verify_pqc_signature,
        assert_merkle_root,
        format,
    )
}
