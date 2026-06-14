// E2E cross-library witness: the full affidavit DX pipeline through the REAL
// binary — emit → assemble → verify → inspect → model → conformance → diagnose —
// in one run, proving the integrated libraries compose at the binary boundary.
//
// This is the maximalist payoff witnessed end-to-end: one receipt flows through
// ggen/clap-noun-verb (CLI), the affidavit court (verify), wasm4pm (model +
// conformance), and lsp-max (diagnose). Failing-when-fake: any stage breaking
// (unregistered verb, stubbed discovery, broken court) fails its assertion.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("affi").expect("affi binary");
    c.current_dir(dir.path());
    c
}

#[test]
fn full_dx_pipeline_through_the_binary() {
    let dir = TempDir::new().expect("tempdir");

    // 1. emit three events (ggen/clap-noun-verb CLI + affidavit chain)
    for (ty, obj) in [("create", "f:artifact"), ("transform", "d:artifact"), ("release", "f:artifact")] {
        affi(&dir)
            .args(["receipt", "emit", "--type", ty, "--object", obj, "--payload", "-"])
            .write_stdin(ty)
            .assert()
            .success();
    }
    // 2. assemble → immutable receipt
    affi(&dir).args(["receipt", "assemble", "--out", "r.json"]).assert().success();

    // 3. verify → ACCEPT (affidavit court: OCEL + chain)
    affi(&dir).args(["receipt", "verify", "r.json"]).assert().success()
        .stderr(predicate::str::contains("verdict: ACCEPT"));

    // 4. inspect → structural analysis (chicago-tdd-flavored)
    affi(&dir).args(["receipt", "inspect", "r.json"]).assert().success()
        .stderr(predicate::str::contains("RECEIPT INSPECTION REPORT"));

    // 5. model → process discovery (wasm4pm) names the activities
    affi(&dir).args(["receipt", "model", "r.json"]).assert().success()
        .stderr(predicate::str::contains("create"))
        .stderr(predicate::str::contains("release"));

    // 6. conformance → fitness/activity_coverage/simplicity (wasm4pm token replay)
    affi(&dir).args(["receipt", "conformance", "r.json"]).assert().success()
        .stderr(predicate::str::contains("fitness (token replay):"));

    // 7. diagnose → clean (lsp-max diagnostics)
    affi(&dir).args(["receipt", "diagnose", "r.json"]).assert().success()
        .stderr(predicate::str::contains("no diagnostics — receipt is clean"));
}
