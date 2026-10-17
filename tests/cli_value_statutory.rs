//! Court: statutory verbs over real receipt chains — `legal-hold status`,
//! `restitution calculate`, `certificate fre-902`.
//!
//! Proves the falsifiability fixes (CLI_VACUITY_AUDIT_26_10_17 defects 4-6):
//! every figure each verb emits is derived from actual chain bytes.
//!
//! Law under test:
//! - `legal-hold status`: on real chains the FRCP 37(e) audit derives
//!   `chains`/`events_preserved` from the loaded receipts; a tampered chain
//!   is spoliation evidence (REJECT exit 2), and an empty store is the typed
//!   refusal REFUSED_NO_CHAINS (exit 224).
//! - `restitution calculate`: azure/aws/gcp over the SAME chain yield three
//!   distinct figures, each exactly rate x compute-events; an unknown
//!   schedule is REFUSED_UNKNOWN_TARIFF_SCHEDULE (exit 224).
//! - `certificate fre-902`: certifies only over verifiable chains, embedding
//!   the actual chain hashes; a tampered chain is REFUSED_CHAIN_UNLOADABLE
//!   (exit 224); zero chains is REFUSED_NO_CHAINS (exit 224).
//!
//! Chicago-style: real `affi` subprocesses, real files, real exit codes.
//! No mocks.
//!
//! Lane: write lane, owns exactly `tests/cli_value_statutory.rs`.
//
// value-matrix: (legal-hold, status)
// value-matrix: (restitution, calculate)
// value-matrix: (certificate, fre-902)
//
// value-matrix: harness — tests/common absent for this lane; harness defined locally.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Stable process-exit codes (src/diag.rs contract; 8-bit POSIX status).
mod xc {
    pub const OK: i32 = 0;
    /// REJECT-class verdict (spoliation / tamper-evidence failure).
    pub const REJECT: i32 = 2;
    /// 1504 CliExecutionError -> 1504 % 256 = 224 (typed refusal carrier).
    pub const CLI_EXEC_ERROR: i32 = 1504 % 256; // 224
}

/// Type names for the refusal taxonomy asserted verbatim.
const REFUSED_NO_CHAINS: &str = "REFUSED_NO_CHAINS";
const REFUSED_CHAIN_UNLOADABLE: &str = "REFUSED_CHAIN_UNLOADABLE";
const REFUSED_UNKNOWN_TARIFF_SCHEDULE: &str = "REFUSED_UNKNOWN_TARIFF_SCHEDULE";

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd.env("AFFI_RECEIPTS_PATH", "receipts");
    cmd
}

fn run_std(dir: &TempDir, args: &[&str]) -> (i32, String, String) {
    let out = StdCommand::new(env!("CARGO_BIN_EXE_affi"))
        .current_dir(dir.path())
        .env("AFFI_RECEIPTS_PATH", "receipts")
        .args(args)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Emit `n` events into the working chain — real subprocesses. Event types
/// are chosen so the first `compute_n` are compute-class (build|compute|ci).
fn emit_events(dir: &TempDir, specs: &[(&str, &str)]) {
    for (i, (ty, obj)) in specs.iter().enumerate() {
        affi(dir)
            .args([
                "receipt",
                "emit",
                "--type",
                ty,
                "--object",
                obj,
                "--payload",
                "-",
            ])
            .write_stdin(format!("statutory court payload {i}"))
            .assert()
            .success();
    }
}

/// Assemble the working chain into `receipts/<name>` and witness ACCEPT.
fn assemble_chain(dir: &TempDir, name: &str) {
    fs::create_dir_all(dir.path().join("receipts")).expect("create receipts dir");
    affi(dir)
        .args(["receipt", "assemble", "--out", &format!("receipts/{name}")])
        .assert()
        .success()
        .stdout(predicate::str::contains("assembled receipt ->"));
    // assemble does not clear the working chain; reset it so the next chain
    // holds only its own events (event-count law below depends on this).
    let w = dir.path().join(".affi/working.json");
    if w.exists() {
        fs::remove_file(&w).expect("reset working chain");
    }
    affi(dir)
        .args(["receipt", "verify", &format!("receipts/{name}")])
        .assert()
        .success()
        .stderr(predicate::str::contains("verdict: ACCEPT"));
}

/// The BLAKE3 chain hash of a receipt file, read from its own bytes.
fn chain_hash(dir: &TempDir, name: &str) -> String {
    let raw =
        fs::read_to_string(dir.path().join("receipts").join(name)).expect("receipt file exists");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("receipt is JSON");
    v["chain_hash"]
        .as_str()
        .expect("chain_hash is a string")
        .to_string()
}

/// Flip a commitment-covered field of a receipt (event_type of seq 0) so the
/// recomputed rolling BLAKE3 no longer matches the stored chain hash.
fn tamper_event_type(dir: &TempDir, name: &str) {
    let p = dir.path().join("receipts").join(name);
    let raw = fs::read_to_string(&p).expect("receipt exists");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("receipt is JSON");
    v["events"][0]["event_type"] = serde_json::json!("forged-by-court");
    fs::write(&p, serde_json::to_string_pretty(&v).expect("serialize")).expect("rewrite receipt");
}

/// Parse the (pretty) JSON object a verb printed to stdout.
fn stdout_json(stdout: &str) -> serde_json::Value {
    let start = stdout.find('{').expect("JSON object on stdout");
    serde_json::from_str(&stdout[start..]).expect("stdout tail is valid JSON")
}

// ===========================================================================
// (legal-hold, status) — positives, value, negatives
// ===========================================================================

/// Value: on real chains the hold audit derives `chains` and
/// `events_preserved` from the actual receipts on disk, not constants.
#[test]
fn legal_hold_derives_chain_and_event_counts_from_real_chains() {
    let dir = TempDir::new().unwrap();
    emit_events(
        &dir,
        &[
            ("build", "repo:main"),
            ("test", "suite:unit"),
            ("test", "suite:e2e"),
        ],
    );
    assemble_chain(&dir, "chain_a.json");
    emit_events(&dir, &[("build", "repo:other")]);
    assemble_chain(&dir, "chain_b.json");

    let (code, out, err) = run_std(&dir, &["legal-hold", "status", "--matter", "m-42"]);
    assert_eq!(code, xc::OK, "stdout={out} stderr={err}");

    let j = stdout_json(&out);
    assert_eq!(j["chains"], 2, "two assembled chains on disk");
    assert_eq!(j["events_preserved"], 4, "3 + 1 events actually emitted");
    assert_eq!(j["matter"], "m-42");
    assert_eq!(j["status"], "EVIDENTIARY_HOLD_ACTIVE");
    assert_eq!(j["spoliation_detected"], false);
    assert_eq!(j["load_failures"], 0);
    assert_eq!(j["receipts_path"], "receipts");
}

/// Positive + negative in one court: a mixed store (one verifiable chain,
/// one tampered copy) is spoliation — REJECT exit 2 with load_failures > 0,
/// even though a loadable chain exists. A hold audit over tampered evidence
/// cannot read as a success.
#[test]
fn legal_hold_flags_tampered_copy_as_spoliation_exit_2() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main"), ("test", "suite:unit")]);
    assemble_chain(&dir, "chain_good.json");
    emit_events(&dir, &[("build", "repo:copy")]);
    assemble_chain(&dir, "chain_tampered.json");
    tamper_event_type(&dir, "chain_tampered.json");

    let (code, out, err) = run_std(&dir, &["legal-hold", "status"]);
    assert_eq!(
        code,
        xc::REJECT,
        "tampered evidence must REJECT: out={out} err={err}"
    );

    let j = stdout_json(&out);
    assert_eq!(j["spoliation_detected"], true);
    assert_eq!(j["load_failures"], 1);
    assert_eq!(j["ok"], false);
    // The surviving chain is still counted — the audit reports, it never hides.
    assert_eq!(j["chains"], 1);
    assert_eq!(j["events_preserved"], 2);
}

/// Negative: empty receipts dir -> typed REFUSED_NO_CHAINS, exit 224.
/// An evidentiary hold cannot be certified over zero chains.
#[test]
fn legal_hold_empty_store_is_typed_refused_no_chains_224() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("receipts")).expect("create empty receipts dir");

    let (code, out, err) = run_std(&dir, &["legal-hold", "status"]);
    assert_eq!(code, xc::CLI_EXEC_ERROR, "out={out} err={err}");
    assert!(
        err.contains(REFUSED_NO_CHAINS),
        "typed refusal in stderr: {err}"
    );
    assert!(
        !out.contains("EVIDENTIARY_HOLD_ACTIVE"),
        "no fabricated hold status"
    );
}

// ===========================================================================
// (restitution, calculate)
// ===========================================================================

/// Value: azure/aws/gcp over the SAME chain give three DISTINCT figures, each
/// exactly rate x compute-events — no constant-dollar fabrication.
#[test]
fn restitution_three_schedules_one_chain_three_distinct_figures() {
    let dir = TempDir::new().unwrap();
    // 3 compute-class events (build) + 1 non-compute (test) — only 3 count.
    emit_events(
        &dir,
        &[
            ("build", "repo:a"),
            ("build", "repo:b"),
            ("build", "repo:c"),
            ("test", "suite:unit"),
        ],
    );
    assemble_chain(&dir, "chain.json");

    let mut figures = Vec::new();
    for (schedule, rate) in [("azure", 0.12_f64), ("aws", 0.09), ("gcp", 0.15)] {
        let (code, out, err) = run_std(
            &dir,
            &["restitution", "calculate", "--tariff-schedule", schedule],
        );
        assert_eq!(code, xc::OK, "schedule={schedule} out={out} err={err}");
        let j = stdout_json(&out);
        assert_eq!(j["tariff_schedule"], schedule);
        assert_eq!(j["compute_events"], 3, "only compute-class events count");
        let avoided = j["avoided_tariff_usd"].as_f64().expect("numeric figure");
        assert!(
            (avoided - rate * 3.0).abs() < 1e-9,
            "{schedule}: avoided_tariff_usd={avoided} must equal rate {rate} x 3 compute events"
        );
        let gain = j["gain_share_due_usd"]
            .as_f64()
            .expect("numeric gain share");
        assert!(
            (gain - avoided / 2.0).abs() <= 0.005 + 1e-9,
            "{schedule}: gain share {gain} must be avoided {avoided} at 50% (cent-level rounding)"
        );
        figures.push(avoided);
    }
    // azure $0.36, aws $0.27, gcp $0.45 — pairwise distinct.
    assert_eq!(figures.len(), 3);
    assert_ne!(figures[0], figures[1]);
    assert_ne!(figures[0], figures[2]);
    assert_ne!(figures[1], figures[2]);
}

/// Negative: unknown schedule -> typed REFUSED_UNKNOWN_TARIFF_SCHEDULE, 224.
#[test]
fn restitution_unknown_schedule_is_typed_refusal_224() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:a")]);
    assemble_chain(&dir, "chain.json");

    let (code, _out, err) = run_std(
        &dir,
        &[
            "restitution",
            "calculate",
            "--tariff-schedule",
            "oracle-cloud",
        ],
    );
    assert_eq!(code, xc::CLI_EXEC_ERROR);
    assert!(
        err.contains(REFUSED_UNKNOWN_TARIFF_SCHEDULE),
        "typed refusal in stderr: {err}"
    );
}

/// Negative: a chain with zero compute-class events is REFUSED_NO_COMPUTE_EVENTS.
#[test]
fn restitution_zero_compute_events_is_typed_refusal() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("test", "suite:unit"), ("deploy", "env:stage")]);
    assemble_chain(&dir, "chain.json");

    let (code, _out, err) = run_std(
        &dir,
        &["restitution", "calculate", "--tariff-schedule", "aws"],
    );
    assert_eq!(code, xc::CLI_EXEC_ERROR);
    assert!(err.contains("REFUSED_NO_COMPUTE_EVENTS"), "stderr: {err}");
    assert!(
        !err.contains("avoided_tariff"),
        "no figure fabricated over zero input"
    );
}

// ===========================================================================
// (certificate, fre-902)
// ===========================================================================

/// Positive + value: fre-902 certifies over verifiable chains and embeds the
/// ACTUAL chain hashes read back from the receipt bytes on disk.
#[test]
fn fre_902_certifies_verifiable_chains_embedding_actual_chain_hashes() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main"), ("test", "suite:unit")]);
    assemble_chain(&dir, "chain_a.json");
    emit_events(&dir, &[("build", "repo:b")]);
    assemble_chain(&dir, "chain_b.json");

    let hash_a = chain_hash(&dir, "chain_a.json");
    let hash_b = chain_hash(&dir, "chain_b.json");
    assert_ne!(hash_a, hash_b, "distinct chains, distinct hashes");

    let (code, out, err) = run_std(&dir, &["certificate", "fre-902"]);
    assert_eq!(code, xc::OK, "out={out} err={err}");
    assert!(out.contains("FEDERAL RULE OF EVIDENCE 902(14)"));
    let j = stdout_json(&out);
    assert_eq!(j["status"], "SELF_AUTHENTICATING_FRE_902_VALID");
    assert_eq!(j["chains_certified"], 2);
    assert_eq!(j["events_certified"], 3);
    // The affidavit embeds the actual chain hashes, not placeholders.
    assert!(
        out.contains(&hash_a) && out.contains(&hash_b),
        "affidavit must embed real chain hashes {hash_a} / {hash_b}:\n{out}"
    );
}

/// Negative: tampered chain -> typed REFUSED_CHAIN_UNLOADABLE, 224.
#[test]
fn fre_902_tampered_chain_is_typed_refused_chain_unloadable_224() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main"), ("test", "suite:unit")]);
    assemble_chain(&dir, "chain.json");
    tamper_event_type(&dir, "chain.json");

    let (code, out, err) = run_std(&dir, &["certificate", "fre-902"]);
    assert_eq!(code, xc::CLI_EXEC_ERROR, "out={out} err={err}");
    assert!(err.contains(REFUSED_CHAIN_UNLOADABLE), "stderr: {err}");
    assert!(
        !out.contains("SELF_AUTHENTICATING"),
        "no affidavit over tampered evidence"
    );
}

/// Negative: empty receipts dir -> typed REFUSED_NO_CHAINS, 224.
#[test]
fn fre_902_empty_store_is_typed_refused_no_chains_224() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("receipts")).expect("create empty receipts dir");

    let (code, _out, err) = run_std(&dir, &["certificate", "fre-902"]);
    assert_eq!(code, xc::CLI_EXEC_ERROR);
    assert!(err.contains(REFUSED_NO_CHAINS), "stderr: {err}");
}

/// Negative: a loadable chain the verifier pipeline REJECTs is
/// REFUSED_CHAIN_REJECTED, 224 — never a certificate.
#[test]
fn fre_902_verifier_rejected_chain_is_typed_refusal_224() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main"), ("test", "suite:unit")]);
    assemble_chain(&dir, "chain.json");

    // Rewrite the receipt with a broken chain hash: parseable JSON, wrong BLAKE3.
    let p = dir.path().join("receipts/chain.json");
    let raw = fs::read_to_string(&p).expect("receipt exists");
    let mut v: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    let ch = v["chain_hash"].as_str().unwrap().to_string();
    // flip the first hex digit (chain hashes are lowercase; '0'/'1' both in-alphabet)
    let flipped = if ch.starts_with('0') {
        format!("1{}", ch.get(1..).unwrap_or(""))
    } else {
        format!("{}1{}", &ch[..1], ch.get(1..).unwrap_or(""))
    };
    v["chain_hash"] = serde_json::json!(flipped);
    fs::write(&p, serde_json::to_string_pretty(&v).expect("serialize")).expect("rewrite");

    let (code, out, err) = run_std(&dir, &["certificate", "fre-902"]);
    assert_eq!(code, xc::CLI_EXEC_ERROR, "out={out} err={err}");
    // Disjunctive typed law: the recomputing chain law refuses it at load
    // (REFUSED_CHAIN_UNLOADABLE) or, failing that, the verifier pipeline
    // refuses it (REFUSED_CHAIN_REJECTED). Either way: typed, 224, no
    // affidavit — never a certificate over a broken chain.
    let typed = err.contains("REFUSED_CHAIN_UNLOADABLE") || err.contains("REFUSED_CHAIN_REJECTED");
    assert!(typed, "typed chain-law refusal in stderr: {err}");
    assert!(
        !out.contains("SELF_AUTHENTICATING"),
        "no affidavit over rejected chain"
    );
}
