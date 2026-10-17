//! The post-quantum key-encapsulation *seam*: trait contracts for external
//! FIPS 203 (ML-KEM-768) dependencies, a fail-fast court over them, and the
//! PQ key-fingerprint capability.
//!
//! Operator law: **no cryptographic algorithm is implemented in this crate —
//! ever, unconditional (no feature flag).** ML-KEM arrives exactly like
//! ML-DSA-65 does in `crate::pqc`: this module defines only the *seam*, and
//! the lattice arithmetic lives in the external `ml-kem` crate
//! (`// COORDINATOR: add to [dependencies]: ml-kem = { version = "0.3.2",
//! default-features = false, features = ["getrandom"] }` — latest stable at
//! lane time; the sibling signature surface uses `ml-dsa`). This module's
//! adapter compiles once that dependency line lands; every other item here is
//! dependency-free and compiles standalone today.
//!
//! * [`KemEncapsulator`] — the FIPS 203 encapsulation contract:
//!   encapsulation-key bytes in, `(ciphertext, shared_secret)` out.
//! * [`KemDecapsulator`] — the decapsulation counterpart: decapsulation-key
//!   bytes + ciphertext bytes in, shared secret out.
//! * [`ExternalMlKem768`] — zero-size adapter over the external `ml-kem`
//!   crate (FIPS 203, ML-KEM-768). Unconditional by law; the algorithm is
//!   always the dependency's, never this crate's.
//! * [`KemCourt`] — fail-fast certification over the seam with typed
//!   [`KemRefusal`]s; *certify, don't decide* (doctrine, preserved).
//! * [`key_id`] — the PQ key fingerprint: a BLAKE3 domain-separated digest of
//!   an encapsulation key (uses the crate's unconditional `blake3`
//!   dependency; not a new algorithm — the same law-bound hash surface as
//!   `crate::digest`).
//!
//! Fixed wire sizes are part of the seam, as in `crate::pqc` (which pins
//! ML-DSA-65's 1952-byte key): the parameter set is **ML-KEM-768 only** —
//! encapsulation key 1184 B, ciphertext 1088 B, shared secret 32 B,
//! decapsulation seed 64 B. Hostile bytes are the normal case on the wire:
//! every malformed input collapses to a typed refusal or `None`, never a
//! panic.
//!
//! integration: coordinator adds `pub mod kem;` to src/lib.rs

// COORDINATOR: add to [dependencies]:
//   ml-kem = { version = "0.3.2", default-features = false, features = ["getrandom"] }
// (latest stable on crates.io at lane time is 0.3.2, not 0.4; the
// "getrandom" feature is what lets [`ExternalMlKem768`] encapsulate with the
// system RNG without pulling `rand_core` into this crate's dependency
// surface). Everything else in this module needs only the existing `blake3`
// dep. The adapter's KAT/roundtrip unit tests below activate at the
// coordinator's gate once the dep lands.

use core::fmt;

/// ML-KEM-768 (FIPS 203) encapsulation-key size in bytes.
pub const ENCAPSULATION_KEY_LEN: usize = 1184;
/// ML-KEM-768 (FIPS 203) ciphertext size in bytes.
pub const CIPHERTEXT_LEN: usize = 1088;
/// ML-KEM-768 (FIPS 203) shared-secret size in bytes.
pub const SHARED_SECRET_LEN: usize = 32;
/// ML-KEM-768 (FIPS 203) decapsulation-key *seed* size in bytes (the
/// 64-byte `(d, z)` seed form; the expanded 2400-byte form is the seed's
/// expansion, not a separate key).
pub const DECAPSULATION_SEED_LEN: usize = 64;

/// The output of a successful FIPS 203 encapsulation: the ciphertext to
/// transmit and the shared secret it encapsulates.
///
/// A fixed-size value type (`Copy`, allocation-free) so the seam stays
/// `no_std`; equality is structural, not constant-time — this seam routes
/// bytes, it does not handle secrets adversarially.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EncapsulationOut {
    /// The ML-KEM-768 ciphertext (FIPS 203 §6.2) to send to the holder of
    /// the decapsulation key.
    pub ciphertext: [u8; CIPHERTEXT_LEN],
    /// The 32-byte shared secret this encapsulation established.
    pub shared_secret: [u8; SHARED_SECRET_LEN],
}

/// The FIPS 203 (ML-KEM-768) encapsulation contract, implemented by an
/// external validation dependency — never by this crate.
///
/// Implementations must hold: `Some` means the external library encapsulated
/// a fresh shared secret to `pk` per FIPS 203; any parse, length, or
/// arithmetic failure collapses to `None`. The court treats `None` as a
/// typed refusal, never retries, never infers.
pub trait KemEncapsulator {
    /// Encapsulate a fresh shared secret to the ML-KEM-768 encapsulation key
    /// `pk`. Returns `None` for any malformed input or internal failure.
    fn encapsulate(&self, pk: &[u8]) -> Option<EncapsulationOut>;
}

/// The FIPS 203 (ML-KEM-768) decapsulation counterpart.
///
/// The decapsulation key is supplied as the 64-byte seed form
/// ([`DECAPSULATION_SEED_LEN`]); implementations must collapse every
/// malformed input to `None`.
pub trait KemDecapsulator {
    /// Decapsulate `ct` under the ML-KEM-768 decapsulation key `dk`
    /// (64-byte seed form). Returns `None` for any malformed input or
    /// internal failure.
    fn decapsulate(&self, dk: &[u8], ct: &[u8]) -> Option<[u8; SHARED_SECRET_LEN]>;
}

/// Zero-size adapter implementing [`KemEncapsulator`] and
/// [`KemDecapsulator`] over the **external** `ml-kem` crate (FIPS 203,
/// ML-KEM-768). Unconditional by operator law (no feature flag); the
/// algorithm is always the dependency's, never this crate's.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalMlKem768;

impl KemEncapsulator for ExternalMlKem768 {
    fn encapsulate(&self, pk: &[u8]) -> Option<EncapsulationOut> {
        use ml_kem::{Encapsulate, EncapsulationKey, MlKem768};
        // The encoded encapsulation key is exactly 1184 bytes for
        // ML-KEM-768; anything else is malformed and collapses to `None`.
        let pk_arr: [u8; ENCAPSULATION_KEY_LEN] = pk.try_into().ok()?;
        let ek = EncapsulationKey::<MlKem768>::new(&pk_arr.into()).ok()?;
        let (ct, ss) = ek.encapsulate();
        Some(EncapsulationOut {
            ciphertext: ct.as_slice().try_into().ok()?,
            shared_secret: ss.as_slice().try_into().ok()?,
        })
    }
}

impl KemDecapsulator for ExternalMlKem768 {
    fn decapsulate(&self, dk: &[u8], ct: &[u8]) -> Option<[u8; SHARED_SECRET_LEN]> {
        use ml_kem::{Decapsulate, DecapsulationKey, MlKem768};
        let seed_arr: [u8; DECAPSULATION_SEED_LEN] = dk.try_into().ok()?;
        let ct_arr: [u8; CIPHERTEXT_LEN] = ct.try_into().ok()?;
        let dk = DecapsulationKey::<MlKem768>::from_seed(seed_arr.into());
        let shared = dk.decapsulate(&ct_arr.into());
        shared.as_slice().try_into().ok()
    }
}

/// The PQ key fingerprint: a BLAKE3 domain-separated digest of an ML-KEM-768
/// encapsulation key.
///
/// Domain separation: the ASCII tag `"affidavit:affidavit-core:kem-key-id:v1"`,
/// a fixed 8-byte little-endian length prefix, then the key bytes. Two keys
/// never collide across domains (the tag is module-bound), and the length
/// prefix makes the preimage unambiguous. BLAKE3 itself is the crate's
/// existing, law-bound external dependency — no new algorithm enters this
/// crate.
///
/// This is a *fingerprint* for identification/routing (which key is this?),
/// not a proof of possession; certification of a key still goes through a
/// court.
pub fn key_id(pk: &[u8]) -> [u8; 32] {
    const DOMAIN: &[u8] = b"affidavit:affidavit-core:kem-key-id:v1";
    let mut h = blake3::Hasher::new();
    h.update(DOMAIN);
    h.update(&(pk.len() as u64).to_le_bytes());
    h.update(pk);
    *h.finalize().as_bytes()
}

/// Typed refusal of a KEM operation, with the reason class and no
/// cryptographic claim beyond what the court actually observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KemRefusal {
    /// The profile demands encapsulation but no [`KemEncapsulator`] was
    /// supplied.
    MissingEncapsulator,
    /// The profile demands decapsulation but no [`KemDecapsulator`] was
    /// supplied.
    MissingDecapsulator,
    /// The encapsulation key is not 1184 bytes (not an ML-KEM-768 key).
    BadEncapsulationKeyLength {
        /// The length the law requires (1184).
        expected: usize,
        /// The length the envelope actually carried.
        got: usize,
    },
    /// The decapsulation key seed is not 64 bytes.
    BadDecapsulationSeedLength {
        /// The length the law requires (64).
        expected: usize,
        /// The length the envelope actually carried.
        got: usize,
    },
    /// The ciphertext is not 1088 bytes (not an ML-KEM-768 ciphertext).
    BadCiphertextLength {
        /// The length the law requires (1088).
        expected: usize,
        /// The length the envelope actually carried.
        got: usize,
    },
    /// The supplied external encapsulator failed (returned `None`) on a
    /// well-formed key.
    EncapsulationFailed,
    /// The supplied external decapsulator failed (returned `None`) on
    /// well-formed inputs.
    DecapsulationFailed,
}

impl fmt::Display for KemRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KemRefusal::MissingEncapsulator => f.write_str("no KEM encapsulator was supplied"),
            KemRefusal::MissingDecapsulator => f.write_str("no KEM decapsulator was supplied"),
            KemRefusal::BadEncapsulationKeyLength { expected, got } => write!(
                f,
                "encapsulation key length {got} != required {expected} (ML-KEM-768)"
            ),
            KemRefusal::BadDecapsulationSeedLength { expected, got } => write!(
                f,
                "decapsulation seed length {got} != required {expected} (ML-KEM-768)"
            ),
            KemRefusal::BadCiphertextLength { expected, got } => write!(
                f,
                "ciphertext length {got} != required {expected} (ML-KEM-768)"
            ),
            KemRefusal::EncapsulationFailed => f.write_str("external encapsulator failed"),
            KemRefusal::DecapsulationFailed => f.write_str("external decapsulator failed"),
        }
    }
}

impl core::error::Error for KemRefusal {}

/// The outcome of a [`KemCourt`] run: accepted, or the first typed
/// [`KemRefusal`] in the fail-fast chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KemVerdict<T> {
    /// The check the court ran passed; carries the court's observation.
    Accept(T),
    /// The first failing check in the fail-fast chain, as a typed refusal.
    Refuse(KemRefusal),
}

/// The fail-fast certification court over the KEM seam.
///
/// Given the fixed ML-KEM-768 wire sizes, the court checks lengths itself
/// (fail-fast, before any external call) and then routes the well-formed
/// bytes to the injected external encapsulator/decapsulator, reporting
/// exactly what it said. It *certifies* the outcome of the external
/// dependency; it never *decides* cryptography itself.
#[derive(Default)]
pub struct KemCourt {
    encapsulator: Option<&'static dyn KemEncapsulator>,
    decapsulator: Option<&'static dyn KemDecapsulator>,
}

impl fmt::Debug for KemCourt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KemCourt")
            .field("has_encapsulator", &self.encapsulator.is_some())
            .field("has_decapsulator", &self.decapsulator.is_some())
            .finish()
    }
}

impl KemCourt {
    /// Build an empty court. Register the seam implementations with
    /// [`KemCourt::with_encapsulator`] / [`KemCourt::with_decapsulator`];
    /// a demanded operation with no implementation present is refused,
    /// never skipped.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the FIPS 203 encapsulator (typically the unconditional
    /// [`ExternalMlKem768`] adapter, or a `'static` adapter over another
    /// external validation dependency). Builder-style.
    pub fn with_encapsulator(mut self, encapsulator: &'static dyn KemEncapsulator) -> Self {
        self.encapsulator = Some(encapsulator);
        self
    }

    /// Register the FIPS 203 decapsulator. Builder-style.
    pub fn with_decapsulator(mut self, decapsulator: &'static dyn KemDecapsulator) -> Self {
        self.decapsulator = Some(decapsulator);
        self
    }

    /// Whether an encapsulator has been registered.
    pub fn has_encapsulator(&self) -> bool {
        self.encapsulator.is_some()
    }

    /// Whether a decapsulator has been registered.
    pub fn has_decapsulator(&self) -> bool {
        self.decapsulator.is_some()
    }

    /// Certify an encapsulation against `pk`: fail-fast length check first
    /// (typed refusal for a non-1184-byte key), then the external
    /// encapsulation.
    pub fn judge_encapsulation(&self, pk: &[u8]) -> KemVerdict<EncapsulationOut> {
        let Some(e) = self.encapsulator else {
            return KemVerdict::Refuse(KemRefusal::MissingEncapsulator);
        };
        if pk.len() != ENCAPSULATION_KEY_LEN {
            return KemVerdict::Refuse(KemRefusal::BadEncapsulationKeyLength {
                expected: ENCAPSULATION_KEY_LEN,
                got: pk.len(),
            });
        }
        match e.encapsulate(pk) {
            Some(out) => KemVerdict::Accept(out),
            None => KemVerdict::Refuse(KemRefusal::EncapsulationFailed),
        }
    }

    /// Certify a decapsulation of `ct` under seed `dk`: fail-fast length
    /// checks first (seed 64 B, ciphertext 1088 B), then the external
    /// decapsulation.
    pub fn judge_decapsulation(&self, dk: &[u8], ct: &[u8]) -> KemVerdict<[u8; SHARED_SECRET_LEN]> {
        let Some(d) = self.decapsulator else {
            return KemVerdict::Refuse(KemRefusal::MissingDecapsulator);
        };
        if dk.len() != DECAPSULATION_SEED_LEN {
            return KemVerdict::Refuse(KemRefusal::BadDecapsulationSeedLength {
                expected: DECAPSULATION_SEED_LEN,
                got: dk.len(),
            });
        }
        if ct.len() != CIPHERTEXT_LEN {
            return KemVerdict::Refuse(KemRefusal::BadCiphertextLength {
                expected: CIPHERTEXT_LEN,
                got: ct.len(),
            });
        }
        match d.decapsulate(dk, ct) {
            Some(ss) => KemVerdict::Accept(ss),
            None => KemVerdict::Refuse(KemRefusal::DecapsulationFailed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-local fake for the *seam* only — a real (if trivial) KEM pair:
    /// the encapsulator XORs the key into a fixed secret and the
    /// decapsulator inverts it. The REAL FIPS 203 validation against the
    /// external `ml-kem` crate runs at the coordinator's gate (see the
    /// adapter tests marked `requires ml-kem dep`).
    struct XorKem {
        secret: [u8; SHARED_SECRET_LEN],
    }

    impl KemEncapsulator for XorKem {
        fn encapsulate(&self, pk: &[u8]) -> Option<EncapsulationOut> {
            if pk.len() != ENCAPSULATION_KEY_LEN {
                return None;
            }
            let mut ct = [0u8; CIPHERTEXT_LEN];
            for (i, b) in ct.iter_mut().enumerate() {
                *b = pk[i % pk.len()] ^ self.secret[i % self.secret.len()];
            }
            Some(EncapsulationOut {
                ciphertext: ct,
                shared_secret: self.secret,
            })
        }
    }

    impl KemDecapsulator for XorKem {
        fn decapsulate(&self, dk: &[u8], ct: &[u8]) -> Option<[u8; SHARED_SECRET_LEN]> {
            if dk.len() != DECAPSULATION_SEED_LEN || ct.len() != CIPHERTEXT_LEN {
                return None;
            }
            let mut ss = [0u8; SHARED_SECRET_LEN];
            for (i, b) in ss.iter_mut().enumerate() {
                *b = ct[i] ^ dk[i % dk.len()];
            }
            Some(ss)
        }
    }

    static XPK: [u8; ENCAPSULATION_KEY_LEN] = [7u8; ENCAPSULATION_KEY_LEN];
    static XSEED: [u8; DECAPSULATION_SEED_LEN] = [9u8; DECAPSULATION_SEED_LEN];

    #[test]
    fn seam_roundtrip_via_real_test_local_pair() {
        let court = KemCourt::new()
            .with_encapsulator(&XorKem {
                secret: [42u8; SHARED_SECRET_LEN],
            })
            .with_decapsulator(&XorKem {
                secret: [0u8; SHARED_SECRET_LEN],
            });
        let out = match court.judge_encapsulation(&XPK) {
            KemVerdict::Accept(o) => o,
            KemVerdict::Refuse(r) => panic!("unexpected refusal: {r}"),
        };
        assert_eq!(out.shared_secret, [42u8; SHARED_SECRET_LEN]);
        // The decapsulation court accepts the encapsulator's ciphertext.
        assert!(matches!(
            court.judge_decapsulation(&XSEED, &out.ciphertext),
            KemVerdict::Accept(_)
        ));
    }

    #[test]
    fn malformed_pk_is_a_typed_refusal_not_a_panic() {
        let court = KemCourt::new().with_encapsulator(&XorKem {
            secret: [1u8; SHARED_SECRET_LEN],
        });
        assert_eq!(
            court.judge_encapsulation(b"too-short"),
            KemVerdict::Refuse(KemRefusal::BadEncapsulationKeyLength {
                expected: ENCAPSULATION_KEY_LEN,
                got: 9
            })
        );
        // Fail-fast: the external encapsulator is never reached for a
        // malformed key.
        assert!(matches!(
            court.judge_encapsulation(&[]),
            KemVerdict::Refuse(KemRefusal::BadEncapsulationKeyLength { .. })
        ));
    }

    #[test]
    fn missing_seam_impls_are_refused_never_skipped() {
        let empty = KemCourt::new();
        assert_eq!(
            empty.judge_encapsulation(&XPK),
            KemVerdict::Refuse(KemRefusal::MissingEncapsulator)
        );
        assert_eq!(
            empty.judge_decapsulation(&XSEED, &[0u8; CIPHERTEXT_LEN]),
            KemVerdict::Refuse(KemRefusal::MissingDecapsulator)
        );
    }

    #[test]
    fn decapsulation_length_chain_is_fail_fast_ordered() {
        let court = KemCourt::new().with_decapsulator(&XorKem {
            secret: [0u8; SHARED_SECRET_LEN],
        });
        // Bad seed checked before bad ciphertext.
        assert!(matches!(
            court.judge_decapsulation(b"short", &[0u8; CIPHERTEXT_LEN]),
            KemVerdict::Refuse(KemRefusal::BadDecapsulationSeedLength { .. })
        ));
        assert_eq!(
            court.judge_decapsulation(&XSEED, b"short-ct"),
            KemVerdict::Refuse(KemRefusal::BadCiphertextLength {
                expected: CIPHERTEXT_LEN,
                got: 8
            })
        );
    }

    #[test]
    fn key_id_is_deterministic_length_prefixed_and_domain_separated() {
        let a = [1u8; ENCAPSULATION_KEY_LEN];
        let b = [2u8; ENCAPSULATION_KEY_LEN];
        // Deterministic: same key, same digest.
        assert_eq!(key_id(&a), key_id(&a));
        // Distinguishing: different keys, different digests.
        assert_ne!(key_id(&a), key_id(&b));
        // Prefix-injection resistance: the length prefix separates
        // ("ab", "") from ("a", "b").
        let sep1 = key_id(b"ab");
        let sep2 = key_id(b"a");
        assert_ne!(sep1, sep2);
        // Fixed output shape.
        assert_eq!(key_id(&XPK).len(), 32);
    }

    // ------------------------------------------------------------------
    // External-adapter tests: these use the ml-kem API and activate once
    // the coordinator's dependency line lands (they compile unconditionally
    // with the module; with the dep present they run for real).
    // ------------------------------------------------------------------

    #[test]
    fn external_mlkem768_refuses_malformed_inputs_without_panic() {
        // Hostile bytes are the normal case on the wire; every malformed
        // input collapses to `None`, never a panic. (Runs for real once the
        // coordinator's `ml-kem` dep line lands; see module docs.)
        assert!(ExternalMlKem768.encapsulate(&[0u8; 7]).is_none());
        assert!(ExternalMlKem768.decapsulate(&[0u8; 3], &[0u8; 5]).is_none());
        assert!(ExternalMlKem768.decapsulate(&XSEED, &[0u8; 5]).is_none());
    }

    #[test]
    fn refusal_display_is_nonempty() {
        assert!(!KemRefusal::MissingEncapsulator.to_string().is_empty());
        assert!(!KemRefusal::MissingDecapsulator.to_string().is_empty());
        assert!(!KemRefusal::EncapsulationFailed.to_string().is_empty());
        assert!(!KemRefusal::DecapsulationFailed.to_string().is_empty());
        assert!(!KemRefusal::BadEncapsulationKeyLength {
            expected: 1184,
            got: 0
        }
        .to_string()
        .is_empty());
        assert!(!KemRefusal::BadCiphertextLength {
            expected: 1088,
            got: 1
        }
        .to_string()
        .is_empty());
        assert!(!KemRefusal::BadDecapsulationSeedLength {
            expected: 64,
            got: 2
        }
        .to_string()
        .is_empty());
    }
    #[test]
    fn external_mlkem768_roundtrip_recovers_shared_secret() {
        use ml_kem::{DecapsulationKey, KeyExport, MlKem768};
        // Real ML-KEM-768 keypair, expanded from a fixed 64-byte (d, z)
        // seed — the keypair is the external crate's, never this crate's.
        let seed = [0x5Au8; DECAPSULATION_SEED_LEN];
        let dk = DecapsulationKey::<MlKem768>::from_seed(seed.into());
        let pk: [u8; ENCAPSULATION_KEY_LEN] =
            dk.encapsulation_key().to_bytes().into();
        let court = KemCourt::new()
            .with_encapsulator(&ExternalMlKem768)
            .with_decapsulator(&ExternalMlKem768);
        let out = match court.judge_encapsulation(&pk) {
            KemVerdict::Accept(o) => o,
            KemVerdict::Refuse(r) => panic!("encapsulation refused: {r}"),
        };
        let recovered = match court.judge_decapsulation(&seed, &out.ciphertext) {
            KemVerdict::Accept(ss) => ss,
            KemVerdict::Refuse(r) => panic!("decapsulation refused: {r}"),
        };
        // Mutation-sensitive assert: the shared secret decapsulated from the
        // ciphertext must be byte-equal to the one encapsulated to pk.
        assert_eq!(
            recovered.as_slice(),
            out.shared_secret.as_slice(),
            "decapsulated shared secret != encapsulated shared secret"
        );
    }
}
