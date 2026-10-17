// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Phase D M2: native `affi` port of the excised `ctdd_ledger` CLI verbs.
//
//! ctdd ledger CLI verbs for `affi` (feature `ctdd`).
//!
//! V1 parity contract with the excised `ctdd_ledger` bin (CTT 26.10.11
//! `src/bin/ctdd_ledger.rs`, recovered from HEAD): `affi` owns all
//! filesystem traversal (the CTT ledger domain layer is in-memory only),
//! while every wire/domain operation calls the CTT lib — `validate_receipt`,
//! `chain_verify`, `seal_dossier`, `verify_sealed_file`, `exit_code`,
//! [`chicago_tdd_tools::ledger::LedgerExitCode`], `sealer::encode/decode`.
//! No wire code is vendored here.
//!
//! Exit-code law (same classes as the excised bin): 0 ok; domain refusals
//! carry the [`LedgerExitCode`] class code; strictly-IO failures exit 2
//! (there is deliberately no `LedgerExitCode::Io` variant — mod.rs owns the
//! enum). A sealed dossier failing verification is tamper evidence, class
//! Tampered (13).

/// Process exit code for an IO-classified failure (no `LedgerExitCode::Io`
/// variant exists — the CTT ledger mod.rs owns the enum — so affi maps IO
/// to 2 itself, exactly like the excised bin).
pub const IO_EXIT: u8 = 2;
/// Exit code when a sealed dossier fails verification (`SealVerdict::Invalid`
/// is tamper evidence, so it lands in the Tampered class, 13). Literal:
/// `From<LedgerExitCode>` is not const-callable.
pub const VERIFY_INVALID_EXIT: u8 = 13;

use chicago_tdd_tools::ledger::receipt::{ChainedCourtReceipt, VerifiedCourtReceipt};
use chicago_tdd_tools::ledger::signing::sealer;
use chicago_tdd_tools::ledger::signing::{KeyStore, Signer};
use chicago_tdd_tools::ledger::{
    chain_verify, exit_code, seal_dossier, validate_receipt, verify_sealed_file,
    CourtLedgerBuilder, DossierBuilder, DossierError, DossierRoot, LedgerError, LedgerExitCode,
    SealError, SealVerdict,
};
use chicago_tdd_tools::observability::receipt::Blake3ReceiptEntry;

/// One structured outcome, shared by the human renderer and the tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtddOutcome {
    /// Whether the verb succeeded.
    pub ok: bool,
    /// Refusal kind (`validate`/`chain`/`seal`/`verify`); `None` on success.
    pub refusal: Option<&'static str>,
    /// Refusal detail, normalized to a single line.
    pub detail: Option<String>,
    /// Number of receipts processed (where meaningful).
    pub receipts: usize,
    /// Lowercase hex of the verified/sealed digest (where meaningful).
    pub digest: Option<String>,
    /// Process exit code (already domain-mapped; 0 on success).
    pub code: u8,
}

impl CtddOutcome {
    fn success(receipts: usize, digest: Option<String>) -> Self {
        Self {
            ok: true,
            refusal: None,
            detail: None,
            receipts,
            digest,
            code: 0,
        }
    }

    fn refusal(kind: &'static str, detail: String, code: u8) -> Self {
        Self {
            ok: false,
            refusal: Some(kind),
            detail: Some(detail),
            receipts: 0,
            digest: None,
            code,
        }
    }
}

/// Collapse any error Display into a single-line detail string.
fn one_line(e: impl std::fmt::Display) -> String {
    let mut s = e.to_string();
    if let Some(pos) = s.find(['\n', '\r']) {
        s.truncate(pos);
    }
    s
}

/// Lowercase hex of a 32-byte digest (stdout rendering only).
pub(crate) fn hex_head(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Print the outcome and exit with the domain-mapped code. Success prints a
/// human one-liner to stdout; refusals print `refusal: <kind> <detail>` to
/// stderr.
pub fn render_and_exit(cmd: &str, outcome: &CtddOutcome) {
    if outcome.ok {
        let mut line = format!("{cmd}: ok ({} receipts", outcome.receipts);
        if let Some(d) = &outcome.digest {
            line.push_str(&format!(", digest {d}"));
        }
        line.push(')');
        outln!("{line}");
    } else {
        eprintln!(
            "refusal: {} {}",
            outcome.refusal.unwrap_or("unknown"),
            outcome.detail.as_deref().unwrap_or("unknown")
        );
    }
    if outcome.code != 0 {
        std::process::exit(i32::from(outcome.code));
    }
}

/// The single mapping from a sealing-engine refusal to the process exit code.
/// `SealError` is outside `exit_code(&LedgerError)`'s domain (mod.rs owns
/// that enum), so affi maps it to `AdmissionRefused` (14), exactly like the
/// excised bin.
fn seal_exit(_e: &SealError) -> u8 {
    u8::from(LedgerExitCode::AdmissionRefused)
}

/// Load (or create) the local ML-DSA-65 signing key and wrap it in a domain
/// [`Signer`]. IO/custody failures are IO-classified (exit 2).
pub fn load_signer() -> Result<Signer, String> {
    let key = KeyStore::new()
        .load_or_create()
        .map_err(|e: SealError| one_line(e))?;
    Signer::from_signing_key(key).map_err(one_line)
}

/// [`load_signer`] for verb entry points: custody failures print and exit 2.
pub fn load_signer_or_exit() -> Signer {
    match load_signer() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("refusal: io {e}");
            std::process::exit(i32::from(IO_EXIT));
        }
    }
}

/// Read from `-`/`None` (stdin) or a file path as raw bytes. All failures are
/// `Err(String)` (IO-classified, exit 2).
fn read_input(path: Option<&str>) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    match path {
        None | Some("-") => {
            use std::io::Read as _;
            let _ = std::io::stdin()
                .read_to_end(&mut buf)
                .map_err(|e| format!("stdin read failed: {e}"))?;
        }
        Some(p) => buf = std::fs::read(p).map_err(|e| format!("{p}: {e}"))?,
    }
    Ok(buf)
}

/// `receipt validate` outcome. Domain refusals are `Ok(CtddOutcome)` with the
/// typed [`LedgerExitCode`] class; only reading the input is `Err` (IO, exit 2).
pub fn validate_outcome(path: Option<&str>) -> Result<CtddOutcome, String> {
    let input = read_input(path)?;
    match validate_receipt(&input) {
        Ok(_verdict) => Ok(CtddOutcome::success(1, None)),
        Err(e) => {
            let code = exit_code(&e);
            Ok(CtddOutcome::refusal("validate", one_line(&e), code))
        }
    }
}

/// Sorted list of plain files under `dir`.
fn sorted_files(dir: &str) -> Result<Vec<std::path::PathBuf>, String> {
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{dir}: {e}"))?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    paths.sort();
    Ok(paths)
}

/// Read every receipt record file in `dir` (sorted by file name), verify each
/// 352-byte record, and collect them as chain-replayable entries. An empty
/// directory yields an empty vector, which `chain_verify` refuses as
/// `TooShort` (class MalformedReceipt, 10).
fn read_records(dir: &str) -> Result<Vec<ChainRecord>, String> {
    let mut records = Vec::new();
    for p in sorted_files(dir)? {
        let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes)
            .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
        records.push(ChainRecord::of(&verified));
    }
    Ok(records)
}

/// `receipt chain-verify` outcome.
pub fn chain_verify_outcome(dir: &str) -> Result<CtddOutcome, String> {
    let records = read_records(dir)?;
    match chain_verify(&records) {
        Ok(report) => Ok(CtddOutcome::success(
            report.count,
            Some(hex_head(&report.head)),
        )),
        Err(e) => {
            let code = exit_code(&e);
            Ok(CtddOutcome::refusal("chain", one_line(&e), code))
        }
    }
}

/// Chain-replayable view of a verified 352-byte record.
///
/// The domain's [`ChainedCourtReceipt`] has no public decode constructor, so
/// this reconstructs the exact `Blake3ReceiptEntry` projection the domain
/// uses (content = record[..304], stored hash = record[320..352], prev = the
/// receipt's parent hash) from a [`VerifiedCourtReceipt`] — same bytes, same
/// law, no second hash implementation (replay still runs through the domain
/// `chain_verify`).
struct ChainRecord {
    prev: [u8; 32],
    content: [u8; 304],
    stored: [u8; 32],
}

impl ChainRecord {
    fn of(verified: &VerifiedCourtReceipt) -> Self {
        let record = verified.as_bytes();
        // The typed accessor is the lawful surface for the parent link.
        let prev = verified.receipt().parent_receipt_hash;
        let mut content = [0u8; 304];
        content.copy_from_slice(&record[..304]);
        let mut stored = [0u8; 32];
        stored.copy_from_slice(&record[320..352]);
        ChainRecord {
            prev,
            content,
            stored,
        }
    }
}

impl Blake3ReceiptEntry for ChainRecord {
    fn prev_hash(&self) -> [u8; 32] {
        self.prev
    }
    fn content_bytes(&self) -> Vec<u8> {
        self.content.to_vec()
    }
    fn stored_hash(&self) -> [u8; 32] {
        self.stored
    }
}

/// Walk a dossier directory: verify each record, group it under its court
/// (the record's validated name field), replay each court's chain through the
/// domain [`CourtLedgerBuilder`], and seal the Merkle root.
pub fn build_dossier_root(dir: &str) -> Result<DossierRoot, String> {
    let mut courts: Vec<CourtLedgerBuilder> = Vec::new();
    for p in sorted_files(dir)? {
        let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let verified = VerifiedCourtReceipt::from_bytes_verified(&bytes)
            .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
        let receipt = verified.receipt();
        let court = receipt.name.clone();
        let chained = ChainedCourtReceipt::from_receipt(&receipt)
            .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
        match courts.iter_mut().find(|b| b.court_id() == court) {
            Some(b) => b
                .append(chained)
                .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?,
            None => {
                let mut b = CourtLedgerBuilder::new(court);
                b.append(chained)
                    .map_err(|e: LedgerError| format!("{}: {}", p.display(), one_line(e)))?;
                courts.push(b);
            }
        }
    }
    let mut dossier = DossierBuilder::new();
    for b in &courts {
        dossier
            .add_court(b.court_id().to_string(), b.head())
            .map_err(|e: DossierError| one_line(e))?;
    }
    dossier.seal().map_err(|e: DossierError| one_line(e))
}

/// Write the sealed-dossier wire bytes to `output`.
fn write_output(path: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| format!("{path}: {e}"))
}

/// `dossier seal` outcome. Seals the Merkle root over the verified records in
/// `dir` with `signer` and writes the 3399-byte sealed wire to `output`.
pub fn seal_outcome(dir: &str, output: &str, signer: &Signer) -> Result<CtddOutcome, String> {
    let root = build_dossier_root(dir)?;
    let sealed = match seal_dossier(&root, signer) {
        Ok(s) => s,
        Err(e) => {
            return Ok(CtddOutcome::refusal("seal", one_line(&e), seal_exit(&e)));
        }
    };
    let bytes = sealer::encode(&sealed);
    write_output(output, &bytes)?;
    Ok(CtddOutcome::success(0, Some(hex_head(&root.0))))
}

/// `dossier verify` outcome: decode the sealed wire file and verify it
/// against `public_key` (fail-closed, subject-binding-before-signature —
/// the law lives in the domain `verify_sealed_file`).
pub fn verify_outcome(file: &str, public_key: &[u8; 1952]) -> Result<CtddOutcome, String> {
    let bytes = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
    let sealed = match sealer::decode(&bytes) {
        Ok(s) => s,
        Err(e) => {
            return Ok(CtddOutcome::refusal("verify", e.to_string(), IO_EXIT));
        }
    };
    match verify_sealed_file(&sealed, public_key) {
        SealVerdict::Valid => Ok(CtddOutcome::success(0, Some(hex_head(&sealed.root)))),
        SealVerdict::Invalid => Ok(CtddOutcome::refusal(
            "verify",
            format!("sealed dossier {} is invalid", hex_head(&sealed.root)),
            VERIFY_INVALID_EXIT,
        )),
    }
}
