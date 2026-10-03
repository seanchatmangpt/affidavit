//! Merkle Mountain Range (MMR) accumulator and inclusion proofs.
//!
//! An MMR is an append-only, content-addressed collection of perfect binary trees
//! (mountains) whose peaks correspond to the binary decomposition of the leaf count.
//!
//! Unlike linear event chains that require $O(N)$ replay to verify an event, an MMR
//! enables $O(\log N)$ logarithmic inclusion proofs (`MmrProof`) for arbitrary events.
//!
//! ## Doctrine: Certify, Don't Decide
//!
//! The accumulator computes content-addressed digests and produces witnesses;
//! it does not decide whether the underlying event data is truthful.

use crate::digest::{ChainHasher, Digest, Fnv256};
use alloc::vec::Vec;
use core::marker::PhantomData;

/// Errors produced during MMR operations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MmrError {
    /// Leaf index is out of bounds for the current MMR size.
    IndexOutOfBounds {
        /// Requested leaf index.
        index: u64,
        /// Total leaves present.
        total_leaves: u64,
    },
    /// Attempted to operate on an empty MMR.
    EmptyMmr,
    /// Verification failed: proof length does not match expected mountain height.
    ProofLengthMismatch,
    /// Verification failed: peak list does not match leaf count decomposition.
    PeakCountMismatch,
}

/// Domain separation tags to prevent second-preimage attacks.
const DOMAIN_LEAF: &[u8] = b"mmr:v1:leaf:";
const DOMAIN_NODE: &[u8] = b"mmr:v1:node:";
const DOMAIN_BAG: &[u8] = b"mmr:v1:bag:";

/// Hash an internal node given its height, left child digest, and right child digest.
pub fn hash_children<H: ChainHasher>(height: u32, left: &Digest, right: &Digest) -> Digest {
    let mut state = H::init();
    H::absorb(&mut state, DOMAIN_NODE);
    H::absorb(&mut state, &height.to_le_bytes());
    H::absorb(&mut state, left.as_bytes());
    H::absorb(&mut state, right.as_bytes());
    H::finish(state)
}

/// Hash a leaf payload into a leaf digest.
pub fn hash_leaf_payload<H: ChainHasher>(payload: &[u8]) -> Digest {
    let mut state = H::init();
    H::absorb(&mut state, DOMAIN_LEAF);
    H::absorb(&mut state, payload);
    H::finish(state)
}

/// Bag an array of mountain peaks into a single root digest.
pub fn bag_peaks<H: ChainHasher>(peaks: &[Digest]) -> Digest {
    if peaks.is_empty() {
        return Digest::ZERO;
    }
    if peaks.len() == 1 {
        return peaks[0];
    }
    let mut state = H::init();
    H::absorb(&mut state, DOMAIN_BAG);
    for peak in peaks {
        H::absorb(&mut state, peak.as_bytes());
    }
    H::finish(state)
}

/// Information about a single mountain peak in the MMR.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MountainPeak {
    /// Height of this mountain (a tree with $2^h$ leaves).
    pub height: u32,
    /// Root digest of this mountain.
    pub digest: Digest,
}

/// Decompose a leaf count $N$ into descending mountain heights ($h_0 > h_1 > \dots$).
///
/// Each 1-bit in the binary representation of $N$ corresponds to a mountain of height $k$
/// containing $2^k$ leaves.
pub fn mountain_heights(mut num_leaves: u64) -> Vec<u32> {
    let mut heights = Vec::new();
    while num_leaves > 0 {
        let bit = 63 - num_leaves.leading_zeros();
        heights.push(bit);
        num_leaves &= !(1u64 << bit);
    }
    heights
}

/// Logarithmic inclusion proof for a leaf in a Merkle Mountain Range.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MmrProof {
    /// 0-based index of the leaf within the MMR.
    pub leaf_index: u64,
    /// Total number of leaves in the MMR when the proof was constructed.
    pub total_leaves: u64,
    /// Sibling digests along the path from the leaf to its mountain peak.
    pub siblings: Vec<Digest>,
    /// All mountain peaks of the MMR, ordered from largest to smallest.
    pub peaks: Vec<Digest>,
}

impl MmrProof {
    /// Verify this inclusion proof against an expected leaf digest and MMR root.
    pub fn verify<H: ChainHasher>(&self, leaf_digest: &Digest, expected_root: &Digest) -> bool {
        if self.leaf_index >= self.total_leaves || self.total_leaves == 0 {
            return false;
        }

        let heights = mountain_heights(self.total_leaves);
        if heights.len() != self.peaks.len() {
            return false;
        }

        // Locate which mountain contains self.leaf_index.
        let mut offset = 0u64;
        let mut mountain_idx = None;
        let mut target_height = 0u32;
        let mut local_index = 0u64;

        for (idx, &height) in heights.iter().enumerate() {
            let count = 1u64 << height;
            if self.leaf_index < offset + count {
                mountain_idx = Some(idx);
                target_height = height;
                local_index = self.leaf_index - offset;
                break;
            }
            offset += count;
        }

        let mountain_idx = match mountain_idx {
            Some(idx) => idx,
            None => return false,
        };

        if self.siblings.len() != target_height as usize {
            return false;
        }

        // Fold siblings from leaf up to the mountain peak.
        let mut current = *leaf_digest;
        let mut bit_tracker = local_index;

        for (level, sibling) in self.siblings.iter().enumerate() {
            let is_right_child = (bit_tracker & 1) == 1;
            current = if is_right_child {
                hash_children::<H>(level as u32, sibling, &current)
            } else {
                hash_children::<H>(level as u32, &current, sibling)
            };
            bit_tracker >>= 1;
        }

        // Verify the derived peak matches the recorded mountain peak.
        if current != self.peaks[mountain_idx] {
            return false;
        }

        // Bag all peaks and check against expected root.
        let root = bag_peaks::<H>(&self.peaks);
        root == *expected_root
    }
}

/// Complete in-memory Merkle Mountain Range.
///
/// Stores leaves and computed intermediate mountain nodes to allow $O(\log N)$
/// proof generation for any historical leaf.
#[derive(Clone, Debug)]
pub struct MmrAccumulator<H: ChainHasher = Fnv256> {
    /// Raw leaves added so far.
    leaves: Vec<Digest>,
    /// Intermediate tree levels per mountain.
    /// Each entry in `mountains` represents a mountain: `levels[0]` are leaves,
    /// `levels[1]` are height-1 parents, up to `levels[height]` (peak).
    mountains: Vec<Vec<Vec<Digest>>>,
    _hasher: PhantomData<H>,
}

impl<H: ChainHasher> Default for MmrAccumulator<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: ChainHasher> MmrAccumulator<H> {
    /// Create an empty Merkle Mountain Range.
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            mountains: Vec::new(),
            _hasher: PhantomData,
        }
    }

    /// Number of leaves in the MMR.
    pub fn num_leaves(&self) -> u64 {
        self.leaves.len() as u64
    }

    /// Append a leaf digest to the MMR.
    ///
    /// Returns the 0-based leaf index of the newly added leaf.
    pub fn append(&mut self, leaf: Digest) -> u64 {
        let index = self.leaves.len() as u64;
        self.leaves.push(leaf);

        // A new leaf starts as a single mountain of height 0: levels = [ [leaf] ]
        let mut new_mountain: Vec<Vec<Digest>> = alloc::vec![alloc::vec![leaf]];

        // Merge adjacent mountains of identical height.
        while let Some(last) = self.mountains.last() {
            let last_height = (last.len() - 1) as u32;
            let current_height = (new_mountain.len() - 1) as u32;
            if last_height == current_height {
                let left_mountain = self.mountains.pop().unwrap();
                let right_mountain = new_mountain;

                // Merge: combine levels.
                let mut merged: Vec<Vec<Digest>> = Vec::with_capacity(left_mountain.len() + 1);
                for lvl in 0..left_mountain.len() {
                    let mut combined_level =
                        Vec::with_capacity(left_mountain[lvl].len() + right_mountain[lvl].len());
                    combined_level.extend_from_slice(&left_mountain[lvl]);
                    combined_level.extend_from_slice(&right_mountain[lvl]);
                    merged.push(combined_level);
                }

                // Compute new peak at height current_height + 1.
                let left_peak = left_mountain.last().unwrap()[0];
                let right_peak = right_mountain.last().unwrap()[0];
                let parent_peak = hash_children::<H>(current_height, &left_peak, &right_peak);

                merged.push(alloc::vec![parent_peak]);

                new_mountain = merged;
            } else {
                break;
            }
        }

        self.mountains.push(new_mountain);
        index
    }

    /// Append raw payload bytes as a leaf to the MMR.
    pub fn append_payload(&mut self, payload: &[u8]) -> u64 {
        let digest = hash_leaf_payload::<H>(payload);
        self.append(digest)
    }

    /// Get all mountain peak digests, from largest mountain to smallest.
    pub fn peaks(&self) -> Vec<Digest> {
        self.mountains
            .iter()
            .map(|m| m.last().unwrap()[0])
            .collect()
    }

    /// Compute the overall bagged root of the MMR.
    pub fn root(&self) -> Digest {
        bag_peaks::<H>(&self.peaks())
    }

    /// Generate an inclusion proof for the leaf at `leaf_index`.
    pub fn prove(&self, leaf_index: u64) -> Result<MmrProof, MmrError> {
        let total_leaves = self.num_leaves();
        if leaf_index >= total_leaves {
            return Err(MmrError::IndexOutOfBounds {
                index: leaf_index,
                total_leaves,
            });
        }

        let heights = mountain_heights(total_leaves);
        let mut offset = 0u64;
        let mut target_mountain_idx = 0;
        let mut local_index = 0u64;

        for (idx, &height) in heights.iter().enumerate() {
            let count = 1u64 << height;
            if leaf_index < offset + count {
                target_mountain_idx = idx;
                local_index = leaf_index - offset;
                break;
            }
            offset += count;
        }

        let mountain = &self.mountains[target_mountain_idx];
        let height = mountain.len() - 1;
        let mut siblings = Vec::with_capacity(height);

        let mut level_idx = local_index;
        for level_nodes in mountain.iter().take(height) {
            let is_right_child = (level_idx & 1) == 1;
            let sibling_idx = if is_right_child {
                level_idx - 1
            } else {
                level_idx + 1
            };
            siblings.push(level_nodes[sibling_idx as usize]);
            level_idx >>= 1;
        }

        Ok(MmrProof {
            leaf_index,
            total_leaves,
            siblings,
            peaks: self.peaks(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_mmr_has_zero_root() {
        let mmr = MmrAccumulator::<Fnv256>::new();
        assert_eq!(mmr.num_leaves(), 0);
        assert_eq!(mmr.root(), Digest::ZERO);
        assert_eq!(mmr.peaks().len(), 0);
    }

    #[test]
    fn single_leaf_mmr() {
        let mut mmr = MmrAccumulator::<Fnv256>::new();
        let leaf = Digest([42u8; 32]);
        let idx = mmr.append(leaf);
        assert_eq!(idx, 0);
        assert_eq!(mmr.num_leaves(), 1);
        assert_eq!(mmr.peaks(), vec![leaf]);
        assert_eq!(mmr.root(), leaf);

        let proof = mmr.prove(0).expect("proof generation succeeds");
        assert_eq!(proof.siblings.len(), 0);
        assert_eq!(proof.peaks, vec![leaf]);
        assert!(proof.verify::<Fnv256>(&leaf, &mmr.root()));
    }

    #[test]
    fn power_of_two_and_irregular_leaf_counts() {
        for n in [2, 3, 4, 7, 8, 15, 16, 27, 32] {
            let mut mmr = MmrAccumulator::<Fnv256>::new();
            let mut leaves = Vec::new();
            for i in 0..n {
                let mut d = [0u8; 32];
                d[0] = (i + 1) as u8;
                d[1] = ((i + 1) >> 8) as u8;
                let digest = Digest(d);
                leaves.push(digest);
                mmr.append(digest);
            }

            let root = mmr.root();
            assert_ne!(root, Digest::ZERO);

            // Verify every leaf has a valid proof
            for (i, leaf) in leaves.iter().enumerate() {
                let proof = mmr.prove(i as u64).expect("valid proof");
                assert!(
                    proof.verify::<Fnv256>(leaf, &root),
                    "leaf {} of {} failed verification",
                    i,
                    n
                );
            }
        }
    }

    #[test]
    fn tamper_resistance() {
        let mut mmr = MmrAccumulator::<Fnv256>::new();
        for i in 0..10 {
            mmr.append(Digest([i as u8; 32]));
        }
        let root = mmr.root();
        let target_idx = 4;
        let original_leaf = Digest([target_idx as u8; 32]);
        let proof = mmr.prove(target_idx).expect("proof");

        // 1. Untampered proof passes
        assert!(proof.verify::<Fnv256>(&original_leaf, &root));

        // 2. Tampered leaf fails
        let tampered_leaf = Digest([99u8; 32]);
        assert!(!proof.verify::<Fnv256>(&tampered_leaf, &root));

        // 3. Tampered sibling fails
        let mut bad_proof = proof.clone();
        if !bad_proof.siblings.is_empty() {
            bad_proof.siblings[0] = Digest([0xff; 32]);
            assert!(!bad_proof.verify::<Fnv256>(&original_leaf, &root));
        }

        // 4. Tampered peak fails
        let mut bad_peak_proof = proof.clone();
        bad_peak_proof.peaks[0] = Digest([0xee; 32]);
        assert!(!bad_peak_proof.verify::<Fnv256>(&original_leaf, &root));

        // 5. Tampered leaf index fails
        let mut bad_idx_proof = proof.clone();
        bad_idx_proof.leaf_index = 5;
        assert!(!bad_idx_proof.verify::<Fnv256>(&original_leaf, &root));

        // 6. Tampered total leaves fails
        let mut bad_total_proof = proof.clone();
        bad_total_proof.total_leaves = 11;
        assert!(!bad_total_proof.verify::<Fnv256>(&original_leaf, &root));
    }
}
