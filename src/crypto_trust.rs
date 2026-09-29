//! Cryptographic trust plane for Affidavit.
//!
//! Affidavit owns the semantics of cryptographic identity and standing:
//! key identifiers, public verification material, custody metadata, generations,
//! revocation epochs, replay admission, canonical signed material, signature
//! verification, and the resulting verified witness.
//!
//! It deliberately does **not** require private-key bytes to live in this crate.
//! Signing is delegated through SigningProvider, allowing non-exportable
//! Secure Enclave, TPM, HSM, KMS, remote signer, or software implementations.
//! SA2A/BRCE may consume the verified standing produced here, but authorization
//! remains outside this module: a valid signature proves who signed exact bytes;
//! it does not decide whether a consequence is permitted.

use crate::types::{canonical_bytes, Blake3Hash};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Stable profile carried by every signature envelope produced by this module.
pub const SIGNATURE_PROFILE: &str = "affidavit/crypto-standing/v1";

/// Domain separator prepended to the canonical signed material.
pub const SIGNING_DOMAIN: &[u8] = b"AFFIDAVIT-CRYPTO-STANDING-v1\0";

/// Signature algorithms understood by Affidavit's trust-plane vocabulary.
///
/// Availability is narrower than vocabulary: callers must still have a compiled
/// verifier for the selected algorithm. Today the built-in post-quantum verifier
/// is ML-DSA-65 behind the pqc feature. ES256 and SLH-DSA identifiers are
/// reserved for provider/verifier adapters and are refused by the built-in
/// verifier until those adapters are admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// ECDSA P-256 with SHA-256 (for Secure Enclave/WebAuthn style providers).
    #[serde(rename = "ES256")]
    Es256,
    /// NIST FIPS 204 ML-DSA-65.
    #[serde(rename = "ML-DSA-65")]
    MlDsa65,
    /// NIST FIPS 205 SLH-DSA-SHA2-128s.
    #[serde(rename = "SLH-DSA-SHA2-128s")]
    SlhDsaSha2_128s,
}

/// Declared custody boundary for a key.
///
/// This is evidence metadata, not proof by itself. A higher-level assurance case
/// may require corroborating attestation for the selected custody class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyCustody {
    /// Apple Secure Enclave or an equivalent non-exportable device key.
    SecureEnclave,
    /// TPM-backed non-exportable key.
    Tpm,
    /// Dedicated hardware security module.
    Hsm,
    /// Cloud key-management service.
    Kms,
    /// Software-custodied key outside Affidavit's registry.
    Software,
    /// Another provider described by a stable implementation identifier.
    External(String),
}

/// Lifecycle state of a registered verification key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyState {
    /// Key may verify envelopes within its validity and epoch bounds.
    Active,
    /// Key is no longer acceptable for new verification decisions.
    Revoked,
}

/// Public verification record owned by Affidavit.
///
/// No private key or seed field exists in this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRecord {
    /// Stable key identifier referenced by signature envelopes.
    pub kid: String,
    /// Signature algorithm bound to this key.
    pub algorithm: SignatureAlgorithm,
    /// Encoded public verification key bytes.
    pub public_key: Vec<u8>,
    /// Stable custody identity used by quorum/independence policy.
    pub custodian_id: String,
    /// Where private signing capability is expected to reside.
    pub custody: KeyCustody,
    /// Monotonic replacement generation.
    pub generation: u64,
    /// Monotonic revocation epoch bound into signed material.
    pub revocation_epoch: u64,
    /// Earliest accepted logical timestamp, inclusive.
    pub not_before: u64,
    /// Latest accepted logical timestamp, exclusive.
    pub expires: u64,
    /// Current key lifecycle state.
    pub state: KeyState,
}

/// Deterministic public-key registry.
///
/// Persist the serialized registry in the host's durable store when restart
/// continuity is required. The type itself makes no durability claim.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRegistry {
    keys: BTreeMap<String, KeyRecord>,
}

impl KeyRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new public verification key.
    pub fn register(&mut self, record: KeyRecord) -> Result<(), CryptoRefusal> {
        validate_key_record(&record)?;
        if self.keys.contains_key(&record.kid) {
            return Err(CryptoRefusal::DuplicateKeyId(record.kid));
        }
        self.keys.insert(record.kid.clone(), record);
        Ok(())
    }

    /// Read a registered key by identifier.
    pub fn get(&self, kid: &str) -> Option<&KeyRecord> {
        self.keys.get(kid)
    }

    /// Revoke a key at a strictly newer revocation epoch.
    pub fn revoke(&mut self, kid: &str, new_epoch: u64) -> Result<(), CryptoRefusal> {
        let record = self
            .keys
            .get_mut(kid)
            .ok_or_else(|| CryptoRefusal::UnknownKey(kid.to_owned()))?;
        if new_epoch <= record.revocation_epoch {
            return Err(CryptoRefusal::StaleRevocationEpoch {
                current: record.revocation_epoch,
                proposed: new_epoch,
            });
        }
        record.revocation_epoch = new_epoch;
        record.state = KeyState::Revoked;
        Ok(())
    }

    /// Iterate the registry in stable key-id order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &KeyRecord)> {
        self.keys.iter()
    }
}

/// Exact material covered by a signature.
///
/// Every authorization-relevant coordinate is inside this structure so changing
/// amount/subject/policy/audience/epoch/nonce changes the signed bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SigningMaterial {
    /// Envelope profile.
    pub profile: String,
    /// Signature algorithm.
    pub algorithm: SignatureAlgorithm,
    /// Registered key identifier.
    pub kid: String,
    /// Digest of the exact admitted subject/effect.
    pub subject_digest: Blake3Hash,
    /// Principal for whom the signature is being made.
    pub principal: String,
    /// Policy epoch used to evaluate the request.
    pub policy_epoch: u64,
    /// Revocation epoch observed by the signer.
    pub revocation_epoch: u64,
    /// Key generation observed by the signer.
    pub generation: u64,
    /// Replay nonce unique for this key.
    pub nonce: String,
    /// Earliest valid logical timestamp, inclusive.
    pub not_before: u64,
    /// Latest valid logical timestamp, exclusive.
    pub expires: u64,
    /// Intended verifier/service boundary.
    pub audience: String,
}

impl SigningMaterial {
    /// Construct v1 material with the fixed Affidavit profile.
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        algorithm: SignatureAlgorithm,
        kid: impl Into<String>,
        subject_digest: Blake3Hash,
        principal: impl Into<String>,
        policy_epoch: u64,
        revocation_epoch: u64,
        generation: u64,
        nonce: impl Into<String>,
        not_before: u64,
        expires: u64,
        audience: impl Into<String>,
    ) -> Self {
        Self {
            profile: SIGNATURE_PROFILE.to_owned(),
            algorithm,
            kid: kid.into(),
            subject_digest,
            principal: principal.into(),
            policy_epoch,
            revocation_epoch,
            generation,
            nonce: nonce.into(),
            not_before,
            expires,
            audience: audience.into(),
        }
    }

    /// Canonical, domain-separated bytes that a provider must sign.
    pub fn signing_bytes(&self) -> Result<Vec<u8>, CryptoRefusal> {
        validate_material(self)?;
        let canonical =
            canonical_bytes(self).map_err(|e| CryptoRefusal::Canonicalization(e.to_string()))?;
        let mut out = Vec::with_capacity(SIGNING_DOMAIN.len() + canonical.len());
        out.extend_from_slice(SIGNING_DOMAIN);
        out.extend_from_slice(&canonical);
        Ok(out)
    }
}

/// Signed cryptographic witness transported between trust domains.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureEnvelope {
    /// Exact signed coordinates.
    pub material: SigningMaterial,
    /// Raw algorithm-native signature bytes.
    pub signature: Vec<u8>,
}

/// Provider seam for private signing capability.
///
/// Implementations may wrap Secure Enclave, HSM, KMS, TPM, remote signers, or
/// software keys. The private key representation never appears in Affidavit's
/// public registry or signature envelope.
pub trait SigningProvider {
    /// Stable key id controlled by this provider.
    fn key_id(&self) -> &str;
    /// Algorithm implemented by this provider.
    fn algorithm(&self) -> SignatureAlgorithm;
    /// Sign exact domain-separated bytes.
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CryptoRefusal>;
}

/// Ask a private-key provider to sign exact Affidavit material.
pub fn sign_with_provider<P: SigningProvider>(
    provider: &P,
    material: SigningMaterial,
) -> Result<SignatureEnvelope, CryptoRefusal> {
    if provider.key_id() != material.kid {
        return Err(CryptoRefusal::ProviderKeyMismatch);
    }
    if provider.algorithm() != material.algorithm {
        return Err(CryptoRefusal::ProviderAlgorithmMismatch);
    }
    let bytes = material.signing_bytes()?;
    let signature = provider.sign(&bytes)?;
    if signature.is_empty() {
        return Err(CryptoRefusal::EmptySignature);
    }
    Ok(SignatureEnvelope {
        material,
        signature,
    })
}

/// Verification constraints supplied by the consuming boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationContext<'a> {
    /// Logical timestamp used for validity-window checks.
    pub now: u64,
    /// Exact expected audience.
    pub audience: &'a str,
    /// Exact expected subject digest.
    pub subject_digest: &'a Blake3Hash,
    /// Lowest accepted policy epoch.
    pub minimum_policy_epoch: u64,
}

/// Replay ledger keyed on (kid, nonce).
///
/// Persist this state when replay resistance must survive process restarts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonceLedger {
    used: BTreeSet<(String, String)>,
}

impl NonceLedger {
    /// Return true when this key/nonce pair has already been admitted.
    pub fn contains(&self, kid: &str, nonce: &str) -> bool {
        self.used.contains(&(kid.to_owned(), nonce.to_owned()))
    }

    fn record(&mut self, kid: &str, nonce: &str) {
        self.used.insert((kid.to_owned(), nonce.to_owned()));
    }
}

/// Unconstructable result of successful cryptographic verification.
///
/// This is cryptographic standing only. It is not an authorization grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifiedSignature {
    /// Key that verified the signature.
    pub kid: String,
    /// Verified algorithm.
    pub algorithm: SignatureAlgorithm,
    /// Registry-bound custodian identity.
    pub custodian_id: String,
    /// Exact verified subject digest.
    pub subject_digest: Blake3Hash,
    /// Policy epoch covered by the signature.
    pub policy_epoch: u64,
    /// Revocation epoch covered by the signature.
    pub revocation_epoch: u64,
    /// Key generation covered by the signature.
    pub generation: u64,
    /// Replay nonce that was admitted.
    pub nonce: String,
    /// Audience covered by the signature.
    pub audience: String,
    /// Commitment to the complete signature envelope.
    pub envelope_hash: Blake3Hash,
    #[serde(skip)]
    _seal: (),
}

/// Typed refusal from the cryptographic trust boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CryptoRefusal {
    /// Registry already contains this key id.
    #[error("duplicate_key_id: {0}")]
    DuplicateKeyId(String),
    /// No registered key exists for the envelope.
    #[error("unknown_key: {0}")]
    UnknownKey(String),
    /// Required text field was empty.
    #[error("empty_field: {0}")]
    EmptyField(&'static str),
    /// Public verification material was empty.
    #[error("empty_public_key")]
    EmptyPublicKey,
    /// Signature bytes were empty.
    #[error("empty_signature")]
    EmptySignature,
    /// Digest was not exactly 64 lowercase hexadecimal characters.
    #[error("malformed_subject_digest")]
    MalformedSubjectDigest,
    /// Validity interval is empty or inverted.
    #[error("invalid_validity_window")]
    InvalidValidityWindow,
    /// Key is revoked.
    #[error("key_revoked")]
    KeyRevoked,
    /// Envelope algorithm and registered key algorithm differ.
    #[error("algorithm_mismatch")]
    AlgorithmMismatch,
    /// Envelope generation and registered key generation differ.
    #[error("generation_mismatch")]
    GenerationMismatch,
    /// Envelope revocation epoch and registry epoch differ.
    #[error("revocation_epoch_mismatch")]
    RevocationEpochMismatch,
    /// Revocation updates must strictly advance the epoch.
    #[error("stale_revocation_epoch: current={current}, proposed={proposed}")]
    StaleRevocationEpoch {
        /// Registry epoch.
        current: u64,
        /// Rejected proposed epoch.
        proposed: u64,
    },
    /// Signed profile is not the current Affidavit crypto profile.
    #[error("wrong_profile")]
    WrongProfile,
    /// Envelope is not valid at the supplied logical time.
    #[error("outside_validity_window")]
    OutsideValidityWindow,
    /// Registry key is not valid at the supplied logical time.
    #[error("key_outside_validity_window")]
    KeyOutsideValidityWindow,
    /// Audience did not exactly match the consumer boundary.
    #[error("audience_mismatch")]
    AudienceMismatch,
    /// Subject digest did not exactly match the admitted subject.
    #[error("subject_mismatch")]
    SubjectMismatch,
    /// Signed policy epoch is older than the consumer allows.
    #[error("stale_policy_epoch")]
    StalePolicyEpoch,
    /// This (kid, nonce) has already been admitted.
    #[error("replay_detected")]
    ReplayDetected,
    /// Selected algorithm has no compiled verifier.
    #[error("verifier_unavailable: {0}")]
    VerifierUnavailable(&'static str),
    /// Public key bytes could not be decoded for the selected algorithm.
    #[error("invalid_public_key")]
    InvalidPublicKey,
    /// Signature bytes could not be decoded for the selected algorithm.
    #[error("invalid_signature_encoding")]
    InvalidSignatureEncoding,
    /// Signature did not verify.
    #[error("signature_invalid")]
    SignatureInvalid,
    /// Provider key does not match material key id.
    #[error("provider_key_mismatch")]
    ProviderKeyMismatch,
    /// Provider algorithm does not match material algorithm.
    #[error("provider_algorithm_mismatch")]
    ProviderAlgorithmMismatch,
    /// Deterministic canonicalization failed.
    #[error("canonicalization: {0}")]
    Canonicalization(String),
    /// External signing provider refused or failed.
    #[error("provider: {0}")]
    Provider(String),
}

/// Verify and replay-admit a signature envelope.
///
/// On success the nonce is consumed and the returned VerifiedSignature can be
/// passed upward as cryptographic standing. Authorization is intentionally a
/// separate decision.
pub fn verify_and_record(
    registry: &KeyRegistry,
    nonces: &mut NonceLedger,
    envelope: &SignatureEnvelope,
    context: &VerificationContext<'_>,
) -> Result<VerifiedSignature, CryptoRefusal> {
    validate_material(&envelope.material)?;

    let material = &envelope.material;
    if nonces.contains(&material.kid, &material.nonce) {
        return Err(CryptoRefusal::ReplayDetected);
    }
    if &material.subject_digest != context.subject_digest {
        return Err(CryptoRefusal::SubjectMismatch);
    }
    if material.audience != context.audience {
        return Err(CryptoRefusal::AudienceMismatch);
    }
    if material.policy_epoch < context.minimum_policy_epoch {
        return Err(CryptoRefusal::StalePolicyEpoch);
    }
    if context.now < material.not_before || context.now >= material.expires {
        return Err(CryptoRefusal::OutsideValidityWindow);
    }

    let key = registry
        .get(&material.kid)
        .ok_or_else(|| CryptoRefusal::UnknownKey(material.kid.clone()))?;

    if key.state != KeyState::Active {
        return Err(CryptoRefusal::KeyRevoked);
    }
    if key.algorithm != material.algorithm {
        return Err(CryptoRefusal::AlgorithmMismatch);
    }
    if key.generation != material.generation {
        return Err(CryptoRefusal::GenerationMismatch);
    }
    if key.revocation_epoch != material.revocation_epoch {
        return Err(CryptoRefusal::RevocationEpochMismatch);
    }
    if context.now < key.not_before || context.now >= key.expires {
        return Err(CryptoRefusal::KeyOutsideValidityWindow);
    }
    if envelope.signature.is_empty() {
        return Err(CryptoRefusal::EmptySignature);
    }

    let signed_bytes = material.signing_bytes()?;
    verify_algorithm(
        material.algorithm,
        &key.public_key,
        &signed_bytes,
        &envelope.signature,
    )?;

    let envelope_bytes =
        canonical_bytes(envelope).map_err(|e| CryptoRefusal::Canonicalization(e.to_string()))?;
    let standing = VerifiedSignature {
        kid: material.kid.clone(),
        algorithm: material.algorithm,
        custodian_id: key.custodian_id.clone(),
        subject_digest: material.subject_digest.clone(),
        policy_epoch: material.policy_epoch,
        revocation_epoch: material.revocation_epoch,
        generation: material.generation,
        nonce: material.nonce.clone(),
        audience: material.audience.clone(),
        envelope_hash: Blake3Hash::from_bytes(&envelope_bytes),
        _seal: (),
    };

    nonces.record(&material.kid, &material.nonce);
    Ok(standing)
}

fn validate_key_record(record: &KeyRecord) -> Result<(), CryptoRefusal> {
    if record.kid.trim().is_empty() {
        return Err(CryptoRefusal::EmptyField("kid"));
    }
    if record.custodian_id.trim().is_empty() {
        return Err(CryptoRefusal::EmptyField("custodian_id"));
    }
    if record.public_key.is_empty() {
        return Err(CryptoRefusal::EmptyPublicKey);
    }
    if record.not_before >= record.expires {
        return Err(CryptoRefusal::InvalidValidityWindow);
    }
    Ok(())
}

fn validate_material(material: &SigningMaterial) -> Result<(), CryptoRefusal> {
    if material.profile != SIGNATURE_PROFILE {
        return Err(CryptoRefusal::WrongProfile);
    }
    for (name, value) in [
        ("kid", material.kid.as_str()),
        ("principal", material.principal.as_str()),
        ("nonce", material.nonce.as_str()),
        ("audience", material.audience.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(CryptoRefusal::EmptyField(name));
        }
    }
    if material.not_before >= material.expires {
        return Err(CryptoRefusal::InvalidValidityWindow);
    }
    let digest = material.subject_digest.as_hex().as_bytes();
    if digest.len() != 64
        || !digest
            .iter()
            .all(|b| b.is_ascii_digit() || (*b >= b'a' && *b <= b'f'))
    {
        return Err(CryptoRefusal::MalformedSubjectDigest);
    }
    Ok(())
}

#[cfg(feature = "pqc")]
fn verify_algorithm(
    algorithm: SignatureAlgorithm,
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoRefusal> {
    match algorithm {
        SignatureAlgorithm::MlDsa65 => {
            use ml_dsa::{KeyInit, MlDsa65, Signature, Verifier, VerifyingKey};

            let verifying_key = VerifyingKey::<MlDsa65>::new_from_slice(public_key)
                .map_err(|_| CryptoRefusal::InvalidPublicKey)?;
            let signature = Signature::<MlDsa65>::try_from(signature)
                .map_err(|_| CryptoRefusal::InvalidSignatureEncoding)?;
            verifying_key
                .verify(message, &signature)
                .map_err(|_| CryptoRefusal::SignatureInvalid)
        }
        SignatureAlgorithm::Es256 => Err(CryptoRefusal::VerifierUnavailable("ES256")),
        SignatureAlgorithm::SlhDsaSha2_128s => {
            Err(CryptoRefusal::VerifierUnavailable("SLH-DSA-SHA2-128s"))
        }
    }
}

#[cfg(not(feature = "pqc"))]
fn verify_algorithm(
    algorithm: SignatureAlgorithm,
    _public_key: &[u8],
    _message: &[u8],
    _signature: &[u8],
) -> Result<(), CryptoRefusal> {
    match algorithm {
        SignatureAlgorithm::MlDsa65 => Err(CryptoRefusal::VerifierUnavailable(
            "ML-DSA-65 (compile with feature pqc)",
        )),
        SignatureAlgorithm::Es256 => Err(CryptoRefusal::VerifierUnavailable("ES256")),
        SignatureAlgorithm::SlhDsaSha2_128s => {
            Err(CryptoRefusal::VerifierUnavailable("SLH-DSA-SHA2-128s"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_record() -> KeyRecord {
        KeyRecord {
            kid: "device-1".to_owned(),
            algorithm: SignatureAlgorithm::MlDsa65,
            public_key: vec![1],
            custodian_id: "custodian-1".to_owned(),
            custody: KeyCustody::SecureEnclave,
            generation: 7,
            revocation_epoch: 11,
            not_before: 100,
            expires: 1_000,
            state: KeyState::Active,
        }
    }

    fn material() -> SigningMaterial {
        SigningMaterial::v1(
            SignatureAlgorithm::MlDsa65,
            "device-1",
            Blake3Hash::from_bytes(b"effect"),
            "principal-1",
            5,
            11,
            7,
            "nonce-1",
            100,
            500,
            "sa2a-authority",
        )
    }

    #[test]
    fn registry_refuses_duplicate_key_ids_and_stale_revocation() {
        let mut registry = KeyRegistry::new();
        registry.register(key_record()).unwrap();
        assert!(matches!(
            registry.register(key_record()),
            Err(CryptoRefusal::DuplicateKeyId(_))
        ));
        assert!(matches!(
            registry.revoke("device-1", 11),
            Err(CryptoRefusal::StaleRevocationEpoch { .. })
        ));
        registry.revoke("device-1", 12).unwrap();
        assert_eq!(registry.get("device-1").unwrap().state, KeyState::Revoked);
    }

    #[test]
    fn signing_bytes_bind_every_security_coordinate() {
        let base = material();
        let bytes = base.signing_bytes().unwrap();
        assert!(bytes.starts_with(SIGNING_DOMAIN));

        let mut changed = base.clone();
        changed.policy_epoch += 1;
        assert_ne!(bytes, changed.signing_bytes().unwrap());

        let mut changed = base.clone();
        changed.nonce.push('x');
        assert_ne!(bytes, changed.signing_bytes().unwrap());

        let mut changed = base;
        changed.audience.push('x');
        assert_ne!(bytes, changed.signing_bytes().unwrap());
    }

    struct RefusingProvider;

    impl SigningProvider for RefusingProvider {
        fn key_id(&self) -> &str {
            "device-1"
        }

        fn algorithm(&self) -> SignatureAlgorithm {
            SignatureAlgorithm::MlDsa65
        }

        fn sign(&self, _message: &[u8]) -> Result<Vec<u8>, CryptoRefusal> {
            Err(CryptoRefusal::Provider("offline".to_owned()))
        }
    }

    #[test]
    fn private_key_provider_failures_are_typed_and_do_not_forge_an_envelope() {
        assert_eq!(
            sign_with_provider(&RefusingProvider, material()).unwrap_err(),
            CryptoRefusal::Provider("offline".to_owned())
        );
    }

    #[cfg(feature = "pqc")]
    mod pqc {
        use super::*;
        use ml_dsa::{
            Generate, KeyExport, Keypair, MlDsa65, SignatureEncoding, Signer, SigningKey,
        };

        struct TestSigner {
            kid: String,
            key: SigningKey<MlDsa65>,
        }

        impl SigningProvider for TestSigner {
            fn key_id(&self) -> &str {
                &self.kid
            }

            fn algorithm(&self) -> SignatureAlgorithm {
                SignatureAlgorithm::MlDsa65
            }

            fn sign(&self, message: &[u8]) -> Result<Vec<u8>, CryptoRefusal> {
                Ok(self.key.sign(message).to_bytes().as_slice().to_vec())
            }
        }

        fn fixture() -> (KeyRegistry, TestSigner, SigningMaterial) {
            let key = SigningKey::<MlDsa65>::generate();
            let public_key = key.verifying_key().to_bytes().as_slice().to_vec();
            let signer = TestSigner {
                kid: "ml-dsa-device-1".to_owned(),
                key,
            };
            let mut registry = KeyRegistry::new();
            registry
                .register(KeyRecord {
                    kid: signer.kid.clone(),
                    algorithm: SignatureAlgorithm::MlDsa65,
                    public_key,
                    custodian_id: "device-custodian-1".to_owned(),
                    custody: KeyCustody::Software,
                    generation: 1,
                    revocation_epoch: 3,
                    not_before: 10,
                    expires: 1_000,
                    state: KeyState::Active,
                })
                .unwrap();
            let material = SigningMaterial::v1(
                SignatureAlgorithm::MlDsa65,
                signer.kid.clone(),
                Blake3Hash::from_bytes(b"exact-effect"),
                "principal-1",
                9,
                3,
                1,
                "nonce-42",
                10,
                500,
                "sa2a-authority",
            );
            (registry, signer, material)
        }

        #[test]
        fn real_ml_dsa_65_round_trip_produces_cryptographic_standing() {
            let (registry, signer, material) = fixture();
            let envelope = sign_with_provider(&signer, material.clone()).unwrap();
            let mut nonces = NonceLedger::default();
            let context = VerificationContext {
                now: 20,
                audience: "sa2a-authority",
                subject_digest: &material.subject_digest,
                minimum_policy_epoch: 9,
            };
            let verified =
                verify_and_record(&registry, &mut nonces, &envelope, &context).unwrap();
            assert_eq!(verified.kid, signer.kid);
            assert_eq!(verified.algorithm, SignatureAlgorithm::MlDsa65);
            assert!(nonces.contains(&verified.kid, &verified.nonce));
        }

        #[test]
        fn bit_flip_wrong_subject_replay_and_revocation_are_refused() {
            let (mut registry, signer, material) = fixture();
            let envelope = sign_with_provider(&signer, material.clone()).unwrap();
            let mut nonces = NonceLedger::default();
            let context = VerificationContext {
                now: 20,
                audience: "sa2a-authority",
                subject_digest: &material.subject_digest,
                minimum_policy_epoch: 9,
            };

            let mut bit_flip = envelope.clone();
            bit_flip.signature[0] ^= 0x01;
            assert!(matches!(
                verify_and_record(&registry, &mut nonces, &bit_flip, &context),
                Err(CryptoRefusal::SignatureInvalid)
                    | Err(CryptoRefusal::InvalidSignatureEncoding)
            ));

            let wrong_subject = Blake3Hash::from_bytes(b"other-effect");
            let wrong_context = VerificationContext {
                subject_digest: &wrong_subject,
                ..context.clone()
            };
            assert_eq!(
                verify_and_record(&registry, &mut nonces, &envelope, &wrong_context).unwrap_err(),
                CryptoRefusal::SubjectMismatch
            );

            verify_and_record(&registry, &mut nonces, &envelope, &context).unwrap();
            assert_eq!(
                verify_and_record(&registry, &mut nonces, &envelope, &context).unwrap_err(),
                CryptoRefusal::ReplayDetected
            );

            let mut second = material;
            second.nonce = "nonce-43".to_owned();
            let second = sign_with_provider(&signer, second).unwrap();
            registry.revoke(&signer.kid, 4).unwrap();
            assert_eq!(
                verify_and_record(&registry, &mut NonceLedger::default(), &second, &context)
                    .unwrap_err(),
                CryptoRefusal::KeyRevoked
            );
        }
    }
}
