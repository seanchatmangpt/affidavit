// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! `aod fold` — trajectory fold monoid on two CTRT shadow tips.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Compose two CTRT shadow tips (`R_A ⊕ R_B`).
#[rustfmt::skip]
#[verb("fold", "aod")]
pub fn aod_fold(
    #[arg(index = 1)]
    left: String,
    #[arg(index = 2)]
    right: String,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::aod_fold(left, right, format)
}
