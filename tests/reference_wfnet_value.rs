// Reference witness: the value-level WfNet<S> — a workflow net wrapping a Petri
// net + final marking, with accessors and the Unknown→Claimed soundness-claim
// transition (COVERAGE.md §2 — value-level WfNet; complements WfNetConst).
//
// WfNet<SoundnessUnknown>::new(net, final_marking) wraps a workflow net;
// final_marking()/net() expose its structure; claim_sound() advances the
// value-level soundness typestate to Claimed (a CLAIM, distinct from a Witnessed
// proof). validate() refuses an empty final marking (MissingFinalMarking).

use wasm4pm_compat::petri::{Arc, Marking, PetriNet, Place, Transition, WfNet};

fn simple_net() -> PetriNet {
    PetriNet::new(
        [Place::new("p0"), Place::new("p1")],
        [Transition::new("t0", "fire")],
        [
            Arc::place_to_transition("p0", "t0"),
            Arc::transition_to_place("t0", "p1"),
        ],
        Marking::new([("p0".to_string(), 1)]),
    )
}

#[test]
fn wfnet_exposes_net_and_final_marking() {
    let wf = WfNet::new(simple_net(), Marking::new([("p1".to_string(), 1)]));
    assert_eq!(wf.validate(), Ok(()), "a WfNet with a final marking admits");
    assert!(wf.final_marking().is_some(), "final marking present");
    assert_eq!(
        wf.net().places().len(),
        2,
        "net accessor reaches the underlying places"
    );
}

#[test]
fn claim_sound_advances_value_level_soundness() {
    // The value-level Unknown → Claimed transition (a claim, not a witnessed proof).
    let wf = WfNet::new(simple_net(), Marking::new([("p1".to_string(), 1)]));
    let _claimed = wf.claim_sound(); // type is now WfNet<SoundnessClaimed>
                                     // (Reaching a Witnessed WfNet, like WfNetConst, requires the sanctioned proof
                                     // path — claim_sound only asserts the claim. The transition type-checks and
                                     // consumes the Unknown net.)
}
