// Golden output tests — lock in the structural format of CLI output for all display verbs.
//
// Each test builds a minimal honest receipt (3 events) using the real binary, then
// asserts that the target verb's stderr output contains the expected structural
// keywords. Failures here mean a handler changed its output format without updating
// this guard.
//
// Pattern mirrors dx_verbs_e2e.rs and e2e.rs: `assert_cmd` + `predicates`, real
// binary invoked via `Command::cargo_bin("affi")`.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary");
    cmd.current_dir(dir.path());
    cmd
}

/// Build a 3-event honest receipt at `out` inside `dir`.
fn build_receipt(dir: &TempDir, out: &str) {
    for (ty, obj) in [
        ("create", "src:artifact"),
        ("transform", "data:artifact"),
        ("release", "app:artifact"),
    ] {
        affi(dir)
            .args([
                "receipt", "emit", "--type", ty, "--object", obj, "--payload", "-",
            ])
            .write_stdin(ty)
            .assert()
            .success();
    }
    affi(dir)
        .args(["receipt", "assemble", "--out", out])
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// 1. verify — exit 0 + ACCEPT verdict on stderr
// ---------------------------------------------------------------------------
#[test]
fn golden_verify_exits_zero_and_emits_accept() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "verify", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("verdict: ACCEPT"));
}

// ---------------------------------------------------------------------------
// 2. show — structural header: format version, event count, chain hash
// ---------------------------------------------------------------------------
#[test]
fn golden_show_contains_structural_header() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "show", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("receipt format:"))
        .stderr(predicate::str::contains("events: 3"))
        .stderr(predicate::str::contains("chain hash:"));
}

// ---------------------------------------------------------------------------
// 3. graph — DFG node/edge counts from wasm4pm discovery
// ---------------------------------------------------------------------------
#[test]
fn golden_graph_contains_dfg_node_and_edge_counts() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "graph", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("directly-follows graph (wasm4pm)"))
        .stderr(predicate::str::contains("nodes (activities):"))
        .stderr(predicate::str::contains("edges (df-relations):"));
}

// ---------------------------------------------------------------------------
// 4. stats — aggregate surface: events, dfg, fitness
// ---------------------------------------------------------------------------
#[test]
fn golden_stats_contains_event_counts_and_fitness() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "stats", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("receipt stats:"))
        .stderr(predicate::str::contains("events: 3"))
        .stderr(predicate::str::contains("dfg:"))
        .stderr(predicate::str::contains("fitness:"));
}

// ---------------------------------------------------------------------------
// 5. model — process model discovery output references wasm4pm and activities
// ---------------------------------------------------------------------------
#[test]
fn golden_model_contains_wasm4pm_process_model_header() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("discovered process model (wasm4pm)"))
        .stderr(predicate::str::contains("create"))
        .stderr(predicate::str::contains("release"));
}

// ---------------------------------------------------------------------------
// 6. conformance — fitness (token replay) + simplicity (Occam) metrics
// ---------------------------------------------------------------------------
#[test]
fn golden_conformance_contains_fitness_and_simplicity() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "conformance", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("conformance metrics:"))
        .stderr(predicate::str::contains("fitness (token replay):"))
        .stderr(predicate::str::contains("simplicity (Occam):"));
}

// ---------------------------------------------------------------------------
// 7. diagnose — clean honest receipt produces no diagnostic squiggles
// ---------------------------------------------------------------------------
#[test]
fn golden_diagnose_reports_clean_for_honest_receipt() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "diagnose", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no diagnostics — receipt is clean"));
}

// ---------------------------------------------------------------------------
// 8. replay — step-by-step trace with → arrow notation and completion line
// ---------------------------------------------------------------------------
#[test]
fn golden_replay_shows_steps_with_arrow_notation() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "replay", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("replay (3 events)"))
        .stderr(predicate::str::contains("→"))
        .stderr(predicate::str::contains("replay complete"));
}

// ---------------------------------------------------------------------------
// 9. inspect — structural analysis report (chicago-tdd fixture analysis)
// ---------------------------------------------------------------------------
#[test]
fn golden_inspect_emits_nonempty_inspection_report() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "inspect", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("RECEIPT INSPECTION REPORT"))
        .stderr(predicate::str::contains("artifact:"));
}
