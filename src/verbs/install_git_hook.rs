// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper auto-generated. The pack is authoritative for the CLI
// *interface* only; the body delegates to a stable consumer-implemented handler.

//! `receipt install-git-hook` verb (auto-generated).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Install a post-commit Git hook that monitors code quality violations
#[verb("install-git-hook", "receipt")]
pub fn install_git_hook(threshold: Option<String>) -> Result<()> {
    crate::handlers::install_git_hook(threshold)
}
