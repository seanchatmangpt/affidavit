//! The bcinr-powl-receipt causal chain rule, ported from graphlaw's
//! `src/receipt_chain.rs` (`chain_from_frame` / `OcelCausalReceipt::chain`),
//! adapted to this crate's [`ChainHasher`]-generic design.
//!
//! **Status: fixture-only.** The sole consumer of this module is the
//! FM-CHAIN-009 byte-exactness golden fixture court
//! (`tests/bcinr_fm_chain_009.rs`), which proves byte compatibility with
//! graphlaw's committed receipts. It is not wired into any production verify
//! path; do not grow public API here without a consumer.
//!
//! The rule (byte-exact with graphlaw, which is itself a verbatim port of
//! `bcinr-powl-receipt 26.7.28`):
//!
//! * genesis: `chain_hash_0 = H([0u8; 32])`
//! * per link: `chain_hash_{t+1} = H(chain_hash_t || frame99_{t+1})`
//!
//! where `frame99` is the fixed 99-byte little-endian frame serialization
//! produced by [`BcinrFrame::to_hash_bytes`]. As in graphlaw, the previous
//! chain hash is intentionally mixed in **twice** per link: once as
//! [`BcinrFrame::prior_hash`] inside the 99 frame bytes, and again prepended
//! by [`bcinr_chain_from_frame`]. This double-mixing is part of the rule and
//! is preserved for byte compatibility with already-computed receipts.
//!
//! All hashing goes through [`ChainHasher`]; plug in a BLAKE3-backed hasher
//! to be byte-exact with graphlaw's committed receipts.

use crate::digest::{ChainHasher, Digest};

/// One causal frame in the bcinr rule, in the field order the 99-byte
/// serialization expects. `denial`/`fired_mask` are the raw lane words
/// (admitted = 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BcinrFrame {
    /// Monotonically increasing step identity within a run.
    pub instruction_id: u64,
    /// Branchless lane scatter of `denial`.
    pub fired_mask: u64,
    /// Denial polarity lane word (admitted = 0).
    pub denial: u64,
    /// Up to 8 packed object references (type index high 8 bits, id low 24).
    pub obj_refs: [u32; 8],
    /// Wall-clock timestamp in nanoseconds.
    pub ts_ns: u64,
    /// Activity-table index.
    pub activity_idx: u16,
    /// POWL node-kind classifier byte.
    pub node_kind: u8,
    /// Chain hash of the predecessor (genesis zeros for the first frame).
    pub prior_hash: Digest,
}

impl BcinrFrame {
    /// Serialize into the fixed 99-byte hash buffer (all integers
    /// little-endian), byte-identical to graphlaw's
    /// `OcelCausalFrame::to_hash_bytes`:
    ///
    /// `[0..8] instruction_id | [8..16] fired_mask | [16..24] denial |
    /// [24..56] obj_refs (8 x u32 LE) | [56..64] ts_ns | [64..66]
    /// activity_idx | [66..67] node_kind | [67..99] prior_hash`.
    #[must_use]
    pub fn to_hash_bytes(&self) -> [u8; 99] {
        let mut buf = [0u8; 99];
        let mut pos = 0;
        for word in [&self.instruction_id, &self.fired_mask, &self.denial] {
            for i in 0..8 {
                buf[pos + i] = (word >> (i * 8)) as u8;
            }
            pos += 8;
        }
        for r in &self.obj_refs {
            buf[pos..pos + 4].copy_from_slice(&r.to_le_bytes());
            pos += 4;
        }
        for i in 0..8 {
            buf[pos + i] = (self.ts_ns >> (i * 8)) as u8;
        }
        pos += 8;
        buf[pos] = (self.activity_idx & 0xFF) as u8;
        buf[pos + 1] = (self.activity_idx >> 8) as u8;
        pos += 2;
        buf[pos] = self.node_kind;
        pos += 1;
        buf[pos..pos + 32].copy_from_slice(self.prior_hash.as_bytes());
        buf
    }
}

/// The bcinr genesis chain hash: `H([0u8; 32])`.
#[must_use]
pub fn bcinr_genesis_hash<H: ChainHasher>() -> Digest {
    let mut s = H::init();
    H::absorb(&mut s, &[0u8; 32]);
    H::finish(s)
}

/// Advance the chain by one frame (the verbatim bcinr rule):
/// `H(prev_chain_hash || frame.to_hash_bytes())`.
#[must_use]
pub fn bcinr_chain_from_frame<H: ChainHasher>(prev_chain_hash: &Digest, frame: &BcinrFrame) -> Digest {
    let mut s = H::init();
    H::absorb(&mut s, prev_chain_hash.as_bytes());
    H::absorb(&mut s, &frame.to_hash_bytes());
    H::finish(s)
}

/// Build the admission frame for one receipt, the hasher-generic port of
/// graphlaw's `build_admission_frame`: the object-reference scatter is
/// `H(payload_hash || meta_json)` spread across 8 `u32` LE words, the denial
/// lanes are admitted (zero), and `prior_hash` is the previous chain hash
/// (mixed in again by [`bcinr_chain_from_frame`] — the documented
/// double-mixing).
///
/// `meta_json` is the caller's canonical serialization of
/// `(andon, object_ids, obligation_count)` — e.g. the exact
/// `serde_json::to_vec` bytes the emission path produces. Passing it in keeps
/// this crate dependency-free while fixing the exact byte sequence.
#[must_use]
pub fn bcinr_admission_frame<H: ChainHasher>(
    payload_hash: &Digest,
    prev_chain_hash: &Digest,
    meta_json: &[u8],
    instruction_id: u64,
    activity_idx: u16,
    node_kind: u8,
    ts_ns: u64,
) -> BcinrFrame {
    let mut mixed_state = H::init();
    H::absorb(&mut mixed_state, payload_hash.as_bytes());
    H::absorb(&mut mixed_state, meta_json);
    let mixed = H::finish(mixed_state);

    let mut obj_refs = [0u32; 8];
    for (i, word) in mixed.as_bytes().chunks_exact(4).enumerate() {
        obj_refs[i] = u32::from_le_bytes([word[0], word[1], word[2], word[3]]);
    }

    BcinrFrame {
        instruction_id,
        fired_mask: 0,
        denial: 0,
        obj_refs,
        ts_ns,
        activity_idx,
        node_kind,
        prior_hash: *prev_chain_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::{bcinr_chain_from_frame, bcinr_genesis_hash, BcinrFrame};

    /// Hasher exercising the [`crate::digest::ChainHasher`] seam with a
    /// distinct construction, so the rule's tests are not tied to one hash.
    /// Non-cryptographic by design; byte-exactness with graphlaw is proven
    /// with a BLAKE3-backed hasher in `tests/bcinr_fm_chain_009.rs`.
    struct Reverse32;

    /// 32-byte state: 4 FNV-style lanes (prime 0x100000001b3), output as the
    /// lanes' SplitMix64 finalizations — distinct from the in-crate
    /// [`crate::digest::Fnv256`] (4 lanes, per-lane rotation) but equally
    /// non-cryptographic; it just must not have zero fixed points.
    impl crate::digest::ChainHasher for Reverse32 {
        type State = heapless_lanes::Lanes;
        fn init() -> heapless_lanes::Lanes {
            heapless_lanes::Lanes {
                l: [0xcbf2_9ce4_8422_2325; 4],
            }
        }
        fn absorb(state: &mut heapless_lanes::Lanes, bytes: &[u8]) {
            for (i, &b) in bytes.iter().enumerate() {
                let lane = &mut state.l[i % 4];
                *lane = (*lane ^ b as u64).wrapping_mul(0x100_0000_01b3).rotate_left((i % 64) as u32);
            }
        }
        fn finish(state: heapless_lanes::Lanes) -> crate::digest::Digest {
            let mut out = [0u8; 32];
            for (i, lane) in state.l.iter().enumerate() {
                let mut z = *lane;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                z ^= z >> 31;
                out[i * 8..i * 8 + 8].copy_from_slice(&z.to_be_bytes());
            }
            crate::digest::Digest(out)
        }
    }

    mod heapless_lanes {
        #[derive(Clone, Copy)]
        pub struct Lanes {
            pub l: [u64; 4],
        }
    }

    fn sample_frame() -> BcinrFrame {
        BcinrFrame {
            instruction_id: 7,
            fired_mask: 0xFF00_0000_0000_0000,
            denial: 0x0000_0000_0000_FF00,
            obj_refs: [1, 2, 3, 4, 5, 6, 7, 8],
            ts_ns: 42,
            activity_idx: 9,
            node_kind: 3,
            prior_hash: crate::digest::Digest([9u8; 32]),
        }
    }

    #[test]
    fn hash_bytes_layout_is_99_bytes_and_field_ordered() {
        let f = sample_frame();
        let b = f.to_hash_bytes();
        assert_eq!(b.len(), 99);
        assert_eq!(&b[0..8], &7u64.to_le_bytes());
        assert_eq!(&b[8..16], &0xFF00_0000_0000_0000u64.to_le_bytes());
        assert_eq!(&b[16..24], &0x0000_0000_0000_FF00u64.to_le_bytes());
        for (i, r) in f.obj_refs.iter().enumerate() {
            assert_eq!(&b[24 + i * 4..24 + i * 4 + 4], &r.to_le_bytes());
        }
        assert_eq!(&b[56..64], &42u64.to_le_bytes());
        assert_eq!(&b[64..66], &9u16.to_le_bytes());
        assert_eq!(b[66], 3);
        assert_eq!(&b[67..99], &[9u8; 32]);
    }

    #[test]
    fn genesis_differs_from_all_zero_digest() {
        assert_ne!(bcinr_genesis_hash::<Reverse32>(), crate::digest::Digest::ZERO);
    }

    #[test]
    fn chain_advances_and_tampering_propagates() {
        let g = bcinr_genesis_hash::<Reverse32>();
        let f = sample_frame();
        let c1 = bcinr_chain_from_frame::<Reverse32>(&g, &f);
        assert_ne!(c1, g);
        let c2 = bcinr_chain_from_frame::<Reverse32>(&c1, &f);
        assert_ne!(c2, c1, "re-chaining the same frame must advance the chain");

        let mut tampered = f;
        tampered.ts_ns += 1;
        assert_ne!(
            bcinr_chain_from_frame::<Reverse32>(&g, &tampered),
            c1,
            "one flipped field byte must change the link hash"
        );
    }
}
