// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! `aod witness` — Historical Witness (envelope-only, zero Region 3).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Historical witness on an expired/rotated `.aod` fossil.
#[rustfmt::skip]
#[verb("witness", "aod")]
pub fn aod_witness(
    #[arg(index = 1)]
    file: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::aod_witness(file, format)
}
