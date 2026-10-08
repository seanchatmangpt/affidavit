//! k-of-n signature quorum over one envelope signing pre-image for the affidavit
//! cryptographic trust plane. Rendered by ggen sync from affidavit-trust-plane-pack
//! (the pack is authoritative; never edit this file).
//!
//! SA2A-C3 seam (cryptographic layer): a share set is admitted when at least `k`
//! of its shares verify under DISTINCT registered key ids over the SAME signing
//! input ([`crate::crypto_trust_envelope::SignatureEnvelope::signing_input`] or
//! [`crate::crypto_trust_sa2a::sa2a_signing_input`] bytes — the caller owns the
//! pre-image; this module signs nothing and canonicalizes nothing). Signer
//! independence is structural: every share carries its own `key_id`, the
//! duplicate-kid court refuses any repeated signer, and the registry binds each
//! kid to distinct custodian key material (the registry itself refuses duplicate
//! fingerprints, so two custodians can never share a kid).
//!
//! Refusal-vs-verdict law (the asymmetry, witnessed by tests):
//! - STRUCTURAL violations refuse the whole share set as typed
//!   [`QuorumError`] values — policy ceiling (`KExceedsN`), share-count floor
//!   (`InsufficientShares`), repeated signer (`DuplicateSigner`), non-admitted
//!   algorithm (`AlgorithmRefused`), registry evidence (`Registry`), and an
//!   UNDECODABLE share (`InvalidShare`). A contaminated set is never counted.
//! - A WELL-FORMED signature that does not verify is an observation, not a
//!   refusal: it simply does not count toward quorum, and the engine returns a
//!   DECIDED NEGATIVE ([`QuorumVerdict`] with `satisfied: false`). A forgery is
//!   evidence about the quorum, not a parse failure.
//!
//! Threshold law: `k` is a per-call parameter; [`QUORUM_DEFAULT_K`] is the
//! graph-rendered default and [`QUORUM_MAX_N`] the policy ceiling. `k = 0` is
//! the vacuous quorum: any share set that passes the structural courts (in
//! particular the empty set) satisfies it with zero valid shares — callers
//! wanting a real threshold must pass `k >= 1`.
//!
//! House law: certify-don't-decide — a satisfied quorum is cryptographic
//! standing per signature set. The plane NEVER grants authority: what the
//! share set may DO remains SA2A's decision. Refusals are typed values, never
//! panics; no `unwrap`/`expect` on production paths.

// Consumed query columns (quorum.rq): default_k, max_n, domain_tag.

use crate::crypto_trust_es256::verify_es256;
use crate::crypto_trust_keys::{AlgorithmId, KeyId, KeyRecord, KeyRegistry, PublicKeyMaterial};
use crate::crypto_trust_pqc::{
    hybrid_verify, ml_dsa65_verify, slh_dsa128s_verify, HybridSignature,
};
use std::collections::BTreeSet;

/// Default quorum threshold (`ctp:quorum-v1` `ctp:quorumDefaultK`). Callers may
/// pass an explicit `k` per verification; this is the policy's standing default.
pub const QUORUM_DEFAULT_K: usize = 2;

/// Policy ceiling on the quorum threshold (`ctp:quorum-v1` `ctp:quorumMaxN`):
/// a `k` above it is structurally unsatisfiable and is refused with
/// [`QuorumError::KExceedsN`] before any share is examined.
pub const QUORUM_MAX_N: usize = 5;

/// Trust-plane domain tag (`ctp:policy-v1` `ctp:domainTag`). The quorum never
/// mixes it into bytes itself — the caller hands over an already
/// domain-separated signing input — but the module states the plane it serves.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// One signer's share of a quorum: which registered key signed, with what
/// algorithm, and the raw signature bytes over the common signing input. The
/// signature encoding is algorithm-family-shaped: ES256 carries DER, ML-DSA-65
/// and SLH-DSA-SHA2-128s carry their raw fixed-length encodings, and the hybrid
/// carries the JSON wire form of
/// [`crate::crypto_trust_pqc::HybridSignature`] (both halves over the same
/// bytes).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SignatureShare {
    /// The registered key id this share claims — one signer, one kid.
    pub key_id: crate::crypto_trust_keys::KeyId,
    /// The algorithm the share claims; the registry record for `key_id` must
    /// agree (a mismatch is a [`QuorumError::Registry`] refusal).
    pub algorithm: crate::crypto_trust_keys::AlgorithmId,
    /// Raw signature bytes over the common signing input.
    pub signature: Vec<u8>,
}

/// Typed refusals of the quorum court. Values, never panics. Every variant is
/// witnessed by a test; structural refusals contaminate the WHOLE share set.
#[derive(Debug, thiserror::Error)]
pub enum QuorumError {
    #[error("quorum k={k} exceeds n={n}")]
    KExceedsN { k: usize, n: usize },
    #[error("only {got}/{needed} signatures verifiable")]
    InsufficientShares { got: usize, needed: usize },
    #[error("duplicate signer {0} — signer independence violated")]
    DuplicateSigner(String),
    #[error("signature {0} invalid")]
    InvalidShare(String),
    #[error("algorithm {0:?} not admitted by policy")]
    AlgorithmRefused(crate::crypto_trust_keys::AlgorithmId),
    #[error("registry: {0}")]
    Registry(String),
}

/// The outcome of a quorum adjudication: a DECISION, never an authority grant.
/// `satisfied: false` is a decided negative (well-formed shares that failed to
/// verify), distinct from the typed [`QuorumError`] refusals for structural
/// violations.
#[derive(Debug, Clone, PartialEq)]
pub struct QuorumVerdict {
    /// Whether the valid-share count met the requested `k`.
    pub satisfied: bool,
    /// Shares that verified under their registered keys.
    pub valid_shares: usize,
    /// Distinct key ids among the valid shares. Equal to `valid_shares` under
    /// the duplicate-kid court; carried as its own field so the verdict stays
    /// honest if the independence law ever widens.
    pub distinct_signers: usize,
}

/// The quorum court: a borrowed key registry plus the admitted-algorithm
/// policy. Generic over the registry so any [`KeyRegistry`] implementation
/// (in-memory, store-backed) can seat the court.
pub struct QuorumEngine<'a, R: KeyRegistry> {
    registry: &'a R,
    allowed_algorithms: BTreeSet<AlgorithmId>,
}

impl<'a, R: KeyRegistry> QuorumEngine<'a, R> {
    /// Seats the court over `registry` with every graph-admitted algorithm
    /// allowed ([`AlgorithmId::all()`], the plane's default policy).
    pub fn new(registry: &'a R) -> QuorumEngine<'a, R> {
        QuorumEngine {
            registry,
            allowed_algorithms: AlgorithmId::all().iter().copied().collect(),
        }
    }

    /// Narrows the admitted-algorithm policy (builder form). A share claiming
    /// an algorithm outside the set is refused with
    /// [`QuorumError::AlgorithmRefused`] before any key material is touched.
    pub fn with_allowed_algorithms(
        mut self,
        algs: impl IntoIterator<Item = AlgorithmId>,
    ) -> QuorumEngine<'a, R> {
        self.allowed_algorithms = algs.into_iter().collect();
        self
    }

    /// The admitted-algorithm policy in force.
    pub fn allowed_algorithms(&self) -> &BTreeSet<AlgorithmId> {
        &self.allowed_algorithms
    }

    /// Adjudicates a share set over `signing_input` against threshold `k`.
    ///
    /// Order of admission (first hit wins):
    /// 1. policy ceiling — `k > [`QUORUM_MAX_N`]` refuses
    ///    [`QuorumError::KExceedsN`]: a threshold above the registered-signer
    ///    ceiling is unsatisfiable by ANY share set, so the refusal names the
    ///    policy, not the count;
    /// 2. share-count floor — `k > shares.len()` refuses
    ///    [`QuorumError::InsufficientShares`] with `needed = k, got = len`;
    /// 3. independence court — a repeated `key_id` refuses
    ///    [`QuorumError::DuplicateSigner`];
    /// 4. algorithm court — a share claiming a non-admitted algorithm refuses
    ///    [`QuorumError::AlgorithmRefused`];
    /// 5. evidence — every share is verified against the REGISTERED public key
    ///    for its kid: a registry miss or a claimed/registered algorithm
    ///    mismatch refuses [`QuorumError::Registry`]; UNDECODABLE signature
    ///    bytes refuse [`QuorumError::InvalidShare`]; a well-formed signature
    ///    that does not verify simply does not count (decided);
    /// 6. verdict — `valid >= k` decides `satisfied: true`; otherwise the
    ///    decided negative `satisfied: false`. A `k = 0` call is the vacuous
    ///    quorum: satisfied by any set that clears the courts, trivially by
    ///    the empty set (with `valid_shares = distinct_signers = 0`).
    pub fn verify_quorum(
        &self,
        signing_input: &[u8],
        shares: &[SignatureShare],
        k: usize,
    ) -> Result<QuorumVerdict, QuorumError> {
        // 1. Policy ceiling: the threshold itself must be admissible before
        // any share is examined.
        if k > QUORUM_MAX_N {
            return Err(QuorumError::KExceedsN { k, n: QUORUM_MAX_N });
        }
        // 2. Share-count floor: enough shares to even reach k.
        if k > shares.len() {
            return Err(QuorumError::InsufficientShares {
                got: shares.len(),
                needed: k,
            });
        }
        // 3. Independence court: one signer, one kid, once.
        let mut seen: BTreeSet<&KeyId> = BTreeSet::new();
        for share in shares {
            if !seen.insert(&share.key_id) {
                return Err(QuorumError::DuplicateSigner(share.key_id.to_string()));
            }
        }
        // 4. Algorithm court: policy admission precedes key material.
        for share in shares {
            if !self.allowed_algorithms.contains(&share.algorithm) {
                return Err(QuorumError::AlgorithmRefused(share.algorithm));
            }
        }
        // 5. Evidence: verify each share against the REGISTERED key.
        let mut valid_shares = 0usize;
        let mut valid_signers: BTreeSet<&KeyId> = BTreeSet::new();
        for share in shares {
            let Some(record) = self.registry.lookup(&share.key_id) else {
                return Err(QuorumError::Registry(format!(
                    "key {} is not registered",
                    share.key_id
                )));
            };
            if self.share_verifies(record, share.algorithm, signing_input, &share.signature)? {
                valid_shares += 1;
                valid_signers.insert(&share.key_id);
            }
        }
        // 6. Verdict: a decided outcome, never an authority grant.
        Ok(QuorumVerdict {
            satisfied: valid_shares >= k,
            valid_shares,
            distinct_signers: valid_signers.len(),
        })
    }

    /// Verifies ONE share against its registered record. `Ok(false)` is the
    /// decided "well-formed but wrong" observation; `Err` is reserved for
    /// registry-evidence violations and undecodable signature bytes.
    fn share_verifies(
        &self,
        record: &KeyRecord,
        claimed: AlgorithmId,
        signing_input: &[u8],
        signature: &[u8],
    ) -> Result<bool, QuorumError> {
        if record.algorithm != claimed {
            return Err(QuorumError::Registry(format!(
                "key {} is registered as {} but the share claims {}",
                record.id,
                record.algorithm.as_str(),
                claimed.as_str()
            )));
        }
        let kid = record.id.to_string();
        match (&record.public_key, claimed) {
            (PublicKeyMaterial::Es256Sec1(pk), AlgorithmId::Es256) => {
                verify_es256(pk, signing_input, signature)
                    .map_err(|_| QuorumError::InvalidShare(kid))
            }
            (PublicKeyMaterial::MlDsa65(pk), AlgorithmId::MlDsa65) => {
                ml_dsa65_verify(pk, signing_input, signature)
                    .map_err(|_| QuorumError::InvalidShare(kid))
            }
            (PublicKeyMaterial::SlhDsa128s(pk), AlgorithmId::SlhDsa128s) => {
                slh_dsa128s_verify(pk, signing_input, signature)
                    .map_err(|_| QuorumError::InvalidShare(kid))
            }
            (PublicKeyMaterial::Hybrid { es256, mldsa65 }, AlgorithmId::HybridEs256MlDsa65) => {
                let sig: HybridSignature = match serde_json::from_slice(signature) {
                    Ok(sig) => sig,
                    Err(_) => return Err(QuorumError::InvalidShare(kid)),
                };
                hybrid_verify(es256, mldsa65, signing_input, &sig)
                    .map_err(|_| QuorumError::InvalidShare(kid))
            }
            (registered, claimed) => Err(QuorumError::Registry(format!(
                "key {} material serves {} but the share claims {}",
                record.id,
                registered.algorithm().as_str(),
                claimed.as_str()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_envelope::{SignatureEnvelope, ENVELOPE_VERSION};
    use crate::crypto_trust_es256::Es256SigningKey;
    use crate::crypto_trust_keys::{
        fingerprint_public_key, CryptoProfile, CustodianIdentity, InMemoryKeyRegistry, KeyOrigin,
    };
    use crate::crypto_trust_pqc::{
        hybrid_sign, ml_dsa65_from_seed, ml_dsa65_sign, slh_dsa128s_from_seed, slh_dsa128s_sign,
        HybridSecret, MlDsa65KeyPair, SlhDsa128sKeyPair, ML_DSA_65_SEED_LEN,
    };

    /// Deterministic ML-DSA randomizer for fixture signatures (the caller owns
    /// rnd; fixed here so fixtures are byte-reproducible).
    const RND: [u8; 32] = [0x5Au8; 32];

    /// A real ES256 fixture signer: deterministic key from a distinct seed.
    fn es_signer(seed_byte: u8) -> Es256SigningKey {
        let mut seed = [0x11u8; 32];
        seed[0] = seed_byte;
        Es256SigningKey::from_seed(&seed).expect("valid fixture scalar")
    }

    /// A real ML-DSA-65 fixture key pair from a distinct seed.
    fn mldsa_pair(seed_byte: u8) -> MlDsa65KeyPair {
        let mut seed = [0x22u8; ML_DSA_65_SEED_LEN];
        seed[0] = seed_byte;
        ml_dsa65_from_seed(&seed)
    }

    /// A real SLH-DSA-SHA2-128s fixture key from a distinct seed triplet.
    fn slh_pair(seed_byte: u8) -> SlhDsa128sKeyPair {
        let mut seeds = [0x33u8; 48];
        seeds[0] = seed_byte;
        slh_dsa128s_from_seed(&seeds)
    }

    /// Registers a real key record (material + fingerprint + DISTINCT
    /// custodian) and returns its kid.
    fn register(
        registry: &mut InMemoryKeyRegistry,
        algorithm: AlgorithmId,
        material: PublicKeyMaterial,
        custodian: &str,
    ) -> KeyId {
        let fingerprint = fingerprint_public_key(algorithm, &material);
        let record = KeyRecord {
            id: KeyId::from_fingerprint(&fingerprint),
            algorithm,
            fingerprint,
            custodian: CustodianIdentity {
                subject: custodian.to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key: material,
            created_epoch: 1_700_000_000,
        };
        let kid = record.id.clone();
        registry.register(record).expect("fixture key registers");
        kid
    }

    fn register_es(
        registry: &mut InMemoryKeyRegistry,
        signer: &Es256SigningKey,
        custodian: &str,
    ) -> KeyId {
        register(
            registry,
            AlgorithmId::Es256,
            PublicKeyMaterial::Es256Sec1(signer.public_key_sec1()),
            custodian,
        )
    }

    fn register_mldsa(
        registry: &mut InMemoryKeyRegistry,
        pair: &MlDsa65KeyPair,
        custodian: &str,
    ) -> KeyId {
        register(
            registry,
            AlgorithmId::MlDsa65,
            PublicKeyMaterial::MlDsa65(pair.public.clone()),
            custodian,
        )
    }

    /// The common signing pre-image: a REAL envelope signing_input naming
    /// `kid` — every fixture signer signs these exact bytes (the SA2A-C3 seam:
    /// k-of-n over ONE signing input).
    fn quorum_signing_input(kid: &KeyId) -> Vec<u8> {
        let envelope = SignatureEnvelope {
            version: ENVELOPE_VERSION.to_string(),
            algorithm: AlgorithmId::Es256,
            key_id: kid.clone(),
            profile: CryptoProfile::Classical,
            policy_epoch: 1,
            revocation_epoch: 0,
            generation: 1,
            nonce: [0u8; 16],
            not_before: 0,
            expires_at: 4_102_444_800,
            subject_digest: [0x07u8; 32],
            audience: "affidavit.cli".to_string(),
        };
        envelope.signing_input()
    }

    fn es_share(kid: &KeyId, signer: &Es256SigningKey, input: &[u8]) -> SignatureShare {
        SignatureShare {
            key_id: kid.clone(),
            algorithm: AlgorithmId::Es256,
            signature: signer.sign(input),
        }
    }

    fn mldsa_share(kid: &KeyId, seed: &[u8; 32], input: &[u8]) -> SignatureShare {
        let signature = ml_dsa65_sign(seed, input, &RND).expect("fixture ML-DSA signature");
        SignatureShare {
            key_id: kid.clone(),
            algorithm: AlgorithmId::MlDsa65,
            signature,
        }
    }

    // -- teeth first: k-of-n over real distinct signers, same signing input --

    #[test]
    fn graph_consts_are_rendered() {
        assert_eq!(QUORUM_DEFAULT_K, 2);
        assert_eq!(QUORUM_MAX_N, 5);
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
    }

    #[test]
    fn k_of_n_teeth_three_real_es256_signers_satisfy() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let b = es_signer(0x02);
        let c = es_signer(0x03);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        register_es(&mut registry, &b, "custodian-b");
        register_es(&mut registry, &c, "custodian-c");

        // ONE signing input (a real envelope pre-image); three distinct real
        // signers over the SAME bytes.
        let signing_input = quorum_signing_input(&kid_a);
        let shares = [
            es_share(&kid_a, &a, &signing_input),
            es_share(
                &registry_key_of(&registry, "custodian-b"),
                &b,
                &signing_input,
            ),
            es_share(
                &registry_key_of(&registry, "custodian-c"),
                &c,
                &signing_input,
            ),
        ];

        let engine = QuorumEngine::new(&registry);
        let verdict = engine
            .verify_quorum(&signing_input, &shares, 2)
            .expect("structural courts pass");
        assert!(verdict.satisfied);
        assert_eq!(verdict.valid_shares, 3);
        assert_eq!(verdict.distinct_signers, 3);

        // k = n also satisfies; the verdict is deterministic (equal on
        // re-adjudication).
        let again = engine
            .verify_quorum(&signing_input, &shares, 2)
            .expect("structural courts pass");
        assert_eq!(again, verdict, "re-adjudication must decide identically");
        let full = engine
            .verify_quorum(&signing_input, &shares, 3)
            .expect("structural courts pass");
        assert!(full.satisfied);
        assert_eq!(full.valid_shares, 3);
    }

    /// Looks up the kid registered under a fixture custodian.
    fn registry_key_of(registry: &InMemoryKeyRegistry, custodian: &str) -> KeyId {
        registry
            .by_custodian(custodian)
            .first()
            .expect("fixture key registered")
            .id
            .clone()
    }

    // -- every QuorumError variant witnessed ---------------------------------

    #[test]
    fn k_exceeds_n_variant_witnessed_above_policy_ceiling() {
        let registry = InMemoryKeyRegistry::new();
        let engine = QuorumEngine::new(&registry);
        match engine.verify_quorum(b"input", &[], QUORUM_MAX_N + 1) {
            Err(QuorumError::KExceedsN { k, n }) => {
                assert_eq!(k, QUORUM_MAX_N + 1);
                assert_eq!(n, QUORUM_MAX_N);
            }
            outcome => panic!("expected KExceedsN, got {outcome:?}"),
        }
    }

    #[test]
    fn insufficient_shares_variant_witnessed_with_exact_counts() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        let signing_input = quorum_signing_input(&kid_a);
        let shares = [es_share(&kid_a, &a, &signing_input)];

        match QuorumEngine::new(&registry).verify_quorum(&signing_input, &shares, 2) {
            Err(QuorumError::InsufficientShares { got, needed }) => {
                assert_eq!(got, 1);
                assert_eq!(needed, 2);
            }
            outcome => panic!("expected InsufficientShares, got {outcome:?}"),
        }
    }

    #[test]
    fn duplicate_signer_variant_witnessed_same_kid_twice() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let b = es_signer(0x02);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        register_es(&mut registry, &b, "custodian-b");
        let signing_input = quorum_signing_input(&kid_a);

        // The SAME kid twice — the second share even carries a DIFFERENT real
        // signer's signature: exactly the impersonation the independence court
        // must refuse before any signature is examined.
        let shares = [
            es_share(&kid_a, &a, &signing_input),
            SignatureShare {
                key_id: kid_a.clone(),
                algorithm: AlgorithmId::Es256,
                signature: b.sign(&signing_input),
            },
        ];
        match QuorumEngine::new(&registry).verify_quorum(&signing_input, &shares, 2) {
            Err(QuorumError::DuplicateSigner(kid)) => assert_eq!(kid, kid_a.to_string()),
            outcome => panic!("expected DuplicateSigner, got {outcome:?}"),
        }
    }

    #[test]
    fn algorithm_refused_variant_witnessed_policy_without_ml_dsa() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let pair = mldsa_pair(0x02);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        register_mldsa(&mut registry, &pair, "custodian-b");
        let signing_input = quorum_signing_input(&kid_a);

        // Real ML-DSA-65 signature (real pqc sign, explicit rnd) — refused on
        // POLICY, not on cryptography: the engine's set omits MlDsa65.
        let shares = [
            es_share(&kid_a, &a, &signing_input),
            mldsa_share(
                &registry_key_of(&registry, "custodian-b"),
                &pair.seed,
                &signing_input,
            ),
        ];
        let engine = QuorumEngine::new(&registry).with_allowed_algorithms([AlgorithmId::Es256]);
        assert_eq!(
            engine.allowed_algorithms(),
            &BTreeSet::from([AlgorithmId::Es256])
        );
        match engine.verify_quorum(&signing_input, &shares, 2) {
            Err(QuorumError::AlgorithmRefused(alg)) => assert_eq!(alg, AlgorithmId::MlDsa65),
            outcome => panic!("expected AlgorithmRefused, got {outcome:?}"),
        }
    }

    #[test]
    fn registry_variant_witnessed_unregistered_kid() {
        let registry = InMemoryKeyRegistry::new();
        // A real signature from a key the registry has never seen.
        let ghost = es_signer(0x66);
        let ghost_kid = KeyId::from_fingerprint(&ghost.key_id_fingerprint());
        let shares = [es_share(&ghost_kid, &ghost, b"input")];
        match QuorumEngine::new(&registry).verify_quorum(b"input", &shares, 1) {
            Err(QuorumError::Registry(msg)) => assert!(msg.contains(&ghost_kid.to_string())),
            outcome => panic!("expected Registry, got {outcome:?}"),
        }
    }

    #[test]
    fn registry_variant_witnessed_claimed_algorithm_mismatches_record() {
        let mut registry = InMemoryKeyRegistry::new();
        let pair = mldsa_pair(0x02);
        let kid = register_mldsa(&mut registry, &pair, "custodian-b");
        // The share claims ES256 for a kid whose registered record is ML-DSA-65.
        let shares = [SignatureShare {
            key_id: kid.clone(),
            algorithm: AlgorithmId::Es256,
            signature: vec![0x30, 0x00],
        }];
        match QuorumEngine::new(&registry).verify_quorum(b"input", &shares, 1) {
            Err(QuorumError::Registry(msg)) => {
                assert!(msg.contains(&kid.to_string()));
                assert!(msg.contains("ES256"));
                assert!(msg.contains("ML-DSA-65"));
            }
            outcome => panic!("expected Registry, got {outcome:?}"),
        }
    }

    #[test]
    fn invalid_share_variant_witnessed_undecodable_bytes() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let pair = mldsa_pair(0x02);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        let kid_b = register_mldsa(&mut registry, &pair, "custodian-b");
        let signing_input = quorum_signing_input(&kid_a);

        // ES256: bytes that are not DER ECDSA-Sig-Value.
        let shares = [SignatureShare {
            key_id: kid_a.clone(),
            algorithm: AlgorithmId::Es256,
            signature: vec![0xFF, 0x00],
        }];
        match QuorumEngine::new(&registry).verify_quorum(&signing_input, &shares, 1) {
            Err(QuorumError::InvalidShare(kid)) => assert_eq!(kid, kid_a.to_string()),
            outcome => panic!("expected InvalidShare, got {outcome:?}"),
        }

        // ML-DSA-65: bytes whose length is not the FIPS 204 encoding.
        let shares = [SignatureShare {
            key_id: kid_b.clone(),
            algorithm: AlgorithmId::MlDsa65,
            signature: vec![0u8; 10],
        }];
        match QuorumEngine::new(&registry).verify_quorum(&signing_input, &shares, 1) {
            Err(QuorumError::InvalidShare(kid)) => assert_eq!(kid, kid_b.to_string()),
            outcome => panic!("expected InvalidShare, got {outcome:?}"),
        }
    }

    // -- the decided negative: well-formed-but-wrong does not refuse ----------

    #[test]
    fn decided_negative_when_well_formed_signature_does_not_verify() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let b = es_signer(0x02);
        let c = es_signer(0x03);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        let kid_b = register_es(&mut registry, &b, "custodian-b");
        let kid_c = register_es(&mut registry, &c, "custodian-c");
        let signing_input = quorum_signing_input(&kid_a);

        // An ATTACKER's key signs well-formed DER, but the share claims
        // kid_c: verifiable shape, wrong key. It must NOT refuse — it just
        // does not count.
        let attacker = es_signer(0xEE);
        let shares = [
            es_share(&kid_a, &a, &signing_input),
            es_share(&kid_b, &b, &signing_input),
            SignatureShare {
                key_id: kid_c.clone(),
                algorithm: AlgorithmId::Es256,
                signature: attacker.sign(&signing_input),
            },
        ];
        let verdict = QuorumEngine::new(&registry)
            .verify_quorum(&signing_input, &shares, 3)
            .expect("structural courts pass");
        assert_eq!(
            verdict,
            QuorumVerdict {
                satisfied: false,
                valid_shares: 2,
                distinct_signers: 2,
            },
            "a well-formed forgery is a decided negative, not a refusal"
        );
    }

    // -- mixed-algorithm quorum ----------------------------------------------

    #[test]
    fn mixed_algorithm_quorum_counts_es256_and_ml_dsa65() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let pair = mldsa_pair(0x02);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        let kid_b = register_mldsa(&mut registry, &pair, "custodian-b");
        let signing_input = quorum_signing_input(&kid_a);

        let shares = [
            es_share(&kid_a, &a, &signing_input),
            mldsa_share(&kid_b, &pair.seed, &signing_input),
        ];
        let verdict = QuorumEngine::new(&registry)
            .verify_quorum(&signing_input, &shares, 2)
            .expect("structural courts pass");
        assert!(verdict.satisfied);
        assert_eq!(verdict.valid_shares, 2);
        assert_eq!(verdict.distinct_signers, 2);
    }

    #[test]
    fn hybrid_share_counts_toward_quorum_over_the_json_wire_form() {
        let mut registry = InMemoryKeyRegistry::new();
        let secret = HybridSecret {
            es256: es_signer(0x44),
            mldsa65_seed: [0x45u8; ML_DSA_65_SEED_LEN],
        };
        let hybrid_kid = {
            let material = PublicKeyMaterial::Hybrid {
                es256: secret.es256.public_key_sec1(),
                mldsa65: ml_dsa65_from_seed(&secret.mldsa65_seed).public,
            };
            register(
                &mut registry,
                AlgorithmId::HybridEs256MlDsa65,
                material,
                "custodian-h",
            )
        };
        let signing_input = quorum_signing_input(&hybrid_kid);

        // The share carries the JSON wire form of both hybrid halves.
        let sig = hybrid_sign(&secret, &signing_input).expect("hybrid signs");
        let share = SignatureShare {
            key_id: hybrid_kid.clone(),
            algorithm: AlgorithmId::HybridEs256MlDsa65,
            signature: serde_json::to_vec(&sig).expect("hybrid signature serializes"),
        };
        let verdict = QuorumEngine::new(&registry)
            .verify_quorum(&signing_input, &[share], 1)
            .expect("structural courts pass");
        assert!(verdict.satisfied);
        assert_eq!(verdict.valid_shares, 1);

        // Tamper one half: well-formed wire, wrong signature -> decided
        // negative, not a refusal.
        let mut forged = hybrid_sign(&secret, &signing_input).expect("hybrid signs");
        let last = forged.es256_der.len() - 1;
        forged.es256_der[last] ^= 0x01;
        let share = SignatureShare {
            key_id: hybrid_kid,
            algorithm: AlgorithmId::HybridEs256MlDsa65,
            signature: serde_json::to_vec(&forged).expect("hybrid signature serializes"),
        };
        let verdict = QuorumEngine::new(&registry)
            .verify_quorum(&signing_input, &[share], 1)
            .expect("structural courts pass");
        assert!(!verdict.satisfied);
        assert_eq!(verdict.valid_shares, 0);
    }

    #[test]
    fn slh_dsa_share_counts_toward_quorum() {
        let mut registry = InMemoryKeyRegistry::new();
        let pair = slh_pair(0x55);
        let kid = register(
            &mut registry,
            AlgorithmId::SlhDsa128s,
            PublicKeyMaterial::SlhDsa128s(pair.public.clone()),
            "custodian-s",
        );
        let signature = slh_dsa128s_sign(&pair.seeds, b"input").expect("SLH-DSA signs");
        let shares = [SignatureShare {
            key_id: kid,
            algorithm: AlgorithmId::SlhDsa128s,
            signature,
        }];
        let verdict = QuorumEngine::new(&registry)
            .verify_quorum(b"input", &shares, 1)
            .expect("structural courts pass");
        assert!(verdict.satisfied);
        assert_eq!(verdict.valid_shares, 1);
    }

    // -- the vacuous threshold ------------------------------------------------

    #[test]
    fn k_zero_is_the_vacuous_quorum_satisfied_with_zero() {
        let registry = InMemoryKeyRegistry::new();
        let verdict = QuorumEngine::new(&registry)
            .verify_quorum(b"input", &[], 0)
            .expect("k=0 clears the courts");
        assert_eq!(
            verdict,
            QuorumVerdict {
                satisfied: true,
                valid_shares: 0,
                distinct_signers: 0,
            }
        );
    }

    // -- wire form ------------------------------------------------------------

    #[test]
    fn signature_share_serde_round_trips() {
        let mut registry = InMemoryKeyRegistry::new();
        let a = es_signer(0x01);
        let kid_a = register_es(&mut registry, &a, "custodian-a");
        let share = es_share(&kid_a, &a, b"input");
        let json = serde_json::to_string(&share).expect("share serializes");
        let back: SignatureShare = serde_json::from_str(&json).expect("share deserializes");
        assert_eq!(back.key_id, share.key_id);
        assert_eq!(back.algorithm, share.algorithm);
        assert_eq!(back.signature, share.signature);
    }
}
