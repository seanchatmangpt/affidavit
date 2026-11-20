// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! `receipt proof-carrying-verify` — verify an ML-DSA-65 seal/v2 over a
//! minted conformance subject.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Verify a proof-carrying subject JSON under seal/v2 context.
#[rustfmt::skip]
#[verb("proof-carrying-verify", "receipt")]
pub fn proof_carrying_verify(
    #[arg(index = 1)]
    subject: String,
    pk: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::proof_carrying_verify(subject, pk, format)
}
