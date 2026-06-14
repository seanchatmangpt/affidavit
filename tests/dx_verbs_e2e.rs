// E2E witness for the DX/QOL capability verbs surfaced this session:
//   affi receipt inspect   — structural analysis (chicago-tdd-style)
//   affi receipt model     — process discovery (wasm4pm)
//   affi receipt diagnose  — LSP-shaped diagnostics (lsp-max)
//   affi receipt completions <shell> — shell completion scripts (bash/zsh/fish/powershell/nushell)
//
// Each runs the REAL binary against a REAL assembled receipt and asserts the
// command produced its capability's output. Failing-when-fake: if a verb were
// unregistered or its handler stubbed, the expected output would be absent.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary");
    cmd.current_dir(dir.path());
    cmd
}

fn build_receipt(dir: &TempDir, out: &str) {
    for (ty, obj) in [("create", "f:artifact"), ("transform", "d:artifact"), ("release", "f:artifact")] {
        affi(dir)
            .args(["receipt", "emit", "--type", ty, "--object", obj, "--payload", "-"])
            .write_stdin(ty)
            .assert()
            .success();
    }
    affi(dir).args(["receipt", "assemble", "--out", out]).assert().success();
}

#[test]
fn inspect_verb_reports_event_distribution() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "inspect", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("RECEIPT INSPECTION REPORT"))
        .stderr(predicate::str::contains("create: 1 events"))
        .stderr(predicate::str::contains("artifact:"));
}

#[test]
fn model_verb_discovers_a_process_model() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "model", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("discovered process model (wasm4pm)"))
        // the discovery output must name the receipt's activities
        .stderr(predicate::str::contains("create"))
        .stderr(predicate::str::contains("release"));
}

#[test]
fn stats_verb_aggregates_all_surfaces() {
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

#[test]
fn graph_verb_discovers_directly_follows_graph() {
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

#[test]
fn replay_verb_shows_steps_in_seq_order() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "replay", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("replay (3 events)"))
        .stderr(predicate::str::contains("step 0: create"))
        .stderr(predicate::str::contains("step 2: release"))
        .stderr(predicate::str::contains("replay complete"));
}

#[test]
fn conformance_verb_reports_fitness_and_metrics() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "conformance", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("conformance metrics:"))
        .stderr(predicate::str::contains("fitness (token replay):"))
        // honest label — activity coverage, NOT van der Aalst precision
        .stderr(predicate::str::contains("activity_coverage:"))
        .stderr(predicate::str::contains("NOT van der Aalst precision"))
        .stderr(predicate::str::contains("simplicity (Occam):"));
}

#[test]
fn diagnose_verb_reports_clean_for_honest_receipt() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "diagnose", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no diagnostics — receipt is clean"));
}

#[test]
fn mutate_verb_demonstrates_tamper_evidence() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "mutate", "r.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("receipt mutate"))
        .stderr(predicate::str::contains("original chain hash:"))
        .stderr(predicate::str::contains("tampered"))
        .stderr(predicate::str::contains("tamper-evidence confirmed"));
}

#[test]
fn bench_verb_reports_microsecond_latencies() {
    let dir = TempDir::new().expect("tempdir");
    build_receipt(&dir, "r.json");
    affi(&dir)
        .args(["receipt", "bench", "--iterations", "10"])
        .assert()
        .success()
        .stderr(predicate::str::contains("receipt bench"))
        .stderr(predicate::str::contains("build_event:"))
        .stderr(predicate::str::contains("µs/op"))
        .stderr(predicate::str::contains("bench complete"));
}

#[test]
fn help_refs_verb_prints_ardprd_map() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["receipt", "help-refs"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ARDPRD cross-reference map"))
        .stderr(predicate::str::contains("FR-1"))
        .stderr(predicate::str::contains("ADR-5"))
        .stderr(predicate::str::contains("Acceptance criterion"));
}

// ---------------------------------------------------------------------------
// `affi receipt completions <shell>` — shell completion script generation.
// Failing-when-fake: if the completions verb were unregistered or the backing
// script files were removed, the expected non-empty stdout would be absent.
// Tests cover all five supported shells: bash, zsh, fish, powershell, nushell.
// ---------------------------------------------------------------------------

#[test]
fn completions_bash_outputs_script() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["receipt", "completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh_outputs_script() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["receipt", "completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_outputs_script() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["receipt", "completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_powershell_outputs_script() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["receipt", "completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_nushell_outputs_script() {
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["receipt", "completions", "nushell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
