// Copyright (c) 2024 Sean Chatman
// SPDX-License-Identifier: MIT OR Apache-2.0
//
//! Acceptance courts for the Phase D M2 port of the excised `ctdd_ledger`
//! CLI verbs as native `affi` verbs (`receipt validate`, `receipt
//! chain-verify`, `dossier seal`, `dossier verify`).
//!
//! Chicago discipline: real 352-byte CTRT court records written to a real
//! tempdir, real deterministic ML-DSA-65 keys (`Signer::from_seed` — no
//! mocks), and assertions on the same [`CtddOutcome`] state the verb layer
//! renders and exits on. The verb wrappers themselves only add rendering +
//! `std::process::exit`, so asserting on the outcome functions is asserting
//! on the full domain path.
//!
//! Run: `cargo test --features ctdd --test ctdd_cli_tests`

#![cfg(feature = "ctdd")]
#![allow(missing_docs)]
#![allow(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use affidavit::ctdd_cli::{
    build_dossier_root, chain_verify_outcome, seal_outcome, validate_outcome, verify_outcome,
    IO_EXIT, VERIFY_INVALID_EXIT,
};
use chicago_tdd_tools::ledger::receipt::{
    ChainedCourtReceipt, CourtOutcome, CourtReceiptV1, ReceiptMetrics, ThermalClass,
};
use chicago_tdd_tools::ledger::signing::sealer::WIRE_LEN;
use chicago_tdd_tools::ledger::signing::signer::Signer;
use chicago_tdd_tools::ledger::LedgerExitCode;

/// Deterministic signer per seed byte (real ML-DSA-65 key, no mocks).
fn signer(seed: u8) -> Signer {
    Signer::from_seed([seed; 32]).expect("deterministic signer from seed")
}

/// One valid 352-byte chained record for `court_id` (Pass, budget met).
fn receipt_bytes(court_id: &str, seed: u8) -> [u8; 352] {
    let receipt = CourtReceiptV1::new(
        subject(seed),
        [0u8; 32],
        ReceiptMetrics {
            timestamp_ms: 1_700_000_000_000,
            run_id: u64::from(seed),
            total_ticks: 9_999,
            wall_ms: 7,
            thermal: ThermalClass::Warm,
            outcome: CourtOutcome::Pass,
            budget_met: true,
        },
        [0u8; 32],
        metadata(seed),
        [0u8; 32],
        court_id,
    )
    .expect("well-formed receipt");
    ChainedCourtReceipt::from_receipt(&receipt)
        .expect("test court ids fit NAME_MAX")
        .encode()
}

fn subject(seed: u8) -> [u8; 32] {
    let mut s = [0u8; 32];
    s[0] = seed;
    s
}

fn metadata(seed: u8) -> [u8; 32] {
    let mut m = [0u8; 32];
    m[31] = seed;
    m
}

/// Court ids `court-00`..`court-19` (20 courts, N01..N20 shape).
fn court_names() -> Vec<String> {
    (0..20u8).map(|i| format!("court-{i:02}")).collect()
}

/// Write one 352-byte record per court into a tempdir receipts dir, sorted
/// or reversed; only write order varies, so file-order invariance of the
/// seal walk is observable without changing content.
fn write_receipts_dir(dir: &std::path::Path, reversed: bool) {
    std::fs::create_dir_all(dir).expect("mkdir");
    let names = court_names();
    let ordered: Vec<(usize, &String)> = if reversed {
        names.iter().enumerate().rev().collect()
    } else {
        names.iter().enumerate().collect()
    };
    for (i, name) in ordered {
        let path = dir.join(format!("{i:02}.ctrt"));
        std::fs::write(
            &path,
            receipt_bytes(name, u8::try_from(i).expect("fits u8")),
        )
        .expect("write");
    }
}

fn tempdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "affi-ctdd-m2-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&d).expect("mkdir");
    d
}

// ─── receipt validate ────────────────────────────────────────────────────────

/// A valid 352-byte record validates: ok, code 0, one receipt.
#[test]
fn validate_accepts_a_valid_record() {
    let dir = tempdir("validate-ok");
    let path = dir.join("a.ctrt");
    std::fs::write(&path, receipt_bytes("court-00", 1)).expect("write");
    let outcome = validate_outcome(Some(path.to_str().expect("utf8"))).expect("not io");
    assert!(outcome.ok);
    assert_eq!(outcome.code, 0);
    assert_eq!(outcome.receipts, 1);
    assert_eq!(outcome.refusal, None);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A truncated record is a MalformedReceipt refusal, class 10 — not an IO
/// accident.
#[test]
fn validate_refuses_a_truncated_record_as_malformed() {
    let dir = tempdir("validate-short");
    let path = dir.join("a.ctrt");
    let full = receipt_bytes("court-00", 1);
    std::fs::write(&path, &full[..300]).expect("write");
    let outcome = validate_outcome(Some(path.to_str().expect("utf8"))).expect("not io");
    assert!(!outcome.ok);
    assert_eq!(outcome.refusal, Some("validate"));
    assert_eq!(outcome.code, u8::from(LedgerExitCode::MalformedReceipt));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A missing input path is IO-classified (Err), exit 2 — the domain layer
/// never sees it.
#[test]
fn validate_missing_file_is_io_classified() {
    let dir = tempdir("validate-missing");
    let path = dir.join("absent.ctrt");
    let err = validate_outcome(Some(path.to_str().expect("utf8"))).expect_err("io-classified");
    assert!(err.contains("absent.ctrt"));
    assert_eq!(IO_EXIT, 2);
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── receipt chain-verify ────────────────────────────────────────────────────

/// Three records from one court replay clean: count 3, head = last stored
/// hash.
#[test]
fn chain_verify_replays_a_clean_chain() {
    let dir = tempdir("chain-ok");
    let mut prev = [0u8; 32];
    for i in 0..3u8 {
        let receipt = CourtReceiptV1::new(
            subject(i + 1),
            prev,
            ReceiptMetrics {
                timestamp_ms: 1_700_000_000_000 + u64::from(i),
                run_id: u64::from(i),
                total_ticks: 9_999,
                wall_ms: 7,
                thermal: ThermalClass::Warm,
                outcome: CourtOutcome::Pass,
                budget_met: true,
            },
            [0u8; 32],
            metadata(i + 1),
            [0u8; 32],
            "court-00",
        )
        .expect("well-formed receipt");
        let chained = ChainedCourtReceipt::from_receipt(&receipt).expect("fits");
        prev = chained.encode()[320..352].try_into().expect("32");
        std::fs::write(dir.join(format!("{i}.ctrt")), chained.encode()).expect("write");
    }
    let outcome = chain_verify_outcome(dir.to_str().expect("utf8")).expect("not io");
    assert!(outcome.ok, "detail: {:?}", outcome.detail);
    assert_eq!(outcome.code, 0);
    assert_eq!(outcome.receipts, 3);
    // Head equals the stored hash of the last record.
    let last = std::fs::read(dir.join("2.ctrt")).expect("read");
    assert_eq!(
        outcome.digest.as_deref(),
        Some(hex_of(&last[320..352].try_into().expect("32"))).as_deref()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// An empty directory is a chain refusal (`TooShort`), not a silent success.
#[test]
fn chain_verify_refuses_an_empty_directory() {
    let dir = tempdir("chain-empty");
    let outcome = chain_verify_outcome(dir.to_str().expect("utf8")).expect("not io");
    assert!(!outcome.ok);
    assert_eq!(outcome.code, u8::from(LedgerExitCode::MalformedReceipt));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Tampering with a record's content is caught at record verification during
/// the directory walk: a typed Tampered refusal naming the file, not a
/// silent chain success. (Same law as the excised bin: per-record verify
/// failures in `read_records` are IO-classified — the chain refusals proper
/// are forward/prev-hash mismatches on otherwise-valid records.)
#[test]
fn chain_verify_detects_a_broken_link() {
    let dir = tempdir("chain-broken");
    write_receipts_dir(&dir, false);
    let path = dir.join("05.ctrt");
    let mut bytes = std::fs::read(&path).expect("read");
    bytes[200] ^= 0xFF;
    std::fs::write(&path, &bytes).expect("write");
    let err = chain_verify_outcome(dir.to_str().expect("utf8")).expect_err("refused");
    assert!(err.contains("05.ctrt"), "detail: {err}");
    assert!(err.contains("digest mismatch"), "detail: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── dossier seal / verify ───────────────────────────────────────────────────

/// Sealing 20 courts writes a fixed-length sealed wire file and verifying it
/// against the same signer's public key succeeds, returning the same root
/// digest the seal reported.
#[test]
fn seal_then_verify_roundtrip_succeeds() {
    let dir = tempdir("seal-ok");
    write_receipts_dir(&dir, false);
    let out = dir.join("sealed.bin");
    let s = signer(7);
    let seal =
        seal_outcome(dir.to_str().expect("utf8"), out.to_str().expect("utf8"), &s).expect("not io");
    assert!(seal.ok, "detail: {:?}", seal.detail);
    assert_eq!(seal.code, 0);
    let wire = std::fs::read(&out).expect("read");
    assert_eq!(wire.len(), WIRE_LEN, "sealed wire must be fixed-length");
    let verify = verify_outcome(out.to_str().expect("utf8"), &s.public_key()).expect("not io");
    assert!(verify.ok, "detail: {:?}", verify.detail);
    assert_eq!(verify.code, 0);
    assert_eq!(
        verify.digest, seal.digest,
        "same dossier root on both sides"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Verifying a sealed dossier against a different key is tamper evidence:
/// Tampered class (13).
#[test]
fn verify_against_a_foreign_key_refuses_tampered() {
    let dir = tempdir("verify-foreign");
    write_receipts_dir(&dir, false);
    let out = dir.join("sealed.bin");
    let seal = seal_outcome(
        dir.to_str().expect("utf8"),
        out.to_str().expect("utf8"),
        &signer(7),
    )
    .expect("not io");
    assert!(seal.ok);
    let verify =
        verify_outcome(out.to_str().expect("utf8"), &signer(9).public_key()).expect("not io");
    assert!(!verify.ok);
    assert_eq!(verify.refusal, Some("verify"));
    assert_eq!(verify.code, VERIFY_INVALID_EXIT);
    assert_eq!(VERIFY_INVALID_EXIT, u8::from(LedgerExitCode::Tampered));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A corrupted sealed wire is undecodable: IO-classified refusal (exit 2).
#[test]
fn verify_a_corrupted_wire_is_io_classified() {
    let dir = tempdir("verify-corrupt");
    write_receipts_dir(&dir, false);
    let out = dir.join("sealed.bin");
    let seal = seal_outcome(
        dir.to_str().expect("utf8"),
        out.to_str().expect("utf8"),
        &signer(7),
    )
    .expect("not io");
    assert!(seal.ok);
    let mut wire = std::fs::read(&out).expect("read");
    // Truncate the wire: the fixed-length decoder refuses before any
    // signature law runs (IO class). In-place corruption lands in the
    // signature/root and is instead tamper evidence (13).
    wire.truncate(WIRE_LEN - 1);
    std::fs::write(&out, &wire).expect("write");
    let outcome =
        verify_outcome(out.to_str().expect("utf8"), &signer(7).public_key()).expect("not io");
    assert!(!outcome.ok);
    assert_eq!(outcome.code, IO_EXIT);
    let _ = std::fs::remove_dir_all(&dir);
}

/// The seal walk groups records by court before Merkle-sealing, so the root
/// is invariant under directory write order (the excised bin sorted by file
/// name; affi keeps that law).
#[test]
fn seal_root_is_invariant_under_file_write_order() {
    let a = tempdir("seal-order-a");
    let b = tempdir("seal-order-b");
    write_receipts_dir(&a, false);
    write_receipts_dir(&b, true);
    let root_a = build_dossier_root(a.to_str().expect("utf8")).expect("seals");
    let root_b = build_dossier_root(b.to_str().expect("utf8")).expect("seals");
    assert_eq!(root_a.0, root_b.0, "root must not depend on readdir order");
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}

/// A directory holding a malformed record refuses the seal walk with a typed
/// MalformedReceipt detail naming the offending file.
#[test]
fn seal_refuses_a_malformed_record_with_typed_detail() {
    let dir = tempdir("seal-bad");
    write_receipts_dir(&dir, false);
    std::fs::write(dir.join("zz.ctrt"), [0u8; 10]).expect("write");
    let err = build_dossier_root(dir.to_str().expect("utf8")).expect_err("refused");
    assert!(err.contains("zz.ctrt"), "detail: {err}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ─── helpers ─────────────────────────────────────────────────────────────────

fn hex_of(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
