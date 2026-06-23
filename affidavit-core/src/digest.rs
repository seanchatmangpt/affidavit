//! Content-address digests and the pluggable chain-hash trait.
//!
//! The verifier is hash-agnostic: it folds a receipt chain through whatever
//! [`ChainHasher`] you give it. A zero-dependency reference hasher, [`Fnv256`],
//! ships in-crate so the whole thing builds and tests with no external crates;
//! for production, implement [`ChainHasher`] over a cryptographic hash (e.g.
//! BLAKE3) to get a tamper-*evident* (not merely tamper-*detecting*) chain.

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
