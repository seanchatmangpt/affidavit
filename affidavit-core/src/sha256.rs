//! A small, dependency-free, `no_std` FIPS 180-4 SHA-256 implementation.
//!
//! Why hand-rolled when `sha2` exists: this crate's core design invariant is
//! **zero `[dependencies]`** (see `Cargo.toml` — the trust-critical verifier
//! must have no supply-chain surface). `sha2` remains a *dev-dependency* and
//! `tests/sha256_chain_hasher.rs` pins this implementation byte-for-byte
//! against it, so correctness does not rest on hand-verification.
//!
//! Correctness anchors:
//! - FIPS 180-4 test vectors (`"abc"`, the two-block `"abcdbcde..."` vector);
//! - differential testing against the `sha2` crate (dev-dep) across length
//!   classes that straddle block boundaries (55/56/63/64/119/120 bytes, ...);
//! - `ChainHasher` tamper courts over real event chains.

use crate::digest::{ChainHasher, Digest};

/// Round constants: fractional parts of cube roots of the first 64 primes
/// (FIPS 180-4, section 4.2.2).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Initial hash values: fractional parts of square roots of the first 8 primes
/// (FIPS 180-4, section 5.3.3).
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// Streaming SHA-256 state (`ChainHasher::State` for [`Sha256`]).
#[derive(Clone, Copy, Debug)]
pub struct Sha256State {
    h: [u32; 8],
    /// Bytes absorbed so far (for padding).
    len: u64,
    /// Partial block buffer.
    buf: [u8; 64],
    /// Bytes currently in `buf`.
    buf_len: usize,
}

impl Sha256State {
    fn compress(&mut self, block: &[u8]) {
        let mut w = [0u32; 64];
        for (i, chunk) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        let temps = [a, b, c, d, e, f, g, h];
        for (hi, t) in self.h.iter_mut().zip(temps) {
            *hi = hi.wrapping_add(t);
        }
    }

    fn finalize_into(mut self, out: &mut [u8; 32]) {
        let bit_len = self.len.wrapping_mul(8);
        // Append 0x80 then zeros then the 8-byte big-endian bit length.
        self.absorb(&[0x80]);
        while self.buf_len != 56 {
            self.absorb(&[0]);
        }
        self.absorb(&bit_len.to_be_bytes());
        for (i, word) in self.h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
    }
}

impl Default for Sha256State {
    fn default() -> Self {
        Sha256State {
            h: H0,
            len: 0,
            buf: [0u8; 64],
            buf_len: 0,
        }
    }
}

impl Sha256State {
    fn absorb(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.buf[self.buf_len] = b;
            self.buf_len += 1;
            self.len = self.len.wrapping_add(1);
            if self.buf_len == 64 {
                let block = self.buf;
                self.buf_len = 0;
                self.compress(&block);
            }
        }
    }
}

/// A `no_std`, zero-dependency SHA-256 [`ChainHasher`] for adversarial
/// settings — the cryptographic replacement for the non-cryptographic
/// [`Fnv256`](crate::digest::Fnv256) reference hasher.
///
/// Differential-tested against the `sha2` crate (dev-dependency); see
/// `tests/sha256_chain_hasher.rs`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sha256;

impl ChainHasher for Sha256 {
    type State = Sha256State;

    fn init() -> Sha256State {
        Sha256State::default()
    }

    fn absorb(state: &mut Sha256State, bytes: &[u8]) {
        Sha256State::absorb(state, bytes);
    }

    fn finish(state: Sha256State) -> Digest {
        let mut out = [0u8; 32];
        state.finalize_into(&mut out);
        Digest(out)
    }
}

/// Convenience: one-shot SHA-256 over a concatenation of byte slices.
#[cfg(feature = "alloc")]
pub fn sha256_concat(parts: &[&[u8]]) -> [u8; 32] {
    let mut s = Sha256State::default();
    for p in parts {
        s.absorb(p);
    }
    let mut out = [0u8; 32];
    s.finalize_into(&mut out);
    out
}
