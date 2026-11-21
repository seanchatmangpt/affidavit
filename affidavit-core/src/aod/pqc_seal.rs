//! AOD PQC seal — FIPS 204 ML-DSA-65 over the 248-byte shadow slot
//! (AFFI-SPEC-26.11.19).
//!
//! Operator law: **no cryptographic arithmetic in-crate.** Signing and
//! verification both delegate to the external `ml-dsa` crate (via
//! [`crate::pqc::ExternalMlDsa65`] on the verify path).
//!
//! ## Wire layout
//!
//! The 352-byte CTRT shadow ends with a fixed [`AOD_PQC_SEAL_LEN`] (248-byte)
//! slot. A full ML-DSA-65 public key (1952 B) and signature (3309 B) cannot
//! fit there, so the file carries them as an opaque trailer **immediately
//! after** the shadow:
//!
//! ```text
//! … | shadow (352) | pk (1952) | sig (3309) | EOF
//! ```
//!
//! Inside the 248-byte slot:
//!
//! | Offset | Len | Content |
//! |--------|-----|---------|
//! | 0..32  | 32  | Seal message `M` = BLAKE3(DOMAIN_AOD_SEAL ‖ …) |
//! | 32..64 | 32  | `key_id` = BLAKE3(DOMAIN_AOD_KEY_ID ‖ pk) |
//! | 64..248| 184 | Signature tip `sig[0..184]` (quick mismatch) |
//!
//! Envelope verification recomputes `M`, checks the tip, then calls
//! ML-DSA-65 `verify_with_context(M, DOMAIN_AOD_SEAL_CTX, sig, pk)`.
//! Refusal on any failure is [`AodRefusal::PqcSealFailure`] (`0xE50C`).

use crate::digest::{Blake3Hasher, ChainHasher};
use crate::pqc::{ExternalMlDsa65, MlDsa65Verifier};
use crate::sealed::{ML_DSA_65_PK_LEN, ML_DSA_65_SIG_LEN};

use super::layout::{AodShadow, AOD_PQC_SEAL_LEN, DOMAIN_AOD_SEAL};
use super::refuse::AodRefusal;

/// FIPS 204 context string for every AOD PQC seal (never stuffed into `M`).
pub const DOMAIN_AOD_SEAL_CTX: &[u8] = b"affidavit-core/aod/seal/v1";

/// Domain tag for the sealing-key fingerprint ([`aod_key_id`]).
pub const DOMAIN_AOD_KEY_ID: &[u8] = b"affidavit-core/aod/key-id/v1";

/// Domain tag for deriving a deterministic ML-DSA seed from the ontology hash
/// when the caller does not supply an explicit seed (CLI / compat path).
#[cfg(any(feature = "alloc", test))]
pub const DOMAIN_AOD_MLDSA_SEED: &[u8] = b"affidavit-core/aod/v1\0mldsa-seed";

/// Bytes of ML-DSA-65 material appended after the 352-byte shadow.
pub const AOD_PQC_MATERIAL_LEN: usize = ML_DSA_65_PK_LEN + ML_DSA_65_SIG_LEN; // 5261

/// Signature tip length stored in `pqc_seal[64..248)`.
pub const AOD_PQC_SIG_TIP_LEN: usize = AOD_PQC_SEAL_LEN - 64; // 184

/// Recompute the 32-byte seal message over shadow identity fields.
///
/// `M = BLAKE3(DOMAIN_AOD_SEAL ‖ payload ‖ epoch_le ‖ nullifier ‖ folded)`.
/// This is the FIPS 204 signed message; the context is [`DOMAIN_AOD_SEAL_CTX`].
pub fn seal_message_digest(
    payload_digest: &[u8; 32],
    lease_epoch: u64,
    lease_nullifier: &[u8; 32],
    folded_receipt: &[u8; 32],
) -> [u8; 32] {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, DOMAIN_AOD_SEAL);
    Blake3Hasher::absorb(&mut st, payload_digest);
    Blake3Hasher::absorb(&mut st, &lease_epoch.to_le_bytes());
    Blake3Hasher::absorb(&mut st, lease_nullifier);
    Blake3Hasher::absorb(&mut st, folded_receipt);
    Blake3Hasher::finish(st).0
}

/// Domain-separated key fingerprint `blake3(DOMAIN_AOD_KEY_ID ‖ pk)`.
pub fn aod_key_id(pk: &[u8]) -> [u8; 32] {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, DOMAIN_AOD_KEY_ID);
    Blake3Hasher::absorb(&mut st, pk);
    Blake3Hasher::finish(st).0
}

/// Derive a deterministic ML-DSA-65 seed from the compiled ontology hash.
///
/// Used by [`super::writer::seal_aod`] when no explicit seed is supplied so
/// the CLI path stays byte-stable without a new flag. Callers that need an
/// independent key pass [`sign_aod_seal`] a fresh seed directly.
#[cfg(any(feature = "alloc", test))]
pub fn derive_mldsa_seed(ontology_hash: &[u8; 32]) -> [u8; 32] {
    let mut st = Blake3Hasher::init();
    Blake3Hasher::absorb(&mut st, DOMAIN_AOD_MLDSA_SEED);
    Blake3Hasher::absorb(&mut st, ontology_hash);
    Blake3Hasher::finish(st).0
}

/// Pack the 248-byte `pqc_seal` slot from message, public key, and signature.
#[cfg(any(feature = "alloc", test))]
pub fn encode_pqc_seal_slot(
    msg: &[u8; 32],
    pk: &[u8],
    sig: &[u8],
) -> Result<[u8; AOD_PQC_SEAL_LEN], AodRefusal> {
    if pk.len() != ML_DSA_65_PK_LEN || sig.len() != ML_DSA_65_SIG_LEN {
        return Err(AodRefusal::PqcSealFailure);
    }
    let mut seal = [0u8; AOD_PQC_SEAL_LEN];
    seal[..32].copy_from_slice(msg);
    seal[32..64].copy_from_slice(&aod_key_id(pk));
    seal[64..].copy_from_slice(&sig[..AOD_PQC_SIG_TIP_LEN]);
    Ok(seal)
}

/// `(pqc_seal slot, pk, sig)` produced by [`compute_pqc_seal`].
#[cfg(any(feature = "alloc", test))]
pub type PqcSealParts = (
    [u8; AOD_PQC_SEAL_LEN],
    [u8; ML_DSA_65_PK_LEN],
    [u8; ML_DSA_65_SIG_LEN],
);

/// Sign shadow identity fields and pack the 248-byte slot + pk‖sig trailer.
#[cfg(any(feature = "alloc", test))]
pub fn compute_pqc_seal(
    payload_digest: &[u8; 32],
    lease_epoch: u64,
    lease_nullifier: &[u8; 32],
    folded_receipt: &[u8; 32],
    mldsa_seed: &[u8; 32],
) -> Result<PqcSealParts, AodRefusal> {
    let msg = seal_message_digest(payload_digest, lease_epoch, lease_nullifier, folded_receipt);
    let (pk, sig) = sign_aod_seal(mldsa_seed, &msg)?;
    let slot = encode_pqc_seal_slot(&msg, &pk, &sig)?;
    Ok((slot, pk, sig))
}

/// Sign seal message `msg` under [`DOMAIN_AOD_SEAL_CTX`] with a seed-derived
/// ML-DSA-65 key. Returns `(pk, sig)` encoded lengths.
///
/// Deterministic: same seed + message ⇒ identical signature bytes.
#[cfg(any(feature = "alloc", test))]
pub fn sign_aod_seal(
    seed: &[u8; 32],
    msg: &[u8; 32],
) -> Result<([u8; ML_DSA_65_PK_LEN], [u8; ML_DSA_65_SIG_LEN]), AodRefusal> {
    use ml_dsa::{KeyInit, Keypair, MlDsa65, SignatureEncoding, SigningKey};

    let sk = SigningKey::<MlDsa65>::new(&ml_dsa::Seed::from(*seed));
    let vk = sk.verifying_key();
    let sig = sk
        .expanded_key()
        .sign_deterministic(msg, DOMAIN_AOD_SEAL_CTX)
        .map_err(|_| AodRefusal::PqcSealFailure)?;
    let pk: [u8; ML_DSA_65_PK_LEN] = vk.encode().into();
    let sig_bytes: [u8; ML_DSA_65_SIG_LEN] = sig.to_bytes().into();
    Ok((pk, sig_bytes))
}

/// Split the post-shadow PQC trailer into `(pk, sig)`.
pub fn split_pqc_material(material: &[u8]) -> Result<(&[u8], &[u8]), AodRefusal> {
    if material.len() != AOD_PQC_MATERIAL_LEN {
        return Err(AodRefusal::PqcSealFailure);
    }
    Ok((&material[..ML_DSA_65_PK_LEN], &material[ML_DSA_65_PK_LEN..]))
}

/// Verify the shadow's 248-byte seal against the post-shadow ML-DSA material.
///
/// Steps: recompute `M`, check slot layout (message + key_id + tip), then
/// [`ExternalMlDsa65::verify_with_context`]. Any mismatch → `0xE50C`.
pub fn verify_pqc_seal(shadow: &AodShadow, pk: &[u8], sig: &[u8]) -> Result<(), AodRefusal> {
    if pk.len() != ML_DSA_65_PK_LEN || sig.len() != ML_DSA_65_SIG_LEN {
        return Err(AodRefusal::PqcSealFailure);
    }

    let msg = seal_message_digest(
        &shadow.payload_digest,
        shadow.lease_epoch,
        &shadow.lease_nullifier,
        &shadow.folded_receipt,
    );
    if shadow.pqc_seal[..32] != msg {
        return Err(AodRefusal::PqcSealFailure);
    }
    if shadow.pqc_seal[32..64] != aod_key_id(pk) {
        return Err(AodRefusal::PqcSealFailure);
    }
    if shadow.pqc_seal[64..] != sig[..AOD_PQC_SIG_TIP_LEN] {
        return Err(AodRefusal::PqcSealFailure);
    }

    if !ExternalMlDsa65.verify_with_context(&msg, DOMAIN_AOD_SEAL_CTX, sig, pk) {
        return Err(AodRefusal::PqcSealFailure);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aod::layout::AOD_PQC_SEAL_LEN;

    const SEED: [u8; 32] = [0xA1; 32];

    fn sample_shadow(
        msg_fields: ([u8; 32], u64, [u8; 32], [u8; 32]),
    ) -> (AodShadow, [u8; ML_DSA_65_PK_LEN], [u8; ML_DSA_65_SIG_LEN]) {
        let (payload, epoch, nullifier, folded) = msg_fields;
        let msg = seal_message_digest(&payload, epoch, &nullifier, &folded);
        let (pk, sig) = sign_aod_seal(&SEED, &msg).expect("sign");
        let pqc_seal = encode_pqc_seal_slot(&msg, &pk, &sig).expect("encode");
        let shadow = AodShadow {
            payload_digest: payload,
            lease_epoch: epoch,
            lease_nullifier: nullifier,
            folded_receipt: folded,
            pqc_seal,
        };
        (shadow, pk, sig)
    }

    #[test]
    fn positive_court_honest_seal_accepts() {
        let (shadow, pk, sig) = sample_shadow(([0x11; 32], 7, [0x22; 32], [0x33; 32]));
        verify_pqc_seal(&shadow, &pk, &sig).expect("honest ML-DSA-65 seal");
        assert_eq!(shadow.pqc_seal.len(), AOD_PQC_SEAL_LEN);
        assert_eq!(AOD_PQC_MATERIAL_LEN, 1952 + 3309);
        assert_eq!(AOD_PQC_SIG_TIP_LEN, 184);
    }

    #[test]
    fn negative_court_sig_bitflip_refuses_e50c() {
        let (shadow, pk, mut sig) = sample_shadow(([0x44; 32], 1, [0x55; 32], [0x66; 32]));
        sig[100] ^= 0x01;
        let err = verify_pqc_seal(&shadow, &pk, &sig).unwrap_err();
        assert_eq!(err, AodRefusal::PqcSealFailure);
        assert_eq!(err.machine_code(), 0xE50C);
    }

    #[test]
    fn negative_court_seal_slot_bitflip_refuses_e50c() {
        let (mut shadow, pk, sig) = sample_shadow(([0x77; 32], 2, [0x88; 32], [0x99; 32]));
        shadow.pqc_seal[0] ^= 0x01; // flip message byte
        let err = verify_pqc_seal(&shadow, &pk, &sig).unwrap_err();
        assert_eq!(err.machine_code(), 0xE50C);
    }

    #[test]
    fn negative_court_wrong_key_refuses_e50c() {
        let (shadow, _pk, sig) = sample_shadow(([0xAA; 32], 3, [0xBB; 32], [0xCC; 32]));
        let other_seed = [0xB2; 32];
        let msg = seal_message_digest(
            &shadow.payload_digest,
            shadow.lease_epoch,
            &shadow.lease_nullifier,
            &shadow.folded_receipt,
        );
        let (other_pk, _) = sign_aod_seal(&other_seed, &msg).unwrap();
        let err = verify_pqc_seal(&shadow, &other_pk, &sig).unwrap_err();
        assert_eq!(err.machine_code(), 0xE50C);
    }

    #[test]
    fn deterministic_seed_same_sig() {
        let msg = seal_message_digest(&[1; 32], 9, &[2; 32], &[3; 32]);
        let (pk1, sig1) = sign_aod_seal(&SEED, &msg).unwrap();
        let (pk2, sig2) = sign_aod_seal(&SEED, &msg).unwrap();
        assert_eq!(pk1, pk2);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn empty_ctx_rejects_context_bound_sig() {
        let msg = seal_message_digest(&[9; 32], 0, &[8; 32], &[7; 32]);
        let (pk, sig) = sign_aod_seal(&SEED, &msg).unwrap();
        assert!(
            ExternalMlDsa65.verify_with_context(&msg, DOMAIN_AOD_SEAL_CTX, &sig, &pk),
            "control"
        );
        assert!(
            !ExternalMlDsa65.verify_with_context(&msg, &[], &sig, &pk),
            "COURT-VACUITY: ctx-bound AOD sig must fail empty ctx"
        );
    }
}
