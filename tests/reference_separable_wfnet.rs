// Reference witness: SeparableWfNet<const S: SoundnessState> — the separability
// wrapper that PRESERVES the WF-net soundness typestate (COVERAGE.md §2 — separable
// WF-net). Complements the WfNetConst soundness-typestate witnesses.
//
// A separable WF-net is one decomposable into independent fragments; the wrapper
// tags a WfNetConst as separable WITHOUT altering its soundness state — the const
// generic S threads through declare_separable unchanged. This witnesses that a net
// declared separable keeps exactly the soundness state it had. Failing-when-fake:
// if declaration silently reset or upgraded S, the asserted state would differ.

use wasm4pm_compat::law::SoundnessState;
use wasm4pm_compat::petri::{SeparableWfNet, WfNetConst};

#[test]
fn declaring_separable_preserves_the_unknown_soundness_state() {
    let net = WfNetConst::<{ SoundnessState::Unknown }>::new();
    let sep = SeparableWfNet::declare_separable(net);
    assert_eq!(
        sep.net.soundness_state(),
        SoundnessState::Unknown,
        "separability is orthogonal to soundness — state preserved"
    );
}

#[test]
fn declaring_separable_preserves_a_claimed_state() {
    let claimed = WfNetConst::<{ SoundnessState::Unknown }>::new().claim_sound();
    let sep = SeparableWfNet::declare_separable(claimed);
    assert_eq!(
        sep.net.soundness_state(),
        SoundnessState::Claimed,
        "a claimed-but-unproven net stays Claimed after separability declaration"
    );
}
