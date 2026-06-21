// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `affi guide search <KEYWORD>` verb — full-text verb discovery.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Search the verb registry by keyword to discover relevant commands
#[verb("search", "guide")]
pub fn guide_search(keyword: String, format: Option<String>) -> Result<()> {
    crate::handlers::guide_search(keyword, format)
}
