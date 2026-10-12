//! Deontic Context Parameter Binding & Machine Code Enforcement (FR-PQC-06).
//!
//! Enforces ML-DSA-65 deontic context parameter binding:
//! `Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch`
//!
//! Provides strict fail-closed verification rejecting context mismatches or
//! signature invalidity, mapped to machine code `0xE301`.
//!
//! Also provides error mapping and verification primitives for:
//! - ML-KEM decapsulation failure (`0xE302`)
//! - Formal oracle divergence (`0xE303`)

use serde::{Deserialize, Serialize};

use crate::crypto_trust_pqc::{
    PqcError, ML_DSA_65_PUBLIC_KEY_LEN, ML_DSA_65_SEED_LEN, ML_DSA_65_SIGNATURE_LEN,
};

/// Domain-separation context string prefix as defined in STRATA FR-PQC-06.
pub const STRATA_CTX_PREFIX: &[u8] = b"STRATA:";

/// Maximum context string length permitted by FIPS 204 ML-DSA (Algorithm 2).
pub const MAX_ML_DSA_CONTEXT_LEN: usize = 255;

/// Machine code for ML-DSA-65 signature failure or context mismatch (`0xE301`).
pub const CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH: u16 = 0xE301;

/// Machine code for ML-KEM decapsulation failure (`0xE302`).
pub const CODE_ML_KEM_DECAPSULATION_FAILURE: u16 = 0xE302;

/// Machine code for formal oracle divergence (`0xE303`).
pub const CODE_FORMAL_ORACLE_DIVERGENCE: u16 = 0xE303;

/// Machine code for a structural deontic context parameter mismatch (`0xE304`).
pub const CODE_CONTEXT_MISMATCH: u16 = 0xE304;

/// Typed refusal errors for deontic context parameter binding and PQC verification.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeonticError {
    /// ML-DSA-65 signature failure or context mismatch (maps to `0xE301`).
    #[error("ML-DSA-65 signature failure or context mismatch (0xE301): {0}")]
    SignatureOrContextMismatch(String),

    /// ML-KEM decapsulation failure (maps to `0xE302`).
    #[error("ML-KEM decapsulation failure (0xE302): {0}")]
    MlKemDecapsulationFailure(String),

    /// Formal oracle divergence (maps to `0xE303`).
    #[error("Formal oracle divergence (0xE303): {0}")]
    FormalOracleDivergence(String),

    /// Context string exceeds FIPS 204 maximum allowed 255 bytes (maps to `0xE301`).
    #[error("Context length exceeds maximum 255 bytes (0xE301): {0} bytes")]
    ContextTooLong(usize),

    /// Public key format or length is invalid (maps to `0xE301`).
    #[error("Malformed public key (0xE301): {0}")]
    MalformedPublicKey(String),

    /// Signature format or length is invalid (maps to `0xE301`).
    #[error("Malformed signature (0xE301): {0}")]
    MalformedSignature(String),

    /// Expected and actual deontic contexts differ structurally (maps to `0xE304`).
    #[error("Deontic context mismatch (0xE304): {0}")]
    ContextMismatch(String),
}

impl DeonticError {
    /// Return the standardized machine error code.
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        match self {
            Self::SignatureOrContextMismatch(_) => CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH,
            Self::MlKemDecapsulationFailure(_) => CODE_ML_KEM_DECAPSULATION_FAILURE,
            Self::FormalOracleDivergence(_) => CODE_FORMAL_ORACLE_DIVERGENCE,
            Self::ContextMismatch(_) => CODE_CONTEXT_MISMATCH,
            Self::ContextTooLong(_) | Self::MalformedPublicKey(_) | Self::MalformedSignature(_) => {
                CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH
            }
        }
    }

    /// Alias for [`machine_code`].
    #[must_use]
    pub const fn code(&self) -> u16 {
        self.machine_code()
    }
}

impl From<PqcError> for DeonticError {
    fn from(err: PqcError) -> Self {
        match err {
            PqcError::MalformedPublicKey => {
                Self::MalformedPublicKey("invalid ML-DSA public key encoding".to_string())
            }
            PqcError::MalformedSignature => {
                Self::MalformedSignature("invalid ML-DSA signature encoding".to_string())
            }
            other => Self::SignatureOrContextMismatch(other.to_string()),
        }
    }
}

/// Deontic context binding parameters scoping an ML-DSA-65 sovereign signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeonticContext {
    /// Jurisdiction scoping the deontic obligation or permission.
    pub jurisdiction: String,
    /// 32-byte cryptographic digest of the active deontic constraint set.
    pub deontic_constraint_hash: [u8; 32],
    /// Monotonic lease epoch scoping authority validity.
    pub lease_epoch: u64,
}

impl DeonticContext {
    /// Construct a new [`DeonticContext`].
    #[must_use]
    pub fn new(
        jurisdiction: impl Into<String>,
        deontic_constraint_hash: [u8; 32],
        lease_epoch: u64,
    ) -> Self {
        Self {
            jurisdiction: jurisdiction.into(),
            deontic_constraint_hash,
            lease_epoch,
        }
    }

    /// Construct domain-separated context string (length-framed):
    /// `Ctx = "STRATA:" || u32le(len)||Jurisdiction || u32le(len)||Hash || u32le(len)||Epoch`
    ///
    /// # Errors
    /// Returns [`DeonticError::ContextTooLong`] if total length exceeds 255 bytes.
    pub fn to_context_bytes(&self) -> Result<Vec<u8>, DeonticError> {
        construct_deontic_context(
            &self.jurisdiction,
            &self.deontic_constraint_hash,
            self.lease_epoch,
        )
    }
}

/// Constructs domain-separated context string from parameters:
/// `Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch`
///
/// Where `LeaseEpoch` is serialized in 8-byte big-endian format.
///
/// # Errors
/// Returns [`DeonticError::ContextTooLong`] if total byte length exceeds 255 bytes.
pub fn construct_deontic_context(
    jurisdiction: &str,
    deontic_constraint_hash: &[u8],
    lease_epoch: u64,
) -> Result<Vec<u8>, DeonticError> {
    construct_deontic_context_raw(
        jurisdiction.as_bytes(),
        deontic_constraint_hash,
        &lease_epoch.to_be_bytes(),
    )
}

/// Constructs domain-separated context string from raw byte slices, framed:
///
/// ```text
/// Ctx = "STRATA:"
///     || u32le(len(Jurisdiction)) || Jurisdiction
///     || u32le(len(Hash))         || DeonticConstraintHash
///     || u32le(len(Epoch))        || Epoch
/// ```
///
/// The u32 little-endian length prefix before each variable-length field makes
/// the encoding prefix-free over field tuples: distinct `(jurisdiction, hash,
/// epoch)` tuples cannot concatenate to the same byte string (SK3 collision
/// falsification closed; regression-witnessed in this module's tests).
///
/// # Errors
/// Returns [`DeonticError::ContextTooLong`] if total byte length exceeds 255 bytes.
pub fn construct_deontic_context_raw(
    jurisdiction: &[u8],
    deontic_constraint_hash: &[u8],
    lease_epoch: &[u8],
) -> Result<Vec<u8>, DeonticError> {
    let total_len = STRATA_CTX_PREFIX.len()
        + 4
        + jurisdiction.len()
        + 4
        + deontic_constraint_hash.len()
        + 4
        + lease_epoch.len();
    if total_len > MAX_ML_DSA_CONTEXT_LEN {
        return Err(DeonticError::ContextTooLong(total_len));
    }
    let mut ctx = Vec::with_capacity(total_len);
    ctx.extend_from_slice(STRATA_CTX_PREFIX);
    ctx.extend_from_slice(&(jurisdiction.len() as u32).to_le_bytes());
    ctx.extend_from_slice(jurisdiction);
    ctx.extend_from_slice(&(deontic_constraint_hash.len() as u32).to_le_bytes());
    ctx.extend_from_slice(deontic_constraint_hash);
    ctx.extend_from_slice(&(lease_epoch.len() as u32).to_le_bytes());
    ctx.extend_from_slice(lease_epoch);
    Ok(ctx)
}

/// HAZARD: unframed legacy form — DO NOT USE in new code.
///
/// `Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch`
/// is a raw concatenation of variable-length fields, so DISTINCT field tuples
/// collide: e.g. `(jur=b"AB", hash=[], epoch=[1,2,3,4,5,6,7,8])` and
/// `(jur=b"A", hash=[b'B'], epoch=[1,2,3,4,5,6,7,8])` both yield
/// `"STRATA:AB" || epoch`, and a signature bound under one tuple verifies
/// under the other (SK3 falsification). Retained only for replaying
/// legacy-serialized contexts; every new context MUST go through
/// [`construct_deontic_context_raw`] (length-prefixed framing).
#[doc(hidden)]
pub fn construct_deontic_context_unframed(
    jurisdiction: &[u8],
    deontic_constraint_hash: &[u8],
    lease_epoch: &[u8],
) -> Result<Vec<u8>, DeonticError> {
    let total_len = STRATA_CTX_PREFIX.len()
        + jurisdiction.len()
        + deontic_constraint_hash.len()
        + lease_epoch.len();
    if total_len > MAX_ML_DSA_CONTEXT_LEN {
        return Err(DeonticError::ContextTooLong(total_len));
    }
    let mut ctx = Vec::with_capacity(total_len);
    ctx.extend_from_slice(STRATA_CTX_PREFIX);
    ctx.extend_from_slice(jurisdiction);
    ctx.extend_from_slice(deontic_constraint_hash);
    ctx.extend_from_slice(lease_epoch);
    Ok(ctx)
}

/// Deterministically signs `msg` with ML-DSA-65 bound to the given deontic context (FIPS 204).
///
/// # Errors
/// Returns [`DeonticError`] if context construction fails or ML-DSA signing refuses.
pub fn sign_with_deontic_context(
    seed: &[u8; ML_DSA_65_SEED_LEN],
    msg: &[u8],
    context: &DeonticContext,
) -> Result<Vec<u8>, DeonticError> {
    let ctx = context.to_context_bytes()?;
    sign_with_context_raw(seed, msg, &ctx)
}

/// Deterministically signs `msg` with ML-DSA-65 bound to raw context bytes (FIPS 204).
///
/// # Errors
/// Returns [`DeonticError`] if context length exceeds 255 bytes or signing fails.
pub fn sign_with_context_raw(
    seed: &[u8; ML_DSA_65_SEED_LEN],
    msg: &[u8],
    ctx: &[u8],
) -> Result<Vec<u8>, DeonticError> {
    if ctx.len() > MAX_ML_DSA_CONTEXT_LEN {
        return Err(DeonticError::ContextTooLong(ctx.len()));
    }
    let sk_seed: ml_dsa::Seed = (*seed).into();
    let esk = ml_dsa::ExpandedSigningKey::<ml_dsa::MlDsa65>::from_seed(&sk_seed);
    let sig = esk
        .sign_deterministic(msg, ctx)
        .map_err(|e| DeonticError::SignatureOrContextMismatch(format!("ML-DSA signing failed: {e:?}")))?;
    Ok(sig.encode().as_slice().to_vec())
}

/// Enforces ML-DSA-65 deontic context binding verification.
///
/// Verifies that `sig` is an authentic ML-DSA-65 signature over `msg` under public key `public`,
/// cryptographically bound to `context`:
/// `Ctx = "STRATA:" || Jurisdiction || DeonticConstraintHash || LeaseEpoch`.
///
/// # Errors
/// Returns [`DeonticError`] mapping to machine code `0xE301` on signature invalidity,
/// context mismatch, malformed public key, or malformed signature.
pub fn verify_with_deontic_context(
    public: &[u8],
    msg: &[u8],
    sig: &[u8],
    context: &DeonticContext,
) -> Result<(), DeonticError> {
    let ctx = context.to_context_bytes()?;
    verify_with_deontic_context_raw(public, msg, sig, &ctx)
}

/// Enforces ML-DSA-65 deontic context parameter binding directly from parameter fields.
///
/// # Errors
/// Returns [`DeonticError`] mapping to machine code `0xE301` on signature failure or context mismatch.
pub fn verify_with_deontic_context_params(
    public: &[u8],
    msg: &[u8],
    sig: &[u8],
    jurisdiction: &str,
    constraint_hash: &[u8],
    lease_epoch: u64,
) -> Result<(), DeonticError> {
    let ctx = construct_deontic_context(jurisdiction, constraint_hash, lease_epoch)?;
    verify_with_deontic_context_raw(public, msg, sig, &ctx)
}

/// Verifies an ML-DSA-65 signature under raw context bytes.
///
/// # Errors
/// Returns [`DeonticError`] mapping to machine code `0xE301` on signature failure or context mismatch.
pub fn verify_with_deontic_context_raw(
    public: &[u8],
    msg: &[u8],
    sig: &[u8],
    ctx: &[u8],
) -> Result<(), DeonticError> {
    if ctx.len() > MAX_ML_DSA_CONTEXT_LEN {
        return Err(DeonticError::ContextTooLong(ctx.len()));
    }
    if public.len() != ML_DSA_65_PUBLIC_KEY_LEN {
        return Err(DeonticError::MalformedPublicKey(format!(
            "expected {ML_DSA_65_PUBLIC_KEY_LEN} bytes, found {}",
            public.len()
        )));
    }
    if sig.len() != ML_DSA_65_SIGNATURE_LEN {
        return Err(DeonticError::MalformedSignature(format!(
            "expected {ML_DSA_65_SIGNATURE_LEN} bytes, found {}",
            sig.len()
        )));
    }

    let pk = ml_dsa::EncodedVerifyingKey::<ml_dsa::MlDsa65>::try_from(public)
        .map_err(|_| DeonticError::MalformedPublicKey("invalid ML-DSA public key encoding".to_string()))?;
    let vk = ml_dsa::VerifyingKey::<ml_dsa::MlDsa65>::decode(&pk);

    let parsed_sig = ml_dsa::Signature::<ml_dsa::MlDsa65>::try_from(sig)
        .map_err(|_| DeonticError::MalformedSignature("invalid ML-DSA signature encoding".to_string()))?;

    let verified = vk.verify_with_context(msg, ctx, &parsed_sig);
    if !verified {
        return Err(DeonticError::SignatureOrContextMismatch(
            "cryptographic verification failed: invalid signature or deontic context mismatch".to_string(),
        ));
    }
    Ok(())
}

/// Enforces structural and semantic equality between expected and actual deontic contexts.
///
/// # Errors
/// Returns [`DeonticError::ContextMismatch`] (`0xE304`) if any parameter diverges.
pub fn enforce_deontic_context_match(
    expected: &DeonticContext,
    actual: &DeonticContext,
) -> Result<(), DeonticError> {
    if expected != actual {
        return Err(DeonticError::ContextMismatch(format!(
            "deontic context parameter mismatch: expected {expected:?}, found {actual:?}"
        )));
    }
    Ok(())
}

/// Enforces ML-KEM decapsulation integrity against an expected shared secret.
///
/// # Errors
/// Returns [`DeonticError::MlKemDecapsulationFailure`] (`0xE302`) if decapsulation failed
/// or yielded an implicitly rejected divergent shared secret.
pub fn verify_ml_kem_decapsulation(
    expected_ss: &[u8],
    decapsulated_ss: &[u8],
) -> Result<(), DeonticError> {
    if expected_ss != decapsulated_ss {
        return Err(DeonticError::MlKemDecapsulationFailure(
            "ML-KEM decapsulated shared secret does not match expected key material (implicit rejection triggered or corrupted ciphertext)"
                .to_string(),
        ));
    }
    Ok(())
}

/// Enforces consensus between a primary implementation and a formal verification oracle.
///
/// # Errors
/// Returns [`DeonticError::FormalOracleDivergence`] (`0xE303`) if primary and oracle disagree.
pub fn verify_formal_oracle_consensus(
    primary_verdict: bool,
    oracle_verdict: bool,
) -> Result<(), DeonticError> {
    if primary_verdict != oracle_verdict {
        return Err(DeonticError::FormalOracleDivergence(format!(
            "formal oracle divergence: primary verification ({primary_verdict}) disagreed with oracle ({oracle_verdict})"
        )));
    }
    Ok(())
}

/// Enforces byte-for-byte agreement between primary signature and formal oracle output.
///
/// # Errors
/// Returns [`DeonticError::FormalOracleDivergence`] (`0xE303`) if signatures diverge.
pub fn verify_formal_oracle_signature_agreement(
    primary_sig: &[u8],
    oracle_sig: &[u8],
) -> Result<(), DeonticError> {
    if primary_sig != oracle_sig {
        return Err(DeonticError::FormalOracleDivergence(
            "formal oracle divergence: primary signature bytes differ from oracle output".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_pqc::ml_dsa65_from_seed;

    #[test]
    fn test_context_string_construction() {
        let jurisdiction = "US-FED";
        let constraint_hash = [0x5Au8; 32];
        let epoch = 42u64;

        let ctx = construct_deontic_context(jurisdiction, &constraint_hash, epoch)
            .expect("context construction succeeds");

        // Format: "STRATA:" || u32le(len)||Jur || u32le(len)||Hash || u32le(len)||Epoch
        let mut expected = Vec::new();
        expected.extend_from_slice(b"STRATA:");
        expected.extend_from_slice(&6u32.to_le_bytes());
        expected.extend_from_slice(b"US-FED");
        expected.extend_from_slice(&32u32.to_le_bytes());
        expected.extend_from_slice(&[0x5Au8; 32]);
        expected.extend_from_slice(&8u32.to_le_bytes());
        expected.extend_from_slice(&42u64.to_be_bytes());

        assert_eq!(ctx, expected);
        assert_eq!(ctx.len(), 7 + 4 + 6 + 4 + 32 + 4 + 8);
    }

    #[test]
    fn test_deontic_context_struct_roundtrip() {
        let constraint_hash = [0xDE; 32];
        let ctx_obj = DeonticContext::new("EU-GDPR-COURT", constraint_hash, 1024);

        let bytes = ctx_obj.to_context_bytes().expect("valid context bytes");
        assert!(bytes.starts_with(b"STRATA:"));
        // Framed: u32le(12) then the jurisdiction, then framed hash and epoch.
        assert_eq!(&bytes[7..11], &13u32.to_le_bytes());
        assert!(bytes[11..].starts_with(b"EU-GDPR-COURT"));

        let tail_epoch = &bytes[bytes.len() - 8..];
        assert_eq!(tail_epoch, &1024u64.to_be_bytes());
    }

    /// SK3 falsification regression: the raw deontic context was an unframed
    /// concatenation of variable-length fields, so distinct
    /// `(jurisdiction, hash, epoch)` tuples collided on identical context
    /// bytes and a signature bound under one tuple cross-verified under the
    /// other. With u32le length framing the two tuples now yield DIFFERENT
    /// context bytes and cross-verification is REFUSED (0xE301).
    #[test]
    fn test_sk3_raw_context_collision_is_closed() {
        let epoch8 = [1u8, 2, 3, 4, 5, 6, 7, 8];
        // A: jurisdiction "AB", empty hash, epoch bytes.
        // B: jurisdiction "A", hash "B", same epoch bytes.
        // Unframed, both concatenate to "STRATA:AB" || epoch8.
        let ctx_a =
            construct_deontic_context_raw(b"AB", &[], &epoch8).expect("context A constructs");
        let ctx_b =
            construct_deontic_context_raw(b"A", &[b'B'], &epoch8).expect("context B constructs");

        assert_ne!(
            ctx_a, ctx_b,
            "framed contexts for distinct field tuples must differ"
        );

        // The legacy unframed form (doc(hidden), hazard-documented) still
        // exhibits the collision — pinned so the hazard claim is non-vacuous.
        let unframed_a = construct_deontic_context_unframed(b"AB", &[], &epoch8)
            .expect("unframed context A constructs");
        let unframed_b = construct_deontic_context_unframed(b"A", &[b'B'], &epoch8)
            .expect("unframed context B constructs");
        assert_eq!(unframed_a, unframed_b);

        // Cross-verification under the framed contexts must REFUSE: sign
        // under framed A, verify under framed B.
        let seed = [0x77u8; ML_DSA_65_SEED_LEN];
        let kp = ml_dsa65_from_seed(&seed).expect("pqc fixture key pair");
        let msg = b"sk3 collision repro";
        let sig = sign_with_context_raw(&kp.seed, msg, &ctx_a).expect("sign under framed A");
        let res = verify_with_deontic_context_raw(&kp.public, msg, &sig, &ctx_b);
        assert!(res.is_err(), "cross-verification on distinct tuples must be refused");
        assert_eq!(res.unwrap_err().machine_code(), 0xE301);
    }

    #[test]
    fn test_signature_validation_with_matching_context() {
        let seed = [0x33u8; ML_DSA_65_SEED_LEN];
        let kp = ml_dsa65_from_seed(&seed).expect("pqc fixture key pair");
        let msg = b"certified sovereign transaction log entry";

        let context = DeonticContext::new("GLOBAL-DEFENSE-ZONE", [0x7Cu8; 32], 888);

        // Sign with context
        let sig = sign_with_deontic_context(&kp.seed, msg, &context).expect("signing with context");
        assert_eq!(sig.len(), ML_DSA_65_SIGNATURE_LEN);

        // Verify with matching context succeeds
        let verdict = verify_with_deontic_context(&kp.public, msg, &sig, &context);
        assert!(verdict.is_ok(), "Matching deontic context must verify successfully");
    }

    #[test]
    fn test_fail_closed_rejection_on_mismatched_jurisdiction() {
        let seed = [0x44u8; ML_DSA_65_SEED_LEN];
        let kp = ml_dsa65_from_seed(&seed).expect("pqc fixture key pair");
        let msg = b"payload to test jurisdiction domain separation";

        let sign_context = DeonticContext::new("JURISDICTION-ALPHA", [0xAAu8; 32], 500);
        let sig = sign_with_deontic_context(&kp.seed, msg, &sign_context).expect("signing succeeds");

        // Verify with mismatched jurisdiction
        let verify_context = DeonticContext::new("JURISDICTION-BETA", [0xAAu8; 32], 500);
        let res = verify_with_deontic_context(&kp.public, msg, &sig, &verify_context);

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.machine_code(), CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH);
        assert_eq!(err.machine_code(), 0xE301);
    }

    #[test]
    fn test_fail_closed_rejection_on_mismatched_constraint_hash() {
        let seed = [0x55u8; ML_DSA_65_SEED_LEN];
        let kp = ml_dsa65_from_seed(&seed).expect("pqc fixture key pair");
        let msg = b"payload to test deontic constraint hash domain separation";

        let hash_a = [0x11u8; 32];
        let mut hash_b = [0x11u8; 32];
        hash_b[31] ^= 0x01; // 1-bit mutation in constraint hash

        let sign_context = DeonticContext::new("JURISDICTION-SHARED", hash_a, 1200);
        let sig = sign_with_deontic_context(&kp.seed, msg, &sign_context).expect("signing succeeds");

        // Verify with mutated constraint hash
        let verify_context = DeonticContext::new("JURISDICTION-SHARED", hash_b, 1200);
        let res = verify_with_deontic_context(&kp.public, msg, &sig, &verify_context);

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.machine_code(), CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH);
        assert_eq!(err.machine_code(), 0xE301);
    }

    #[test]
    fn test_fail_closed_rejection_on_mismatched_epoch() {
        let seed = [0x66u8; ML_DSA_65_SEED_LEN];
        let kp = ml_dsa65_from_seed(&seed).expect("pqc fixture key pair");
        let msg = b"payload to test lease epoch domain separation";

        let sign_context = DeonticContext::new("JURISDICTION-SHARED", [0x99u8; 32], 100);
        let sig = sign_with_deontic_context(&kp.seed, msg, &sign_context).expect("signing succeeds");

        // Verify with mismatched epoch (epoch expired / rolled over)
        let verify_context = DeonticContext::new("JURISDICTION-SHARED", [0x99u8; 32], 101);
        let res = verify_with_deontic_context(&kp.public, msg, &sig, &verify_context);

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.machine_code(), CODE_DEONTIC_SIG_OR_CONTEXT_MISMATCH);
        assert_eq!(err.machine_code(), 0xE301);
    }

    #[test]
    fn test_enforce_deontic_context_match_rejections() {
        let ctx_orig = DeonticContext::new("ZONE-1", [0x10u8; 32], 7);
        assert!(enforce_deontic_context_match(&ctx_orig, &ctx_orig).is_ok());

        // Differing jurisdiction
        let ctx_diff_jur = DeonticContext::new("ZONE-2", [0x10u8; 32], 7);
        let res = enforce_deontic_context_match(&ctx_orig, &ctx_diff_jur);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().machine_code(), 0xE304);

        // Differing hash
        let ctx_diff_hash = DeonticContext::new("ZONE-1", [0x20u8; 32], 7);
        let res = enforce_deontic_context_match(&ctx_orig, &ctx_diff_hash);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().machine_code(), 0xE304);

        // Differing epoch
        let ctx_diff_epoch = DeonticContext::new("ZONE-1", [0x10u8; 32], 8);
        let res = enforce_deontic_context_match(&ctx_orig, &ctx_diff_epoch);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().machine_code(), 0xE304);
    }

    #[test]
    fn test_context_mismatch_variant_maps_to_0xE304() {
        let err = DeonticError::ContextMismatch("structural divergence".to_string());
        assert_eq!(err.machine_code(), CODE_CONTEXT_MISMATCH);
        assert_eq!(err.machine_code(), 0xE304);
        assert_eq!(err.code(), 0xE304);
    }

    #[test]
    fn test_context_length_overflow_rejection() {
        // ML-DSA limits context to 255 bytes
        let huge_jurisdiction = "A".repeat(260);
        let res = construct_deontic_context(&huge_jurisdiction, &[0u8; 32], 1);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.machine_code(), 0xE301);
        match err {
            DeonticError::ContextTooLong(len) => assert!(len > 255),
            other => panic!("expected ContextTooLong, found {other:?}"),
        }
    }

    #[test]
    fn test_ml_kem_decapsulation_error_mapping() {
        let authentic_ss = [0x42u8; 32];
        let corrupted_ss = [0x43u8; 32];

        // Matching secret passes
        assert!(verify_ml_kem_decapsulation(&authentic_ss, &authentic_ss).is_ok());

        // Mismatched secret fails with 0xE302
        let res = verify_ml_kem_decapsulation(&authentic_ss, &corrupted_ss);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.machine_code(), CODE_ML_KEM_DECAPSULATION_FAILURE);
        assert_eq!(err.machine_code(), 0xE302);
    }

    #[test]
    fn test_formal_oracle_divergence_error_mapping() {
        // Consensus passes
        assert!(verify_formal_oracle_consensus(true, true).is_ok());
        assert!(verify_formal_oracle_consensus(false, false).is_ok());

        // Divergence fails with 0xE303
        let res = verify_formal_oracle_consensus(true, false);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert_eq!(err.machine_code(), CODE_FORMAL_ORACLE_DIVERGENCE);
        assert_eq!(err.machine_code(), 0xE303);

        // Signature agreement checks
        let sig_a = vec![0x12, 0x34];
        let sig_b = vec![0x12, 0x35];
        assert!(verify_formal_oracle_signature_agreement(&sig_a, &sig_a).is_ok());
        let sig_res = verify_formal_oracle_signature_agreement(&sig_a, &sig_b);
        assert!(sig_res.is_err());
        assert_eq!(sig_res.unwrap_err().machine_code(), 0xE303);
    }
}
