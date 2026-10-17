// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `receipt chain-verify` verb (Phase D M2, feature `ctdd`): replay the BLAKE3
// receipt chain across all 352-byte court records in a directory, mapping
// every domain refusal to its [`chicago_tdd_tools::ledger::LedgerExitCode`]
// class.

//! `receipt chain-verify` verb.

#[cfg(feature = "ctdd")]
use clap_noun_verb::Result;

/// Verify the BLAKE3 receipt chain across all court records in a directory
#[cfg(feature = "ctdd")]
#[rustfmt::skip]
#[verb("chain-verify", "receipt")]
pub fn receipt_chain_verify(
    #[arg(index = 1)]
    dir: String,
) -> Result<()> {
    crate::ctdd_cli::chain_verify_outcome(&dir)
        .map(|outcome| crate::ctdd_cli::render_and_exit("receipt chain-verify", &outcome))
        .map_err(|e| {
            eprintln!("refusal: io {e}");
            std::process::exit(i32::from(crate::ctdd_cli::IO_EXIT));
        })
}
