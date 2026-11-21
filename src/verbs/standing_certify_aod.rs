// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! `standing certify-aod` verb — AOD Live Gate format → PARTIAL_ALIVE standing.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Seal PARTIAL_ALIVE standing from a Live-Gate-admissible `.aod`.
#[rustfmt::skip]
#[verb("certify-aod", "standing")]
pub fn standing_certify_aod(
    receipt: String,
    aod: String,
    subject: String,
    scope: String,
    out: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::standing_certify_aod(
        receipt,
        aod,
        subject,
        scope,
        out,
        format,
    )
}
