//! Capability register row #10 — OCEL conformance court over compat DFG
//! fitness/precision with `admit_flat` grounded claims.
//!
//! Pipeline (fail-closed):
//! 1. Build an [`ObjectCentricConformanceClaim`] scoped to the case type and
//!    claiming fitness+precision.
//! 2. `admit_flat()` must pass — an ungrounded claim never yields a number.
//! 3. Discover the observed DFG via `wasm4pm_compat::dfg::discover_ocel_dfg`.
//! 4. Emit `dfg_fitness` / `dfg_precision` against the normative arc set.
//!
//! Falsifier: mutate one activity in the normative arcs → fitness flips;
//! empty `log_ref` / empty object-type scope → typed refusal, no scores.

use wasm4pm_compat::dfg::{dfg_fitness, dfg_precision, discover_ocel_dfg};
use wasm4pm_compat::interop::ConformanceTriple;
use wasm4pm_compat::object_centric_conformance::{
    ObjectCentricConformanceClaim, ObjectCentricConformanceRefusal, ObjectTypeConformance,
};
use wasm4pm_compat::ocel::OCEL;

use crate::mining::MiningError;

/// Stable typed refusal when the object-centric claim fails `admit_flat`.
pub const REFUSED_OCEL_CONFORMANCE_UNGROUNDED: &str = "REFUSED_OCEL_CONFORMANCE_UNGROUNDED";

/// Grounded DFG fitness/precision scores for one object type.
#[derive(Debug, Clone, PartialEq)]
pub struct GroundedDfgConformance {
    /// Object type the claim was scoped to.
    pub object_type: String,
    /// Opaque log reference the claim was grounded against.
    pub log_ref: String,
    /// Fraction of normative arcs present in the observed DFG.
    pub fitness: f64,
    /// Fraction of observed DFG arcs that appear in the normative model.
    pub precision: f64,
}

fn refusal_msg(r: ObjectCentricConformanceRefusal) -> String {
    let law = match r {
        ObjectCentricConformanceRefusal::UngroundedClaim => "UngroundedClaim",
        ObjectCentricConformanceRefusal::NoObjectTypesScoped => "NoObjectTypesScoped",
        ObjectCentricConformanceRefusal::UnscopedObjectType => "UnscopedObjectType",
        _ => "Unspecified",
    };
    format!("{REFUSED_OCEL_CONFORMANCE_UNGROUNDED}: {law}")
}

/// Admit the object-centric fitness+precision claim, then compute DFG scores.
///
/// `normative_arcs` is the reference model `(source, target)` list. An empty
/// normative set yields fitness 1.0 by the compat law; that is only reachable
/// after `admit_flat` succeeds.
pub fn grounded_dfg_conformance(
    ocel: &OCEL,
    object_type: &str,
    log_ref: &str,
    normative_arcs: &[(String, String)],
) -> Result<GroundedDfgConformance, MiningError> {
    let claim = ObjectCentricConformanceClaim::new(
        vec![ObjectTypeConformance {
            object_type: object_type.to_owned(),
            triple: ConformanceTriple::fitness_and_precision(),
        }],
        log_ref,
    );
    if let Err(r) = claim.admit_flat() {
        return Err(MiningError::Alignment(refusal_msg(r)));
    }

    let observed = discover_ocel_dfg(ocel);
    let fitness = dfg_fitness(&observed, normative_arcs);
    let precision = dfg_precision(&observed, normative_arcs);
    Ok(GroundedDfgConformance {
        object_type: object_type.to_owned(),
        log_ref: log_ref.to_owned(),
        fitness,
        precision,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::admit;
    use crate::chain::ChainAssembler;
    use crate::mining::ocel_flatten_loss_court::fixture_ocel;
    use crate::ocel::{build_event, object_ref, SeqCounter};

    fn fixture_ocel_log() -> OCEL {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for (act, objs) in [
            ("create", vec![object_ref("o1", "order")]),
            ("pay", vec![object_ref("o1", "order")]),
            ("ship", vec![object_ref("o1", "order")]),
        ] {
            asm.append(build_event(act, objs, act.as_bytes(), &mut counter).unwrap())
                .unwrap();
        }
        let admitted = admit(asm.finalize()).expect("admit");
        fixture_ocel(&admitted.value).expect("fixture ocel")
    }

    #[test]
    fn grounded_claim_emits_fitness_and_precision() {
        let ocel = fixture_ocel_log();
        let normative = vec![
            ("create".into(), "pay".into()),
            ("pay".into(), "ship".into()),
        ];
        let g = grounded_dfg_conformance(&ocel, "order", "log:fixture", &normative)
            .expect("grounded claim must emit scores");
        assert_eq!(g.fitness, 1.0);
        assert_eq!(g.precision, 1.0);
        assert_eq!(g.object_type, "order");
    }

    #[test]
    fn activity_mutation_flips_fitness() {
        let ocel = fixture_ocel_log();
        let honest = vec![
            ("create".into(), "pay".into()),
            ("pay".into(), "ship".into()),
        ];
        let mutated = vec![
            ("create".into(), "pay".into()),
            ("pay".into(), "CANCEL".into()), // mutated target
        ];
        let a = grounded_dfg_conformance(&ocel, "order", "log:fixture", &honest).unwrap();
        let b = grounded_dfg_conformance(&ocel, "order", "log:fixture", &mutated).unwrap();
        assert_eq!(a.fitness, 1.0);
        assert!(
            b.fitness < 1.0,
            "mutation must flip fitness; got {}",
            b.fitness
        );
        assert_ne!(a.fitness, b.fitness);
    }

    #[test]
    fn ungrounded_claim_refuses_before_any_score() {
        let ocel = fixture_ocel_log();
        let normative = vec![("create".into(), "pay".into())];
        let err = grounded_dfg_conformance(&ocel, "order", "", &normative)
            .expect_err("empty log_ref must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains(REFUSED_OCEL_CONFORMANCE_UNGROUNDED),
            "expected typed refusal, got {msg}"
        );
    }
}
