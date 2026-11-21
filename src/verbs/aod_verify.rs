// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//!
//! `aod verify --live` — Live Gate: ontology match + monoatomic lease burn.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Live Gate over a sealed `.aod` (requires `--live`).
///
/// Production default: static [`crate::aod_ontology::compiled_fixture`].
/// `--ontology` / `--strides` remain optional court overrides only.
#[rustfmt::skip]
#[verb("verify", "aod")]
pub fn aod_verify(
    #[arg(index = 1)]
    file: String,
    live: bool,
    ontology: Option<String>,
    strides: Option<String>,
    registry: Option<String>,
) -> Result<()> {
    crate::handlers::aod_verify(file, live, ontology, strides, registry)
}
