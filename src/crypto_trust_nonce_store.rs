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
// clears. On load, the LAST line for a journal key wins — a restamp appended
// by a previous process supersedes its earlier record.
//
// ATOMICITY LAW (mirrors `crypto_trust_journal_persist` exactly): `prune`
// never writes the live path in place. The next file image is written to a
// same-directory tmp sibling `.<name>.tmp-<pid>-<blake3(time, pid, path)
// [..16]>` (same filesystem, so rename is atomic; unique per attempt, so two
// processes never stage into one temporary and a pre-placed name cannot be
// predicted), created with `create_new(true)` — a pre-placed symlink (or any
// leftover) at the staged path is REFUSED at the open, never followed, so the
// arbitrary-file-clobber class is closed structurally — synced to disk with
// `sync_all` BEFORE the rename, then renamed over the live path (a failed
// rename removes the temporary best-effort). A reader on the other side of
// the rename therefore never observes a partial file, and a crash mid-write
// leaves the previous image intact and at most an orphaned tmp file. A
// nanos-clock tie producing a duplicate staging name is a typed refusal
// (fail-closed), never a clobber.
//
// LOCK LAW (bounded, honest, std-only; mirrors `crypto_trust_journal_persist`
// exactly): `record` and `prune` are read-modify-write transitions (refresh →
// check → write). Without a cross-process lock, two handles can refresh the
// same image, both admit the same nonce, and replay evidence is doubled
// instead of refused. Every mutating transition therefore holds a lock file
// (`<name>.lock`, created with `create_new(true)`) across the WHOLE
// refresh→check→write span and removes it on drop.
//
// Why not `flock`: the plane's admitted dependency closure takes no new
// crates (fs2/libc would be new deps) and std exposes no flock. The O_EXCL
// lock file is the honest std-only equivalent, with two real tradeoffs, both
// handled and neither hidden: (1) the kernel does not reclaim the lock on
// process death — the holder stamps `acquired=<unix>` + pid into the file,
// and an acquirer that finds the stamp (or an unparsable/missing stamp,
// judged by mtime age) older than [`LOCK_STALE_SECONDS`] steals the lock. A
// live holder paused longer than that inside the critical section can
// therefore be stolen from — acknowledged: the section is microseconds, the
// window is 5 seconds, and a stolen holder's drop removes the file only if
// its own token is still on it, so a steal never pulls the lock out from
// under the new holder. Staleness is age-based, not pid-liveness-based (no
// portable pid-liveness check in std). (2) Acquisition is a bounded retry
// loop ([`LOCK_TIMEOUT`]): past the deadline the transition is refused
// [`NonceStoreError::Lock`] with nothing written; retry is lawful.
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
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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
    /// The cross-process transition lock could not be acquired within the
    /// bounded wait ([`LOCK_TIMEOUT`]): another holder is alive
    /// mid-transition. Nothing was read as fresh, nothing was written;
    /// retrying the record is lawful. Distinct from [`NonceStoreError::Io`] —
    /// the journal itself is fine, contention refused the transition.
    #[error("lock: {0}")]
    Lock(String),
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

/// The tmp sibling used by the atomic prune rewrite: same directory as the
/// live path (same filesystem, so `rename` is atomic), name
/// `.<name>.tmp-<pid>-<blake3(time, pid, path)[..16]>`. Unique per attempt —
/// two processes (or two attempts of one process) never stage into the same
/// temporary, and the name is not predictable in advance, so an attacker
/// cannot pre-plant the path the next rewrite will stage at. A nanos-clock
/// tie duplicating a name is a typed refusal at the `create_new(true)` open
/// (fail-closed), never a clobber.
fn tmp_sibling(path: &Path) -> PathBuf {
    let name = path.file_name().map_or_else(
        || "nonces.jsonl".to_string(),
        |n| n.to_string_lossy().to_string(),
    );
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

/// Seconds after which a lock file's stamp (or its mtime, for an
/// unparsable/missing stamp) marks its holder dead and the lock stealable.
pub(crate) const LOCK_STALE_SECONDS: u64 = 5;

/// Bounded total wait for lock acquisition; past it the transition is
/// refused [`NonceStoreError::Lock`] with nothing written.
pub(crate) const LOCK_TIMEOUT: Duration = Duration::from_secs(LOCK_STALE_SECONDS);

/// The `<name>.lock` path guarding mutations of the journal at `path`.
fn lock_path(path: &Path) -> PathBuf {
    let name = path.file_name().map_or_else(
        || "nonces.jsonl".to_string(),
        |n| n.to_string_lossy().to_string(),
    );
    path.with_file_name(format!("{name}.lock"))
}

/// Seconds since the Unix epoch (clock fallback 0 is honest here: it only
/// makes a stamp look maximally old, i.e. stealable, for a broken clock).
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The staleness decision for one observed lock file, pure so every branch
/// is witnessed by the court: `body` is the file content (None = unreadable),
/// `mtime_age` its age (None = unstatable). A parsable `acquired=<unix>`
/// stamp decides by stamp age; anything unparsable falls back to mtime age
/// and fails CLOSED — no readable age, no steal.
fn lock_is_stale(body: Option<&str>, mtime_age: Option<Duration>, now_unix: u64) -> bool {
    if let Some(text) = body {
        if let Some(acquired) = text
            .lines()
            .find_map(|line| line.strip_prefix("acquired="))
            .and_then(|value| value.trim().parse::<u64>().ok())
        {
            return now_unix.saturating_sub(acquired) >= LOCK_STALE_SECONDS;
        }
    }
    mtime_age.is_some_and(|age| age >= Duration::from_secs(LOCK_STALE_SECONDS))
}

/// A held transition lock. Drop removes the lock file — but only if the
/// holder's own token is still on it: a stale-steal may have handed the lock
/// to another acquirer while this holder was paused past the staleness
/// window, and that steal must never be undone by the old holder's exit.
#[derive(Debug)]
struct FileLock {
    path: PathBuf,
    token: String,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Ok(current) = std::fs::read_to_string(&self.path) {
            if current == self.token {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

/// Acquire the transition lock for the journal at `path` with the standard
/// 5s bounded wait. O_EXCL `create_new(true)` is the whole mutual-exclusion
/// mechanism: exactly one acquirer materializes the file; everyone else
/// finds it existing, judges it live or stale, and either waits its bounded
/// turn or steals a provably dead one.
fn acquire_lock(path: &Path) -> Result<FileLock, NonceStoreError> {
    acquire_lock_within(path, LOCK_TIMEOUT)
}

/// The bounded acquire loop (timeout made explicit so the court can witness
/// both the refusal and the steal without sleeping real seconds).
fn acquire_lock_within(path: &Path, timeout: Duration) -> Result<FileLock, NonceStoreError> {
    let lock = lock_path(path);
    // The lock lives beside the journal, whose directory may not exist yet on
    // a first-ever record (before any write has created it).
    if let Some(dir) = lock.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    let deadline = Instant::now() + timeout;
    let mut backoff = Duration::from_millis(1);
    loop {
        match OpenOptions::new().write(true).create_new(true).open(&lock) {
            Ok(mut file) => {
                let token = format!("acquired={}\npid={}\n", unix_now(), std::process::id());
                file.write_all(token.as_bytes())?;
                return Ok(FileLock { path: lock, token });
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                let body = std::fs::read_to_string(&lock).ok();
                let mtime_age = std::fs::metadata(&lock).ok().and_then(|meta| {
                    meta.modified()
                        .ok()
                        .and_then(|modified| modified.elapsed().ok())
                });
                if lock_is_stale(body.as_deref(), mtime_age, unix_now()) {
                    // Steal. Racing thieves that lose the remove simply
                    // re-enter the loop against the winner's fresh stamp.
                    let _ = std::fs::remove_file(&lock);
                }
            }
            Err(err) => return Err(NonceStoreError::from(err)),
        }
        if Instant::now() >= deadline {
            return Err(NonceStoreError::Lock(format!(
                "transition lock {} not acquired within {timeout:?}",
                lock.display()
            )));
        }
        std::thread::sleep(backoff);
        backoff = (backoff * 2).min(Duration::from_millis(25));
    }
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
    /// Parse the journal file at `path` into the entry index, with the
    /// module's typed refusals. An absent file is the empty index; an
    /// existing file is parsed line-by-line — header checked, LAST
    /// occurrence of a journal key wins (restamps supersede).
    fn read_entries(path: &Path) -> Result<BTreeMap<String, (String, u64)>, NonceStoreError> {
        let mut entries = BTreeMap::new();
        if !path.exists() {
            return Ok(entries);
        }
        let file = File::open(path)?;
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
        Ok(entries)
    }

    /// Open (or adopt) the journal at `path` with replay window
    /// `window_seconds`. An absent file is an empty journal (parent
    /// directories are created on first write); an existing file is parsed
    /// line-by-line with typed refusals — last occurrence of a journal key
    /// wins (restamps supersede).
    pub fn open<P: Into<PathBuf>>(path: P, window_seconds: u64) -> Result<Self, NonceStoreError> {
        let path = path.into();
        let entries = Self::read_entries(&path)?;
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

    /// Record sight of `(kid, nonce)` at instant `at`, DURABLY. The whole
    /// refresh→check→append span runs under the cross-process transition
    /// lock (LOCK LAW): the view is re-read from disk first (the
    /// cross-process sync point — evidence another handle or process
    /// appended is never missed), then the window law runs (strictly-inside
    /// repeat -> [`NonceStoreError::ReplayRejected`], boundary-or-beyond ->
    /// restamp), then the entry line is appended and `sync_data`d. Two
    /// handles racing the same nonce serialize and exactly one admits — the
    /// loser is a typed [`NonceStoreError::ReplayRejected`], never a silent
    /// lost update.
    pub fn record(&mut self, kid: &str, nonce: &[u8; 16], at: u64) -> Result<(), NonceStoreError> {
        let _guard = acquire_lock(&self.path)?;
        self.refresh()?;
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
    /// entries evicted. The view is refreshed from disk first and the whole
    /// refresh→retain→rewrite span runs under the cross-process transition
    /// lock (LOCK LAW), same as [`record`](DiskNonceJournal::record). The
    /// surviving journal is rewritten atomically (unique sibling temp file
    /// staged with `create_new(true)`, `sync_all`, `rename`) — the boundary
    /// matches [`record`](DiskNonceJournal::record): an entry pruned at
    /// `now` is exactly an entry whose re-record at `now` would be admitted.
    pub fn prune(&mut self, now: u64) -> Result<usize, NonceStoreError> {
        let _guard = acquire_lock(&self.path)?;
        self.refresh()?;
        let before = self.entries.len();
        self.entries
            .retain(|_, entry| now.saturating_sub(entry.1) < self.window_seconds);
        let evicted = before - self.entries.len();
        if evicted == 0 {
            return Ok(0);
        }
        let tmp = tmp_sibling(&self.path);
        self.rewrite_to(&tmp)?;
        Ok(evicted)
    }

    /// Re-read the durable file into the in-memory index — the cross-process
    /// sync point every mutating transition runs first, so evidence another
    /// handle or process appended is never missed. A file that does not
    /// exist (yet) syncs to the empty index.
    fn refresh(&mut self) -> Result<(), NonceStoreError> {
        self.entries = Self::read_entries(&self.path)?;
        Ok(())
    }

    /// The atomic rewrite law with the staging path made explicit — the
    /// exact code [`prune`](DiskNonceJournal::prune) runs, with only the
    /// (unpredictable) staging name lifted into an argument so the
    /// symlink-attack court can plant one and watch the refusal fire. The
    /// next file image is written to `tmp` with `create_new(true)` —
    /// refusing, never following, anything already at the staged path, so a
    /// pre-placed symlink cannot redirect the write — synced with
    /// `sync_all`, and renamed over the live path (a failed rename removes
    /// the temporary best-effort).
    fn rewrite_to(&self, tmp: &Path) -> Result<(), NonceStoreError> {
        if let Some(dir) = tmp.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir)?;
            }
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(tmp)
            .map_err(|err| {
                NonceStoreError::Io(std::io::Error::new(
                    err.kind(),
                    format!(
                        "stage {}: refused (never follows an existing path): {err}",
                        tmp.display()
                    ),
                ))
            })?;
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
        match std::fs::rename(tmp, &self.path) {
            Ok(()) => Ok(()),
            Err(err) => {
                let _ = std::fs::remove_file(tmp);
                Err(NonceStoreError::Io(err))
            }
        }
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

    /// Count orphaned staging temporaries (`.tmp-*`) and transition locks
    /// (`.lock`) beside the JOURNAL: after any completed transition there
    /// must be none (the atomic rewrite renames its tmp away or fails
    /// typed; the lock guard removes its file on drop). The check lists the
    /// JOURNAL'S OWN PARENT DIRECTORY — the directory the rewrite stages
    /// in, derived from the journal path itself — never some scratch root
    /// the journal only happens to sit under today; a scan of the wrong
    /// directory counts nothing and carries no bits (the vacuous-check
    /// lesson).
    fn tmp_leftovers(journal_path: &Path) -> usize {
        let dir = journal_path.parent().expect("journal has a parent");
        std::fs::read_dir(dir)
            .expect("journal dir reads")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                name.contains(".tmp-") || name.ends_with(".lock")
            })
            .count()
    }

    // -- teeth first: the rendered consts are the graph facts ----------------

    #[test]
    fn rendered_consts_are_graph_facts() {
        // The render-time failure mode this court exists for: an earlier
        // sync failed on an xsd:integer parse and the canonical module
        // shipped with raw double-brace template literals in place of
        // graph facts. A rendered const carrying a placeholder is not a
        // fact — it is an unrendered template.
        for (name, value) in [
            ("DOMAIN_TAG", DOMAIN_TAG),
            ("REPLAY_KEY", REPLAY_KEY),
            ("STORE_FILE", STORE_FILE),
        ] {
            assert!(
                !value.contains(concat!("{", "{")),
                "{name} renders a template placeholder, not a graph fact: {value:?}"
            );
            assert!(!value.trim().is_empty(), "{name} rendered empty");
        }
        // And they are THE graph's facts, pinned so a query regression
        // cannot silently rebind them.
        assert_eq!(DOMAIN_TAG, "affidavit.crypto-trust-plane.v1");
        assert_eq!(DEFAULT_WINDOW_SECONDS, 300);
        assert_eq!(REPLAY_KEY, "kid,nonce");
        assert_eq!(STORE_FILE, ".affi/keys.json");
        assert_eq!(default_journal_path(), PathBuf::from(".affi/nonces.jsonl"));
        // Cross-module graph consistency: every lane rendered the same facts.
        assert_eq!(DOMAIN_TAG, crate::crypto_trust_canonical::DOMAIN_TAG);
        assert_eq!(
            DEFAULT_WINDOW_SECONDS,
            crate::crypto_trust_lifecycle::NONCE_WINDOW_SECONDS
        );
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
        // No orphaned temp files survive — scanned beside the JOURNAL (its
        // own parent), where a leak would actually land.
        assert_eq!(
            tmp_leftovers(&path),
            0,
            "rewrite leaked a staging tmp beside the journal"
        );
        // The detector has teeth: a planted leftover in the journal's own
        // parent IS counted (a scan that cannot see a real leak is vacuous).
        let planted = dir.join(".nonces.jsonl.tmp-999999-deadbeef");
        std::fs::write(&planted, b"orphan").expect("plant leftover");
        assert_eq!(tmp_leftovers(&path), 1, "planted leftover must be seen");
        std::fs::remove_file(&planted).expect("clean up plant");
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
