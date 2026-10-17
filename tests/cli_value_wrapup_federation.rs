//! Wrap-up lane: federation nouns — `standing`, `ecosystem`, `errc` — value
//! matrix coverage over the real `affi` binary (Chicago style: real
//! subprocesses, real sealed receipts, assertions on real process state).
//!
//! Each verb gets a positive control (real sealed receipt, exit 0, value
//! asserts on the sealed artifact) and a negative control (typed refusal or
//! REJECT exit 2). Local harness (tests/common absent for this lane).

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

// value-matrix: (standing, certify)
// value-matrix: (standing, verify)
// value-matrix: (ecosystem, certify)
// value-matrix: (ecosystem, verify)
// value-matrix: (errc, certify)
// value-matrix: (errc, verify)
// value-matrix: (errc, assure)
// value-matrix: (errc, verify-assurance)

const EXIT_OK: i32 = 0;
const REJECT: i32 = 2;

fn run_in(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out: Output = Command::new(env!("CARGO_BIN_EXE_affi"))
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

fn write(dir: &Path, name: &str, contents: &str) -> String {
    fs::write(dir.join(name), contents).expect("fixture written");
    name.to_string()
}

fn read_json(dir: &Path, name: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(dir.join(name)).expect("artifact readable"))
        .expect("artifact is JSON")
}

/// Emit one real event and assemble it into `source.json` — the admitted
/// subject every federation law certifies over.
fn admitted_source(dir: &Path) -> String {
    let payload = dir.join("p.txt");
    fs::write(&payload, "exact-head execution\n").expect("payload");
    let (code, _, stderr) = run_in(
        dir,
        &[
            "receipt",
            "emit",
            "--event-type",
            "build",
            "--object",
            "repo:git",
            "--payload",
            payload.to_str().expect("utf8 payload path"),
        ],
    );
    assert_eq!(code, EXIT_OK, "[fixture emit] {stderr}");
    let (code, _, stderr) = run_in(dir, &["receipt", "assemble", "--out", "source.json"]);
    assert_eq!(code, EXIT_OK, "[fixture assemble] {stderr}");
    "source.json".to_string()
}

const ALIVE_OBSERVATION: &str = r#"{
  "subject": {
    "subject": "seanchatmangpt/affidavit",
    "base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "candidate": "cccccccccccccccccccccccccccccccccccccccc"
  },
  "observation_commitment": "0000000000000000000000000000000000000000000000000000000000000000",
  "standing": "ALIVE",
  "execution": {
    "command": "cargo test --all-targets",
    "exit_code": 0,
    "result_commitment": "1111111111111111111111111111111111111111111111111111111111111111"
  },
  "verification": {
    "command": "cargo clippy --all-targets -- -D warnings",
    "exit_code": 0,
    "report_commitment": "2222222222222222222222222222222222222222222222222222222222222222"
  },
  "replay": {
    "command": "cargo test --test cli_value_wrapup_federation",
    "environment_commitment": "3333333333333333333333333333333333333333333333333333333333333333"
  },
  "previous_receipt": null
}"#;

fn seal_standing(dir: &Path, source: &str, out: &str) -> (i32, String, String) {
    let observation = write(dir, "standing-observation.json", ALIVE_OBSERVATION);
    run_in(
        dir,
        &[
            "standing",
            "certify",
            "--receipt",
            source,
            "--observation",
            &observation,
            "--scope",
            "repo:seanchatmangpt/affidavit",
            "--out",
            out,
        ],
    )
}

/// ERRC observation: one lawful ELIMINATE claim (baseline 4 -> candidate 0).
fn errc_observation_json() -> String {
    serde_json::json!({
        "subject": {
            "subject": "seanchatmangpt/affidavit",
            "base": "a".repeat(40),
            "tree": "b".repeat(40),
            "candidate": "c".repeat(40),
        },
        "observation_commitment": "5".repeat(64),
        "claims": [{
            "id": "eliminate-unreachable-kernel",
            "target": "affi CLI surface over the federation kernel",
            "quadrant": "ELIMINATE",
            "measure": {
                "metric": "kernel_certifiers_without_cli_surface",
                "unit": "certifiers",
                "baseline": 4,
                "candidate": 0,
            },
            "evidence_commitment": "6".repeat(64),
        }],
        "preserved_invariants": [{
            "id": "certify-not-decide",
            "statement": "The CLI reaches the kernel's laws; it never adds one.",
            "evidence_commitment": "7".repeat(64),
        }],
        "replay": {
            "command": "cargo test --test cli_value_wrapup_federation",
            "environment_commitment": "8".repeat(64),
        },
        "previous_receipt": null,
    })
    .to_string()
}

fn seal_errc(dir: &Path, source: &str) -> (i32, String, String) {
    let obs = write(dir, "errc-observation.json", &errc_observation_json());
    run_in(
        dir,
        &[
            "errc",
            "certify",
            "--receipt",
            source,
            "--observation",
            &obs,
            "--out",
            "errc.json",
        ],
    )
}

fn witness_ledger() -> String {
    serde_json::json!([{
        "claim_id": "eliminate-unreachable-kernel",
        "verifier": "cargo test --test cli_value_wrapup_federation",
        "evidence_locator": "tests/cli_value_wrapup_federation.rs",
        "observed_result": "federation verbs reachable from the affi binary",
        "evidence_commitment": "9".repeat(64),
        "exclusions": ["Does not establish operational utility."],
    }])
    .to_string()
}

// ---------------------------------------------------------------------------
// standing
// ---------------------------------------------------------------------------

/// [standing certify + standing verify] Positive: an ALIVE observation seals
/// a standing/v2 receipt whose standing is ALIVE, and verify re-runs the law.
#[test]
fn standing_certify_then_verify_value() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, stdout, stderr) = seal_standing(dir.path(), &source, "standing.json");
    assert_eq!(
        code, EXIT_OK,
        "[standing certify] out={stdout} err={stderr}"
    );

    let sealed = read_json(dir.path(), "standing.json");
    assert_eq!(
        sealed["profile"], "affidavit/standing/v2",
        "value: profile must name the standing v2 law"
    );
    assert_eq!(
        sealed["standing"], "ALIVE",
        "value: an ALIVE observation must certify ALIVE"
    );
    assert_eq!(
        sealed["authority"]["scope"], "repo:seanchatmangpt/affidavit",
        "value: the authority scope is bound into the sealed receipt"
    );

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &["standing", "verify", "--receipt", "standing.json"],
    );
    assert_eq!(code, EXIT_OK, "[standing verify] out={stdout} err={stderr}");
    assert!(
        stdout.contains("ACCEPT") || stdout.to_lowercase().contains("alive"),
        "value: verify must affirm the sealed standing; out={stdout}"
    );
}

/// [standing verify] Negative: a byte-level tamper of the sealed receipt must
/// REJECT at exit 2 — the law re-runs, it does not trust the seal.
#[test]
fn standing_verify_rejects_a_tampered_receipt() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, _, stderr) = seal_standing(dir.path(), &source, "standing.json");
    assert_eq!(code, EXIT_OK, "[fixture seal] {stderr}");

    let body = fs::read_to_string(dir.path().join("standing.json")).expect("read");
    let tampered = body.replace("ALIVE", "ZOMBIE");
    assert_ne!(body, tampered, "tamper must actually change the bytes");
    fs::write(dir.path().join("tampered.json"), tampered).expect("write tampered");

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &["standing", "verify", "--receipt", "tampered.json"],
    );
    assert_eq!(
        code, REJECT,
        "[standing verify tampered] out={stdout} err={stderr}"
    );
}

/// [standing certify] Negative: an unbounded (blank) authority scope is
/// refused before anything is sealed — no artifact is written.
#[test]
fn standing_certify_refuses_an_unbounded_scope() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let observation = write(dir.path(), "standing-observation.json", ALIVE_OBSERVATION);
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &observation,
            "--scope",
            "   ",
            "--out",
            "unbounded.json",
        ],
    );
    assert_eq!(
        code, REJECT,
        "[standing certify blank scope] out={stdout} err={stderr}"
    );
    assert!(
        stderr.to_lowercase().contains("authority"),
        "value: refusal must name the authority law; err={stderr}"
    );
    assert!(
        !dir.path().join("unbounded.json").exists(),
        "value: a refused certification must not seal an artifact"
    );
}

// ---------------------------------------------------------------------------
// ecosystem
// ---------------------------------------------------------------------------

/// [ecosystem certify + ecosystem verify] Positive: one sealed ALIVE member
/// satisfies a RUNTIME_EXECUTION quorum of 1; verify re-runs the law.
#[test]
fn ecosystem_certify_federates_then_verifies() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, _, stderr) = seal_standing(dir.path(), &source, "member.json");
    assert_eq!(code, EXIT_OK, "[fixture member seal] {stderr}");
    let member = read_json(dir.path(), "member.json");

    let federation = serde_json::json!({
        "subject": {
            "subject": "chatman/ecosystem",
            "base": "a".repeat(40),
            "tree": "b".repeat(40),
            "candidate": "c".repeat(40),
        },
        "observation_commitment": "4".repeat(64),
        "requirements": [{ "role": "RUNTIME_EXECUTION", "minimum_alive": 1 }],
        "members": [{ "role": "RUNTIME_EXECUTION", "standing_receipt": member }],
        "previous_receipt": null,
    });
    let path = write(
        dir.path(),
        "federation.json",
        &serde_json::to_string_pretty(&federation).expect("serializes"),
    );

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "ecosystem",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--out",
            "ecosystem.json",
        ],
    );
    assert_eq!(
        code, EXIT_OK,
        "[ecosystem certify] out={stdout} err={stderr}"
    );

    let sealed = read_json(dir.path(), "ecosystem.json");
    assert_eq!(sealed["profile"], "affidavit/ecosystem/v1");
    assert_eq!(
        sealed["standing"], "ALIVE",
        "value: quorum 1/1 ALIVE members must reach ALIVE"
    );
    assert_eq!(
        sealed["coverage"][0]["satisfied"], true,
        "value: the declared RUNTIME_EXECUTION quorum is satisfied"
    );

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &["ecosystem", "verify", "--receipt", "ecosystem.json"],
    );
    assert_eq!(
        code, EXIT_OK,
        "[ecosystem verify] out={stdout} err={stderr}"
    );
}

/// [ecosystem verify] Negative: a byte-level tamper of a sealed federation
/// receipt must REJECT at exit 2 — quorum is re-derived, not trusted.
#[test]
fn ecosystem_verify_rejects_a_tampered_receipt() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, _, stderr) = seal_standing(dir.path(), &source, "member.json");
    assert_eq!(code, EXIT_OK, "[fixture member seal] {stderr}");
    let member = read_json(dir.path(), "member.json");
    let federation = serde_json::json!({
        "subject": {
            "subject": "chatman/ecosystem",
            "base": "a".repeat(40),
            "tree": "b".repeat(40),
            "candidate": "c".repeat(40),
        },
        "observation_commitment": "4".repeat(64),
        "requirements": [{ "role": "RUNTIME_EXECUTION", "minimum_alive": 1 }],
        "members": [{ "role": "RUNTIME_EXECUTION", "standing_receipt": member }],
        "previous_receipt": null,
    });
    let path = write(
        dir.path(),
        "federation.json",
        &serde_json::to_string_pretty(&federation).expect("serializes"),
    );
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "ecosystem",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--out",
            "ecosystem.json",
        ],
    );
    assert_eq!(
        code, EXIT_OK,
        "[fixture ecosystem certify] out={stdout} err={stderr}"
    );

    let body = fs::read_to_string(dir.path().join("ecosystem.json")).expect("read");
    let tampered = body.replace("ALIVE", "ZOMBIE");
    assert_ne!(body, tampered, "tamper must actually change the bytes");
    fs::write(dir.path().join("tampered.json"), tampered).expect("write tampered");

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &["ecosystem", "verify", "--receipt", "tampered.json"],
    );
    assert_eq!(
        code, REJECT,
        "[ecosystem verify tampered] out={stdout} err={stderr}"
    );
}

/// [ecosystem certify] Negative: an unmet quorum (requirement demands 2 ALIVE,
/// only 1 member) must not certify ALIVE standing.
#[test]
fn ecosystem_certify_with_unmet_quorum_does_not_reach_alive() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, _, stderr) = seal_standing(dir.path(), &source, "member.json");
    assert_eq!(code, EXIT_OK, "[fixture member seal] {stderr}");
    let member = read_json(dir.path(), "member.json");
    let federation = serde_json::json!({
        "subject": {
            "subject": "chatman/ecosystem",
            "base": "a".repeat(40),
            "tree": "b".repeat(40),
            "candidate": "c".repeat(40),
        },
        "observation_commitment": "4".repeat(64),
        "requirements": [{ "role": "RUNTIME_EXECUTION", "minimum_alive": 2 }],
        "members": [{ "role": "RUNTIME_EXECUTION", "standing_receipt": member }],
        "previous_receipt": null,
    });
    let path = write(
        dir.path(),
        "federation.json",
        &serde_json::to_string_pretty(&federation).expect("serializes"),
    );
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "ecosystem",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--out",
            "ecosystem.json",
        ],
    );
    assert_eq!(
        code, EXIT_OK,
        "[ecosystem certify] out={stdout} err={stderr}"
    );
    let sealed = read_json(dir.path(), "ecosystem.json");
    assert_ne!(
        sealed["standing"], "ALIVE",
        "value: 1 member against a quorum of 2 must not reach ALIVE"
    );
}

// ---------------------------------------------------------------------------
// errc
// ---------------------------------------------------------------------------

/// [errc certify + errc verify] Positive: a lawful ELIMINATE claim seals an
/// errc/v1 receipt with the right quadrant count; verify re-runs the law.
#[test]
fn errc_certify_then_verify_value() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, stdout, stderr) = seal_errc(dir.path(), &source);
    assert_eq!(code, EXIT_OK, "[errc certify] out={stdout} err={stderr}");

    let sealed = read_json(dir.path(), "errc.json");
    assert_eq!(sealed["profile"], "affidavit/errc/v1");
    assert_eq!(
        sealed["quadrant_counts"]["eliminate"], 1,
        "value: the single ELIMINATE claim is counted"
    );
    assert_eq!(
        sealed["quadrant_counts"]["reduce"], 0,
        "value: unclaimed quadrants count zero"
    );

    let (code, stdout, stderr) = run_in(dir.path(), &["errc", "verify", "--receipt", "errc.json"]);
    assert_eq!(code, EXIT_OK, "[errc verify] out={stdout} err={stderr}");
}

/// [errc certify] Negative: an ELIMINATE claim whose measure has a non-zero
/// candidate contradicts the directional law and must be refused.
#[test]
fn errc_certify_refuses_a_self_contradicting_claim() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let obs = errc_observation_json();
    // Mutate the measure through the value, not the string: ELIMINATE with a
    // non-zero candidate contradicts the directional law and must be refused.
    let mut val: serde_json::Value = serde_json::from_str(&obs).expect("observation parses");
    val["claims"][0]["measure"]["candidate"] = serde_json::json!(2);
    let obs = serde_json::to_string_pretty(&val).expect("reserializes");
    let obs_path = write(dir.path(), "errc-bad-observation.json", &obs);
    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "errc",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &obs_path,
            "--out",
            "errc-bad.json",
        ],
    );
    assert_eq!(
        code, REJECT,
        "[errc certify contradictory] out={stdout} err={stderr}"
    );
    assert!(
        !dir.path().join("errc-bad.json").exists(),
        "value: a refused claim must not seal an artifact"
    );
}

/// [errc assure + errc verify-assurance] Positive: a witness ledger bound
/// one-to-one to the parent's claims seals an assurance ledger and re-verifies
/// against its exact parent.
#[test]
fn errc_assure_then_verify_assurance_value() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, _, stderr) = seal_errc(dir.path(), &source);
    assert_eq!(code, EXIT_OK, "[fixture errc seal] {stderr}");
    let parent = write(
        dir.path(),
        "parent.json",
        &fs::read_to_string(dir.path().join("errc.json")).expect("read parent"),
    );
    let witnesses = write(dir.path(), "witnesses.json", &witness_ledger());

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "errc",
            "assure",
            "--parent",
            &parent,
            "--witnesses",
            &witnesses,
            "--out",
            "assurance.json",
        ],
    );
    assert_eq!(code, EXIT_OK, "[errc assure] out={stdout} err={stderr}");

    let sealed = read_json(dir.path(), "assurance.json");
    assert_eq!(sealed["profile"], "affidavit/errc-claim-assurance/v1");
    assert_eq!(
        sealed["claim_ids"][0], "eliminate-unreachable-kernel",
        "value: the assurance ledger binds the parent's claim id"
    );

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "errc",
            "verify-assurance",
            "--receipt",
            "assurance.json",
            "--parent",
            &parent,
        ],
    );
    assert_eq!(
        code, EXIT_OK,
        "[errc verify-assurance] out={stdout} err={stderr}"
    );
}

/// [errc assure] Negative: an empty witness ledger cannot satisfy the
/// one-witness-per-claim bijection and must be refused.
#[test]
fn errc_assure_refuses_a_ledger_missing_a_witness() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(dir.path());
    let (code, _, stderr) = seal_errc(dir.path(), &source);
    assert_eq!(code, EXIT_OK, "[fixture errc seal] {stderr}");
    let parent = "errc.json".to_string();
    let witnesses = write(dir.path(), "witnesses.json", "[]");

    let (code, stdout, stderr) = run_in(
        dir.path(),
        &[
            "errc",
            "assure",
            "--parent",
            &parent,
            "--witnesses",
            &witnesses,
            "--out",
            "nope.json",
        ],
    );
    assert_eq!(
        code, REJECT,
        "[errc assure empty ledger] out={stdout} err={stderr}"
    );
    assert!(
        stderr.contains("claim assurance refused"),
        "value: refusal must name the claim-assurance law; err={stderr}"
    );
    assert!(
        !dir.path().join("nope.json").exists(),
        "value: a refused assurance must not seal an artifact"
    );
}
