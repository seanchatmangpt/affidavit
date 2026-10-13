//! Direct NIST FIPS 180-4 known-answer self-court for the zero-dependency
//! in-crate [`Sha256`] `ChainHasher` (src/sha256.rs).
//!
//! The seam tests (`tests/sha256_chain_hasher.rs`) pin cross-repo behavior via
//! the `sha2` dev-dependency; this court pins the primitive itself against the
//! standard's published vectors, plus padding-boundary cases, chaining
//! semantics, and a wrong-IV sabotage twin proving the court is non-vacuous
//! (the "abc" vector genuinely pins the FIPS 180-4 §5.3.3 initial hash values).
//!
//! Ground truth: digests below are the FIPS 180-4 published vectors verbatim;
//! the two padding-boundary digests (56/112 bytes) were derived with an
//! independent conformant implementation (Python `hashlib`) and are stable
//! literals, not computed at test time.

use affidavit_core::{ChainHasher, Sha256};

/// Hash `data` through the real `ChainHasher` three-call shape.
fn kat(data: &[u8]) -> String {
    let mut state = Sha256::init();
    Sha256::absorb(&mut state, data);
    let digest = Sha256::finish(state);
    digest.0.iter().map(|b| format!("{b:02x}")).collect()
}

/// FIPS 180-4 official one-block vector: SHA-256("abc").
#[test]
fn kat_fips_abc() {
    assert_eq!(
        kat(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

/// FIPS 180-4 official two-block vector (multi-block absorption path).
#[test]
fn kat_fips_two_block() {
    assert_eq!(
        kat(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

/// Empty input: the zero-length message vector.
#[test]
fn kat_empty() {
    assert_eq!(
        kat(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

/// 8-byte all-'a' message (short-message class).
#[test]
fn kat_eight_as() {
    assert_eq!(kat(b"aaaaaaaa"), "1f3ce40415a2081fa3eee75fc39fff8e56c22270d1a978a7249b592dcebd20b4");
}

/// One million 'a' bytes, absorbed by real iteration in 64 KiB chunks —
/// no hardcoded shortcut for the long-message class.
#[test]
fn kat_million_as() {
    const CHUNK: [u8; 64 * 1024] = [b'a'; 64 * 1024];
    const REMAINDER: usize = 1_000_000 % CHUNK.len();
    let mut state = Sha256::init();
    for _ in 0..(1_000_000 / CHUNK.len()) {
        Sha256::absorb(&mut state, &CHUNK);
    }
    Sha256::absorb(&mut state, &[b'a'; REMAINDER]);
    let digest = Sha256::finish(state);
    let hex: String = digest.0.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        hex,
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

/// Length-448-bit boundary: 56 bytes of 'a' — the message plus 0x80 fills the
/// block exactly to the length-reservation edge (padding spills to a second
/// block). Padding-boundary class.
#[test]
fn kat_boundary_448_bits() {
    assert_eq!(
        kat(&[b'a'; 56]),
        "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a"
    );
}

/// Length-896-bit boundary: 112 bytes of 'a' — two full blocks, padding in a
/// third. Padding-boundary class.
#[test]
fn kat_boundary_896_bits() {
    assert_eq!(
        kat(&[b'a'; 112]),
        "f54353008a2553262ecdc4a34749563ba0950e8b0fc8652780b0a614b99683c1"
    );
}

/// Chaining semantics (documented `ChainHasher` property):
/// H_{n+1} = compress(H_n, block) — block-at-a-time absorption must equal the
/// one-shot hash of the concatenation, across chunking strategies.
#[test]
fn chaining_blockwise_equals_one_shot() {
    // Deterministic 640-byte body = exactly 10 full blocks.
    let body: Vec<u8> = (0..640u32).map(|i| (i % 251) as u8).collect();

    // One-shot.
    let mut whole = Sha256::init();
    Sha256::absorb(&mut whole, &body);
    let one_shot = Sha256::finish(whole);

    // Strict block-at-a-time (each absorb is exactly one 64-byte block).
    for chunk in 1..=3usize {
        let mut streamed = Sha256::init();
        for piece in body.chunks(chunk * 64) {
            Sha256::absorb(&mut streamed, piece);
        }
        let streamed_digest = Sha256::finish(streamed);
        assert_eq!(streamed_digest, one_shot, "chunk stride {chunk}x64 diverged");
    }
}

// ---------------------------------------------------------------------------
// Sabotage twin: prove the KATs above are non-vacuous.
//
// `Sha256State` fields are private, so this twin carries an independent,
// self-contained copy of the FIPS 180-4 compression with a deliberately
// corrupted initial value (IV[0] flipped). If the "abc" KAT could not
// distinguish a wrong IV, it would carry no bits; this test witnesses that
// it does.
// ---------------------------------------------------------------------------

const K_SABOTAGE: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// One compression over a single 64-byte block from an explicit IV.
fn compress_from_iv(iv: [u32; 8], block: &[u8; 64]) -> [u32; 8] {
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
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = iv;
    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let t1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K_SABOTAGE[i])
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
    [
        iv[0].wrapping_add(a),
        iv[1].wrapping_add(b),
        iv[2].wrapping_add(c),
        iv[3].wrapping_add(d),
        iv[4].wrapping_add(e),
        iv[5].wrapping_add(f),
        iv[6].wrapping_add(g),
        iv[7].wrapping_add(h),
    ]
}

/// SHA-256 of a one-block message from an explicit IV (correct padding for
/// `len == 3` bytes: 0x80, zeros, 24-bit length).
fn sha256_from_iv(iv: [u32; 8], msg: &[u8; 3]) -> [u8; 32] {
    let mut block = [0u8; 64];
    block[..3].copy_from_slice(msg);
    block[3] = 0x80;
    block[63] = 24; // 3 bytes * 8 bits
    let h = compress_from_iv(iv, &block);
    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Sabotage twin: the FIPS IV must matter. With IV[0] corrupted the "abc"
/// digest must differ from both the standard vector and the real hasher's
/// output — otherwise the KAT above would vacuously pass any implementation.
#[test]
fn sabotage_wrong_iv_is_refused_by_kat() {
    const FIPS_IV: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut sabotaged = FIPS_IV;
    sabotaged[0] ^= 1;

    // The in-twin reference with the CORRECT IV reproduces the standard "abc"
    // vector — proving the twin's own machinery is faithful.
    let mut twin_correct = [0u8; 32];
    twin_correct.copy_from_slice(&sha256_from_iv(FIPS_IV, b"abc"));
    assert_eq!(
        twin_correct,
        Sha256::finish({
            let mut s = Sha256::init();
            Sha256::absorb(&mut s, b"abc");
            s
        })
        .0,
        "sabotage twin with correct IV disagrees with the real hasher"
    );

    let bad = sha256_from_iv(sabotaged, b"abc");
    let expected: [u8; 32] = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    assert_ne!(bad, expected, "wrong IV matched the FIPS vector — KAT is vacuous");
    assert_ne!(
        bad,
        Sha256::finish({
            let mut s = Sha256::init();
            Sha256::absorb(&mut s, b"abc");
            s
        })
        .0,
        "wrong IV matched the real hasher — KAT is vacuous"
    );
}
