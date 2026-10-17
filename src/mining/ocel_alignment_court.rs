//! Capability register row #40 — alignment-claim grounding court.
//!
//! Derives the move sequence from the actual trace-vs-model comparison,
//! attaches `process_ref` = chain hash, and requires
//! `wasm4pm_compat::alignment::AlignmentClaim::admit_flat() == Ok(())` before
//! any fitness value is emitted. A zero-move or empty-ref alignment is a typed
//! refusal — never a fitness of 0.0. Mutating the move sequence changes
//! `synchronous_count()` and the verdict.

use wasm4pm_compat::alignment::{AlignmentClaim, AlignmentRefusal, MoveKind};
use wasm4pm_compat::petri::PetriNet;

use crate::mining::MiningError;
use crate::types::AdmittedReceipt;

/// A grounded alignment result: the claim plus the fitness derived from it.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundedFitness {
    /// The grounded alignment claim (moves derived from the real comparison;
    /// `process_ref` is the receipt's BLAKE3 chain hash).
    pub claim: AlignmentClaim,
    /// Alignment fitness: `synchronous_count() / moves.len()` in [0.0, 1.0].
    pub fitness: f64,
    /// Human-readable interpretation (see [`crate::mining::interpret_score`]).
    pub interpretation: &'static str,
}

impl GroundedFitness {
    /// Emits the fitness **only after** the underlying claim admits.
    ///
    /// Zero-move and ungrounded alignments are typed refusals — a fitness of
    /// 0.0 is never fabricated from a vacuous claim.
    pub fn emit(claim: AlignmentClaim) -> Result<Self, MiningError> {
        // The grounding law: admit_flat() must be Ok before any value.
        if let Err(refusal) = claim.admit_flat() {
            return Err(MiningError::Alignment(refusal_to_msg(&refusal)));
        }
        let sync = claim.synchronous_count();
        let total = claim.moves.len();
        let fitness = sync as f64 / total as f64;
        Ok(GroundedFitness {
            interpretation: crate::mining::interpret_score(fitness),
            fitness,
            claim,
        })
    }
}

/// Map a compat refusal to its stable, typed message string.
fn refusal_to_msg(r: &AlignmentRefusal) -> String {
    match r {
        AlignmentRefusal::EmptyAlignment => {
            "REFUSED_EMPTY_ALIGNMENT: zero-move alignment is vacuous; no fitness is emitted"
                .to_string()
        }
        AlignmentRefusal::UngroundedAlignment => {
            "REFUSED_UNGROUNDED_ALIGNMENT: empty process_ref; no fitness is emitted".to_string()
        }
        // The compat refusal enum is #[non_exhaustive]; fold future variants
        // into an equally typed refusal rather than panicking.
        _ => "REFUSED_ALIGNMENT_UNSPECIFIED".to_string(),
    }
}

/// Derive an alignment claim from the *actual* trace-vs-model comparison.
///
/// The trace is the receipt's event-type sequence; the model's activity set is
/// its transition labels. Each trace step that the model can replay (its
/// activity label exists as a transition) is a synchronous move; each step the
/// model cannot replay is a log-only move. `process_ref` is bound to the
/// receipt's chain hash so the claim is grounded to the exact evidence subject.
pub fn derive_alignment_claim(admitted: &AdmittedReceipt, model: &PetriNet) -> AlignmentClaim {
    let receipt = &admitted.value;
    let model_activities: std::collections::HashSet<&str> =
        model.transitions().iter().map(|t| t.label()).collect();
    let moves: Vec<MoveKind> = receipt
        .events
        .iter()
        .map(|e| {
            if model_activities.contains(e.event_type.as_str()) {
                MoveKind::Synchronous
            } else {
                MoveKind::LogOnly
            }
        })
        .collect();
    AlignmentClaim::new(
        moves,
        format!("affidavit:receipt:chain-hash:{}", receipt.chain_hash),
    )
}

/// Full grounded-fitness path: derive → admit → emit.
pub fn grounded_alignment_fitness(
    admitted: &AdmittedReceipt,
    model: &PetriNet,
) -> Result<GroundedFitness, MiningError> {
    GroundedFitness::emit(derive_alignment_claim(admitted, model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::admit;
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::types::AdmittedReceipt;

    fn admitted_receipt(event_types: &[&str]) -> AdmittedReceipt {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for (i, ty) in event_types.iter().enumerate() {
            let payload = format!("p{i}");
            asm.append(
                build_event(
                    *ty,
                    vec![object_ref("o1", "artifact")],
                    payload.as_bytes(),
                    &mut counter,
                )
                .unwrap(),
            )
            .unwrap();
        }
        admit(asm.finalize()).expect("fixture receipt must admit")
    }

    fn two_transition_net() -> PetriNet {
        use wasm4pm_compat::petri::{Arc, Marking, Place, Transition};
        PetriNet::new(
            [Place::new("start"), Place::new("end")],
            [
                Transition::new("create", "create"),
                Transition::new("update", "update"),
            ],
            [
                Arc::place_to_transition("start", "create"),
                Arc::transition_to_place("create", "end"),
                Arc::place_to_transition("end", "update"),
            ],
            Marking::new([("start".to_string(), 1)]),
        )
    }

    #[test]
    fn moves_are_derived_from_the_actual_comparison() {
        let admitted = admitted_receipt(&["create", "update", "archive"]);
        let net = two_transition_net();
        let claim = derive_alignment_claim(&admitted, &net);
        assert_eq!(
            claim.moves,
            vec![
                MoveKind::Synchronous,
                MoveKind::Synchronous,
                MoveKind::LogOnly
            ],
            "only 'archive' is a log-only move under this model"
        );
        assert_eq!(claim.synchronous_count(), 2);
        assert_eq!(claim.log_only_count(), 1);
        // Grounded to the exact evidence subject.
        assert!(claim
            .process_ref
            .contains(&format!("{}", admitted.value.chain_hash)));
        assert!(claim.is_grounded());
    }

    #[test]
    fn grounded_fitness_emits_only_after_admission() {
        let admitted = admitted_receipt(&["create", "update"]);
        let net = two_transition_net();
        let gf = grounded_alignment_fitness(&admitted, &net).expect("must emit");
        assert_eq!(gf.fitness, 1.0);
        assert_eq!(gf.interpretation, "perfect-fit");
    }

    #[test]
    fn zero_move_alignment_is_typed_refusal_never_fitness_zero() {
        // A zero-move (vacuous) alignment is refused typed — never laundered
        // into a fitness value of 0.0. (An objectless/empty receipt cannot be
        // admitted per the OCEL court, so the claim is exercised directly.)
        let claim = AlignmentClaim::new(vec![], "affidavit:receipt:chain-hash:aa");
        let err = GroundedFitness::emit(claim).expect_err("zero-move alignment must refuse");
        let msg = format!("{err}");
        assert!(
            msg.contains("REFUSED_EMPTY_ALIGNMENT"),
            "expected typed empty-alignment refusal, got: {msg}"
        );
    }

    #[test]
    fn mutating_the_move_sequence_changes_sync_count_and_verdict() {
        let admitted = admitted_receipt(&["create", "update", "archive"]);
        let net = two_transition_net();
        let base = derive_alignment_claim(&admitted, &net);
        let base_gf = GroundedFitness::emit(base.clone()).unwrap();

        // Mutate one Synchronous -> LogOnly.
        let mut mutated = base.clone();
        mutated.moves[1] = MoveKind::LogOnly;
        let mut_gf = GroundedFitness::emit(mutated.clone()).unwrap();

        assert_ne!(
            base.synchronous_count(),
            mutated.synchronous_count(),
            "mutation must change synchronous_count"
        );
        assert_ne!(base_gf.fitness, mut_gf.fitness);
        assert_ne!(base_gf.interpretation, mut_gf.interpretation);

        // Ungrounded (empty process_ref) is a typed refusal too.
        let ungrounded = AlignmentClaim::new(base.moves.clone(), "");
        let err = GroundedFitness::emit(ungrounded).expect_err("must refuse");
        assert!(format!("{err}").contains("REFUSED_UNGROUNDED_ALIGNMENT"));
    }
}
