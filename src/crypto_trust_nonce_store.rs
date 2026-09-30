//! Durable, replay-resistant nonce journal for the affidavit cryptographic trust plane.
//
// Consumed query columns (nonce-store.rq): domain_tag, window, replay_key, store_file.
// Rendered by ggen (affidavit-trust-plane-pack) from the ctp: graph.
// Edit the ontology and re-render; never edit this file by hand.
//
// Capability: the rendered plane's replay journal
// ([`crate::crypto_trust_envelope::NonceJournal`]) is in-memory — replay
// resistance dies with the process. This module persists the SAME window law
// to disk as an append-only JSONL journal, so (kid, nonce) admissions survive
// process restarts: a re-opened journal refuses a replayed nonce exactly as
// the live one would.
//
// Window law (the envelope module's, persisted verbatim): a repeat strictly
// inside the acceptance window (`at - seen_at < window`) is
// [`NonceStoreError::ReplayRejected`]; a repeat at or beyond the boundary
// restamps the entry and is admitted. The journal never prunes implicitly —
// call [`DiskNonceJournal::prune`]; eviction is explicit and counted.
//
// Durability law: `record` appends one JSON line and `sync_data`s it before
// returning, so an admitted nonce is durable the moment the caller's `?`
// clears. `prune` rewrites the surviving entries to a sibling temporary file
// (`.<name>.tmp-<pid>`), `sync_all`s it, and `rename`s it into place (atomic
// within one directory). On load, the LAST line for a journal key wins — a
// restamp appended by a previous process supersedes its earlier record.
//
// Tamper law: the journal is parse-checked line-by-line on open; a mutated,
// truncated, or foreign line surfaces as a typed
// [`NonceStoreError::Corrupt`] (with its 1-based line number) or
// [`NonceStoreError::WrongFormat`] header refusal — never silent acceptance.
// Checksummed at-rest protection for KEY material lives in the key store
// ([`crate::crypto_trust_store`]); the nonce journal's at-rest contract is
// parse-refusal, because every entry is semantically a "seen at" marker whose
// corruption can only over- or under-admit within one window, both decided
// again by the verifier on every replay check.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Trust-plane policy domain tag (ctp:policy-v1 ctp:domainTag), stamped in
/// the journal header. Conformance-tested equal to
/// [`crate::crypto_trust_canonical::DOMAIN_TAG`]: both render the same graph
/// fact.
pub const DOMAIN_TAG: &str = "affidavit.crypto-trust-plane.v1";

/// Default replay acceptance window in seconds (ctp:nonce-policy-v1
/// ctp:windowSeconds). Conformance-tested equal to
/// [`crate::crypto_trust_lifecycle::NONCE_WINDOW_SECONDS`].
pub const DEFAULT_WINDOW_SECONDS: u64 = 300;

/// Replay tuple identity (ctp:nonce-policy-v1 ctp:replayKey). The journal
/// key is exactly this tuple: `kid` + ":" + lowercase hex of the 16-byte
/// nonce.
pub const REPLAY_KEY: &str = "kid,nonce";

/// The graph-bound key store location (ctp:store-v1 ctp:storeFile); the
/// default journal path is DERIVED as its sibling (`<parent>/nonces.jsonl`)
/// so key material and nonce state share one graph-owned home directory.
pub const STORE_FILE: &str = ".affi/keys.json";

/// Journal format identity stamped as the header's `format` field; open
/// refuses any other value ([`NonceStoreError::WrongFormat`]).
pub const JOURNAL_FORMAT: &str = "CTP-NONCE-JOURNAL-v1";

/// The derived default journal location: the sibling of the graph-bound
/// [`STORE_FILE`], e.g. `.affi/keys.json` -> `.affi/nonces.jsonl`.
pub fn default_journal_path() -> PathBuf {
    let store = Path::new(STORE_FILE);
    match store.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join("nonces.jsonl"),
        _ => PathBuf::from("nonces.jsonl"),
    }
}

/// Typed refusal of a durable journal operation; refusal-as-value, never a
/// panic. (Not `PartialEq`: the `Io` variant carries `std::io::Error`, which
/// this toolchain does not hold comparable.)
#[derive(Debug, thiserror::Error)]
pub enum NonceStoreError {
    /// The underlying file system refused.
    #[error("nonce journal io error: {0}")]
    Io(#[from] std::io::Error),
    /// The header names a different format (or is missing one).
    #[error("wrong journal format: expected {expected}, found {found}")]
    WrongFormat {
        /// The only accepted format identity ([`JOURNAL_FORMAT`]).
        expected: String,
        /// The refused header's format value (or "missing").
        found: String,
    },
    /// A line could not be parsed as this journal's entry shape.
    #[error("corrupt journal line {line}: {reason}")]
    Corrupt {
        /// 1-based line number (the header is line 1).
        line: usize,
        /// Why the line was refused.
        reason: String,
    },
    /// The (kid, nonce) was already admitted strictly inside the window.
    #[error("nonce replay for key {0}")]
    ReplayRejected(String),
}

/// Lowercase-hex encoding for journal keys and nothing else (the envelope
/// module's law: lowercase hex digits and `:` are disjoint alphabets and the
/// suffix has fixed length 32, so distinct (kid, nonce) pairs can never
/// collide).
fn hex_encode(bytes: &[u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(32);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// The journal key for the graph's replay tuple ([`REPLAY_KEY`] =
/// "kid,nonce"): `kid + ":" + lowercase_hex(nonce)` — the same shape the
/// envelope module's [`crate::crypto_trust_envelope::NonceJournal`] uses.
fn journal_key(kid: &str, nonce: &[u8; 16]) -> String {
    format!("{kid}:{}", hex_encode(nonce))
}

fn parse_nonce_hex(s: &str) -> Result<[u8; 16], String> {
    let lower_hex = |b: u8| b.is_ascii_digit() || matches!(b, b'a'..=b'f');
    if s.len() != 32 || !s.bytes().all(lower_hex) {
        return Err("nonce is not 32 lowercase hex characters".to_string());
    }
    let mut out = [0u8; 16];
    for (i, byte) in out.iter_mut().enumerate() {
        let hi = (s.as_bytes()[2 * i] as char)
            .to_digit(16)
            .ok_or("bad hex")?;
        let lo = (s.as_bytes()[2 * i + 1] as char)
            .to_digit(16)
            .ok_or("bad hex")?;
        *byte = ((hi << 4) | lo) as u8;
    }
    Ok(out)
}

/// A durable, replay-resistant (kid, nonce) journal. The in-memory index
/// mirrors the journal file exactly after [`open`](DiskNonceJournal::open);
/// every admitted `record` is on disk before it is in the index's past.
#[derive(Debug)]
pub struct DiskNonceJournal {
    path: PathBuf,
    entries: BTreeMap<String, (String, u64)>,
    window_seconds: u64,
}

impl DiskNonceJournal {
    /// Open (or adopt) the journal at `path` with replay window
    /// `window_seconds`. An absent file is an empty journal (parent
    /// directories are created on first write); an existing file is parsed
    /// line-by-line with typed refusals — last occurrence of a journal key
    /// wins (restamps supersede).
    pub fn open<P: Into<PathBuf>>(path: P, window_seconds: u64) -> Result<Self, NonceStoreError> {
        let path = path.into();
        let mut entries = BTreeMap::new();
        if path.exists() {
            let file = File::open(&path)?;
            let mut line_no = 0usize;
            for line in BufReader::new(file).lines() {
                line_no += 1;
                let line = line?;
                if line_no == 1 {
                    Self::check_header(&line, line_no)?;
                    continue;
                }
                if line.trim().is_empty() {
                    continue;
                }
                let (key, kid, at) = Self::parse_entry(&line, line_no)?;
                entries.insert(key, (kid, at));
            }
        }
        Ok(DiskNonceJournal {
            path,
            entries,
            window_seconds,
        })
    }

    /// Open the derived [`default_journal_path`] with the graph's
    /// [`DEFAULT_WINDOW_SECONDS`].
    pub fn open_default() -> Result<Self, NonceStoreError> {
        Self::open(default_journal_path(), DEFAULT_WINDOW_SECONDS)
    }

    /// The journal's location.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Distinct (kid, nonce) pairs currently journaled.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no (kid, nonce) pair is journaled.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The instant (kid, nonce) was last recorded, if present.
    pub fn seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> {
        self.entries.get(&journal_key(kid, nonce)).map(|e| e.1)
    }

    /// Record sight of `(kid, nonce)` at instant `at`, DURABLY: the window
    /// law runs first (strictly-inside repeat ->
    /// [`NonceStoreError::ReplayRejected`], boundary-or-beyond -> restamp),
    /// then the entry line is appended and `sync_data`d. Returns only after
    /// the admission is on disk.
    pub fn record(&mut self, kid: &str, nonce: &[u8; 16], at: u64) -> Result<(), NonceStoreError> {
        let key = journal_key(kid, nonce);
        if let Some(entry) = self.entries.get(&key) {
            if at.saturating_sub(entry.1) < self.window_seconds {
                return Err(NonceStoreError::ReplayRejected(kid.to_string()));
            }
        }
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let line = format!(
            "{{\"kid\":{},\"nonce\":\"{}\",\"at\":{}}}\n",
            serde_json::to_string(kid).unwrap_or_else(|_| "\"\"".to_string()),
            hex_encode(nonce),
            at
        );
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(line.as_bytes())?;
        file.sync_data()?;
        self.entries.insert(key, (kid.to_string(), at));
        Ok(())
    }

    /// Evict every entry whose recorded instant lies at or beyond the window
    /// boundary (`now - seen_at >= window_seconds`); returns the number of
    /// entries evicted. The surviving journal is rewritten atomically
    /// (sibling temp file, `sync_all`, `rename`) — the boundary matches
    /// [`record`](DiskNonceJournal::record): an entry pruned at `now` is
    /// exactly an entry whose re-record at `now` would be admitted.
    pub fn prune(&mut self, now: u64) -> Result<usize, NonceStoreError> {
        let before = self.entries.len();
        self.entries
            .retain(|_, entry| now.saturating_sub(entry.1) < self.window_seconds);
        let evicted = before - self.entries.len();
        if evicted == 0 {
            return Ok(0);
        }
        let parent = self
            .path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(&parent)?;
        }
        let tmp = parent.join(format!(
            ".{}.tmp-{}",
            self.path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "nonces.jsonl".to_string()),
            std::process::id()
        ));
        {
            let mut file = File::create(&tmp)?;
            file.write_all(Self::header_line().as_bytes())?;
            file.write_all(b"\n")?;
            for (key, (kid, at)) in &self.entries {
                let nonce_hex = key.rsplit(':').next().unwrap_or("");
                let line = format!(
                    "{{\"kid\":{},\"nonce\":\"{}\",\"at\":{}}}\n",
                    serde_json::to_string(kid).unwrap_or_else(|_| "\"\"".to_string()),
                    nonce_hex,
                    at
                );
                file.write_all(line.as_bytes())?;
            }
            file.sync_all()?;
        }
        std::fs::rename(&tmp, &self.path)?;
        Ok(evicted)
    }

    fn header_line() -> String {
        format!(
            "{{\"format\":\"{}\",\"domain\":\"{}\",\"replay_key\":\"{}\",\"window_seconds\":{}}}",
            JOURNAL_FORMAT, DOMAIN_TAG, REPLAY_KEY, DEFAULT_WINDOW_SECONDS
        )
    }

    fn check_header(line: &str, line_no: usize) -> Result<(), NonceStoreError> {
        let v: Value = serde_json::from_str(line).map_err(|e| NonceStoreError::Corrupt {
            line: line_no,
            reason: e.to_string(),
        })?;
        let found = v
            .get("format")
            .and_then(Value::as_str)
            .unwrap_or("missing")
            .to_string();
        if found != JOURNAL_FORMAT {
            return Err(NonceStoreError::WrongFormat {
                expected: JOURNAL_FORMAT.to_string(),
                found,
            });
        }
        Ok(())
    }

    fn parse_entry(line: &str, line_no: usize) -> Result<(String, String, u64), NonceStoreError> {
        let bad = |reason: &str| NonceStoreError::Corrupt {
            line: line_no,
            reason: reason.to_string(),
        };
        let v: Value = serde_json::from_str(line).map_err(|e| bad(&e.to_string()))?;
        let kid = v
            .get("kid")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("kid missing or not a string"))?
            .to_string();
        if kid.is_empty() {
            return Err(bad("kid empty"));
        }
        let nonce_hex = v
            .get("nonce")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("nonce missing or not a string"))?;
        let nonce = parse_nonce_hex(nonce_hex).map_err(|e| bad(&e))?;
        let at = v
            .get("at")
            .and_then(Value::as_u64)
            .ok_or_else(|| bad("at missing or not an integer"))?;
        Ok((journal_key(&kid, &nonce), kid, at))
    }
}

/// Append the header on journal creation; internal helper used by tests and
/// by writers that must materialize an empty-but-valid journal.
impl DiskNonceJournal {
    /// Create a fresh journal file at `path` with the canonical header.
    /// Refuses ([`NonceStoreError::WrongFormat`] via re-open law) nothing —
    /// creation fails only on [`NonceStoreError::Io`]. An existing file is
    /// left untouched; call [`open`](DiskNonceJournal::open) instead.
    pub fn create<P: Into<PathBuf>>(path: P) -> Result<Self, NonceStoreError> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        if !path.exists() {
            let mut file = File::create(&path)?;
            file.write_all(Self::header_line().as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_data()?;
        }
        Self::open(path, DEFAULT_WINDOW_SECONDS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_trust_envelope::{NonceJournal, ENVELOPE_VERSION};
    use std::sync::atomic::{AtomicU64, Ordering};

    /// Unique scratch directory per call (no tempfile dependency; std only).
    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let dir = std::env::temp_dir().join(format!("ctp-nonce-{tag}-{pid}-{n}"));
        std::fs::create_dir_all(&dir).expect("create scratch dir");
        dir
    }

    fn nonce(tag: u8) -> [u8; 16] {
        [tag; 16]
    }

    #[test]
    fn graph_consts_conform_to_the_modules_they_bind() {
        assert_eq!(DOMAIN_TAG, crate::crypto_trust_canonical::DOMAIN_TAG);
        assert_eq!(
            DEFAULT_WINDOW_SECONDS,
            crate::crypto_trust_lifecycle::NONCE_WINDOW_SECONDS
        );
        assert_eq!(REPLAY_KEY, "kid,nonce");
        assert_eq!(default_journal_path(), PathBuf::from(".affi/nonces.jsonl"));
        // The journal rides the envelope version it defends.
        assert_eq!(
            ENVELOPE_VERSION,
            crate::crypto_trust_envelope::ENVELOPE_VERSION
        );
    }

    #[test]
    fn admission_survives_process_restart_and_replays_are_refused_across_open() {
        let dir = temp_dir("restart");
        let path = dir.join("nonces.jsonl");
        {
            let mut journal = DiskNonceJournal::create(&path).expect("create journal");
            journal
                .record("kid-a", &nonce(1), 100)
                .expect("first admission is on disk");
            assert_eq!(journal.len(), 1);
        }
        // "New process": fresh open of the same path.
        let mut journal = DiskNonceJournal::open(&path, DEFAULT_WINDOW_SECONDS).expect("reopen");
        assert_eq!(journal.seen("kid-a", &nonce(1)), Some(100));
        assert!(matches!(
            journal.record("kid-a", &nonce(1), 200),
            Err(NonceStoreError::ReplayRejected(kid)) if kid == "kid-a"
        ));
        journal
            .record("kid-a", &nonce(2), 200)
            .expect("fresh nonce admitted");
    }

    #[test]
    fn disk_journal_implements_the_envelope_modules_window_law_exactly() {
        // Twin check: in-memory NonceJournal and DiskNonceJournal must make
        // the SAME admit/reject decision at every instant for the same
        // (kid, nonce) sequence — the disk journal is the memory journal's
        // durable shadow, not a different law.
        let dir = temp_dir("twin");
        let path = dir.join("nonces.jsonl");
        let mut disk = DiskNonceJournal::create(&path).expect("create");
        let mut memory = NonceJournal::default();
        let window = DEFAULT_WINDOW_SECONDS;
        for (at, expect_admit) in [(100, true), (100 + window - 1, false), (100 + window, true)] {
            let disk_decision = disk.record("kid-twin", &nonce(3), at).is_ok();
            let memory_decision = memory.record("kid-twin", nonce(3), at, window).is_ok();
            assert_eq!(
                disk_decision, expect_admit,
                "disk decision diverged at at={at}"
            );
            assert_eq!(
                memory_decision, expect_admit,
                "memory decision diverged at at={at}"
            );
            assert_eq!(disk_decision, memory_decision);
        }
    }

    #[test]
    fn restamped_entry_supersedes_across_restart_last_line_wins() {
        let dir = temp_dir("restamp");
        let path = dir.join("nonces.jsonl");
        let mut journal = DiskNonceJournal::create(&path).expect("create");
        journal.record("kid-r", &nonce(4), 100).expect("first");
        // At-or-beyond boundary: restamps.
        journal
            .record("kid-r", &nonce(4), 100 + DEFAULT_WINDOW_SECONDS)
            .expect("restamp");
        drop(journal);
        let reopened = DiskNonceJournal::open(&path, DEFAULT_WINDOW_SECONDS).expect("reopen");
        assert_eq!(
            reopened.seen("kid-r", &nonce(4)),
            Some(100 + DEFAULT_WINDOW_SECONDS),
            "the later line must supersede on load"
        );
    }

    #[test]
    fn prune_evicts_counted_and_rewrites_atomically() {
        let dir = temp_dir("prune");
        let path = dir.join("nonces.jsonl");
        let mut journal = DiskNonceJournal::create(&path).expect("create");
        journal.record("kid-p", &nonce(5), 100).expect("old");
        journal.record("kid-p", &nonce(6), 1_000).expect("new");
        let evicted = journal.prune(100 + DEFAULT_WINDOW_SECONDS).expect("prune");
        assert_eq!(evicted, 1);
        assert!(journal.seen("kid-p", &nonce(5)).is_none());
        assert_eq!(journal.seen("kid-p", &nonce(6)), Some(1_000));
        drop(journal);
        let reopened = DiskNonceJournal::open(&path, DEFAULT_WINDOW_SECONDS).expect("reopen");
        assert_eq!(reopened.len(), 1, "prune must be durable");
        assert!(reopened.seen("kid-p", &nonce(6)).is_some());
        // No orphaned temp files survive.
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "temp file leaked: {leftovers:?}");
    }

    #[test]
    fn mutated_and_foreign_lines_are_typed_refusals_with_line_numbers() {
        let dir = temp_dir("corrupt");
        let path = dir.join("nonces.jsonl");
        DiskNonceJournal::create(&path).expect("create");
        let mut raw = std::fs::read_to_string(&path).expect("read");

        raw.push_str("{\"kid\":\"kid-c\",\"nonce\":\"zz\",\"at\":1}\n");
        std::fs::write(&path, raw).expect("write corrupt line");

        match DiskNonceJournal::open(&path, DEFAULT_WINDOW_SECONDS) {
            Err(NonceStoreError::Corrupt { line: 2, .. }) => {}
            other => panic!("expected Corrupt at line 2, got {other:?}"),
        }

        let foreign = dir.join("foreign.jsonl");
        std::fs::write(&foreign, "{\"format\":\"SOME-OTHER-v9\"}\n").expect("write foreign");
        assert!(matches!(
            DiskNonceJournal::open(&foreign, DEFAULT_WINDOW_SECONDS),
            Err(NonceStoreError::WrongFormat { .. })
        ));
    }

    #[test]
    fn absent_file_is_an_empty_journal_and_nested_parents_are_created() {
        let dir = temp_dir("absent");
        let path = dir.join("nested").join("nonces.jsonl");
        let mut journal =
            DiskNonceJournal::open(&path, DEFAULT_WINDOW_SECONDS).expect("open absent");
        assert!(journal.is_empty());
        // First write creates the missing parent directories.
        journal
            .record("kid-n", &nonce(7), 1)
            .expect("record into nested dir");
        assert!(path.exists());
    }
}
