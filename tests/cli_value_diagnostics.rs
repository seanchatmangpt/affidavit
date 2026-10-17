//! Court: value-diagnostics verbs over the real `affi` binary —
//! `receipt why | fix | diagnose | diff | graph | replay | timeline`.
//!
//! Chicago discipline: every assertion lands on real process output (exit
//! codes, stdout/stderr text) produced by the built binary against real
//! emitted+assembled receipts. No mocks, no fakes. Each test is marker-tagged
//! per verb so a failing assertion names its verb lane.
//!
//! Typed-exit law under test: CLI execution errors are `CliExecutionError`
//! (1504), which encodes as 1504 % 256 == 224 on the process exit status
//! (`src/diag.rs`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Local harness (tests/common/mod.rs is absent in this tree)
// ---------------------------------------------------------------------------

const EXIT_OK: i32 = 0;
const EXIT_EXECUTION_ERROR: i32 = 1504 % 256; // 224
const EXIT_VERIFY_REJECT: i32 = 2;

struct World {
    _dir: TempDir,
    cwd: PathBuf,
}

impl World {
    fn new(tag: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let cwd = dir.path().join(tag);
        std::fs::create_dir_all(&cwd).expect("mkdir");
        World { _dir: dir, cwd }
    }

    fn affi(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_affi"))
            .args(args)
            .current_dir(&self.cwd)
            .output()
            .expect("spawn affi")
    }

    fn run(&self, args: &[&str]) -> (i32, String, String) {
        let out = self.affi(args);
        (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    /// Emit two real events and assemble an honest, chain-valid receipt.
    /// Returns (honest_path, tampered_path); the tampered copy has one event
    /// type flipped and its chain_hash zeroed, so chain_integrity must fail.
    fn receipt_pair(&self) -> (PathBuf, PathBuf) {
        let p1 = self.cwd.join("p1.txt");
        let p2 = self.cwd.join("p2.txt");
        std::fs::write(&p1, "source bytes for stage one\n").expect("write p1");
        std::fs::write(&p2, "source bytes for stage two\n").expect("write p2");

        let (code, _, stderr) = self.run(&[
            "receipt",
            "emit",
            "--type",
            "seed",
            "--object",
            "art1:artifact:input",
            "--payload",
            "p1.txt",
        ]);
        assert_eq!(code, EXIT_OK, "[emit #1] must succeed: {stderr}");
        let (code, _, stderr) = self.run(&[
            "receipt",
            "emit",
            "--type",
            "validate",
            "--object",
            "art1:artifact:output",
            "--payload",
            "p2.txt",
        ]);
        assert_eq!(code, EXIT_OK, "[emit #2] must succeed: {stderr}");

        let (code, stdout, stderr) = self.run(&["receipt", "assemble", "--out", "receipt.json"]);
        assert_eq!(code, EXIT_OK, "[assemble] must succeed: {stderr}");
        assert!(
            stdout.contains("assembled receipt"),
            "[assemble] must report assembly: {stdout}"
        );

        let honest = self.cwd.join("honest.json");
        let tampered = self.cwd.join("tampered.json");
        std::fs::copy(self.cwd.join("receipt.json"), &honest).expect("copy honest");
        std::fs::copy(self.cwd.join("receipt.json"), &tampered).expect("copy tampered");
        // Real byte-level tamper: flip one event_type and zero the chain hash.
        let body = std::fs::read_to_string(&tampered).expect("read tampered");
        let body = body.replacen("\"seed\"", "\"tampered\"", 1).replacen(
            &format!("\"chain_hash\": \"{}\"", "0".repeat(0)),
            &format!("\"chain_hash\": \"{}\"", "0".repeat(64)),
            1,
        );
        std::fs::write(&tampered, body).expect("write tampered");
        (honest, tampered)
    }
}

// ---------------------------------------------------------------------------
// receipt why — ACCEPT vs REJECT must be explained differently
// ---------------------------------------------------------------------------

/// [why] Honest receipt: ACCEPT framing, no failing stage, exit 0.
#[test]
fn why_honest_receipt_explains_accept() {
    let w = World::new("why-accept");
    let (honest, _tampered) = w.receipt_pair();
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "why",
        "--receipt",
        honest.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(code, EXIT_OK, "[why] honest must exit 0: {stdout}{stderr}");
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("ACCEPT"),
        "[why] honest output must say ACCEPT: {text}"
    );
    assert!(
        !text.contains("REJECT"),
        "[why] honest output must not say REJECT: {text}"
    );
    assert!(
        !text.contains("STAGE FAILED"),
        "[why] honest output must list no failing stage: {text}"
    );
}

/// [why] Tampered receipt: REJECT framing, chain_integrity named, remediation present.
#[test]
fn why_tampered_receipt_explains_reject() {
    let w = World::new("why-reject");
    let (_honest, tampered) = w.receipt_pair();
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "why",
        "--receipt",
        tampered.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        code, EXIT_OK,
        "[why] is an explainer: it must not crash: {stdout}{stderr}"
    );
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("REJECT"),
        "[why] tampered output must say REJECT: {text}"
    );
    assert!(
        text.contains("chain_integrity"),
        "[why] failing stage must be named: {text}"
    );
    assert!(
        text.contains("STAGE FAILED"),
        "[why] must show stage-by-stage remediation: {text}"
    );
    assert!(text.contains("Fix:"), "[why] must carry a fix hint: {text}");
}

/// [why] Value law: tampered-vs-honest worlds produce different explanations.
#[test]
fn why_distinguishes_the_two_worlds() {
    let w = World::new("why-worlds");
    let (honest, tampered) = w.receipt_pair();
    let name = |p: &Path| p.file_name().unwrap().to_str().unwrap().to_string();
    let (_, out_honest, _) = w.run(&["receipt", "why", "--receipt", &name(&honest)]);
    let (_, out_tampered, _) = w.run(&["receipt", "why", "--receipt", &name(&tampered)]);
    assert_ne!(
        out_honest, out_tampered,
        "[why] ACCEPT world and REJECT world must be explained differently"
    );
}

// ---------------------------------------------------------------------------
// receipt fix — quarantine must really quarantine, deterministically
// ---------------------------------------------------------------------------

/// [fix] Missing file is a typed CliExecutionError (224), never a panic.
#[test]
fn fix_missing_file_is_typed_224() {
    let w = World::new("fix-missing");
    let (code, _, stderr) = w.run(&["receipt", "fix", "--receipt", "no-such-receipt.json"]);
    assert_eq!(
        code, EXIT_EXECUTION_ERROR,
        "[fix] missing file must exit 224: {stderr}"
    );
    assert!(
        stderr.contains("Receipt not found"),
        "[fix] typed message must name the failure: {stderr}"
    );
}

/// [fix] Quarantine moves the subject out of place, writes a sidecar, and the
/// sidecar is a pure function of the subject (byte-identical across runs).
#[test]
fn fix_quarantine_moves_file_and_sidecar_is_deterministic() {
    let w = World::new("fix-quarantine");
    let (_honest, tampered) = w.receipt_pair();
    let subject = "tampered.json";
    let sidecar = "tampered.quarantine.why.json";
    let quarantined = "tampered.quarantine.json";

    let (code, stdout, stderr) = w.run(&["receipt", "fix", "--receipt", subject]);
    assert_eq!(
        code, EXIT_OK,
        "[fix] quarantine must exit 0: {stdout}{stderr}"
    );
    assert!(
        !w.cwd.join(subject).exists(),
        "[fix] subject must be moved out of place"
    );
    assert!(
        w.cwd.join(quarantined).exists(),
        "[fix] quarantined copy must exist: {stdout}"
    );
    assert!(
        stdout.contains(sidecar) || w.cwd.join(sidecar).exists(),
        "[fix] sidecar must be written: {stdout}"
    );

    let first = std::fs::read(w.cwd.join(sidecar)).expect("read sidecar run 1");
    assert!(!first.is_empty(), "[fix] sidecar must be non-empty");

    // Restore the identical subject and quarantine again: bytes must match.
    std::fs::copy(w.cwd.join(quarantined), w.cwd.join(subject)).expect("restore subject");
    let (code, _, stderr) = w.run(&["receipt", "fix", "--receipt", subject]);
    assert_eq!(
        code, EXIT_OK,
        "[fix] second quarantine must exit 0: {stderr}"
    );
    let second = std::fs::read(w.cwd.join(sidecar)).expect("read sidecar run 2");
    assert_eq!(
        first, second,
        "[fix] sidecar must be byte-identical across runs (pure function of subject)"
    );
}

// ---------------------------------------------------------------------------
// receipt diagnose — must mirror the verify verdict
// ---------------------------------------------------------------------------

/// [diagnose] Honest world: clean/ACCEPT, zero diagnostics — mirrors verify ACCEPT.
#[test]
fn diagnose_mirrors_verify_on_honest_receipt() {
    let w = World::new("diag-honest");
    let (honest, _tampered) = w.receipt_pair();
    let (vcode, _, _) = w.run(&[
        "receipt",
        "verify",
        "--receipt",
        honest.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        vcode, EXIT_OK,
        "[diagnose] precondition: honest verify must ACCEPT"
    );
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "diagnose",
        honest.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        code, EXIT_OK,
        "[diagnose] honest must exit 0: {stdout}{stderr}"
    );
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("clean") && text.contains("ACCEPT"),
        "[diagnose] honest must mirror verify ACCEPT: {text}"
    );
    assert!(
        !text.contains("FAIL"),
        "[diagnose] honest must report no FAIL stage: {text}"
    );
}

/// [diagnose] Tampered world: chain_integrity FAIL — mirrors verify REJECT (exit 2).
#[test]
fn diagnose_mirrors_verify_on_tampered_receipt() {
    let w = World::new("diag-tampered");
    let (_honest, tampered) = w.receipt_pair();
    let (vcode, _, _) = w.run(&[
        "receipt",
        "verify",
        "--receipt",
        tampered.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        vcode, EXIT_VERIFY_REJECT,
        "[diagnose] precondition: tampered verify must REJECT with exit 2"
    );
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "diagnose",
        tampered.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        code, EXIT_OK,
        "[diagnose] is an explainer: it must not crash: {stdout}{stderr}"
    );
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("FAIL"),
        "[diagnose] tampered must show a FAIL stage: {text}"
    );
    assert!(
        text.contains("chain_integrity"),
        "[diagnose] must name the same failing stage verify reports: {text}"
    );
}

/// [diagnose] Value law: tampered-vs-honest worlds produce different diagnostics.
#[test]
fn diagnose_distinguishes_the_two_worlds() {
    let w = World::new("diag-worlds");
    let (honest, tampered) = w.receipt_pair();
    let name = |p: &Path| p.file_name().unwrap().to_str().unwrap().to_string();
    let (_, out_honest, _) = w.run(&["receipt", "diagnose", &name(&honest)]);
    let (_, out_tampered, _) = w.run(&["receipt", "diagnose", &name(&tampered)]);
    assert_ne!(
        out_honest, out_tampered,
        "[diagnose] ACCEPT world and REJECT world must diagnose differently"
    );
}

// ---------------------------------------------------------------------------
// receipt diff — structural difference between two real chains
// ---------------------------------------------------------------------------

/// [diff] Two real chains built from different event sequences must report
/// added events with a + prefix and a summary count.
#[test]
fn diff_reports_added_events_between_two_real_chains() {
    let w = World::new("diff-added");
    let (honest, _tampered) = w.receipt_pair();

    // Build a second, longer real chain in a separate working store.
    let dir2 = w.cwd.join("chain-b");
    std::fs::create_dir_all(&dir2).expect("mkdir chain-b");
    std::fs::write(dir2.join("q1.txt"), "chain b bytes\n").expect("write q1");
    let emit = |args: &[&str]| {
        let (code, _, stderr) = w.run(args);
        assert_eq!(code, EXIT_OK, "[diff] chain-b emit must succeed: {stderr}");
    };
    // Note: emit/assemble operate on the cwd store; run chain-b via explicit cwd.
    let _ = &dir2;
    let _ = emit;
    // Simplest real second chain: emit into a fresh temp dir.
    let w2 = World::new("diff-added-b");
    std::fs::write(w2.cwd.join("q1.txt"), "chain b bytes\n").expect("write q1");
    std::fs::write(w2.cwd.join("q2.txt"), "more chain b bytes\n").expect("write q2");
    for (ty, obj, payload) in [
        ("seed", "art2:artifact:input", "q1.txt"),
        ("audit", "art2:artifact:middle", "q2.txt"),
        ("validate", "art2:artifact:output", "q2.txt"),
    ] {
        let (code, _, stderr) = w2.run(&[
            "receipt",
            "emit",
            "--type",
            ty,
            "--object",
            obj,
            "--payload",
            payload,
        ]);
        assert_eq!(
            code, EXIT_OK,
            "[diff] chain-b emit {ty} must succeed: {stderr}"
        );
    }
    let (code, _, stderr) = w2.run(&["receipt", "assemble", "--out", "receipt.json"]);
    assert_eq!(
        code, EXIT_OK,
        "[diff] chain-b assemble must succeed: {stderr}"
    );
    let chain_b = w2.cwd.join("receipt.json");
    let chain_b_dst = w.cwd.join("chain-b.json");
    std::fs::copy(&chain_b, &chain_b_dst).expect("copy chain-b");

    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "diff",
        honest.file_name().unwrap().to_str().unwrap(),
        chain_b_dst.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        code, EXIT_OK,
        "[diff] two real chains must diff cleanly: {stderr}"
    );
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("+ "),
        "[diff] must report added events: {text}"
    );
    assert!(
        text.contains("audit"),
        "[diff] the added 'audit' event must appear: {text}"
    );
    assert!(
        text.contains("added, ") && text.contains(" removed, "),
        "[diff] must print the added/removed/modified summary: {text}"
    );
    assert!(
        !text.contains("No differences found"),
        "[diff] two different chains must not be reported identical: {text}"
    );
}

/// [diff] Structurally invalid input is refused with a typed 224.
#[test]
fn diff_refuses_structurally_invalid_input_typed() {
    let w = World::new("diff-invalid");
    let (honest, _tampered) = w.receipt_pair();
    std::fs::write(w.cwd.join("junk.json"), "this is not json").expect("write junk");
    let (code, _, stderr) = w.run(&[
        "receipt",
        "diff",
        honest.file_name().unwrap().to_str().unwrap(),
        "junk.json",
    ]);
    assert_eq!(
        code, EXIT_EXECUTION_ERROR,
        "[diff] invalid input must exit 224: {stderr}"
    );
    assert!(
        stderr.contains("Failed to parse"),
        "[diff] typed message must name the parse failure: {stderr}"
    );
}

/// [diff] A tampered receipt (parseable but chain-invalid) is refused typed 224.
#[test]
fn diff_refuses_tampered_receipt_typed_224() {
    let w = World::new("diff-tampered");
    let (honest, tampered) = w.receipt_pair();
    let (code, _, stderr) = w.run(&[
        "receipt",
        "diff",
        honest.file_name().unwrap().to_str().unwrap(),
        tampered.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(
        code, EXIT_EXECUTION_ERROR,
        "[diff] tampered input must exit 224: {stderr}"
    );
    assert!(
        stderr.contains("chain hash mismatch"),
        "[diff] typed message must name the chain mismatch: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// receipt graph — DFG derived from the real event sequence
// ---------------------------------------------------------------------------

/// [graph] 2 events of 2 distinct types → 2 nodes, 1 df-edge, correct start/end.
#[test]
fn graph_derives_nodes_and_edges_from_real_event_sequence() {
    let w = World::new("graph");
    let (honest, _tampered) = w.receipt_pair();
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "graph",
        honest.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(code, EXIT_OK, "[graph] must exit 0: {stdout}{stderr}");
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("nodes (activities): 2"),
        "[graph] 2 distinct event types → 2 nodes: {text}"
    );
    assert!(
        text.contains("edges (df-relations): 1"),
        "[graph] 2 events → 1 directly-follows edge: {text}"
    );
    assert!(
        text.contains("start activities: seed"),
        "[graph] start must come from the real first event: {text}"
    );
    assert!(
        text.contains("end activities: validate"),
        "[graph] end must come from the real last event: {text}"
    );
}

/// [graph] Missing path is typed 224.
#[test]
fn graph_missing_path_typed_224() {
    let w = World::new("graph-missing");
    let (code, _, stderr) = w.run(&["receipt", "graph", "nope.json"]);
    assert_eq!(
        code, EXIT_EXECUTION_ERROR,
        "[graph] missing path must exit 224: {stderr}"
    );
    assert!(
        stderr.contains("nope.json"),
        "[graph] must name the missing path: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// receipt replay — traces the real chain
// ---------------------------------------------------------------------------

/// [replay] Each real event gets a step trace in lawful seq order.
#[test]
fn replay_traces_the_real_chain() {
    let w = World::new("replay");
    let (honest, _tampered) = w.receipt_pair();
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "replay",
        honest.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(code, EXIT_OK, "[replay] must exit 0: {stdout}{stderr}");
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("replay (2 events)"),
        "[replay] must count the real events: {text}"
    );
    assert!(
        text.contains("step 0: seed"),
        "[replay] step 0 must be the real first event: {text}"
    );
    assert!(
        text.contains("step 1: validate"),
        "[replay] step 1 must be the real second event: {text}"
    );
    assert!(
        text.contains("replay complete — 2 steps"),
        "[replay] must close with the step count: {text}"
    );
}

/// [replay] Missing path is typed 224.
#[test]
fn replay_missing_path_typed_224() {
    let w = World::new("replay-missing");
    let (code, _, stderr) = w.run(&["receipt", "replay", "nope.json"]);
    assert_eq!(
        code, EXIT_EXECUTION_ERROR,
        "[replay] missing path must exit 224: {stderr}"
    );
    assert!(
        stderr.contains("nope.json"),
        "[replay] must name the missing path: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// receipt timeline — lists real events
// ---------------------------------------------------------------------------

/// [timeline] Lists the real events of the chain with types and event ids.
#[test]
fn timeline_lists_real_events() {
    let w = World::new("timeline");
    let (honest, _tampered) = w.receipt_pair();
    let (code, stdout, stderr) = w.run(&[
        "receipt",
        "timeline",
        "--receipts-path",
        honest.file_name().unwrap().to_str().unwrap(),
    ]);
    assert_eq!(code, EXIT_OK, "[timeline] must exit 0: {stdout}{stderr}");
    let text = format!("{stdout}{stderr}");
    assert!(
        text.contains("2 total events"),
        "[timeline] must count real events: {text}"
    );
    assert!(
        text.contains("\"seed\""),
        "[timeline] must list the real seed event: {text}"
    );
    assert!(
        text.contains("\"validate\""),
        "[timeline] must list the real validate event: {text}"
    );
    assert!(
        text.contains("evt-0") && text.contains("evt-1"),
        "[timeline] must carry real event ids: {text}"
    );
}

/// [timeline] Missing path is typed 224.
#[test]
fn timeline_missing_path_typed_224() {
    let w = World::new("timeline-missing");
    let (code, _, stderr) = w.run(&["receipt", "timeline", "--receipts-path", "nope.json"]);
    assert_eq!(
        code, EXIT_EXECUTION_ERROR,
        "[timeline] missing path must exit 224: {stderr}"
    );
    assert!(
        stderr.contains("nope.json"),
        "[timeline] must name the missing path: {stderr}"
    );
}
