//! Admission gate: the Layer 2 seam where the structural law decides whether a
//! receipt may become `Admitted`.
//!
//! Per ARDPRD §4 (the court/producer seam) and ADR-2/ADR-3/ADR-4, the only path
//! to `Evidence<Receipt, Admitted, AffidavitReceiptChain>` runs through
//! [`admit`], which:
//!
//! 1. re-runs the structural certify pipeline ([`crate::verifier::verify`]),
//!    which checks chain integrity, continuity, commitment well-formedness, and
//!    the conformance profile, and
//! 2. mints `Admitted` **only** if the verdict is ACCEPT, otherwise returns a
//!    named [`AffidavitRefusal`].
//!
//! This is the load-bearing law of the whole project: a receipt that did not
//! pass the court has no path to the `Admitted` type. Minting `Admitted` by
//! `Admission::new(arbitrary_bytes)` — a fiat cast — is exactly the inversion
//! this module exists to forbid. The witness that the law holds is
//! [`tests::forged_receipt_cannot_be_admitted`]: a structurally-invalid receipt
//! is refused by name and never reaches `Admitted`.

use crate::types::{AdmittedReceipt, AffidavitReceiptChain, Receipt};
use wasm4pm_compat::admission::Admission;
use wasm4pm_compat::ocel::{
    EventObjectLink, Object, ObjectChange, ObjectObjectLink, OcelEvent, OcelLog, OcelRefusal,
};

/// Named refusal: why a receipt was denied admission. A refusal is a first-class
/// outcome carrying a specific reason, never a bare "invalid input" (the
/// wasm4pm-compat refusal discipline).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AffidavitRefusal {
    /// The wasm4pm-compat OCEL structural law refused the receipt's event/object
    /// graph (e.g. `EmptyEventObjectLinks`, `DanglingEventObjectLink`). This is
    /// the court's law, run from outside the producer — the receipt's own bytes
    /// cannot satisfy it without a genuine event-to-object structure.
    OcelLawViolation(OcelRefusal),
    /// The affidavit certify pipeline rejected the receipt; carries the failing
    /// stage and the verdict reason (chain integrity, continuity, commitments).
    StructuralLawViolation {
        /// The first stage that failed, for a precise refusal name.
        stage: String,
        /// The verdict's summary reason.
        reason: String,
    },
}

impl std::fmt::Display for AffidavitRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AffidavitRefusal::OcelLawViolation(r) => write!(f, "ocel_law_violation: {r}"),
            AffidavitRefusal::StructuralLawViolation { stage, reason } => {
                write!(f, "structural_law_violation[{stage}]: {reason}")
            }
        }
    }
}

impl std::error::Error for AffidavitRefusal {}

/// Project an affidavit [`Receipt`] into the wasm4pm-compat OCEL log shape so the
/// court's structural law ([`OcelLog::validate`]) can adjudicate it. Each event
/// becomes an `OcelEvent`; each distinct object ref becomes an `Object`; each
/// event→object reference becomes an `EventObjectLink`. This is the boundary
/// projection — affidavit's receipt is OCEL-shaped, so the mapping is total.
fn project_to_ocel(receipt: &Receipt) -> OcelLog {
    let mut objects: Vec<Object> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut events: Vec<OcelEvent> = Vec::new();
    let mut e2o: Vec<EventObjectLink> = Vec::new();

    for ev in &receipt.events {
        events.push(OcelEvent::new(&ev.id, &ev.event_type));
        for o in &ev.objects {
            if seen.insert(o.id.as_str()) {
                objects.push(Object::new(&o.id, &o.obj_type));
            }
            e2o.push(EventObjectLink::new(&ev.id, &o.id));
        }
    }

    OcelLog::new(
        objects,
        events,
        e2o,
        Vec::<ObjectObjectLink>::new(),
        Vec::<ObjectChange>::new(),
    )
}

/// The Layer 2 transition: `Receipt` → `Evidence<Receipt, Admitted, AffidavitReceiptChain>`.
///
/// Mints `Admitted` ONLY after BOTH courts pass, atomically (ARDPRD §4):
///
/// 1. the **wasm4pm-compat OCEL structural law** ([`OcelLog::validate`]) — the
///    receipt's event/object graph is non-empty and link-consistent, refusing
///    by name (`EmptyEventObjectLinks` / `DanglingEventObjectLink`) otherwise;
/// 2. the **affidavit certify pipeline** ([`crate::verifier::verify`]) — chain
///    integrity (recompute, not trust the stored hash), continuity, commitments.
///
/// On any refusal, `Admitted` is never produced. This is the ONLY honest path to
/// the admitted carrier; `show`/`load` deliberately do not mint it. The witnesses
/// in this module's tests fail if either court is removed — the green is false
/// when the work is faked.
pub fn admit(receipt: Receipt) -> Result<AdmittedReceipt, AffidavitRefusal> {
    // Court 1 — the wasm4pm-compat OCEL structural law. Runs from outside the
    // producer; a receipt with no event-object structure cannot satisfy it.
    project_to_ocel(&receipt)
        .validate()
        .map_err(AffidavitRefusal::OcelLawViolation)?;

    // Court 2 — affidavit's chain/continuity/commitment law. Re-runs the
    // irreproducible BLAKE3 chain rather than trusting the stored hash.
    let verdict = crate::verifier::verify(&receipt);
    if !verdict.accepted {
        let first_fail = verdict
            .outcomes
            .iter()
            .find(|o| !o.passed)
            .map(|o| o.stage.clone())
            .unwrap_or_else(|| "unknown".to_string());
        return Err(AffidavitRefusal::StructuralLawViolation {
            stage: first_fail,
            reason: verdict.reason,
        });
    }

    // Both courts passed. Mint Admitted through the wasm4pm-compat Admission
    // carrier — the single sealed point, reached ONLY after both laws held.
    Ok(Admission::<Receipt, AffidavitReceiptChain>::new(receipt).into_evidence())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocel::{build_event, object_ref, SeqCounter};
    use crate::types::{Blake3Hash, OperationEvent};

    /// Build an honest, court-passing receipt.
    fn honest_receipt() -> Receipt {
        let mut asm = crate::chain::ChainAssembler::new();
        let mut counter = SeqCounter::new();
        let event = build_event(
            "create",
            vec![object_ref("file-1", "artifact")],
            b"content",
            &mut counter,
        )
        .expect("build event");
        asm.append(event).expect("append");
        asm.finalize()
    }

    #[test]
    fn honest_receipt_is_admitted() {
        // POSITIVE control: a court-passing receipt reaches Admitted.
        let receipt = honest_receipt();
        let admitted = admit(receipt.clone());
        assert!(admitted.is_ok(), "honest receipt must be admittable");
        // The admitted carrier holds the same receipt.
        assert_eq!(admitted.unwrap().into_inner(), receipt);
    }

    #[test]
    fn empty_object_links_refused_by_ocel_court() {
        // THE dangling/empty-room witness: a CHAIN-CONSISTENT receipt whose
        // event has NO objects passes every affidavit verifier stage (chain,
        // continuity, commitments) — the producer's own checks see nothing
        // wrong. Only the wasm4pm-compat OCEL court, run from outside, refuses
        // it by name (EmptyEventObjectLinks). If admit() did not run that court,
        // this objectless receipt would be admitted — so this test fails when
        // the OCEL law is removed.
        let objectless = OperationEvent {
            id: "evt-0".to_string(),
            seq: 0,
            event_type: "create".to_string(),
            objects: vec![], // <- the empty room
            payload_commitment: Blake3Hash::from_bytes(b"content"),
        };
        let chain_hash = crate::chain::recompute_chain(std::slice::from_ref(&objectless))
            .expect("recompute chain");
        let receipt = Receipt::sealed(
            crate::chain::FORMAT_VERSION.to_string(),
            vec![objectless],
            chain_hash,
        );

        // Confirm the producer's own pipeline sees nothing wrong (the door is closed).
        assert!(
            crate::verifier::verify(&receipt).accepted,
            "the affidavit verifier alone ACCEPTS the objectless receipt — proving the OCEL court is what catches it"
        );

        // The external court refuses it by name.
        let result = admit(receipt);
        assert_eq!(
            result.err(),
            Some(AffidavitRefusal::OcelLawViolation(
                wasm4pm_compat::ocel::OcelRefusal::EmptyEventObjectLinks
            )),
            "objectless receipt MUST be refused by the OCEL court (EmptyEventObjectLinks)"
        );
    }

    #[test]
    fn forged_receipt_cannot_be_admitted() {
        // THE load-bearing witness (ARDPRD §9): a structurally-invalid receipt
        // is REFUSED by name and never reaches Admitted. If this test passes
        // whether or not admit() runs the law, the law is fake — so we forge a
        // receipt that the chain accepts byte-wise but the structural pipeline
        // must reject, and assert the refusal.
        //
        // Forge: a chain-consistent receipt whose events violate continuity
        // (seq starts at 5, not 0). chain_hash is recomputed to match, so this
        // is NOT a chain-hash-mismatch — it is a structural-law violation that
        // ONLY admit()'s verifier pass can catch.
        let forged_event = OperationEvent {
            id: "evt-5".to_string(),
            seq: 5, // illegal: continuity requires contiguous-from-0
            event_type: "create".to_string(),
            objects: vec![object_ref("file-1", "artifact")],
            payload_commitment: Blake3Hash::from_bytes(b"content"),
        };
        // Recompute a matching chain so the failure is structural, not a hash mismatch.
        let chain_hash = crate::chain::recompute_chain(std::slice::from_ref(&forged_event))
            .expect("recompute chain");
        let forged = Receipt::sealed(
            crate::chain::FORMAT_VERSION.to_string(),
            vec![forged_event],
            chain_hash,
        );

        let result = admit(forged);
        assert!(
            result.is_err(),
            "forged (continuity-violating) receipt MUST be refused — admit() must run the structural law"
        );
        match result.unwrap_err() {
            AffidavitRefusal::StructuralLawViolation { stage, .. } => {
                // The refusal names the failing stage — a first-class "no".
                assert_eq!(
                    stage, "continuity",
                    "refusal must name the continuity stage that caught the forgery"
                );
            }
            other => panic!("expected continuity StructuralLawViolation, got {other:?}"),
        }
    }
}
