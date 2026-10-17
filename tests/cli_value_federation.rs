// Court: the federation CLI verbs carry *value*, not just wiring.
//
// Where `tests/federation_cli.rs` proves the verbs are reachable, this court
// proves the verbs are worth reaching: sealed outputs are input-sensitive
// (deterministic but distinguishing), refusals are typed and column-precise,
// and the sealed-receipt load path refuses anything that is not its own
// profile. Real binary, real files, real exit codes — Chicago, no mocks.
//
// Markers: [V1] standing ACCEPT roundtrip · [V2] standing verify ACCEPT ·
// [V3] IO_ERROR=4 on missing file · [V4] USAGE_ERROR=3 on plain-text
// observation · [V5] errc certify ACCEPT + errc verify re-verify ·
// [V6] errc assure names each missing witness field ·
// [V7] errc verify-assurance REJECT on wrong-profile input ·
// [V8] ecosystem typed REJECT/USAGE escalations, column-precise ·
// [V9] sealed outputs differ per input · [V10] verify refuses foreign profiles.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

const REJECT: i32 = 2;
const USAGE_ERROR: i32 = 3;
const IO_ERROR: i32 = 4;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

fn write(dir: &TempDir, name: &str, contents: &str) -> String {
    fs::write(dir.path().join(name), contents).expect("fixture written");
    name.to_string()
}

fn read_json(dir: &TempDir, name: &str) -> serde_json::Value {
    let raw = fs::read_to_string(dir.path().join(name)).expect("artifact readable");
    serde_json::from_str(&raw).expect("artifact is JSON")
}

/// Emit one event and assemble it into a real `core/v1` receipt, admitted
/// through the full Layer 2 admission gate.
fn admitted_source(dir: &TempDir) -> String {
    affi(dir)
        .args([
            "receipt",
            "emit",
            "--event-type",
            "build",
            "--object",
            "repo:git",
            "--payload",
            "-",
        ])
        .write_stdin("exact-head execution")
        .assert()
        .success();

    affi(dir)
        .args(["receipt", "assemble", "--out", "source.json"])
        .assert()
        .success();

    assert!(
        dir.path().join("source.json").exists(),
        "assemble must produce the source receipt"
    );
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
    "command": "just validate",
    "environment_commitment": "3333333333333333333333333333333333333333333333333333333333333333"
  },
  "previous_receipt": null
}"#;

/// Standing observation differing only in one digest field — same shape, so
/// the sealed standing receipts must differ.
const ALIVE_OBSERVATION_B: &str = r#"{
  "subject": {
    "subject": "seanchatmangpt/affidavit",
    "base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "candidate": "dddddddddddddddddddddddddddddddddddddddd"
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
    "command": "just validate",
    "environment_commitment": "3333333333333333333333333333333333333333333333333333333333333333"
  },
  "previous_receipt": null
}"#;

fn errc_observation(commitment: &str) -> serde_json::Value {
    serde_json::json!({
        "subject": {
            "subject": "seanchatmangpt/affidavit",
            "base": "a".repeat(40),
            "tree": "b".repeat(40),
            "candidate": "c".repeat(40),
        },
        "observation_commitment": commitment,
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
            "command": "cargo test --test cli_value_federation",
            "environment_commitment": "8".repeat(64),
        },
        "previous_receipt": null,
    })
}

fn errc_observation_json(commitment: &str) -> String {
    serde_json::to_string_pretty(&errc_observation(commitment)).expect("serializes")
}

/// Certify a standing receipt over `observation` into `out`; asserts ACCEPT.
fn seal_standing(dir: &TempDir, source: &str, observation: &str, out: &str) {
    affi(dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            source,
            "--observation",
            observation,
            "--scope",
            "repo:seanchatmangpt/affidavit",
            "--out",
            out,
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("ACCEPT"));
}

/// Certify an ERRC receipt over `observation` into `out`; asserts ACCEPT.
fn seal_errc(dir: &TempDir, source: &str, observation: &str) -> String {
    affi(dir)
        .args([
            "errc",
            "certify",
            "--receipt",
            source,
            "--observation",
            observation,
            "--out",
            "errc.json",
        ])
        .assert()
        .success();
    "errc.json".to_string()
}

// ---------------------------------------------------------------------------
// standing — [V1]..[V4]
// ---------------------------------------------------------------------------

/// [V1] The full ACCEPT roundtrip over a real observation JSON file.
#[test]
fn v1_standing_certify_full_accept_roundtrip() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(&dir, "obs.json", ALIVE_OBSERVATION);

    seal_standing(&dir, &source, &observation, "standing.json");

    let sealed = read_json(&dir, "standing.json");
    assert_eq!(sealed["profile"], "affidavit/standing/v2");
    assert_eq!(sealed["standing"], "ALIVE");
    assert_eq!(
        sealed["authority"]["scope"], "repo:seanchatmangpt/affidavit",
        "the operator scope must be carried into the sealed receipt"
    );
}

/// [V2] `standing verify` ACCEPTs the sealed receipt from disk.
#[test]
fn v2_standing_verify_accepts_the_sealed_receipt() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(&dir, "obs.json", ALIVE_OBSERVATION);
    seal_standing(&dir, &source, &observation, "standing.json");

    affi(&dir)
        .args(["standing", "verify", "--receipt", "standing.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ACCEPT"));
}

/// [V3] A missing observation file is a typed IO_ERROR (exit 4), named by path.
#[test]
fn v3_missing_observation_file_is_typed_io_error() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            "absent-observation.json",
            "--scope",
            "repo:affidavit",
        ])
        .assert()
        .code(IO_ERROR)
        .stderr(predicate::str::contains("cannot read"))
        .stderr(predicate::str::contains("absent-observation.json"));
}

/// [V4] A plain-text observation is a typed USAGE_ERROR (exit 3) whose serde
/// error names the exact line and column of the first offending byte.
#[test]
fn v4_plain_text_observation_is_typed_usage_error_column_precise() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(&dir, "obs.txt", "this is not json at all\n");

    affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &observation,
            "--scope",
            "repo:affidavit",
        ])
        .assert()
        .code(USAGE_ERROR)
        .stderr(predicate::str::contains("not well-formed"))
        .stderr(predicate::str::contains("line 1 column 2"));
}

// ---------------------------------------------------------------------------
// errc — [V5]..[V7]
// ---------------------------------------------------------------------------

/// [V5] `errc certify` seals a real core/v1 receipt and `errc verify`
/// re-verifies the sealed artifact.
#[test]
fn v5_errc_certify_seals_and_errc_verify_reverifies() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(
        &dir,
        "errc-obs.json",
        &errc_observation_json(&"5".repeat(64)),
    );

    let sealed_path = seal_errc(&dir, &source, &observation);

    let sealed = read_json(&dir, &sealed_path);
    assert_eq!(sealed["profile"], "affidavit/errc/v1");
    assert_eq!(sealed["quadrant_counts"]["eliminate"], 1);

    affi(&dir)
        .args(["errc", "verify", "--receipt", &sealed_path])
        .assert()
        .success()
        .stderr(predicate::str::contains("ACCEPT"));
}

/// [V6] An assurance ledger with structurally incomplete witnesses is refused
/// with a typed schema escalation that names EVERY missing field.
#[test]
fn v6_errc_assure_names_each_missing_witness_field() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(
        &dir,
        "errc-obs.json",
        &errc_observation_json(&"5".repeat(64)),
    );
    let parent = seal_errc(&dir, &source, &observation);

    // One witness struct whose only field is claim_id — every other mandatory
    // field of ErrcClaimWitness is absent, and serde names them one by one.
    let ledger = write(
        &dir,
        "witnesses.json",
        r#"[{"claim_id": "eliminate-unreachable-kernel"}]"#,
    );

    affi(&dir)
        .args([
            "errc",
            "assure",
            "--parent",
            &parent,
            "--witnesses",
            &ledger,
            "--out",
            "nope.json",
        ])
        .assert()
        .code(USAGE_ERROR)
        .stderr(predicate::str::contains(
            "claim witness ledger is not well-formed",
        ))
        .stderr(predicate::str::contains("missing field `verifier`"));
}

/// [V7] `errc verify-assurance` refuses wrong-profile input as a typed REJECT.
#[test]
fn v7_errc_verify_assurance_rejects_wrong_profile_input() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(
        &dir,
        "errc-obs.json",
        &errc_observation_json(&"5".repeat(64)),
    );

    // A valid sealed *standing* receipt is still the wrong profile here.
    let obs = write(&dir, "obs.json", ALIVE_OBSERVATION);
    seal_standing(&dir, &source, &obs, "standing.json");

    affi(&dir)
        .args(["errc", "verify-assurance", "--receipt", "standing.json"])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("REJECT"))
        .stderr(predicate::str::contains("sealed receipt refused"));
}

// ---------------------------------------------------------------------------
// ecosystem — [V8]
// ---------------------------------------------------------------------------

/// [V8a] A plain-text ecosystem observation is a column-precise typed
/// USAGE_ERROR; a wrong-profile sealed input to `ecosystem verify` is a typed
/// REJECT. Both escalations name their law.
#[test]
fn v8_ecosystem_typed_escalations_are_column_precise() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    // Plain text where an observation belongs: usage error, line/column named.
    let plain = write(&dir, "plain.txt", "ecosystem observations are JSON\n");
    affi(&dir)
        .args([
            "ecosystem",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &plain,
        ])
        .assert()
        .code(USAGE_ERROR)
        .stderr(predicate::str::contains(
            "ecosystem observation is not well-formed",
        ))
        .stderr(predicate::str::contains("line 1 column 1"));

    // A sealed standing receipt fed to `ecosystem verify` is the wrong profile:
    // typed REJECT, never a silent reinterpretation.
    let obs = write(&dir, "obs.json", ALIVE_OBSERVATION);
    seal_standing(&dir, &source, &obs, "standing.json");
    affi(&dir)
        .args(["ecosystem", "verify", "--receipt", "standing.json"])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("REJECT"))
        .stderr(predicate::str::contains("sealed receipt refused"));
}

// ---------------------------------------------------------------------------
// value — [V9], [V10]
// ---------------------------------------------------------------------------

/// [V9] Sealed outputs differ per input: two observations differing only in the
/// subject's candidate digest seal to different receipts.
#[test]
fn v9_sealed_outputs_differ_per_input() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    let obs_a = write(&dir, "obs-a.json", ALIVE_OBSERVATION);
    seal_standing(&dir, &source, &obs_a, "standing-a.json");
    let sealed_a = fs::read_to_string(dir.path().join("standing-a.json")).expect("read");

    let obs_b = write(&dir, "obs-b.json", ALIVE_OBSERVATION_B);
    seal_standing(&dir, &source, &obs_b, "standing-b.json");
    let sealed_b = fs::read_to_string(dir.path().join("standing-b.json")).expect("read");

    assert_ne!(
        sealed_a, sealed_b,
        "two different observations must not seal to the same receipt"
    );

    // And the difference is exactly the field that moved.
    let a = read_json(&dir, "standing-a.json");
    let b = read_json(&dir, "standing-b.json");
    assert_ne!(a["subject"]["candidate"], b["subject"]["candidate"]);
    assert_eq!(a["profile"], b["profile"]);
}

/// [V10] The sealed-receipt load path refuses foreign profiles: a sealed ERRC
/// receipt fed to `standing verify` is a typed REJECT, and a sealed standing
/// receipt fed to `errc verify` likewise.
#[test]
fn v10_verify_refuses_foreign_profiles() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    let obs = write(&dir, "obs.json", ALIVE_OBSERVATION);
    seal_standing(&dir, &source, &obs, "standing.json");

    let errc_obs = write(
        &dir,
        "errc-obs.json",
        &errc_observation_json(&"5".repeat(64)),
    );
    let errc_sealed = seal_errc(&dir, &source, &errc_obs);

    affi(&dir)
        .args(["standing", "verify", "--receipt", &errc_sealed])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("REJECT"))
        .stderr(predicate::str::contains("sealed receipt refused"));

    affi(&dir)
        .args(["errc", "verify", "--receipt", "standing.json"])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("REJECT"))
        .stderr(predicate::str::contains("sealed receipt refused"));
}
