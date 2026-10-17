//! Court: CLI value matrix for the six analytics/intelligence receipt verbs —
//! causality-chain, bus-factor, dora-metrics, team-velocity, portfolio-health,
//! orphaned-code. Per verb: >=1 positive (real effect on real state),
//! >=1 negative (typed refusal), >=1 value assertion (output derives from the
//! actual receipts under test, not from the test's own construction).
//!
//! Chicago-style: spawns the real `affi` binary, builds real multi-receipt
//! worlds in isolated TempDirs, asserts on real exit codes and the real JSON
//! the handlers printed. No mocks.
//!
//! Lane: write lane, owned file: tests/cli_value_analytics_c.rs. No git.
//
// value-matrix: (receipt, causality-chain)
// value-matrix: (receipt, bus-factor)
// value-matrix: (receipt, dora-metrics)
// value-matrix: (receipt, team-velocity)
// value-matrix: (receipt, portfolio-health)
// value-matrix: (receipt, orphaned-code)
//
// value-matrix: harness — tests/common/mod.rs absent; harness defined locally.

use assert_cmd::Command;
use std::process::Command as StdCommand;
use tempfile::TempDir;

/// Stable process-exit codes from `src/diag.rs::exit_codes` (public contract).
mod xc {
    /// CLI framework codes 1500..=1509 ride mod 256 on the 8-bit POSIX status:
    /// 1504 (CliExecutionError) -> 224 — the exit the handler refusals ride.
    pub const CLI_EXEC_ERROR: i32 = 1504 % 256; // 224
}

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

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

/// Emit one event into the working chain. Payload rides stdin (`-` source).
fn emit(dir: &TempDir, event_type: &str, object: &str, payload: &str) {
    affi(dir)
        .args([
            "receipt",
            "emit",
            "--type",
            event_type,
            "--object",
            object,
            "--payload",
            "-",
        ])
        .write_stdin(payload)
        .assert()
        .success();
}

/// Seal the working chain as `receipt_k.json` and reset `.affi/` so the next
/// receipt in the same world starts from seq 0 (independent receipts).
fn seal_and_reset(dir: &TempDir, k: usize) {
    let out = format!("receipt_{k}.json");
    affi(dir)
        .args(["receipt", "assemble", "--out", &out])
        .assert()
        .success();
    assert!(
        dir.path().join(&out).exists(),
        "assemble must produce {out}"
    );
    let affi_dir = dir.path().join(".affi");
    match std::fs::remove_dir_all(&affi_dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => panic!("reset working chain: {e}"),
    }
}

/// Parse the FIRST JSON value (object or array) in `s`, tolerating the known
/// trailing `null` residual the framework prints for unit-returning verbs
/// (cli_dispatch.rs documents this OPEN residual — never asserted as a pass).
fn first_json(s: &str) -> serde_json::Value {
    let start = s
        .find(|c| c == '{' || c == '[')
        .expect("stdout must contain JSON");
    let mut stream =
        serde_json::Deserializer::from_str(&s[start..]).into_iter::<serde_json::Value>();
    stream
        .next()
        .expect("first JSON value parses")
        .expect("first JSON value is well-formed")
}

// ---------------------------------------------------------------------------
// World builders — real receipts, real chains, real file state
// ---------------------------------------------------------------------------

/// Healthy world: 2 receipts, active multi-event chains, deploys/tests, no
/// security events.
fn world_healthy(dir: &TempDir) {
    // Receipt 0: build -> test -> deploy on svc-a:service
    emit(dir, "build", "svc-a:service", "b1");
    emit(dir, "test", "svc-a:service", "t1");
    emit(dir, "deploy", "svc-a:service", "d1");
    seal_and_reset(dir, 0);

    // Receipt 1: test -> pull_request -> merge -> deploy on svc-b:service
    emit(dir, "test", "svc-b:service", "t2");
    emit(dir, "pull_request", "svc-b:service", "p1");
    emit(dir, "merge", "svc-b:service", "m1");
    emit(dir, "deploy", "svc-b:service", "d2");
    seal_and_reset(dir, 1);
}

/// Degraded world: 2 stale single-event receipts, one security incident.
fn world_degraded(dir: &TempDir) {
    emit(dir, "security", "svc-a:service", "sec1");
    seal_and_reset(dir, 0);
    emit(dir, "vuln", "svc-b:service", "v1");
    seal_and_reset(dir, 1);
}

/// Bus-factor world: `service` referenced by 2 receipts; `database` by only 1.
fn world_bus_factor(dir: &TempDir) {
    emit(dir, "deploy", "svc-a:service,db-main:database", "d1");
    seal_and_reset(dir, 0);
    emit(dir, "deploy", "svc-b:service", "d2");
    seal_and_reset(dir, 1);
}

/// Orphan world: one 1-event receipt (orphan) + one active deploy chain.
fn world_orphan(dir: &TempDir) {
    emit(dir, "init", "ghost:repo", "i1");
    seal_and_reset(dir, 0);
    emit(dir, "deploy", "live:repo", "d1");
    emit(dir, "test", "live:repo", "t1");
    seal_and_reset(dir, 1);
}

// ---------------------------------------------------------------------------
// causality-chain
// ---------------------------------------------------------------------------

#[test]
fn causality_chain_walks_real_chain_from_real_start_event() {
    let dir = TempDir::new().expect("tempdir");
    world_healthy(&dir);

    // Walk from the real "build" event in receipt 0 — the handler matches by
    // event id OR event type; assert the walk covers the real downstream chain
    // (build -> test -> deploy) in seq order.
    let (code, stdout, _stderr) = run_std(
        &dir,
        &[
            "receipt",
            "causality-chain",
            "--start-event",
            "build",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "walk over a real start event must exit 0: {stdout}"
    );

    let chain = first_json(&stdout);
    let arr = chain.as_array().expect("chain renders as a JSON array");
    assert!(
        arr.len() >= 3,
        "chain must cover the real downstream events, got {} steps: {stdout}",
        arr.len()
    );
    let types: Vec<&str> = arr
        .iter()
        .filter_map(|e| e["event_type"].as_str())
        .collect();
    assert_eq!(
        &types[..3],
        &["build", "test", "deploy"][..],
        "chain must walk the real seq order build -> test -> deploy: {types:?}"
    );
}

#[test]
fn causality_chain_unknown_start_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    world_healthy(&dir);

    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "causality-chain",
            "--start-event",
            "evt-that-never-was",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed refusal exit: out={stdout} err={stderr}"
    );
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("REFUSED_START_EVENT_NOT_FOUND"),
        "refusal must be typed REFUSED_START_EVENT_NOT_FOUND: {combined}"
    );
    assert!(
        combined.contains("evt-that-never-was"),
        "refusal must name the offending start event: {combined}"
    );
}

#[test]
fn causality_chain_missing_path_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "causality-chain",
            "--start-event",
            "build",
            "--receipts-path",
            "no/such/dir",
        ],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed path refusal exit: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("Path not found"),
        "path refusal must be loud: {stdout} {stderr}"
    );
}

// ---------------------------------------------------------------------------
// bus-factor
// ---------------------------------------------------------------------------

#[test]
fn bus_factor_partitions_real_object_types() {
    let dir = TempDir::new().expect("tempdir");
    world_bus_factor(&dir);

    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "bus-factor",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "bus-factor over a real world must exit 0: {stdout} {stderr}"
    );

    let v = first_json(&stdout);
    let factors = v["bus_factors"]
        .as_array()
        .expect("bus_factors array present");
    assert!(
        factors.len() >= 2,
        "real object-type partition must contain >= 2 types: {stdout}"
    );

    let find = |t: &str| {
        factors
            .iter()
            .find(|f| f["object_type"] == t)
            .unwrap_or_else(|| panic!("object type {t} must appear in the partition: {stdout}"))
    };

    // `database` is referenced by exactly 1 receipt -> HIGH risk.
    let db = find("database");
    assert_eq!(db["bus_factor"], 1, "database is single-receipt: {stdout}");
    assert_eq!(
        db["risk"], "HIGH",
        "single-receipt type is HIGH risk: {stdout}"
    );

    // `service` is referenced by both receipts -> LOW risk.
    let svc = find("service");
    assert_eq!(
        svc["bus_factor"], 2,
        "service is shared by both receipts: {stdout}"
    );
    assert_eq!(
        svc["risk"], "MEDIUM",
        "two-receipt type is MEDIUM risk (handler law: 1=HIGH, <=2=MEDIUM, else LOW): {stdout}"
    );

    assert_eq!(
        v["high_risk"], 1,
        "exactly the single-receipt type is HIGH risk: {stdout}"
    );
}

#[test]
fn bus_factor_missing_path_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_std(
        &dir,
        &["receipt", "bus-factor", "--receipts-path", "no/such/dir"],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed path refusal exit: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("Path not found"),
        "path refusal must be loud: {stdout} {stderr}"
    );
}

// ---------------------------------------------------------------------------
// dora-metrics
// ---------------------------------------------------------------------------

#[test]
fn dora_metrics_scans_real_deploy_and_incident_events() {
    let dir = TempDir::new().expect("tempdir");
    world_healthy(&dir);

    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "dora-metrics",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "dora over a real world must exit 0: {stdout} {stderr}"
    );

    let v = first_json(&stdout);
    let dora = &v["dora"];
    assert_eq!(
        dora["deployment_frequency"]["deploys_found"], 2,
        "both real deploys must be counted: {stdout}"
    );
    // Honest zero: this world has no incident/failure events at all.
    assert_eq!(
        dora["change_failure_rate"]["incidents"], 0,
        "zero incidents must be reported as an honest zero: {stdout}"
    );
    assert_eq!(
        dora["change_failure_rate"]["value"], 0.0,
        "change failure rate derives from real events, so it is 0.0 here: {stdout}"
    );
    assert_eq!(
        v["receipts_analyzed"], 2,
        "both receipts on disk must be analyzed: {stdout}"
    );
    assert_eq!(
        v["total_events"], 7,
        "3 + 4 real events across the two receipts: {stdout}"
    );
}

#[test]
fn dora_metrics_empty_store_documents_honest_zeros() {
    let dir = TempDir::new().expect("tempdir");
    // World with events but zero deploys and zero incidents: every DORA
    // deploy-side signal must be an honest zero derived from the scan, not a
    // fabricated number.
    emit(&dir, "docs", "readme:doc", "r1");
    seal_and_reset(&dir, 0);

    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "dora-metrics",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "dora over a deploy-free world must still exit 0: {stdout} {stderr}"
    );

    let v = first_json(&stdout);
    let dora = &v["dora"];
    assert_eq!(
        dora["deployment_frequency"]["deploys_found"], 0,
        "honest zero deploys: {stdout}"
    );
    assert_eq!(
        dora["deployment_frequency"]["value"], 0.0,
        "honest zero frequency: {stdout}"
    );
    assert_eq!(
        dora["change_failure_rate"]["incidents"], 0,
        "honest zero incidents: {stdout}"
    );
    assert_eq!(
        dora["mttr"]["recoveries"], 0,
        "honest zero recoveries: {stdout}"
    );
}

#[test]
fn dora_metrics_missing_path_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_std(
        &dir,
        &["receipt", "dora-metrics", "--receipts-path", "no/such/dir"],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed path refusal exit: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("Path not found"),
        "path refusal must be loud: {stdout} {stderr}"
    );
}

// ---------------------------------------------------------------------------
// team-velocity
// ---------------------------------------------------------------------------

#[test]
fn team_velocity_counts_real_pr_and_merge_events() {
    let dir = TempDir::new().expect("tempdir");
    world_healthy(&dir);

    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "team-velocity",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "team-velocity over a real world must exit 0: {stdout} {stderr}"
    );

    let v = first_json(&stdout);
    assert_eq!(
        v["pr_events"], 1,
        "1 real pull_request event in the world: {stdout}"
    );
    assert_eq!(
        v["merge_events"], 1,
        "1 real merge event in the world: {stdout}"
    );
    assert_eq!(v["receipts"], 2, "both receipts counted: {stdout}");
    assert_eq!(v["total_events"], 7, "3 + 4 real events: {stdout}");
    // pr_to_merge_ratio = 1 / 1, derived from the real counts.
    assert_eq!(
        v["pr_to_merge_ratio"], 1.0,
        "ratio must derive from the real pr/merge counts: {stdout}"
    );
}

#[test]
fn team_velocity_missing_path_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_std(
        &dir,
        &["receipt", "team-velocity", "--receipts-path", "no/such/dir"],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed path refusal exit: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("Path not found"),
        "path refusal must be loud: {stdout} {stderr}"
    );
}

// ---------------------------------------------------------------------------
// portfolio-health
// ---------------------------------------------------------------------------

#[test]
fn portfolio_health_score_moves_when_signals_change() {
    // Two worlds -> two different scores. That difference IS the value
    // assertion: the score derives from real signals, not a constant.
    let healthy = TempDir::new().expect("tempdir");
    world_healthy(&healthy);
    let (hc, hout, herr) = run_std(
        &healthy,
        &[
            "receipt",
            "portfolio-health",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(hc, 0, "healthy-world scan must exit 0: {hout} {herr}");
    let h = first_json(&hout);

    let degraded = TempDir::new().expect("tempdir");
    world_degraded(&degraded);
    let (dc, dout, derr) = run_std(
        &degraded,
        &[
            "receipt",
            "portfolio-health",
            "--receipts-path",
            ".",
            "--format",
            "json",
        ],
    );
    assert_eq!(dc, 0, "degraded-world scan must exit 0: {dout} {derr}");
    let d = first_json(&dout);

    let hs = h["health_score"]
        .as_f64()
        .expect("health_score is a number");
    let ds = d["health_score"]
        .as_f64()
        .expect("health_score is a number");

    // Signal provenance: the healthy world is all-active, test+deploy rich,
    // security-free; the degraded world is all-stale, single-event, with a
    // security incident. The handler's law: active_ratio + test_ratio +
    // deploy_ratio + security_bonus.
    let h_active = h["portfolio"]["active"].as_u64().expect("active count");
    let d_active = d["portfolio"]["active"].as_u64().expect("active count");
    assert_eq!(
        h_active, 2,
        "both healthy receipts are multi-event (active): {hout}"
    );
    assert_eq!(
        d_active, 0,
        "both degraded receipts are single-event (stale): {dout}"
    );
    assert_eq!(
        h["signals"]["security_events"], 0,
        "healthy world has no security events: {hout}"
    );
    assert!(
        d["signals"]["security_events"]
            .as_u64()
            .expect("security count")
            >= 1,
        "degraded world carries a real security event: {dout}"
    );

    assert!(
        hs > ds,
        "healthy score ({hs}) must exceed degraded score ({ds}) — the score must move when signals move"
    );
    assert!(
        hs >= 75.0,
        "healthy world rates GOOD: score={hs} out={hout}"
    );
    assert_eq!(h["rating"], "GOOD", "healthy rating: {hout}");
    assert_eq!(d["rating"], "POOR", "degraded rating: {dout}");
}

#[test]
fn portfolio_health_missing_path_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "portfolio-health",
            "--receipts-path",
            "no/such/dir",
        ],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed path refusal exit: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("Path not found"),
        "path refusal must be loud: {stdout} {stderr}"
    );
}

// ---------------------------------------------------------------------------
// orphaned-code
// ---------------------------------------------------------------------------

#[test]
fn orphaned_code_classification_derives_from_real_events() {
    let dir = TempDir::new().expect("tempdir");
    world_orphan(&dir);

    let (code, stdout, stderr) = run_std(
        &dir,
        &[
            "receipt",
            "orphaned-code",
            "--receipts-path",
            ".",
            "--days",
            "30",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "orphaned-code over a real world must exit 0: {stdout} {stderr}"
    );

    let v = first_json(&stdout);
    assert_eq!(v["total_receipts"], 2, "both receipts scanned: {stdout}");
    assert_eq!(
        v["orphaned_count"], 1,
        "exactly the single-event ghost receipt is orphaned: {stdout}"
    );
    let orphans = v["orphaned"].as_array().expect("orphaned array");
    let o = &orphans[0];
    assert_eq!(
        o["events"], 1,
        "the orphan carries exactly its 1 real event: {stdout}"
    );
    assert!(
        o["event_types"]
            .as_array()
            .expect("event_types array")
            .iter()
            .any(|t| t == "init"),
        "orphan classification must derive from the real event types: {stdout}"
    );
}

#[test]
fn orphaned_code_missing_path_is_typed_refusal() {
    let dir = TempDir::new().expect("tempdir");
    let (code, stdout, stderr) = run_std(
        &dir,
        &["receipt", "orphaned-code", "--receipts-path", "no/such/dir"],
    );
    assert_eq!(
        code,
        xc::CLI_EXEC_ERROR,
        "typed path refusal exit: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("Path not found"),
        "path refusal must be loud: {stdout} {stderr}"
    );
}
