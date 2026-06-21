// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `receipt why` verb — plain-language rejection explanation.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Explain in plain language why a receipt was rejected, with stage-by-stage remediation.
#[verb("why", "receipt")]
pub fn why(receipt: String, format: Option<String>) -> Result<()> {
    crate::handlers::why(receipt, format)
}
