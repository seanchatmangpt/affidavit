//! `affi` — The Provenance CLI
//!
//! A cryptographic provenance engine for high-assurance systems. Assembles, seals,
//! and certifies append-only BLAKE3 chains of operation-events (receipts).
//!
//! **Philosophy:** Certify, don't decide. The verifier checks a receipt against
//! a format standard without deciding whether work is honest (Rice's theorem).
//!
//! **Usage:** Run with `--help` to see 77 commands across 11 groups (emit,
//! verify, sbom, audit, and the federation courts `standing` / `ecosystem` /
//! `errc`). For guided examples, see the
//! [README](https://github.com/seanchatmangpt/affidavit/blob/main/README.md).
//!
//! Hand-written static binary entry point (not generated). Delegates to
//! [`affidavit::run()`] after answering `--version` itself — see below.

/// Answer a top-level `--version` / `-V` before the framework does.
///
/// `clap-noun-verb` builds its root command with `.version(env!(
/// "CARGO_PKG_VERSION"))`, which expands against *its own* manifest at the time
/// the library was compiled — so the framework reports `cli 26.6.2` rather than
/// the affidavit version. That is not cosmetic here: the chain genesis seed is
/// `concat!("affidavit-v", env!("CARGO_PKG_VERSION"), "-genesis")`, so a
/// receipt only verifies under the exact binary version that assembled it. An
/// operator diagnosing a cross-version `chain_integrity` failure reads this
/// string first, and it has to be the truth.
///
/// Only a bare top-level `affi --version` is intercepted; `affi <noun> <verb>
/// --version` still reaches the framework unchanged.
fn version_requested(args: &[String]) -> bool {
    matches!(
        args.first().map(String::as_str),
        Some("--version") | Some("-V")
    )
}

fn main() -> clap_noun_verb::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if version_requested(&args) {
        println!("affi {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    affidavit::run()
}

#[cfg(test)]
mod tests {
    use super::version_requested;

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn a_bare_top_level_version_flag_is_intercepted() {
        assert!(version_requested(&args(&["--version"])));
        assert!(version_requested(&args(&["-V"])));
    }

    #[test]
    fn a_subcommand_version_flag_is_left_to_the_framework() {
        assert!(!version_requested(&args(&[
            "receipt",
            "verify",
            "--version"
        ])));
        assert!(!version_requested(&args(&["standing", "certify"])));
        assert!(!version_requested(&args(&[])));
    }
}
