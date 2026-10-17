//! Content-address digests and the pluggable chain-hash trait.
//!
//! The verifier is hash-agnostic: it folds a receipt chain through whatever
//! [`ChainHasher`] you give it. An in-crate reference hasher (no crypto, no
//! unsafe), [`Fnv256`], ships here so the whole thing builds and tests
//! without a cryptographic crate;
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

/// An in-crate reference hasher (no crypto, no unsafe), **non-cryptographic** (4-lane FNV-1a with
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

/// Keyed domain separation over the hasher seam: derive a digest from an
/// input bound to a fixed context string.
///
/// This is the KDF-flavored extension of [`ChainHasher`]. The default folds
/// `context || input` through the implementing hasher's own three-call
/// interface, so any [`ChainHasher`] gets a `derive` for free; a cryptographic
/// hasher is expected to override it with its dependency's real keyed/derive
/// mode (see [`Blake3Hasher`]).
///
/// **No cryptographic algorithm is implemented in this crate**: the BLAKE3
/// override delegates entirely to the external `blake3` crate's
/// `Hasher::new_derive_key` (keyed mode), and [`Fnv256`]'s inherited default
/// is — like all of [`Fnv256`] — **non-cryptographic** and must not be used
/// adversarially.
pub trait ChainKdf {
    /// Derive a [`Digest`] from `input` under `context`.
    ///
    /// Two different contexts applied to the same input must yield different
    /// digests (domain separation is real, not cosmetic). The path is
    /// allocation-free beyond what the underlying hasher itself uses.
    fn derive(&self, input: &[u8], context: &str) -> Digest;

    /// Extend the derivation into an arbitrary-length XOF stream.
    ///
    /// Fills `out` entirely and also returns the fixed 32-byte [`Digest`]
    /// for the same (input, context). The XOF stream is prefix-consistent:
    /// the first 32 bytes of any fill must equal [`ChainKdf::derive`] for
    /// the same arguments.
    ///
    /// Implementations that cannot offer a real extendable stream pin their
    /// typed standing here rather than silently emitting a wrong one.
    fn derive_xof(&self, input: &[u8], context: &str, out: &mut [u8]) -> Digest;

    /// Inherited default: fold `context || input` through this hasher's own
    /// [`ChainHasher`] interface. Deterministic and diffusing, but carries
    /// whatever (non-)cryptographic standing the hasher itself has.
    fn derive_default(input: &[u8], context: &str) -> Digest
    where
        Self: ChainHasher,
    {
        let mut state = Self::init();
        Self::absorb(&mut state, context.as_bytes());
        Self::absorb(&mut state, input);
        Self::finish(state)
    }
}

impl ChainKdf for Blake3Hasher {
    /// Delegates entirely to the external `blake3` crate: keyed
    /// (`new_derive_key`) mode, then `finalize_xof().fill(out)`. No BLAKE3
    /// rounds are implemented here, so `out` receives the real BLAKE3
    /// extendable-output stream, prefix-consistent with
    /// [`ChainKdf::derive`] by construction of the underlying dependency.
    #[inline]
    fn derive_xof(&self, input: &[u8], context: &str, out: &mut [u8]) -> Digest {
        let mut hasher = blake3::Hasher::new_derive_key(context);
        hasher.update(input);
        hasher.finalize_xof().fill(out);
        Digest(*hasher.finalize().as_bytes())
    }
    /// Delegates entirely to the external `blake3` crate's keyed
    /// (`new_derive_key`) mode — no BLAKE3 rounds are implemented here.
    /// Allocation on this path is whatever `blake3` itself performs; nothing
    /// is allocated in this crate.
    #[inline]
    fn derive(&self, input: &[u8], context: &str) -> Digest {
        let mut hasher = blake3::Hasher::new_derive_key(context);
        hasher.update(input);
        Digest(*hasher.finalize().as_bytes())
    }
}

impl ChainKdf for Fnv256 {
    /// **UNSUPPORTED(xof)**: FNV-1a is a plain iterated hash with no
    /// extendable-output mode. Pinned behavior: `out` is left **untouched**
    /// (zero bytes written — never a fake stream) and the plain
    /// [`ChainKdf::derive`] digest is returned, so an unsupported standing
    /// is detectable (`derive_xof(...) == derive(...)`) instead of silently
    /// wrong. Non-cryptographic regardless: tests/demos only.
    #[inline]
    fn derive_xof(&self, input: &[u8], context: &str, _out: &mut [u8]) -> Digest {
        self.derive(input, context)
    }

    /// [`Fnv256`] uses the inherited `context || input` fold; like everything
    /// FNV-based here it is **non-cryptographic** (no collision resistance,
    /// no keyed security) and is for tests/demos only.
    #[inline]
    fn derive(&self, input: &[u8], context: &str) -> Digest {
        Self::derive_default(input, context)
    }
}

#[cfg(test)]
mod kdf_tests {
    use super::*;

    /// Context domain separation is real: same input, two different contexts
    /// produce different digests on both hashers.
    #[test]
    fn kdf_different_contexts_differ_on_same_input() {
        for ctx in ["a", "affidavit receipt chain v1", "x".repeat(64).as_str()] {
            let b1 = Blake3Hasher.derive(b"payload", ctx);
            let b2 = Blake3Hasher.derive(b"payload", "unrelated other context");
            assert_ne!(b1, b2, "blake3 derive must separate contexts");

            let f1 = Fnv256.derive(b"payload", ctx);
            let f2 = Fnv256.derive(b"payload", "unrelated other context");
            assert_ne!(f1, f2, "fnv derive_default must separate contexts");
        }
    }

    /// Domain separation is not cosmetic: `derive(input, ctx)` must differ
    /// from the plain chain-hash of `input` alone.
    #[test]
    fn kdf_derive_differs_from_plain_chain_hash() {
        let plain = {
            let mut s = Blake3Hasher::init();
            Blake3Hasher::absorb(&mut s, b"payload");
            Blake3Hasher::finish(s)
        };
        let derived = Blake3Hasher.derive(b"payload", "ctx");
        assert_ne!(plain, derived, "blake3 keyed mode must domain-separate");

        let fnv_plain = {
            let mut s = Fnv256::init();
            Fnv256::absorb(&mut s, b"payload");
            Fnv256::finish(s)
        };
        assert_ne!(
            fnv_plain,
            Fnv256.derive(b"payload", "ctx"),
            "fnv context-prefix fold must differ from the bare fold"
        );
    }

    /// Determinism: identical (input, context) pairs reproduce byte-identical
    /// digests on both hashers.
    #[test]
    fn kdf_deterministic_same_args_identical_digests() {
        for _ in 0..2 {
            assert_eq!(
                Blake3Hasher.derive(b"determinism", "ctx-a"),
                Blake3Hasher.derive(b"determinism", "ctx-a")
            );
            assert_eq!(
                Fnv256.derive(b"determinism", "ctx-a"),
                Fnv256.derive(b"determinism", "ctx-a")
            );
        }
    }

    /// Positive control (vacuity sweep): emptying the context string changes
    /// the digest. If this ever fails, the context is not being absorbed and
    /// the tests above are vacuous.
    #[test]
    fn kdf_vacuity_empty_context_changes_digest() {
        assert_ne!(
            Blake3Hasher.derive(b"payload", ""),
            Blake3Hasher.derive(b"payload", "nonempty")
        );
        assert_ne!(
            Fnv256.derive(b"payload", ""),
            Fnv256.derive(b"payload", "nonempty")
        );
        // And the degenerate case: empty context is NOT the plain hash.
        let plain = {
            let mut s = Blake3Hasher::init();
            Blake3Hasher::absorb(&mut s, b"payload");
            Blake3Hasher::finish(s)
        };
        assert_ne!(Blake3Hasher.derive(b"payload", ""), plain);
    }

    /// Court 1 — prefix property: the first 32 bytes of a 64-byte XOF fill
    /// are byte-identical to the 32-byte `derive` digest. A re-hash
    /// "XOF" (hashing the stream again) drifts from this.
    #[test]
    fn xof_prefix_property_matches_derive_digest() {
        let input = b"xof prefix court";
        let ctx = "affidavit xof v1";
        let expected = Blake3Hasher.derive(input, ctx);

        let mut out = [0u8; 64];
        let head = Blake3Hasher.derive_xof(input, ctx, &mut out);
        assert_eq!(head, expected, "returned digest must equal derive");
        assert_eq!(&out[..32], &expected.0[..], "XOF[0..32] must equal derive");
        assert_ne!(
            &out[32..],
            &expected.0[..],
            "XOF[32..64] must be new stream bytes, not a repeat of the digest"
        );
    }

    /// Court 2 — same position, same bytes: filling two different window
    /// sizes yields identical bytes in the shared prefix, and the returned
    /// digest is window-independent.
    #[test]
    fn xof_same_position_same_fill_bytes() {
        let input = b"position court";
        let ctx = "affidavit xof v1";

        let mut short = [0u8; 16];
        let mut long = [0u8; 96];
        let h1 = Blake3Hasher.derive_xof(input, ctx, &mut short);
        let h2 = Blake3Hasher.derive_xof(input, ctx, &mut long);
        assert_eq!(h1, h2);
        assert_eq!(short[..], long[..16], "stream prefix must be stable");
    }

    /// Court 3 — determinism: two calls with identical arguments fill
    /// byte-identical streams.
    #[test]
    fn xof_deterministic_two_calls() {
        let mut a = [0u8; 64];
        let mut b = [0u8; 64];
        Blake3Hasher.derive_xof(b"det", "affidavit xof v1", &mut a);
        Blake3Hasher.derive_xof(b"det", "affidavit xof v1", &mut b);
        assert_eq!(a, b);
    }

    /// Court 4 — vacuity sweep: the empty context must change the XOF
    /// stream; if it did not, the context would not be reaching the keyed
    /// mode and every domain-separation claim above would be vacuous.
    #[test]
    fn xof_vacuity_empty_context_changes_stream() {
        let mut with = [0u8; 64];
        let mut without = [0u8; 64];
        Blake3Hasher.derive_xof(b"payload", "affidavit xof v1", &mut with);
        Blake3Hasher.derive_xof(b"payload", "", &mut without);
        assert_ne!(with, without);
    }

    /// Typed UNSUPPORTED(xof) standing, pinned: Fnv256 writes nothing to
    /// `out` and returns the plain `derive` digest, so the unsupported
    /// standing is detectable rather than silently wrong.
    #[test]
    fn xof_fnv_unsupported_pins_untouched_out_and_plain_digest() {
        let input = b"fnv";
        let ctx = "affidavit xof v1";
        let plain = Fnv256.derive(input, ctx);

        let mut out = [7u8; 32];
        let got = Fnv256.derive_xof(input, ctx, &mut out);
        assert_eq!(got, plain, "must return the plain derive digest");
        assert!(
            out.iter().all(|&b| b == 7),
            "out must be left untouched by the UNSUPPORTED(xof) impl"
        );
    }
}
