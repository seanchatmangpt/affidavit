//! ML-DSA-65, SLH-DSA-SHA2-128s, and hybrid ES256+ML-DSA-65 signing providers (FIPS 204 / FIPS 205) for the affidavit cryptographic trust plane.
// Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//
// This module REPLACES the retired blake3-mock `1000x_post_quantum_sealing.rs`:
// every post-quantum claim here is real cryptography —
//   - ML-DSA-65 via `ml-dsa` (FIPS 204): seed keygen, explicit-rnd
//     deterministic `Sign_internal`, fixed-length encoded signatures;
//   - SLH-DSA-SHA2-128s via `slh-dsa` (FIPS 205): three-seed keygen,
//     deterministic signing, fixed-length encoded signatures;
//   - hybrid ES256+ML-DSA-65 composed at the ENCODED-BYTES level (both halves
//     sign the same message; verification requires BOTH halves).
// All parameters below are rendered from the ctp: graph (pqc.rq), never
// literals: lengths, seed sizes, and spec references are graph facts.
//
// House law: refusal-as-value (typed [`PqcError`], never panics); every
// variant witnessed by tests; teeth-before-tamper (round-trips are proven
// before each tamper falsifier); no unwrap/expect outside tests.

use rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::crypto_trust_canonical::digest;
use crate::crypto_trust_es256::{verify_es256, Es256SigningKey};

/// Parameter-set authority for ML-DSA-65 (ctp:param-FIPS204-MLDSA65 ctp:specRef).
pub const ML_DSA_65_SPEC_REF: &str = "FIPS 204";
/// ML-DSA-65 encoded public key length in bytes
/// (ctp:alg-ML-DSA-65 ctp:publicKeyLength = FIPS 204).
pub const ML_DSA_65_PUBLIC_KEY_LEN: usize = 1952;
/// ML-DSA-65 encoded signature length in bytes
/// (ctp:alg-ML-DSA-65 ctp:signatureLength = FIPS 204).
pub const ML_DSA_65_SIGNATURE_LEN: usize = 3309;
/// ML-DSA-65 seed length in bytes (ctp:alg-ML-DSA-65 ctp:seedLength).
pub const ML_DSA_65_SEED_LEN: usize = 32;
/// Parameter-set authority for SLH-DSA-SHA2-128s
/// (ctp:param-FIPS205-SLHDSA128S ctp:specRef).
pub const SLH_DSA_128S_SPEC_REF: &str = "FIPS 205";
/// SLH-DSA-SHA2-128s seed length in bytes
/// (ctp:alg-SLH-DSA-SHA2-128s ctp:seedLength = sk_seed || sk_prf || pk_seed).
pub const SLH_DSA_128S_SEED_LEN: usize = 48;
/// Parameter-set authority for the hybrid composition
/// (ctp:param-HYBRID-ES256-MLDSA65 ctp:specRef, the LAMPS composite draft).
pub const HYBRID_SPEC_REF: &str = "draft-ietf-lamps-pq-composite-sig";
/// Domain tag as bound by pqc.rq (ctp:policy-v1 ctp:domainTag). Cross-rule
/// consistency with [`crate::crypto_trust_canonical::DOMAIN_TAG`] is tested.
pub const PQC_DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// One SLH-DSA-SHA2-128s seed slice: the graph seed length split three ways
/// (FIPS 205 n = SLH_DSA_128S_SEED_LEN / 3 = 16 for the 128s parameter set).
const SLH_DSA_128S_N: usize = SLH_DSA_128S_SEED_LEN / 3;

/// Typed refusals of the PQ signing providers. Values, never panics.
#[derive(Debug, thiserror::Error)]
pub enum PqcError {
    #[error("ml-dsa: {0}")]
    MlDsa(String),
    #[error("slh-dsa: {0}")]
    SlhDsa(String),
    #[error("es256 half of hybrid: {0}")]
    Es256Half(#[from] crate::crypto_trust_es256::Es256Error),
    #[error("malformed public key")]
    MalformedPublicKey,
    #[error("malformed signature")]
    MalformedSignature,
}

// ── ML-DSA-65 (FIPS 204) ────────────────────────────────────────────────────

/// ML-DSA-65 key pair held as the 32-byte seed (the preferred FIPS 204
/// serialization; ctp:alg-ML-DSA-65 ctp:seedLength) plus the derived raw
/// encoded public key (exactly [`ML_DSA_65_PUBLIC_KEY_LEN`] bytes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MlDsa65KeyPair {
    pub seed: [u8; ML_DSA_65_SEED_LEN],
    pub public: Vec<u8>,
}

/// Fresh ML-DSA-65 key pair from the operating-system RNG.
pub fn ml_dsa65_generate() -> Result<MlDsa65KeyPair, PqcError> {
    let mut seed = [0u8; ML_DSA_65_SEED_LEN];
    rand_core::OsRng.fill_bytes(&mut seed);
    Ok(ml_dsa65_from_seed(&seed))
}

/// Deterministically derives the ML-DSA-65 key pair from a 32-byte seed
/// (FIPS 204 ML-DSA.KeyGen_internal; same seed => same public key).
#[must_use]
pub fn ml_dsa65_from_seed(seed: &[u8; ML_DSA_65_SEED_LEN]) -> MlDsa65KeyPair {
    let sk_seed: ml_dsa::Seed = (*seed).into();
    let esk = ml_dsa::ExpandedSigningKey::<ml_dsa::MlDsa65>::from_seed(&sk_seed);
    let vk = esk.verifying_key();
    MlDsa65KeyPair {
        seed: *seed,
        public: vk.encode().as_slice().to_vec(),
    }
}

/// Signs `msg` with ML-DSA-65 under an EXPLICIT 32-byte randomizer `rnd`
/// (FIPS 204 ML-DSA.Sign_internal: the caller owns rnd, so signing is
/// reproducible — same seed, message and rnd give byte-identical signatures).
/// Returns the raw fixed-length encoding (exactly
/// [`ML_DSA_65_SIGNATURE_LEN`] bytes).
pub fn ml_dsa65_sign(
    seed: &[u8; ML_DSA_65_SEED_LEN],
    msg: &[u8],
    rnd: &[u8; 32],
) -> Result<Vec<u8>, PqcError> {
    let sk_seed: ml_dsa::Seed = (*seed).into();
    let esk = ml_dsa::ExpandedSigningKey::<ml_dsa::MlDsa65>::from_seed(&sk_seed);
    let rnd: ml_dsa::B32 = (*rnd).into();
    let sig = esk.sign_internal(&[msg], &rnd);
    Ok(sig.encode().as_slice().to_vec())
}

/// Verifies a raw encoded ML-DSA-65 signature over `msg` under the raw
/// encoded public key. Refusals: [`PqcError::MalformedPublicKey`] when the
/// key is not exactly [`ML_DSA_65_PUBLIC_KEY_LEN`] bytes;
/// [`PqcError::MalformedSignature`] when the signature is not exactly
/// [`ML_DSA_65_SIGNATURE_LEN`] bytes or fails structural decoding.
pub fn ml_dsa65_verify(public: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, PqcError> {
    let pk = ml_dsa::EncodedVerifyingKey::<ml_dsa::MlDsa65>::try_from(public)
        .map_err(|_| PqcError::MalformedPublicKey)?;
    let vk = ml_dsa::VerifyingKey::<ml_dsa::MlDsa65>::decode(&pk);
    let parsed = ml_dsa::Signature::<ml_dsa::MlDsa65>::try_from(sig)
        .map_err(|_| PqcError::MalformedSignature)?;
    Ok(vk.verify_internal(msg, &parsed))
}

// ── SLH-DSA-SHA2-128s (FIPS 205) ────────────────────────────────────────────

/// SLH-DSA-SHA2-128s key material: the 48-byte seed triplet
/// `sk_seed || sk_prf || pk_seed` (FIPS 205 slh_keygen_internal inputs,
/// ctp:alg-SLH-DSA-SHA2-128s ctp:seedLength) plus the derived raw encoded
/// public key (`2n` = 32 bytes).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlhDsa128sKeyPair {
    pub seeds: [u8; SLH_DSA_128S_SEED_LEN],
    pub public: Vec<u8>,
}

/// Splits the 48-byte seed triplet into its three n-byte FIPS 205 slices.
#[must_use]
fn slh_dsa128s_split(seeds: &[u8; SLH_DSA_128S_SEED_LEN]) -> (&[u8], &[u8], &[u8]) {
    let n = SLH_DSA_128S_N;
    (&seeds[..n], &seeds[n..2 * n], &seeds[2 * n..])
}

/// Fresh SLH-DSA-SHA2-128s key material from the operating-system RNG.
pub fn slh_dsa128s_generate() -> Result<SlhDsa128sKeyPair, PqcError> {
    let mut seeds = [0u8; SLH_DSA_128S_SEED_LEN];
    rand_core::OsRng.fill_bytes(&mut seeds);
    Ok(slh_dsa128s_from_seed(&seeds))
}

/// Deterministically derives the SLH-DSA-SHA2-128s key pair from the 48-byte
/// seed triplet (FIPS 205 slh_keygen_internal; same seeds => same public key).
#[must_use]
pub fn slh_dsa128s_from_seed(seeds: &[u8; SLH_DSA_128S_SEED_LEN]) -> SlhDsa128sKeyPair {
    use slh_dsa::signature::Keypair;

    let (sk_seed, sk_prf, pk_seed) = slh_dsa128s_split(seeds);
    let sk =
        slh_dsa::SigningKey::<slh_dsa::Sha2_128s>::slh_keygen_internal(sk_seed, sk_prf, pk_seed);
    SlhDsa128sKeyPair {
        seeds: *seeds,
        public: sk.verifying_key().to_bytes().to_vec(),
    }
}

/// Signs `msg` with SLH-DSA-SHA2-128s. Deterministic by construction: the
/// crate's default sign path derives the FIPS 205 randomizer from pk_seed,
/// so the same seeds and message always give byte-identical signatures.
pub fn slh_dsa128s_sign(
    seeds: &[u8; SLH_DSA_128S_SEED_LEN],
    msg: &[u8],
) -> Result<Vec<u8>, PqcError> {
    use slh_dsa::signature::Signer;

    let (sk_seed, sk_prf, pk_seed) = slh_dsa128s_split(seeds);
    let sk =
        slh_dsa::SigningKey::<slh_dsa::Sha2_128s>::slh_keygen_internal(sk_seed, sk_prf, pk_seed);
    let sig = sk
        .try_sign(msg)
        .map_err(|e| PqcError::SlhDsa(format!("sign failed: {e}")))?;
    Ok(sig.to_vec())
}

/// Verifies a raw encoded SLH-DSA-SHA2-128s signature over `msg` under the
/// raw encoded public key. Refusals: [`PqcError::MalformedPublicKey`] when
/// the key is not exactly `2n` bytes; [`PqcError::MalformedSignature`] when
/// the signature length is not the FIPS 205 encoded size.
pub fn slh_dsa128s_verify(public: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, PqcError> {
    let vk = slh_dsa::VerifyingKey::<slh_dsa::Sha2_128s>::try_from(public)
        .map_err(|_| PqcError::MalformedPublicKey)?;
    let parsed = slh_dsa::Signature::<slh_dsa::Sha2_128s>::try_from(sig)
        .map_err(|_| PqcError::MalformedSignature)?;
    use slh_dsa::signature::Verifier;
    Ok(vk.verify(msg, &parsed).is_ok())
}

// ── Hybrid ES256 + ML-DSA-65 (encoded-bytes composition) ────────────────────

/// Composite signing secret for ctp:alg-HYBRID-ES256-MLDSA65
/// (ctp:composedOf alg-ES256 + alg-ML-DSA-65). Both halves sign the SAME
/// message; the composition lives at the encoded-bytes level, never by
/// bridging trait objects across the crates' signature-trait versions.
pub struct HybridSecret {
    pub es256: Es256SigningKey,
    pub mldsa65_seed: [u8; ML_DSA_65_SEED_LEN],
}

/// Both encoded signature halves of a hybrid signature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridSignature {
    /// ES256 (ECDSA P-256/SHA-256) signature over the message, DER-encoded.
    pub es256_der: Vec<u8>,
    /// ML-DSA-65 raw encoded signature over the SAME message.
    pub mldsa65: Vec<u8>,
}

/// Deterministic ML-DSA randomizer for the hybrid: the 32-byte BLAKE3 digest
/// of the domain-separated pre-image of the message under the trust-plane
/// domain tag. The graph marks the hybrid `ctp:deterministic true`; binding
/// the randomizer to the domain keeps signatures reproducible AND
/// domain-separated (a signature crafted in another domain diverges).
#[must_use]
fn hybrid_mldsa_rnd(msg: &[u8]) -> [u8; 32] {
    digest(
        PQC_DOMAIN_TAG,
        &[b"ctp.pqc.hybrid-es256-mldsa65.rnd" as &[u8], msg],
    )
}

/// Signs `msg` with BOTH hybrid halves over the same bytes: ES256 (DER, RFC
/// 6979 deterministic) and ML-DSA-65 (raw encoded, deterministic under the
/// domain-bound randomizer).
pub fn hybrid_sign(secret: &HybridSecret, msg: &[u8]) -> Result<HybridSignature, PqcError> {
    let es256_der = secret.es256.sign(msg);
    let mldsa65 = ml_dsa65_sign(&secret.mldsa65_seed, msg, &hybrid_mldsa_rnd(msg))?;
    Ok(HybridSignature { es256_der, mldsa65 })
}

/// Verifies BOTH hybrid halves over the same message. BOTH must verify: the
/// hybrid is broken if either half fails, so a forged or stripped half is
/// refused even when the other half is genuine.
pub fn hybrid_verify(
    es256_pk: &[u8],
    mldsa65_pk: &[u8],
    msg: &[u8],
    sig: &HybridSignature,
) -> Result<bool, PqcError> {
    let es_ok = verify_es256(es256_pk, msg, &sig.es256_der)?;
    let pq_ok = ml_dsa65_verify(mldsa65_pk, msg, &sig.mldsa65)?;
    Ok(es_ok && pq_ok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DOMAIN_TAG;

    const MSG: &[u8] = b"affidavit pqc lane: ML-DSA-65 / SLH-DSA-SHA2-128s / hybrid";
    const SEED_A: [u8; 32] = [0xA5u8; 32];
    const RND_A: [u8; 32] = [0x5Au8; 32];
    const SEED_B: [u8; 32] = [0x3Cu8; 32];
    const SLH_SEEDS_A: [u8; 48] = [0x11u8; 48];
    const SLH_SEEDS_B: [u8; 48] = [0x22u8; 48];

    #[test]
    fn graph_consts_are_the_fips_values() {
        assert_eq!(ML_DSA_65_SPEC_REF, "FIPS 204");
        assert_eq!(ML_DSA_65_PUBLIC_KEY_LEN, 1952);
        assert_eq!(ML_DSA_65_SIGNATURE_LEN, 3309);
        assert_eq!(ML_DSA_65_SEED_LEN, 32);
        assert_eq!(SLH_DSA_128S_SPEC_REF, "FIPS 205");
        assert_eq!(SLH_DSA_128S_SEED_LEN, 48);
        assert_eq!(SLH_DSA_128S_N, 16);
        assert_eq!(HYBRID_SPEC_REF, "draft-ietf-lamps-pq-composite-sig");
        assert_eq!(PQC_DOMAIN_TAG, DOMAIN_TAG);
    }

    #[test]
    fn ml_dsa65_round_trip() {
        let kp = ml_dsa65_from_seed(&SEED_A);
        let sig = ml_dsa65_sign(&kp.seed, MSG, &RND_A).expect("ml-dsa sign");
        let ok = ml_dsa65_verify(&kp.public, MSG, &sig).expect("ml-dsa verify");
        assert!(ok);
    }

    #[test]
    fn ml_dsa65_generate_is_os_rng_and_round_trips() {
        let kp = ml_dsa65_generate().expect("ml-dsa keygen");
        assert_eq!(kp.seed.len(), ML_DSA_65_SEED_LEN);
        assert_eq!(kp.public.len(), ML_DSA_65_PUBLIC_KEY_LEN);
        let sig = ml_dsa65_sign(&kp.seed, MSG, &RND_A).expect("ml-dsa sign");
        assert!(ml_dsa65_verify(&kp.public, MSG, &sig).expect("ml-dsa verify"));
    }

    #[test]
    fn ml_dsa65_encoded_lengths_match_the_graph_consts() {
        let kp = ml_dsa65_from_seed(&SEED_A);
        assert_eq!(kp.public.len(), ML_DSA_65_PUBLIC_KEY_LEN);
        let sig = ml_dsa65_sign(&kp.seed, MSG, &RND_A).expect("ml-dsa sign");
        assert_eq!(
            sig.len(),
            ML_DSA_65_SIGNATURE_LEN,
            "ml-dsa 0.1.1 encoded signature must be the FIPS 204 length 3309"
        );
    }

    #[test]
    fn ml_dsa65_is_deterministic_in_seed_msg_and_rnd() {
        let sig1 = ml_dsa65_sign(&SEED_A, MSG, &RND_A).expect("sign 1");
        let sig2 = ml_dsa65_sign(&SEED_A, MSG, &RND_A).expect("sign 2");
        assert_eq!(sig1, sig2, "same seed+msg+rnd must give identical bytes");
        let kp1 = ml_dsa65_from_seed(&SEED_A);
        let kp2 = ml_dsa65_from_seed(&SEED_A);
        assert_eq!(kp1, kp2);
    }

    #[test]
    fn ml_dsa65_diverges_on_rnd_seed_and_message() {
        let base = ml_dsa65_sign(&SEED_A, MSG, &RND_A).expect("base sign");
        let other_rnd = ml_dsa65_sign(&SEED_A, MSG, &[0u8; 32]).expect("other rnd");
        let other_seed = ml_dsa65_sign(&SEED_B, MSG, &RND_A).expect("other seed");
        let other_msg = ml_dsa65_sign(&SEED_A, b"other", &RND_A).expect("other msg");
        assert_ne!(base, other_rnd, "explicit rnd must reach the signer");
        assert_ne!(base, other_seed);
        assert_ne!(base, other_msg);
    }

    #[test]
    fn ml_dsa65_tamper_msg_falsifier() {
        let kp = ml_dsa65_from_seed(&SEED_A);
        let sig = ml_dsa65_sign(&kp.seed, MSG, &RND_A).expect("sign");
        let mut tampered_msg = MSG.to_vec();
        tampered_msg[0] ^= 0x01;
        assert!(!ml_dsa65_verify(&kp.public, &tampered_msg, &sig).expect("verify"));
    }

    #[test]
    fn ml_dsa65_tamper_sig_falsifier() {
        let kp = ml_dsa65_from_seed(&SEED_A);
        let mut sig = ml_dsa65_sign(&kp.seed, MSG, &RND_A).expect("sign");
        sig[0] ^= 0x01;
        assert!(!ml_dsa65_verify(&kp.public, MSG, &sig).expect("verify"));
    }

    #[test]
    fn ml_dsa65_cross_key_refusal() {
        let kp_a = ml_dsa65_from_seed(&SEED_A);
        let kp_b = ml_dsa65_from_seed(&SEED_B);
        let sig = ml_dsa65_sign(&kp_a.seed, MSG, &RND_A).expect("sign under A");
        assert!(!ml_dsa65_verify(&kp_b.public, MSG, &sig).expect("verify under B"));
    }

    #[test]
    fn ml_dsa65_malformed_public_key_is_typed() {
        let kp = ml_dsa65_from_seed(&SEED_A);
        let sig = ml_dsa65_sign(&kp.seed, MSG, &RND_A).expect("sign");
        let short = [0u8; ML_DSA_65_PUBLIC_KEY_LEN - 1];
        let err = ml_dsa65_verify(&short, MSG, &sig).expect_err("short pk refused");
        assert!(matches!(err, PqcError::MalformedPublicKey), "got {err:?}");
    }

    #[test]
    fn ml_dsa65_malformed_signature_is_typed() {
        let kp = ml_dsa65_from_seed(&SEED_A);
        let short = [0u8; ML_DSA_65_SIGNATURE_LEN - 1];
        let err = ml_dsa65_verify(&kp.public, MSG, &short).expect_err("short sig refused");
        assert!(matches!(err, PqcError::MalformedSignature), "got {err:?}");
    }

    #[test]
    fn slh_dsa128s_round_trip() {
        let kp = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let sig = slh_dsa128s_sign(&kp.seeds, MSG).expect("slh-dsa sign");
        let ok = slh_dsa128s_verify(&kp.public, MSG, &sig).expect("slh-dsa verify");
        assert!(ok);
    }

    #[test]
    fn slh_dsa128s_generate_is_os_rng_and_round_trips() {
        let kp = slh_dsa128s_generate().expect("slh-dsa keygen");
        assert_eq!(kp.seeds.len(), SLH_DSA_128S_SEED_LEN);
        assert_eq!(kp.public.len(), 2 * SLH_DSA_128S_N);
        let sig = slh_dsa128s_sign(&kp.seeds, MSG).expect("slh-dsa sign");
        assert!(slh_dsa128s_verify(&kp.public, MSG, &sig).expect("slh-dsa verify"));
    }

    #[test]
    fn slh_dsa128s_encoded_lengths_match_fips_205() {
        let kp = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        assert_eq!(kp.seeds.len(), SLH_DSA_128S_SEED_LEN);
        assert_eq!(
            kp.public.len(),
            2 * SLH_DSA_128S_N,
            "SLH-DSA-SHA2-128s public key is pk_seed || pk_root = 2n = 32 bytes"
        );
        let sig = slh_dsa128s_sign(&kp.seeds, MSG).expect("slh-dsa sign");
        assert_eq!(
            sig.len(),
            7_856,
            "SLH-DSA-SHA2-128s encoded signature must be the FIPS 205 length 7856"
        );
    }

    #[test]
    fn slh_dsa128s_is_deterministic_in_seeds_and_message() {
        let sig1 = slh_dsa128s_sign(&SLH_SEEDS_A, MSG).expect("sign 1");
        let sig2 = slh_dsa128s_sign(&SLH_SEEDS_A, MSG).expect("sign 2");
        assert_eq!(sig1, sig2, "same seeds+msg must give identical bytes");
        let kp1 = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let kp2 = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        assert_eq!(kp1, kp2);
    }

    #[test]
    fn slh_dsa128s_tamper_msg_falsifier() {
        let kp = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let sig = slh_dsa128s_sign(&kp.seeds, MSG).expect("sign");
        let mut tampered_msg = MSG.to_vec();
        tampered_msg[1] ^= 0x80;
        assert!(!slh_dsa128s_verify(&kp.public, &tampered_msg, &sig).expect("verify"));
    }

    #[test]
    fn slh_dsa128s_tamper_sig_falsifier() {
        let kp = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let mut sig = slh_dsa128s_sign(&kp.seeds, MSG).expect("sign");
        sig[100] ^= 0x01;
        assert!(!slh_dsa128s_verify(&kp.public, MSG, &sig).expect("verify"));
    }

    #[test]
    fn slh_dsa128s_cross_key_refusal() {
        let kp_a = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let kp_b = slh_dsa128s_from_seed(&SLH_SEEDS_B);
        let sig = slh_dsa128s_sign(&kp_a.seeds, MSG).expect("sign under A");
        assert!(!slh_dsa128s_verify(&kp_b.public, MSG, &sig).expect("verify under B"));
    }

    #[test]
    fn slh_dsa128s_malformed_public_key_is_typed() {
        let kp = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let sig = slh_dsa128s_sign(&kp.seeds, MSG).expect("sign");
        let short = [0u8; 2 * SLH_DSA_128S_N - 1];
        let err = slh_dsa128s_verify(&short, MSG, &sig).expect_err("short pk refused");
        assert!(matches!(err, PqcError::MalformedPublicKey), "got {err:?}");
    }

    #[test]
    fn slh_dsa128s_malformed_signature_is_typed() {
        let kp = slh_dsa128s_from_seed(&SLH_SEEDS_A);
        let short = [0u8; 100];
        let err = slh_dsa128s_verify(&kp.public, MSG, &short).expect_err("short sig refused");
        assert!(matches!(err, PqcError::MalformedSignature), "got {err:?}");
    }

    fn hybrid_secret(seed: [u8; 32], mldsa65_seed: [u8; 32]) -> HybridSecret {
        HybridSecret {
            es256: Es256SigningKey::from_seed(&seed).expect("valid p256 scalar"),
            mldsa65_seed,
        }
    }

    fn hybrid_secret_a() -> HybridSecret {
        hybrid_secret([0x42u8; 32], SEED_A)
    }

    #[test]
    fn hybrid_round_trip() {
        let secret = hybrid_secret_a();
        let mldsa_kp = ml_dsa65_from_seed(&SEED_A);
        let es256_pk = secret.es256.public_key_sec1();
        let sig = hybrid_sign(&secret, MSG).expect("hybrid sign");
        let ok = hybrid_verify(&es256_pk, &mldsa_kp.public, MSG, &sig).expect("hybrid verify");
        assert!(ok);
    }

    #[test]
    fn hybrid_signature_is_deterministic_and_serde_round_trips() {
        let secret = hybrid_secret_a();
        let s1 = hybrid_sign(&secret, MSG).expect("sign 1");
        let s2 = hybrid_sign(&secret, MSG).expect("sign 2");
        assert_eq!(s1, s2, "hybrid is ctp:deterministic true");
        let json = serde_json::to_string(&s1).expect("serialize hybrid sig");
        let back: HybridSignature = serde_json::from_str(&json).expect("deserialize hybrid sig");
        assert_eq!(back, s1);
    }

    #[test]
    fn hybrid_tamper_es256_half_only_falsifier() {
        let secret = hybrid_secret_a();
        let mldsa_kp = ml_dsa65_from_seed(&SEED_A);
        let es256_pk = secret.es256.public_key_sec1();
        let mut sig = hybrid_sign(&secret, MSG).expect("hybrid sign");
        sig.es256_der[0] ^= 0x01;
        assert!(
            !hybrid_verify(&es256_pk, &mldsa_kp.public, MSG, &sig).unwrap_or(false),
            "hybrid must be broken when ONLY the es256 half is tampered \
             (Ok(false) or a typed refusal both count as broken)"
        );
    }

    #[test]
    fn hybrid_tamper_mldsa_half_only_falsifier() {
        let secret = hybrid_secret_a();
        let mldsa_kp = ml_dsa65_from_seed(&SEED_A);
        let es256_pk = secret.es256.public_key_sec1();
        let mut sig = hybrid_sign(&secret, MSG).expect("hybrid sign");
        let last = sig.mldsa65.len() - 1;
        sig.mldsa65[last] ^= 0x01;
        assert!(
            !hybrid_verify(&es256_pk, &mldsa_kp.public, MSG, &sig).unwrap_or(false),
            "hybrid must be broken when ONLY the mldsa65 half is tampered \
             (Ok(false) or a typed refusal both count as broken)"
        );
    }

    #[test]
    fn hybrid_wrong_msg_falsifier() {
        let secret = hybrid_secret_a();
        let mldsa_kp = ml_dsa65_from_seed(&SEED_A);
        let es256_pk = secret.es256.public_key_sec1();
        let sig = hybrid_sign(&secret, MSG).expect("hybrid sign");
        assert!(
            !hybrid_verify(&es256_pk, &mldsa_kp.public, b"other message", &sig).expect("verify"),
            "hybrid must refuse when the message differs"
        );
    }

    #[test]
    fn hybrid_cross_key_refusal() {
        let secret_a = hybrid_secret_a();
        let secret_b = HybridSecret {
            es256: Es256SigningKey::from_seed(&[0x99u8; 32]).expect("valid p256 scalar"),
            mldsa65_seed: SEED_B,
        };
        let es256_pk_b = secret_b.es256.public_key_sec1();
        let mldsa_kp_b = ml_dsa65_from_seed(&SEED_B);
        let sig = hybrid_sign(&secret_a, MSG).expect("hybrid sign under A");
        assert!(
            !hybrid_verify(&es256_pk_b, &mldsa_kp_b.public, MSG, &sig).expect("verify"),
            "both halves cross-keyed must fail together"
        );
    }

    #[test]
    fn hybrid_mldsa_rnd_is_domain_separated() {
        let rnd = hybrid_mldsa_rnd(MSG);
        assert_ne!(rnd, [0u8; 32]);
        let other_domain = digest(
            "another.domain",
            &[b"ctp.pqc.hybrid-es256-mldsa65.rnd" as &[u8], MSG],
        );
        assert_ne!(rnd, other_domain, "randomizer must be domain-bound");
        assert_ne!(hybrid_mldsa_rnd(b"other"), rnd);
    }

    #[test]
    fn error_variants_render_their_typed_messages() {
        let mldsa_err = PqcError::MlDsa("seed refused".to_string());
        assert_eq!(mldsa_err.to_string(), "ml-dsa: seed refused");
        let slh_err = PqcError::SlhDsa("sign failed".to_string());
        assert_eq!(slh_err.to_string(), "slh-dsa: sign failed");
        let pk_err = PqcError::MalformedPublicKey;
        assert_eq!(pk_err.to_string(), "malformed public key");
        let sig_err = PqcError::MalformedSignature;
        assert_eq!(sig_err.to_string(), "malformed signature");
    }

    #[test]
    fn es256_half_error_converts_into_pqc_error() {
        let es_err: crate::crypto_trust_es256::Es256Error =
            crate::crypto_trust_es256::Es256Error::MalformedPublicKey;
        let pqc_err: PqcError = es_err.into();
        assert!(matches!(pqc_err, PqcError::Es256Half(_)), "got {pqc_err:?}");
        assert!(pqc_err.to_string().starts_with("es256 half of hybrid: "));
    }
}
