//! Durable, tamper-evident public-key store for the affidavit cryptographic trust plane.
//
// Consumed query columns (store.rq): domain_tag, digest_alg, store_format, store_file.
// Rendered by ggen (affidavit-trust-plane-pack) from the ctp: graph.
// Edit the ontology and re-render; never edit this file by hand.
//
// House law: certify-don't-decide — the store persists admitted key records
// and detects at-rest mutation; it never confers authority. Refusals are
// typed values, never panics.
//
// Tamper-evidence law: a store file's checksum is the lowercase hex of
// `digest(DOMAIN_TAG, &[jcs(records).as_bytes()])` — the same domain-separated
// BLAKE3 used by the rest of the plane, taken over the JCS canonicalization of
// the records alone (the checksum never covers itself). Load recomputes the
// checksum and refuses any divergence as
// [`StoreError::ChecksumMismatch`]; mutation of a stored byte therefore
// surfaces as a typed refusal on the next read or register, never as silent
// acceptance.
//
// Atomicity law: writes go to a unique sibling temporary file
// (`.<name>.tmp-<pid>-<blake3(time, pid, path)[..16]>`), created with
// `create_new(true)` so anything already staged at that path — a pre-placed
// symlink above all — is REFUSED at the open, never followed, closing the
// arbitrary-file-clobber class structurally; the temporary is then flushed
// with `sync_all` and moved into place with `rename`. On POSIX and Windows
// `rename` within one directory is atomic, so a crash before the rename
// leaves the previous store intact (readers never see a partial file) and at
// worst an orphaned dot-prefixed temporary. The pid + blake3(time) suffixes
// keep concurrent processes from staging into one temporary and keep the
// staging name unpredictable in advance; a nanos-clock tie duplicating a
// name is a typed refusal (fail-closed), never a clobber. On a failed rename
// the temporary is removed best-effort.

use crate::crypto_trust_keys::{KeyId, KeyRecord, KeyRegistry, RegistryError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The staging sibling for an atomic write of the file at `path`: same
/// directory (same filesystem, so `rename` is atomic), name
/// `.<name>.tmp-<pid>-<blake3(time, pid, path)[..16]>` — unique per attempt
/// (concurrent processes never stage into one temporary) and unpredictable
/// in advance (an attacker cannot pre-plant the path the next write stages
/// at). A nanos-clock tie duplicating a name is a typed refusal at the
/// `create_new(true)` open, never a clobber.
fn tmp_sibling(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "store".to_string(), |n| n.to_string_lossy().to_string());
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let seed = format!("{nanos}|{}|{}", std::process::id(), path.display());
    let digest = blake3::hash(seed.as_bytes()).to_hex();
    path.with_file_name(format!(
        ".{name}.tmp-{}-{}",
        std::process::id(),
        &digest[..16]
    ))
}

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag) mixed into the
/// store checksum digest. Conformance-tested equal to
/// [`crate::crypto_trust_keys::DOMAIN_TAG`]: both render the same graph fact.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Digest algorithm over the domain-separated checksum pre-image
/// (ctp:canon-JCS ctp:digestAlgorithm). Conformance-tested equal to
/// [`crate::crypto_trust_canonical::DIGEST_ALGORITHM`].
pub const DIGEST_ALGORITHM: &str = "BLAKE3";

/// Store format/version identity stamped in the file
/// (ctp:store-v1 ctp:storeFormat). The value IS the store version: load
/// refuses any other value ([`StoreError::WrongFormat`]).
pub const STORE_FORMAT: &str = "CTP-STORE-v1";

/// Store file location (ctp:store-v1 ctp:storeFile), relative to the consumer
/// repository root.
pub const STORE_FILE: &str = ".affi/keys.json";

/// An at-rest key store file: format identity, records, tamper-evidence
/// checksum. The checksum binds the records (never the format field — the
/// format is checked structurally before the checksum runs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyStoreFile {
    /// Must equal [`STORE_FORMAT`] on load ([`StoreError::WrongFormat`]).
    pub format: String,
    /// Admitted key records, in registration order.
    pub records: Vec<KeyRecord>,
    /// Lowercase hex of `digest(DOMAIN_TAG, &[jcs(records).as_bytes()])`.
    pub checksum: String,
}

/// Typed refusals of the key store. Values, never panics.
///
/// The first five variants are the contract surface; `Canonical` completes it:
/// [`crate::crypto_trust_canonical::jcs`] refuses integers beyond double
/// precision (2^53) as non-canonical, so a record carrying such a
/// `created_epoch` cannot be checksummed — the store refuses it (certify,
/// never coerce) instead of silently hashing an unparsable pre-image.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// A stored record byte changed after the checksum was written.
    #[error("store checksum mismatch: claimed {claimed}, recomputed {recomputed}")]
    ChecksumMismatch {
        /// Checksum found in the file.
        claimed: String,
        /// Checksum recomputed over the records as read.
        recomputed: String,
    },
    /// The file's format identity is not [`STORE_FORMAT`].
    #[error("wrong store format: {0}")]
    WrongFormat(String),
    /// The store location is unwritable or unreadable (missing parent
    /// directory, path component is a regular file, permission denied, ...).
    #[error("key store io: {0}")]
    Io(String),
    /// A record cannot be canonicalized for checksumming (integer beyond
    /// double precision — see [`crate::crypto_trust_canonical::jcs`]).
    #[error("key store canonicalization: {0}")]
    Canonical(String),
    /// A refusal surfaced by the underlying key registry (duplicate id,
    /// duplicate fingerprint).
    #[error("key registry: {0}")]
    Registry(#[from] RegistryError),
    /// The store file is not valid JSON, or a record cannot be (de)serialized.
    #[error("key store serialization: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Checksum over store records: lowercase hex of the domain-separated
/// BLAKE3 digest over `jcs(records)`. Deterministic; diverges on any record
/// byte change (tamper-evidence law).
pub fn checksum_for(records: &[KeyRecord]) -> Result<String, StoreError> {
    let value = serde_json::to_value(records)?;
    let canonical = crate::crypto_trust_canonical::jcs(&value)
        .map_err(|e| StoreError::Canonical(e.to_string()))?;
    Ok(crate::crypto_trust_canonical::digest_hex(
        DOMAIN_TAG,
        &[canonical.as_bytes()],
    ))
}

/// A [`KeyRegistry`] persisted at a fixed path, tamper-evident at rest.
///
/// Write path: every `register` performs load → duplicate check → modify →
/// atomic write against the file's CURRENT contents, so concurrent stores at
/// one path never silently drop each other's admissions.
///
/// Read path: trait reads ([`KeyRegistry::lookup`], [`KeyRegistry::by_custodian`],
/// [`KeyRegistry::contains`], [`KeyRegistry::len`]) serve the in-memory mirror
/// loaded by [`FileKeyStore::open`] (and refreshed by each register). To read
/// the file's fresh contents with the full tamper law, use [`FileKeyStore::load`]
/// / [`FileKeyStore::records`]; [`FileKeyStore::open`] refuses a tampered file
/// outright, so a mirror was lawful when it was loaded.
///
/// An absent store file is an empty store: the first register creates it
/// (creating the parent directory). An UNWRITABLE location — a path component
/// that is a regular file, a missing directory that cannot be created,
/// permission denial — is a typed [`StoreError::Io`].
#[derive(Debug)]
pub struct FileKeyStore {
    path: PathBuf,
    records: BTreeMap<KeyId, KeyRecord>,
}

impl FileKeyStore {
    /// Opens (and validates) the store at `path`: runs the full load law —
    /// JSON parse, format identity, checksum — and refuses a tampered or
    /// foreign-format file with the exact [`StoreError`].
    pub fn open<P: Into<PathBuf>>(path: P) -> Result<FileKeyStore, StoreError> {
        let path = path.into();
        let probe = FileKeyStore {
            path,
            records: BTreeMap::new(),
        };
        let loaded = probe.load()?;
        let mut records = BTreeMap::new();
        for record in loaded.records {
            records.insert(record.id.clone(), record);
        }
        Ok(FileKeyStore {
            path: probe.path,
            records,
        })
    }

    /// The store file location.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads the file's fresh contents under the full load law: an absent file
    /// is an empty store; a wrong format identity is
    /// [`StoreError::WrongFormat`] (checked before the checksum, so a
    /// foreign-format file is refused on its structure, not its bytes); a
    /// checksum divergence is [`StoreError::ChecksumMismatch`] with the
    /// claimed and recomputed values.
    pub fn load(&self) -> Result<KeyStoreFile, StoreError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(KeyStoreFile {
                    format: STORE_FORMAT.to_string(),
                    records: Vec::new(),
                    checksum: checksum_for(&[])?,
                });
            }
            Err(e) => {
                return Err(StoreError::Io(format!("read {}: {e}", self.path.display())));
            }
        };
        let store: KeyStoreFile = serde_json::from_str(&text)?;
        if store.format != STORE_FORMAT {
            return Err(StoreError::WrongFormat(store.format));
        }
        let recomputed = checksum_for(&store.records)?;
        if store.checksum != recomputed {
            return Err(StoreError::ChecksumMismatch {
                claimed: store.checksum,
                recomputed,
            });
        }
        Ok(store)
    }

    /// Freshly-read records ([`FileKeyStore::load`] then take the records).
    pub fn records(&self) -> Result<Vec<KeyRecord>, StoreError> {
        Ok(self.load()?.records)
    }

    /// Registers a record with the full typed refusal surface:
    /// duplicate id / fingerprint as [`StoreError::Registry`], tampered or
    /// foreign-format file as [`StoreError::ChecksumMismatch`] /
    /// [`StoreError::WrongFormat`], unwritable location as
    /// [`StoreError::Io`], unparsable file as
    /// [`StoreError::Serialization`], uncanonicalizable record as
    /// [`StoreError::Canonical`].
    pub fn register_checked(&mut self, record: KeyRecord) -> Result<(), StoreError> {
        let mut store = self.load()?;
        if store.records.iter().any(|r| r.id == record.id) {
            return Err(StoreError::Registry(RegistryError::Duplicate(record.id)));
        }
        if store
            .records
            .iter()
            .any(|r| r.fingerprint == record.fingerprint)
        {
            return Err(StoreError::Registry(RegistryError::DuplicateFingerprint(
                record.fingerprint.as_hex(),
            )));
        }
        store.records.push(record);
        store.checksum = checksum_for(&store.records)?;
        self.write_atomic(&store)?;
        self.records.clear();
        for record in store.records {
            self.records.insert(record.id.clone(), record);
        }
        Ok(())
    }

    /// Writes the store file atomically: serialize → unique sibling temporary
    /// ([`tmp_sibling`], created with `create_new(true)`) → `sync_all` →
    /// `rename`. The rename lands all bytes at once or none, so readers
    /// either see the previous store or the complete new one; a crash before
    /// the rename leaves the old store intact and at most an orphaned
    /// dot-prefixed temporary. On a failed rename the temporary is removed
    /// best-effort.
    fn write_atomic(&self, store: &KeyStoreFile) -> Result<(), StoreError> {
        let body = serde_json::to_string_pretty(store)?;
        let tmp = tmp_sibling(&self.path);
        self.write_atomic_via(body.as_bytes(), &tmp)
    }

    /// The write law with the staging path made explicit — the exact code
    /// [`FileKeyStore::write_atomic`] runs, with only the (unpredictable)
    /// staging name lifted into an argument so the symlink-attack court can
    /// plant one and watch the refusal fire. The stage open runs
    /// `create_new(true)`: anything already at `tmp` (a pre-placed symlink
    /// above all) is refused, never followed.
    fn write_atomic_via(&self, body: &[u8], tmp: &Path) -> Result<(), StoreError> {
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)
            .map_err(|e| StoreError::Io(format!("create dir {}: {e}", parent.display())))?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(tmp)
            .map_err(|e| {
                StoreError::Io(format!(
                    "stage {}: refused (never follows an existing path): {e}",
                    tmp.display()
                ))
            })?;
        file.write_all(body)
            .map_err(|e| StoreError::Io(format!("write {}: {e}", tmp.display())))?;
        file.sync_all()
            .map_err(|e| StoreError::Io(format!("sync {}: {e}", tmp.display())))?;
        match std::fs::rename(tmp, &self.path) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(tmp);
                Err(StoreError::Io(format!(
                    "rename {} -> {}: {e}",
                    tmp.display(),
                    self.path.display()
                )))
            }
        }
    }
}

impl KeyRegistry for FileKeyStore {
    /// Load → duplicate check → modify → atomic write. Duplicate refusals are
    /// exact ([`RegistryError::Duplicate`] / [`RegistryError::DuplicateFingerprint`]).
    /// Environmental refusals (checksum mismatch, wrong format, io,
    /// serialization, canonicalization) cannot be expressed by
    /// [`RegistryError`]; they surface as [`RegistryError::Unknown(id)`] —
    /// after the refusal the id is factually not admitted — and their exact
    /// typed value is on [`FileKeyStore::register_checked`].
    fn register(&mut self, record: KeyRecord) -> Result<(), RegistryError> {
        let id = record.id.clone();
        match self.register_checked(record) {
            Ok(()) => Ok(()),
            Err(StoreError::Registry(refusal)) => Err(refusal),
            Err(_) => Err(RegistryError::Unknown(id)),
        }
    }

    /// Looks a record up in the in-memory mirror (see the read-path note on
    /// [`FileKeyStore`]).
    fn lookup(&self, id: &KeyId) -> Option<&KeyRecord> {
        self.records.get(id)
    }

    /// All mirrored records in custody of the given subject.
    fn by_custodian(&self, subject: &str) -> Vec<&KeyRecord> {
        self.records
            .values()
            .filter(|record| record.custodian.subject == subject)
            .collect()
    }

    /// Whether the mirror holds the given key id.
    fn contains(&self, id: &KeyId) -> bool {
        self.records.contains_key(id)
    }

    /// Number of mirrored records.
    fn len(&self) -> usize {
        self.records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DIGEST_ALGORITHM as CANONICAL_DIGEST_ALGORITHM;
    use crate::crypto_trust_keys::{
        fingerprint_public_key, AlgorithmId, CustodianIdentity, KeyOrigin, PublicKeyMaterial,
        DOMAIN_TAG as KEYS_DOMAIN_TAG,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Unique scratch directory per call (no tempfile dependency; std only).
    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("ctp-store-{tag}-{pid}-{n}"));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

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

    fn es256_record(subject: &str, tag: u8) -> KeyRecord {
        record(subject, AlgorithmId::Es256, tag)
    }

    /// Flips exactly one byte inside the stored records: the custodian subject
    /// "subject-a" becomes "tubject-a" (first occurrence; the key id is hex
    /// and never contains it). The result stays valid JSON, so the refusal
    /// must come from the checksum, not the parser.
    fn flip_one_record_byte(text: &str) -> String {
        let flipped = text.replacen("\"subject-a\"", "\"tubject-a\"", 1);
        assert_ne!(flipped, text, "fixture must contain the flipped byte");
        flipped
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(DIGEST_ALGORITHM, "BLAKE3");
        assert_eq!(STORE_FORMAT, "CTP-STORE-v1");
        assert_eq!(STORE_FILE, ".affi/keys.json");
    }

    #[test]
    fn rendered_consts_agree_with_sibling_modules() {
        // Same graph fact rendered by two rules: the store checksum domain
        // and the digest algorithm must equal the keys/canonical modules'.
        assert_eq!(DOMAIN_TAG, KEYS_DOMAIN_TAG);
        assert_eq!(DIGEST_ALGORITHM, CANONICAL_DIGEST_ALGORITHM);
    }

    #[test]
    fn register_persist_reload_lookup_round_trips() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("keys.json");
        let rec = es256_record("subject-a", 1);
        let rec_id = rec.id.clone();
        {
            let mut store = FileKeyStore::open(&path).expect("open fresh store");
            assert!(store.is_empty());
            store.register(rec.clone()).expect("register");
        }
        let reopened = FileKeyStore::open(&path).expect("reopen store");
        assert_eq!(reopened.len(), 1);
        assert!(reopened.contains(&rec_id));
        assert_eq!(reopened.lookup(&rec_id), Some(&rec));
        assert_eq!(reopened.by_custodian("subject-a").len(), 1);
        assert_eq!(reopened.by_custodian("subject-z").len(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn absent_store_file_is_an_empty_store() {
        let dir = temp_dir("absent");
        let path = dir.join("nested").join("keys.json");
        let mut store = FileKeyStore::open(&path).expect("absent file is empty");
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
        assert!(!store.contains(&KeyId("afk1_0000000000000000".to_string())));
        store
            .register(es256_record("subject-a", 1))
            .expect("first admission creates the store");
        assert!(path.is_file(), "store file created at {:?}", path);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tampered_records_refused_as_checksum_mismatch() {
        let dir = temp_dir("tamper");
        let path = dir.join("keys.json");
        let rec = es256_record("subject-a", 1);
        {
            let mut store = FileKeyStore::open(&path).expect("open fresh store");
            store.register(rec).expect("register");
        }
        let intact = std::fs::read_to_string(&path).expect("read store file");
        let claimed: String = serde_json::from_str::<KeyStoreFile>(&intact)
            .expect("parse intact store")
            .checksum;
        // Flip a byte inside the records: subject "subject-a" -> "tubject-a".
        let tampered = flip_one_record_byte(&intact);
        std::fs::write(&path, tampered).expect("write tampered store");
        let err = FileKeyStore::open(&path).expect_err("tampered store must be refused");
        match err {
            StoreError::ChecksumMismatch {
                claimed: c,
                recomputed: r,
            } => {
                assert_eq!(c, claimed, "claimed checksum is the pre-tamper value");
                assert_ne!(r, claimed, "recomputed checksum must diverge");
            }
            other => panic!("expected ChecksumMismatch, got {other:?}"),
        }
        // The write path is tamper-evident too: register re-loads and refuses.
        let mut store = FileKeyStore {
            path: path.clone(),
            records: BTreeMap::new(),
        };
        let err = store
            .register_checked(es256_record("subject-b", 2))
            .expect_err("register on tampered store must be refused");
        assert!(matches!(err, StoreError::ChecksumMismatch { .. }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_format_refused_before_checksum() {
        let dir = temp_dir("format");
        let path = dir.join("keys.json");
        let foreign = KeyStoreFile {
            format: "CTP-STORE-v0".to_string(),
            records: Vec::new(),
            checksum: "not-even-a-checksum".to_string(),
        };
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&foreign).expect("serialize"),
        )
        .expect("write foreign-format store");
        let store = FileKeyStore {
            path: path.clone(),
            records: BTreeMap::new(),
        };
        let err = store.load().expect_err("foreign format must be refused");
        match err {
            StoreError::WrongFormat(fmt) => assert_eq!(fmt, "CTP-STORE-v0"),
            other => panic!("expected WrongFormat, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn malformed_store_file_refused_as_serialization() {
        let dir = temp_dir("malformed");
        let path = dir.join("keys.json");
        // Invalid JSON without any literal brace pairs (Tera renders this file).
        std::fs::write(&path, "[format: oops]").expect("write malformed store");
        let store = FileKeyStore {
            path: path.clone(),
            records: BTreeMap::new(),
        };
        let err = store.load().expect_err("malformed json must be refused");
        assert!(matches!(err, StoreError::Serialization(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unwritable_location_refused_as_io() {
        let dir = temp_dir("unwritable");
        // (a) Dir-as-file: a store path beneath a REGULAR FILE is unreadable
        // (ENOTDIR on read), so the load law refuses at open with Io.
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, b"not a directory").expect("write blocker file");
        let path = blocker.join("keys.json");
        let err = FileKeyStore::open(&path).expect_err("location under a file must be refused");
        assert!(matches!(err, StoreError::Io(_)));
        // (b) The location turns unwritable between open and register: the
        // write path (create_dir_all / staging) refuses with Io.
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).expect("create sub dir");
        let path = sub.join("keys.json");
        let mut store = FileKeyStore::open(&path).expect("absent file is empty");
        std::fs::remove_dir(&sub).expect("remove sub dir");
        std::fs::write(&sub, b"now a regular file").expect("replace with regular file");
        let err = store
            .register_checked(es256_record("subject-a", 1))
            .expect_err("register into a blocked location must be refused");
        assert!(matches!(err, StoreError::Io(_)));
        // The trait surface maps the environmental refusal lossily (documented):
        // RegistryError cannot express Io, so the id comes back as Unknown.
        let rec = es256_record("subject-b", 2);
        let expected_id = rec.id.clone();
        let err = store
            .register(rec)
            .expect_err("trait register must refuse too");
        assert_eq!(err, RegistryError::Unknown(expected_id));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn duplicate_refusals_via_pre_seeded_file() {
        let dir = temp_dir("duplicate");
        let path = dir.join("keys.json");
        let rec = es256_record("subject-a", 1);
        {
            let mut store = FileKeyStore::open(&path).expect("open fresh store");
            store.register(rec.clone()).expect("seed first record");
        }
        let mut store = FileKeyStore::open(&path).expect("reopen pre-seeded store");
        // Same id, freshly built: exact Duplicate refusal.
        let err = store
            .register_checked(rec.clone())
            .expect_err("duplicate id must be refused");
        match err {
            StoreError::Registry(RegistryError::Duplicate(id)) => assert_eq!(id, rec.id),
            other => panic!("expected Registry(Duplicate), got {other:?}"),
        }
        // Same fingerprint under a different id: exact DuplicateFingerprint.
        let mut other_id = es256_record("subject-b", 1);
        other_id.id = KeyId("afk1_explicitimport".to_string());
        assert_ne!(other_id.id, rec.id);
        let fingerprint_hex = rec.fingerprint.as_hex();
        let err = store
            .register_checked(other_id)
            .expect_err("duplicate fingerprint must be refused");
        match err {
            StoreError::Registry(RegistryError::DuplicateFingerprint(hex)) => {
                assert_eq!(hex, fingerprint_hex)
            }
            other => panic!("expected Registry(DuplicateFingerprint), got {other:?}"),
        }
        // The trait surface passes registry refusals through exactly.
        let err = store
            .register(rec)
            .expect_err("trait register must refuse duplicates");
        match err {
            RegistryError::Duplicate(_) => {}
            other => panic!("expected RegistryError::Duplicate, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_write_leaves_no_temporaries_behind() {
        let dir = temp_dir("atomic");
        let path = dir.join("keys.json");
        let mut store = FileKeyStore::open(&path).expect("open fresh store");
        store
            .register(es256_record("subject-a", 1))
            .expect("first register");
        store
            .register(es256_record("subject-b", 2))
            .expect("second register");
        let mut entries: Vec<String> = std::fs::read_dir(&dir)
            .expect("list scratch dir")
            .map(|e| {
                e.expect("dir entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        entries.sort();
        assert_eq!(entries, vec!["keys.json".to_string()], "no .tmp-* remains");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn symlink_at_staging_path_refused_and_victim_never_followed() {
        let dir = temp_dir("symlink");
        let path = dir.join("keys.json");
        let store = FileKeyStore::open(&path).expect("open fresh store");
        let body = serde_json::to_string_pretty(&KeyStoreFile {
            format: STORE_FORMAT.to_string(),
            records: vec![],
            checksum: checksum_for(&[]).expect("empty checksum"),
        })
        .expect("serialize body");

        // Attack: a symlink planted at a staging path pointing at a victim
        // file. The stage open runs create_new(true), so the pre-placed name
        // is REFUSED (EEXIST) — never followed, never clobbered.
        let victim = dir.join("victim.json");
        std::fs::write(&victim, b"original victim bytes").expect("victim writes");
        let planted = dir.join(".keys.json.tmp-attack");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&victim, &planted).expect("symlink planted");
        let err = store
            .write_atomic_via(body.as_bytes(), &planted)
            .expect_err("planted symlink must be refused");
        assert!(matches!(err, StoreError::Io(_)), "{err:?}");
        assert_eq!(
            std::fs::read(&victim).expect("victim reads"),
            b"original victim bytes",
            "the symlink must never be followed"
        );

        // A DANGLING symlink is refused the same way; nothing materializes
        // through it.
        #[cfg(unix)]
        {
            let dangling = dir.join(".keys.json.tmp-dangling");
            std::os::unix::fs::symlink(dir.join("no-such-target"), &dangling)
                .expect("dangling symlink planted");
            assert!(store.write_atomic_via(body.as_bytes(), &dangling).is_err());
            assert!(!dir.join("no-such-target").exists());
        }

        // The honest path still lands, and the production staging name is
        // dot-prefixed and unique per attempt.
        let tmp = tmp_sibling(&path);
        store
            .write_atomic_via(body.as_bytes(), &tmp)
            .expect("honest staged write lands");
        assert!(path.is_file(), "the rename landed over the live path");
        let name = tmp
            .file_name()
            .expect("tmp has a file name")
            .to_string_lossy()
            .to_string();
        assert!(name.starts_with(".keys.json.tmp-"), "{name}");
        assert_ne!(
            tmp_sibling(&path),
            tmp,
            "staging name carries per-attempt entropy"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn checksum_is_deterministic_and_diverges() {
        let a = es256_record("subject-a", 1);
        let b = es256_record("subject-a", 2);
        let empty = checksum_for(&[]).expect("empty checksum");
        assert_eq!(empty, checksum_for(&[]).expect("empty checksum again"));
        let ca = checksum_for(std::slice::from_ref(&a)).expect("checksum a");
        assert_eq!(
            ca,
            checksum_for(std::slice::from_ref(&a)).expect("checksum a again")
        );
        assert_ne!(
            ca,
            checksum_for(std::slice::from_ref(&b)).expect("checksum b")
        );
        assert_ne!(
            checksum_for(&[a.clone(), b.clone()]).expect("checksum a+b"),
            checksum_for(&[b.clone(), a.clone()]).expect("checksum b+a"),
            "record order must move the checksum"
        );
        // Whole-file byte identity across independent stores.
        let dir = temp_dir("byte-identical");
        let p1 = dir.join("one.json");
        let p2 = dir.join("two.json");
        for path in [&p1, &p2] {
            let mut store = FileKeyStore::open(path).expect("open fresh store");
            store.register(b.clone()).expect("register");
        }
        assert_eq!(
            std::fs::read(&p1).expect("read one"),
            std::fs::read(&p2).expect("read two")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn created_epoch_beyond_double_precision_refused_canonical() {
        // JCS refuses integers beyond 2^53: the store refuses to checksum
        // (and therefore to admit) such a record — certify, never coerce.
        let dir = temp_dir("canonical");
        let path = dir.join("keys.json");
        let mut rec = es256_record("subject-a", 1);
        rec.created_epoch = 9_007_199_254_740_993; // 2^53 + 1
        let mut store = FileKeyStore::open(&path).expect("open fresh store");
        let err = store
            .register_checked(rec)
            .expect_err("uncanonicalizable record must be refused");
        assert!(matches!(err, StoreError::Canonical(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn trait_parity_with_in_memory_registry() {
        use crate::crypto_trust_keys::InMemoryKeyRegistry;
        let dir = temp_dir("parity");
        let path = dir.join("keys.json");
        let mut file_store = FileKeyStore::open(&path).expect("open fresh store");
        let mut memory = InMemoryKeyRegistry::new();
        let a = es256_record("subject-a", 1);
        let b = record("subject-a", AlgorithmId::MlDsa65, 2);
        let c = es256_record("subject-b", 3);
        for rec in [&a, &b, &c] {
            file_store.register(rec.clone()).expect("file register");
            memory.register(rec.clone()).expect("memory register");
            assert_eq!(file_store.len(), memory.len());
        }
        assert_eq!(file_store.lookup(&a.id), memory.lookup(&a.id));
        assert_eq!(file_store.contains(&c.id), memory.contains(&c.id));
        assert!(!file_store.contains(&KeyId("afk1_0000000000000000".to_string())));
        assert_eq!(
            file_store.by_custodian("subject-a").len(),
            memory.by_custodian("subject-a").len()
        );
        assert_eq!(file_store.by_custodian("subject-z").len(), 0);
        assert!(!file_store.is_empty());
        // Duplicate through the trait: identical refusal on both registries.
        let err_file = file_store.register(a.clone()).expect_err("file duplicate");
        let err_memory = memory.register(a).expect_err("memory duplicate");
        assert_eq!(err_file, err_memory);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn store_file_serde_round_trips() {
        let store_file = KeyStoreFile {
            format: STORE_FORMAT.to_string(),
            records: vec![es256_record("subject-a", 1)],
            checksum: checksum_for(&[es256_record("subject-a", 1)]).expect("checksum"),
        };
        let json = serde_json::to_string(&store_file).expect("serialize store file");
        let back: KeyStoreFile = serde_json::from_str(&json).expect("deserialize store file");
        assert_eq!(back, store_file);
    }
}
