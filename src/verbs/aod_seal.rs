// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! `aod seal` — write a Zero-String Sealed `.aod` under a monoatomic lease.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Write 64B-aligned avec buffers and append a 352B CTRT shadow.
#[allow(clippy::too_many_arguments)] // clap verb args mirror the handler 1:1
#[rustfmt::skip]
#[verb("seal", "aod")]
pub fn aod_seal(
    ontology: String,
    lease: String,
    #[arg(alias = "in")]
    input: String,
    out: String,
    rows: u64,
    epoch: Option<u64>,
    folded: Option<String>,
) -> Result<()> {
    crate::handlers::aod_seal(ontology, lease, input, out, rows, epoch, folded)
}
