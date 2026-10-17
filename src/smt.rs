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

    /// Remove a batch of keys, returning the root after the final removal.
    ///
    /// Convenience over [`StateTree::remove`]; each key is removed in order
    /// against the running root. An empty batch is a no-op returning the
    /// current root.
    ///
    /// # Errors
    ///
    /// Returns [`SmtError::Tree`] if any underlying removal fails.
    pub fn remove_batch(&mut self, keys: &[StateKey]) -> Result<Option<StateRoot>, SmtError> {
        let mut root = self.root();
        for k in keys {
            root = self.remove(k)?;
        }
        Ok(root)
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

/// monotree's `BitsLen` = `u16`, big-endian (`bytes_to_int`).
fn be16(b: [u8; 2]) -> usize {
    (usize::from(b[0]) << 8) | usize::from(b[1])
}

/// Extract the consumed key-path range `(start, end)` and the 32-byte fixed
/// path bit array from one monotree proof-step cut. Empirically pinned layout
/// (monotree serializes `Unit.bits` as a fixed `BIT_ARRAY_LEN = 32` bytes with
/// absolute bit indexing):
/// - soft cut:      `[rs:2][re:2][path:32][0x00]` (37 bytes)
/// - hard-left cut: `[rsL:2][reL:2][pathL:32][rsR:2][reR:2][pathR:32][hashR:32][0x01]` (105)
/// - hard-right cut: `[hashL:32][rsL:2][reL:2][pathL:32][rsR:2][reR:2][pathR:32][0x01]` (137)
#[must_use]
fn step_path_range<'a>(cut: &'a [u8], right: bool) -> Option<(usize, usize, &'a [u8])> {
    let ac = |s: usize, e: usize| {
        let n = (e - (s - s % 8)) / 8;
        if e % 8 == 0 {
            n
        } else {
            n + 1
        }
    };
    if right {
        // [hashL:32][rsL:2][reL:2][pathL:acL][rsR:2][reR:2][pathR:acR][0x01]
        if cut.len() < 38 || *cut.last()? != 0x01 {
            return None;
        }
        let (rs_l, re_l) = (be16([cut[32], cut[33]]), be16([cut[34], cut[35]]));
        let o = 36 + ac(rs_l, re_l);
        if cut.len() < o + 5 {
            return None;
        }
        let rs = be16([cut[o], cut[o + 1]]);
        let re = be16([cut[o + 2], cut[o + 3]]);
        let pl = ac(rs, re);
        if cut.len() != o + 4 + pl + 1 {
            return None;
        }
        Some((rs, re, cut.get(o + 4..o + 4 + pl)?))
    } else {
        if cut.len() < 4 {
            return None;
        }
        let rs = be16([cut[0], cut[1]]);
        let re = be16([cut[2], cut[3]]);
        if re > 256 || rs >= re {
            return None;
        }
        let pl = ac(rs, re);
        match cut.len() {
            n if n == pl + 5 && cut.get(pl + 4) == Some(&0x00) => {
                Some((rs, re, cut.get(4..4 + pl)?))
            }
            n if n > pl + 5 && cut.last() == Some(&0x01) => {
                // Hard-node left branch: [rsL:2][reL:2][pathL][rsR:2][reR:2][pathR][hashR:32][0x01]
                let o = 4 + pl;
                if cut.len() < o + 5 {
                    return None;
                }
                let (rs_r, re_r) = (be16([cut[o], cut[o + 1]]), be16([cut[o + 2], cut[o + 3]]));
                let pl_r = ac(rs_r, re_r);
                if cut.len() != o + 4 + pl_r + 32 + 1 {
                    return None;
                }
                Some((rs, re, cut.get(4..4 + pl)?))
            }
            _ => None,
        }
    }
}

/// Whether an inclusion proof's steps route exactly to `key`: the cut-encoded
/// path ranges must tile the full 256-bit key space in order, and every path
/// bit must match the key. monotree's `verify_proof` folds only the leaf value
/// and the step cuts — the proof's `key` field is decorative — so this is the
/// binding a revocation fence actually rests on.
#[must_use]
fn proof_routes_key(proof: &InclusionProof) -> bool {
    let key = proof.key;
    let bit = |i: usize| (key[i / 8] >> (7 - i % 8)) & 1;
    let mut expected = 0usize;
    for (right, cut) in &proof.steps {
        let Some((rs, re, path)) = step_path_range(cut, *right) else {
            return false;
        };
        if rs != expected || re <= rs || re > 256 {
            return false;
        }
        for i in rs..re {
            // The stored path slice begins at absolute byte floor(rs/8).
            let Some(&byte) = path.get(i / 8 - rs / 8) else {
                return false;
            };
            let bitpos = 7 - i % 8;
            if (byte >> bitpos) & 1 != bit(i) {
                return false;
            }
        }
        expected = re;
    }
    expected == 256
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
    // Key binding: the embedded proof must route exactly `neighbor_key`.
    // Without this, a valid inclusion proof for an unrelated key (with its
    // matching leaf value) satisfies the cryptographic check while the
    // prefix law is argued from a different key entirely.
    if witness.neighbor_proof.key != witness.neighbor_key
        || !proof_routes_key(&witness.neighbor_proof)
    {
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
    fn post_revocation_absence_court() {
        let mut tree = StateTree::new();
        tree.insert(&key(0x10), &value(1)).expect("insert");
        tree.insert(&key(0x20), &value(2)).expect("insert");
        tree.insert(&key(0x30), &value(3)).expect("insert");
        let pre_root = tree.root().expect("pre-removal root");

        tree.remove(&key(0x10)).expect("remove");
        let post_root = tree.root().expect("post-removal root");
        assert_ne!(pre_root, post_root, "removal must advance the root");

        let witness = tree.prove_absence(&key(0x10)).expect("absence witness");
        let neighbor_value = tree
            .get(&witness.neighbor_key)
            .expect("get")
            .expect("neighbor committed");

        // Admits against the post-removal root.
        assert!(verify_absence(&post_root, &neighbor_value, &witness));
        // The same witness is refused against the pre-removal root.
        assert!(!verify_absence(&pre_root, &neighbor_value, &witness));

        // Re-insert: inclusion at the newest root.
        let newest_root = tree.insert(&key(0x10), &value(1)).expect("re-insert");
        let proof = tree.prove_inclusion(&key(0x10)).expect("proof");
        assert!(verify_inclusion(&newest_root, &value(1), &proof));
        assert_eq!(tree.root(), Some(newest_root));
    }

    #[test]
    fn batch_removal_absence_court() {
        let mut tree = StateTree::new();
        tree.insert(&key(0x0F), &value(1)).expect("insert");
        tree.insert(&key(0xF0), &value(2)).expect("insert");
        tree.insert(&key(0x33), &value(3)).expect("insert");
        let pre_root = tree.root().expect("pre-removal root");

        let removed = [key(0x0F), key(0xF0)];
        let final_root = tree.remove_batch(&removed).expect("remove_batch");
        assert_eq!(final_root, tree.root());
        assert_eq!(tree.len(), 1);

        let mut witnesses = Vec::new();
        for k in &removed {
            let w = tree.prove_absence(k).expect("absence witness");
            assert!(
                !verify_absence(
                    &pre_root,
                    &tree.get(&w.neighbor_key).expect("get").expect("present"),
                    &w
                ),
                "witness must be refused against the pre-removal root"
            );
            witnesses.push(w);
        }
        let post_root = tree.root().expect("post-removal root");
        for w in &witnesses {
            let nv = tree.get(&w.neighbor_key).expect("get").expect("present");
            assert!(verify_absence(&post_root, &nv, w));
        }

        // The surviving key still proves inclusion at the newest root.
        let survivor_proof = tree.prove_inclusion(&key(0x33)).expect("proof");
        assert!(verify_inclusion(&post_root, &value(3), &survivor_proof));

        // Vacuity: a wrong-path retirement (key 0x33 removed from the shadow
        // index but not the tree) must flip this court — absence for 0x33
        // would then be provable against a tree that still commits it.
        assert_eq!(tree.prove_absence(&key(0x33)), Err(SmtError::NoProof));
    }

    // ---- Cycle-4 falsifier courts ----

    // Falsifier for the absence-witness key-binding defect: the embedded
    // neighbor_proof may route a DIFFERENT key than witness.neighbor_key,
    // because verify_inclusion never reads `proof.key`. Attack: neighbor_key
    // shares the claimed prefix with queried_key (prefix law holds), but the
    // embedded proof is a valid inclusion proof for an unrelated key under
    // the same root. If verify_absence admits this, a revocation fence could
    // be bypassed with a proof from an unrelated subtree.
    #[test]
    fn absence_witness_refuses_neighbor_proof_key_mismatch() {
        let mut tree = StateTree::new();
        // Neighbor candidate: shares long prefix with the queried key.
        tree.insert(&key(0x40), &value(1)).expect("insert");
        // Unrelated key in a far subtree, under the same root.
        tree.insert(&key(0xB0), &value(2)).expect("insert");
        let root = tree.root().expect("root");

        let queried = key(0x41); // 0b0100_0001, shares 7-bit prefix with 0x40
        let witness = tree.prove_absence(&queried).expect("absence witness");
        assert_eq!(witness.neighbor_key, key(0x40));
        assert!(
            verify_absence(&root, &value(1), &witness),
            "honest witness must verify"
        );

        // Forge: swap in a valid inclusion proof for the unrelated key.
        let forged = AbsenceWitness {
            neighbor_proof: tree.prove_inclusion(&key(0xB0)).expect("unrelated proof"),
            ..witness
        };
        // The attacker also supplies the matching leaf value — the verifier
        // takes neighbor_value from the prover, so a coherent (proof, value)
        // pair from an unrelated subtree is the real attack.
        assert!(
            !verify_absence(&root, &value(2), &forged),
            "FLIP = defect CONFIRMED: proof routing an unrelated key admitted"
        );
    }

    // Negative control for the forged-witness court: the honest witness built
    // from prove_absence must still verify after the key-binding repair, and
    // tampering the claimed prefix must still refuse.
    #[test]
    fn absence_witness_prefix_tamper_still_refused() {
        let mut tree = StateTree::new();
        tree.insert(&key(0x40), &value(1)).expect("insert");
        tree.insert(&key(0xB0), &value(2)).expect("insert");
        let root = tree.root().expect("root");
        let mut w = tree.prove_absence(&key(0x41)).expect("witness");
        assert!(verify_absence(&root, &value(1), &w));
        w.claimed_prefix_bits += 1;
        assert!(
            !verify_absence(&root, &value(1), &w),
            "prefix lie must refuse"
        );
        w.claimed_prefix_bits -= 1;
        w.neighbor_key = key(0xB0);
        assert!(
            !verify_absence(&root, &value(1), &w),
            "neighbor-key lie must refuse"
        );
    }

    // Vacuity court: an InclusionProof with zero steps folds to the bare
    // leaf hash under monotree's verify_proof; against a multi-entry root
    // it must refuse, or any value could be 'proven' present.
    #[test]
    fn empty_step_proof_refused_against_multi_entry_root() {
        let mut tree = StateTree::new();
        tree.insert(&key(0x01), &value(1)).expect("insert");
        tree.insert(&key(0x02), &value(2)).expect("insert");
        let root = tree.root().expect("root");
        let empty = InclusionProof {
            key: key(0x01),
            steps: vec![],
        };
        assert!(
            !verify_inclusion(&root, &value(1), &empty),
            "FLIP = vacuity defect: empty proof admitted against multi-entry root"
        );
        // Same refusal at a single-entry root: monotree's root is never the
        // bare leaf hash, so zero steps are never sufficient. Pinned here so
        // a monotree upgrade that changes this flips loudly.
        let mut single = StateTree::new();
        single.insert(&key(0x01), &value(1)).expect("insert");
        let sroot = single.root().expect("root");
        let empty_single = InclusionProof {
            key: key(0x01),
            steps: vec![],
        };
        assert!(
            !verify_inclusion(&sroot, &value(1), &empty_single),
            "monotree single-entry root semantics changed: empty proof admitted"
        );
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
