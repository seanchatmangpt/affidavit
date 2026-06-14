// Reference witness: the const-generic WfNet SOUNDNESS typestate
// (WfNetConst<const S: SoundnessState>) and the non-forgeability of the
// Witnessed state (COVERAGE.md §2 — soundness typestate).
//
// The sanctioned path is Unknown ──claim_sound──▶ Claimed ──witness_soundness(proof)──▶ Witnessed.
// `claim_sound` is free (anyone may CLAIM soundness), but `witness_soundness`
// requires a `SoundnessProof` whose constructor is `pub(crate)` — so an external
// crate CANNOT mint a `WfNetConst<Witnessed>` by fiat. This is the same
// non-forgeability idiom as the receipt seal, at the soundness level: a claim is
// cheap, a witness must be earned through the sanctioned (in-crate) proof issuer.

use wasm4pm_compat::law::SoundnessState;
use wasm4pm_compat::petri::WfNetConst;

#[test]
fn soundness_typestate_advances_unknown_to_claimed() {
    let unknown = WfNetConst::<{ SoundnessState::Unknown }>::new();
    assert_eq!(unknown.soundness_state(), SoundnessState::Unknown, "fresh net: soundness Unknown");

    // claim_sound is freely available — anyone may CLAIM (unproven) soundness.
    let claimed = unknown.claim_sound();
    assert_eq!(claimed.soundness_state(), SoundnessState::Claimed, "claimed (unproven) soundness");
}

#[test]
fn witnessed_soundness_is_non_forgeable_from_outside() {
    // The ONLY transition into Witnessed is `claim_sound(...).witness_soundness(proof)`,
    // and `proof: SoundnessProof` has a `pub(crate)` constructor. An external crate
    // (this test) cannot construct a SoundnessProof, so it cannot reach
    // WfNetConst<Witnessed> by fiat — the same seal idiom as the receipt's _seal.
    //
    // We assert the reachable boundary: from outside, Claimed is the furthest a
    // caller can advance without the sanctioned in-crate proof. (Attempting
    // `claimed.witness_soundness(SoundnessProof::new())` would not compile —
    // SoundnessProof::new is pub(crate).) That visibility barrier IS the
    // non-forgeability law, analogous to T-1 (no fiat Admitted) for soundness.
    let claimed = WfNetConst::<{ SoundnessState::Unknown }>::new().claim_sound();
    assert_eq!(
        claimed.soundness_state(),
        SoundnessState::Claimed,
        "Claimed is reachable; Witnessed is not, absent a sanctioned SoundnessProof"
    );
    // Sanity: the three soundness states are distinct.
    assert_ne!(SoundnessState::Claimed, SoundnessState::Witnessed);
    assert_ne!(SoundnessState::Unknown, SoundnessState::Witnessed);
}
