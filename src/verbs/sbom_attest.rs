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

//! `receipt sbom-attest` verb (rendered).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
/// Emit a SLSA-flavored provenance attestation linking SBOM address to receipt
#[rustfmt::skip]
#[verb("sbom-attest", "receipt")]
pub fn sbom_attest(
    sbom_path: String,
    receipt: Option<String>,
    format: Option<String>,
) -> Result<()> {
    crate::handlers::sbom_attest(
        sbom_path,
        receipt,
        format,
    )
}
