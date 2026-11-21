//! Durable monoatomic AOD lease nullifier store (AFFI-SPEC-26.11.19 PR-5).
//!
//! Replaces the naive hex-lines `FileLeaseRegistry` with a fail-closed,
//! HMAC-BLAKE3-sealed snapshot aligned with the AFRN anti-replay pattern in
//! [`crate::replay_nullifier`].
//!
//! # On-disk format (`AFAL` v1)
//!
//! Sealed envelope (same outer shape as AFRN):
//! `tag(32) ++ body`
//!
//! Body (little-endian integers):
//! `magic("AFAL") | version(u16) | reserved(u16)
//!  | count(u64) | (lease_epoch(u64) ++ nullifier(32))*`
//!
//! Records are ordered by nullifier for deterministic serialization. The MAC
//! key lives beside the snapshot at `{path}.key` (32 bytes, mode 0600 on
//! unix). Atomic commit is temp-file + rename.
//!
//! # Fail-closed law
//!
//! * Missing snapshot → empty registry (lawful first run).
//! * Truncated / wrong magic / bad version / trailing bytes / MAC mismatch →
//!   typed refusal; never silent skip or empty re-admit.
//! * Re-burning a spent nullifier → [`AodRefusal::LeaseSpentOrExpired`]
//!   (`0xE50D`).
//! * Persist failure during burn rolls back the in-memory insert and refuses
//!   `0xE50D` so an admit is never claimed without durable storage.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use affidavit_core::{AodRefusal, LeaseRegistry};
use thiserror::Error;

/// Default relative path for the sealed AOD lease snapshot.
pub const DEFAULT_LEASE_REGISTRY_PATH: &str = ".affi/aod-leases";

/// Snapshot magic (`b"AFAL"` — Affidavit AOD Lease).
const SNAPSHOT_MAGIC: &[u8; 4] = b"AFAL";
/// Snapshot format version.
const SNAPSHOT_VERSION: u16 = 1;
/// Fixed body header: magic(4) + version(2) + reserved(2) + count(u64).
const SNAPSHOT_HEADER_LEN: usize = 4 + 2 + 2 + 8;
/// HMAC-BLAKE3 tag length prepended to a sealed snapshot.
const SNAPSHOT_MAC_LEN: usize = 32;
/// One burned record: lease_epoch(8) + nullifier(32).
const RECORD_LEN: usize = 8 + 32;

/// Typed persistence / decode refusals for the AOD lease store.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AodLeaseRegistryError {
    /// Snapshot or key file unreadable / unwritable / structurally invalid.
    #[error("REFUSED_LEASE_REGISTRY [0xE102 InvalidPersistenceStorage]: {0}")]
    InvalidPersistenceStorage(String),
    /// MAC mismatch or corrupt AFAL body.
    #[error("REFUSED_LEASE_REGISTRY [0xE102 CorruptLedger]: {0}")]
    CorruptLedger(String),
}

impl AodLeaseRegistryError {
    /// Machine code shared with the AFRN persistence band (`0xE102`).
    #[must_use]
    pub const fn machine_code(&self) -> u16 {
        0xE102
    }
}

/// Durable monoatomic lease registry backed by an AFAL sealed snapshot.
#[derive(Debug)]
pub struct AodLeaseRegistry {
    path: PathBuf,
    key_path: PathBuf,
    mac_key: [u8; 32],
    /// nullifier → lease_epoch that burned it.
    burned: BTreeMap<[u8; 32], u64>,
}

impl AodLeaseRegistry {
    /// Open (or create on first run) the sealed registry at `path`.
    ///
    /// The MAC key is loaded from `{path}.key`, created with OS entropy when
    /// absent (or a fixed compat key under `AFFI_NULLIFIER_DETERMINISTIC_KEY=1`).
    ///
    /// # Errors
    ///
    /// Fail-closed on unreadable / forged / truncated / corrupt snapshots.
    pub fn open(path: &Path) -> Result<Self, AodLeaseRegistryError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                        "cannot create lease registry parent {}: {e}",
                        parent.display()
                    ))
                })?;
            }
        }

        let key_path = key_path_for(path);
        let mac_key = load_or_create_mac_key(&key_path)?;

        let burned = if path.exists() {
            let bytes = std::fs::read(path).map_err(|e| {
                AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                    "lease registry unreadable {}: {e}",
                    path.display()
                ))
            })?;
            unseal(&bytes, &mac_key)?
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            path: path.to_path_buf(),
            key_path,
            mac_key,
            burned,
        })
    }

    /// Path of the sealed snapshot file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Path of the MAC key file.
    #[must_use]
    pub fn key_path(&self) -> &Path {
        &self.key_path
    }

    /// Number of burned nullifiers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.burned.len()
    }

    /// True when no nullifiers have been burned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.burned.is_empty()
    }

    /// Whether `nullifier` has already been burned.
    #[must_use]
    pub fn contains(&self, nullifier: &[u8; 32]) -> bool {
        self.burned.contains_key(nullifier)
    }

    /// Persist the sealed snapshot atomically (temp + rename).
    ///
    /// # Errors
    ///
    /// Surfaces write / rename failures as typed persistence refusals.
    pub fn persist(&self) -> Result<(), AodLeaseRegistryError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                        "cannot create lease registry parent {}: {e}",
                        parent.display()
                    ))
                })?;
            }
        }
        let sealed = seal(&self.burned, &self.mac_key);
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, &sealed).map_err(|e| {
            AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                "lease registry write failed {}: {e}",
                tmp.display()
            ))
        })?;
        std::fs::rename(&tmp, &self.path).map_err(|e| {
            AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                "lease registry commit failed {}: {e}",
                self.path.display()
            ))
        })?;
        Ok(())
    }
}

impl LeaseRegistry for AodLeaseRegistry {
    fn burn(
        &mut self,
        lease_epoch: u64,
        nullifier: &[u8; 32],
    ) -> core::result::Result<(), AodRefusal> {
        if self.burned.contains_key(nullifier) {
            return Err(AodRefusal::LeaseSpentOrExpired);
        }
        self.burned.insert(*nullifier, lease_epoch);
        // Durable monoatomic admit: persist before returning Ok. On failure,
        // roll back so in-memory state matches disk and refuse closed.
        if let Err(_e) = self.persist() {
            self.burned.remove(nullifier);
            return Err(AodRefusal::LeaseSpentOrExpired);
        }
        Ok(())
    }
}

fn key_path_for(snapshot: &Path) -> PathBuf {
    let mut s = snapshot.as_os_str().to_owned();
    s.push(".key");
    PathBuf::from(s)
}

fn snapshot_mac(mac_key: &[u8; 32], body: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_keyed(mac_key);
    hasher.update(body);
    *hasher.finalize().as_bytes()
}

fn tags_equal(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn to_bytes(burned: &BTreeMap<[u8; 32], u64>) -> Vec<u8> {
    let mut out = Vec::with_capacity(SNAPSHOT_HEADER_LEN + burned.len() * RECORD_LEN);
    out.extend_from_slice(SNAPSHOT_MAGIC);
    out.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(burned.len() as u64).to_le_bytes());
    for (nullifier, epoch) in burned {
        out.extend_from_slice(&epoch.to_le_bytes());
        out.extend_from_slice(nullifier);
    }
    out
}

fn seal(burned: &BTreeMap<[u8; 32], u64>, mac_key: &[u8; 32]) -> Vec<u8> {
    let body = to_bytes(burned);
    let mut out = Vec::with_capacity(SNAPSHOT_MAC_LEN + body.len());
    out.extend_from_slice(&snapshot_mac(mac_key, &body));
    out.extend_from_slice(&body);
    out
}

fn unseal(
    bytes: &[u8],
    mac_key: &[u8; 32],
) -> Result<BTreeMap<[u8; 32], u64>, AodLeaseRegistryError> {
    if bytes.len() < SNAPSHOT_MAC_LEN + SNAPSHOT_HEADER_LEN {
        return Err(AodLeaseRegistryError::InvalidPersistenceStorage(format!(
            "sealed AFAL truncated: {} bytes < tag {SNAPSHOT_MAC_LEN} + header {SNAPSHOT_HEADER_LEN}",
            bytes.len()
        )));
    }
    let (tag_bytes, body) = bytes.split_at(SNAPSHOT_MAC_LEN);
    let mut tag = [0u8; 32];
    tag.copy_from_slice(tag_bytes);
    let expected = snapshot_mac(mac_key, body);
    if !tags_equal(&expected, &tag) {
        return Err(AodLeaseRegistryError::CorruptLedger(
            "AFAL authentication failed (HMAC-BLAKE3 tag mismatch: forged or truncated)".into(),
        ));
    }
    load_body(body)
}

fn load_body(bytes: &[u8]) -> Result<BTreeMap<[u8; 32], u64>, AodLeaseRegistryError> {
    if bytes.len() < SNAPSHOT_HEADER_LEN {
        return Err(AodLeaseRegistryError::InvalidPersistenceStorage(format!(
            "AFAL body truncated: {} bytes < header {SNAPSHOT_HEADER_LEN}",
            bytes.len()
        )));
    }
    if &bytes[0..4] != SNAPSHOT_MAGIC {
        return Err(AodLeaseRegistryError::CorruptLedger(
            "AFAL magic mismatch (expected AFAL)".into(),
        ));
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().expect("header bounds checked"));
    if version != SNAPSHOT_VERSION {
        return Err(AodLeaseRegistryError::CorruptLedger(format!(
            "unsupported AFAL version {version}"
        )));
    }
    let count = u64::from_le_bytes(bytes[8..16].try_into().expect("header bounds checked"));
    let need = (count as usize)
        .checked_mul(RECORD_LEN)
        .and_then(|n| n.checked_add(SNAPSHOT_HEADER_LEN))
        .ok_or_else(|| {
            AodLeaseRegistryError::InvalidPersistenceStorage("AFAL count overflow".into())
        })?;
    if need != bytes.len() {
        return Err(AodLeaseRegistryError::InvalidPersistenceStorage(format!(
            "AFAL length mismatch: header claims {count} records → {need} bytes, got {}",
            bytes.len()
        )));
    }
    if count > (1 << 40) {
        return Err(AodLeaseRegistryError::InvalidPersistenceStorage(format!(
            "implausible AFAL count {count}"
        )));
    }

    let mut burned = BTreeMap::new();
    let mut off = SNAPSHOT_HEADER_LEN;
    for _ in 0..count {
        let epoch = u64::from_le_bytes(bytes[off..off + 8].try_into().expect("bounds checked"));
        off += 8;
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&bytes[off..off + 32]);
        off += 32;
        if burned.insert(nullifier, epoch).is_some() {
            return Err(AodLeaseRegistryError::CorruptLedger(
                "AFAL duplicate nullifier record".into(),
            ));
        }
    }
    debug_assert_eq!(off, bytes.len());
    Ok(burned)
}

fn load_or_create_mac_key(key_path: &Path) -> Result<[u8; 32], AodLeaseRegistryError> {
    let deterministic =
        std::env::var_os("AFFI_NULLIFIER_DETERMINISTIC_KEY").is_some_and(|v| v == "1");
    if deterministic {
        return Ok([0xADu8; 32]);
    }

    if key_path.exists() {
        let bytes = std::fs::read(key_path).map_err(|e| {
            AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                "lease MAC key unreadable {}: {e}",
                key_path.display()
            ))
        })?;
        if bytes.len() != 32 {
            return Err(AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                "lease MAC key malformed {}: expected 32 bytes, got {}",
                key_path.display(),
                bytes.len()
            )));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        return Ok(key);
    }

    if let Some(parent) = key_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                    "cannot create lease key parent {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }

    #[cfg(feature = "crypto-trust")]
    let key: [u8; 32] = {
        use rand_core::{OsRng, RngCore};
        let mut k = [0u8; 32];
        OsRng.fill_bytes(&mut k);
        k
    };
    #[cfg(not(feature = "crypto-trust"))]
    let key: [u8; 32] = blake3::hash(b"affidavit/aod-lease-mac-key/no-crypto-trust").into();

    #[cfg(unix)]
    {
        let mut f = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(key_path)
            .map_err(|e| {
                AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                    "lease MAC key unwritable {}: {e}",
                    key_path.display()
                ))
            })?;
        f.write_all(&key).map_err(|e| {
            AodLeaseRegistryError::InvalidPersistenceStorage(format!(
                "lease MAC key write failed {}: {e}",
                key_path.display()
            ))
        })?;
    }
    #[cfg(not(unix))]
    std::fs::write(key_path, &key).map_err(|e| {
        AodLeaseRegistryError::InvalidPersistenceStorage(format!(
            "lease MAC key write failed {}: {e}",
            key_path.display()
        ))
    })?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn n(tag: u8) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[0] = tag;
        out
    }

    #[test]
    fn burn_persist_reopen_refuses_replay() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("aod-leases");
        let mut a = AodLeaseRegistry::open(&path).unwrap();
        assert!(a.burn(1, &n(0x11)).is_ok());
        assert_eq!(a.len(), 1);
        drop(a);

        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > SNAPSHOT_MAC_LEN + 4);
        assert_eq!(&bytes[SNAPSHOT_MAC_LEN..SNAPSHOT_MAC_LEN + 4], b"AFAL");

        let mut b = AodLeaseRegistry::open(&path).unwrap();
        assert!(b.contains(&n(0x11)));
        let err = b.burn(1, &n(0x11)).unwrap_err();
        assert_eq!(err, AodRefusal::LeaseSpentOrExpired);
        assert_eq!(err.machine_code(), 0xE50D);
    }

    #[test]
    fn forged_mac_fails_closed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("aod-leases");
        let mut a = AodLeaseRegistry::open(&path).unwrap();
        a.burn(1, &n(0x22)).unwrap();
        drop(a);

        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] ^= 0xff;
        std::fs::write(&path, &bytes).unwrap();

        let err = AodLeaseRegistry::open(&path).unwrap_err();
        assert!(matches!(err, AodLeaseRegistryError::CorruptLedger(_)));
        assert_eq!(err.machine_code(), 0xE102);
    }

    #[test]
    fn hex_lines_legacy_fails_closed() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("aod-leases");
        // Create key so open reaches body decode rather than key errors.
        let key_path = key_path_for(&path);
        std::fs::write(&key_path, [0xABu8; 32]).unwrap();
        std::fs::write(&path, b"deadbeef\n").unwrap();
        let err = AodLeaseRegistry::open(&path).unwrap_err();
        assert!(
            matches!(
                err,
                AodLeaseRegistryError::InvalidPersistenceStorage(_)
                    | AodLeaseRegistryError::CorruptLedger(_)
            ),
            "err={err}"
        );
    }

    #[test]
    fn seal_round_trip_empty() {
        let key = [0x5Eu8; 32];
        let empty = BTreeMap::new();
        let sealed = seal(&empty, &key);
        let restored = unseal(&sealed, &key).unwrap();
        assert!(restored.is_empty());
    }
}
