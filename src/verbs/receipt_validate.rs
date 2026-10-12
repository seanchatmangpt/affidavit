// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `receipt validate` verb (Phase D M2, feature `ctdd`): fully verify one
// 352-byte CTRT court record (file path, or `-`/omitted for stdin), mapping
// every domain refusal to its [`chicago_tdd_tools::ledger::LedgerExitCode`]
// class.

//! `receipt validate` verb.

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// Validate one 352-byte court receipt record (file path, or `-` for stdin)
#[rustfmt::skip]
#[verb("validate", "receipt")]
pub fn receipt_validate(
    #[arg(index = 1)]
    path: Option<String>,
) -> Result<()> {
    crate::ctdd_cli::validate_outcome(path.as_deref())
        .map(|outcome| crate::ctdd_cli::render_and_exit("receipt validate", &outcome))
        .map_err(|e| {
            eprintln!("refusal: io {e}");
            std::process::exit(i32::from(crate::ctdd_cli::IO_EXIT));
        })
}
