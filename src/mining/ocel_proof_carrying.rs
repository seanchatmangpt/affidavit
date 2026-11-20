//! Proof-carrying OCEL v2 conformance — bind grounded DFG scores to the
//! seal/v2 subject digest.
//!
//! Affidavit certifies; it does not decide. A conformance score is only
//! proof-carrying when:
//! 1. The object-centric claim passed `admit_flat` (row #10 court).
//! 2. The emitted scores are folded into a deterministic subject digest.
//! 3. Consumers seal that digest under FIPS 204 context
//!    `affidavit-core/seal/v2` ([`affidavit_core::DOMAIN_SEAL`]).
//!
//! This module mints the seal **subject** (32-byte BLAKE3). Signing stays with
//! consumers (operator law: core has no signer role). The court proves that
//! mutating fitness/precision moves the subject digest, and that an ungrounded
//! claim never yields a seal subject.

use blake3::Hasher;

use crate::mining::ocel_compat_court::{grounded_dfg_conformance, GroundedDfgConformance};
use crate::mining::MiningError;
use wasm4pm_compat::ocel::OCEL;

/// Domain tag for the proof-carrying conformance subject pre-image.
pub const DOMAIN_OCEL_CONFORMANCE_SUBJECT: &[u8] = b"affidavit/ocel-conformance-subject/v1";

/// A seal-ready subject: grounded scores + their content address.
#[derive(Debug, Clone, PartialEq)]
pub struct ProofCarryingConformance {
    /// Grounded DFG fitness/precision (post `admit_flat`).
    pub scores: GroundedDfgConformance,
    /// BLAKE3 subject digest — feed to `seal_message` / seal/v2.
    pub subject_digest: [u8; 32],
}

/// Canonical pre-image for the conformance subject (deterministic field order).
fn subject_preimage(scores: &GroundedDfgConformance) -> Vec<u8> {
    // Fixed layout: domain || log_ref || NUL || object_type || NUL ||
    // fitness bits (LE f64) || precision bits (LE f64).
    let mut out = Vec::with_capacity(
        DOMAIN_OCEL_CONFORMANCE_SUBJECT.len()
            + scores.log_ref.len()
            + scores.object_type.len()
            + 2
            + 16,
    );
    out.extend_from_slice(DOMAIN_OCEL_CONFORMANCE_SUBJECT);
    out.extend_from_slice(scores.log_ref.as_bytes());
    out.push(0);
    out.extend_from_slice(scores.object_type.as_bytes());
    out.push(0);
    out.extend_from_slice(&scores.fitness.to_bits().to_le_bytes());
    out.extend_from_slice(&scores.precision.to_bits().to_le_bytes());
    out
}

/// Digest grounded scores into a seal/v2 subject.
pub fn subject_digest_of(scores: &GroundedDfgConformance) -> [u8; 32] {
    let mut h = Hasher::new();
    h.update(&subject_preimage(scores));
    *h.finalize().as_bytes()
}

/// Run the row-10 court, then mint the proof-carrying seal subject.
pub fn proof_carrying_dfg_conformance(
    ocel: &OCEL,
    object_type: &str,
    log_ref: &str,
    normative_arcs: &[(String, String)],
) -> Result<ProofCarryingConformance, MiningError> {
    let scores = grounded_dfg_conformance(ocel, object_type, log_ref, normative_arcs)?;
    let subject_digest = subject_digest_of(&scores);
    // Seal subject must never be the all-zero encoding (seal/v2 EmptyChainHead).
    if subject_digest == [0u8; 32] {
        return Err(MiningError::Alignment(
            "REFUSED_OCEL_SUBJECT_DEGENERATE: subject digest is all-zero".into(),
        ));
    }
    Ok(ProofCarryingConformance {
        scores,
        subject_digest,
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
        for act in ["create", "pay", "ship"] {
            asm.append(
                build_event(
                    act,
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
    fn mints_non_zero_subject_compatible_with_seal_v2() {
        let ocel = fixture_ocel_log();
        let normative = vec![
            ("create".into(), "pay".into()),
            ("pay".into(), "ship".into()),
        ];
        let pc = proof_carrying_dfg_conformance(&ocel, "order", "log:fixture", &normative)
            .expect("must mint");
        assert_ne!(pc.subject_digest, [0u8; 32]);
        // seal/v2 contract (affidavit-core): message is the raw 32-byte head;
        // context is b"affidavit-core/seal/v2". Consumers pass subject_digest
        // as M and that context as ctx to sign_deterministic.
        assert_eq!(pc.subject_digest.len(), 32);
    }

    #[test]
    fn score_mutation_moves_subject_digest() {
        let ocel = fixture_ocel_log();
        let honest = vec![
            ("create".into(), "pay".into()),
            ("pay".into(), "ship".into()),
        ];
        let mutated = vec![
            ("create".into(), "pay".into()),
            ("pay".into(), "CANCEL".into()),
        ];
        let a = proof_carrying_dfg_conformance(&ocel, "order", "log:fixture", &honest).unwrap();
        let b = proof_carrying_dfg_conformance(&ocel, "order", "log:fixture", &mutated).unwrap();
        assert_ne!(
            a.subject_digest, b.subject_digest,
            "fitness/precision mutation must move the seal subject"
        );
    }

    #[test]
    fn ungrounded_claim_never_yields_a_seal_subject() {
        let ocel = fixture_ocel_log();
        let normative = vec![("create".into(), "pay".into())];
        let err = proof_carrying_dfg_conformance(&ocel, "order", "", &normative)
            .expect_err("empty log_ref must refuse before minting a subject");
        assert!(err
            .to_string()
            .contains("REFUSED_OCEL_CONFORMANCE_UNGROUNDED"));
    }
}
