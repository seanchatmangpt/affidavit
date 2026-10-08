//! Key identity, custody semantics, and the public-key registry for the affidavit cryptographic trust plane.
//
// Consumed query columns (keys.rq): domain_tag, algs, origins, providers.
// Rendered by ggen (affidavit-trust-plane-pack) from the ctp: graph.
// Edit the ontology and re-render; never edit this file by hand.
//
// House law: certify-don't-decide — keys prove identity and custody; they
// never confer authority. Refusals are typed values, never panics.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag).
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Asymmetric signature algorithms admitted by the trust plane, rendered from
/// the `ctp:alg-*` individuals in graph order (ordered by `ctp:algorithmName`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlgorithmId {
    /// ES256
    Es256,
    /// ES256+ML-DSA-65
    HybridEs256MlDsa65,
    /// ML-DSA-65
    MlDsa65,
    /// SLH-DSA-SHA2-128s
    SlhDsa128s,
    /// Ed25519 (AG1 capability lane: Ed25519 envelope signing parity).
    Ed25519,
    /// ES256K — ECDSA P-256-K1/secp256k1, SHA-256 (RFC 8812 `ES256K`;
    /// AG1 capability lane: secp256k1 envelope signing parity).
    Es256k,
}

impl AlgorithmId {
    /// Canonical algorithm name as declared in the graph (`ctp:algorithmName`).
    pub fn as_str(self) -> &'static str {
        match self {
            AlgorithmId::Ed25519 => "ED25519",
            AlgorithmId::Es256 => "ES256",
            AlgorithmId::HybridEs256MlDsa65 => "ES256+ML-DSA-65",
            AlgorithmId::Es256k => "ES256K",
            AlgorithmId::MlDsa65 => "ML-DSA-65",
            AlgorithmId::SlhDsa128s => "SLH-DSA-SHA2-128s",
        }
    }

    /// Fixed public-key length in bytes as declared by the graph
    /// (`ctp:publicKeyLength`), or [`None`] when the graph row omits the
    /// length (projected as 0 = variable/undeclared; use
    /// [`PublicKeyMaterial::encoded_len`] for the concrete byte count of a
    /// specific key).
    pub fn public_key_len(self) -> Option<usize> {
        match self {
            AlgorithmId::Ed25519 => Some(32),
            AlgorithmId::Es256 => Some(65),
            AlgorithmId::HybridEs256MlDsa65 => None,
            AlgorithmId::Es256k => Some(33),
            AlgorithmId::MlDsa65 => Some(1952),
            AlgorithmId::SlhDsa128s => None,
        }
    }

    /// Assurance profile the algorithm serves under the trust-plane policy.
    pub fn profile(self) -> CryptoProfile {
        match self {
            AlgorithmId::Ed25519 => CryptoProfile::Classical,
            AlgorithmId::Es256 => CryptoProfile::Classical,
            AlgorithmId::HybridEs256MlDsa65 => CryptoProfile::Hybrid,
            AlgorithmId::Es256k => CryptoProfile::Classical,
            AlgorithmId::MlDsa65 => CryptoProfile::Pqc,
            AlgorithmId::SlhDsa128s => CryptoProfile::Pqc,
        }
    }

    /// Every algorithm admitted by the graph, in graph order.
    pub fn all() -> &'static [AlgorithmId] {
        &[
            AlgorithmId::Ed25519,
            AlgorithmId::Es256,
            AlgorithmId::HybridEs256MlDsa65,
            AlgorithmId::Es256k,
            AlgorithmId::MlDsa65,
            AlgorithmId::SlhDsa128s,
        ]
    }
}

/// Assurance profile: classical, post-quantum, or hybrid composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CryptoProfile {
    /// Classical cryptography only (e.g. ECDSA P-256).
    Classical,
    /// Composite classical + post-quantum material.
    Hybrid,
    /// Post-quantum only.
    Pqc,
}

impl CryptoProfile {
    /// Variant name; the serde wire form is SCREAMING_SNAKE_CASE.
    pub fn as_str(self) -> &'static str {
        match self {
            CryptoProfile::Classical => "Classical",
            CryptoProfile::Hybrid => "Hybrid",
            CryptoProfile::Pqc => "Pqc",
        }
    }
}

/// Key custody provider kinds, rendered from the graph's key providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyProviderKind {
    Hsm,
    SecureEnclave,
    Software,
}

impl KeyProviderKind {
    /// Provider kind tag as declared in the graph (`ctp:providerKind`).
    pub fn kind(self) -> &'static str {
        match self {
            KeyProviderKind::Hsm => "HSM",
            KeyProviderKind::SecureEnclave => "SECURE_ENCLAVE",
            KeyProviderKind::Software => "SOFTWARE",
        }
    }

    /// Whether the provider may export key material
    /// (`ctp:keyMaterialExportable`).
    pub fn key_material_exportable(self) -> bool {
        match self {
            KeyProviderKind::Hsm => false,
            KeyProviderKind::SecureEnclave => false,
            KeyProviderKind::Software => true,
        }
    }
}

/// Registry-unique key identifier: `afk1_` + the first 16 hex characters of
/// the key fingerprint (21 characters total).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KeyId(pub String);

impl KeyId {
    /// Derives the identifier from a key fingerprint.
    pub fn from_fingerprint(fingerprint: &KeyFingerprint) -> KeyId {
        KeyId(format!("afk1_{}", hex_encode(&fingerprint.0[..8])))
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// BLAKE3 digest, domain-separated under [`DOMAIN_TAG`], over the canonical
/// key encoding (algorithm name + public-key bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KeyFingerprint(pub [u8; 32]);

impl KeyFingerprint {
    /// Lowercase hex encoding (64 characters).
    pub fn as_hex(&self) -> String {
        hex_encode(&self.0)
    }
}

/// Who holds custody of a key: a subject, optionally bound to a device and an
/// organization. Identity only — custody never confers authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustodianIdentity {
    pub subject: String,
    pub device: Option<String>,
    pub org: Option<String>,
}

/// How key material entered the registry, rendered from the graph's key
/// origins. Serde is internally tagged on `"kind"` with SCREAMING_SNAKE_CASE
/// values (`GENERATED`, `IMPORTED`, `HARDWARE_ATTESTED`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyOrigin {
    /// Key lives in hardware and was attested by the named device.
    HardwareAttested { device: String },
    /// Key was imported from an external source.
    Imported { source: String },
    /// Key was generated inside the trust plane.
    Generated,
}

/// Public-key bytes as admitted by the registry, one shape per algorithm
/// family, rendered from the graph's algorithms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKeyMaterial {
    /// Variable-length SEC1 encoding (compressed or uncompressed point).
    Es256Sec1(Vec<u8>),
    /// Composite classical + post-quantum key material.
    Hybrid { es256: Vec<u8>, mldsa65: Vec<u8> },
    /// Fixed-length ML-DSA-65 public key (1952 bytes).
    MlDsa65(Vec<u8>),
    /// Fixed-length SLH-DSA-SHA2-128s public key (0 bytes).
    SlhDsa128s(Vec<u8>),
    /// Raw Ed25519 public key (32 bytes) — AG1 capability lane.
    Ed25519(Vec<u8>),
    /// Compressed SEC1 secp256k1 public key (33 bytes) — AG1 capability lane.
    Es256kSec1(Vec<u8>),
}

impl PublicKeyMaterial {
    /// The algorithm this material serves.
    pub fn algorithm(&self) -> AlgorithmId {
        match self {
            PublicKeyMaterial::Es256Sec1(_) => AlgorithmId::Es256,
            PublicKeyMaterial::Hybrid { .. } => AlgorithmId::HybridEs256MlDsa65,
            PublicKeyMaterial::MlDsa65(_) => AlgorithmId::MlDsa65,
            PublicKeyMaterial::SlhDsa128s(_) => AlgorithmId::SlhDsa128s,
            PublicKeyMaterial::Ed25519(_) => AlgorithmId::Ed25519,
            PublicKeyMaterial::Es256kSec1(_) => AlgorithmId::Es256k,
        }
    }

    /// Canonical encoded length in bytes: the graph-declared fixed length for
    /// families whose graph row declares `publicKeyLength`, or the concrete
    /// buffer size where the row omits it (hybrid material is the sum of its
    /// components).
    pub fn encoded_len(&self) -> usize {
        match self {
            PublicKeyMaterial::Es256Sec1(_) => 65,
            PublicKeyMaterial::Hybrid { es256, mldsa65 } => es256.len() + mldsa65.len(),
            PublicKeyMaterial::MlDsa65(_) => 1952,
            PublicKeyMaterial::SlhDsa128s(bytes) => bytes.len(),
            PublicKeyMaterial::Ed25519(bytes) => bytes.len(),
            PublicKeyMaterial::Es256kSec1(_) => 33,
        }
    }

    /// Canonical byte encoding used for fingerprinting: the raw key bytes;
    /// hybrid material concatenates classical-then-post-quantum.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            PublicKeyMaterial::Es256Sec1(bytes) => bytes.clone(),
            PublicKeyMaterial::Hybrid { es256, mldsa65 } => {
                let mut out = Vec::with_capacity(es256.len() + mldsa65.len());
                out.extend_from_slice(es256);
                out.extend_from_slice(mldsa65);
                out
            }
            PublicKeyMaterial::MlDsa65(bytes) => bytes.clone(),
            PublicKeyMaterial::SlhDsa128s(bytes) => bytes.clone(),
            PublicKeyMaterial::Ed25519(bytes) => bytes.clone(),
            PublicKeyMaterial::Es256kSec1(bytes) => bytes.clone(),
        }
    }
}

/// A registered public key: identity, custody, provenance, and material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyRecord {
    pub id: KeyId,
    pub algorithm: AlgorithmId,
    pub fingerprint: KeyFingerprint,
    pub custodian: CustodianIdentity,
    pub origin: KeyOrigin,
    pub public_key: PublicKeyMaterial,
    pub created_epoch: u64,
}

/// Fingerprints canonical key bytes under the trust-plane domain tag:
/// BLAKE3 digest over `domain_separated(DOMAIN_TAG, [algorithm name, key encoding])`. The
/// fingerprint binds the bytes to the declared algorithm and the domain, so
/// the same bytes under a different algorithm or domain diverge.
pub fn fingerprint_public_key(
    algorithm: AlgorithmId,
    public_key: &PublicKeyMaterial,
) -> KeyFingerprint {
    let encoding = public_key.canonical_bytes();
    // BLAKE3 over the domain-separated encoding -- never the pre-image itself
    // (fingerprint law: domain-separated digest, divergence by alg and bytes).
    let fingerprint = crate::crypto_trust_canonical::digest(
        DOMAIN_TAG,
        &[algorithm.as_str().as_bytes(), encoding.as_slice()],
    );
    KeyFingerprint(fingerprint)
}

/// Typed refusals of the key registry. Values, never panics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RegistryError {
    #[error("duplicate key {0}")]
    Duplicate(KeyId),
    #[error("duplicate fingerprint {0}")]
    DuplicateFingerprint(String),
    #[error("unknown key {0}")]
    Unknown(KeyId),
}

/// The public-key registry: identity-addressed, fingerprint-unique.
pub trait KeyRegistry {
    /// Admits a key record; refuses duplicates by id and by fingerprint.
    fn register(&mut self, record: KeyRecord) -> Result<(), RegistryError>;
    /// Looks a record up by key id.
    fn lookup(&self, id: &KeyId) -> Option<&KeyRecord>;
    /// All records in custody of the given subject.
    fn by_custodian(&self, subject: &str) -> Vec<&KeyRecord>;
    /// Whether the registry holds the given key id.
    fn contains(&self, id: &KeyId) -> bool;
    /// Number of registered keys.
    fn len(&self) -> usize;
    /// Whether the registry holds no keys.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory [`KeyRegistry`] ordered by [`KeyId`], refusing duplicate ids and
/// duplicate fingerprints (each by its own typed refusal).
#[derive(Debug, Default, Clone)]
pub struct InMemoryKeyRegistry {
    records: BTreeMap<KeyId, KeyRecord>,
    fingerprints: BTreeSet<KeyFingerprint>,
}

impl InMemoryKeyRegistry {
    /// Creates an empty registry.
    pub fn new() -> InMemoryKeyRegistry {
        InMemoryKeyRegistry::default()
    }
}

impl KeyRegistry for InMemoryKeyRegistry {
    fn register(&mut self, record: KeyRecord) -> Result<(), RegistryError> {
        if self.records.contains_key(&record.id) {
            return Err(RegistryError::Duplicate(record.id));
        }
        if self.fingerprints.contains(&record.fingerprint) {
            return Err(RegistryError::DuplicateFingerprint(
                record.fingerprint.as_hex(),
            ));
        }
        self.fingerprints.insert(record.fingerprint);
        self.records.insert(record.id.clone(), record);
        Ok(())
    }

    fn lookup(&self, id: &KeyId) -> Option<&KeyRecord> {
        self.records.get(id)
    }

    fn by_custodian(&self, subject: &str) -> Vec<&KeyRecord> {
        self.records
            .values()
            .filter(|record| record.custodian.subject == subject)
            .collect()
    }

    fn contains(&self, id: &KeyId) -> bool {
        self.records.contains_key(id)
    }

    fn len(&self) -> usize {
        self.records.len()
    }
}

/// Lowercase hex encoding without a hex-crate dependency.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::domain_separated;
    use serde_json::to_value;

    /// Builds key material of an algorithm's declared length, salted by `tag`
    /// so distinct test keys never share a fingerprint.
    fn material_for(alg: AlgorithmId, tag: u8) -> PublicKeyMaterial {
        match alg {
            AlgorithmId::Es256 => PublicKeyMaterial::Es256Sec1(vec![tag; 65]),
            AlgorithmId::HybridEs256MlDsa65 => PublicKeyMaterial::Hybrid {
                es256: vec![tag; AlgorithmId::Es256.public_key_len().unwrap_or(33)],
                mldsa65: vec![
                    tag.wrapping_add(1);
                    AlgorithmId::MlDsa65.public_key_len().unwrap_or(1952)
                ],
            },
            AlgorithmId::MlDsa65 => PublicKeyMaterial::MlDsa65(vec![tag; 1952]),
            AlgorithmId::SlhDsa128s => PublicKeyMaterial::SlhDsa128s(vec![tag; 33]),
            AlgorithmId::Ed25519 => PublicKeyMaterial::Ed25519(vec![tag; 32]),
            AlgorithmId::Es256k => PublicKeyMaterial::Es256kSec1(vec![tag; 33]),
        }
    }

    fn record(subject: &str, alg: AlgorithmId, tag: u8) -> KeyRecord {
        let public_key = material_for(alg, tag);
        let fingerprint = fingerprint_public_key(alg, &public_key);
        KeyRecord {
            id: KeyId::from_fingerprint(&fingerprint),
            algorithm: alg,
            fingerprint,
            custodian: CustodianIdentity {
                subject: subject.to_string(),
                device: None,
                org: None,
            },
            origin: KeyOrigin::Generated,
            public_key,
            created_epoch: 1_700_000_000,
        }
    }

    #[test]
    fn domain_tag_is_rendered_from_the_graph() {
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
    }

    #[test]
    fn algorithms_are_ordered_by_graph_name_and_unique() {
        let names: Vec<&str> = AlgorithmId::all().iter().map(|a| a.as_str()).collect();
        assert!(!names.is_empty());
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        assert!(names.windows(2).all(|w| w[0] != w[1]));
    }

    #[test]
    fn algorithm_id_serde_round_trips_all_variants() {
        for alg in AlgorithmId::all() {
            let json = serde_json::to_string(alg).expect("serialize algorithm");
            let back: AlgorithmId = serde_json::from_str(&json).expect("deserialize algorithm");
            assert_eq!(&back, alg);
        }
    }

    #[test]
    fn es256_wire_form_is_screaming_snake_case() {
        assert_eq!(
            serde_json::to_string(&AlgorithmId::Es256).expect("serialize ES256"),
            "\"ES256\""
        );
    }

    #[test]
    fn profiles_follow_the_pinned_policy() {
        assert_eq!(AlgorithmId::Es256.profile(), CryptoProfile::Classical);
        assert_eq!(
            AlgorithmId::HybridEs256MlDsa65.profile(),
            CryptoProfile::Hybrid
        );
        assert_eq!(AlgorithmId::MlDsa65.profile(), CryptoProfile::Pqc);
        assert_eq!(AlgorithmId::SlhDsa128s.profile(), CryptoProfile::Pqc);
    }

    #[test]
    fn crypto_profile_serde_round_trips_and_as_str_matches() {
        for profile in [
            CryptoProfile::Classical,
            CryptoProfile::Hybrid,
            CryptoProfile::Pqc,
        ] {
            let json = serde_json::to_string(&profile).expect("serialize profile");
            let back: CryptoProfile = serde_json::from_str(&json).expect("deserialize profile");
            assert_eq!(back, profile);
            assert_eq!(back.as_str(), profile.as_str());
        }
    }

    #[test]
    fn public_key_len_matches_the_graph() {
        assert_eq!(AlgorithmId::Es256.public_key_len(), Some(65));
        assert_eq!(AlgorithmId::HybridEs256MlDsa65.public_key_len(), None);
        assert_eq!(AlgorithmId::MlDsa65.public_key_len(), Some(1952));
        assert_eq!(AlgorithmId::SlhDsa128s.public_key_len(), None);
    }

    #[test]
    fn encoded_len_agrees_with_declared_fixed_length() {
        for alg in AlgorithmId::all() {
            let material = material_for(*alg, 1);
            match alg.public_key_len() {
                Some(len) => assert_eq!(material.encoded_len(), len),
                None => assert!(material.encoded_len() > 0),
            }
        }
    }

    #[test]
    fn hybrid_encoded_len_sums_components() {
        let mldsa_len = AlgorithmId::MlDsa65.public_key_len().unwrap_or(1952);
        let es256_len = AlgorithmId::Es256.public_key_len().unwrap_or(33);
        let material = PublicKeyMaterial::Hybrid {
            es256: vec![1; es256_len],
            mldsa65: vec![2; mldsa_len],
        };
        assert_eq!(material.encoded_len(), es256_len + mldsa_len);
    }

    #[test]
    fn public_key_material_reports_its_algorithm() {
        for alg in AlgorithmId::all() {
            assert_eq!(material_for(*alg, 1).algorithm(), *alg);
        }
    }

    #[test]
    fn key_provider_kind_renders_from_the_graph() {
        assert_eq!(KeyProviderKind::Hsm.kind(), "HSM");
        assert!(!KeyProviderKind::Hsm.key_material_exportable());
        let back: KeyProviderKind =
            serde_json::from_str("\"HSM\"").expect("deserialize provider kind");
        assert_eq!(back, KeyProviderKind::Hsm);
        assert_eq!(KeyProviderKind::SecureEnclave.kind(), "SECURE_ENCLAVE");
        assert!(!KeyProviderKind::SecureEnclave.key_material_exportable());
        let back: KeyProviderKind =
            serde_json::from_str("\"SECURE_ENCLAVE\"").expect("deserialize provider kind");
        assert_eq!(back, KeyProviderKind::SecureEnclave);
        assert_eq!(KeyProviderKind::Software.kind(), "SOFTWARE");
        assert!(KeyProviderKind::Software.key_material_exportable());
        let back: KeyProviderKind =
            serde_json::from_str("\"SOFTWARE\"").expect("deserialize provider kind");
        assert_eq!(back, KeyProviderKind::Software);
    }

    #[test]
    fn key_origin_serde_round_trips_all_variants() {
        let origins = [
            KeyOrigin::Generated,
            KeyOrigin::Imported {
                source: "pkcs12://backup".to_string(),
            },
            KeyOrigin::HardwareAttested {
                device: "yubikey-9c".to_string(),
            },
        ];
        for origin in origins {
            let json = serde_json::to_string(&origin).expect("serialize origin");
            let back: KeyOrigin = serde_json::from_str(&json).expect("deserialize origin");
            assert_eq!(back, origin);
        }
    }

    #[test]
    fn key_origin_wire_tags_are_screaming_snake_kinds() {
        assert_eq!(
            to_value(KeyOrigin::Generated).expect("origin value")["kind"],
            "GENERATED"
        );
        assert_eq!(
            to_value(KeyOrigin::Imported {
                source: "s".to_string()
            })
            .expect("origin value")["kind"],
            "IMPORTED"
        );
        assert_eq!(
            to_value(KeyOrigin::HardwareAttested {
                device: "d".to_string()
            })
            .expect("origin value")["kind"],
            "HARDWARE_ATTESTED"
        );
    }

    #[test]
    fn key_record_serde_round_trips() {
        let rec = record("subject-a", AlgorithmId::HybridEs256MlDsa65, 7);
        let json = serde_json::to_string(&rec).expect("serialize record");
        let back: KeyRecord = serde_json::from_str(&json).expect("deserialize record");
        assert_eq!(back, rec);
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let alg = AlgorithmId::Es256;
        let pk = material_for(alg, 1);
        assert_eq!(
            fingerprint_public_key(alg, &pk),
            fingerprint_public_key(alg, &pk)
        );
    }

    #[test]
    fn fingerprint_diverges_by_algorithm_for_same_bytes() {
        let bytes = vec![7u8; 32];
        let slh = fingerprint_public_key(
            AlgorithmId::SlhDsa128s,
            &PublicKeyMaterial::SlhDsa128s(bytes.clone()),
        );
        let mldsa =
            fingerprint_public_key(AlgorithmId::MlDsa65, &PublicKeyMaterial::MlDsa65(bytes));
        assert_ne!(slh, mldsa);
    }

    #[test]
    fn fingerprint_diverges_by_bytes_for_same_algorithm() {
        let alg = AlgorithmId::Es256;
        let a = fingerprint_public_key(alg, &PublicKeyMaterial::Es256Sec1(vec![1; 33]));
        let b = fingerprint_public_key(alg, &PublicKeyMaterial::Es256Sec1(vec![2; 33]));
        assert_ne!(a, b);
    }

    #[test]
    fn fingerprint_is_domain_separated() {
        let alg = AlgorithmId::Es256;
        let pk = material_for(alg, 1);
        let fingerprint = fingerprint_public_key(alg, &pk);
        let other_domain = domain_separated(
            "other-domain",
            &[alg.as_str().as_bytes(), pk.canonical_bytes().as_slice()],
        );
        assert_ne!(&fingerprint.0[..], &other_domain[..32]);
    }

    #[test]
    fn fingerprint_hex_is_64_lowercase_characters() {
        let alg = AlgorithmId::Es256;
        let hex = fingerprint_public_key(alg, &material_for(alg, 1)).as_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn key_id_format_law() {
        let alg = AlgorithmId::Es256;
        let fingerprint = fingerprint_public_key(alg, &material_for(alg, 1));
        let id = KeyId::from_fingerprint(&fingerprint);
        assert!(id.to_string().starts_with("afk1_"));
        assert_eq!(id.to_string().len(), 5 + 16);
        assert_eq!(&id.to_string()[5..], &fingerprint.as_hex()[..16]);
        assert_eq!(KeyId::from_fingerprint(&fingerprint), id);
    }

    #[test]
    fn register_then_lookup_round_trips() {
        let mut registry = InMemoryKeyRegistry::new();
        let rec = record("subject-a", AlgorithmId::Es256, 1);
        registry.register(rec.clone()).expect("first registration");
        assert!(registry.contains(&rec.id));
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
        let found = registry.lookup(&rec.id).expect("registered key found");
        assert_eq!(*found, rec);
    }

    #[test]
    fn register_refuses_duplicate_id_by_name() {
        let mut registry = InMemoryKeyRegistry::new();
        let rec = record("subject-a", AlgorithmId::Es256, 1);
        let id = rec.id.clone();
        registry.register(rec.clone()).expect("first registration");
        let err = registry
            .register(rec)
            .expect_err("duplicate id must be refused");
        assert_eq!(err, RegistryError::Duplicate(id));
    }

    #[test]
    fn register_refuses_duplicate_fingerprint_under_different_id() {
        let mut registry = InMemoryKeyRegistry::new();
        let first = record("subject-a", AlgorithmId::Es256, 1);
        let fingerprint_hex = first.fingerprint.as_hex();
        let mut second = record("subject-b", AlgorithmId::Es256, 1);
        second.id = KeyId("afk1_explicitimport".to_string());
        assert_ne!(first.id, second.id);
        registry.register(first).expect("first registration");
        let err = registry
            .register(second)
            .expect_err("duplicate fingerprint must be refused");
        assert_eq!(err, RegistryError::DuplicateFingerprint(fingerprint_hex));
    }

    #[test]
    fn by_custodian_filters_by_subject() {
        let mut registry = InMemoryKeyRegistry::new();
        let a1 = record("subject-a", AlgorithmId::Es256, 1);
        let a2 = record("subject-a", AlgorithmId::MlDsa65, 2);
        let b1 = record("subject-b", AlgorithmId::SlhDsa128s, 3);
        for rec in [&a1, &a2, &b1] {
            registry.register(rec.clone()).expect("registration");
        }
        let got = registry.by_custodian("subject-a");
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|rec| rec.custodian.subject == "subject-a"));
        assert!(registry.by_custodian("subject-z").is_empty());
    }

    #[test]
    fn unknown_key_lookups_are_none() {
        let registry = InMemoryKeyRegistry::new();
        let ghost = KeyId("afk1_0000000000000000".to_string());
        assert!(!registry.contains(&ghost));
        assert!(registry.lookup(&ghost).is_none());
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }
}
