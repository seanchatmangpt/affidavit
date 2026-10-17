// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// `dossier seal` verb (Phase D M2, feature `ctdd`): walk a dossier directory
// of 352-byte court records, group them per court, replay each court chain
// through the domain `CourtLedgerBuilder`, Merkle-seal the dossier root, and
// seal it with the local ML-DSA-65 key into the fixed-length sealed wire
// (`sealer::encode`).

//! `dossier seal` verb.

#[cfg(feature = "ctdd")]
use clap_noun_verb::Result;

/// Seal a dossier directory of court records into a signed sealed-dossier wire file
#[cfg(feature = "ctdd")]
#[rustfmt::skip]
#[verb("seal", "dossier")]
pub fn dossier_seal(
    #[arg(index = 1)]
    dir: String,
    output: String,
) -> Result<()> {
    let signer = crate::ctdd_cli::load_signer_or_exit();
    crate::ctdd_cli::seal_outcome(&dir, &output, &signer)
        .map(|outcome| crate::ctdd_cli::render_and_exit("dossier seal", &outcome))
        .map_err(|e| {
            eprintln!("refusal: io {e}");
            std::process::exit(i32::from(crate::ctdd_cli::IO_EXIT));
        })
}
