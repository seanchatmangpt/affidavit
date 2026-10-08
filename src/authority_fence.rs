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

// --- Fast-path actuation gate ------------------------------------------------
// Cost-ordered composition of the cheap screens ahead of any expensive
// cryptography: cuckoo replay screen O(1) → HLC monotonicity O(1) → SMT
// revocation absence O(log 256). Heavy witnesses (threshold signatures, ZK
// range proofs) are verified only AFTER the fast path admits — and the
// permit is re-confirmed against the live root at actuation time.
#[cfg(all(feature = "hlc", feature = "replay-filter"))]
pub mod fast_path {
    use super::{gate_do, AuthorityFence, DoWitness, FenceError, StateRoot};
    use crate::hlc::{HlcClock, HlcError, HlcTimestamp};
    use crate::replay_filter::ReplayFilter;

    /// The claim an effector presents at the fast-path gate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConsequenceClaim {
        /// Replay-screen key (nonce): first sight admits, any resubmission
        /// refuses. Note: the id is burned at screen time even if a later
        /// stage refuses — a refused claim can never be replayed.
        pub consequence_id: [u8; 32],
        /// The authority whose revocation status the fence checks.
        pub authority_id: [u8; 32],
        /// The upstream causal stamp to absorb into the local clock.
        pub received: HlcTimestamp,
    }

    /// Fast-path refusals, named in the order they can fire.
    #[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
    pub enum FastPathRefusal {
        /// The cuckoo screen has already seen this consequence id.
        #[error("replay refused: consequence id already witnessed")]
        Replay,
        /// The received stamp violates causality or the skew bound.
        #[error("temporal order refused: {0}")]
        ClockSkew(#[from] HlcError),
        /// The authority id is in the revocation set at the live root.
        #[error("authority revoked at live root")]
        Revoked,
        /// A fence-internal refusal (stale, unknown, or tree failure).
        #[error("fence refused: {0}")]
        Fence(#[from] FenceError),
    }

    /// Everything the caller needs to actuate after heavy checks: the
    /// revocation permit cut at gate time, the causal stamp the gate
    /// issued, and the root the permit was cut against.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct AdmittedProof {
        /// The revocation-set permit (inclusion or absence witness).
        pub permit: DoWitness,
        /// The stamp the gate issued after absorbing the claim's stamp.
        pub admitted_at: HlcTimestamp,
        /// The revocation-set root at gate time. Re-check against the live
        /// root before actuation: [`AdmittedProof::confirm`].
        pub gate_root: Option<StateRoot>,
    }

    impl AdmittedProof {
        /// Re-confirm the permit against the fence's **live** root.
        ///
        /// Call this after any expensive work (signature aggregation, ZK
        /// verification) and immediately before `DO`: if the root advanced
        /// between gate and actuation, the refusal is typed, not silent.
        ///
        /// # Errors
        ///
        /// See [`gate_do`].
        pub fn confirm(&self, fence: &AuthorityFence) -> Result<(), FenceError> {
            gate_do(&self.permit, fence.root())
        }
    }

    /// The fast-path actuation gate: screens in strict cost order, then
    /// issues a permit for post-crypto confirmation.
    ///
    /// 1. **Cuckoo replay screen** — O(1), sub-microsecond: catches replays
    ///    before anything else runs.
    /// 2. **HLC monotonicity** — O(1) integer compare: catches temporal
    ///    drift and causality violations.
    /// 3. **SMT revocation absence** — O(log 256) hash path: catches
    ///    revoked leases without any broker round-trip.
    ///
    /// Heavy cryptography (FROST aggregation, ZK range proofs) belongs
    /// AFTER this gate, followed by [`AdmittedProof::confirm`].
    ///
    /// # Errors
    ///
    /// Typed refusals in [`FastPathRefusal`]; never panics.
    pub fn verify_fast_path(
        fence: &mut AuthorityFence,
        replay: &mut ReplayFilter,
        clock: &mut HlcClock,
        claim: &ConsequenceClaim,
    ) -> Result<AdmittedProof, FastPathRefusal> {
        // Stage 1 (cheapest): replay screen. Commit-on-claim.
        if !replay
            .witness(&claim.consequence_id)
            .map_err(|e| FenceError::Tree(e.to_string()))?
        {
            return Err(FastPathRefusal::Replay);
        }
        // Stage 2: causal order. Absorbing refuses future-skewed stamps.
        let admitted_at = clock.receive(claim.received)?;
        // Stage 3: revocation absence at the live root.
        let permit = match fence.prepare_do_permit(&claim.authority_id) {
            Ok(witness) => witness,
            Err(FenceError::Revoked) => return Err(FastPathRefusal::Revoked),
            Err(e) => return Err(FastPathRefusal::Fence(e)),
        };
        Ok(AdmittedProof {
            permit,
            admitted_at,
            gate_root: fence.root(),
        })
    }
}

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

#[cfg(all(test, feature = "hlc", feature = "replay-filter"))]
mod fast_path_tests {
    use super::fast_path::{verify_fast_path, ConsequenceClaim, FastPathRefusal};
    use super::AuthorityFence;
    use crate::hlc::{timestamp, HlcClock};
    use crate::replay_filter::ReplayFilter;

    fn id(seed: u8) -> [u8; 32] {
        let mut k = [0u8; 32];
        k[0] = seed;
        k
    }

    #[test]
    fn honest_claim_admits_with_permit_and_stamp() {
        let mut fence = AuthorityFence::new();
        let mut replay = ReplayFilter::new(1_000);
        let mut clock = HlcClock::with_skew_bound(1_000);
        let upstream = clock.send();
        let claim = ConsequenceClaim {
            consequence_id: id(1),
            authority_id: id(2),
            received: upstream,
        };
        let proof =
            verify_fast_path(&mut fence, &mut replay, &mut clock, &claim).expect("admitted");
        assert_eq!(proof.gate_root, fence.root());
        proof.confirm(&fence).expect("confirm at live root");
    }

    #[test]
    fn replayed_claim_refused_at_the_screen() {
        let mut fence = AuthorityFence::new();
        let mut replay = ReplayFilter::new(1_000);
        let mut clock = HlcClock::with_skew_bound(1_000);
        let stamp = clock.send();
        let claim = ConsequenceClaim {
            consequence_id: id(9),
            authority_id: id(2),
            received: stamp,
        };
        verify_fast_path(&mut fence, &mut replay, &mut clock, &claim).expect("first admits");
        assert_eq!(
            verify_fast_path(&mut fence, &mut replay, &mut clock, &claim),
            Err(FastPathRefusal::Replay),
            "second sight is the replay refusal"
        );
    }

    #[test]
    fn revoked_authority_refused_before_heavy_work() {
        let mut fence = AuthorityFence::new();
        fence.revoke(&id(3)).expect("revoke");
        let mut replay = ReplayFilter::new(1_000);
        let mut clock = HlcClock::with_skew_bound(1_000);
        let stamp = clock.send();
        let claim = ConsequenceClaim {
            consequence_id: id(4),
            authority_id: id(3),
            received: stamp,
        };
        assert_eq!(
            verify_fast_path(&mut fence, &mut replay, &mut clock, &claim),
            Err(FastPathRefusal::Revoked)
        );
    }

    #[test]
    fn root_advance_between_gate_and_actuation_refuses_confirm() {
        let mut fence = AuthorityFence::new();
        fence
            .revoke(&id(0x80))
            .expect("seed revocation so root is Some");
        let mut replay = ReplayFilter::new(1_000);
        let mut clock = HlcClock::with_skew_bound(1_000);
        let stamp = clock.send();
        let claim = ConsequenceClaim {
            consequence_id: id(5),
            authority_id: id(6),
            received: stamp,
        };
        let proof = verify_fast_path(&mut fence, &mut replay, &mut clock, &claim)
            .expect("admitted at gate");
        // Root advances after the gate cut the permit.
        fence.revoke(&id(0x40)).expect("advance root");
        assert_eq!(
            proof.confirm(&fence),
            Err(super::FenceError::WitnessStaleOrInvalid),
            "stale permit refuses at actuation"
        );
    }

    #[test]
    fn skew_bound_refuses_far_future_stamps() {
        let mut fence = AuthorityFence::new();
        let mut replay = ReplayFilter::new(1_000);
        let mut clock = HlcClock::with_skew_bound(1_000);
        let far_future = timestamp(u64::MAX / 2, 0);
        let claim = ConsequenceClaim {
            consequence_id: id(7),
            authority_id: id(8),
            received: far_future,
        };
        assert!(matches!(
            verify_fast_path(&mut fence, &mut replay, &mut clock, &claim),
            Err(FastPathRefusal::ClockSkew(
                crate::hlc::HlcError::ClockSkewExceeded { .. }
            ))
        ));
        // The claim's consequence id was burned at the screen even though a
        // later stage refused — a refused claim can never be replayed.
        assert!(
            !replay.witness(&id(7)).expect("capacity"),
            "id must already be recorded"
        );
    }
}
