// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! `aod rotate` — rematerialize under a fresh monoatomic lease + folded tip.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Rotate a prior `.aod` into a fresh lease and folded CTRT shadow.
#[rustfmt::skip]
#[verb("rotate", "aod")]
pub fn aod_rotate(
    #[arg(index = 1)]
    old: String,
    lease: String,
    out: String,
    epoch: Option<u64>,
) -> Result<()> {
    crate::handlers::aod_rotate(old, lease, out, epoch)
}
