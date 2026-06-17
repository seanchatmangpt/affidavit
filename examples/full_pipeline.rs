#![cfg(feature = "lsp")]
//! Cross-product coherence: the whole provenance pipeline composing every
//! affidavit module through ONE receipt.
//!
//! ocel (build events) -> chain (assemble + finalize Receipt) -> verifier
//! (verify) -> admission (admit: OCEL court + chain verifier) ->
//! discovery/wasm4pm (discover_from_admitted) -> lsp (verdict_to_diagnostics).
//!
//! Each hop ASSERTS its real contract so a regression at any boundary fails the
//! example with a non-zero exit. The edge being demonstrated is the CLEAN-PATH
//! coherence guarantee: an accepted verdict must yield an empty diagnostic set
//! (no squiggles) AND the admitted receipt must discover a model naming the very
//! activities that were emitted. If admission silently faked acceptance, the
//! diagnostics-empty assertion or the activity-naming assertion would break.
//!
//! See the crate-level //! doc in src/lib.rs.

use affidavit::admission::admit;
use affidavit::chain::ChainAssembler;
use affidavit::discovery::discover_from_admitted;
use affidavit::lsp::verdict_to_diagnostics;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::verifier::verify;

fn main() {
    // --- hop 1: ocel — build well-formed operation events ----------------
    let mut counter = SeqCounter::new();
    let activities = ["seeded", "bred", "validated", "released"];
    let mut asm = ChainAssembler::new();

    for act in activities {
        let event = build_event(
            act,
            vec![object_ref("artifact-1", "artifact")],
            act.as_bytes(),
            &mut counter,
        )
        .expect("ocel: event must be well-formed");
        asm.append(event).expect("chain: append must fold cleanly");
    }
    assert_eq!(asm.len(), activities.len(), "all events appended");

    // --- hop 2: chain — finalize into an immutable Receipt ---------------
    let receipt = asm.finalize();
    assert_eq!(receipt.events.len(), activities.len());

    // --- hop 3: verifier — verify yields an ACCEPTED verdict -------------
    let verdict = verify(&receipt);
    assert!(
        verdict.accepted,
        "verifier: clean chain must ACCEPT, got reason={}",
        verdict.reason
    );
    assert!(
        verdict.outcomes.iter().all(|o| o.passed),
        "verifier: every stage must pass on the clean path"
    );

    // --- hop 4: admission — admit through BOTH courts (Ok) ---------------
    let admitted = admit(receipt).expect("admission: both courts must admit the clean receipt");

    // --- hop 5: discovery/wasm4pm — discover from the admitted carrier ---
    let model = discover_from_admitted(&admitted);
    for act in activities {
        assert!(
            model.contains(act),
            "discovery: discovered model must name activity {act:?}; model was: {model}"
        );
    }

    // --- hop 6: lsp — verdict_to_diagnostics on the CLEAN verdict --------
    // The edge: an accepted verdict must produce ZERO diagnostics (no squiggles).
    let diagnostics = verdict_to_diagnostics(&verdict);
    assert!(
        diagnostics.is_empty(),
        "lsp: clean accepted verdict must yield NO diagnostics, got {}",
        diagnostics.len()
    );

    println!("cross-product coherence holds: 6 hops, accepted, model named all activities, 0 diagnostics");
}
