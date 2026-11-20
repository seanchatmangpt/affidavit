//! Capability register row #94 — DECLARE constraint temper court.
//!
//! Compat `DeclareConstraint::validate` only checks *shape*. This court adds
//! the behavioural check for [`DeclareTemplate::NotCoExistence`] over a flat
//! activity sequence projected from an OCEL case:
//! - satisfying log (A and B never both occur) → ACCEPT
//! - temper (both A and B present) → REFUSED_DECLARE_VIOLATION
//!
//! Falsifier: a NotCoExistence violation that ACCEPTs fails the court.

use std::collections::HashSet;

use wasm4pm_compat::declare::{Activity, DeclareConstraint, DeclareScope, DeclareTemplate};
use wasm4pm_compat::ocel::OCEL;

use crate::mining::MiningError;

/// Typed refusal when a DECLARE behavioural law is violated.
pub const REFUSED_DECLARE_VIOLATION: &str = "REFUSED_DECLARE_VIOLATION";

/// Typed refusal when the constraint fails structural `validate()`.
pub const REFUSED_DECLARE_MALFORMED: &str = "REFUSED_DECLARE_MALFORMED";

/// Project the activity sequence for one object instance from an OCEL.
fn case_activities(ocel: &OCEL, object_id: &str) -> Vec<String> {
    let mut events: Vec<(i64, String, String)> = ocel
        .events
        .iter()
        .filter(|e| e.relationships.iter().any(|r| r.object_id == object_id))
        .map(|e| {
            (
                e.time.timestamp_millis(),
                e.id.clone(),
                e.event_type.clone(),
            )
        })
        .collect();
    events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    events.into_iter().map(|(_, _, act)| act).collect()
}

/// Evaluate `NotCoExistence(activation, target)` over `activities`.
fn not_coexistence_holds(activities: &[String], a: &str, b: &str) -> bool {
    let set: HashSet<&str> = activities.iter().map(String::as_str).collect();
    !(set.contains(a) && set.contains(b))
}

/// Admit a NotCoExistence constraint against one OCEL case, or refuse typed.
pub fn admit_not_coexistence(
    ocel: &OCEL,
    object_id: &str,
    activation: &str,
    target: &str,
    scope_type: &str,
) -> Result<(), MiningError> {
    let constraint = DeclareConstraint::binary(
        DeclareTemplate::NotCoExistence,
        Activity::new(activation),
        Activity::new(target),
        DeclareScope::SingleObjectScope(scope_type.into()),
    );
    if let Err(r) = constraint.validate() {
        return Err(MiningError::Alignment(format!(
            "{REFUSED_DECLARE_MALFORMED}: {r}"
        )));
    }
    let acts = case_activities(ocel, object_id);
    if not_coexistence_holds(&acts, activation, target) {
        Ok(())
    } else {
        Err(MiningError::Alignment(format!(
            "{REFUSED_DECLARE_VIOLATION}: NotCoExistence({activation},{target}) violated on case {object_id}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::admit;
    use crate::chain::ChainAssembler;
    use crate::mining::ocel_flatten_loss_court::fixture_ocel;
    use crate::ocel::{build_event, object_ref, SeqCounter};

    fn ocel_with(acts: &[&str]) -> OCEL {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        for act in acts {
            asm.append(
                build_event(
                    *act,
                    vec![object_ref("o1", "order")],
                    act.as_bytes(),
                    &mut counter,
                )
                .unwrap(),
            )
            .unwrap();
        }
        let admitted = admit(asm.finalize()).unwrap();
        fixture_ocel(&admitted.value).unwrap()
    }

    #[test]
    fn satisfying_log_accepts_not_coexistence() {
        let ocel = ocel_with(&["create", "pay", "ship"]);
        admit_not_coexistence(&ocel, "o1", "create", "CANCEL", "order")
            .expect("create and CANCEL never co-occur");
    }

    #[test]
    fn violation_refuses_typed() {
        let ocel = ocel_with(&["create", "pay", "CANCEL"]);
        let err = admit_not_coexistence(&ocel, "o1", "create", "CANCEL", "order")
            .expect_err("both present must refuse");
        assert!(err.to_string().contains(REFUSED_DECLARE_VIOLATION));
    }

    #[test]
    fn temper_falsifier_flips_accept_to_refuse() {
        let clean = ocel_with(&["create", "pay"]);
        assert!(admit_not_coexistence(&clean, "o1", "create", "CANCEL", "order").is_ok());
        let tempered = ocel_with(&["create", "pay", "CANCEL"]);
        assert!(admit_not_coexistence(&tempered, "o1", "create", "CANCEL", "order").is_err());
    }
}
