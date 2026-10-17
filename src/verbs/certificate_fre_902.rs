// Copyright (c) 2026 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `certificate fre-902` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Generates self-authenticating electronic records affidavit under FRE 902(14)
#[rustfmt::skip]
#[verb("fre-902", "certificate")]
pub fn certificate_fre_902() -> Result<()> {
    crate::handlers::certificate_fre_902()
}
