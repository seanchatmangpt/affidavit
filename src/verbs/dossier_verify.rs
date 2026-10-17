// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `dossier verify` verb (Phase D, feature `ctdd`): decode a sealed-dossier
// wire file (`sealer::decode`, 3399-byte fixed-length wire) and verify it
// against the local ML-DSA-65 key store. A failing verdict is tamper
// evidence: Tampered class (exit 13).

//! `dossier verify` verb.

#[cfg(feature = "ctdd")]
use clap_noun_verb::Result;

/// Verify a previously sealed dossier wire file against the local key store
#[cfg(feature = "ctdd")]
#[rustfmt::skip]
#[verb("verify", "dossier")]
pub fn dossier_verify(
    #[arg(index = 1)]
    file: String,
) -> Result<()> {
    let signer = crate::ctdd_cli::load_signer_or_exit();
    crate::ctdd_cli::verify_outcome(&file, &signer.public_key())
        .map(|outcome| crate::ctdd_cli::render_and_exit("dossier verify", &outcome))
        .map_err(|e| {
            eprintln!("refusal: io {e}");
            std::process::exit(i32::from(crate::ctdd_cli::IO_EXIT));
        })
}
