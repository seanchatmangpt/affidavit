//! Capability register row #76 — data-quality grounding gate BEFORE any
//! fitness/alignment claim.
//!
//! Wires `wasm4pm_compat::data_quality::{DataQualityClaim, DataQualityRefusal}`
//! into the row-10/40 court path as a preflight: an ungrounded dimension (a
//! completeness claim over a log with severed case links) yields a typed
//! refusal BEFORE any fitness number. A fitness value is never emitted from an
//! ungrounded log.
//!
//! ## Adaptation to the real compat API (disclosed deviations)
//!
//! 1. The compat `DataQualityClaim::admit_flat()` gates the claim *shape* only
//!    (non-empty dimensions + non-empty `log_ref`); it never inspects log
//!    contents. Dimension-vs-log grounding (e.g. a completeness claim over a
//!    log with severed case links) is enforced court-side here, structurally:
//!    a `Completeness`-dimension claim is grounded only when every event in the
//!    OCEL projection carries at least one E2O relationship to an object of the
//!    case type — i.e. [`crate::mining::ocel_flatten_loss_court::flattening_lost_items`]
//!    is empty.
//! 2. `DataQualityRefusal` is `#[non_exhaustive]`, so compat refusals are
//!    folded into stable typed codes (same discipline as the row-40 court's
//!    `refusal_to_msg`) carried on [`crate::mining::MiningError::DataQuality`].
//!
//! Falsifier court: ungrounded log -> refusal precedes fitness; grounded log ->
//! fitness flows; dropping the preflight (the mutation) flips the court — the
//! row-40 alignment path alone happily emits a fitness from the same severed
//! log, which the test witnesses explicitly.

use wasm4pm_compat::data_quality::{DataQualityClaim, DataQualityDimension, DataQualityRefusal};
use wasm4pm_compat::ocel::OCEL;
use wasm4pm_compat::petri::PetriNet;

use crate::mining::ocel_alignment_court::GroundedFitness;
use crate::mining::MiningError;
use crate::types::AdmittedReceipt;

/// Stable typed refusal code: the data-quality preflight refused the log (or
/// the claim shape) — no fitness may follow.
pub const REFUSED_DATA_QUALITY_UNGROUNDED: &str = "REFUSED_DATA_QUALITY_UNGROUNDED";

/// Map a compat [`DataQualityRefusal`] to its stable, typed code string.
fn refusal_code(r: &DataQualityRefusal) -> &'static str {
    match r {
        DataQualityRefusal::VacuousQualityClaim => "VACUOUS_QUALITY_CLAIM",
        DataQualityRefusal::UngroundedQualityClaim => "UNGROUNDED_QUALITY_CLAIM",
        // The compat refusal enum is #[non_exhaustive]; fold future variants
        // into an equally typed code rather than panicking.
        _ => "QUALITY_CLAIM_UNSPECIFIED",
    }
}

/// The data-quality grounding preflight over the OCEL projection.
///
/// Sequence (each step precedes any fitness):
///
/// 1. `claim.admit_flat()` — the compat shape gate (typed compat law).
/// 2. `claim.is_grounded()` — asserted conjunctively with (1) so the two compat
///    seams named in the register row are both exercised.
/// 3. Structural dimension grounding: a claim covering
///    [`DataQualityDimension::Completeness`] over a log with severed case
///    links (events referencing no object of `case_type`, per the row-39 loss
///    traversal) is ungrounded — `REFUSED_DATA_QUALITY_UNGROUNDED` precedes
///    any fitness.
pub fn preflight_data_quality(
    ocel: &OCEL,
    case_type: &str,
    claim: &DataQualityClaim,
) -> Result<(), MiningError> {
    // 1+2. The compat shape gate, both seams.
    if let Err(refusal) = claim.admit_flat() {
        return Err(MiningError::DataQuality(format!(
            "{REFUSED_DATA_QUALITY_UNGROUNDED}: {} ({})",
            refusal_code(&refusal),
            refusal
        )));
    }
    debug_assert!(
        claim.is_grounded() == claim.admit_flat().is_ok(),
        "compat is_grounded must track admit_flat on the shape gate"
    );
    if !claim.is_grounded() {
        return Err(MiningError::DataQuality(format!(
            "{REFUSED_DATA_QUALITY_UNGROUNDED}: QUALITY_CLAIM_UNGROUNDED"
        )));
    }

    // 3. Structural dimension grounding.
    if claim
        .dimensions
        .contains(&DataQualityDimension::Completeness)
    {
        let severed =
            crate::mining::ocel_flatten_loss_court::flattening_lost_items(ocel, case_type);
        if !severed.is_empty() {
            return Err(MiningError::DataQuality(format!(
                "{REFUSED_DATA_QUALITY_UNGROUNDED}: SEVERED_CASE_LINKS(count={}, first={})",
                severed.len(),
                severed[0]
            )));
        }
    }
    Ok(())
}

/// Derive the data-quality claim grounded to the exact evidence subject:
/// claimed `dimensions` against `log_ref` = the receipt's BLAKE3 chain hash.
pub fn derive_data_quality_claim(
    chain_hash: &str,
    dimensions: Vec<DataQualityDimension>,
) -> DataQualityClaim {
    DataQualityClaim::new(
        dimensions,
        format!("affidavit:ocel:chain-hash:{chain_hash}"),
    )
}

/// Full gated path: data-quality preflight -> row-40 alignment court.
///
/// The preflight runs BEFORE `GroundedFitness::emit`; an ungrounded log or
/// claim yields [`REFUSED_DATA_QUALITY_UNGROUNDED`] and no fitness value is
/// ever constructed.
pub fn grounded_alignment_fitness(
    admitted: &AdmittedReceipt,
    model: &PetriNet,
    ocel: &OCEL,
    case_type: &str,
    claim: &DataQualityClaim,
) -> Result<GroundedFitness, MiningError> {
    preflight_data_quality(ocel, case_type, claim)?;
    crate::mining::ocel_alignment_court::grounded_alignment_fitness(admitted, model)
}

/// Mutation seam for the falsifier court: the row-40 alignment path WITHOUT
/// the data-quality preflight (the exact mutation whose effect the court
/// proves). Deliberately `pub` and named, never called outside tests.
pub fn fitness_without_preflight(
    admitted: &AdmittedReceipt,
    model: &PetriNet,
) -> Result<GroundedFitness, MiningError> {
    crate::mining::ocel_alignment_court::grounded_alignment_fitness(admitted, model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::admit;
    use crate::chain::ChainAssembler;
    use crate::mining::ocel_flatten_loss_court::fixture_ocel;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::types::AdmittedReceipt;
    use wasm4pm_compat::petri::{Arc, Marking, Place, Transition};

    fn admitted_receipt(event_types: &[&str], objects: &[(&str, &str)]) -> AdmittedReceipt {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let objs: Vec<_> = objects
            .iter()
            .map(|(id, ty)| object_ref(*id, *ty))
            .collect();
        for (i, ty) in event_types.iter().enumerate() {
            let payload = format!("p{i}");
            asm.append(build_event(*ty, objs.clone(), payload.as_bytes(), &mut counter).unwrap())
                .unwrap();
        }
        admit(asm.finalize()).expect("fixture receipt must admit")
    }

    fn two_transition_net() -> PetriNet {
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

    fn full_completeness_claim(admitted: &AdmittedReceipt) -> DataQualityClaim {
        derive_data_quality_claim(
            &format!("{}", admitted.value.chain_hash),
            vec![DataQualityDimension::Completeness],
        )
    }

    #[test]
    fn grounded_log_and_claim_let_fitness_flow() {
        let admitted = admitted_receipt(&["create", "update"], &[("o1", "order")]);
        let ocel = fixture_ocel(&admitted.value).expect("fixture ocel");
        let net = two_transition_net();
        let claim = full_completeness_claim(&admitted);
        let gf = grounded_alignment_fitness(&admitted, &net, &ocel, "order", &claim)
            .expect("grounded log + grounded claim must emit fitness");
        assert_eq!(gf.fitness, 1.0);
        assert_eq!(gf.interpretation, "perfect-fit");
    }

    #[test]
    fn ungrounded_log_refuses_BEFORE_any_fitness_and_the_mutation_flips_the_court() {
        let admitted = admitted_receipt(&["create", "update"], &[("o1", "order")]);
        let mut ocel = fixture_ocel(&admitted.value).expect("fixture ocel");
        // Sever every case link: evt-0 loses its order relationship.
        for ev in &mut ocel.events {
            ev.relationships.retain(|r| r.object_id != "o1");
        }
        let net = two_transition_net();
        let claim = full_completeness_claim(&admitted);

        // The mutation (row-40 path alone, no preflight) WOULD emit a fitness
        // from this severed log — the flip witness.
        let mutated = fitness_without_preflight(&admitted, &net)
            .expect("mutation must emit — that is the defect the gate closes");
        assert_eq!(mutated.fitness, 1.0);

        // The gated path refuses typed, BEFORE any fitness.
        let err = grounded_alignment_fitness(&admitted, &net, &ocel, "order", &claim)
            .expect_err("ungrounded log must refuse");
        let msg = format!("{err}");
        assert!(
            msg.contains("REFUSED_DATA_QUALITY_UNGROUNDED") && msg.contains("SEVERED_CASE_LINKS"),
            "expected typed severed-links refusal, got: {msg}"
        );
    }

    #[test]
    fn compat_shape_gate_fires_before_structural_and_before_fitness() {
        let admitted = admitted_receipt(&["create", "update"], &[("o1", "order")]);
        let ocel = fixture_ocel(&admitted.value).expect("fixture ocel");
        let net = two_transition_net();

        // Ungrounded claim shape: empty log_ref -> compat UngroundedQualityClaim.
        let ungrounded = DataQualityClaim::new(vec![DataQualityDimension::Completeness], "");
        let err = grounded_alignment_fitness(&admitted, &net, &ocel, "order", &ungrounded)
            .expect_err("ungrounded claim must refuse");
        let msg = format!("{err}");
        assert!(
            msg.contains("REFUSED_DATA_QUALITY_UNGROUNDED")
                && msg.contains("UNGROUNDED_QUALITY_CLAIM"),
            "expected typed UNGROUNDED_QUALITY_CLAIM, got: {msg}"
        );

        // Vacuous claim shape: zero dimensions -> compat VacuousQualityClaim.
        let vacuous = DataQualityClaim::new(vec![], "affidavit:ocel:chain-hash:x");
        let err = grounded_alignment_fitness(&admitted, &net, &ocel, "order", &vacuous)
            .expect_err("vacuous claim must refuse");
        let msg = format!("{err}");
        assert!(
            msg.contains("VACUOUS_QUALITY_CLAIM"),
            "expected typed VACUOUS_QUALITY_CLAIM, got: {msg}"
        );
    }

    #[test]
    fn non_completeness_dimension_does_not_trigger_structural_gate() {
        // Correctness-only claim over the severed log: the structural severed-
        // links gate is Completeness-scoped, so this path admits (and the
        // compat shape gate passes); the fitness flows.
        let admitted = admitted_receipt(&["create", "update"], &[("o1", "order")]);
        let mut ocel = fixture_ocel(&admitted.value).expect("fixture ocel");
        for ev in &mut ocel.events {
            ev.relationships.retain(|r| r.object_id != "o1");
        }
        let net = two_transition_net();
        let claim = derive_data_quality_claim(
            &format!("{}", admitted.value.chain_hash),
            vec![DataQualityDimension::Correctness],
        );
        let gf = grounded_alignment_fitness(&admitted, &net, &ocel, "order", &claim)
            .expect("correctness claim over severed log is not the gated dimension");
        assert_eq!(gf.fitness, 1.0);
    }
}
