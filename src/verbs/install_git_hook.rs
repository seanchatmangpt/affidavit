// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin verb wrapper rendered from O* by ggen. The pack is authoritative for the
// CLI *interface* only; the body delegates to a stable consumer-implemented
// handler. There is NO logic slot here — business logic lives behind the seam in
// `crate::handlers::*`, which is hand-written (a missing impl is a compile error).
//
// Consumed query columns (verb-signatures.rq): noun_name, verb_name, verb_about,
// return_type, handler_name, args.

//! `receipt install-git-hook` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Install a post-commit Git hook that monitors code quality violations
#[rustfmt::skip]
#[verb("install-git-hook", "receipt")]
pub fn install_git_hook(
    threshold: Option<String>,
) -> Result<()> {
    crate::handlers::install_git_hook(
        threshold,
    )
}
