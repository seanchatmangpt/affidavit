//! Capability register row #94 — DECLARE constraint temper court.
//!
//! Compat `DeclareConstraint::validate` only checks *shape*. This court adds
//! behavioural checks over a flat activity sequence projected from an OCEL
//! case for the core templates needed for proof-carrying OCEL v2 conformance:
//!
//! Unary: Existence, Absence, Init
//! Binary: Response, Precedence, Succession, CoExistence, RespondedExistence,
//!         NotCoExistence, ExclusiveChoice, NotSuccession
//!
//! Satisfying log → ACCEPT; temper → `REFUSED_DECLARE_VIOLATION`.
//! Falsifier: a violation that ACCEPTs fails the court.

use std::collections::HashSet;

use wasm4pm_compat::declare::{Activity, DeclareConstraint, DeclareScope, DeclareTemplate};
use wasm4pm_compat::ocel::OCEL;

use crate::mining::MiningError;

/// Typed refusal when a DECLARE behavioural law is violated.
pub const REFUSED_DECLARE_VIOLATION: &str = "REFUSED_DECLARE_VIOLATION";

/// Typed refusal when the constraint fails structural `validate()`.
pub const REFUSED_DECLARE_MALFORMED: &str = "REFUSED_DECLARE_MALFORMED";

/// Typed refusal when the template has no behavioural evaluator yet.
pub const REFUSED_DECLARE_UNSUPPORTED: &str = "REFUSED_DECLARE_UNSUPPORTED";

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

fn holds(template: DeclareTemplate, acts: &[String], a: &str, b: Option<&str>) -> bool {
    let set: HashSet<&str> = acts.iter().map(String::as_str).collect();
    match template {
        DeclareTemplate::Existence => set.contains(a),
        DeclareTemplate::Absence => !set.contains(a),
        DeclareTemplate::Init => acts.first().map(String::as_str) == Some(a),
        DeclareTemplate::NotCoExistence => {
            let b = b.expect("binary");
            !(set.contains(a) && set.contains(b))
        }
        DeclareTemplate::CoExistence => {
            let b = b.expect("binary");
            set.contains(a) == set.contains(b)
        }
        DeclareTemplate::RespondedExistence => {
            let b = b.expect("binary");
            !set.contains(a) || set.contains(b)
        }
        DeclareTemplate::ExclusiveChoice => {
            let b = b.expect("binary");
            set.contains(a) ^ set.contains(b)
        }
        DeclareTemplate::Response => {
            let b = b.expect("binary");
            // Every A must be followed later by a B.
            let mut pending = 0i32;
            for act in acts {
                if act == a {
                    pending += 1;
                } else if act == b && pending > 0 {
                    pending -= 1;
                }
            }
            pending == 0
        }
        DeclareTemplate::Precedence => {
            let b = b.expect("binary");
            // Every B must be preceded earlier by an A.
            let mut seen_a = false;
            for act in acts {
                if act == a {
                    seen_a = true;
                } else if act == b && !seen_a {
                    return false;
                }
            }
            true
        }
        DeclareTemplate::Succession => {
            holds(DeclareTemplate::Response, acts, a, b)
                && holds(DeclareTemplate::Precedence, acts, a, b)
        }
        DeclareTemplate::NotSuccession => {
            let b = b.expect("binary");
            // No A followed later by a B.
            let mut seen_a = false;
            for act in acts {
                if act == a {
                    seen_a = true;
                } else if act == b && seen_a {
                    return false;
                }
            }
            true
        }
        _ => false, // caller maps to REFUSED_DECLARE_UNSUPPORTED
    }
}

fn supported(template: DeclareTemplate) -> bool {
    matches!(
        template,
        DeclareTemplate::Existence
            | DeclareTemplate::Absence
            | DeclareTemplate::Init
            | DeclareTemplate::NotCoExistence
            | DeclareTemplate::CoExistence
            | DeclareTemplate::RespondedExistence
            | DeclareTemplate::ExclusiveChoice
            | DeclareTemplate::Response
            | DeclareTemplate::Precedence
            | DeclareTemplate::Succession
            | DeclareTemplate::NotSuccession
    )
}

/// Admit a DECLARE constraint against one OCEL case, or refuse typed.
pub fn admit_declare(
    ocel: &OCEL,
    object_id: &str,
    template: DeclareTemplate,
    activation: &str,
    target: Option<&str>,
    scope_type: &str,
) -> Result<(), MiningError> {
    if !supported(template) {
        return Err(MiningError::Alignment(format!(
            "{REFUSED_DECLARE_UNSUPPORTED}: {template:?} has no behavioural evaluator yet"
        )));
    }
    let constraint = if template.arity() == 1 {
        DeclareConstraint::unary(
            template,
            Activity::new(activation),
            DeclareScope::SingleObjectScope(scope_type.into()),
        )
    } else {
        let t = target.ok_or_else(|| {
            MiningError::Alignment(format!(
                "{REFUSED_DECLARE_MALFORMED}: binary template requires target"
            ))
        })?;
        DeclareConstraint::binary(
            template,
            Activity::new(activation),
            Activity::new(t),
            DeclareScope::SingleObjectScope(scope_type.into()),
        )
    };
    if let Err(r) = constraint.validate() {
        return Err(MiningError::Alignment(format!(
            "{REFUSED_DECLARE_MALFORMED}: {r}"
        )));
    }
    let acts = case_activities(ocel, object_id);
    if holds(template, &acts, activation, target) {
        Ok(())
    } else {
        Err(MiningError::Alignment(format!(
            "{REFUSED_DECLARE_VIOLATION}: {template:?}({activation},{}) violated on case {object_id}",
            target.unwrap_or("-")
        )))
    }
}

/// Admit a NotCoExistence constraint (row 94 first template; kept as a named seam).
pub fn admit_not_coexistence(
    ocel: &OCEL,
    object_id: &str,
    activation: &str,
    target: &str,
    scope_type: &str,
) -> Result<(), MiningError> {
    admit_declare(
        ocel,
        object_id,
        DeclareTemplate::NotCoExistence,
        activation,
        Some(target),
        scope_type,
    )
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

    #[test]
    fn response_accepts_and_temper_refuses() {
        let ok = ocel_with(&["create", "approve", "ship"]);
        admit_declare(
            &ok,
            "o1",
            DeclareTemplate::Response,
            "create",
            Some("approve"),
            "order",
        )
        .expect("create followed by approve");
        let bad = ocel_with(&["create", "ship"]);
        let err = admit_declare(
            &bad,
            "o1",
            DeclareTemplate::Response,
            "create",
            Some("approve"),
            "order",
        )
        .expect_err("missing approve must refuse");
        assert!(err.to_string().contains(REFUSED_DECLARE_VIOLATION));
    }

    #[test]
    fn precedence_accepts_and_temper_refuses() {
        let ok = ocel_with(&["pick", "ship"]);
        admit_declare(
            &ok,
            "o1",
            DeclareTemplate::Precedence,
            "pick",
            Some("ship"),
            "order",
        )
        .unwrap();
        let bad = ocel_with(&["ship", "pick"]);
        assert!(admit_declare(
            &bad,
            "o1",
            DeclareTemplate::Precedence,
            "pick",
            Some("ship"),
            "order",
        )
        .is_err());
    }

    #[test]
    fn succession_is_response_and_precedence() {
        let ok = ocel_with(&["ship", "deliver"]);
        admit_declare(
            &ok,
            "o1",
            DeclareTemplate::Succession,
            "ship",
            Some("deliver"),
            "order",
        )
        .unwrap();
        let only_response_broken = ocel_with(&["deliver", "ship"]);
        assert!(admit_declare(
            &only_response_broken,
            "o1",
            DeclareTemplate::Succession,
            "ship",
            Some("deliver"),
            "order",
        )
        .is_err());
    }

    #[test]
    fn existence_absence_init_unary_courts() {
        let ocel = ocel_with(&["create", "pay"]);
        admit_declare(
            &ocel,
            "o1",
            DeclareTemplate::Existence,
            "create",
            None,
            "order",
        )
        .unwrap();
        admit_declare(
            &ocel,
            "o1",
            DeclareTemplate::Absence,
            "CANCEL",
            None,
            "order",
        )
        .unwrap();
        admit_declare(&ocel, "o1", DeclareTemplate::Init, "create", None, "order").unwrap();
        assert!(admit_declare(
            &ocel,
            "o1",
            DeclareTemplate::Init,
            "pay",
            None,
            "order",
        )
        .is_err());
        assert!(admit_declare(
            &ocel,
            "o1",
            DeclareTemplate::Absence,
            "create",
            None,
            "order",
        )
        .is_err());
    }

    #[test]
    fn coexistence_and_exclusive_choice() {
        let both = ocel_with(&["create", "pay"]);
        admit_declare(
            &both,
            "o1",
            DeclareTemplate::CoExistence,
            "create",
            Some("pay"),
            "order",
        )
        .unwrap();
        assert!(admit_declare(
            &both,
            "o1",
            DeclareTemplate::ExclusiveChoice,
            "create",
            Some("pay"),
            "order",
        )
        .is_err());
        let one = ocel_with(&["create"]);
        admit_declare(
            &one,
            "o1",
            DeclareTemplate::ExclusiveChoice,
            "create",
            Some("pay"),
            "order",
        )
        .unwrap();
    }

    #[test]
    fn unsupported_template_refuses_typed() {
        let ocel = ocel_with(&["a", "b"]);
        let err = admit_declare(
            &ocel,
            "o1",
            DeclareTemplate::ChainResponse,
            "a",
            Some("b"),
            "order",
        )
        .expect_err("chain templates still OPEN");
        assert!(err.to_string().contains(REFUSED_DECLARE_UNSUPPORTED));
    }
}
