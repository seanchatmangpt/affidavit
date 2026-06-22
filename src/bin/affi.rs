//! `affi` — The Provenance CLI
//!
//! A cryptographic provenance engine for high-assurance systems. Assembles, seals,
//! and certifies append-only BLAKE3 chains of operation-events (receipts).
//!
//! **Philosophy:** Certify, don't decide. The verifier checks a receipt against
//! a format standard without deciding whether work is honest (Rice's theorem).
//!
//! **Usage:** Run with `--help` to see 65+ commands (emit, verify, sbom, audit, etc.).
//! For guided examples, see the [README](https://github.com/seanchatmangpt/affidavit/blob/main/README.md).
//!
//! Hand-written static binary entry point (not generated). Delegates to [`affidavit::run()`].

fn main() -> clap_noun_verb::Result<()> {
    affidavit::run()
}
