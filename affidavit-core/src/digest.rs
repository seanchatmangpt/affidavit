//! Content-address digests and the pluggable chain-hash trait.
//!
//! The verifier is hash-agnostic: it folds a receipt chain through whatever
//! [`ChainHasher`] you give it. A zero-dependency reference hasher, [`Fnv256`],
//! ships in-crate so the whole thing builds and tests with no external crates;
//! for production, use the cryptographic [`Blake3Hasher`] (or implement
//! [`ChainHasher`] over another external crate) to get a tamper-*evident* (not
//! merely tamper-*detecting*) chain.
//!
//! **No cryptographic algorithm is implemented in this crate.** This is an
//! operator law, in force crate-wide: hashing and post-quantum verification
//! exist only as trait seams ([`ChainHasher`], and the PQC verifiers in
//! `crate::pqc`); the algorithms themselves come exclusively from external
//! dependencies — [`Blake3Hasher`] delegates to the unconditional `blake3`
//! crate, and nothing here re-implements a single round of any cipher or hash.
//! Dropping in a hasher means implementing [`ChainHasher`] over an external
//! crate and switching the turbofish — nothing else changes.

/// A 32-byte content-address digest.
///
/// Deliberately a fixed-size value type: it is `Copy`, allocation-free, and
/// usable in `no_std` without `alloc`. Equality is constant-shape (not
/// constant-*time*); this is a structural verifier, not a secrets-handling one.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    /// The all-zero digest, used as the "unset" sentinel the profile check rejects.
    pub const ZERO: Digest = Digest([0u8; 32]);

    /// Borrow the raw 32 bytes.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// True iff every byte is zero (i.e. this is [`Digest::ZERO`]).
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

/// A streaming hash used to fold the receipt chain.
///
/// Implementations are stateless markers (e.g. [`Fnv256`]); the running state
/// lives in [`ChainHasher::State`]. The three-call shape (`init` → `absorb`* →
/// `finish`) is enough to hash structured data without allocating an
/// intermediate buffer, which keeps the verify path `no_std` + no-`alloc`.
pub trait ChainHasher {
    /// The running hash state.
    type State;

    /// Start a fresh hash state.
    fn init() -> Self::State;

    /// Absorb a byte slice into the state.
    fn absorb(state: &mut Self::State, bytes: &[u8]);

    /// Finalize the state into a [`Digest`].
    fn finish(state: Self::State) -> Digest;
}

/// A zero-dependency, **non-cryptographic** reference hasher (4-lane FNV-1a with
/// a SplitMix64 finalizer), producing a 256-bit [`Digest`].
///
/// It is deterministic and diffuses input well enough to detect tampering in
/// tests and demos, but it is **not** collision-resistant. Swap in a
/// [`ChainHasher`] backed by a cryptographic hash for any adversarial setting —
/// the rest of the crate is unchanged.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fnv256;

const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const FNV_BASIS: [u64; 4] = [
    0xcbf2_9ce4_8422_2325,
    0x9e37_79b9_7f4a_7c15,
    0x5851_f42d_4c95_7f2d,
    0x1405_7b7e_f767_814f,
];

/// SplitMix64 finalizer — cheap, well-studied avalanche for the output lanes.
#[inline]
fn splitmix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

impl ChainHasher for Fnv256 {
    type State = [u64; 4];

    #[inline]
    fn init() -> [u64; 4] {
        FNV_BASIS
    }

    #[inline]
    fn absorb(state: &mut [u64; 4], bytes: &[u8]) {
        for &b in bytes {
            for (lane_idx, lane) in state.iter_mut().enumerate() {
                let mixed =
                    (*lane ^ (b as u64).wrapping_add(lane_idx as u64)).wrapping_mul(FNV_PRIME);
                // Per-lane rotation makes the four lanes diverge so the 256-bit
                // output is not just the 64-bit hash repeated.
                *lane = mixed.rotate_left((lane_idx as u32 * 7 + 1) & 63);
            }
        }
    }

    #[inline]
    fn finish(state: [u64; 4]) -> Digest {
        let mut out = [0u8; 32];
        for (lane_idx, &lane) in state.iter().enumerate() {
            let v = splitmix(lane ^ (lane_idx as u64).wrapping_mul(FNV_PRIME));
            out[lane_idx * 8..lane_idx * 8 + 8].copy_from_slice(&v.to_be_bytes());
        }
        Digest(out)
    }
}

/// A **cryptographic** chain hasher delegating entirely to the external
/// `blake3` crate (unconditional dependency, `default-features = false`).
///
/// Per the no-in-crate-crypto operator law, no part of the BLAKE3 algorithm is
/// implemented here: [`ChainHasher::State`] is `blake3::Hasher`, and the three
/// seam calls map 1:1 onto `blake3::Hasher::new` / `update` / `finalize`.
/// Finalization takes the first 32 output bytes (the BLAKE3 default
/// `OUT_LEN = 32`), which is exactly the plain `finalize()` hash.
///
/// Unlike [`Fnv256`], this hasher is collision-resistant, so chains folded
/// through it are tamper-*evident* against adversaries, not just against
/// accidental corruption.
#[derive(Clone, Copy, Debug, Default)]
pub struct Blake3Hasher;

impl ChainHasher for Blake3Hasher {
    /// The external BLAKE3 running state.
    type State = blake3::Hasher;

    #[inline]
    fn init() -> Self::State {
        blake3::Hasher::new()
    }

    #[inline]
    fn absorb(state: &mut Self::State, bytes: &[u8]) {
        blake3::Hasher::update(state, bytes);
    }

    #[inline]
    fn finish(state: Self::State) -> Digest {
        Digest(*state.finalize().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::compute_chain_hash;
    use crate::Event;

    fn event(seq: u64, id: &'static str, commitment: Digest) -> Event<'static> {
        Event {
            seq,
            event_id: id,
            event_type: "build",
            commitment,
        }
    }

    /// Known-answer vector: `blake3(b"abc")`, folded through the
    /// `ChainHasher` interface with the **external** `blake3` implementation —
    /// proves the seam admits a real cryptographic algorithm without any
    /// in-crate crypto. Expected value is taken from the dependency itself
    /// (`blake3::hash`), not hardcoded, so the vector stays honest if the
    /// pinned `blake3` version ever changes.
    #[test]
    fn external_blake3_chain_hasher_kat_abc() {
        let mut s = Blake3Hasher::init();
        Blake3Hasher::absorb(&mut s, b"abc");
        let d = Blake3Hasher::finish(s);
        assert_eq!(d.as_bytes(), blake3::hash(b"abc").as_bytes());
        // One-shot blake3 must equal the streamed seam path byte-for-byte.
        let mut streamed = blake3::Hasher::new();
        streamed.update(b"a");
        streamed.update(b"bc");
        assert_eq!(d.as_bytes(), streamed.finalize().as_bytes());
        assert_ne!(d, Fnv256::finish(Fnv256::init()));
    }

    /// Hasher identity: folding the same bytes through [`Fnv256`] and
    /// [`Blake3Hasher`] must produce different digests — the turbofish choice
    /// is visible in the output.
    #[test]
    fn hasher_identity_fnv_vs_blake3_differ() {
        let mut s = Blake3Hasher::init();
        Blake3Hasher::absorb(&mut s, b"abc");
        let b3 = Blake3Hasher::finish(s);

        let mut f = Fnv256::init();
        Fnv256::absorb(&mut f, b"abc");
        let fnv = Fnv256::finish(f);

        assert_ne!(b3, fnv);
        assert_ne!(
            compute_chain_hash::<Blake3Hasher>(&[]),
            compute_chain_hash::<Fnv256>(&[]),
        );
    }

    /// Two-event chain hash is deterministic: same events + same hasher =>
    /// byte-identical head, on both hashers (in-crate reference + external).
    #[test]
    fn two_event_chain_hash_deterministic() {
        let mk = || {
            vec![
                event(0, "evt-0", Digest([1u8; 32])),
                event(1, "evt-1", Digest([2u8; 32])),
            ]
        };
        let a = compute_chain_hash::<Blake3Hasher>(&mk());
        let b = compute_chain_hash::<Blake3Hasher>(&mk());
        let f = compute_chain_hash::<Fnv256>(&mk());
        assert_eq!(a, b, "same events+hasher must reproduce the same head");
        assert!(!a.is_zero(), "head must not be the unset sentinel");
        assert_ne!(a, f, "hasher identity must be visible in the head");
    }
}
