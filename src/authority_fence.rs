//! SMT-backed authority fence: `PREPARE` → `FENCE` → `DO` with cryptographic
//! revocation checks (RFC-SA2A-004 §10/§11/§27 composition).
//!
//! The fence commits revocations into a sparse Merkle tree. Immediately
//! before `DO`, the effector requires a witness against the **current**
//! revocation-set root:
//!
//! - a revocation refuses via inclusion: the authority id provably maps to
//!   a tombstone under the live root;
//! - an admissible authority proves **absence** on its own path: no entry
//!   routes to its id at the live root — a sub-millisecond, local,
//!   zero-trust check with no broker round-trip.
//!
//! A witness produced at an older root fails the `DO` gate: staleness is
//! cryptographic, not timestamp-heuristic.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! The fence certifies "this id is not in the committed revocation set at
//! this root". It does not decide whether the underlying grant was lawful —
//! that admission happened upstream and is receipted separately.

use crate::smt::{
    common_prefix_len, verify_absence, verify_inclusion, AbsenceWitness, InclusionProof, StateKey,
    StateRoot, StateTree, StateValue,
};
use bitvec::prelude::{BitSlice, Msb0};
use bitvec::view::BitView;
use thiserror::Error;

/// Marker value committed under a revoked authority id.
pub const REVOCATION_TOMBSTONE: StateValue = [0xFF; 32];

/// Errors and refusals produced by the authority fence.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FenceError {
    /// The tree operation behind the fence failed.
    #[error("fence tree operation failed: {0}")]
    Tree(String),
    /// The id is already in the revocation set.
    #[error("authority already revoked: {0}")]
    AlreadyRevoked(String),
    /// The id is not in the revocation set, so it cannot be un-revoked.
    #[error("authority was not revoked: {0}")]
    NotRevoked(String),
    /// The `DO` gate refused: the witness does not hold under the live root.
    #[error("DO gate refused: witness does not hold under live root")]
    WitnessStaleOrInvalid,
    /// The `DO` gate refused: the authority id is revoked at the live root.
    #[error("DO gate refused: authority revoked at live root")]
    Revoked,
}

/// The revocation-set fence.
#[derive(Default)]
pub struct AuthorityFence {
    tree: StateTree,
}

/// Witness produced at `PREPARE`/`FENCE` time, checked again at `DO`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoWitness {
    /// Inclusion witness: the id provably maps to a tombstone (used by
    /// auditors to demonstrate a refusal).
    Revoked {
        /// The revoked authority id.
        id: StateKey,
        /// Inclusion proof of the tombstone.
        proof: InclusionProof,
    },
    /// Absence witness: no entry routes to the id (the `DO` permit).
    Admitted(AbsenceWitness),
}

impl AuthorityFence {
    /// An empty fence: nothing revoked.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The live revocation-set root. `None` when nothing has ever been
    /// revoked.
    #[must_use]
    pub fn root(&self) -> Option<StateRoot> {
        self.tree.root()
    }

    /// Revoke an authority id: commit the tombstone under the live root.
    ///
    /// # Errors
    ///
    /// Returns [`FenceError::AlreadyRevoked`] if the id is already in the
    /// revocation set.
    pub fn revoke(&mut self, id: &StateKey) -> Result<StateRoot, FenceError> {
        if matches!(self.tree.get(id), Ok(Some(_))) {
            return Err(FenceError::AlreadyRevoked(hex_id(id)));
        }
        self.tree
            .insert(id, &REVOCATION_TOMBSTONE)
            .map_err(|e| FenceError::Tree(e.to_string()))
    }

    /// Lift a revocation (lease renewal): remove the tombstone.
    ///
    /// # Errors
    ///
    /// Returns [`FenceError::NotRevoked`] if no tombstone is committed.
    pub fn unrevoke(&mut self, id: &StateKey) -> Result<Option<StateRoot>, FenceError> {
        if !matches!(self.tree.get(id), Ok(Some(_))) {
            return Err(FenceError::NotRevoked(hex_id(id)));
        }
        self.tree
            .remove(id)
            .map_err(|e| FenceError::Tree(e.to_string()))
    }

    /// Produce a revocation-inclusion witness for an auditor.
    ///
    /// # Errors
    ///
    /// Returns [`FenceError::NotRevoked`] if the id has no tombstone.
    pub fn prove_revoked(&mut self, id: &StateKey) -> Result<DoWitness, FenceError> {
        let proof = self
            .tree
            .prove_inclusion(id)
            .map_err(|_| FenceError::NotRevoked(hex_id(id)))?;
        Ok(DoWitness::Revoked { id: *id, proof })
    }

    /// Produce the `DO` permit: an absence witness for the id under the
    /// live root.
    ///
    /// # Errors
    ///
    /// Returns [`FenceError::Revoked`] if the id *is* revoked (the caller
    /// asked for a permit for a revoked id), or [`FenceError::Tree`] on
    /// internal failures. An empty fence yields an unconditional permit.
    pub fn prepare_do_permit(&mut self, id: &StateKey) -> Result<DoWitness, FenceError> {
        if self.tree.is_empty() {
            return Ok(DoWitness::Admitted(AbsenceWitness {
                queried_key: *id,
                neighbor_key: *id,
                neighbor_proof: InclusionProof {
                    key: *id,
                    steps: Vec::new(),
                },
                claimed_prefix_bits: 0,
            }));
        }
        if matches!(self.tree.get(id), Ok(Some(_))) {
            return Err(FenceError::Revoked);
        }
        let witness = self
            .tree
            .prove_absence(id)
            .map_err(|e| FenceError::Tree(e.to_string()))?;
        Ok(DoWitness::Admitted(witness))
    }
}

/// The `DO` gate: admit the effector only if the witness holds under the
/// **live** root.
///
/// # Errors
///
/// - [`FenceError::WitnessStaleOrInvalid`] for witnesses that fail under
///   the live root (stale, tampered, or from a different fence).
/// - [`FenceError::Revoked`] for a valid revocation-inclusion witness
///   presented at the gate (the effector must not run).
pub fn gate_do(witness: &DoWitness, live_root: Option<StateRoot>) -> Result<(), FenceError> {
    // An empty structural permit is valid only against an empty fence: a
    // fence that has revoked anything has a Some root and the structural
    // shortcut must not pass through it.
    let is_structural_permit = matches!(witness, DoWitness::Admitted(absence)
        if absence.claimed_prefix_bits == 0
            && absence.queried_key == absence.neighbor_key
            && absence.neighbor_proof.steps.is_empty());
    if is_structural_permit {
        if live_root.is_none() {
            return Ok(());
        }
        return Err(FenceError::WitnessStaleOrInvalid);
    }

    let root = live_root.ok_or(FenceError::WitnessStaleOrInvalid)?;
    match witness {
        DoWitness::Revoked { proof, .. } => {
            if verify_inclusion(&root, &REVOCATION_TOMBSTONE, proof) {
                Err(FenceError::Revoked)
            } else {
                Err(FenceError::WitnessStaleOrInvalid)
            }
        }
        DoWitness::Admitted(absence) => {
            // The verifier needs the neighbor's committed value to check
            // the inclusion half; recompute it from the tombstone law: the
            // only value ever committed is the tombstone.
            if verify_absence(&root, &REVOCATION_TOMBSTONE, absence) {
                Ok(())
            } else {
                Err(FenceError::WitnessStaleOrInvalid)
            }
        }
    }
}

/// Bit-level check used by tests and auditors: does the queried key route
/// into the neighbor's sibling slot at `depth`?
#[must_use]
pub fn routes_to_sibling_slot(queried: &StateKey, neighbor: &StateKey, depth: usize) -> bool {
    let q: &BitSlice<u8, Msb0> = queried.view_bits::<Msb0>();
    let n: &BitSlice<u8, Msb0> = neighbor.view_bits::<Msb0>();
    common_prefix_len(q, n) == depth && q.get(depth) != n.get(depth)
}

fn hex_id(id: &StateKey) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(seed: u8) -> StateKey {
        let mut k = [0u8; 32];
        k[0] = seed;
        k
    }

    #[test]
    fn unrevoked_id_admits_at_do() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(0x80)).expect("revoke one");
        let witness = fence.prepare_do_permit(&id(0x7F)).expect("permit");
        gate_do(&witness, fence.root()).expect("DO admitted");
    }

    #[test]
    fn revoked_id_refuses_at_do() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(1)).expect("revoke");
        assert_eq!(fence.prepare_do_permit(&id(1)), Err(FenceError::Revoked));
        // The auditor's inclusion witness also refuses at the gate — with
        // the revocation verdict.
        let witness = fence.prove_revoked(&id(1)).expect("revocation witness");
        assert_eq!(gate_do(&witness, fence.root()), Err(FenceError::Revoked));
    }

    #[test]
    fn stale_root_refuses_the_gate() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(0x80)).expect("revoke");
        let witness = fence.prepare_do_permit(&id(0x40)).expect("permit");
        // The root advances after the witness was cut.
        fence.revoke(&id(0x01)).expect("revoke again");
        assert_eq!(
            gate_do(&witness, fence.root()),
            Err(FenceError::WitnessStaleOrInvalid)
        );
    }

    #[test]
    fn unrevoke_restores_admission() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(9)).expect("revoke");
        fence.unrevoke(&id(9)).expect("unrevoke");
        let witness = fence.prepare_do_permit(&id(9)).expect("permit after lift");
        gate_do(&witness, fence.root()).expect("DO admitted");
    }

    #[test]
    fn double_revoke_and_phantom_unrevoke_are_refused() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(2)).expect("revoke");
        assert!(matches!(
            fence.revoke(&id(2)),
            Err(FenceError::AlreadyRevoked(_))
        ));
        assert!(matches!(
            fence.unrevoke(&id(3)),
            Err(FenceError::NotRevoked(_))
        ));
    }

    #[test]
    fn empty_fence_yields_structural_permit_only() {
        let mut fence = AuthorityFence::new();
        let witness = fence.prepare_do_permit(&id(4)).expect("permit");
        gate_do(&witness, fence.root()).expect("admitted");
        // A fabricated "empty fence" witness must not pass against a
        // non-empty fence.
        let mut other = AuthorityFence::new();
        other.revoke(&id(0x80)).expect("revoke");
        assert_eq!(
            gate_do(&witness, other.root()),
            Err(FenceError::WitnessStaleOrInvalid)
        );
    }

    #[test]
    fn routing_law_holds_on_real_witnesses() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(0b1000_0000)).expect("revoke");
        let witness = fence.prepare_do_permit(&id(0b0111_1111)).expect("permit");
        if let DoWitness::Admitted(absence) = &witness {
            assert!(routes_to_sibling_slot(
                &absence.queried_key,
                &absence.neighbor_key,
                absence.claimed_prefix_bits
            ));
        } else {
            panic!("expected absence witness");
        }
        gate_do(&witness, fence.root()).expect("admitted");
    }
}
