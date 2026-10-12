// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `dossier emit` verb: aggregates court receipts from a directory into a
// NATO STANAG 4107 JSON dossier.

//! `dossier emit` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Emit a signed NATO STANAG 4107 JSON dossier from court receipts
#[rustfmt::skip]
#[verb("emit", "dossier")]
pub fn dossier_emit(
    receipts_dir: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::dossier_emit(
        receipts_dir,
        out,
        format,
    )
}
