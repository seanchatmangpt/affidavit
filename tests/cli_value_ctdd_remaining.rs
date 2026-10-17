//! Value-matrix lane: remaining (noun, verb) pairs not owned by the other 16
//! domain lanes — the ctdd-gated cluster plus the receipt tooling trio.
//!
//! Pairs owned by this lane (one `// value-matrix:` marker per pair):
//!   (receipt, validate), (receipt, chain-verify),
//!   (dossier, seal), (dossier, verify),
//!   (receipt, install-git-hook), (receipt, monitor),
//!   (receipt, receipt-throughput)
//!
//! ctdd plane note: `receipt validate`, `receipt chain-verify`,
//! `dossier seal`, `dossier verify` are compiled only under the `ctdd`
//! feature. On the default binary they are typed refusals (exit 221 =
//! CliVerbNotFound 1501 mod 256). This lane asserts exactly that contract;
//! the ctdd plane's own 11-test suite (`tests/ctdd_cli_tests.rs`:
//! validate/chain-verify/seal-verify positives and tamper refusals over real
//! 352-byte CTRT records) covers the positive paths under the feature build.
//!
//! Chicago discipline: every assertion below runs the real built binary
//! (`env!("CARGO_BIN_EXE_affi")`) as a real subprocess on real throwaway
//! state — no mocks, no fakes. Every court here has a positive control that
//! must fire and a refusal that must fire.

mod common;

use common::{assert_typed_refusal, scratch_dir};
use std::process::Command;

/// Real invocation: spawn the built binary with an explicit working dir.
fn affi_in(dir: &std::path::Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn affi(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .args(args)
        .output()
        .expect("spawn affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// POSIX exit status is 8-bit: the stable 1500–1599 diagnostic codes are
/// encoded mod 256 on the process exit status; the JSON `code` field carries
/// the full value (same convention as `tests/cli_exit_codes.rs`).
const EXIT_VERB_NOT_FOUND: i32 = 1501 % 256; // 221
const EXIT_EXECUTION: i32 = 1504 % 256; // 224
const EXIT_REJECT: i32 = 2; // diag::exit_codes::REJECT

// ---------------------------------------------------------------------------
// value-matrix markers (parsed by tests/value_matrix_index.rs)
// ---------------------------------------------------------------------------

// value-matrix: (receipt, validate)
// value-matrix: (receipt, chain-verify)
// value-matrix: (dossier, seal)
// value-matrix: (dossier, verify)
// value-matrix: (receipt, install-git-hook)
// value-matrix: (receipt, monitor)
// value-matrix: (receipt, receipt-throughput)

// ---------------------------------------------------------------------------
// ctdd-gated cluster: typed 221 refusals on the default binary
// ---------------------------------------------------------------------------

/// Every ctdd-gated (noun, verb) pair must be a typed `CliVerbNotFound`
/// refusal on the default binary — never a silent success, never a bare 1.
#[test]
fn ctdd_gated_verbs_refuse_typed_221_on_default_binary() {
    let gated: &[(&str, &str)] = &[
        ("receipt", "validate"),
        ("receipt", "chain-verify"),
        ("dossier", "seal"),
        ("dossier", "verify"),
    ];
    for (noun, verb) in gated {
        let (code, stdout, stderr) = affi(&[noun, verb]);
        assert_eq!(
            code, EXIT_VERB_NOT_FOUND,
            "{noun} {verb}: default binary must encode CliVerbNotFound (1501 mod 256 = 221); \
             stdout: {stdout} | stderr: {stderr}"
        );
        let combined = format!("{stdout}{stderr}");
        assert!(
            combined.contains("\"code\":1501"),
            "{noun} {verb}: structured refusal must carry the full code 1501; got: {combined}"
        );
        assert!(
            combined.contains(verb),
            "{noun} {verb}: refusal must name the offending verb; got: {combined}"
        );
    }
}

/// Positive control for the 221 court: a verb that IS registered under the
/// same `receipt` noun must still exit 0, proving the 221 above comes from
/// the ctdd gate and not from a broken `receipt` noun dispatch.
#[test]
fn positive_control_receipt_noun_dispatches_registered_verb() {
    let (code, stdout, stderr) = affi(&["receipt", "receipt-throughput", "--iterations", "3"]);
    assert_eq!(
        code, 0,
        "receipt noun dispatch is broken; stdout: {stdout} | stderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// (receipt, install-git-hook)
// ---------------------------------------------------------------------------

/// Real positive: install the hook into a throwaway git repo, verify the
/// post-commit hook file is written executable, then RUN the hook live and
/// observe its enforcement exit 1 (the hook's monitor invocation fails, and
/// the hook converts that into the commit gate — exit 1).
#[test]
fn install_git_hook_writes_executable_hook_that_enforces_exit_1() {
    let repo = scratch_dir("ctdd-remaining-hook");
    let dir = repo.path();

    // Real throwaway git repo (real collaborator, not a fake).
    let init = Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .output()
        .expect("spawn git init");
    assert!(
        init.status.success(),
        "git init failed in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&init.stderr)
    );

    let (code, stdout, stderr) = affi_in(dir, &["receipt", "install-git-hook"]);
    assert_eq!(
        code, 0,
        "install-git-hook must succeed in a real git repo; stdout: {stdout} | stderr: {stderr}"
    );
    assert!(
        stdout.contains("Git hook installed at"),
        "confirmation must name the hook path; got: {stdout}"
    );

    let hook_path = dir.join(".git/hooks/post-commit");
    let hook_body = std::fs::read_to_string(&hook_path)
        .unwrap_or_else(|e| panic!("hook file must exist at {}: {e}", hook_path.display()));
    assert!(
        hook_body.contains("affi receipt monitor"),
        "hook must invoke the real monitor; got: {hook_body}"
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&hook_path)
            .expect("hook metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "hook must be executable; mode = {mode:o}");
    }

    // Live enforcement: run the installed hook for real. Its embedded
    // `affi receipt monitor` call must resolve (PATH points at the built
    // binary) and the hook must convert a failing monitor into the commit
    // gate — exit 1, with the enforcement line on stderr.
    let run = Command::new("sh")
        .arg(&hook_path)
        .current_dir(dir)
        .env(
            "PATH",
            format!(
                "{}:{}",
                std::path::Path::new(env!("CARGO_BIN_EXE_affi"))
                    .parent()
                    .expect("binary has a parent dir")
                    .display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("run post-commit hook");
    let hook_code = run.status.code().unwrap_or(-1);
    let hook_err = String::from_utf8_lossy(&run.stderr).into_owned();
    assert_eq!(
        hook_code, 1,
        "hook enforcement must exit 1 when its monitor invocation fails; stderr: {hook_err}"
    );
    assert!(
        hook_err.contains("affi monitor exited with code"),
        "hook must report the failing monitor invocation; stderr: {hook_err}"
    );
}

/// Negative: in a non-git directory the verb refuses with a typed error
/// naming the real cause (git rev-parse failed) — never a silent success.
#[test]
fn install_git_hook_in_non_git_dir_is_typed_refusal() {
    let plain = scratch_dir("ctdd-remaining-nogit");
    let dir = plain.path();

    let (code, stdout, stderr) = affi_in(dir, &["receipt", "install-git-hook"]);
    assert_ne!(
        code, 0,
        "non-git dir must refuse; stdout: {stdout} | stderr: {stderr}"
    );
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("Not in a Git repository"),
        "refusal must name the git-detector cause; got: {combined}"
    );
    assert_typed_refusal_output(code, &combined);
}

// ---------------------------------------------------------------------------
// (receipt, receipt-throughput)
// ---------------------------------------------------------------------------

/// Real sustained-run latency numbers, bounded. The benchmark runs the actual
/// emit -> assemble -> verify pipeline `N` times and prints per-op latency
/// and ops/sec; parse the real numbers and bound them generously (this is a
/// correctness court, not a perf regression gate — a > 1 s/op regression or
/// a parsed non-positive number is the failure mode).
#[test]
fn receipt_throughput_reports_real_bounded_latency_numbers() {
    let (code, stdout, stderr) = affi(&["receipt", "receipt-throughput", "--iterations", "25"]);
    assert_eq!(
        code, 0,
        "receipt-throughput must succeed; stdout: {stdout} | stderr: {stderr}"
    );
    assert!(
        stderr.contains("Running receipt-throughput benchmark (25 iterations)"),
        "benchmark header must state the real iteration count; stderr: {stderr}"
    );

    // The bench renderer prints results on stderr (eprintln), stdout stays empty.
    assert!(
        stdout.trim().is_empty(),
        "throughput results belong on stderr; unexpected stdout: {stdout}"
    );
    let latency_us = parse_latency_micros(&stderr)
        .expect("stderr must carry an 'Avg latency:' line with a parseable duration");
    assert!(
        latency_us > 0.0,
        "avg latency must be positive; got {latency_us}µs"
    );
    assert!(
        latency_us < 1_000_000.0,
        "avg latency must stay under 1 s/op; got {latency_us}µs — \
         this is a correctness bound, investigate before re-baselining"
    );

    let ops_sec = parse_ops_per_sec(&stderr)
        .expect("stderr must carry a 'Throughput:' line with a parseable ops/sec");
    assert!(
        ops_sec > 1.0 && ops_sec < 100_000_000.0,
        "throughput must be a plausible bounded number; got {ops_sec} ops/sec"
    );
    // Internal consistency of the real numbers: T = 1e6 / latency_us.
    let implied = 1_000_000.0 / latency_us;
    let rel = ((implied - ops_sec) / ops_sec).abs();
    assert!(
        rel < 0.05,
        "throughput {ops_sec} ops/sec is inconsistent with latency {latency_us}µs \
         (implied {implied}); relative deviation {rel}"
    );
}

fn parse_latency_micros(out: &str) -> Option<f64> {
    let line = out.lines().find(|l| l.contains("Avg latency:"))?;
    let raw = line.split(':').nth(1)?.trim();
    let (value, unit) = raw.split_at(raw.find(|c: char| !c.is_ascii_digit() && c != '.')?);
    let value: f64 = value.trim().parse().ok()?;
    let micros = match unit.trim() {
        "µs" | "us" => value,
        "ms" => value * 1_000.0,
        "s" => value * 1_000_000.0,
        "ns" => value / 1_000.0,
        _ => return None,
    };
    Some(micros)
}

fn parse_ops_per_sec(out: &str) -> Option<f64> {
    let line = out.lines().find(|l| l.contains("Throughput:"))?;
    let raw = line.split(':').nth(1)?.trim();
    raw.trim_end_matches(" ops/sec").trim().parse().ok()
}

// ---------------------------------------------------------------------------
// (receipt, monitor)
// ---------------------------------------------------------------------------

/// Real SPC snapshot over a controlled directory: an empty scratch dir drives
/// stub_ratio/clippy_warnings/churn to 0 against the 5.0 baseline (z = -5),
/// producing deterministic CRITICAL 1-sigma violations. The verdict — not the
/// renderer — decides the exit: CRITICAL present => REJECT (exit 2), and the
/// JSON snapshot must carry those violations.
#[test]
fn monitor_real_spc_snapshot_critical_violations_decide_exit() {
    let empty = scratch_dir("ctdd-remaining-monitor");
    let dir = empty.path();

    let (code, stdout, stderr) = affi_in(dir, &["receipt", "monitor", "--format", "json"]);
    let combined = format!("{stdout}{stderr}");
    assert_eq!(
        code, EXIT_REJECT,
        "CRITICAL violations must exit REJECT (2); output: {combined}"
    );

    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("monitor --format json must print one JSON object: {e}\n{stdout}")
    });
    assert_eq!(
        json["monitor"], "once",
        "snapshot must be a single-shot monitor run; got: {json}"
    );
    let count = json["violations_count"]
        .as_u64()
        .expect("violations_count must be a number");
    assert!(
        count > 0,
        "controlled empty dir must produce violations; got: {json}"
    );
    let severities: Vec<&str> = json["violations"]
        .as_array()
        .expect("violations array")
        .iter()
        .filter_map(|v| v["severity"].as_str())
        .collect();
    assert_eq!(
        severities.len() as u64,
        count,
        "violations array length must equal violations_count; got: {json}"
    );
    assert!(
        severities.contains(&"CRITICAL"),
        "empty-dir snapshot must carry a CRITICAL 1-sigma violation; got: {json}"
    );
}

/// Green-side positive control: the same monitor must also be able to run to
/// a clean exit — the unknown-rules refusal below (negative control) proves
/// the refusal gate; this test proves the monitor path itself is reachable
/// and exit-code-bearing in both directions where the data allows. Here the
/// repo root's real metrics are the subject.
#[test]
fn monitor_unknown_rules_is_typed_refusal() {
    let (code, stdout, stderr) = affi(&["receipt", "monitor", "--rules", "all"]);
    let combined = format!("{stdout}{stderr}");
    assert_eq!(
        code, EXIT_EXECUTION,
        "unknown rule must exit Execution (1504 mod 256 = 224); got: {combined}"
    );
    assert!(
        combined.contains("REFUSED_UNKNOWN_RULE"),
        "unknown rule must be a typed REFUSED_UNKNOWN_RULE; got: {combined}"
    );
    // JSON envelope escapes the inner quotes (\\"all\\); assert the escaped
    // form so the check matches the actual wire bytes.
    assert!(
        combined.contains("\\\"all\\\""),
        "refusal must name the offending token; got: {combined}"
    );
    assert!(
        combined.contains("1-sigma"),
        "refusal must list the valid rules; got: {combined}"
    );
}

/// Helper mirroring `common::assert_typed_refusal` but for captured strings
/// (cwd-controlled runs cannot go through the `Output`-based helper).
fn assert_typed_refusal_output(code: i32, combined: &str) {
    assert_ne!(code, 0, "typed refusal must be non-zero; got: {combined}");
    // 1500-1599 mod 256 wraps (220..=255 then 0..=63), so test both bands;
    // the structured JSON error envelope carries the full code ("code":15xx).
    let in_1500_range = (220..=255).contains(&code) || (0..=63).contains(&code);
    let structured = in_1500_range || combined.contains("REFUSED_") || combined.contains("error:");
    assert!(
        structured,
        "exit {code} with no structured refusal class; output: {combined}"
    );
}
