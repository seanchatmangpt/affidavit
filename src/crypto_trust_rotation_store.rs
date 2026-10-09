//! Durable append-only store for key-rotation records (CTP-ROTSTORE-v1) of
//! the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! Store law: every rotation ceremony
//! ([`crate::crypto_trust_rotation::RotationRecord`]) persists as ONE JCS
//! (RFC 8785) canonical JSON line at the graph's store file. The record
//! alone is not enough to re-verify a takeover — it carries key IDS, not the
//! successor public keys — so each line is a format-stamped envelope
//! `{format, record, successor_public_es256, successor_public_mldsa65}`:
//! the successor's hybrid public material travels BESIDE the record it
//! signed, and every load re-runs the rotation module's
//! [`crate::crypto_trust_rotation::verify_rotation`] per record against that
//! material before trusting the line.
//!
//! Append law (teeth-first): [`RotationStore::append`] verifies the record
//! against the supplied successor material BEFORE any byte is written — an
//! unverifiable record is refused as [`RotationStoreError::Tampered`] (or
//! its provider/serialization refusals) and the store is left byte-identical.
//! Writing itself is a single append of one canonical line followed by
//! `sync_data`: the record is durable the moment the call returns, and the
//! store never rewrites or reorders earlier lines (append-only).
//!
//! Tamper law: [`RotationStore::load`] fails closed. An absent store is an
//! [`RotationStoreError::Io`] refusal (a missing history is NOT an empty
//! history — without verified records the caller knows nothing about
//! takeovers). Every line is parsed, format-stamp-checked, and re-verified;
//! the first bad line stops the load as a typed refusal naming its 1-based
//! line number ([`RotationStoreError::Tampered`] for a record whose signed
//! digest or successor identity no longer holds, [`WrongFormat`] for a
//! foreign stamp, [`Serialization`] for unparsable bytes, [`Verification`]
//! for a structurally broken signature).
//!
//! Query law: the latest appended record per key id wins (append order is
//! ceremony order). [`RotationStore::latest_for_successor`] answers "what is
//! the newest record attesting THIS key's takeover" (matches
//! `new_key_id`); [`RotationStore::latest_for_predecessor`] answers "what
//! replaced THIS key" (matches `old_key_id`).
//!
//! Cross-process law: the store handle is stateless — every operation
//! re-reads the file, so the file is the only truth and two handles (two
//! processes) never diverge. Determinism law: JCS lines + deterministic
//! ceremonies (RFC 6979 ES256, domain-bound ML-DSA rnd) mean equal ceremony
//! sequences render byte-identical stores.
//!
//! House law: refusal-as-value (typed [`RotationStoreError`], never panics);
//! every variant witnessed by tests; teeth-before-tamper (the honest
//! append/load round trip is proven before each tamper falsifier); no
//! unwrap/expect outside tests.

// Consumed query columns (rotation-store.rq): store_file, domain_tag.

use crate::crypto_trust_canonical::jcs;
use crate::crypto_trust_rotation::{verify_rotation, RotationError, RotationRecord};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Rotation-store file location (ctp:rotation-store-v1
/// ctp:rotationStoreFile), relative to the consumer repository root. The
/// default of every `Option<&Path>` on this surface.
pub const ROTATION_STORE_FILE: &str = ".affi/rotations.jsonl";

/// Trust-plane domain tag (ctp:policy-v1 ctp:domainTag). Carried so the
/// store surface states the plane it serves; conformance-tested equal to the
/// rotation and canonicalization modules' tags at render time.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Store wire-format identity stamped on every persisted line;
/// [`RotationStore::load`] refuses any other value.
pub const STORE_FORMAT: &str = "CTP-ROTSTORE-v1";

/// One persisted line: the signed rotation record plus the successor's
/// hybrid public material it must verify under. The envelope is the
/// serialization unit of the store; construction goes through
/// [`RotationStore::append`] (which verifies first), never directly.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RotationStoreEnvelope {
    /// Wire-format stamp ([`STORE_FORMAT`]); a foreign value refuses the line.
    pub format: String,
    /// The rotation ceremony's record (signature included).
    pub record: RotationRecord,
    /// Successor ES256 SEC1 point (classical half of the hybrid key).
    pub successor_public_es256: Vec<u8>,
    /// Successor raw ML-DSA-65 encoded public key (PQ half).
    pub successor_public_mldsa65: Vec<u8>,
}

/// A store line that has PASSED per-record verification on load: the
/// 1-based line it came from, the record, and the successor material it
/// verified under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedRotation {
    /// 1-based line number in the store file.
    pub line: u64,
    /// The verified rotation record.
    pub record: RotationRecord,
    /// The successor ES256 SEC1 point the record verified under.
    pub successor_public_es256: Vec<u8>,
    /// The successor ML-DSA-65 public key the record verified under.
    pub successor_public_mldsa65: Vec<u8>,
}

/// Typed refusals of the rotation store. Values, never panics. Line numbers
/// are 1-based and name the exact offending line of the store file.
#[derive(Debug, thiserror::Error)]
pub enum RotationStoreError {
    /// The store location is unreadable or unwritable. Load refuses an
    /// ABSENT store here too: a missing history is not an empty history.
    #[error("rotation store io: {0}")]
    Io(String),
    /// The line is not a valid store envelope (unparsable JSON, missing
    /// fields, or content that cannot canonicalize).
    #[error("rotation store line {line}: not a valid store envelope: {message}")]
    Serialization { line: u64, message: String },
    /// The line's format stamp is not [`STORE_FORMAT`].
    #[error("rotation store line {line}: foreign format stamp {found:?}: expected {STORE_FORMAT}")]
    WrongFormat { line: u64, found: String },
    /// The line's record no longer verifies: its signed digest does not
    /// match its content, or its `new_key_id` does not match the successor
    /// public material stored beside it. Either way the line was tampered
    /// with or is not the record it claims to be.
    #[error("rotation store line {line}: rotation record tampered or successor material mismatch (kid {kid})")]
    Tampered { line: u64, kid: String },
    /// The line's record cannot be verified at all (structurally broken
    /// signature the provider refuses, or a canonicalization refusal).
    #[error("rotation store line {line}: record verification failed: {message}")]
    Verification { line: u64, message: String },
}

/// Maps the rotation module's refusals onto the store's line-named
/// vocabulary (the single honest wiring, used by append and load alike).
fn map_verify_err(err: RotationError, line: u64, kid: &str) -> RotationStoreError {
    match err {
        RotationError::RecordTampered | RotationError::SuccessorSignatureInvalid => {
            RotationStoreError::Tampered {
                line,
                kid: kid.to_string(),
            }
        }
        RotationError::Serialization(message) => {
            RotationStoreError::Serialization { line, message }
        }
        other => RotationStoreError::Verification {
            line,
            message: other.to_string(),
        },
    }
}

/// Handle on the durable rotation store. Stateless by law: the FILE is the
/// only truth, every operation re-reads it, so two handles in two processes
/// can never diverge. `Default` resolves the graph const
/// [`ROTATION_STORE_FILE`].
#[derive(Debug, Clone)]
pub struct RotationStore {
    path: PathBuf,
}

impl Default for RotationStore {
    fn default() -> Self {
        RotationStore {
            path: PathBuf::from(ROTATION_STORE_FILE),
        }
    }
}

impl RotationStore {
    /// A handle on the store at `path`.
    pub fn new(path: impl Into<PathBuf>) -> RotationStore {
        RotationStore { path: path.into() }
    }

    /// The store file this handle names.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Appends one rotation ceremony to the store. Order of judgment:
    ///
    /// 1. Teeth FIRST: the record must verify under the supplied successor
    ///    material ([`verify_rotation`]) — a takeover that cannot be
    ///    re-proven is never persisted. Refusal is typed and names the line
    ///    the record WOULD have occupied; the store stays byte-identical.
    /// 2. Canonicalize: the envelope (`format` stamp + record + successor
    ///    material) is serialized to JCS canonical JSON — one line, no
    ///    pretty-print drift.
    /// 3. Append + `sync_data`: the line is durable when this returns.
    ///
    /// Returns the 1-based line number the record now occupies.
    pub fn append(
        &self,
        record: &RotationRecord,
        successor_public_es256: &[u8],
        successor_public_mldsa65: &[u8],
    ) -> Result<u64, RotationStoreError> {
        let line = self.next_line_number()?;
        match verify_rotation(record, successor_public_es256, successor_public_mldsa65) {
            Ok(true) => {}
            Ok(false) => {
                return Err(RotationStoreError::Verification {
                    line,
                    message: String::from(
                        "verify_rotation reported false for the supplied material",
                    ),
                })
            }
            Err(err) => return Err(map_verify_err(err, line, &record.new_key_id)),
        }
        let envelope = RotationStoreEnvelope {
            format: String::from(STORE_FORMAT),
            record: record.clone(),
            successor_public_es256: successor_public_es256.to_vec(),
            successor_public_mldsa65: successor_public_mldsa65.to_vec(),
        };
        let value =
            serde_json::to_value(&envelope).map_err(|e| RotationStoreError::Serialization {
                line,
                message: e.to_string(),
            })?;
        let canonical = jcs(&value).map_err(|e| RotationStoreError::Serialization {
            line,
            message: e.to_string(),
        })?;
        self.append_line(canonical.as_bytes())?;
        Ok(line)
    }

    /// Loads and re-verifies EVERY line of the store, in append order.
    /// Fail-closed: the first bad line refuses the whole load as a typed
    /// [`RotationStoreError`] naming that 1-based line — a history with one
    /// unprovable takeover is not partially trusted. An absent store is an
    /// [`RotationStoreError::Io`] refusal (a missing history is not an
    /// empty one).
    pub fn load(&self) -> Result<Vec<VerifiedRotation>, RotationStoreError> {
        let text = std::fs::read_to_string(&self.path)
            .map_err(|e| RotationStoreError::Io(format!("read {}: {e}", self.path.display())))?;
        let mut verified = Vec::new();
        for (index, line_text) in text.lines().enumerate() {
            let line = index as u64 + 1;
            let envelope: RotationStoreEnvelope =
                serde_json::from_str(line_text).map_err(|e| RotationStoreError::Serialization {
                    line,
                    message: e.to_string(),
                })?;
            if envelope.format != STORE_FORMAT {
                return Err(RotationStoreError::WrongFormat {
                    line,
                    found: envelope.format,
                });
            }
            match verify_rotation(
                &envelope.record,
                &envelope.successor_public_es256,
                &envelope.successor_public_mldsa65,
            ) {
                Ok(true) => verified.push(VerifiedRotation {
                    line,
                    record: envelope.record,
                    successor_public_es256: envelope.successor_public_es256,
                    successor_public_mldsa65: envelope.successor_public_mldsa65,
                }),
                Ok(false) => {
                    return Err(RotationStoreError::Verification {
                        line,
                        message: String::from(
                            "verify_rotation reported false for the stored material",
                        ),
                    })
                }
                Err(err) => return Err(map_verify_err(err, line, &envelope.record.new_key_id)),
            }
        }
        Ok(verified)
    }

    /// The newest record attesting the takeover OF `kid` (matches
    /// `new_key_id`; the store carries that key's successor material in the
    /// same line). `Ok(None)` when the store holds no such record.
    pub fn latest_for_successor(
        &self,
        kid: &str,
    ) -> Result<Option<VerifiedRotation>, RotationStoreError> {
        let records = self.load()?;
        Ok(records
            .into_iter()
            .rev()
            .find(|v| v.record.new_key_id == kid))
    }

    /// The newest record attesting what replaced `kid` (matches
    /// `old_key_id`). `Ok(None)` when the store holds no such record.
    pub fn latest_for_predecessor(
        &self,
        kid: &str,
    ) -> Result<Option<VerifiedRotation>, RotationStoreError> {
        let records = self.load()?;
        Ok(records
            .into_iter()
            .rev()
            .find(|v| v.record.old_key_id == kid))
    }

    /// 1-based line number the NEXT append would occupy (0 lines -> 1).
    /// Evidence for append refusals; a real IO problem surfaces again (and
    /// typed) at the write itself.
    fn next_line_number(&self) -> Result<u64, RotationStoreError> {
        let lines = std::fs::read_to_string(&self.path)
            .map(|text| text.lines().count() as u64)
            .unwrap_or(0);
        Ok(lines + 1)
    }

    /// The single append transition: create parent directories, open in
    /// append mode, write the canonical line + newline, `sync_data`.
    fn append_line(&self, body: &[u8]) -> Result<(), RotationStoreError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    RotationStoreError::Io(format!("create dir {}: {e}", parent.display()))
                })?;
            }
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| RotationStoreError::Io(format!("open {}: {e}", self.path.display())))?;
        file.write_all(body)
            .map_err(|e| RotationStoreError::Io(format!("write {}: {e}", self.path.display())))?;
        file.write_all(b"\n").map_err(|e| {
            RotationStoreError::Io(format!("write newline {}: {e}", self.path.display()))
        })?;
        file.sync_data()
            .map_err(|e| RotationStoreError::Io(format!("sync {}: {e}", self.path.display())))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DOMAIN_TAG as CANONICAL_DOMAIN_TAG;
    use crate::crypto_trust_es256::Es256SigningKey;
    use crate::crypto_trust_keys::KeyId;
    use crate::crypto_trust_pqc::{ml_dsa65_from_seed, HybridSecret, ML_DSA_65_SIGNATURE_LEN};
    use crate::crypto_trust_rotation::rotate_es256_to_hybrid;
    use std::sync::atomic::{AtomicU64, Ordering};

    const OLD_SEED: [u8; 32] = [0x11u8; 32];
    const OLD_SEED_2: [u8; 32] = [0x22u8; 32];
    const OLD_SEED_3: [u8; 32] = [0x33u8; 32];
    const HYBRID_ES_SEED: [u8; 32] = [0x42u8; 32];
    const HYBRID_ML_SEED: [u8; 32] = [0xA5u8; 32];
    const OTHER_HYBRID_ES_SEED: [u8; 32] = [0x77u8; 32];
    const OTHER_HYBRID_ML_SEED: [u8; 32] = [0x99u8; 32];
    const AT: u64 = 1_700_000_000;
    const XPROC_AT: u64 = 1_800_000_000;
    const XPROC_ENV: &str = "CTP_ROTSTORE_XPROC_STORE";

    /// Unique scratch directory per call (no tempfile dependency; std only).
    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("ctp-rotation-store-{tag}-{pid}-{n}"));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    fn old_key(seed: &[u8; 32]) -> Es256SigningKey {
        Es256SigningKey::from_seed(seed).expect("valid p256 scalar")
    }

    fn hybrid_secret() -> crate::crypto_trust_pqc::HybridSecret {
        crate::crypto_trust_pqc::HybridSecret {
            es256: Es256SigningKey::from_seed(&HYBRID_ES_SEED).expect("valid p256 scalar"),
            mldsa65_seed: HYBRID_ML_SEED,
        }
    }

    fn successor_pks(secret: &crate::crypto_trust_pqc::HybridSecret) -> (Vec<u8>, Vec<u8>) {
        (
            secret.es256.public_key_sec1(),
            ml_dsa65_from_seed(&secret.mldsa65_seed).public,
        )
    }

    /// One honest ceremony: `old_seed`'s CLASSICAL key rotates to THE hybrid
    /// successor at `at`.
    fn ceremony(old_seed: &[u8; 32], at: u64) -> RotationRecord {
        rotate_es256_to_hybrid(&old_key(old_seed), &hybrid_secret(), at).expect("admitted rotation")
    }

    /// A SECOND, distinct hybrid successor (own ES256 + ML-DSA-65 seeds) so
    /// tests can exercise successor-identity refusals honestly.
    fn other_hybrid_secret() -> HybridSecret {
        HybridSecret {
            es256: Es256SigningKey::from_seed(&OTHER_HYBRID_ES_SEED).expect("valid p256 scalar"),
            mldsa65_seed: OTHER_HYBRID_ML_SEED,
        }
    }

    /// One honest ceremony under the GIVEN successor secret.
    fn ceremony_under(secret: &HybridSecret, old_seed: &[u8; 32], at: u64) -> RotationRecord {
        rotate_es256_to_hybrid(&old_key(old_seed), secret, at).expect("admitted rotation")
    }

    fn store_in(dir: &Path) -> RotationStore {
        RotationStore::new(dir.join("rotations.jsonl"))
    }

    fn file_text(path: &Path) -> String {
        std::fs::read_to_string(path).expect("read store file")
    }

    /// Rewrites line `line_no` (1-based) by applying `mutate` to its parsed
    /// JSON value. The rewritten file stays one-object-per-line JSON.
    fn rewrite_line(path: &Path, line_no: u64, mutate: impl FnOnce(&mut serde_json::Value)) {
        let text = file_text(path);
        let mut lines: Vec<String> = text.lines().map(std::string::ToString::to_string).collect();
        let mut value: serde_json::Value =
            serde_json::from_str(&lines[(line_no - 1) as usize]).expect("parse line");
        mutate(&mut value);
        lines[(line_no - 1) as usize] = serde_json::to_string(&value).expect("serialize line");
        std::fs::write(path, lines.join("\n") + "\n").expect("rewrite store file");
    }

    // -- teeth first: ceremony -> append -> load -> re-verify -----------------

    #[test]
    fn append_then_load_round_trips_and_reverifies_every_record() {
        let dir = temp_dir("roundtrip");
        let store = store_in(&dir);
        let record = ceremony(&OLD_SEED, AT);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());

        let line = store
            .append(&record, &es_pk, &ml_pk)
            .expect("honest record appends");
        assert_eq!(line, 1, "first append occupies line 1");
        assert!(
            store.path().is_file(),
            "store file created at {:?}",
            store.path()
        );

        let records = store.load().expect("store loads");
        assert_eq!(records.len(), 1);
        let got = &records[0];
        assert_eq!(got.line, 1);
        assert_eq!(got.record, record);
        assert_eq!(got.successor_public_es256, es_pk);
        assert_eq!(got.successor_public_mldsa65, ml_pk);
        // The loaded record still re-verifies under its carried material.
        assert!(verify_rotation(&got.record, &es_pk, &ml_pk).expect("re-verify"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn append_refuses_to_persist_an_unverifiable_record() {
        let dir = temp_dir("teeth-gate");
        let store = store_in(&dir);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());

        // (a) Tampered record (flipped timestamp) + the TRUE successor
        // material: the signature no longer covers the content.
        let mut tampered = ceremony(&OLD_SEED, AT);
        tampered.rotated_at ^= 0x01;
        let err = store
            .append(&tampered, &es_pk, &ml_pk)
            .expect_err("tampered record must never be persisted");
        match err {
            RotationStoreError::Tampered { line, kid } => {
                assert_eq!(line, 1);
                assert_eq!(kid, tampered.new_key_id);
            }
            other => panic!("expected Tampered, got {other:?}"),
        }
        assert!(
            !store.path().exists(),
            "a refused append must not create the store"
        );

        // (b) Honest record + the WRONG successor material: the identity
        // digest refuses before any signature work (not this record's keys).
        let honest = ceremony(&OLD_SEED, AT);
        let wrong_es = Es256SigningKey::from_seed(&OTHER_HYBRID_ES_SEED)
            .expect("valid p256 scalar")
            .public_key_sec1();
        let wrong_ml = ml_dsa65_from_seed(&OTHER_HYBRID_ML_SEED).public;
        let err = store
            .append(&honest, &wrong_es, &wrong_ml)
            .expect_err("record under foreign successor material must be refused");
        assert!(
            matches!(err, RotationStoreError::Tampered { line: 1, .. }),
            "got {err:?}"
        );

        // (c) Truncated successor signature: undecodable wire form.
        let mut truncated = ceremony(&OLD_SEED, AT);
        truncated
            .successor_signature
            .truncate(ML_DSA_65_SIGNATURE_LEN - 1);
        let err = store
            .append(&truncated, &es_pk, &ml_pk)
            .expect_err("undecodable signature must be refused");
        assert!(
            matches!(err, RotationStoreError::Tampered { .. }),
            "got {err:?}"
        );

        assert!(!store.path().exists(), "nothing was persisted");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn append_refusal_names_the_line_it_would_have_occupied() {
        let dir = temp_dir("line-number");
        let store = store_in(&dir);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        store
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("first append");
        store
            .append(&ceremony(&OLD_SEED_2, AT + 1), &es_pk, &ml_pk)
            .expect("second append");
        let before = file_text(store.path());

        let mut tampered = ceremony(&OLD_SEED_3, AT + 2);
        tampered.old_key_id = String::from("afk1_forgedhistory0000");
        let err = store
            .append(&tampered, &es_pk, &ml_pk)
            .expect_err("forged record refused");
        match err {
            RotationStoreError::Tampered { line, .. } => assert_eq!(line, 3, "would-be line"),
            other => panic!("expected Tampered, got {other:?}"),
        }
        assert_eq!(
            file_text(store.path()),
            before,
            "refused append leaves the store byte-identical"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- determinism: same ceremonies -> byte-identical stores ----------------

    #[test]
    fn same_ceremonies_render_byte_identical_stores() {
        let dir = temp_dir("determinism");
        let one = RotationStore::new(dir.join("one.jsonl"));
        let two = RotationStore::new(dir.join("two.jsonl"));
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        for store in [&one, &two] {
            store
                .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
                .expect("append one");
            store
                .append(&ceremony(&OLD_SEED_2, AT + 1), &es_pk, &ml_pk)
                .expect("append two");
        }
        assert_eq!(
            std::fs::read(one.path()).expect("read one"),
            std::fs::read(two.path()).expect("read two"),
            "deterministic ceremonies + JCS lines = byte-identical stores"
        );
        // And the bytes move once any input moves.
        let three = RotationStore::new(dir.join("three.jsonl"));
        three
            .append(&ceremony(&OLD_SEED, AT + 1), &es_pk, &ml_pk)
            .expect("append with a different instant");
        assert_ne!(
            std::fs::read(one.path()).expect("read one"),
            std::fs::read(three.path()).expect("read three")
        );
        // Every persisted line is exactly its own JCS canonicalization.
        for line in file_text(one.path()).lines() {
            let value: serde_json::Value = serde_json::from_str(line).expect("parse line");
            assert_eq!(jcs(&value).expect("recanonicalize"), line);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- latest-record queries per kid ----------------------------------------

    #[test]
    fn latest_queries_win_by_append_order_for_both_key_roles() {
        let dir = temp_dir("latest");
        let store = store_in(&dir);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        // Three ceremonies, ONE successor: old1 -> succ, old2 -> succ,
        // old3 -> succ. All share new_key_id.
        let r1 = ceremony(&OLD_SEED, AT);
        let r2 = ceremony(&OLD_SEED_2, AT + 1);
        let r3 = ceremony(&OLD_SEED_3, AT + 2);
        assert_eq!(r1.new_key_id, r2.new_key_id);
        assert_eq!(r2.new_key_id, r3.new_key_id);
        assert_ne!(r1.old_key_id, r2.old_key_id);
        for record in [&r1, &r2, &r3] {
            store.append(record, &es_pk, &ml_pk).expect("append");
        }

        // Successor-keyed: the newest takeover of THE successor kid wins.
        let latest = store
            .latest_for_successor(&r3.new_key_id)
            .expect("load")
            .expect("successor kid present");
        assert_eq!(latest.line, 3);
        assert_eq!(latest.record, r3);

        // Predecessor-keyed: what replaced old2.
        let latest = store
            .latest_for_predecessor(&r2.old_key_id)
            .expect("load")
            .expect("predecessor kid present");
        assert_eq!(latest.record, r2);

        // Unknown kids answer None, not an error.
        assert!(store
            .latest_for_successor("afk1_missingkey000000")
            .expect("load")
            .is_none());
        assert!(store
            .latest_for_predecessor("afk1_missingkey000000")
            .expect("load")
            .is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- cross-process reopen (the point of a durable store) ------------------

    #[test]
    fn cross_process_reopen_carries_records_and_appends_forward() {
        let dir = temp_dir("xproc");
        let path = dir.join("rotations.jsonl");
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());

        // Process A: one ceremony, then the handle is dropped — only the
        // file carries the takeover evidence.
        {
            let a = RotationStore::new(&path);
            a.append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
                .expect("process A appends");
        }

        // Process B: fresh handle, empty memory; it sees A's record, and
        // its own append lands AFTER A's (append order preserved).
        {
            let b = RotationStore::new(&path);
            let records = b.load().expect("process B reopens from disk");
            assert_eq!(records.len(), 1, "process B sees process A's record");
            b.append(&ceremony(&OLD_SEED_2, AT + 1), &es_pk, &ml_pk)
                .expect("process B appends");
        }

        // Process C: fresh handle; both records verify from disk alone and
        // the latest-query answers across the process boundary.
        let c = RotationStore::new(&path);
        let records = c.load().expect("process C reopens from disk");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].record.rotated_at, AT);
        assert_eq!(records[1].record.rotated_at, AT + 1);
        assert_eq!(
            c.latest_for_predecessor(&records[0].record.old_key_id)
                .expect("load")
                .expect("present")
                .line,
            1
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The child half of [`second_os_process_appends_are_witnessed`].
    /// `#[ignore]`d so ordinary runs skip it; the parent test invokes it in
    /// a REAL second operating-system process through the test binary.
    #[test]
    #[ignore]
    fn xproc_child_appends_from_a_second_os_process() {
        let path = std::env::var(XPROC_ENV).expect("child run requires the store path env");
        let store = RotationStore::new(path);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        let line = store
            .append(&ceremony(&OLD_SEED_2, XPROC_AT), &es_pk, &ml_pk)
            .expect("child process appends");
        assert!(line >= 1);
    }

    #[test]
    fn second_os_process_appends_are_witnessed() {
        let dir = temp_dir("osproc");
        let path = dir.join("rotations.jsonl");
        // This process seeds one record first.
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        RotationStore::new(&path)
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("parent appends");

        // A REAL second OS process runs the child test against the same
        // store file and appends its own ceremony.
        let exe = std::env::current_exe().expect("locate test binary");
        let output = std::process::Command::new(exe)
            .args([
                "crypto_trust_rotation_store::tests::xproc_child_appends_from_a_second_os_process",
                "--exact",
                "--ignored",
                "--nocapture",
            ])
            .env(XPROC_ENV, &path)
            .output()
            .expect("spawn second os process");
        assert!(
            output.status.success(),
            "second-process child run failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        // This process reopens and sees BOTH takeovers, both re-verified
        // against their carried successor material.
        let store = RotationStore::new(&path);
        let records = store.load().expect("both processes' records verify");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].record.rotated_at, AT);
        assert_eq!(records[1].record.rotated_at, XPROC_AT);
        assert_eq!(records[1].line, 2, "child appended after the parent");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- tamper falsifiers: every refusal names its line -----------------------

    #[test]
    fn tampered_record_content_refuses_naming_the_line() {
        let dir = temp_dir("tamper");
        let path = dir.join("rotations.jsonl");
        let store = RotationStore::new(&path);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        store
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("first append");
        store
            .append(&ceremony(&OLD_SEED_2, AT + 1), &es_pk, &ml_pk)
            .expect("second append");
        let intact = file_text(&path);

        // Flip line 2's signed content (rotated_at): valid JSON envelope,
        // valid format stamp — the refusal must come from re-verification.
        rewrite_line(&path, 2, |value| {
            value["record"]["rotated_at"] =
                serde_json::json!(value["record"]["rotated_at"].as_u64().expect("u64") + 1);
        });
        let err = store.load().expect_err("tampered line 2 must refuse");
        match err {
            RotationStoreError::Tampered { line, kid } => {
                assert_eq!(line, 2, "the refusal names the tampered line");
                assert!(!kid.is_empty());
            }
            other => panic!("expected Tampered, got {other:?}"),
        }

        // The same flip on line 1 names line 1 (the number is evidence, not
        // a constant).
        std::fs::write(&path, &intact).expect("restore store");
        rewrite_line(&path, 1, |value| {
            value["record"]["to_profile"] = serde_json::json!("PQC");
        });
        let err = store.load().expect_err("tampered line 1 must refuse");
        assert!(
            matches!(err, RotationStoreError::Tampered { line: 1, .. }),
            "got {err:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn swapped_successor_material_refuses_naming_the_line() {
        let dir = temp_dir("swap");
        let path = dir.join("rotations.jsonl");
        let store = RotationStore::new(&path);
        // TWO DIFFERENT successors, so cross-substitution is detectable.
        let first = hybrid_secret();
        let second = other_hybrid_secret();
        let (es1, ml1) = successor_pks(&first);
        let (es2, ml2) = successor_pks(&second);
        assert_ne!(es1, es2, "fixture successors must be distinct keys");
        store
            .append(&ceremony_under(&first, &OLD_SEED, AT), &es1, &ml1)
            .expect("first append");
        store
            .append(&ceremony_under(&second, &OLD_SEED_2, AT + 1), &es2, &ml2)
            .expect("second append");

        // Replace line 2's successor ES256 material with line 1's: the
        // stored record's key id no longer matches its carried keys, so the
        // identity digest refuses — the store detects a key-swap attack
        // without any signature work.
        let first_es: Vec<u8> = {
            let text = file_text(&path);
            let line: serde_json::Value =
                serde_json::from_str(text.lines().next().expect("line 1")).expect("parse");
            serde_json::from_value(line["successor_public_es256"].clone()).expect("es bytes")
        };
        rewrite_line(&path, 2, |value| {
            value["successor_public_es256"] =
                serde_json::Value::Array(first_es.iter().map(|&b| serde_json::json!(b)).collect());
        });
        let err = store.load().expect_err("swapped material must refuse");
        assert!(
            matches!(err, RotationStoreError::Tampered { line: 2, .. }),
            "got {err:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn io_variant_witnessed_by_absent_store_and_unusable_locations() {
        let dir = temp_dir("io");
        // (a) Fail-closed load: an absent store is NOT an empty history.
        let store = store_in(&dir);
        let err = store.load().expect_err("absent store must refuse");
        assert!(matches!(err, RotationStoreError::Io(_)), "got {err:?}");
        assert!(
            store.latest_for_successor("afk1_missingkey000000").is_err(),
            "queries refuse too, they never answer from nothing"
        );
        // (b) A store path beneath a REGULAR FILE is unwritable.
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, b"not a directory").expect("write blocker file");
        let blocked = RotationStore::new(blocker.join("rotations.jsonl"));
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        let err = blocked
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect_err("location under a file must be refused");
        assert!(matches!(err, RotationStoreError::Io(_)), "got {err:?}");
        // (c) A store path that IS a directory is unopenable for append.
        let as_dir = RotationStore::new(dir.join("as-dir"));
        std::fs::create_dir_all(as_dir.path()).expect("create dir target");
        let err = as_dir
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect_err("directory-as-store must be refused");
        assert!(matches!(err, RotationStoreError::Io(_)), "got {err:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn serialization_variant_witnessed_by_broken_lines() {
        let dir = temp_dir("serialization");
        let path = dir.join("rotations.jsonl");
        let store = RotationStore::new(&path);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        store
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("first append");

        // (a) Line 2 is not JSON at all.
        {
            let mut file = OpenOptions::new().append(true).open(&path).expect("open");
            file.write_all(b"not json at all\n")
                .expect("write junk line");
        }
        let err = store.load().expect_err("unparsable line must refuse");
        assert!(
            matches!(err, RotationStoreError::Serialization { line: 2, .. }),
            "got {err:?}"
        );

        // (b) A structurally valid JSON object that is not an envelope
        // (missing fields) refuses with the line number.
        let partial = serde_json::to_string(&serde_json::json!({ "format": STORE_FORMAT }))
            .expect("serialize partial envelope");
        std::fs::write(&path, partial + "\n").expect("write partial envelope");
        let err = store.load().expect_err("partial envelope must refuse");
        assert!(
            matches!(err, RotationStoreError::Serialization { line: 1, .. }),
            "got {err:?}"
        );

        // (c) An empty line IN THE MIDDLE is not silently skipped. Start
        // from a clean two-line store so earlier debris cannot answer first.
        let clean = dir.join("clean.jsonl");
        let clean_store = RotationStore::new(&clean);
        clean_store
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("clean first append");
        clean_store
            .append(&ceremony(&OLD_SEED_2, AT + 1), &es_pk, &ml_pk)
            .expect("clean second append");
        let text = file_text(&clean);
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        lines.insert(1, String::new());
        std::fs::write(&clean, lines.join("\n") + "\n").expect("write hole");
        let err = clean_store.load().expect_err("an empty line is a refusal");
        assert!(
            matches!(err, RotationStoreError::Serialization { line: 2, .. }),
            "got {err:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_format_stamp_refuses_naming_the_line() {
        let dir = temp_dir("format");
        let path = dir.join("rotations.jsonl");
        let store = RotationStore::new(&path);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        store
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("append");
        rewrite_line(&path, 1, |value| {
            value["format"] = serde_json::json!("CTP-ROTSTORE-v0");
        });
        let err = store.load().expect_err("foreign stamp must refuse");
        match err {
            RotationStoreError::WrongFormat { line, found } => {
                assert_eq!(line, 1);
                assert_eq!(found, "CTP-ROTSTORE-v0");
            }
            other => panic!("expected WrongFormat, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn structurally_broken_signature_refuses_as_verification_naming_the_line() {
        let dir = temp_dir("verification");
        let path = dir.join("rotations.jsonl");
        let store = RotationStore::new(&path);
        let (es_pk, ml_pk) = successor_pks(&hybrid_secret());
        store
            .append(&ceremony(&OLD_SEED, AT), &es_pk, &ml_pk)
            .expect("append");
        // Same total length (the fixed-length hybrid split still succeeds),
        // broken DER structure in the classical half: the provider refuses.
        rewrite_line(&path, 1, |value| {
            let sig = value["record"]["successor_signature"][0]
                .as_u64()
                .expect("byte");
            value["record"]["successor_signature"][0] = serde_json::json!(sig ^ 0xFF);
        });
        let err = store.load().expect_err("broken DER must be refused");
        assert!(
            matches!(err, RotationStoreError::Verification { line: 1, .. }),
            "got {err:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- empty store + rendered graph facts ------------------------------------

    #[test]
    fn empty_store_file_loads_to_zero_records() {
        let dir = temp_dir("empty");
        let path = dir.join("rotations.jsonl");
        std::fs::write(&path, "").expect("write empty store");
        let records = RotationStore::new(&path)
            .load()
            .expect("an existing but empty store is a valid empty history");
        assert!(records.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(ROTATION_STORE_FILE, ".affi/rotations.jsonl");
        assert_eq!(STORE_FORMAT, "CTP-ROTSTORE-v1");
        assert!(!DOMAIN_TAG.is_empty());
        // The default handle resolves the graph const.
        assert_eq!(
            RotationStore::default().path(),
            Path::new(ROTATION_STORE_FILE)
        );
    }

    #[test]
    fn rendered_consts_agree_with_sibling_modules() {
        // Same graph fact rendered by three rules: the store surface's
        // domain tag must equal the canonicalization module's and the
        // rotation ceremony's plane tag.
        assert_eq!(DOMAIN_TAG, CANONICAL_DOMAIN_TAG);
        assert_eq!(
            DOMAIN_TAG,
            crate::crypto_trust_rotation::ROTATION_DOMAIN_TAG
        );
        // The successor identity the store indexes by IS the trust-plane
        // key id format (afk1_ + 16 hex chars).
        let kid = KeyId::from_fingerprint(&crate::crypto_trust_keys::KeyFingerprint([7u8; 32]));
        assert_eq!(kid.to_string().len(), 21);
        assert!(kid.to_string().starts_with("afk1_"));
    }
}
