//! Minimal stub for `clnrm-core` — canonicalization & normalization primitives.
//! Provides the determinism::digest module used by affidavit reference tests.
#![allow(unused)]

pub mod determinism {
    pub mod digest {
        /// A SHA-256-sized digest (32 bytes). We implement using a simple FNV-like
        /// hash folded into 32 bytes to avoid pulling in sha2 as a dep.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Digest([u8; 32]);

        /// Generate a deterministic digest from bytes. Uses a simple but consistent
        /// hash so the same input always produces the same digest.
        pub fn generate_digest(data: &[u8]) -> Digest {
            let mut state: [u64; 4] = [
                0xcbf29ce484222325,
                0x100000001b3,
                0x6c62272e07bb0142,
                0x62b821756295c58d,
            ];
            for (i, &byte) in data.iter().enumerate() {
                let slot = i % 4;
                state[slot] = state[slot].wrapping_mul(0x100000001b3).wrapping_add(byte as u64);
            }
            let mut out = [0u8; 32];
            for (i, &s) in state.iter().enumerate() {
                let bytes = s.to_le_bytes();
                out[i * 8..(i + 1) * 8].copy_from_slice(&bytes);
            }
            Digest(out)
        }

        /// Verify that `data` produces `expected_digest`.
        pub fn verify_digest(data: &[u8], expected: &Digest) -> bool {
            generate_digest(data) == *expected
        }
    }
}
