//! Sparse Merkle tree state accumulator (monotree, BLAKE3).
//!
//! An SMT over 32-byte keys provides sub-linear inclusion witnesses for
//! key-value state claims **and** absence witnesses: proof that a key routes
//! into an empty subtree. This is the primitive behind zero-trust revocation
//! checks at a `DO` fence — no broker round-trip, one local root comparison.
//!
//! ## Proof semantics (exact, and the standing ceiling)
//!
//! monotree generates inclusion proofs for present keys only. An
//! [`AbsenceWitness`] is therefore composed: an inclusion proof for the
//! *divergent neighbor* (the present key sharing the longest common bit
//! prefix with the queried key) plus the claimed prefix depth. Verification
//! checks (a) the neighbor's inclusion cryptographically against the root
//! and (b) that the queried key and neighbor key agree on the claimed
//! prefix and diverge immediately after — so the queried key provably routes
//! into the neighbor's empty sibling slot. The witness does NOT prove no
//! other key exists elsewhere in the tree; it proves this key has no entry
//! on its own path, which is the property a revocation fence needs.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! Roots and proofs certify state claims. The tree never decides whether a
//! value is truthful — only that it was committed under a key at a root.

use bitvec::prelude::BitSlice;
use bitvec::view::BitView;
use monotree::hasher::Blake3 as MonotreeBlake3;
use monotree::Monotree;
use thiserror::Error;

/// monotree's hash length: 32 bytes (BLAKE3 output).
pub use monotree::HASH_LEN;

/// Errors produced by SMT operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SmtError {
    /// The tree operation failed.
    #[error("SMT operation failed: {0}")]
    Tree(String),
    /// No proof could be generated (empty tree or unreachable path).
    #[error("no proof available for key")]
    NoProof,
}

/// A 32-byte state key.
pub type StateKey = [u8; 32];
/// A 32-byte state value (the leaf commitment).
pub type StateValue = [u8; 32];
/// A 32-byte SMT root commitment.
pub type StateRoot = [u8; 32];

/// Inclusion proof for a present key: monotree's `(is_right, cut_bytes)`
/// step list, root-to-leaf ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof {
    /// The key this proof was generated for.
    pub key: StateKey,
    /// monotree proof steps.
    pub steps: Vec<(bool, Vec<u8>)>,
}

/// Absence proof for a key with no entry on its path.
///
/// See the module docs for the exact check performed by
/// [`verify_absence`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsenceWitness {
    /// The key whose absence is claimed.
    pub queried_key: StateKey,
    /// The present neighbor key that diverges from `queried_key`.
    pub neighbor_key: StateKey,
    /// monotree inclusion proof for `neighbor_key` under `root`.
    pub neighbor_proof: InclusionProof,
    /// Length in bits of the claimed common prefix between the queried key
    /// and the neighbor key. Must be followed by an immediate divergence.
    pub claimed_prefix_bits: usize,
}

/// A sparse Merkle tree over 32-byte keys, backed by monotree's in-memory
/// database and BLAKE3.
///
/// The wrapper keeps a shadow key index (`keys`) solely to select divergent
/// neighbors for absence-witness generation; verifiers never see or trust
/// it — verification touches only the root and the proof bytes.
#[derive(Default)]
pub struct StateTree {
    tree: Monotree<monotree::database::MemoryDB, MonotreeBlake3>,
    root: Option<monotree::Hash>,
    keys: std::collections::BTreeMap<StateKey, StateValue>,
}

impl StateTree {
    /// An empty tree rooted at the monotree empty-root (`None`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The current root commitment. `None` for the empty tree.
    #[must_use]
    pub fn root(&self) -> Option<StateRoot> {
        self.root
    }

    /// Insert a key-value commitment, returning the new root.
    ///
    /// # Errors
    ///
    /// Returns [`SmtError::Tree`] if the underlying tree operation fails.
    pub fn insert(&mut self, key: &StateKey, value: &StateValue) -> Result<StateRoot, SmtError> {
        let root = self
            .tree
            .insert(self.root.as_ref(), key, value)
            .map_err(|e| SmtError::Tree(e.to_string()))?;
        self.root = root;
        self.keys.insert(*key, *value);
        root.ok_or_else(|| SmtError::Tree("insert produced no root".to_string()))
    }

    /// Fetch the committed value for a key.
    ///
    /// # Errors
    ///
    /// Returns [`SmtError::Tree`] if the underlying tree operation fails.
    pub fn get(&mut self, key: &StateKey) -> Result<Option<StateValue>, SmtError> {
        self.tree
            .get(self.root.as_ref(), key)
            .map_err(|e| SmtError::Tree(e.to_string()))
    }

    /// Remove a key's commitment, returning the new root.
    ///
    /// `None` is the legitimate empty-tree root: removing the last entry
    /// returns the tree to its initial state.
    ///
    /// # Errors
    ///
    /// Returns [`SmtError::Tree`] if the underlying tree operation fails.
    pub fn remove(&mut self, key: &StateKey) -> Result<Option<StateRoot>, SmtError> {
        let root = self
            .tree
            .remove(self.root.as_ref(), key)
            .map_err(|e| SmtError::Tree(e.to_string()))?;
        self.root = root;
        self.keys.remove(key);
        Ok(root)
    }

    /// Number of committed entries (shadow index; generation-side only).
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Whether the tree commits no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Generate an inclusion proof for a present key.
    ///
    /// # Errors
    ///
    /// Returns [`SmtError::NoProof`] if the key has no committed entry or
    /// the tree cannot route to it.
    pub fn prove_inclusion(&mut self, key: &StateKey) -> Result<InclusionProof, SmtError> {
        if !self.keys.contains_key(key) {
            return Err(SmtError::NoProof);
        }
        let steps = self
            .tree
            .get_merkle_proof(self.root.as_ref(), key)
            .map_err(|e| SmtError::Tree(e.to_string()))?
            .ok_or(SmtError::NoProof)?;
        Ok(InclusionProof { key: *key, steps })
    }

    /// Generate an absence witness for a key with no committed entry.
    ///
    /// Selects the present key sharing the longest common bit prefix with
    /// `key` and proves its inclusion; the verifier re-checks the prefix
    /// law from the proof alone.
    ///
    /// # Errors
    ///
    /// Returns [`SmtError::NoProof`] if the tree is empty or the key
    /// already has an entry (that is [`prove_inclusion`]'s job).
    pub fn prove_absence(&mut self, key: &StateKey) -> Result<AbsenceWitness, SmtError> {
        if self.keys.contains_key(key) {
            return Err(SmtError::NoProof);
        }
        let queried_bits = key.view_bits::<bitvec::prelude::Msb0>();
        let neighbor = self
            .keys
            .keys()
            .max_by_key(|candidate| {
                let candidate_bits = candidate.view_bits::<bitvec::prelude::Msb0>();
                common_prefix_len(queried_bits, candidate_bits)
            })
            .copied()
            .ok_or(SmtError::NoProof)?;
        let claimed_prefix_bits =
            common_prefix_len(queried_bits, neighbor.view_bits::<bitvec::prelude::Msb0>());
        let neighbor_proof = self.prove_inclusion(&neighbor)?;
        Ok(AbsenceWitness {
            queried_key: *key,
            neighbor_key: neighbor,
            neighbor_proof,
            claimed_prefix_bits,
        })
    }
}

/// Length in bits of the common prefix of two same-length bit slices.
#[must_use]
pub fn common_prefix_len(
    a: &BitSlice<u8, bitvec::prelude::Msb0>,
    b: &BitSlice<u8, bitvec::prelude::Msb0>,
) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

/// Verify an inclusion proof cryptographically against a root.
///
/// The proof fold recomputes the root from the committed leaf value; both
/// the key routing and the value binding are checked in that single
/// evaluation.
#[must_use]
pub fn verify_inclusion(
    root: &StateRoot,
    expected_value: &StateValue,
    proof: &InclusionProof,
) -> bool {
    monotree::verify_proof(
        &MonotreeBlake3,
        Some(root),
        expected_value,
        Some(&proof.steps),
    )
}

/// Verify an absence witness against a root.
///
/// Checks: (a) the neighbor's inclusion proof is cryptographically valid
/// (both for the neighbor key and its committed value); (b) the queried key
/// and neighbor key agree on exactly `claimed_prefix_bits` and diverge on
/// the next bit — so the queried key routes into the neighbor's empty
/// sibling subtree and cannot have an entry on its own path.
#[must_use]
pub fn verify_absence(
    root: &StateRoot,
    neighbor_value: &StateValue,
    witness: &AbsenceWitness,
) -> bool {
    if witness.queried_key == witness.neighbor_key {
        return false;
    }
    if !verify_inclusion(root, neighbor_value, &witness.neighbor_proof) {
        return false;
    }
    let queried = witness.queried_key.view_bits::<bitvec::prelude::Msb0>();
    let neighbor = witness.neighbor_key.view_bits::<bitvec::prelude::Msb0>();
    let actual_prefix = common_prefix_len(queried, neighbor);
    actual_prefix == witness.claimed_prefix_bits
        && queried.get(witness.claimed_prefix_bits) != neighbor.get(witness.claimed_prefix_bits)
        && witness.claimed_prefix_bits < 256
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(seed: u8) -> StateKey {
        let mut k = [0u8; 32];
        k[0] = seed;
        k
    }

    fn value(seed: u8) -> StateValue {
        let mut v = [0u8; 32];
        v[0] = seed.wrapping_mul(7);
        v
    }

    #[test]
    fn insert_get_roundtrip_and_root_progression() {
        let mut tree = StateTree::new();
        assert_eq!(tree.root(), None);
        let r1 = tree.insert(&key(1), &value(1)).expect("insert");
        assert_ne!(r1, key(0));
        assert_eq!(tree.get(&key(1)).expect("get"), Some(value(1)));
        let r2 = tree.insert(&key(2), &value(2)).expect("insert");
        assert_ne!(r1, r2, "each insert advances the root");
    }

    #[test]
    fn inclusion_proof_verifies_and_tampering_fails() {
        let mut tree = StateTree::new();
        tree.insert(&key(9), &value(9)).expect("insert");
        tree.insert(&key(10), &value(10)).expect("insert");
        let root = tree.root().expect("root");
        let proof = tree.prove_inclusion(&key(9)).expect("proof");
        assert!(verify_inclusion(&root, &value(9), &proof));

        let mut tampered = proof.clone();
        tampered.steps[0].0 = !tampered.steps[0].0;
        assert!(!verify_inclusion(&root, &value(9), &tampered));
        assert!(!verify_inclusion(&root, &value(10), &proof));
    }

    #[test]
    fn absence_witness_verifies_for_uncommitted_key() {
        let mut tree = StateTree::new();
        tree.insert(&key(0b1000_0000), &value(1)).expect("insert");
        let root = tree.root().expect("root");
        let witness = tree.prove_absence(&key(0b0111_1111)).expect("absence");
        assert!(verify_absence(&root, &value(1), &witness));
    }

    #[test]
    fn absence_witness_refuses_committed_key_and_empty_tree() {
        let mut tree = StateTree::new();
        assert_eq!(tree.prove_absence(&key(1)), Err(SmtError::NoProof));
        tree.insert(&key(1), &value(1)).expect("insert");
        assert_eq!(tree.prove_absence(&key(1)), Err(SmtError::NoProof));
    }

    #[test]
    fn remove_restores_empty_root() {
        let mut tree = StateTree::new();
        tree.insert(&key(3), &value(3)).expect("insert");
        let root = tree.remove(&key(3)).expect("remove");
        assert_eq!(root, None, "removing the last entry empties the tree");
        assert_eq!(tree.get(&key(3)).expect("get"), None);
        assert_eq!(tree.root(), None);
        assert!(tree.is_empty());
    }

    #[test]
    fn stale_root_rejects_fresh_proof() {
        let mut tree = StateTree::new();
        tree.insert(&key(1), &value(1)).expect("insert");
        let stale_root = tree.root();
        tree.insert(&key(2), &value(2)).expect("insert");
        let proof = tree.prove_inclusion(&key(1)).expect("proof");
        assert!(!verify_inclusion(
            &stale_root.expect("root"),
            &value(1),
            &proof
        ));
    }
}
