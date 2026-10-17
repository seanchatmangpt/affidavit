//! Value/analytics CLI court A — the analytics verbs must derive their output
//! from the real chain on disk, never from canned strings.
//!
//! Law (same as the vacuity tripwire): each cell runs the built `affi` binary
//! against real fixtures and asserts the outcome (exit code + derived fields)
//! flips with the input. Two different worlds must produce different outputs.
//!
//! Lane ownership: tests/cli_value_analytics_a.rs only.
//! Verbs covered: receipt root-cause, audit, query, model, conformance,
//! coverage-analysis, tech-debt.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_affi")
}

/// Serialize the suite: emit/assemble drive machine-level state (the shared
/// cargo-linked binary, env-resolved receipt paths) and flaked when raced.
static SUITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
thread_local! {
    static SUITE_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}
fn lock_suite() -> SuiteGuard {
    if SUITE_DEPTH.with(|d| d.get()) == 0 {
        let g = SUITE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        SUITE_DEPTH.with(|d| d.set(1));
        return SuiteGuard(Some(g));
    }
    SuiteGuard(None)
}
struct SuiteGuard(Option<std::sync::MutexGuard<'static, ()>>);
impl Drop for SuiteGuard {
    fn drop(&mut self) {
        if self.0.is_some() {
            SUITE_DEPTH.with(|d| d.set(0));
        }
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "affi-value-analytics-a-{}-{}",
        tag,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Emit the given (event_type, object) events in `dir` and assemble them into
/// `<dir>/receipts/<name>.json`. Returns the receipts dir path.
fn fixture_chain(tag: &str, name: &str, events: &[(&str, &str)]) -> PathBuf {
    let _g = lock_suite();
    let dir = scratch(tag);
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).expect("create receipts dir");
    let payload = dir.join("p.bin");
    fs::write(&payload, b"payload").unwrap();

    for (event_type, object) in events {
        let out = Command::new(bin())
            .args([
                "receipt",
                "emit",
                "--event-type",
                event_type,
                "--object",
                object,
                "--payload",
                payload.to_str().unwrap(),
            ])
            .current_dir(&dir)
            .output()
            .expect("run affi emit");
        assert!(
            out.status.success(),
            "emit {event_type} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let st = Command::new(bin())
        .args([
            "receipt",
            "assemble",
            "--out",
            &format!("receipts/{name}.json"),
        ])
        .current_dir(&dir)
        .status()
        .expect("run affi assemble");
    assert!(st.success(), "assemble {name} failed");
    receipts
}

fn run(args: &[String], dir: &PathBuf, env: Option<(&str, &PathBuf)>) -> (i32, String, String) {
    let mut cmd = Command::new(bin());
    cmd.args(args).current_dir(dir);
    if let Some((k, v)) = env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("run affi");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn combined(a: &str, b: &str) -> String {
    format!("{a}{b}")
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

// ---------------------------------------------------------------------------
// root-cause: real backward walk over the chain
// ---------------------------------------------------------------------------

#[test]
fn root_cause_walks_the_real_backward_chain() {
    let _g = lock_suite();
    // World A: build(0) -> test(1) -> release(2). Effect=release walks back
    // over build and test; probable root is the most recent preceding (test).
    let receipts = fixture_chain(
        "rc-a",
        "a",
        &[
            ("build", "repo:main"),
            ("test", "suite:unit"),
            ("release", "f:artifact"),
        ],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    let (code, stdout, _stderr) = run(
        &args(&[
            "receipt",
            "root-cause",
            "--effect-event",
            "release",
            "--receipts-path",
            receipts.join("a.json").to_str().unwrap(),
            "--format",
            "json",
        ]),
        &dir,
        None,
    );
    assert_eq!(code, 0, "root-cause must succeed on a real chain");
    assert!(stdout.contains("\"effect_seq\": 2"), "stdout: {stdout}");
    assert!(
        stdout.contains("\"preceding_events\": 2"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("\"event_type\": \"test\""),
        "probable root must be the real seq-1 test event: {stdout}"
    );
    assert!(
        stdout.contains("\"event_type\": \"build\""),
        "stdout: {stdout}"
    );

    // World B: a different chain — different effect, different causal chain.
    let receipts_b = fixture_chain(
        "rc-b",
        "b",
        &[("build", "repo:main"), ("deploy", "env:prod")],
    );
    let dir_b = receipts_b.parent().unwrap().to_path_buf();
    let (code_b, stdout_b, _stderr_b) = run(
        &args(&[
            "receipt",
            "root-cause",
            "--effect-event",
            "deploy",
            "--receipts-path",
            receipts_b.join("b.json").to_str().unwrap(),
            "--format",
            "json",
        ]),
        &dir_b,
        None,
    );
    let _ = code_b;
    assert!(
        stdout_b.contains("\"effect_seq\": 1"),
        "stdout_b: {stdout_b}"
    );
    assert!(
        stdout_b.contains("\"preceding_events\": 1"),
        "world B must derive its own chain: {stdout_b}"
    );
    assert!(
        !stdout_b.contains("\"event_type\": \"test\""),
        "world B output must not leak world A's events"
    );
}

#[test]
fn root_cause_missing_event_and_missing_path_are_typed_failures() {
    let _g = lock_suite();
    let receipts = fixture_chain("rc-miss", "a", &[("build", "repo:main")]);
    let dir = receipts.parent().unwrap().to_path_buf();

    // Effect not present in the chain: honest not-found, no fabricated RCA.
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "root-cause",
            "--effect-event",
            "nonexistent-effect",
            "--receipts-path",
            receipts.join("a.json").to_str().unwrap(),
        ]),
        &dir,
        None,
    );
    assert_eq!(
        code, 0,
        "not-found is an honest empty result: {stdout}{stderr}"
    );
    assert!(
        combined(&stdout, &stderr).contains("not found"),
        "must state the effect was not found: {stdout}{stderr}"
    );

    // Missing path: typed refusal, non-zero exit.
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "root-cause",
            "--effect-event",
            "x",
            "--receipts-path",
            "/nonexistent/zz.json",
        ]),
        &dir,
        None,
    );
    assert_ne!(code, 0, "missing path must fail, stdout: {stdout}");
    assert!(
        combined(&stdout, &stderr).contains("Path not found"),
        "typed missing-path message: {stdout}{stderr}"
    );
}

// ---------------------------------------------------------------------------
// audit: real counts, empty-dir refusal, tamper rejection
// ---------------------------------------------------------------------------

#[test]
fn audit_reports_real_counts_and_refuses_empty_dir() {
    let _g = lock_suite();
    // Two clean chains -> both checked and accepted.
    let receipts = fixture_chain(
        "aud-a",
        "a",
        &[("build", "repo:main"), ("test", "suite:unit")],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    // Add a second distinct chain.
    let out = Command::new(bin())
        .args([
            "receipt",
            "emit",
            "--event-type",
            "license-scan",
            "--object",
            "MIT:license",
            "--payload",
            dir.join("p.bin").to_str().unwrap(),
        ])
        .current_dir(&dir)
        .output()
        .expect("emit license");
    assert!(out.status.success());
    let st = Command::new(bin())
        .args(["receipt", "assemble", "--out", "receipts/chain2.json"])
        .current_dir(&dir)
        .status()
        .expect("assemble chain2");
    assert!(st.success());

    let (code, stdout, stderr) = run(
        &args(&["receipt", "audit"]),
        &dir,
        Some(("AFFI_RECEIPTS_PATH", &receipts)),
    );
    assert_eq!(code, 0, "clean chains must audit clean: {stdout}{stderr}");
    assert!(stdout.contains("\"chains_checked\": 2"), "stdout: {stdout}");
    assert!(stdout.contains("\"accepted\": 2"), "stdout: {stdout}");
    assert!(
        stdout.contains("\"rejected\": 0"),
        "stdout: {stdout}{stderr}"
    );

    // Empty dir -> typed REFUSED_NO_CHAINS (exit 224 = 1504 mod 256).
    let empty = scratch("aud-empty");
    fs::create_dir_all(empty.join("none")).unwrap();
    let (code, stdout, stderr) = run(
        &args(&["receipt", "audit"]),
        &empty,
        Some(("AFFI_RECEIPTS_PATH", &empty.join("none"))),
    );
    assert_eq!(code, 224, "empty dir must exit 224: {stdout}{stderr}");
    assert!(
        combined(&stdout, &stderr).contains("REFUSED_NO_CHAINS"),
        "typed refusal required: {stdout}{stderr}"
    );
}

#[test]
fn audit_rejects_a_tampered_chain_with_exit_2() {
    let _g = lock_suite();
    let receipts = fixture_chain(
        "aud-tamper",
        "a",
        &[("build", "repo:main"), ("test", "suite:unit")],
    );
    let dir = receipts.parent().unwrap().to_path_buf();

    // Tamper: rewrite an event_type in the assembled JSON (chain hash no longer
    // matches the recomputed rolling hash).
    let path = receipts.join("a.json");
    let raw = fs::read_to_string(&path).unwrap();
    let tampered = raw.replace("\"build\"", "\"buildx\"");
    assert_ne!(raw, tampered, "tamper must actually change the bytes");
    fs::write(&path, tampered).unwrap();

    let (code, stdout, stderr) = run(
        &args(&["receipt", "audit"]),
        &dir,
        Some(("AFFI_RECEIPTS_PATH", &receipts)),
    );
    assert_eq!(
        code, 2,
        "tampered chain must exit 2 (REJECT): {stdout}{stderr}"
    );
    assert!(
        stdout.contains("\"rejected\": 1") && stdout.contains("REJECT"),
        "audit must report the rejected chain with real counts: {stdout}{stderr}"
    );
}

// ---------------------------------------------------------------------------
// query: real term matching, honest miss, unknown syntax note
// ---------------------------------------------------------------------------

#[test]
fn query_matches_real_terms_and_miss_is_honest_empty() {
    let _g = lock_suite();
    let receipts = fixture_chain(
        "qry-a",
        "a",
        &[
            ("build", "repo:main"),
            ("test", "suite:unit"),
            ("release", "f:artifact"),
        ],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    let path = receipts.join("a.json").to_str().unwrap().to_string();

    // type= filter hits exactly the release event.
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "query",
            "--q",
            "type=release",
            "--receipts-path",
            &path,
        ]),
        &dir,
        None,
    );
    assert_eq!(code, 0);
    assert!(
        stdout.contains("1 match(es)") && stdout.contains("release"),
        "query must match the real release event: {stdout}{stderr}"
    );

    // Miss: honest empty result at exit 0, zero fabricated matches.
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "query",
            "--q",
            "type=does-not-exist",
            "--receipts-path",
            &path,
        ]),
        &dir,
        None,
    );
    assert_eq!(code, 0);
    assert!(
        stdout.contains("0 match(es)"),
        "miss must be an honest zero: {stdout}{stderr}"
    );

    // Unknown syntax: the parser falls back to substring match on event_type,
    // so `!!` matches nothing and exits 0 — an honest empty result, NOT a
    // fabricated hit. (Typed-refusal pass for malformed query syntax: not yet
    // landed; noted here rather than asserted.)
    let (code, stdout, _stderr) = run(
        &args(&["receipt", "query", "--q", "!!!", "--receipts-path", &path]),
        &dir,
        None,
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("0 match(es)"), "stdout: {stdout}");
}

#[test]
fn query_missing_path_is_typed_failure() {
    let _g = lock_suite();
    let receipts = fixture_chain("qry-miss", "a", &[("build", "repo:main")]);
    let dir = receipts.parent().unwrap().to_path_buf();
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "query",
            "--q",
            "type=build",
            "--receipts-path",
            "/nonexistent/zz.json",
        ]),
        &dir,
        None,
    );
    assert_ne!(code, 0, "missing path must fail: {stdout}{stderr}");
    assert!(
        combined(&stdout, &stderr).contains("Path not found"),
        "typed missing-path message: {stdout}{stderr}"
    );
}

// ---------------------------------------------------------------------------
// model + conformance: real discovery / real fitness from the chain
// ---------------------------------------------------------------------------

#[test]
fn model_discovers_real_activities_from_the_chain() {
    let _g = lock_suite();
    let receipts = fixture_chain(
        "mdl-a",
        "a",
        &[
            ("build", "repo:main"),
            ("test", "suite:unit"),
            ("release", "f:artifact"),
        ],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    let path = receipts.join("a.json");
    let (code, stdout, stderr) = run(
        &args(&["receipt", "model", path.to_str().unwrap()]),
        &dir,
        None,
    );
    assert_eq!(code, 0, "stdout: {stdout} stderr: {stderr}");
    let out = format!("{stdout}{stderr}");
    assert!(out.contains("build"), "must discover build: {out}");
    assert!(out.contains("test"), "must discover test: {out}");
    assert!(out.contains("release"), "must discover release: {out}");

    // Different world: different activities.
    let receipts_b = fixture_chain("mdl-b", "b", &[("plan", "p:x"), ("code", "c:y")]);
    let dir_b = receipts_b.parent().unwrap().to_path_buf();
    let (code_b, stdout_b, stderr_b) = run(
        &args(&[
            "receipt",
            "model",
            receipts_b.join("b.json").to_str().unwrap(),
        ]),
        &dir_b,
        None,
    );
    assert_eq!(code_b, 0, "stderr_b: {stderr_b}");
    let out_b = format!("{stdout_b}{stderr_b}");
    assert!(out_b.contains("plan") && out_b.contains("code"), "{out_b}");
    assert!(
        !out_b.contains("activity: release"),
        "world B must not leak world A's activities: {out_b}"
    );

    // Missing receipt path: typed refusal.
    let (code, stdout, stderr) = run(
        &args(&["receipt", "model", "/nonexistent/zz.json"]),
        &dir,
        None,
    );
    assert_ne!(code, 0);
    assert!(
        combined(&stdout, &stderr).contains("No such file or directory"),
        "typed missing-path message: {stdout}{stderr}"
    );
}

#[test]
fn conformance_computes_real_metrics_from_the_chain() {
    let _g = lock_suite();
    let receipts = fixture_chain(
        "cnf-a",
        "a",
        &[
            ("build", "repo:main"),
            ("test", "suite:unit"),
            ("release", "f:artifact"),
        ],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    let path = receipts.join("a.json");
    let (code, stdout, stderr) = run(
        &args(&["receipt", "conformance", path.to_str().unwrap()]),
        &dir,
        None,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let out = format!("{stdout}{stderr}");
    assert!(
        out.contains("fitness (token replay):"),
        "conformance must compute fitness: {out}"
    );
    assert!(out.contains("activity_coverage:"), "{out}");
    assert!(out.contains("simplicity"), "{out}");

    // Distinct world (1 unique type over 1 event) — coverage is 1.0000 there;
    // world A has 3 unique over 3 (also 1.0000), so instead assert the metric
    // lines parse as real floats derived from the chain (not "null"/"NaN").
    for line in out.lines() {
        if line.contains("fitness") || line.contains("coverage") || line.contains("simplicity") {
            let val = line.rsplit(':').next().unwrap().trim();
            let num = val.parse::<f64>();
            assert!(
                num.is_ok(),
                "metric must be a real number, got '{val}': {out}"
            );
            let n = num.unwrap();
            assert!(
                n.is_finite() && (0.0..=1.0).contains(&n),
                "metric {n} out of range: {out}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// coverage-analysis + tech-debt: real ratios / real event classification
// ---------------------------------------------------------------------------

#[test]
fn coverage_analysis_reports_real_ratios() {
    let _g = lock_suite();
    // World A: 1 test event of 3 -> 33.3%.
    let receipts = fixture_chain(
        "cov-a",
        "a",
        &[
            ("build", "repo:main"),
            ("test", "suite:unit"),
            ("release", "f:artifact"),
        ],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "coverage-analysis",
            "--receipts-path",
            receipts.join("a.json").to_str().unwrap(),
            "--format",
            "json",
        ]),
        &dir,
        None,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"test_events\": 1") && stdout.contains("\"total_events\": 3"),
        "real counts: {stdout}"
    );
    assert!(
        stdout.contains("33.3"),
        "ratio must be derived from the chain: {stdout}"
    );

    // World B: no test events -> 0.0%. Output must flip with the input.
    let receipts_b = fixture_chain(
        "cov-b",
        "b",
        &[("build", "repo:main"), ("release", "f:artifact")],
    );
    let dir_b = receipts_b.parent().unwrap().to_path_buf();
    let (code_b, stdout_b, _e) = run(
        &args(&[
            "receipt",
            "coverage-analysis",
            "--receipts-path",
            receipts_b.join("b.json").to_str().unwrap(),
            "--format",
            "json",
        ]),
        &dir_b,
        None,
    );
    assert_eq!(code_b, 0);
    assert!(
        stdout_b.contains("\"test_events\": 0") && stdout_b.contains("0.0"),
        "world B must be honestly zero: {stdout_b}"
    );

    // Missing path: typed failure.
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "coverage-analysis",
            "--receipts-path",
            "/nonexistent/zz.json",
        ]),
        &dir,
        None,
    );
    assert_ne!(code, 0);
    assert!(
        combined(&stdout, &stderr).contains("Path not found"),
        "{stdout}{stderr}"
    );
}

#[test]
fn tech_debt_classifies_real_events() {
    let _g = lock_suite();
    // Debt world: refactor + hotfix events classify as debt/churn.
    let receipts = fixture_chain(
        "debt-a",
        "a",
        &[
            ("build", "repo:main"),
            ("refactor", "m:mod"),
            ("hotfix", "m:mod"),
            ("refactor", "m:other"),
        ],
    );
    let dir = receipts.parent().unwrap().to_path_buf();
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "tech-debt",
            "--receipts-path",
            receipts.join("a.json").to_str().unwrap(),
            "--format",
            "json",
        ]),
        &dir,
        None,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"refactor_events\": 2") && stdout.contains("\"churn_events\": 1"),
        "real classification: {stdout}"
    );
    assert!(stdout.contains("\"total_events\": 4"), "{stdout}");
    assert!(stdout.contains("75.0"), "debt ratio must be 75%: {stdout}");
    assert!(
        stdout.contains("\"HIGH\""),
        "75% > 20% must assess HIGH: {stdout}"
    );

    // Clean world: zero debt -> LOW, output flips with input.
    let receipts_b = fixture_chain("debt-b", "b", &[("build", "repo:main")]);
    let dir_b = receipts_b.parent().unwrap().to_path_buf();
    let (code_b, stdout_b, _e) = run(
        &args(&[
            "receipt",
            "tech-debt",
            "--receipts-path",
            receipts_b.join("b.json").to_str().unwrap(),
            "--format",
            "json",
        ]),
        &dir_b,
        None,
    );
    assert_eq!(code_b, 0);
    assert!(
        stdout_b.contains("0.0") && stdout_b.contains("\"LOW\""),
        "clean world must be LOW: {stdout_b}"
    );

    // Missing path: typed failure.
    let (code, stdout, stderr) = run(
        &args(&[
            "receipt",
            "tech-debt",
            "--receipts-path",
            "/nonexistent/zz.json",
        ]),
        &dir,
        None,
    );
    assert_ne!(code, 0);
    assert!(
        combined(&stdout, &stderr).contains("Path not found"),
        "{stdout}{stderr}"
    );
}
