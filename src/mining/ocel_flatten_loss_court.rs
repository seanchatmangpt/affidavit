//! Capability register row #39 — flattening-loss accounting court.
//!
//! Binds `wasm4pm_compat::ocel::flatten::flatten` and
//! `wasm4pm_compat::loss::{LossChain, LossReport}` into the Receipt→OCEL
//! pipeline: a flattened projection carries a [`LossChain`]; a projection with
//! a non-lossless chain is refused typed `REFUSED_FLATTENING_LOSS` unless each
//! lost item is explicitly admitted.
//!
//! Falsifier: sever one E2O relationship in a fixture log → `into_lost()`
//! contents change → verdict flips ACCEPT→refusal.

use std::collections::HashSet;

use wasm4pm_compat::loss::{LossChain, LossPolicy, LossReport, ProjectionName};
use wasm4pm_compat::ocel::flatten::{flatten, FlatLog};
use wasm4pm_compat::ocel::{OCELType, OCEL};

use crate::mining::MiningError;
use crate::types::Receipt;

/// Stable typed refusal code emitted when flattening loss is not admitted.
pub const REFUSED_FLATTENING_LOSS: &str = "REFUSED_FLATTENING_LOSS";

/// Stable projection name for the Receipt→OCEL→flat projection boundary.
pub const FLATTEN_PROJECTION: ProjectionName = ProjectionName("affidavit:receipt→ocel→flat");

/// Shape marker: object-centric event log side of the projection boundary.
pub enum OcelShape {}
/// Shape marker: flattened (classical, single-case) log side.
pub enum XesShape {}

/// The verdict of the flattening-loss admission court.
#[derive(Debug)]
pub enum FlattenLossVerdict {
    /// The projection is admitted. Carries the flattened log and its loss chain.
    Accept {
        /// The flattened projection of the log onto the case type.
        flat: FlatLog,
        /// Every loss step recorded for this projection (empty ⇒ lossless).
        chain: LossChain,
    },
    /// The projection is refused: unadmitted flattening loss.
    Refused {
        /// Always [`REFUSED_FLATTENING_LOSS`].
        code: &'static str,
        /// Event ids dropped by the flattening (the loss items).
        lost: Vec<String>,
        /// The subset of `lost` the caller did not explicitly admit.
        unadmitted: Vec<String>,
    },
}

/// Derive the concrete loss record for flattening `ocel` to `case_type`:
/// the event ids that reference no object of the case type (the divergence
/// boundary that `flatten` drops). Computed from the log itself, never from a
/// caller-supplied loss claim.
pub fn flattening_lost_items(ocel: &OCEL, case_type: &str) -> Vec<String> {
    ocel.events
        .iter()
        .filter(|e| {
            !e.relationships.iter().any(|r| {
                ocel.objects
                    .iter()
                    .any(|o| o.id == r.object_id && o.object_type == case_type)
            })
        })
        .map(|e| e.id.clone())
        .collect()
}

/// Canonical deterministic Receipt→OCEL fixture substrate.
///
/// Deviations from the naive `receipt_to_ocel` output (disclosed):
/// 1. The compat `OCELEvent::new` stamps `chrono::Utc::now()` — a wall clock —
///    which would make the flatten ordering (`(time, event id)`)
///    nondeterministic. Event times are rewritten to `seq` seconds past the
///    Unix epoch (deserialized via serde_json because `chrono` is not a direct
///    dependency under the `core` feature).
/// 2. `OCEL::new` leaves `object_types` empty, but `flatten` refuses an
///    undeclared case type; the object types present in the log are declared
///    explicitly.
pub fn fixture_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> {
    let mut ocel = crate::mining::receipt_to_ocel(receipt)?;
    for ev in &mut ocel.events {
        let seq: u64 = ev
            .id
            .rsplit('-')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let (mm, ss) = (seq / 60, seq % 60);
        // Type of `ev.time` (chrono DateTime<FixedOffset>) is inferred from the
        // assignment target; chrono is not nameable under `core`.
        ev.time = serde_json::from_str(&format!("\"1970-01-01T00:{mm:02}:{ss:02}Z\""))
            .map_err(|e| MiningError::OcelConversion(format!("deterministic time: {e}")))?;
    }
    let mut types: Vec<OCELType> = Vec::new();
    for o in &ocel.objects {
        if !types.iter().any(|t| t.name == o.object_type) {
            types.push(OCELType {
                name: o.object_type.clone(),
                attributes: Vec::new(),
            });
        }
    }
    ocel.object_types = types;
    Ok(ocel)
}

/// Run the flattening-loss court over `ocel` flattened to `case_type`.
///
/// `admitted_losses` is the explicit admission set: every dropped event id
/// must appear here or the projection is refused `REFUSED_FLATTENING_LOSS`.
pub fn admit_flattened_projection(
    ocel: &OCEL,
    case_type: &str,
    admitted_losses: &HashSet<String>,
) -> FlattenLossVerdict {
    // Undeclared case type: the compat flatten would error; refuse typed
    // instead of panicking or laundering the error into a loss verdict.
    if !ocel.object_types.iter().any(|t| t.name == case_type) {
        return FlattenLossVerdict::Refused {
            code: REFUSED_FLATTENING_LOSS,
            lost: Vec::new(),
            unadmitted: vec![format!("undeclared case type: {case_type}")],
        };
    }

    let lost = flattening_lost_items(ocel, case_type);

    if lost.is_empty() {
        // Vacuously lossless: valid even under RefuseLoss.
        let report = LossReport::<OcelShape, XesShape, Vec<String>>::new(
            FLATTEN_PROJECTION,
            LossPolicy::AllowLossWithReport,
            Vec::new(),
        );
        debug_assert!(report.is_lossless());
        return match flatten(ocel, case_type) {
            Ok(flat) => FlattenLossVerdict::Accept {
                flat,
                chain: LossChain::new(),
            },
            Err(reason) => FlattenLossVerdict::Refused {
                code: REFUSED_FLATTENING_LOSS,
                lost: vec![reason.clone()],
                unadmitted: vec![reason],
            },
        };
    }

    let unadmitted: Vec<String> = lost
        .iter()
        .filter(|id| !admitted_losses.contains(*id))
        .cloned()
        .collect();

    if !unadmitted.is_empty() {
        return FlattenLossVerdict::Refused {
            code: REFUSED_FLATTENING_LOSS,
            lost,
            unadmitted,
        };
    }

    // Every lost item explicitly admitted: the projection proceeds under
    // AllowLossWithReport with an itemized LossReport folded into a LossChain.
    let report = LossReport::<OcelShape, XesShape, Vec<String>>::new(
        FLATTEN_PROJECTION,
        LossPolicy::AllowLossWithReport,
        lost.clone(),
    );
    debug_assert!(!report.is_lossless());
    let mut chain = LossChain::new();
    chain.push(report.summary("SeveredE2OArc"));
    match flatten(ocel, case_type) {
        Ok(flat) => FlattenLossVerdict::Accept { flat, chain },
        Err(reason) => FlattenLossVerdict::Refused {
            code: REFUSED_FLATTENING_LOSS,
            lost: vec![reason.clone()],
            unadmitted: vec![reason],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::admit;
    use crate::chain::ChainAssembler;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::types::AdmittedReceipt;

    /// evt-0 references {o1(order), i1(item)}; evt-1 references {o1(order)};
    /// evt-2 references {i1(item)} only.
    fn admit_fixture() -> AdmittedReceipt {
        let mut asm = ChainAssembler::new();
        let mut counter = SeqCounter::new();
        asm.append(
            build_event(
                "create",
                vec![object_ref("o1", "order"), object_ref("i1", "item")],
                b"1",
                &mut counter,
            )
            .unwrap(),
        )
        .unwrap();
        asm.append(
            build_event(
                "update",
                vec![object_ref("o1", "order")],
                b"2",
                &mut counter,
            )
            .unwrap(),
        )
        .unwrap();
        asm.append(
            build_event("ship", vec![object_ref("i1", "item")], b"3", &mut counter).unwrap(),
        )
        .unwrap();
        admit(asm.finalize()).expect("fixture receipt must admit")
    }

    #[test]
    fn lossy_but_fully_admitted_projection_is_accepted_with_itemized_chain() {
        let ocel = fixture_ocel(&admit_fixture().value).expect("fixture ocel");
        let mut admitted = HashSet::new();
        admitted.insert("evt-2".to_string());
        match admit_flattened_projection(&ocel, "order", &admitted) {
            FlattenLossVerdict::Accept { flat, chain } => {
                assert_eq!(flat.cases.len(), 1, "one order object -> one case");
                assert_eq!(flat.cases[0].trace, vec!["create", "update"]);
                assert_eq!(chain.len(), 1, "one admitted loss step is chained");
                assert!(!chain.is_lossless());
                assert_eq!(chain.steps()[0].category(), "SeveredE2OArc");
            }
            other => panic!("expected ACCEPT, got {other:?}"),
        }
    }

    #[test]
    fn severing_one_e2o_arc_changes_loss_contents_and_flips_verdict() {
        let mut ocel = fixture_ocel(&admit_fixture().value).expect("fixture ocel");
        let baseline_lost = flattening_lost_items(&ocel, "order");
        assert_eq!(baseline_lost, vec!["evt-2".to_string()]);

        // Sever: evt-0 loses its order relationship.
        for ev in &mut ocel.events {
            if ev.id == "evt-0" {
                ev.relationships.retain(|r| r.object_id != "o1");
            }
        }
        let severed_lost = flattening_lost_items(&ocel, "order");
        assert_ne!(
            baseline_lost, severed_lost,
            "severing one E2O relationship must change the loss contents"
        );
        assert!(severed_lost.contains(&"evt-0".to_string()));

        // Empty admission set -> typed refusal (the flip).
        match admit_flattened_projection(&ocel, "order", &HashSet::new()) {
            FlattenLossVerdict::Refused {
                code,
                lost,
                unadmitted,
            } => {
                assert_eq!(code, REFUSED_FLATTENING_LOSS);
                assert_eq!(lost, severed_lost);
                assert_eq!(unadmitted, severed_lost);
            }
            other => panic!("expected REFUSED_FLATTENING_LOSS, got {other:?}"),
        }
    }

    #[test]
    fn every_lost_item_admitted_is_accepted() {
        let mut ocel = fixture_ocel(&admit_fixture().value).expect("fixture ocel");
        for ev in &mut ocel.events {
            if ev.id == "evt-0" {
                ev.relationships.retain(|r| r.object_id != "o1");
            }
        }
        let severed_lost = flattening_lost_items(&ocel, "order");
        let admitted: HashSet<String> = severed_lost.iter().cloned().collect();
        match admit_flattened_projection(&ocel, "order", &admitted) {
            FlattenLossVerdict::Accept { flat, chain } => {
                assert_eq!(flat.cases.len(), 1);
                assert_eq!(chain.len(), 1);
            }
            other => panic!("expected ACCEPT with all losses admitted, got {other:?}"),
        }
    }

    #[test]
    fn undeclared_case_type_is_typed_refusal_not_panic() {
        let ocel = fixture_ocel(&admit_fixture().value).expect("fixture ocel");
        match admit_flattened_projection(&ocel, "invoice", &HashSet::new()) {
            FlattenLossVerdict::Refused { code, .. } => {
                assert_eq!(code, REFUSED_FLATTENING_LOSS);
            }
            other => panic!("expected typed refusal, got {other:?}"),
        }
    }

    #[test]
    fn loss_chain_via_into_lost_round_trip() {
        // into_lost() contents change when an E2O arc is severed (register
        // falsifier phrasing): the LossReport's lost items ARE the severed
        // events. Exercise the into_lost seam directly.
        let lost = vec!["evt-2".to_string()];
        let report = LossReport::<OcelShape, XesShape, Vec<String>>::new(
            FLATTEN_PROJECTION,
            LossPolicy::AllowLossWithReport,
            lost.clone(),
        );
        assert_eq!(report.into_lost(), lost);
    }
}
