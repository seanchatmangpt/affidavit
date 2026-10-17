//! Provider-polymorphic signing seam for the affidavit cryptographic trust plane.
//
// Consumed query columns (provider.rq): domain_tag, envelope_version.
// Rendered by ggen (affidavit-trust-plane-pack) from the ctp: graph.
// Edit the ontology and re-render; never edit this file by hand.
//
// House law: the private key never appears in the registry, the envelope, or
// this module — a provider is addressed by its public coordinates only
// (key id + algorithm), and the private capability is exercised exclusively
// through [`SigningProvider::sign`]. The seam is dispatch-only: it binds the
// provider to the graph-declared envelope identity, computes the exact
// domain-separated signing input via the envelope module's own law
// ([`SignatureEnvelope::signing_input_checked`]), and returns the signature
// DETACHED ([`DetachedSignature`]) because the rendered plane's wire form is
// envelope-plus-signature ([`crate::crypto_trust_verify::VerificationEngine::verify_envelope`]
// takes them separately). The seam never adjudicates: signing proves who
// signed exact bytes; authorization stays outside the trust plane.
//
// Dispatch law: [`sign_with_provider`] refuses a provider whose key id or
// algorithm disagrees with the envelope BEFORE touching private capability —
// a mismatched provider never emits bytes. An empty signature is a typed
// refusal, never an envelope.

use crate::crypto_trust_envelope::{EnvelopeError, SignatureEnvelope};
use crate::crypto_trust_es256::{Es256Error, Es256SigningKey};
use crate::crypto_trust_keys::{
    AlgorithmId, CustodianIdentity, KeyId, KeyOrigin, KeyRecord, PublicKeyMaterial,
};

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag).
/// Conformance-tested equal to [`crate::crypto_trust_canonical::DOMAIN_TAG`]:
/// both render the same graph fact.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Envelope version identity the seam signs under (ctp:policy-v1
/// ctp:envelopeVersion). Conformance-tested equal to
/// [`crate::crypto_trust_envelope::ENVELOPE_VERSION`].
pub const ENVELOPE_VERSION: &str = "CTP-ENVELOPE-v1";

/// Typed refusal of a provider operation; refusal-as-value, never a panic.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProviderRefusal {
    /// Provider controls a key id other than the envelope's.
    #[error("provider key mismatch: envelope {envelope}, provider {provider}")]
    KeyMismatch {
        /// Key id the envelope names.
        envelope: String,
        /// Key id the provider controls.
        provider: String,
    },
    /// Provider implements an algorithm other than the envelope's.
    #[error("provider algorithm mismatch: envelope {envelope}, provider {provider}")]
    AlgorithmMismatch {
        /// Algorithm the envelope names.
        envelope: &'static str,
        /// Algorithm the provider implements.
        provider: &'static str,
    },
    /// The envelope's signed bytes could not be reconstructed (the envelope
    /// module owns the canonicalization law).
    #[error("malformed envelope: {0}")]
    MalformedEnvelope(String),
    /// The provider returned zero signature bytes.
    #[error("empty signature from provider")]
    EmptySignature,
    /// The provider itself refused or failed; the reason is the provider's.
    #[error("provider: {0}")]
    Provider(String),
}

impl From<Es256Error> for ProviderRefusal {
    fn from(err: Es256Error) -> Self {
        ProviderRefusal::Provider(err.to_string())
    }
}

/// The private-capability seam: sign exact domain-separated bytes.
///
/// Implementations wrap software keys, Secure Enclave, TPM, HSM, KMS, or
/// remote signers. The implementation — never this trait — decides where the
/// private key lives; nothing here can export or observe it.
pub trait SigningProvider {
    /// Stable key id controlled by this provider. MUST equal the envelope's
    /// key id for [`sign_with_provider`] to dispatch.
    fn key_id(&self) -> KeyId;
    /// Algorithm this provider signs with. MUST equal the envelope's
    /// algorithm for [`sign_with_provider`] to dispatch.
    fn algorithm(&self) -> AlgorithmId;
    /// Sign the exact domain-separated signing input. Refusal is typed and
    /// must not forge bytes.
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ProviderRefusal>;
}

/// An envelope plus its detached signature, ready for
/// [`crate::crypto_trust_verify::VerificationEngine::verify_envelope`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetachedSignature {
    /// The signed-bytes envelope (the twelve graph-declared fields).
    pub envelope: SignatureEnvelope,
    /// Raw algorithm-native signature over the envelope's signing input.
    pub signature: Vec<u8>,
}

/// Ask a private-key provider to sign the envelope's exact signing input.
///
/// Dispatch order is fixed: (1) key id match, (2) algorithm match, (3)
/// reconstruct the signing input (envelope-owned canonicalization law), (4)
/// invoke the provider, (5) refuse an empty signature. On success the nonce
/// is NOT yet consumed — replay admission is the verifier's decision
/// ([`crate::crypto_trust_verify::VerificationEngine`]), not the signer's.
pub fn sign_with_provider<P: SigningProvider>(
    provider: &P,
    envelope: SignatureEnvelope,
) -> Result<DetachedSignature, ProviderRefusal> {
    if provider.key_id() != envelope.key_id {
        return Err(ProviderRefusal::KeyMismatch {
            envelope: envelope.key_id.0.clone(),
            provider: provider.key_id().0,
        });
    }
    if provider.algorithm() != envelope.algorithm {
        return Err(ProviderRefusal::AlgorithmMismatch {
            envelope: envelope.algorithm.as_str(),
            provider: provider.algorithm().as_str(),
        });
    }
    let input = envelope
        .signing_input_checked()
        .map_err(|e: EnvelopeError| ProviderRefusal::MalformedEnvelope(e.to_string()))?;
    let signature = provider.sign(&input)?;
    if signature.is_empty() {
        return Err(ProviderRefusal::EmptySignature);
    }
    Ok(DetachedSignature {
        envelope,
        signature,
    })
}

/// Software-custodied ES256 provider over the rendered plane's own
/// [`Es256SigningKey`] — the reference implementation proving the seam wires
/// to the plane's real signer, real fingerprint, and real verification
/// engine end to end. `Debug` is hand-written and deliberately omits the
/// key: private material never reaches debug output.
pub struct SoftwareEs256Provider {
    id: KeyId,
    key: Es256SigningKey,
}

impl std::fmt::Debug for SoftwareEs256Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoftwareEs256Provider")
            .field("id", &self.id)
            .field("algorithm", &AlgorithmId::Es256)
            .finish_non_exhaustive()
    }
}

impl SoftwareEs256Provider {
    /// Generate a fresh software key under `id`.
    pub fn generate(id: KeyId) -> Result<Self, ProviderRefusal> {
        Ok(SoftwareEs256Provider {
            id,
            key: Es256SigningKey::generate()?,
        })
    }

    /// Deterministically derive the software key from `seed` under `id`
    /// (tests and KAT vectors; production uses [`generate`]).
    pub fn from_seed(id: KeyId, seed: &[u8; 32]) -> Result<Self, ProviderRefusal> {
        Ok(SoftwareEs256Provider {
            id,
            key: Es256SigningKey::from_seed(seed)?,
        })
    }

    /// The registry record this provider's public side admits: fingerprint
    /// from the plane's own key law, origin `Generated`, SEC1 public bytes.
    /// The private key stays here; only this record leaves.
    pub fn key_record(&self, custodian: CustodianIdentity, created_epoch: u64) -> KeyRecord {
        KeyRecord {
            id: self.id.clone(),
            algorithm: AlgorithmId::Es256,
            fingerprint: self.key.key_id_fingerprint(),
            custodian,
            origin: KeyOrigin::Generated,
            public_key: PublicKeyMaterial::Es256Sec1(self.key.public_key_sec1()),
            created_epoch,
        }
    }
}

impl SigningProvider for SoftwareEs256Provider {
    fn key_id(&self) -> KeyId {
        self.id.clone()
    }

    fn algorithm(&self) -> AlgorithmId {
        AlgorithmId::Es256
    }

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, ProviderRefusal> {
        Ok(self.key.sign(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_keys::{InMemoryKeyRegistry, KeyRegistry};
    use crate::crypto_trust_lifecycle::RevocationList;
    use crate::crypto_trust_verify::{CryptographicStanding, TrustPolicy, VerificationEngine};

    fn provider() -> SoftwareEs256Provider {
        SoftwareEs256Provider::from_seed(KeyId("afk1_provider-test-1".to_string()), &[7u8; 32])
            .expect("seed provider")
    }

    fn envelope_for(kid: KeyId) -> SignatureEnvelope {
        SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::Es256,
            key_id: kid,
            profile: crate::crypto_trust_keys::CryptoProfile::Classical,
            policy_epoch: 1,
            revocation_epoch: 0,
            generation: 1,
            nonce: [9u8; 16],
            not_before: 10,
            expires_at: 1_000,
            subject_digest: [3u8; 32],
            audience: "affidavit.cli".to_string(),
        }
    }

    #[test]
    fn graph_consts_conform_to_the_modules_they_bind() {
        assert_eq!(DOMAIN_TAG, crate::crypto_trust_canonical::DOMAIN_TAG);
        assert_eq!(
            ENVELOPE_VERSION,
            crate::crypto_trust_envelope::ENVELOPE_VERSION
        );
    }

    #[test]
    fn seam_round_trips_through_the_real_verification_engine() {
        let p = provider();
        let mut registry = InMemoryKeyRegistry::new();
        registry
            .register(p.key_record(
                CustodianIdentity {
                    subject: "subject-seam".to_string(),
                    device: None,
                    org: None,
                },
                1,
            ))
            .expect("register provider key");
        let engine = VerificationEngine::new(
            registry,
            RevocationList::default(),
            crate::crypto_trust_envelope::NonceJournal::default(),
            TrustPolicy::from_graph_defaults().with_now(20),
        );

        let env = envelope_for(p.key_id());
        let detached = sign_with_provider(&p, env).expect("sign via seam");
        let verdict = engine
            .verify_envelope(&detached.envelope, &detached.signature)
            .expect("adjudicate");
        assert_eq!(verdict.standing, CryptographicStanding::Valid);
    }

    #[test]
    fn mismatched_provider_identity_is_refused_before_private_capability() {
        let p = provider();
        let other_kid = KeyId("afk1_other-key".to_string());
        let env = envelope_for(other_kid);
        assert!(matches!(
            sign_with_provider(&p, env),
            Err(ProviderRefusal::KeyMismatch { .. })
        ));

        let env = envelope_for(p.key_id());
        let mut wrong_alg = env.clone();
        wrong_alg.algorithm = AlgorithmId::MlDsa65;
        assert!(matches!(
            sign_with_provider(&p, wrong_alg),
            Err(ProviderRefusal::AlgorithmMismatch { .. })
        ));
    }

    #[test]
    fn empty_signature_is_a_typed_refusal_never_an_envelope() {
        struct EmptySigner(KeyId);
        impl SigningProvider for EmptySigner {
            fn key_id(&self) -> KeyId {
                self.0.clone()
            }
            fn algorithm(&self) -> AlgorithmId {
                AlgorithmId::Es256
            }
            fn sign(&self, _message: &[u8]) -> Result<Vec<u8>, ProviderRefusal> {
                Ok(Vec::new())
            }
        }
        let p = provider();
        let env = envelope_for(p.key_id());
        assert_eq!(
            sign_with_provider(&EmptySigner(p.key_id()), env).unwrap_err(),
            ProviderRefusal::EmptySignature
        );
    }

    #[test]
    fn an_uncanonicalizable_envelope_is_refused_not_silently_signed() {
        // Any integer field beyond 2^53 makes the JCS law refuse the signed
        // input; the seam surfaces that as a typed refusal instead of asking
        // the provider to sign unreconstructable bytes.
        let p = provider();
        let mut env = envelope_for(p.key_id());
        env.policy_epoch = 9_007_199_254_740_993; // 2^53 + 1
        assert!(matches!(
            sign_with_provider(&p, env),
            Err(ProviderRefusal::MalformedEnvelope(_))
        ));
    }
}
