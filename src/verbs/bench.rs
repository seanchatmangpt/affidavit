// Hand-authored verb wrapper (pending ontology addition + ggen re-render).
//! `receipt bench` verb (DX/QOL capability surface).

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;

/// `affi receipt bench` — inline performance check for receipt operations.
///
/// Times the emit→assemble→verify cycle and reports latencies.
/// A quick alternative to `cargo bench` for CI regression detection.
/// (ARDPRD §3 NFR-1, NFR-2)
#[verb("bench", "receipt")]
pub fn bench(
    iterations: Option<u32>,
) -> Result<()> {
    crate::handlers::bench(iterations)
}
