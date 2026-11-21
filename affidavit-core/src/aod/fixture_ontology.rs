//! Static ggen-style [`CompiledOntology`] for production Live Gate admits
//! (AFFI-SPEC-26.11.19 PR topology §4.1).
//!
//! Closed-domain symbols live in the binary; on-disk `.aod` artifacts carry
//! only [`FIXTURE_ONTOLOGY_HASH`] and positional buffers. When `ggen` cannot
//! run in an environment, this module is the checked-in projection — regenerate
//! via `scripts/regen_aod_fixture_ontology.sh` (or the unit test below) after
//! changing [`FIXTURE_SCHEMA_ID`] / [`FIXTURE_STRIDES`].
//!
//! Ontology source snippet: `ontology/aod-fixture.ttl`.

use crate::digest::{Blake3Hasher, ChainHasher};

use super::live::CompiledOntology;

/// Domain tag for ontology-hash derivation (compile-time only; never on disk).
pub const DOMAIN_AOD_ONTOLOGY: &[u8] = b"affidavit-core/aod/v1\0ontology";

/// Schema identity for the default single-buffer u64 column ontology.
pub const FIXTURE_SCHEMA_ID: &[u8] = b"aod.fixture.v1.u64_col";

/// Default per-row byte strides (one positional buffer, 8-byte cells).
pub const FIXTURE_STRIDES: &[u64] = &[8];

/// Checked-in BLAKE3 pin of [`derive_ontology_hash`]`(FIXTURE_SCHEMA_ID, FIXTURE_STRIDES)`.
///
/// Hex: `9a96ff6a3aa16735678d70df6e8302105dd48e5ac71bb1dabbf6adb3b811773b`
pub const FIXTURE_ONTOLOGY_HASH: [u8; 32] = [
    0x9A, 0x96, 0xFF, 0x6A, 0x3A, 0xA1, 0x67, 0x35, 0x67, 0x8D, 0x70, 0xDF, 0x6E, 0x83, 0x02, 0x10,
    0x5D, 0xD4, 0x8E, 0x5A, 0xC7, 0x1B, 0xB1, 0xDA, 0xBB, 0xF6, 0xAD, 0xB3, 0xB8, 0x11, 0x77, 0x3B,
];

/// Lowercase hex of [`FIXTURE_ONTOLOGY_HASH`] for CLI / seal callers.
pub const FIXTURE_ONTOLOGY_HASH_HEX: &str =
    "9a96ff6a3aa16735678d70df6e8302105dd48e5ac71bb1dabbf6adb3b811773b";

/// Derive `ontology_hash` = BLAKE3 over the domain-separated canon:
///
/// `DOMAIN_AOD_ONTOLOGY || schema_id || 0x00 || le_u32(stride_count) || Σ le_u64(stride)`.
pub fn derive_ontology_hash(schema_id: &[u8], strides: &[u64]) -> [u8; 32] {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, DOMAIN_AOD_ONTOLOGY);
    Blake3Hasher::absorb(&mut st, schema_id);
    Blake3Hasher::absorb(&mut st, &[0u8]);
    let n = strides.len() as u32;
    Blake3Hasher::absorb(&mut st, &n.to_le_bytes());
    for s in strides {
        Blake3Hasher::absorb(&mut st, &s.to_le_bytes());
    }
    Blake3Hasher::finish(st).0
}

/// Production Live Gate binding: fixture hash + default strides.
#[inline]
pub const fn compiled_fixture() -> CompiledOntology<'static> {
    CompiledOntology {
        ontology_hash: &FIXTURE_ONTOLOGY_HASH,
        strides: FIXTURE_STRIDES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_hash_pin_matches_derivation() {
        let got = derive_ontology_hash(FIXTURE_SCHEMA_ID, FIXTURE_STRIDES);
        assert_eq!(
            got, FIXTURE_ONTOLOGY_HASH,
            "FIXTURE_ONTOLOGY_HASH pin drifted — run scripts/regen_aod_fixture_ontology.sh"
        );
    }

    #[test]
    fn fixture_hex_matches_bytes() {
        let mut hex = [0u8; 64];
        for (i, b) in FIXTURE_ONTOLOGY_HASH.iter().enumerate() {
            let s = format!("{:02x}", b);
            hex[i * 2] = s.as_bytes()[0];
            hex[i * 2 + 1] = s.as_bytes()[1];
        }
        assert_eq!(
            core::str::from_utf8(&hex).unwrap(),
            FIXTURE_ONTOLOGY_HASH_HEX
        );
    }

    #[test]
    fn compiled_fixture_strides_are_default() {
        let c = compiled_fixture();
        assert_eq!(c.ontology_hash, &FIXTURE_ONTOLOGY_HASH);
        assert_eq!(c.strides, FIXTURE_STRIDES);
        assert_eq!(c.expected_byte_length(0, 2), Some(16));
    }

    #[test]
    fn schema_drift_changes_hash() {
        let other = derive_ontology_hash(b"aod.fixture.v1.other", FIXTURE_STRIDES);
        assert_ne!(other, FIXTURE_ONTOLOGY_HASH);
    }
}
