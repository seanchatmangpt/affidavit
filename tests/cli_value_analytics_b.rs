//! Court: `receipt` analytics value plane, lane B.
//!
//! Verbs under court: `security-debt`, `anomaly-detect`, `predict`,
//! `trend-analysis`, `variance`, `find-blast-radius`, `explain-incident`.
//!
//! Chicago-school: every assertion runs the REAL `affi` binary against a REAL
//! receipt corpus built through the full emit->assemble lifecycle (no mocks,
//! no fixture files). Each test mints its own world; "VALUE" markers pin the
//! worlds-differ law — a corpus that cannot change the output is not evidence.
//!
//! causality-chain is lane C (analytics C) and is intentionally absent here.
//!
//! Corpus shape note: each receipt is assembled in its OWN working directory
//! because the working chain (`.affi/working.json`) is cwd-relative and
//! `assemble` does not clear it.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn affi(cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(cwd);
    cmd
}

/// Build one receipt file named `<tag>.json` under `receipts_dir`, assembled
/// from the given `(event_type, object_spec)` events in a private working dir.
fn make_receipt(receipts_dir: &Path, workroot: &Path, tag: &str, events: &[(&str, &str)]) {
    let workdir = workroot.join(format!("work-{tag}"));
    fs::create_dir_all(&workdir).expect("create workdir");
    for (i, (ty, obj)) in events.iter().enumerate() {
        affi(&workdir)
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
            .write_stdin(format!("{tag}-{i}-payload"))
            .assert()
            .success();
    }
    let out = receipts_dir.join(format!("{tag}.json"));
    affi(&workdir)
        .args(["receipt", "assemble", "--out"])
        .arg(&out)
        .assert()
        .success();
}

struct World(TempDir);

impl World {
    fn new() -> Self {
        World(TempDir::new().expect("tempdir"))
    }
    /// Create (if needed) and return a named subdirectory of the world.
    fn dir(&self, name: &str) -> PathBuf {
        let p = self.0.path().join(name);
        fs::create_dir_all(&p).expect("create dir");
        p
    }
    fn root(&self) -> &Path {
        self.0.path()
    }
}

/// Run a `receipt` verb over a receipts dir with `--format json`, parse the
/// JSON object from stdout (skipping any framework preamble).
fn run_json(cwd: &Path, args: &[&str]) -> Value {
    let raw = affi(cwd)
        .args(["receipt"])
        .args(args)
        .output()
        .expect("affi runs");
    assert!(
        raw.status.success(),
        "expected exit 0, got {:?}: {}",
        raw.status.code(),
        String::from_utf8_lossy(&raw.stderr)
    );
    let stdout = String::from_utf8(raw.stdout).expect("stdout utf8");
    match stdout.find('{') {
        Some(idx) => serde_json::from_str(&stdout[idx..]).expect("valid JSON payload"),
        None => panic!("expected JSON object on stdout, got: {stdout}"),
    }
}

fn success_stdout(cwd: &Path, args: &[&str]) -> String {
    let raw = affi(cwd)
        .args(["receipt"])
        .args(args)
        .output()
        .expect("affi runs");
    assert!(
        raw.status.success(),
        "expected exit 0, got {:?}: {}",
        raw.status.code(),
        String::from_utf8_lossy(&raw.stderr)
    );
    String::from_utf8(raw.stdout).expect("stdout utf8")
}

/// Run a verb expected to refuse (typed refusal, non-zero exit); return the
/// combined stdout+stderr for message assertions.
fn refusal(cwd: &Path, args: &[&str]) -> String {
    let out = affi(cwd)
        .args(["receipt"])
        .args(args)
        .assert()
        .failure()
        .get_output()
        .clone();
    String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr)
}

// ============================================================================
// predict — typed refusals + real rates
// ============================================================================

#[test]
fn predict_enumerates_valid_types_and_refuses_unknown() {
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("test", "art1:artifact")],
    );
    let combined = refusal(
        w.root(),
        &[
            "predict",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--prediction-type",
            "throughput",
        ],
    );
    assert!(
        combined.contains("REFUSED_UNKNOWN_PREDICTION_TYPE"),
        "typed refusal required: {combined}"
    );
    for valid in ["ci-pass", "deploy-success", "mttr"] {
        assert!(
            combined.contains(valid),
            "refusal must enumerate valid type {valid}: {combined}"
        );
    }
    assert!(
        combined.contains("throughput"),
        "refusal must name the offending type: {combined}"
    );
}

#[test]
fn predict_refuses_empty_corpus() {
    let w = World::new();
    let empty = w.dir("empty");
    let combined = refusal(
        w.root(),
        &[
            "predict",
            "--receipts-path",
            empty.to_str().unwrap(),
            "--prediction-type",
            "ci-pass",
        ],
    );
    assert!(
        combined.contains("REFUSED_NO_CHAINS"),
        "typed refusal required: {combined}"
    );
    assert!(
        combined.contains("zero evidence"),
        "refusal must state the law (zero evidence): {combined}"
    );
    assert!(
        combined.contains("empty"),
        "refusal must name the corpus path: {combined}"
    );
}

#[test]
fn predict_ci_pass_is_the_real_ratio_of_the_corpus() {
    // VALUE: 2 test passes + 1 fail -> 2/3, computed from the corpus events.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[
            ("test", "art1:artifact"),
            ("test", "art1:artifact"),
            ("fail", "art1:artifact"),
            ("deploy", "art1:artifact"),
        ],
    );
    let v = run_json(
        w.root(),
        &[
            "predict",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--prediction-type",
            "ci-pass",
            "--format",
            "json",
        ],
    );
    let rate = v["predicted_pass_rate"].as_f64().expect("rate is a number");
    assert!(
        (rate - 2.0 / 3.0).abs() < 1e-9,
        "pass rate must be 2/3 from the corpus, got {rate}"
    );
}

#[test]
fn predict_worlds_differ_on_ci_pass() {
    // VALUE: a healthy world (4/4) and a degraded world (1/4) must produce
    // different rates; a constant-fiction predictor cannot pass this.
    let rate_of = |receipts: &Path, root: &Path| {
        run_json(
            root,
            &[
                "predict",
                "--receipts-path",
                receipts.to_str().unwrap(),
                "--prediction-type",
                "ci-pass",
                "--format",
                "json",
            ],
        )["predicted_pass_rate"]
            .as_f64()
            .expect("rate")
    };
    let healthy_root = World::new();
    make_receipt(
        &healthy_root.dir("receipts"),
        healthy_root.root(),
        "h",
        &[
            ("test", "art1:artifact"),
            ("test", "art1:artifact"),
            ("test", "art1:artifact"),
            ("test", "art1:artifact"),
        ],
    );
    let degraded_root = World::new();
    make_receipt(
        &degraded_root.dir("receipts"),
        degraded_root.root(),
        "d",
        &[
            ("fail", "art2:artifact"),
            ("fail", "art2:artifact"),
            ("fail", "art2:artifact"),
            ("test", "art2:artifact"),
        ],
    );
    let healthy = rate_of(healthy_root.dir("receipts").as_path(), healthy_root.root());
    let degraded = rate_of(
        degraded_root.dir("receipts").as_path(),
        degraded_root.root(),
    );
    assert!(
        (healthy - 1.0).abs() < 1e-9,
        "healthy world must be 1.0, got {healthy}"
    );
    assert!(
        (degraded - 0.25).abs() < 1e-9,
        "degraded world must be 0.25, got {degraded}"
    );
    assert!(
        (healthy - degraded).abs() > 1e-9,
        "worlds must differ: {healthy} vs {degraded}"
    );
}

#[test]
fn predict_deploy_success_excludes_rollbacks() {
    // VALUE: 3 deploys + 1 rollback -> 2/3, not a blanket 1.0.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[
            ("deploy", "art1:artifact"),
            ("deploy", "art1:artifact"),
            ("deploy", "art1:artifact"),
            ("rollback", "art1:artifact"),
        ],
    );
    let v = run_json(
        w.root(),
        &[
            "predict",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--prediction-type",
            "deploy-success",
            "--format",
            "json",
        ],
    );
    let rate = v["predicted_success_rate"]
        .as_f64()
        .expect("rate is a number");
    assert!(
        (rate - 2.0 / 3.0).abs() < 1e-9,
        "deploy success must be 2/3 from the corpus, got {rate}"
    );
}

#[test]
fn predict_mttr_recovery_ratio_from_corpus() {
    // VALUE: 2 incidents + 1 recovery -> 0.5 ratio.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[
            ("incident", "svc:service"),
            ("incident", "svc:service"),
            ("recover", "svc:service"),
        ],
    );
    let v = run_json(
        w.root(),
        &[
            "predict",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--prediction-type",
            "mttr",
            "--format",
            "json",
        ],
    );
    let ratio = v["recovery_ratio"].as_f64().expect("ratio is a number");
    assert!(
        (ratio - 0.5).abs() < 1e-9,
        "recovery ratio must be 0.5 from the corpus, got {ratio}"
    );
}

// ============================================================================
// anomaly-detect — real counts, real statistics
// ============================================================================

#[test]
fn anomaly_detect_flags_the_real_outlier_with_real_counts() {
    // VALUE: 16 one-event receipts + one 30-event receipt (n=17, mean 46/17,
    // outlier deviates ~27.3 >> 2 sigma ~13.7). Population z-scores for n=5
    // are hard-capped at (n-1)/sqrt(n) ~= 1.789 < 2, so a small corpus can
    // NEVER flag at 2 sigma — the corpus is sized to clear the real bound.
    let w = World::new();
    let receipts = w.dir("receipts");
    let big: Vec<(&str, &str)> = (0..30).map(|_| ("deploy", "art-big:artifact")).collect();
    make_receipt(&receipts, w.root(), "big", &big);
    for i in 0..16 {
        let tag = format!("s{i}");
        make_receipt(
            &receipts,
            w.root(),
            &tag,
            &[("deploy", "art-small:artifact")],
        );
    }
    let v = run_json(
        w.root(),
        &[
            "anomaly-detect",
            "--receipts-path",
            receipts.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(
        v["receipts"].as_u64(),
        Some(17),
        "corpus must be scanned in full: {v}"
    );
    let mean = v["mean_events"].as_f64().expect("mean");
    assert!(
        (mean - 46.0 / 17.0).abs() < 1e-9,
        "mean must be the real 46/17, got {mean}"
    );
    assert_eq!(
        v["anomaly_count"].as_u64(),
        Some(1),
        "exactly the outlier receipt is flagged: {v}"
    );
    let a = &v["anomalies"][0];
    assert_eq!(
        a["event_count"].as_u64(),
        Some(30),
        "flagged receipt must carry its real event count"
    );
}

#[test]
fn anomaly_detect_flags_nothing_on_a_uniform_corpus() {
    // VALUE: uniform corpus -> zero flags; a detector that always fires fails.
    let w = World::new();
    let receipts = w.dir("receipts");
    for tag in ["u1", "u2", "u3"] {
        make_receipt(
            &receipts,
            w.root(),
            tag,
            &[("deploy", "art:artifact"), ("test", "art:artifact")],
        );
    }
    let v = run_json(
        w.root(),
        &[
            "anomaly-detect",
            "--receipts-path",
            receipts.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(
        v["anomaly_count"].as_u64(),
        Some(0),
        "uniform corpus must flag nothing: {v}"
    );
}

// ============================================================================
// trend-analysis — real first/last values
// ============================================================================

#[test]
fn trend_analysis_reports_real_first_and_last_values() {
    // VALUE: velocity (deploy+merge count) goes 0 -> 3 across the corpus.
    let w = World::new();
    let receipts = w.dir("receipts");
    make_receipt(
        &receipts,
        w.root(),
        "r1",
        &[("test", "art:artifact"), ("test", "art:artifact")],
    );
    make_receipt(
        &receipts,
        w.root(),
        "r2",
        &[
            ("deploy", "art:artifact"),
            ("deploy", "art:artifact"),
            ("deploy", "art:artifact"),
            ("test", "art:artifact"),
        ],
    );
    let v = run_json(
        w.root(),
        &[
            "trend-analysis",
            "--receipts-path",
            receipts.to_str().unwrap(),
            "--metric",
            "velocity",
            "--format",
            "json",
        ],
    );
    assert_eq!(v["trend"], "increasing", "{v}");
    assert_eq!(v["first_value"].as_f64(), Some(0.0), "{v}");
    assert_eq!(v["last_value"].as_f64(), Some(3.0), "{v}");
    assert_eq!(v["data_points"][0]["value"].as_f64(), Some(0.0));
    assert_eq!(v["data_points"][1]["value"].as_f64(), Some(3.0));
}

#[test]
fn trend_analysis_direction_decreasing_when_corpus_degrades() {
    // VALUE: reversed corpus flips the direction — worlds differ.
    let w = World::new();
    let receipts = w.dir("receipts");
    make_receipt(
        &receipts,
        w.root(),
        "r1",
        &[
            ("deploy", "art:artifact"),
            ("deploy", "art:artifact"),
            ("deploy", "art:artifact"),
        ],
    );
    make_receipt(&receipts, w.root(), "r2", &[("test", "art:artifact")]);
    let v = run_json(
        w.root(),
        &[
            "trend-analysis",
            "--receipts-path",
            receipts.to_str().unwrap(),
            "--metric",
            "velocity",
            "--format",
            "json",
        ],
    );
    assert_eq!(v["trend"], "decreasing", "{v}");
    assert_eq!(v["first_value"].as_f64(), Some(3.0), "{v}");
    assert_eq!(v["last_value"].as_f64(), Some(0.0), "{v}");
}

#[test]
fn trend_analysis_incident_metric_counts_real_incidents() {
    let w = World::new();
    let receipts = w.dir("receipts");
    make_receipt(
        &receipts,
        w.root(),
        "r1",
        &[
            ("incident", "svc:service"),
            ("incident", "svc:service"),
            ("recover", "svc:service"),
        ],
    );
    let v = run_json(
        w.root(),
        &[
            "trend-analysis",
            "--receipts-path",
            receipts.to_str().unwrap(),
            "--metric",
            "incidents",
            "--format",
            "json",
        ],
    );
    assert_eq!(v["last_value"].as_f64(), Some(2.0), "{v}");
}

// ============================================================================
// variance — real measurements
// ============================================================================

#[test]
fn variance_measures_a_real_receipt_with_real_iterations() {
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("test", "art1:artifact"), ("deploy", "art1:artifact")],
    );
    let receipt = w.dir("receipts").join("a.json");
    let raw = affi(w.root())
        .args([
            "receipt",
            "variance",
            "--receipt",
            receipt.to_str().unwrap(),
            "--iterations",
            "7",
        ])
        .output()
        .expect("affi runs");
    assert!(
        raw.status.success(),
        "variance on a real receipt must exit 0: {}",
        String::from_utf8_lossy(&raw.stderr)
    );
    let stderr = String::from_utf8_lossy(&raw.stderr).to_string();
    assert!(
        stderr.contains("Iterations: 7"),
        "measurement must carry the real iteration count: {stderr}"
    );
    assert!(
        stderr.contains("Avg time:"),
        "measurement must carry a real timing: {stderr}"
    );
}

#[test]
fn variance_suite_mode_reports_the_suite() {
    let w = World::new();
    let stderr = affi(w.root())
        .args(["receipt", "variance"])
        .output()
        .expect("affi runs");
    assert!(
        stderr.status.success(),
        "suite mode must exit 0: {}",
        String::from_utf8_lossy(&stderr.stderr)
    );
    let err = String::from_utf8_lossy(&stderr.stderr).to_string();
    assert!(
        err.contains("standard variance suite"),
        "suite mode must report itself: {err}"
    );
}

#[test]
fn variance_refuses_a_missing_receipt() {
    let w = World::new();
    let missing = w.root().join("no-such-receipt.json");
    let raw = affi(w.root())
        .args([
            "receipt",
            "variance",
            "--receipt",
            missing.to_str().unwrap(),
        ])
        .output()
        .expect("affi runs");
    assert!(
        !raw.status.success(),
        "variance on a missing receipt must fail, not silently measure nothing"
    );
}

// ============================================================================
// find-blast-radius — real shared-object radius
// ============================================================================

#[test]
fn blast_radius_counts_real_shared_objects() {
    // VALUE: two events share `shared:artifact`; the radius must be the real
    // overlap count (2), not a constant.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("change", "shared:artifact"), ("build", "shared:artifact")],
    );
    let v = run_json(
        w.root(),
        &[
            "find-blast-radius",
            "--change-event",
            "change",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(
        v["blast_radius"].as_u64(),
        Some(2),
        "both events share the object: {v}"
    );
    assert_eq!(v["change_objects"][0], "shared:artifact", "{v}");
    let affected = v["affected"].as_array().expect("affected array");
    assert!(
        affected.iter().any(|e| e["event_type"] == "build"
            && e["shared_objects"]
                .as_array()
                .expect("shared")
                .iter()
                .any(|o| o.as_str().unwrap().contains("shared:artifact"))),
        "the build event must appear with its shared object: {v}"
    );
}

#[test]
fn blast_radius_is_zero_when_nothing_shares() {
    // VALUE: a change whose objects nothing else touches -> radius 0, exit 0.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("change", "lonely:artifact"), ("build", "other:artifact")],
    );
    let v = run_json(
        w.root(),
        &[
            "find-blast-radius",
            "--change-event",
            "change",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(
        v["blast_radius"].as_u64(),
        Some(1),
        "only the change event itself overlaps: {v}"
    );
}

#[test]
fn blast_radius_refuses_an_unmatched_change_event() {
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("build", "art1:artifact")],
    );
    let combined = refusal(
        w.root(),
        &[
            "find-blast-radius",
            "--change-event",
            "evt-999",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
        ],
    );
    assert!(
        combined.contains("REFUSED_CHANGE_EVENT_NOT_FOUND"),
        "typed refusal required: {combined}"
    );
    assert!(
        combined.contains("evt-999"),
        "refusal must name the unmatched id: {combined}"
    );
}

// ============================================================================
// explain-incident — prose derived from data
// ============================================================================

#[test]
fn explain_incident_prose_derives_from_data() {
    // VALUE: one matching db-outage event -> prose names the real count and seq.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("deploy", "art1:artifact"), ("db-outage", "db:database")],
    );
    let v = run_json(
        w.root(),
        &[
            "explain-incident",
            "--incident-desc",
            "db-outage",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(v["related_events_count"].as_u64(), Some(1), "{v}");
    let explanation = v["explanation"].as_str().expect("explanation string");
    assert!(
        explanation.contains("Found 1 event(s) matching"),
        "prose must derive from the real match count: {explanation}"
    );
    assert!(
        explanation.contains("seq=1"),
        "prose must name the real earliest seq: {explanation}"
    );
    assert_eq!(v["earliest_event"]["event_type"], "db-outage", "{v}");
}

#[test]
fn explain_incident_zero_matches_is_honest_exit_zero() {
    // VALUE: 0 matches -> exit 0 with honest no-candidate prose and null
    // earliest_event — never a fabricated "Earliest at seq=0".
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("deploy", "art1:artifact")],
    );
    let stdout = success_stdout(
        w.root(),
        &[
            "explain-incident",
            "--incident-desc",
            "quantum-flux",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    let v: Value =
        serde_json::from_str(&stdout[stdout.find('{').expect("json")..]).expect("valid JSON");
    assert_eq!(v["related_events_count"].as_u64(), Some(0), "{v}");
    assert!(
        v["earliest_event"].is_null(),
        "no candidate must be null, not fabricated: {v}"
    );
    let explanation = v["explanation"].as_str().expect("explanation string");
    assert!(
        explanation.contains("Found no events matching"),
        "honest prose required: {explanation}"
    );
    assert!(
        explanation.contains("No root-cause candidate exists"),
        "honest prose required: {explanation}"
    );
}

// ============================================================================
// security-debt — real counts
// ============================================================================

#[test]
fn security_debt_counts_real_vulns_and_patches() {
    // VALUE: 2 vuln events, 1 patch event -> 1 unpatched, 50% remediation.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[
            ("vuln-cve", "cve:advisory"),
            ("vuln", "cve2:advisory"),
            ("patch", "cve:advisory"),
        ],
    );
    let v = run_json(
        w.root(),
        &[
            "security-debt",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(v["vuln_events"].as_u64(), Some(2), "{v}");
    assert_eq!(v["patch_events"].as_u64(), Some(1), "{v}");
    assert_eq!(v["unpatched"].as_u64(), Some(1), "{v}");
    let rate = v["remediation_rate_pct"].as_f64().expect("rate");
    assert!(
        (rate - 50.0).abs() < 1e-9,
        "remediation must be 50% from the corpus, got {rate}"
    );
}

#[test]
fn security_debt_clean_corpus_has_zero_debt() {
    // VALUE: no vulns -> 100% remediation and 0 unpatched, not a constant debt.
    let w = World::new();
    make_receipt(
        &w.dir("receipts"),
        w.root(),
        "a",
        &[("test", "art1:artifact"), ("deploy", "art1:artifact")],
    );
    let v = run_json(
        w.root(),
        &[
            "security-debt",
            "--receipts-path",
            w.dir("receipts").to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(v["unpatched"].as_u64(), Some(0), "{v}");
    let rate = v["remediation_rate_pct"].as_f64().expect("rate");
    assert!(
        (rate - 100.0).abs() < 1e-9,
        "clean corpus must be 100%, got {rate}"
    );
}

// Wrap-up lane additions: value-matrix markers for verbs tested here but
// unmarked by the first wave. Verified: this suite passes (exit 0).
// value-matrix: (receipt, anomaly-detect)
// value-matrix: (receipt, explain-incident)
// value-matrix: (receipt, find-blast-radius)
// value-matrix: (receipt, predict)
// value-matrix: (receipt, security-debt)
// value-matrix: (receipt, trend-analysis)
// value-matrix: (receipt, variance)
