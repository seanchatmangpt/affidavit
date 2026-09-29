//! Durable publication file for the signed revocation list (CTP-CRL-v1) of
//! the affidavit cryptographic trust plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//!
//! File-surface law: the signed publication
//! ([`crate::crypto_trust_revocation::SignedRevocationList`]) persists at the
//! graph's publication file as JCS (RFC 8785) canonical JSON, so the bytes on
//! disk are exactly the canonical form the signature covers — no pretty-print
//! drift between wire and file. [`CrlFile::write`] stages the bytes to a
//! same-directory temporary file, syncs, and renames them into place: a crash
//! before the rename leaves the previous publication intact (readers never
//! see a partial CRL).
//!
//! Fail-closed law: [`CrlFile::read`] refuses an absent file. A missing CRL
//! is NOT an empty CRL — without a verified publication the verifier knows
//! nothing about revocations, and assuming "nothing is revoked" from missing
//! bytes would invert the safety property. A foreign-format stamp is refused
//! as [`CrlFileError::WrongFormat`]; unparsable bytes as
//! [`CrlFileError::Serialization`].
//!
//! Admission law: this surface only carries bytes, never the admission
//! decision. [`CrlFile::publish_verified`] publishes through the revocation
//! module's [`crate::crypto_trust_revocation::publish`] (sign-first), and
//! [`CrlFile::load_and_apply`] admits through
//! [`crate::crypto_trust_revocation::apply_to`], which verifies the issuer
//! signature FIRST, then the epoch freshness grace, then merges. The
//! revocation module's typed refusals are wired honestly into
//! [`CrlFileError`]; a refused apply leaves the target list unchanged (the
//! atomicity is the revocation module's, witnessed again end-to-end here).
//!
//! Determinism law: the same list, key, epoch, and instant render
//! byte-identical files — RFC 6979 signatures are deterministic, the record
//! mirror is sorted by `kid`, and the file bytes are the JCS canonicalization.
//!
//! House law: refusals are typed values ([`CrlFileError`]), never panics; no
//! `unwrap`/`expect` on production paths.

// Consumed query columns (crl-file.rq): publication_file, domain_tag.

use crate::crypto_trust_canonical::jcs;
use crate::crypto_trust_es256::Es256SigningKey;
use crate::crypto_trust_keys::KeyId;
use crate::crypto_trust_lifecycle::RevocationList;
use crate::crypto_trust_revocation::{
    apply_to, publish, RevocationPubError, SignedRevocationList, CRL_FORMAT,
};
use std::io::Write;
use std::path::{Path, PathBuf};

/// CRL publication file location (ctp:crl-file-v1 ctp:publicationFile),
/// relative to the consumer repository root. The default of every
/// `Option<&Path>` on this surface.
pub const CRL_FILE: &str = ".affi/crl.json";

/// Trust-plane domain tag (ctp:policy-v1 ctp:domainTag). Carried so the file
/// surface states the plane it serves; conformance-tested equal to the
/// revocation and canonicalization modules' tags at render time.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Typed refusal of a CRL file transition. Refusals are data: every variant
/// carries the evidence a verifier needs to act on the denial. The variants
/// wire the underlying revocation module's [`RevocationPubError`] kinds
/// honestly (see the mapping on [`CrlFile::load_and_apply`]).
#[derive(Debug, thiserror::Error)]
pub enum CrlFileError {
    /// The file's format stamp is not [`CRL_FORMAT`].
    #[error("bad crl file format {0:?}: expected {CRL_FORMAT}")]
    WrongFormat(String),
    /// The file location is unwritable or unreadable (path component is a
    /// regular file, permission denied, missing parent, absent file — a
    /// missing CRL is refused, never treated as an empty one).
    #[error("crl file io: {0}")]
    Io(String),
    /// The file is not valid JSON, the publication does not deserialize, or
    /// its content cannot be canonicalized (JCS refuses integers beyond
    /// double precision).
    #[error("crl file serialization: {0}")]
    Serialization(String),
    /// The file's issuer signature is malformed or does not verify over the
    /// publication's canonical preimage: the bytes on disk are not what the
    /// issuer signed.
    #[error("crl file verification: {0}")]
    Verification(String),
    /// The publication in the file is stale: its epoch (or signing instant)
    /// lags the verifier's state beyond the staleness grace.
    #[error("crl file staleness: {0}")]
    Staleness(String),
}

/// Handle on the durable CRL publication file. `Default` resolves the graph
/// const [`CRL_FILE`]; every operation also takes an explicit
/// `Option<&Path>` override so callers can stage files outside the default
/// location (tests, multi-plane layouts).
#[derive(Debug, Clone)]
pub struct CrlFile {
    path: PathBuf,
}

impl Default for CrlFile {
    fn default() -> Self {
        CrlFile {
            path: PathBuf::from(CRL_FILE),
        }
    }
}

impl CrlFile {
    /// A handle on the publication file at `path`.
    pub fn new(path: impl Into<PathBuf>) -> CrlFile {
        CrlFile { path: path.into() }
    }

    /// The publication file this handle names.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Effective publication path: the explicit override, or the graph const
    /// [`CRL_FILE`] when `None`.
    fn resolve(path: Option<&Path>) -> PathBuf {
        path.map_or_else(|| PathBuf::from(CRL_FILE), Path::to_path_buf)
    }

    /// Serializes `list` to JCS canonical JSON and writes it atomically to
    /// `path` (`None` = the graph const [`CRL_FILE`]). Write law: stage to a
    /// sibling temporary file (`.<name>.tmp-<pid>`), `sync_all`, then
    /// `rename` into place — the rename lands all bytes at once or none, so
    /// a crash leaves the previous publication intact and at most an
    /// orphaned dot-prefixed temporary (removed best-effort on a failed
    /// rename). Returns the path written.
    ///
    /// Refusals: an unusable location is [`CrlFileError::Io`]; content that
    /// cannot be canonicalized (an instant beyond 2^53, per JCS) is
    /// [`CrlFileError::Serialization`] — the file is never written with
    /// non-canonical bytes.
    pub fn write(
        list: &SignedRevocationList,
        path: Option<&Path>,
    ) -> Result<PathBuf, CrlFileError> {
        let path = Self::resolve(path);
        let value =
            serde_json::to_value(list).map_err(|e| CrlFileError::Serialization(e.to_string()))?;
        let canonical = jcs(&value).map_err(|e| CrlFileError::Serialization(e.to_string()))?;
        Self::write_atomic(&path, canonical.as_bytes())?;
        Ok(path)
    }

    /// Reads the publication file at `path` (`None` = the graph const
    /// [`CRL_FILE`]). Read law: an absent file is refused as
    /// [`CrlFileError::Io`] (a missing CRL is not an empty CRL — fail
    /// closed), unparsable bytes as [`CrlFileError::Serialization`], and a
    /// foreign format stamp as [`CrlFileError::WrongFormat`]. No signature
    /// check happens here: verification is admission's job
    /// ([`CrlFile::load_and_apply`]).
    pub fn read(path: Option<&Path>) -> Result<SignedRevocationList, CrlFileError> {
        let path = Self::resolve(path);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| CrlFileError::Io(format!("read {}: {e}", path.display())))?;
        let list: SignedRevocationList =
            serde_json::from_str(&text).map_err(|e| CrlFileError::Serialization(e.to_string()))?;
        if list.format != CRL_FORMAT {
            return Err(CrlFileError::WrongFormat(list.format));
        }
        Ok(list)
    }

    /// Publishes `revocations` under `issuer` (signature-first, through the
    /// revocation module's law) and writes the signed publication to `out`
    /// (`None` = the graph const [`CRL_FILE`]). Returns the path written.
    pub fn publish_verified(
        revocations: &RevocationList,
        issuer: &Es256SigningKey,
        issuer_kid: &KeyId,
        epoch: u64,
        at: u64,
        out: Option<&Path>,
    ) -> Result<PathBuf, CrlFileError> {
        let crl = publish(revocations, issuer, issuer_kid, epoch, at).map_err(map_pub_err)?;
        Self::write(&crl, out)
    }

    /// Reads the publication at `path` (`None` = the graph const
    /// [`CRL_FILE`]) and applies it to `revocations` through the revocation
    /// module's admission law: issuer signature FIRST, then the epoch
    /// freshness grace, then the merge. Returns the number of records
    /// applied. A refusal is atomic — the list is left unchanged.
    ///
    /// Refusal wiring (honest, exhaustive over [`RevocationPubError`]):
    /// `BadFormat` -> [`CrlFileError::WrongFormat`]; `BadSignature` ->
    /// [`CrlFileError::Verification`]; `StaleEpoch` ->
    /// [`CrlFileError::Staleness`] carrying the full evidence tuple;
    /// `Serialization` and `Lifecycle` (both canonicalization/projection
    /// -shape refusals of the publication or its source list) ->
    /// [`CrlFileError::Serialization`].
    pub fn load_and_apply(
        path: Option<&Path>,
        revocations: &mut RevocationList,
        issuer_pk: &[u8],
        current_epoch: u64,
        max_staleness: u64,
        now: u64,
    ) -> Result<usize, CrlFileError> {
        let crl = Self::read(path)?;
        apply_to(
            revocations,
            &crl,
            issuer_pk,
            current_epoch,
            max_staleness,
            now,
        )
        .map_err(map_pub_err)
    }

    /// Atomic write: serialize -> sibling temporary file (`.<name>.tmp-<pid>`)
    /// -> `sync_all` -> `rename`. On a failed rename the temporary is removed
    /// best-effort.
    fn write_atomic(path: &Path, body: &[u8]) -> Result<(), CrlFileError> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)
            .map_err(|e| CrlFileError::Io(format!("create dir {}: {e}", parent.display())))?;
        let name = path
            .file_name()
            .ok_or_else(|| CrlFileError::Io(format!("path has no file name: {}", path.display())))?
            .to_string_lossy()
            .into_owned();
        let tmp = parent.join(format!(".{name}.tmp-{}", std::process::id()));
        {
            let mut file = std::fs::File::create(&tmp)
                .map_err(|e| CrlFileError::Io(format!("create {}: {e}", tmp.display())))?;
            file.write_all(body)
                .map_err(|e| CrlFileError::Io(format!("write {}: {e}", tmp.display())))?;
            file.sync_all()
                .map_err(|e| CrlFileError::Io(format!("sync {}: {e}", tmp.display())))?;
        }
        match std::fs::rename(&tmp, path) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(CrlFileError::Io(format!(
                    "rename {} -> {}: {e}",
                    tmp.display(),
                    path.display()
                )))
            }
        }
    }
}

/// The single honest wiring of the revocation module's refusals into the
/// file surface's vocabulary (documented on [`CrlFile::load_and_apply`]).
fn map_pub_err(err: RevocationPubError) -> CrlFileError {
    match err {
        RevocationPubError::BadFormat(format) => CrlFileError::WrongFormat(format),
        RevocationPubError::BadSignature => CrlFileError::Verification(String::from(
            "publication signature does not verify",
        )),
        RevocationPubError::StaleEpoch {
            published,
            current,
            max_staleness,
        } => CrlFileError::Staleness(format!(
            "stale publication: epoch {published} vs current {current}, max staleness {max_staleness}"
        )),
        RevocationPubError::Serialization(message) | RevocationPubError::Lifecycle(message) => {
            CrlFileError::Serialization(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_canonical::DOMAIN_TAG as CANONICAL_DOMAIN_TAG;
    use crate::crypto_trust_lifecycle::MAX_REVOCATION_STALENESS_SECONDS;
    use crate::crypto_trust_revocation::{verify_publication, DOMAIN_TAG as REVOCATION_DOMAIN_TAG};
    use std::sync::atomic::{AtomicU64, Ordering};

    const KID_A: &str = "k-a";
    const KID_B: &str = "k-b";
    const REASON: &str = "compromised";
    const PUBLISHED_AT: u64 = 1_000_000;

    /// Unique scratch directory per call (no tempfile dependency; std only).
    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("ctp-crl-file-{tag}-{pid}-{n}"));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    fn fixture_key() -> Es256SigningKey {
        let mut seed = [0x0Cu8; 32];
        seed[0] = 0x42;
        Es256SigningKey::from_seed(&seed).expect("valid fixture scalar")
    }

    fn fixture_list() -> RevocationList {
        let mut list = RevocationList::default();
        list.revoke(KID_A, 999_000, REASON.to_string());
        list.revoke(KID_B, 999_500, "superseded".to_string());
        list
    }

    fn issuer_kid(issuer: &Es256SigningKey) -> KeyId {
        KeyId::from_fingerprint(&issuer.key_id_fingerprint())
    }

    fn file_text(path: &Path) -> String {
        std::fs::read_to_string(path).expect("read crl file")
    }

    // -- teeth first: publish -> write -> read -> verify -> apply ------------

    #[test]
    fn publish_write_read_verify_apply_round_trip() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        let source = fixture_list();

        let written = CrlFile::publish_verified(
            &source,
            &issuer,
            &kid,
            source.current_epoch(),
            PUBLISHED_AT,
            Some(&path),
        )
        .expect("publishes and writes");
        assert_eq!(written, path);
        assert!(path.is_file(), "publication file created at {:?}", path);

        let crl = CrlFile::read(Some(&path)).expect("reads back");
        assert_eq!(crl.format, CRL_FORMAT);
        assert_eq!(crl.issuer_kid, kid.0);
        assert_eq!(crl.revoked.len(), 2);
        assert!(
            verify_publication(&crl, &issuer.public_key_sec1()).expect("verification runs"),
            "the published file must verify under the issuer key"
        );

        let mut target = RevocationList::default();
        let current = target.current_epoch();
        let applied = CrlFile::load_and_apply(
            Some(&path),
            &mut target,
            &issuer.public_key_sec1(),
            current,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect("verified file applies");
        assert_eq!(applied, 2, "every record of a verified file applies");
        assert!(target.is_revoked(KID_A));
        assert_eq!(target.revoked_at(KID_B), Some(999_500));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- determinism: same inputs -> byte-identical files ---------------------

    #[test]
    fn same_inputs_render_identical_file_bytes() {
        let dir = temp_dir("determinism");
        let one = dir.join("one.json");
        let two = dir.join("two.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        for path in [&one, &two] {
            CrlFile::publish_verified(
                &fixture_list(),
                &issuer,
                &kid,
                42,
                PUBLISHED_AT,
                Some(path.as_path()),
            )
            .expect("publishes");
        }
        assert_eq!(
            std::fs::read(&one).expect("read one"),
            std::fs::read(&two).expect("read two"),
            "RFC 6979 + sorted mirror + JCS = byte-identical files"
        );
        // And the bytes move once any input moves.
        let three = dir.join("three.json");
        CrlFile::publish_verified(
            &fixture_list(),
            &issuer,
            &kid,
            42,
            PUBLISHED_AT + 1,
            Some(&three),
        )
        .expect("publishes");
        assert_ne!(
            std::fs::read(&one).expect("read one"),
            std::fs::read(&three).expect("read three")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- every variant witnessed ----------------------------------------------

    #[test]
    fn tampered_file_refused_as_verification() {
        let dir = temp_dir("tamper");
        let path = dir.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        CrlFile::publish_verified(&fixture_list(), &issuer, &kid, 1, PUBLISHED_AT, Some(&path))
            .expect("publishes");

        // Flip one byte inside the SIGNED content (kid "k-a" -> "k-b"): the
        // file stays valid JSON with a valid format stamp, so the refusal
        // must come from the issuer signature, not the parser.
        let intact = file_text(&path);
        let tampered = intact.replacen("\"kid\":\"k-a\"", "\"kid\":\"k-b\"", 1);
        assert_ne!(tampered, intact, "fixture must contain the flipped byte");
        std::fs::write(&path, tampered).expect("write tampered file");

        let mut target = RevocationList::default();
        let before = serde_json::to_value(&target).expect("list projects");
        let err = CrlFile::load_and_apply(
            Some(&path),
            &mut target,
            &issuer.public_key_sec1(),
            1,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect_err("tampered file must be refused");
        match err {
            CrlFileError::Verification(message) => {
                assert!(message.contains("does not verify"), "{message}");
            }
            other => panic!("expected Verification, got {other:?}"),
        }
        assert_eq!(
            serde_json::to_value(&target).expect("list projects"),
            before,
            "a refused file apply must be atomic"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_format_stamp_refused_on_read() {
        let dir = temp_dir("format");
        let path = dir.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        CrlFile::publish_verified(&fixture_list(), &issuer, &kid, 1, PUBLISHED_AT, Some(&path))
            .expect("publishes");
        let intact = file_text(&path);
        let foreign = intact.replacen(
            &format!("\"format\":\"{CRL_FORMAT}\""),
            "\"format\":\"CTP-CRL-v0\"",
            1,
        );
        assert_ne!(foreign, intact, "fixture must carry the foreign stamp");
        std::fs::write(&path, foreign).expect("write foreign-format file");
        let err = CrlFile::read(Some(&path)).expect_err("foreign format must be refused");
        match err {
            CrlFileError::WrongFormat(format) => assert_eq!(format, "CTP-CRL-v0"),
            other => panic!("expected WrongFormat, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stale_file_refused_as_staleness_past_grace() {
        let dir = temp_dir("stale");
        let path = dir.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        CrlFile::publish_verified(
            &fixture_list(),
            &issuer,
            &kid,
            1_000,
            PUBLISHED_AT,
            Some(&path),
        )
        .expect("publishes");
        let crl = CrlFile::read(Some(&path)).expect("reads back");

        // Exactly at the grace boundary the file still applies.
        let mut target = RevocationList::default();
        let at_bound = CrlFile::load_and_apply(
            Some(&path),
            &mut target,
            &issuer.public_key_sec1(),
            crl.epoch + MAX_REVOCATION_STALENESS_SECONDS,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        );
        assert!(
            at_bound.is_ok(),
            "lag exactly at the bound is inside the grace"
        );

        // One epoch past the grace: typed Staleness refusal.
        let mut untouched = RevocationList::default();
        let err = CrlFile::load_and_apply(
            Some(&path),
            &mut untouched,
            &issuer.public_key_sec1(),
            crl.epoch + MAX_REVOCATION_STALENESS_SECONDS + 1,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect_err("stale file must be refused");
        match err {
            CrlFileError::Staleness(message) => assert!(
                message.contains(&format!("max staleness {MAX_REVOCATION_STALENESS_SECONDS}")),
                "{message}"
            ),
            other => panic!("expected Staleness, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn io_variant_witnessed_by_unusable_locations() {
        let dir = temp_dir("io");
        // (a) Dir-as-file: a CRL path beneath a REGULAR FILE is unwritable
        // (the staging law refuses at create_dir_all), so publish refuses
        // with Io.
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, b"not a directory").expect("write blocker file");
        let blocked = blocker.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        let err = CrlFile::publish_verified(
            &fixture_list(),
            &issuer,
            &kid,
            1,
            PUBLISHED_AT,
            Some(&blocked),
        )
        .expect_err("location under a file must be refused");
        assert!(matches!(err, CrlFileError::Io(_)));
        // (b) Fail-closed read: an absent file is NOT an empty CRL.
        let absent = dir.join("absent").join("crl.json");
        let err = CrlFile::read(Some(&absent)).expect_err("absent CRL must be refused");
        assert!(matches!(err, CrlFileError::Io(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn serialization_variant_witnessed() {
        let dir = temp_dir("serialization");
        let path = dir.join("crl.json");
        // (a) A file that is not JSON at all: the read law refuses with
        // Serialization (typed, never a panic).
        std::fs::write(&path, "[format: oops]").expect("write malformed file");
        let err = CrlFile::read(Some(&path)).expect_err("malformed json must be refused");
        assert!(matches!(err, CrlFileError::Serialization(_)));
        // (b) Production-path refusal: JCS refuses integers beyond double
        // precision (2^53), so a signing instant past that bound cannot be
        // published to a file at all.
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        let beyond_jcs = 9_007_199_254_740_993_u64; // 2^53 + 1
        let err =
            CrlFile::publish_verified(&fixture_list(), &issuer, &kid, 1, beyond_jcs, Some(&path))
                .expect_err("instant beyond JCS precision must be refused");
        assert!(matches!(err, CrlFileError::Serialization(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- empty list + atomicity of the write -----------------------------------

    #[test]
    fn empty_list_publishes_and_applies_zero() {
        let dir = temp_dir("empty");
        let path = dir.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        CrlFile::publish_verified(
            &RevocationList::default(),
            &issuer,
            &kid,
            0,
            PUBLISHED_AT,
            Some(&path),
        )
        .expect("empty list publishes");
        let crl = CrlFile::read(Some(&path)).expect("reads back");
        assert!(crl.revoked.is_empty());
        let mut target = RevocationList::default();
        let applied = CrlFile::load_and_apply(
            Some(&path),
            &mut target,
            &issuer.public_key_sec1(),
            0,
            MAX_REVOCATION_STALENESS_SECONDS,
            PUBLISHED_AT,
        )
        .expect("empty publication applies");
        assert_eq!(applied, 0);
        assert!(!target.is_revoked(KID_A));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_write_leaves_no_temporaries_behind() {
        let dir = temp_dir("atomic");
        let path = dir.join("crl.json");
        let issuer = fixture_key();
        let kid = issuer_kid(&issuer);
        CrlFile::publish_verified(&fixture_list(), &issuer, &kid, 1, PUBLISHED_AT, Some(&path))
            .expect("first publish");
        CrlFile::publish_verified(&fixture_list(), &issuer, &kid, 2, PUBLISHED_AT, Some(&path))
            .expect("second publish overwrites atomically");
        let mut entries: Vec<String> = std::fs::read_dir(&dir)
            .expect("list scratch dir")
            .map(|entry| {
                entry
                    .expect("dir entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        entries.sort();
        assert_eq!(entries, vec!["crl.json".to_string()], "no .tmp-* remains");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // -- graph facts rendered ---------------------------------------------------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(CRL_FILE, ".affi/crl.json");
        assert!(!DOMAIN_TAG.is_empty());
        // The default handle resolves the graph const.
        assert_eq!(CrlFile::default().path(), Path::new(CRL_FILE));
    }

    #[test]
    fn rendered_consts_agree_with_sibling_modules() {
        // Same graph fact rendered by three rules: the file surface's domain
        // tag must equal the revocation and canonicalization modules'.
        assert_eq!(DOMAIN_TAG, REVOCATION_DOMAIN_TAG);
        assert_eq!(DOMAIN_TAG, CANONICAL_DOMAIN_TAG);
    }
}
