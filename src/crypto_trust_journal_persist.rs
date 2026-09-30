//! Durable, cross-process nonce ledger for the affidavit cryptographic trust
//! plane.
//! Rendered by ggen sync from affidavit-trust-plane-pack (the pack is authoritative; never edit this file).
//! Consumed query columns (journal-persist.rq): journal_file, domain_tag.
//!
//! Closes the wave-5 review gap "NonceJournal is per-invocation": the
//! in-memory nonce journal of `crypto_trust_envelope` lives and dies with one
//! verifier invocation, so replay evidence could never survive the process
//! that witnessed it. This module is the durable twin: an append-only JSONL
//! file of `{kid, nonce_hex, seen_at}` records at [`PERSIST_FILE`], one JSON
//! object per line, so a fresh process re-opens the file and is refused on a
//! nonce it never held in memory.
//!
//! WINDOW LAW (mirrors `crypto_trust_envelope::NonceJournal` byte for byte):
//! the acceptance window is the half-open `[first_seen, first_seen +
//! window_seconds)`; a repeat with `at - seen_at < window_seconds` is refused
//! [`PersistError::Replay`]; at or beyond the boundary (`>=`) the new
//! sighting is appended and admitted; `prune` evicts exactly the entries a
//! re-record at `now` would admit. The ledger key is `kid + ":" +
//! lowercase_hex(nonce)` — the graph's replay tuple
//! (ctp:nonce-policy-v1 ctp:replayKey = "kid,nonce") — collision-free because
//! key ids are hex-derived and the nonce suffix is fixed-length.
//!
//! ATOMICITY LAW: `record` and `prune` never write the live path in place.
//! The next file image is written to a same-directory tmp file (same
//! filesystem, so rename is atomic), synced to disk with `sync_all` BEFORE
//! the rename, then renamed over the live path. A reader on the other side of
//! the rename therefore never observes a partial file, and a crash mid-write
//! leaves the previous image intact and at most an orphaned tmp file.
//!
//! CORRUPTION LAW: the wire form is never trusted without re-derivation.
//! Every load/refresh/re-verify parses each line and validates its
//! `nonce_hex` as exactly 32 lowercase hex chars; the first offending line is
//! refused [`PersistError::Corruption`] with its exact 1-based line number.
//!
//! EVIDENCE SPLIT (honest, no fake bridging): the rendered
//! `crypto_trust_journal::StandingJournal` carries NO nonce state — its
//! entries bind key_id, epochs, standing, commitments and hashes only. The
//! surfaces therefore split by authority: THIS ledger is authoritative for
//! (kid, nonce) replay evidence across processes; StandingJournal stays
//! authoritative for sealed receipt evidence. They are joined by key_id only,
//! never by a forged nonce column, and no mirror function is offered — there
//! is nothing lawful to mirror from.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Default durable ledger file (ctp:journal-persist-v1
/// ctp:persistenceFile), relative to the consumer repository root. One
/// [`NonceRecord`] JSON object per line; the file grows append-only and is
/// rewritten in full only by [`NonceLedgerFile::prune`].
pub const PERSIST_FILE: &str = ".affi/nonce-journal.jsonl";

/// Trust-plane domain tag (ctp:policy-v1 ctp:domainTag) whose replay tuple
/// this ledger is evidence for. Replay evidence minted under any other domain
/// tag is not this ledger's evidence; the test court pins this const to
/// `crypto_trust_journal::JOURNAL_DOMAIN` so every lane renders the same
/// graph fact.
pub const PERSIST_DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// One durable nonce sighting: the replay tuple plus the instant it was
/// recorded. The JSONL wire form is exactly this struct, one line per record,
/// written in field order kid, nonce_hex, seen_at.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NonceRecord {
    /// The registry key the nonce arrived under.
    pub kid: String,
    /// Lowercase hex of the 16-byte nonce: exactly 32 chars.
    pub nonce_hex: String,
    /// Seconds-since-epoch instant the sighting was recorded.
    pub seen_at: u64,
}

/// Typed refusal of a durable-ledger transition. Refusals are data: every
/// variant names the exact state that broke the durable replay-evidence law.
#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    /// `(kid, nonce)` was already seen strictly inside the acceptance window
    /// (`at - seen_at < window_seconds`): replay refused, nothing written.
    #[error("nonce replay for key {kid}")]
    Replay {
        /// The key the replayed nonce arrived under.
        kid: String,
    },
    /// The filesystem refused a durable transition (open, read, write, sync,
    /// rename) — including `load` of a ledger file that does not exist
    /// (callers that mean "start fresh" use `open_or_create`).
    #[error("io: {0}")]
    Io(String),
    /// The wire form is corrupt: the first offending line (1-based) either
    /// does not parse as a [`NonceRecord`] or carries a `nonce_hex` that is
    /// not 32 lowercase hex chars. The ledger refuses rather than coercing.
    #[error("corruption at line {line}: {reason}")]
    Corruption {
        /// 1-based number of the first offending line.
        line: usize,
        /// Why the line is corrupt (parse error or hex law).
        reason: String,
    },
    /// Canonical line serialization refused the material. Defensive: over
    /// [`NonceRecord`]'s closed field domain (`String` + `u64`) the JSON
    /// serializer is total, so this variant is the typed refusal the flush
    /// path carries for the transition that cannot currently fire.
    #[error("serialization: {0}")]
    Serialization(String),
}

impl From<std::io::Error> for PersistError {
    fn from(err: std::io::Error) -> Self {
        PersistError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(err: serde_json::Error) -> Self {
        PersistError::Serialization(err.to_string())
    }
}

/// Ledger key for the graph's replay tuple (ctp:nonce-policy-v1
/// ctp:replayKey = "kid,nonce"): `kid + ":" + lowercase_hex(nonce)` — the
/// same law as `crypto_trust_envelope`'s internal `nonce_journal_key`.
/// Collision-free: key ids are hex-derived (fixed alphabet, no `:`) and the
/// nonce suffix has fixed length 32, so distinct (kid, nonce) pairs map to
/// distinct keys.
fn ledger_key(kid: &str, nonce_hex: &str) -> String {
    format!("{kid}:{nonce_hex}")
}

/// Lowercase hex encoding (the wire law for `nonce_hex`).
fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

/// Strict decode of `nonce_hex` into exactly 16 bytes: refuses wrong length,
/// non-hex characters, and uppercase (the wire form is lowercase by law —
/// the ledger key law depends on the fixed lowercase alphabet).
fn hex_decode_nonce(s: &str) -> Option<[u8; 16]> {
    let bytes = s.as_bytes();
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 16];
    for (i, pair) in out.iter_mut().enumerate() {
        let hi = hex_val(bytes[2 * i])?;
        let lo = hex_val(bytes[2 * i + 1])?;
        *pair = (hi << 4) | lo;
    }
    Some(out)
}

/// The tmp sibling used by the atomic flush: same directory as the live path
/// (same filesystem, so `rename` is atomic), name derived from the live file
/// name.
fn tmp_sibling(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map_or_else(|| "ledger".to_string(), |n| n.to_string_lossy().to_string());
    path.with_file_name(format!("{name}.tmp"))
}

/// The durable, cross-process nonce ledger: an append-only JSONL file of
/// [`NonceRecord`]s at a fixed path, held against an in-memory view that is
/// refreshed from disk at every mutating transition (the cross-process sync
/// point — evidence written by another process is never missed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonceLedgerFile {
    /// The durable JSONL path (from [`PERSIST_FILE`] in production).
    path: PathBuf,
    /// The in-memory view: every record on disk, in file order (duplicates
    /// of one replay tuple are later re-sightings past the window boundary).
    entries: Vec<NonceRecord>,
}

impl NonceLedgerFile {
    /// Parse ledger bytes line by line and validate every record. The first
    /// offending line is refused [`PersistError::Corruption`] with its exact
    /// 1-based number — an empty line is corruption too, because every byte
    /// of the file is written by [`NonceLedgerFile::flush`].
    fn parse(contents: &str) -> Result<Vec<NonceRecord>, PersistError> {
        let mut entries = Vec::new();
        for (index, line) in contents.lines().enumerate() {
            let line_no = index + 1;
            if line.trim().is_empty() {
                return Err(PersistError::Corruption {
                    line: line_no,
                    reason: "empty line".to_string(),
                });
            }
            let record: NonceRecord =
                serde_json::from_str(line).map_err(|err| PersistError::Corruption {
                    line: line_no,
                    reason: err.to_string(),
                })?;
            if hex_decode_nonce(&record.nonce_hex).is_none() {
                return Err(PersistError::Corruption {
                    line: line_no,
                    reason: format!(
                        "nonce_hex {:?} is not 32 lowercase hex chars",
                        record.nonce_hex
                    ),
                });
            }
            entries.push(record);
        }
        Ok(entries)
    }

    /// Load an existing ledger file and re-verify every byte it reads. A
    /// missing file is a typed [`PersistError::Io`] refusal — the caller
    /// asked for evidence that does not exist; callers that mean "start
    /// fresh" use [`NonceLedgerFile::open_or_create`].
    pub fn load(path: impl Into<PathBuf>) -> Result<Self, PersistError> {
        let path = path.into();
        let contents = fs::read_to_string(&path)?;
        let entries = Self::parse(&contents)?;
        Ok(NonceLedgerFile { path, entries })
    }

    /// Open the ledger, starting from the empty ledger when the file does
    /// not exist yet (the file materializes at the first flush). An existing
    /// file is parsed and verified exactly like [`NonceLedgerFile::load`].
    pub fn open_or_create(path: impl Into<PathBuf>) -> Result<Self, PersistError> {
        let path = path.into();
        match fs::read_to_string(&path) {
            Ok(contents) => {
                let entries = Self::parse(&contents)?;
                Ok(NonceLedgerFile { path, entries })
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(NonceLedgerFile {
                path,
                entries: Vec::new(),
            }),
            Err(err) => Err(PersistError::from(err)),
        }
    }

    /// Re-read the durable file into the in-memory view — the cross-process
    /// sync point every mutating transition runs first, so evidence another
    /// process appended is never missed. A file that does not exist (yet)
    /// syncs to the empty view.
    pub fn refresh(&mut self) -> Result<(), PersistError> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                self.entries = Self::parse(&contents)?;
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.entries.clear();
                Ok(())
            }
            Err(err) => Err(PersistError::from(err)),
        }
    }

    /// Record sight of `(kid, nonce)` at instant `at` — the durable twin of
    /// `crypto_trust_envelope::NonceJournal::record` with the SAME window
    /// law: a repeat strictly inside the half-open window
    /// (`at - seen_at < window_seconds`) is refused
    /// [`PersistError::Replay`] and nothing is written; at or beyond the
    /// boundary the new sighting is appended. The ledger is refreshed from
    /// disk FIRST, then the record is appended and the file rewritten
    /// atomically (same-directory tmp, `sync_all`, rename).
    pub fn record(
        &mut self,
        kid: &str,
        nonce: [u8; 16],
        at: u64,
        window_seconds: u64,
    ) -> Result<(), PersistError> {
        self.refresh()?;
        if let Some(seen_at) = self.seen(kid, &nonce) {
            if at.saturating_sub(seen_at) < window_seconds {
                return Err(PersistError::Replay {
                    kid: kid.to_string(),
                });
            }
        }
        self.entries.push(NonceRecord {
            kid: kid.to_string(),
            nonce_hex: hex_encode(&nonce),
            seen_at: at,
        });
        self.flush()
    }

    /// The instant `(kid, nonce)` was last recorded, if present — the latest
    /// sighting wins, in the in-memory view as of the last
    /// load/open_or_create/refresh/record/prune.
    pub fn seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> {
        let key = ledger_key(kid, &hex_encode(nonce));
        self.entries
            .iter()
            .rfind(|record| ledger_key(&record.kid, &record.nonce_hex) == key)
            .map(|record| record.seen_at)
    }

    /// Evict every record whose instant lies at or beyond the window
    /// boundary (`now - seen_at >= window_seconds`) — the boundary matches
    /// [`NonceLedgerFile::record`]: an entry pruned at `now` is exactly an
    /// entry whose re-record at `now` would be admitted — and atomically
    /// rewrite the file without them. Returns the number of evicted records;
    /// a zero count writes nothing. The view is refreshed from disk first.
    pub fn prune(&mut self, now: u64, window_seconds: u64) -> Result<usize, PersistError> {
        self.refresh()?;
        let before = self.entries.len();
        self.entries
            .retain(|record| now.saturating_sub(record.seen_at) < window_seconds);
        let evicted = before - self.entries.len();
        if evicted > 0 {
            self.flush()?;
        }
        Ok(evicted)
    }

    /// Re-read the durable file from disk and re-verify every byte: every
    /// line parses as a [`NonceRecord`] and every `nonce_hex` is 32
    /// lowercase hex chars; the first offending line is refused
    /// [`PersistError::Corruption`] with its exact 1-based number. A file
    /// that does not exist yet verifies as the empty ledger.
    pub fn verify_integrity(&self) -> Result<(), PersistError> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                Self::parse(&contents)?;
                Ok(())
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(PersistError::from(err)),
        }
    }

    /// Atomic full-file flush: every record is serialized (refusals map to
    /// [`PersistError::Serialization`]), the next file image is written to a
    /// same-directory tmp file, synced with `sync_all`, and renamed over the
    /// live path — a reader after the rename never sees a partial file, and
    /// the tmp file never survives a completed flush.
    fn flush(&self) -> Result<(), PersistError> {
        if let Some(dir) = self.path.parent() {
            if !dir.as_os_str().is_empty() {
                fs::create_dir_all(dir)?;
            }
        }
        let mut lines = Vec::with_capacity(self.entries.len());
        for record in &self.entries {
            lines.push(serde_json::to_string(record)?);
        }
        let body = if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        };
        let tmp = tmp_sibling(&self.path);
        let mut file = fs::File::create(&tmp)?;
        file.write_all(body.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    /// The durable path this ledger writes.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The records of the current in-memory view, in file order (read-only).
    pub fn entries(&self) -> &[NonceRecord] {
        &self.entries
    }

    /// Number of records in the current in-memory view.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the current in-memory view holds no records.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// The plane domain every lane must render identically.
    const DOMAIN: &str = "affidavit.crypto-trust-plane.v1";
    const KID_A: &str = "afk1_aaaaaaaaaaaaaaaa";
    const KID_B: &str = "afk1_bbbbbbbbbbbbbbbb";

    static SCRATCH_SEQ: AtomicUsize = AtomicUsize::new(0);

    /// A unique scratch directory per call: tests never share a path, and a
    /// "fresh struct" test double really is fresh.
    fn scratch_dir(tag: &str) -> PathBuf {
        let n = SCRATCH_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "ctp-journal-persist-{tag}-{}-{n}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("scratch dir creates");
        dir
    }

    fn ledger_in(dir: &Path) -> PathBuf {
        dir.join(PERSIST_FILE).with_file_name("nonce-journal.jsonl")
    }

    /// Count orphaned tmp files: after any completed transition there must
    /// be none (the atomic flush renames its tmp away or fails typed).
    fn tmp_leftovers(dir: &Path) -> usize {
        fs::read_dir(dir)
            .expect("scratch dir reads")
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .count()
    }

    fn nonce(byte: u8) -> [u8; 16] {
        [byte; 16]
    }

    // -- teeth first: the rendered consts are the graph facts ----------------

    #[test]
    fn consts_render_pack_ontology() {
        assert_eq!(PERSIST_FILE, ".affi/nonce-journal.jsonl");
        assert_eq!(PERSIST_DOMAIN_TAG, DOMAIN);
        // Cross-module graph consistency: every lane rendered the same facts.
        assert_eq!(
            PERSIST_DOMAIN_TAG,
            crate::crypto_trust_journal::JOURNAL_DOMAIN
        );
        assert_eq!(
            PERSIST_DOMAIN_TAG,
            crate::crypto_trust_canonical::DOMAIN_TAG
        );
    }

    // -- THE cross-process test: the point of the lane -----------------------

    #[test]
    fn cross_process_replay_is_refused_from_disk() {
        let dir = scratch_dir("xproc");
        let path = ledger_in(&dir);

        // Process A: first sighting, then the struct is dropped — its memory
        // is gone and only the file carries the evidence.
        {
            let mut a = NonceLedgerFile::open_or_create(&path).expect("process A opens");
            assert!(a.is_empty());
            a.record(KID_A, nonce(7), 1_000, 600)
                .expect("first sighting records");
            assert_eq!(a.len(), 1);
            assert_eq!(tmp_leftovers(&dir), 0, "flush leaves no tmp behind");
        }

        // Process B: fresh struct, empty memory, the file is the only
        // witness. The same (kid, nonce) inside the window is refused.
        let mut b = NonceLedgerFile::load(&path).expect("process B reopens from disk");
        assert_eq!(b.len(), 1, "process B sees process A's sighting");
        assert_eq!(b.seen(KID_A, &nonce(7)), Some(1_000));
        match b.record(KID_A, nonce(7), 1_001, 600) {
            Err(PersistError::Replay { kid }) => assert_eq!(kid, KID_A),
            other => panic!("expected Replay, got {other:?}"),
        }
        // A refused record must not mutate the durable file.
        let wire = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(wire.lines().count(), 1, "replay refusal writes nothing");
        assert_eq!(tmp_leftovers(&dir), 0);

        // Window expiry admits: at 1_000 + 600 the half-open window has
        // closed, so the re-sighting is appended and becomes the latest.
        b.record(KID_A, nonce(7), 1_600, 600)
            .expect("expired sighting re-records");
        assert_eq!(b.len(), 2);
        assert_eq!(
            b.seen(KID_A, &nonce(7)),
            Some(1_600),
            "latest sighting wins"
        );
        let wire = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(wire.lines().count(), 2, "both sightings are durable");
        b.verify_integrity().expect("durable file verifies");
    }

    #[test]
    fn window_is_half_open_like_the_in_memory_journal() {
        let dir = scratch_dir("window");
        let path = ledger_in(&dir);
        let mut ledger = NonceLedgerFile::open_or_create(&path).expect("opens");
        ledger
            .record(KID_A, nonce(1), 5_000, 600)
            .expect("first sighting");

        // 5_599 - 5_000 = 599 < 600: strictly inside -> refused.
        match ledger.record(KID_A, nonce(1), 5_599, 600) {
            Err(PersistError::Replay { kid }) => assert_eq!(kid, KID_A),
            other => panic!("expected Replay at 599, got {other:?}"),
        }
        // 5_600 - 5_000 = 600 >= 600: boundary closed -> admitted.
        ledger
            .record(KID_A, nonce(1), 5_600, 600)
            .expect("boundary sighting admitted");
        assert_eq!(ledger.len(), 2);
        assert_eq!(ledger.seen(KID_A, &nonce(1)), Some(5_600));
    }

    #[test]
    fn late_opened_handle_never_misses_another_process_record() {
        let dir = scratch_dir("two-handles");
        let path = ledger_in(&dir);
        let mut a = NonceLedgerFile::open_or_create(&path).expect("A opens");
        a.record(KID_A, nonce(1), 100, 600).expect("A records");

        // B opens after A's write and appends its own tuple.
        let mut b = NonceLedgerFile::open_or_create(&path).expect("B opens");
        b.record(KID_B, nonce(2), 150, 600).expect("B records");
        let wire = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(wire.lines().count(), 2, "both processes' records persist");

        // A's stale in-memory view is refreshed before its replay check, so
        // it is refused on the tuple B recorded — cross-handle replay dies
        // exactly like cross-process replay.
        match a.record(KID_B, nonce(2), 160, 600) {
            Err(PersistError::Replay { kid }) => assert_eq!(kid, KID_B),
            other => panic!("expected cross-handle Replay, got {other:?}"),
        }
    }

    // -- load / open_or_create / integrity -----------------------------------

    #[test]
    fn load_of_a_missing_ledger_refuses_io_and_open_starts_fresh() {
        let dir = scratch_dir("missing");
        let path = ledger_in(&dir);
        match NonceLedgerFile::load(&path) {
            Err(PersistError::Io(_)) => {}
            other => panic!("expected Io, got {other:?}"),
        }
        // The same path through open_or_create starts the empty ledger and
        // creates the parent directories on first flush.
        let mut fresh = NonceLedgerFile::open_or_create(&path).expect("starts fresh");
        assert!(fresh.is_empty());
        assert_eq!(fresh.seen(KID_A, &nonce(9)), None);
        fresh
            .record(KID_A, nonce(9), 10, 600)
            .expect("record creates the file");
        assert!(path.is_file(), "flush materializes the ledger file");
        assert_eq!(fresh.len(), 1);
    }

    #[test]
    fn verify_integrity_accepts_honest_files_and_the_unborn_ledger() {
        let dir = scratch_dir("integrity-ok");
        let path = ledger_in(&dir);
        let mut ledger = NonceLedgerFile::open_or_create(&path).expect("opens");
        ledger
            .verify_integrity()
            .expect("a ledger whose file does not exist yet verifies as empty");
        for i in 0..5u8 {
            ledger
                .record(KID_A, nonce(0x20 + i), 100 + i as u64, 600)
                .expect("record");
        }
        ledger.verify_integrity().expect("honest file verifies");
        assert_eq!(ledger.entries().len(), 5);
        assert_eq!(ledger.path(), path);
    }

    // -- corruption: the wire form is never trusted --------------------------

    /// Writes 3 honest records, then returns the wire text.
    fn three_records(dir: &Path) -> (PathBuf, String) {
        let path = ledger_in(dir);
        let mut ledger = NonceLedgerFile::open_or_create(&path).expect("opens");
        for (i, kid) in [KID_A, KID_B, KID_A].iter().enumerate() {
            ledger
                .record(kid, nonce(0x30 + i as u8), 100 * (i as u64 + 1), 60_000)
                .expect("record");
        }
        let wire = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(wire.lines().count(), 3);
        (path, wire)
    }

    #[test]
    fn unparseable_line_refuses_corruption_with_exact_line() {
        let dir = scratch_dir("corrupt-json");
        let (path, wire) = three_records(&dir);
        // A ledger opened on the HONEST image, held across the tamper.
        let mut ledger = NonceLedgerFile::load(&path).expect("the honest image loads");

        let mut lines: Vec<String> = wire.lines().map(|l| l.to_string()).collect();
        // Flip line 2's first byte: the JSON object is gone.
        let damaged = lines[1].replacen('{', "X", 1);
        assert_ne!(damaged, lines[1], "the flip must change the line");
        lines[1] = damaged;
        fs::write(&path, lines.join("\n")).expect("tampered file writes");

        match NonceLedgerFile::load(&path) {
            Err(PersistError::Corruption { line, reason }) => {
                assert_eq!(line, 2);
                assert!(!reason.is_empty());
            }
            other => panic!("expected Corruption at line 2, got {other:?}"),
        }
        // And the already-open ledger refuses the tampered wire on
        // transition: refresh runs before any replay check or write.
        match ledger.record(KID_A, nonce(0x99), 10_000, 600) {
            Err(PersistError::Corruption { line, .. }) => assert_eq!(line, 2),
            other => panic!("expected Corruption on transition, got {other:?}"),
        }
        // And the refused transition wrote nothing: the line count held.
        let after = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(after.lines().count(), 3);
    }

    #[test]
    fn invalid_nonce_hex_refuses_corruption_with_exact_line() {
        let dir = scratch_dir("corrupt-hex");
        let (path, wire) = three_records(&dir);
        let mut lines: Vec<String> = wire.lines().map(|l| l.to_string()).collect();
        // Line 3 stays valid JSON but its nonce_hex breaks the hex law
        // (uppercase first, then wrong length): both are Corruption.
        for (bad_hex, case) in [
            ("AB0123456789abcdef0123456789abcd", "uppercase"),
            ("zz0123456789abcdef0123456789abcd", "non-hex"),
            ("short", "wrong length"),
        ] {
            let mut tampered: serde_json::Value =
                serde_json::from_str(&lines[2]).expect("line parses");
            tampered["nonce_hex"] = serde_json::Value::from(bad_hex);
            lines[2] = serde_json::to_string(&tampered).expect("tampered line serializes");
            fs::write(&path, lines.join("\n")).expect("tampered file writes");
            match NonceLedgerFile::load(&path) {
                Err(PersistError::Corruption { line, reason }) => {
                    assert_eq!(line, 3, "case {case}");
                    assert!(reason.contains("nonce_hex"), "case {case}: {reason}");
                }
                other => panic!("expected Corruption for {case}, got {other:?}"),
            }
        }
    }

    #[test]
    fn empty_line_refuses_corruption_with_exact_line() {
        let dir = scratch_dir("corrupt-blank");
        let (path, wire) = three_records(&dir);
        let mut lines: Vec<String> = wire.lines().map(|l| l.to_string()).collect();
        lines.insert(1, String::new());
        fs::write(&path, lines.join("\n")).expect("tampered file writes");
        match NonceLedgerFile::load(&path) {
            Err(PersistError::Corruption { line, reason }) => {
                assert_eq!(line, 2);
                assert_eq!(reason, "empty line");
            }
            other => panic!("expected Corruption, got {other:?}"),
        }
    }

    // -- prune ----------------------------------------------------------------

    #[test]
    fn prune_rewrites_the_file_without_expired_entries_and_counts() {
        let dir = scratch_dir("prune");
        let path = ledger_in(&dir);
        let mut ledger = NonceLedgerFile::open_or_create(&path).expect("opens");
        ledger.record(KID_A, nonce(1), 100, 600).expect("record");
        ledger.record(KID_B, nonce(2), 300, 600).expect("record");
        ledger.record(KID_A, nonce(3), 500, 600).expect("record");

        // now=900: 900-100=800 (expired), 900-300=600 (boundary, expired),
        // 900-500=400 (kept) — the boundary matches the record() law.
        let evicted = ledger.prune(900, 600).expect("prune");
        assert_eq!(evicted, 2);
        assert_eq!(ledger.len(), 1);
        assert_eq!(ledger.seen(KID_A, &nonce(1)), None);
        assert_eq!(ledger.seen(KID_B, &nonce(2)), None);
        assert_eq!(ledger.seen(KID_A, &nonce(3)), Some(500));
        let wire = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(
            wire.lines().count(),
            1,
            "the file is rewritten, not just the view"
        );
        ledger.verify_integrity().expect("rewritten file verifies");
        assert_eq!(tmp_leftovers(&dir), 0, "rewrite is the atomic flush");

        // A zero-count prune writes nothing.
        let before = fs::read_to_string(&path).expect("durable file reads");
        let evicted = ledger.prune(900, 600).expect("second prune");
        assert_eq!(evicted, 0);
        let after = fs::read_to_string(&path).expect("durable file reads");
        assert_eq!(before, after, "no eviction, no write");
    }

    // -- determinism + every PersistError variant witnessed -------------------

    #[test]
    fn same_records_produce_identical_bytes_from_fresh_ledgers() {
        let dir_a = scratch_dir("det-a");
        let dir_b = scratch_dir("det-b");
        let mut a = NonceLedgerFile::open_or_create(ledger_in(&dir_a)).expect("A");
        let mut b = NonceLedgerFile::open_or_create(ledger_in(&dir_b)).expect("B");
        for (kid, byte, at) in [(KID_A, 1u8, 100u64), (KID_B, 2, 200), (KID_A, 3, 300)] {
            a.record(kid, nonce(byte), at, 60_000).expect("A records");
            b.record(kid, nonce(byte), at, 60_000).expect("B records");
        }
        let wire_a = fs::read_to_string(a.path()).expect("A reads");
        let wire_b = fs::read_to_string(b.path()).expect("B reads");
        assert_eq!(
            wire_a, wire_b,
            "deterministic transitions, byte-identical file"
        );
    }

    #[test]
    fn io_variant_witnessed_by_real_refused_transition() {
        // A target whose parent is a FILE: the filesystem refuses the open
        // with a real OS error, and both constructors refuse Io instead of
        // pretending the path is a ledger.
        let dir = scratch_dir("io");
        let blocker = dir.join("blocker");
        fs::write(&blocker, b"not a directory").expect("blocker writes");
        let path = blocker.join("nonce-journal.jsonl");
        match NonceLedgerFile::open_or_create(&path) {
            Err(PersistError::Io(_)) => {}
            other => panic!("expected Io from open_or_create, got {other:?}"),
        }
        match NonceLedgerFile::load(&path) {
            Err(PersistError::Io(_)) => {}
            other => panic!("expected Io from load, got {other:?}"),
        }
        assert!(
            !path.exists(),
            "no durable file may appear behind an Io refusal"
        );
    }

    #[test]
    fn serialization_variant_witnessed_by_real_conversion_and_defensive_flush_map() {
        // Honest bound, stated in the module docs: over NonceRecord's closed
        // field domain (String + u64) serde_json serialization is total, so
        // no input can fire PersistError::Serialization through the public
        // API — fabricating one would be a fake. The witness is the REAL
        // conversion the flush path carries: a genuine serde_json error
        // funneled through From into the variant, and the flush mapping
        // itself (`serde_json::to_string(record)?`).
        let parse_error = serde_json::from_str::<NonceRecord>("{not json").unwrap_err();
        let mapped = PersistError::from(parse_error);
        assert!(matches!(mapped, PersistError::Serialization(_)));
        assert!(mapped.to_string().starts_with("serialization:"));
    }
}
