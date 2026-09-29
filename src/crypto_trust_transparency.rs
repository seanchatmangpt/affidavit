//! RFC 9162-style append-only transparency log over envelope commitments for the
//! affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Law (RFC 9162 §2.1, Merkle Tree Hashes, with the RFC's SHA-256 HASH replaced
//! by domain-separated BLAKE3 under ctp:merkleDomain so no digest is valid in
//! any other domain or plane):
//! - MTH of the empty set = HASH()
//! - MTH of a single leaf set containing d0 = HASH(0x00 || d0)
//! - MTH of D[n] = HASH(0x01 || MTH(D[0:k]) || MTH(D[k:n])), k = largest power
//!   of two smaller than n
//!
//! The log only grows ([`TransparencyLog::append`]); heads are a pure function
//! of the leaf prefix, so a previously published head never moves (tested) and
//! inclusion/consistency proofs against it stay verifiable forever.
//!
//! House law: refusal-as-value. Out-of-range leaf indices and consistency sizes
//! are typed [`TransparencyError`] variants; nothing here panics, coerces, or
//! fakes. Verification is total over its inputs and returns `false` on any
//! malformed proof instead of refusing to answer.

use crate::crypto_trust_canonical::digest;

/// Log wire-format identity (ctp:transparency-v1 ctp:logFormat).
pub const LOG_FORMAT: &str = "CTP-TRANS-v1";

/// Domain-separation domain for the Merkle Tree Hash HASH function
/// (ctp:transparency-v1 ctp:merkleDomain). This is THE hash domain of the log:
/// every 0x00/0x01-prefixed tree-hash digest below runs through
/// [`crate::crypto_trust_canonical::digest`] with this domain.
pub const MERKLE_DOMAIN: &str = "affidavit.crypto-trust-plane.v1/merkle";

/// Domain bound to inclusion-proof evidence (ctp:transparency-v1
/// ctp:inclusionDomain). Distinct from [`MERKLE_DOMAIN`] and
/// [`CONSISTENCY_DOMAIN`] by graph law; the domain-coherence test holds the
/// rendered constants to the shared domain-tag prefix.
pub const INCLUSION_DOMAIN: &str = "affidavit.crypto-trust-plane.v1/inclusion";

/// Domain bound to consistency-proof evidence (ctp:transparency-v1
/// ctp:consistencyDomain). Distinct from [`MERKLE_DOMAIN`] and
/// [`INCLUSION_DOMAIN`] by graph law; the domain-coherence test holds the
/// rendered constants to the shared domain-tag prefix.
pub const CONSISTENCY_DOMAIN: &str = "affidavit.crypto-trust-plane.v1/consistency";

/// Typed refusal of an inadmissible proof request over the log.
#[derive(Debug, thiserror::Error)]
pub enum TransparencyError {
    /// The requested leaf index does not exist in the log.
    #[error("leaf index {0} out of range")]
    LeafOutOfRange(u64),
    /// The requested consistency size is not a valid prefix of the log.
    #[error("consistency size {0} exceeds log size {1}")]
    SizeOutOfRange(u64, u64),
}

/// The log's HASH function (RFC 9162 §2.1.1): BLAKE3 over the
/// [`MERKLE_DOMAIN`]-separated pre-image, via the canonical module.
fn hash(parts: &[&[u8]]) -> [u8; 32] {
    digest(MERKLE_DOMAIN, parts)
}

/// The RFC 9162 leaf tag: the one-leaf MTH clause, HASH(0x00 || d0).
fn leaf_hash(leaf: &[u8; 32]) -> [u8; 32] {
    hash(&[&[0x00u8], leaf])
}

/// The RFC 9162 node tag: MTH(D[n]) = HASH(0x01 || left || right).
fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    hash(&[&[0x01u8], left, right])
}

/// k = largest power of two smaller than n (RFC 9162 §2.1.1). Caller
/// guarantees n >= 2, so k >= 1.
fn split_point(n: usize) -> usize {
    let mut k = 1;
    while k * 2 < n {
        k *= 2;
    }
    k
}

/// Merkle Tree Hash of `leaves[lo..hi]` (RFC 9162 §2.1.1) — exactly the
/// three-clause MTH definition: empty, single leaf (0x00 tag), split at k.
fn mth_range(leaves: &[[u8; 32]], lo: usize, hi: usize) -> [u8; 32] {
    match hi - lo {
        0 => hash(&[]),
        1 => leaf_hash(&leaves[lo]),
        n => {
            let k = split_point(n);
            let left = mth_range(leaves, lo, lo + k);
            let right = mth_range(leaves, lo + k, hi);
            node_hash(&left, &right)
        }
    }
}

/// RFC 9162 §2.1.3.1 PATH(m, D[lo:hi]) recursion: sibling node hashes from the
/// leaf up, ordered leaf-ward first.
fn path_range(leaves: &[[u8; 32]], lo: usize, hi: usize, m: usize) -> Vec<[u8; 32]> {
    if hi - lo == 1 {
        return Vec::new();
    }
    let k = split_point(hi - lo);
    let mut p = if m < lo + k {
        path_range(leaves, lo, lo + k, m)
    } else {
        path_range(leaves, lo + k, hi, m)
    };
    if m < lo + k {
        p.push(mth_range(leaves, lo + k, hi));
    } else {
        p.push(mth_range(leaves, lo, lo + k));
    }
    p
}

/// RFC 9162 §2.1.4.1 SUBPROOF(m, D[lo:hi], b) recursion for consistency
/// proofs. `b` is the seed flag: the root of the first tree contributes
/// nothing when reached exactly (`m == hi - lo`, `b` true) because the
/// verifier re-supplies it as `first_hash`.
fn subproof_range(leaves: &[[u8; 32]], lo: usize, hi: usize, m: usize, b: bool) -> Vec<[u8; 32]> {
    if hi - lo == m {
        if b {
            Vec::new()
        } else {
            vec![mth_range(leaves, lo, hi)]
        }
    } else {
        let k = split_point(hi - lo);
        if m <= k {
            let mut p = subproof_range(leaves, lo, lo + k, m, b);
            p.push(mth_range(leaves, lo + k, hi));
            p
        } else {
            let mut p = subproof_range(leaves, lo + k, hi, m - k, false);
            p.push(mth_range(leaves, lo, lo + k));
            p
        }
    }
}

/// An append-only Merkle log of 32-byte commitments (e.g. envelope
/// commitment digests). Leaf i is the i-th appended commitment itself; the
/// RFC 9162 0x00 leaf tag is applied inside the tree hash, not at append
/// time, so a stored leaf is always the caller's exact bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TransparencyLog {
    leaves: Vec<[u8; 32]>,
}

impl TransparencyLog {
    /// An empty log.
    pub fn new() -> Self {
        Self { leaves: Vec::new() }
    }

    /// Append `commitment` and return its leaf index (the previous size).
    pub fn append(&mut self, commitment: [u8; 32]) -> u64 {
        let index = self.leaves.len() as u64;
        self.leaves.push(commitment);
        index
    }

    /// Number of leaves in the log.
    pub fn len(&self) -> u64 {
        self.leaves.len() as u64
    }

    /// True while nothing has been appended.
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// The leaves observed at construction (read-only access for auditors).
    pub fn leaf(&self, index: u64) -> Option<&[u8; 32]> {
        self.leaves.get(index as usize)
    }

    /// The current Merkle Tree Hash (RFC 9162 §2.1.1). The empty log's head
    /// is HASH() — defined, deterministic, and the anchor every first proof
    /// is measured from.
    pub fn head(&self) -> [u8; 32] {
        mth_range(&self.leaves, 0, self.leaves.len())
    }

    /// RFC 9162 §2.1.3.1 inclusion proof (PATH recursion) for leaf `idx`
    /// against the current tree. Refused as
    /// [`TransparencyError::LeafOutOfRange`] when `idx` is not an existing
    /// leaf.
    pub fn inclusion_proof(&self, idx: u64) -> Result<InclusionProof, TransparencyError> {
        if idx >= self.len() {
            return Err(TransparencyError::LeafOutOfRange(idx));
        }
        Ok(InclusionProof {
            leaf_index: idx,
            tree_size: self.len(),
            path: path_range(&self.leaves, 0, self.leaves.len(), idx as usize),
        })
    }

    /// RFC 9162 §2.1.4.1 consistency proof (SUBPROOF recursion) that the
    /// first `first` leaves of the CURRENT tree are a prefix commitment —
    /// i.e. binding the current head to the head of the first-`first`-leaf
    /// tree. Refused as [`TransparencyError::SizeOutOfRange`] when `first`
    /// is 0 or beyond the current size.
    pub fn consistency_proof(&self, first: u64) -> Result<ConsistencyProof, TransparencyError> {
        let n = self.len();
        if first == 0 || first > n {
            return Err(TransparencyError::SizeOutOfRange(first, n));
        }
        Ok(ConsistencyProof {
            first,
            second: n,
            path: subproof_range(&self.leaves, 0, self.leaves.len(), first as usize, true),
        })
    }
}

/// RFC 9162 §2.1.3 inclusion proof: the sibling-node path from leaf
/// `leaf_index` of a `tree_size`-leaf tree up to the root.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InclusionProof {
    pub leaf_index: u64,
    pub tree_size: u64,
    pub path: Vec<[u8; 32]>,
}

/// RFC 9162 §2.1.3.2 inclusion verification, with the RFC's SHA-256 HASH
/// replaced by the [`MERKLE_DOMAIN`]-separated hash. Total: any malformed
/// proof (bad index, short/long path) returns `false`, never panics.
pub fn verify_inclusion(leaf: &[u8; 32], proof: &InclusionProof, head: &[u8; 32]) -> bool {
    // Step 1: the leaf must exist in the claimed tree.
    if proof.leaf_index >= proof.tree_size {
        return false;
    }
    let mut fn_node = proof.leaf_index;
    let mut sn_node = proof.tree_size - 1;
    let mut r = leaf_hash(leaf);
    for p in &proof.path {
        if sn_node == 0 {
            // Step 4 guard: the path is longer than the tree is deep.
            return false;
        }
        if fn_node & 1 == 1 || fn_node == sn_node {
            // Step 4.1: sibling on the left (or this node is the rightmost
            // subtree root): r = HASH(0x01 || p || r).
            r = node_hash(p, &r);
            // Step 4.1.2: when fn was even (pure fn == sn case), right-shift
            // both equally until LSB(fn) is set or fn is 0.
            if fn_node & 1 == 0 {
                while fn_node & 1 == 0 && fn_node != 0 {
                    fn_node >>= 1;
                    sn_node >>= 1;
                }
            }
        } else {
            // Step 4.2: sibling on the right: r = HASH(0x01 || r || p).
            r = node_hash(&r, p);
        }
        // Step 4 final: right-shift both one time.
        fn_node >>= 1;
        sn_node >>= 1;
    }
    // Step 5: the path must be exactly deep enough and land on the head.
    sn_node == 0 && &r == head
}

/// RFC 9162 §2.1.4 consistency proof between the first-`first`-leaf tree and
/// the `second`-leaf tree of one log.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsistencyProof {
    pub first: u64,
    pub second: u64,
    pub path: Vec<[u8; 32]>,
}

/// RFC 9162 §2.1.4.2 consistency verification, with the RFC's SHA-256 HASH
/// replaced by the [`MERKLE_DOMAIN`]-separated hash. Total: any malformed
/// proof returns `false`, never panics. A proof whose sizes are equal is the
/// RFC's empty SUBPROOF case and succeeds exactly when both heads match.
pub fn verify_consistency(
    proof: &ConsistencyProof,
    first_head: &[u8; 32],
    second_head: &[u8; 32],
) -> bool {
    let (first, second) = (proof.first, proof.second);
    // Only 0 < first <= second is a well-formed consistency claim
    // (RFC 9162 §2.1.4.2 preamble).
    if first == 0 || first > second {
        return false;
    }
    if first == second {
        // Empty proof: the two claims are the same tree or they are not.
        return proof.path.is_empty() && first_head == second_head;
    }
    // Step 1+2: an exact power-of-two first tree is re-supplied by the
    // verifier — prepend first_hash to the path.
    let mut path: Vec<[u8; 32]> = Vec::with_capacity(proof.path.len() + 1);
    if first.is_power_of_two() {
        path.push(*first_head);
    }
    path.extend_from_slice(&proof.path);
    // Step 1 (RFC wording): an empty path cannot bind anything.
    let Some(mut fr) = path.first().copied() else {
        return false;
    };
    let mut sr = fr;
    // Step 3: right-shift both equally while LSB(fn) is set.
    let mut fn_node = first - 1;
    let mut sn_node = second - 1;
    while fn_node & 1 == 1 {
        fn_node >>= 1;
        sn_node >>= 1;
    }
    // Step 5: walk the remaining path values.
    for c in path.iter().skip(1) {
        if sn_node == 0 {
            return false;
        }
        if fn_node & 1 == 1 || fn_node == sn_node {
            // fr = HASH(0x01 || c || fr); sr = HASH(0x01 || c || sr).
            fr = node_hash(c, &fr);
            sr = node_hash(c, &sr);
            if fn_node & 1 == 0 {
                while fn_node & 1 == 0 && fn_node != 0 {
                    fn_node >>= 1;
                    sn_node >>= 1;
                }
            }
        } else {
            // sr = HASH(0x01 || sr || c); fr is untouched.
            sr = node_hash(&sr, c);
        }
        // Right-shift both one time.
        fn_node >>= 1;
        sn_node >>= 1;
    }
    // Step 6: fr must reconstruct first_hash, sr second_hash, path exact.
    sn_node == 0 && &fr == first_head && &sr == second_head
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DOMAIN_TAG;

    /// Deterministic pseudo-commitments: digest of the index under the log's
    /// own merkle domain (distinct, reproducible test leaves).
    fn leaf(i: u64) -> [u8; 32] {
        digest(MERKLE_DOMAIN, &[i.to_be_bytes().as_slice()])
    }

    fn log_with(n: u64) -> TransparencyLog {
        let mut log = TransparencyLog::new();
        for i in 0..n {
            log.append(leaf(i));
        }
        log
    }

    // -- teeth first: every leaf verifies against the head -------------------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(LOG_FORMAT, "CTP-TRANS-v1");
        assert_eq!(MERKLE_DOMAIN, "affidavit.crypto-trust-plane.v1/merkle");
        assert_eq!(
            INCLUSION_DOMAIN,
            "affidavit.crypto-trust-plane.v1/inclusion"
        );
        assert_eq!(
            CONSISTENCY_DOMAIN,
            "affidavit.crypto-trust-plane.v1/consistency"
        );
    }

    #[test]
    fn domain_tag_is_owned_by_canonical_module_and_domains_cohere() {
        // The graph's domain tag must be exactly the canonical module's
        // single-owner DOMAIN_TAG (no second copy may drift).
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        // All three proof domains live under it and are pairwise distinct.
        for domain in [MERKLE_DOMAIN, INCLUSION_DOMAIN, CONSISTENCY_DOMAIN] {
            let prefix = format!("{DOMAIN_TAG}/");
            assert!(
                domain.starts_with(&prefix),
                "domain {domain} must carry the plane's domain-tag prefix"
            );
        }
        assert_ne!(MERKLE_DOMAIN, INCLUSION_DOMAIN);
        assert_ne!(MERKLE_DOMAIN, CONSISTENCY_DOMAIN);
        assert_ne!(INCLUSION_DOMAIN, CONSISTENCY_DOMAIN);
    }

    #[test]
    fn every_leaf_of_every_size_has_a_valid_inclusion_proof() {
        let n = 9u64; // 9 forces an unbalanced k=8 root split on the right
        let log = log_with(n);
        let head = log.head();
        assert_eq!(log.len(), n);
        assert!(!log.is_empty());
        for i in 0..n {
            let proof = log
                .inclusion_proof(i)
                .unwrap_or_else(|e| panic!("leaf {i} must have a proof: {e}"));
            assert_eq!(proof.leaf_index, i);
            assert_eq!(proof.tree_size, n);
            assert!(
                verify_inclusion(&leaf(i), &proof, &head),
                "leaf {i} must verify against the head"
            );
            // A different leaf must NOT verify this proof.
            let other = leaf((i + 1) % n);
            assert!(
                !verify_inclusion(&other, &proof, &head),
                "leaf {i}'s proof must refuse leaf {}",
                (i + 1) % n
            );
        }
    }

    #[test]
    fn append_returns_sequential_leaf_indices() {
        let mut log = TransparencyLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        for i in 0..5 {
            assert_eq!(log.append(leaf(i)), i);
            assert_eq!(log.len(), i + 1);
        }
        assert!(!log.is_empty());
        assert_eq!(log.leaf(3), Some(&leaf(3)));
        assert_eq!(log.leaf(5), None);
    }

    // -- RFC 9162 §2.1.1 structure vectors, recomputed directly --------------

    #[test]
    fn empty_tree_head_is_hash_of_nothing() {
        let log = TransparencyLog::new();
        let expected = digest(MERKLE_DOMAIN, &[]);
        assert_eq!(log.head(), expected);
        assert_ne!(expected, [0u8; 32]);
    }

    #[test]
    fn single_leaf_head_is_hash_of_0x00_and_leaf() {
        let log = log_with(1);
        let d0 = leaf(0);
        let expected = digest(MERKLE_DOMAIN, &[&[0x00u8], &d0]);
        assert_eq!(log.head(), expected);
    }

    #[test]
    fn two_leaf_head_is_hash_of_0x01_and_both_leaf_hashes() {
        let log = log_with(2);
        let h0 = digest(MERKLE_DOMAIN, &[&[0x00u8], &leaf(0)]);
        let h1 = digest(MERKLE_DOMAIN, &[&[0x00u8], &leaf(1)]);
        let expected = digest(MERKLE_DOMAIN, &[&[0x01u8], &h0, &h1]);
        assert_eq!(log.head(), expected);
    }

    #[test]
    fn three_leaf_head_splits_at_k_equals_two() {
        // MTH(D[3]) = HASH(0x01 || MTH(D[0:2]) || MTH(D[2:3])) — k = 2.
        let log = log_with(3);
        let h01 = log_with(2).head();
        let h2 = digest(MERKLE_DOMAIN, &[&[0x00u8], &leaf(2)]);
        let expected = digest(MERKLE_DOMAIN, &[&[0x01u8], &h01, &h2]);
        assert_eq!(log.head(), expected);
    }

    #[test]
    fn head_is_deterministic_and_domain_bound() {
        let a = log_with(6).head();
        let b = log_with(6).head();
        assert_eq!(a, b);
        // The same leaves hashed under the sibling domains are different
        // digests: the merkle domain is the only valid head domain.
        let log = log_with(1);
        let d0 = log.leaf(0).expect("leaf 0");
        assert_ne!(log.head(), digest(INCLUSION_DOMAIN, &[&[0x00u8], d0]));
        assert_ne!(log.head(), digest(CONSISTENCY_DOMAIN, &[&[0x00u8], d0]));
    }

    // -- inclusion refusals and tamper teeth ----------------------------------

    #[test]
    fn inclusion_proof_refuses_out_of_range_leaf() {
        let log = log_with(4);
        match log.inclusion_proof(4) {
            Err(TransparencyError::LeafOutOfRange(idx)) => assert_eq!(idx, 4),
            outcome => panic!("expected LeafOutOfRange(4), got {outcome:?}"),
        }
        let empty = TransparencyLog::new();
        match empty.inclusion_proof(0) {
            Err(TransparencyError::LeafOutOfRange(idx)) => assert_eq!(idx, 0),
            outcome => panic!("expected LeafOutOfRange(0), got {outcome:?}"),
        }
    }

    #[test]
    fn error_display_strings_are_wire_stable() {
        assert_eq!(
            TransparencyError::LeafOutOfRange(7).to_string(),
            "leaf index 7 out of range"
        );
        assert_eq!(
            TransparencyError::SizeOutOfRange(9, 4).to_string(),
            "consistency size 9 exceeds log size 4"
        );
    }

    #[test]
    fn tampered_inclusion_path_fails() {
        let log = log_with(8);
        let head = log.head();
        let mut proof = log.inclusion_proof(5).expect("proof exists");
        assert!(verify_inclusion(&leaf(5), &proof, &head));
        let last = proof.path.len() - 1;
        proof.path[last][0] ^= 0x01;
        assert!(!verify_inclusion(&leaf(5), &proof, &head));
    }

    #[test]
    fn truncated_and_extended_inclusion_paths_fail() {
        let log = log_with(8);
        let head = log.head();
        let mut proof = log.inclusion_proof(6).expect("proof exists");
        proof.path.pop();
        assert!(!verify_inclusion(&leaf(6), &proof, &head));
        proof.path.push([0u8; 32]);
        assert!(!verify_inclusion(&leaf(6), &proof, &head));
    }

    #[test]
    fn wrong_leaf_wrong_head_and_bad_index_fail_inclusion() {
        let log = log_with(5);
        let head = log.head();
        let proof = log.inclusion_proof(2).expect("proof exists");
        // Wrong leaf.
        assert!(!verify_inclusion(&leaf(1), &proof, &head));
        // Wrong head.
        let mut other_head = head;
        other_head[31] ^= 0x01;
        assert!(!verify_inclusion(&leaf(2), &proof, &other_head));
        // Self-inconsistent index/size claim.
        let bogus = InclusionProof {
            leaf_index: 5,
            tree_size: 5,
            path: proof.path.clone(),
        };
        assert!(!verify_inclusion(&leaf(0), &bogus, &head));
    }

    // -- consistency: all pairwise sizes verify, tamper refused ---------------

    #[test]
    fn consistency_between_all_prefix_sizes_of_1_to_8_verifies() {
        for n in 1..=8u64 {
            let full = log_with(n);
            let second_head = full.head();
            for m in 1..=n {
                let proof = full
                    .consistency_proof(m)
                    .unwrap_or_else(|e| panic!("consistency {m}->{n} must exist: {e}"));
                assert_eq!(proof.first, m);
                assert_eq!(proof.second, n);
                let first_head = log_with(m).head();
                assert!(
                    verify_consistency(&proof, &first_head, &second_head),
                    "consistency {m} -> {n} must verify"
                );
                // Swapped heads must fail (distinct only when m < n).
                if m < n {
                    assert!(
                        !verify_consistency(&proof, &second_head, &first_head),
                        "consistency {m} -> {n} must refuse swapped heads"
                    );
                }
            }
        }
    }

    #[test]
    fn consistency_refuses_out_of_range_sizes() {
        let log = log_with(4);
        match log.consistency_proof(0) {
            Err(TransparencyError::SizeOutOfRange(0, 4)) => {}
            outcome => panic!("expected SizeOutOfRange(0, 4), got {outcome:?}"),
        }
        match log.consistency_proof(5) {
            Err(TransparencyError::SizeOutOfRange(5, 4)) => {}
            outcome => panic!("expected SizeOutOfRange(5, 4), got {outcome:?}"),
        }
        let empty = TransparencyLog::new();
        match empty.consistency_proof(1) {
            Err(TransparencyError::SizeOutOfRange(1, 0)) => {}
            outcome => panic!("expected SizeOutOfRange(1, 0), got {outcome:?}"),
        }
    }

    #[test]
    fn tampered_consistency_path_fails() {
        let log = log_with(6);
        let first_head = log_with(2).head();
        let second_head = log.head();
        let mut proof = log.consistency_proof(2).expect("proof exists");
        assert!(verify_consistency(&proof, &first_head, &second_head));
        proof.path[0][7] ^= 0x80;
        assert!(!verify_consistency(&proof, &first_head, &second_head));
    }

    #[test]
    fn malformed_consistency_claims_fail() {
        let log = log_with(4);
        let first_head = log_with(2).head();
        let second_head = log.head();
        // first > second is never a valid claim.
        let backwards = ConsistencyProof {
            first: 3,
            second: 2,
            path: Vec::new(),
        };
        assert!(!verify_consistency(&backwards, &first_head, &second_head));
        // Zero-sized first tree is refused.
        let zero = ConsistencyProof {
            first: 0,
            second: 4,
            path: vec![first_head],
        };
        assert!(!verify_consistency(&zero, &first_head, &second_head));
        // Equal sizes succeed only with the empty proof and equal heads.
        let same = log.consistency_proof(4).expect("proof exists");
        assert!(same.path.is_empty());
        assert!(verify_consistency(&same, &second_head, &second_head));
        assert!(!verify_consistency(&same, &first_head, &second_head));
    }

    // -- append-only law: history never moves ---------------------------------

    #[test]
    fn published_head_and_proofs_survive_further_appends() {
        let mut log = TransparencyLog::new();
        for i in 0..5 {
            log.append(leaf(i));
        }
        let head_at_5 = log.head();
        let proofs: Vec<InclusionProof> = (0..5)
            .map(|i| log.inclusion_proof(i).expect("proof exists"))
            .collect();
        let consistency_1_to_5 = log.consistency_proof(1).expect("proof exists");
        let consistency_3_to_5 = log.consistency_proof(3).expect("proof exists");

        // Grow the log.
        for i in 5..9 {
            log.append(leaf(i));
        }
        assert_eq!(log.len(), 9);

        // The old head is still exactly the head of the unchanged prefix.
        assert_eq!(head_at_5, log_with(5).head());
        // Every proof minted before the appends still verifies — against the
        // old head, with no recomputation of history.
        for (i, proof) in proofs.iter().enumerate() {
            assert!(
                verify_inclusion(&leaf(i as u64), proof, &head_at_5),
                "pre-append proof for leaf {i} must survive growth"
            );
        }
        let head_at_1 = log_with(1).head();
        let head_at_3 = log_with(3).head();
        assert!(verify_consistency(
            &consistency_1_to_5,
            &head_at_1,
            &head_at_5
        ));
        assert!(verify_consistency(
            &consistency_3_to_5,
            &head_at_3,
            &head_at_5
        ));

        // And the grown log binds the old head through a fresh consistency
        // proof: 5 -> 9 witnesses append-only extension, not a rehash.
        let bridge = log.consistency_proof(5).expect("proof exists");
        assert!(verify_consistency(&bridge, &head_at_5, &log.head()));
    }

    #[test]
    fn growing_log_changes_head_deterministically() {
        let mut a = TransparencyLog::new();
        let mut b = TransparencyLog::new();
        for i in 0..6 {
            a.append(leaf(i));
            b.append(leaf(i));
            assert_eq!(a.head(), b.head(), "same prefix must give same head");
        }
        a.append(leaf(99));
        assert_ne!(a.head(), b.head());
        b.append(leaf(99));
        assert_eq!(a.head(), b.head());
    }

    // -- proof structs round-trip their wire form ------------------------------

    #[test]
    fn proof_structs_round_trip() {
        let log = log_with(7);
        let inclusion = log.inclusion_proof(3).expect("proof exists");
        let encoded = serde_json::to_string(&inclusion).expect("serializes");
        let decoded: InclusionProof = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded, inclusion);

        let consistency = log.consistency_proof(4).expect("proof exists");
        let encoded = serde_json::to_string(&consistency).expect("serializes");
        let decoded: ConsistencyProof = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded, consistency);
    }
}
