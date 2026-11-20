// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! `receipt proof-carrying` verb — mint a seal/v2 subject from grounded
//! OCEL DFG conformance (admit_flat → fitness/precision → BLAKE3 digest).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Mint a proof-carrying OCEL DFG conformance subject (seal/v2 M).
#[allow(clippy::too_many_arguments)] // clap verb args mirror the handler 1:1
#[rustfmt::skip]
#[verb("proof-carrying", "receipt")]
pub fn proof_carrying(
    receipt: Option<String>,
    ocel: Option<String>,
    object_type: String,
    log_ref: Option<String>,
    model: Option<String>,
    format: Option<String>,
    out: Option<String>,
    seal_seed: Option<String>,
) -> Result<()> {
    crate::handlers::proof_carrying(
        receipt,
        ocel,
        object_type,
        log_ref,
        model,
        format,
        out,
        seal_seed,
    )
}
