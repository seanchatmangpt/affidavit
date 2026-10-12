// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `dossier inspect` verb: verifies the dossier JSON, checks Merkle root, and
// validates ML-DSA-65 post-quantum signature.

//! `dossier inspect` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Inspect a NATO STANAG 4107 JSON dossier and verify Merkle root / ML-DSA-65 signature
#[rustfmt::skip]
#[verb("inspect", "dossier")]
pub fn dossier_inspect(
    #[arg(index = 1)]
    path: String,
    verify_pqc_signature: Option<String>,
    assert_merkle_root: Option<bool>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::dossier_inspect(
        path,
        verify_pqc_signature,
        assert_merkle_root.unwrap_or(false),
        format,
    )
}
