//! Court: CLI value matrix for the six core receipt verbs — emit, assemble,
//! verify, show, inspect, stats. Per verb: >=1 positive (real effect on real
//! state), >=1 negative (typed, stable exit), >=1 value assertion (output
//! derives from the actual chain under test).
//!
//! Chicago-style: spawns the real `affi` binary; asserts on real files and
//! real process exit codes. No mocks.
//!
//! Lane: write lane, owns exactly `tests/cli_value_lifecycle_core.rs`.
//
// value-matrix: (receipt, emit)
// value-matrix: (receipt, assemble)
// value-matrix: (receipt, verify)
// value-matrix: (receipt, show)
// value-matrix: (receipt, inspect)
// value-matrix: (receipt, stats)
//
// value-matrix: harness — tests/common/mod.rs absent; harness defined locally.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Stable process-exit codes from `src/diag.rs::exit_codes` (public contract).
mod xc {
    pub const OK: i32 = 0;
    pub const REJECT: i32 = 2;
    /// CLI framework codes 1500..=1509 ride mod 256 on the 8-bit POSIX status:
    /// 1502 (CliArgumentError) -> 222, 1504 (CliExecutionError) -> 224.
    pub const CLI_ARG_ERROR: i32 = 1502 % 256; // 222
    pub const CLI_EXEC_ERROR: i32 = 1504 % 256; // 224
}

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

fn run_std(dir: &TempDir, args: &[&str]) -> (i32, String, String) {
    let out = StdCommand::new(env!("CARGO_BIN_EXE_affi"))
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Emit n events into the working chain in `dir` — real subprocesses.
fn emit_events(dir: &TempDir, events: &[(&str, &str, &str)]) {
    for (ty, obj, payload) in events {
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
            .write_stdin(*payload)
            .assert()
            .success();
    }
}

/// Assemble the working chain in `dir` into `name`; witness it verifies ACCEPT.
fn assemble_verified(dir: &TempDir, name: &str) {
    affi(dir)
        .args(["receipt", "assemble", "--out", name])
        .assert()
        .success()
        .stdout(predicate::str::contains("assembled receipt ->"));
    affi(dir)
        .args(["receipt", "verify", name])
        .assert()
        .success()
        .stderr(predicate::str::contains("verdict: ACCEPT"));
}

/// Parse the working.json event array length.
fn working_len(dir: &TempDir) -> usize {
    let raw = fs::read_to_string(dir.path().join(".affi/working.json"))
        .expect("working.json must exist after emit");
    serde_json::from_str::<serde_json::Value>(&raw)
        .expect("working.json is valid JSON")
        .as_array()
        .expect("working.json is an event array")
        .len()
}

/// Extract the `code` field from a structured JSON diagnostic on stderr.
fn diag_code(stderr: &str) -> u64 {
    let start = stderr
        .find("{\"code\":")
        .or_else(|| stderr.find("{\"code\": "))
        .expect("structured JSON diagnostic on stderr");
    let tail = &stderr[start..];
    let end = tail.find('}').expect("JSON diagnostic terminates");
    let v: serde_json::Value =
        serde_json::from_str(&tail[..=end]).expect("diagnostic is valid JSON");
    v["code"].as_u64().expect("code is numeric")
}

// ===========================================================================
// (receipt, emit)
// ===========================================================================

/// Positive + value: emit has a real effect — working.json grows on disk —
/// and the growth is exactly one event per emit with the requested type.
#[test]
fn emit_grows_working_receipt_by_one_event_per_call() {
    let dir = TempDir::new().unwrap();
    assert!(!dir.path().join(".affi/working.json").exists());

    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "build",
            "--object",
            "repo:main",
            "--payload",
            "-",
        ])
        .write_stdin("first event payload")
        .assert()
        .success()
        .stdout(predicate::str::contains("seq 0"));
    assert_eq!(working_len(&dir), 1, "one emit -> exactly one event");

    affi(&dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "test",
            "--object",
            "suite:unit",
            "--payload",
            "-",
        ])
        .write_stdin("second event payload")
        .assert()
        .success()
        .stdout(predicate::str::contains("seq 1"));
    assert_eq!(working_len(&dir), 2, "second emit appends, not overwrites");

    let raw = fs::read_to_string(dir.path().join(".affi/working.json")).unwrap();
    assert!(
        raw.contains("\"build\"") && raw.contains("\"test\""),
        "working.json must carry both emitted event types"
    );
}

/// Negative: emit without required args is a typed CLI argument error (1502,
/// exit status 222), not a bare 1 or a panic.
#[test]
fn emit_missing_required_args_is_typed_cli_arg_error_222() {
    let dir = TempDir::new().unwrap();
    let (code, _out, err) = run_std(&dir, &["receipt", "emit"]);
    assert_eq!(code, xc::CLI_ARG_ERROR, "stderr was: {err}");
    assert_eq!(
        diag_code(&err),
        1502,
        "full stable code in diagnostic: {err}"
    );
    assert!(
        !dir.path().join(".affi/working.json").exists(),
        "no state written on refusal"
    );
}

// ===========================================================================
// (receipt, assemble)
// ===========================================================================

/// Positive + value: assemble consumes the working chain and produces an
/// immutable receipt file whose content address differs for different chains.
#[test]
fn assemble_produces_verifiable_receipt_from_actual_chain() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main", "payload-A")]);
    emit_events(&dir, &[("test", "suite:unit", "payload-B")]);

    let out = affi(&dir)
        .args(["receipt", "assemble", "--out", "r.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("assembled receipt ->"))
        .stdout(predicate::str::contains("content address:"));
    let _ = out;

    let receipt: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("r.json")).unwrap())
            .expect("assembled receipt is valid JSON");
    assert_eq!(receipt["format_version"], "core/v1");
    assert_eq!(
        receipt["events"].as_array().expect("events array").len(),
        2,
        "receipt carries exactly the assembled chain"
    );
    assert!(
        !receipt["chain_hash"].as_str().unwrap_or("").is_empty(),
        "receipt carries a chain hash"
    );
}

/// Value: two different chains assemble to different content addresses —
/// the address is derived from the chain, not from wall-clock or defaults.
#[test]
fn assemble_content_address_differs_for_different_chains() {
    let d1 = TempDir::new().unwrap();
    let d2 = TempDir::new().unwrap();
    emit_events(&d1, &[("build", "repo:main", "chain-one-payload")]);
    emit_events(&d2, &[("build", "repo:main", "chain-two-payload")]);

    let addr = |d: &TempDir| -> String {
        let (_, stdout, _) = run_std(d, &["receipt", "assemble", "--out", "r.json"]);
        stdout
            .lines()
            .find(|l| l.starts_with("content address:"))
            .expect("content address line")
            .trim_start_matches("content address:")
            .trim()
            .to_string()
    };
    let a1 = addr(&d1);
    let a2 = addr(&d2);
    assert_ne!(a1, a2, "different chains must produce different addresses");
}

/// Negative: assemble with an empty (absent) working chain is a typed refusal,
/// not a bare 1, and names the remedy.
#[test]
fn assemble_empty_chain_is_typed_refusal() {
    let dir = TempDir::new().unwrap();
    let (code, _out, err) = run_std(&dir, &["receipt", "assemble", "--out", "never.json"]);
    assert_ne!(code, 0, "empty assemble must fail");
    assert!(err.contains("nothing to assemble"), "stderr: {err}");
    assert!(
        err.contains("\"code\":"),
        "structured diagnostic present: {err}"
    );
    assert!(
        !dir.path().join("never.json").exists(),
        "refused assemble must not write a receipt"
    );
}

// ===========================================================================
// (receipt, verify)
// ===========================================================================

/// Positive: verify ACCEPTs an honest receipt (exit 0).
#[test]
fn verify_honest_receipt_accepts_exit_0() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main", "honest payload")]);
    assemble_verified(&dir, "honest.json");

    affi(&dir)
        .args(["receipt", "verify", "honest.json"])
        .assert()
        .success() // exit 0
        .stderr(predicate::str::contains("verdict: ACCEPT"))
        .stderr(predicate::str::contains("all stages passed"));
}

/// Negative: verify on a tampered receipt REJECTs with the stable exit code 2
/// and names the failing stage.
#[test]
fn verify_tampered_receipt_rejects_exit_2() {
    let dir = TempDir::new().unwrap();
    emit_events(&dir, &[("build", "repo:main", "honest payload")]);
    assemble_verified(&dir, "t.json");

    // Tamper: edit an event field after sealing.
    let raw = fs::read_to_string(dir.path().join("t.json")).unwrap();
    let tampered = raw.replace("build", "hacked");
    assert_ne!(raw, tampered, "tamper must change the bytes");
    fs::write(dir.path().join("t.json"), tampered).unwrap();

    let (code, _out, err) = run_std(&dir, &["receipt", "verify", "t.json"]);
    assert_eq!(code, xc::REJECT, "stderr: {err}");
    assert!(err.contains("verdict: REJECT"), "stderr: {err}");
}

/// Negative: verify on a garbage file is a typed execution error (1504,
/// exit status 224), never a bare 1 or a panic.
#[test]
fn verify_garbage_file_is_typed_exec_error_224() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("garbage.json"),
        b"\x00\x01not a receipt at all",
    )
    .unwrap();
    let (code, _out, err) = run_std(&dir, &["receipt", "verify", "garbage.json"]);
    assert_eq!(code, xc::CLI_EXEC_ERROR, "stderr: {err}");
    assert_eq!(
        diag_code(&err),
        1504,
        "full stable code in diagnostic: {err}"
    );
}

// ===========================================================================
// (receipt, show)
// ===========================================================================

/// Positive + value: show output derives from the actual chain — event count
/// and ids match what was emitted, and differ across chains.
#[test]
fn show_output_derives_from_actual_chain() {
    let dir = TempDir::new().unwrap();
    emit_events(
        &dir,
        &[("build", "repo:main", "p1"), ("test", "suite:unit", "p2")],
    );
    assemble_verified(&dir, "s.json");

    affi(&dir)
        .args(["receipt", "show", "s.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("events: 2"))
        .stderr(predicate::str::contains("evt-0"))
        .stderr(predicate::str::contains("evt-1"))
        .stderr(predicate::str::contains("chain hash:"));

    // Value: a different chain yields different show output.
    let other = TempDir::new().unwrap();
    emit_events(&other, &[("deploy", "app:service", "p9")]);
    assemble_verified(&other, "s.json");
    let (code, out1, err1) = run_std(&dir, &["receipt", "show", "s.json"]);
    let (code2, out2, err2) = run_std(&other, &["receipt", "show", "s.json"]);
    assert_eq!(code, 0);
    assert_eq!(code2, 0);
    assert_ne!(
        format!("{out1}{err1}"),
        format!("{out2}{err2}"),
        "show output must distinguish the two chains"
    );
}

/// Negative: show on a missing file is a typed refusal (nonzero + structured
/// diagnostic), not a silent success or a bare 1.
#[test]
fn show_missing_file_is_typed_refusal() {
    let dir = TempDir::new().unwrap();
    let (code, _out, err) = run_std(&dir, &["receipt", "show", "nope.json"]);
    assert_ne!(code, 0);
    assert!(err.contains("\"code\":"), "structured diagnostic: {err}");
}

// ===========================================================================
// (receipt, inspect)
// ===========================================================================

/// Positive + value: inspect output reflects the real event/object
/// distribution of the chain under test and distinguishes chains.
#[test]
fn inspect_output_derives_from_actual_chain() {
    let dir = TempDir::new().unwrap();
    emit_events(
        &dir,
        &[("build", "repo:main", "p1"), ("test", "suite:unit", "p2")],
    );
    assemble_verified(&dir, "i.json");

    affi(&dir)
        .args(["receipt", "inspect", "i.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("events:"))
        .stderr(predicate::str::contains("build: 1 events"));

    let (o1, e1) = {
        let (c, o, e) = run_std(&dir, &["receipt", "inspect", "i.json"]);
        assert_eq!(c, 0);
        (format!("{o}{e}"), ())
    };
    let other = TempDir::new().unwrap();
    emit_events(&other, &[("deploy", "app:svc", "q")]);
    assemble_verified(&other, "i.json");
    let (c, o, e) = run_std(&other, &["receipt", "inspect", "i.json"]);
    assert_eq!(c, 0);
    let o2 = format!("{o}{e}");
    assert_ne!(o1, o2, "inspect must distinguish different chains");
    let _ = e1;
}

/// Negative: inspect on a garbage file is a typed refusal (nonzero, structured
/// diagnostic naming a stable code).
#[test]
fn inspect_garbage_file_is_typed_refusal() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("g.json"), b"{not json at all").unwrap();
    let (code, _out, err) = run_std(&dir, &["receipt", "inspect", "g.json"]);
    assert_ne!(code, 0);
    assert!(err.contains("\"code\":"), "structured diagnostic: {err}");
}

// ===========================================================================
// (receipt, stats)
// ===========================================================================

/// Positive + value: stats output is chain-derived — it distinguishes a
/// 2-event chain from a 1-event chain.
#[test]
fn stats_output_derives_from_actual_chain() {
    let one = TempDir::new().unwrap();
    emit_events(&one, &[("build", "repo:main", "solo")]);
    assemble_verified(&one, "st.json");

    let two = TempDir::new().unwrap();
    emit_events(
        &two,
        &[("build", "repo:main", "a"), ("test", "suite:unit", "b")],
    );
    assemble_verified(&two, "st.json");

    let (c1, o1, e1) = run_std(&one, &["receipt", "stats", "st.json"]);
    let (c2, o2, e2) = run_std(&two, &["receipt", "stats", "st.json"]);
    assert_eq!(c1, 0, "stderr: {e1}");
    assert_eq!(c2, 0, "stderr: {e2}");
    let s1 = format!("{o1}{e1}");
    let s2 = format!("{o2}{e2}");
    assert_ne!(s1, s2, "stats must distinguish 1-event vs 2-event chains");
    assert!(s2.contains('2'), "2-event chain stats must mention 2: {s2}");
    assert!(s1.contains('1'), "1-event chain stats must mention 1: {s1}");
}

/// Negative: stats on a missing file is a typed refusal (nonzero, structured
/// diagnostic), never a bare 1.
#[test]
fn stats_missing_file_is_typed_refusal() {
    let dir = TempDir::new().unwrap();
    let (code, _out, err) = run_std(&dir, &["receipt", "stats", "absent.json"]);
    assert_ne!(code, 0);
    assert!(err.contains("\"code\":"), "structured diagnostic: {err}");
}
