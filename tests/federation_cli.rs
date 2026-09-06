// End-to-end tests for the v26.9.6 federation courts.
//
// These drive the REAL `affi` binary, not the library. Before v26.9.6 the
// evidence federation kernel (standing / ecosystem / errc / claim-assurance)
// was library-only: `certify_standing` and friends existed and were unit
// tested, but nothing in `src/verbs/`, `src/handlers.rs`, or `src/registry.rs`
// referenced them, so an operator could not reach a single one of them.
//
// Every test below fails if that wiring is removed — the AGENTS.md admission
// criterion: removing the integration must break a test that exercises the
// real capability. `cargo test --lib` cannot substitute for these, because the
// library tests pass whether or not the CLI surface exists.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Stable exit codes from `src/diag.rs`. Duplicated here deliberately: an
/// end-to-end test asserts the *contract*, not the constant.
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

/// Emit one event and assemble it into `source.json`, the admitted subject
/// every federation court certifies over.
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

/// Certify a sealed standing receipt at `out` and return its path.
fn seal_standing(dir: &TempDir, source: &str, out: &str) -> String {
    let observation = write(dir, "standing-observation.json", ALIVE_OBSERVATION);
    affi(dir)
        .args([
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
        ])
        .assert()
        .success();
    out.to_string()
}

// ---------------------------------------------------------------------------
// standing
// ---------------------------------------------------------------------------

#[test]
fn standing_certify_then_verify_completes_through_the_cli() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let sealed = seal_standing(&dir, &source, "standing.json");

    let receipt = read_json(&dir, &sealed);
    assert_eq!(receipt["profile"], "affidavit/standing/v2");
    assert_eq!(receipt["standing"], "ALIVE");
    assert_eq!(
        receipt["authority"]["capability"], "affidavit.certify-standing",
        "the CLI must record its own bounded capability, never inherit the subject's"
    );
    assert_eq!(
        receipt["authority"]["scope"], "repo:seanchatmangpt/affidavit",
        "the operator-supplied scope must be carried into the sealed receipt"
    );

    // The sealed artifact survives its own law on the way back in.
    affi(&dir)
        .args(["standing", "verify", "--receipt", &sealed])
        .assert()
        .success()
        .stderr(predicate::str::contains("ACCEPT"));
}

#[test]
fn a_tampered_standing_receipt_is_rejected_with_exit_code_two() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let sealed = seal_standing(&dir, &source, "standing.json");

    // Flip one field the receipt hash covers. Nothing about the JSON is
    // malformed — only the law fails.
    let mut receipt = read_json(&dir, &sealed);
    receipt["subject"]["candidate"] = serde_json::json!("d".repeat(40));
    let tampered = write(
        &dir,
        "tampered.json",
        &serde_json::to_string_pretty(&receipt).expect("serializes"),
    );

    affi(&dir)
        .args(["standing", "verify", "--receipt", &tampered])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("REJECT"))
        .stderr(predicate::str::contains("standing_receipt_hash_mismatch"));
}

#[test]
fn alive_without_replay_evidence_is_refused_by_name() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    let mut observation: serde_json::Value =
        serde_json::from_str(ALIVE_OBSERVATION).expect("observation parses");
    observation["replay"] = serde_json::Value::Null;
    let path = write(
        &dir,
        "no-replay.json",
        &serde_json::to_string_pretty(&observation).expect("serializes"),
    );

    // The asymmetric ALIVE law is the kernel's, not the CLI's. The CLI must
    // surface the refusal by its exact name rather than softening it.
    affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--scope",
            "repo:affidavit",
            "--out",
            "should-not-exist.json",
        ])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("alive_missing_replay"));

    assert!(
        !dir.path().join("should-not-exist.json").exists(),
        "a refused certification must not write an artifact"
    );
}

#[test]
fn an_unbounded_authority_scope_is_refused_before_anything_is_sealed() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(&dir, "standing-observation.json", ALIVE_OBSERVATION);

    affi(&dir)
        .args([
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
        ])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("authority"));

    assert!(!dir.path().join("unbounded.json").exists());
}

// ---------------------------------------------------------------------------
// ecosystem
// ---------------------------------------------------------------------------

#[test]
fn ecosystem_certify_federates_a_sealed_member_and_reports_quorum() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let member = read_json(&dir, &seal_standing(&dir, &source, "member.json"));

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
        &dir,
        "federation.json",
        &serde_json::to_string_pretty(&federation).expect("serializes"),
    );

    affi(&dir)
        .args([
            "ecosystem",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--out",
            "ecosystem.json",
        ])
        .assert()
        .success();

    let sealed = read_json(&dir, "ecosystem.json");
    assert_eq!(sealed["profile"], "affidavit/ecosystem/v1");
    assert_eq!(sealed["standing"], "ALIVE");
    assert_eq!(
        sealed["coverage"][0]["satisfied"], true,
        "the declared RUNTIME_EXECUTION quorum is met by the single ALIVE member"
    );

    affi(&dir)
        .args(["ecosystem", "verify", "--receipt", "ecosystem.json"])
        .assert()
        .success();
}

#[test]
fn a_federation_whose_quorum_is_unmet_does_not_reach_alive() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let member = read_json(&dir, &seal_standing(&dir, &source, "member.json"));

    // The member witnesses RUNTIME_EXECUTION; the federation demands an ALIVE
    // FORMAL_PROOF member as well, and there is none.
    let federation = serde_json::json!({
        "subject": {
            "subject": "chatman/ecosystem",
            "base": "a".repeat(40),
            "tree": "b".repeat(40),
            "candidate": "c".repeat(40),
        },
        "observation_commitment": "4".repeat(64),
        "requirements": [
            { "role": "RUNTIME_EXECUTION", "minimum_alive": 1 },
            { "role": "FORMAL_PROOF", "minimum_alive": 1 },
        ],
        "members": [{ "role": "RUNTIME_EXECUTION", "standing_receipt": member }],
        "previous_receipt": null,
    });
    let path = write(
        &dir,
        "federation.json",
        &serde_json::to_string_pretty(&federation).expect("serializes"),
    );

    affi(&dir)
        .args([
            "ecosystem",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--out",
            "ecosystem.json",
        ])
        .assert()
        .success();

    let sealed = read_json(&dir, "ecosystem.json");
    assert_ne!(
        sealed["standing"], "ALIVE",
        "an unmet role quorum must not be crowned ALIVE"
    );
}

// ---------------------------------------------------------------------------
// errc and claim assurance
// ---------------------------------------------------------------------------

fn errc_observation() -> serde_json::Value {
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
            "command": "cargo test --test federation_cli",
            "environment_commitment": "8".repeat(64),
        },
        "previous_receipt": null,
    })
}

fn seal_errc(dir: &TempDir, source: &str) -> String {
    let path = write(
        dir,
        "errc-observation.json",
        &serde_json::to_string_pretty(&errc_observation()).expect("serializes"),
    );
    affi(dir)
        .args([
            "errc",
            "certify",
            "--receipt",
            source,
            "--observation",
            &path,
            "--out",
            "errc.json",
        ])
        .assert()
        .success();
    "errc.json".to_string()
}

#[test]
fn errc_certify_seals_a_directional_claim_and_counts_its_quadrant() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let sealed = read_json(&dir, &seal_errc(&dir, &source));

    assert_eq!(sealed["profile"], "affidavit/errc/v1");
    assert_eq!(
        sealed["source"]["commit"], "60d38265b8d1d94c43f04ca6bdb8537184e510a8",
        "the ERRC lineage is fixed to the archaeological source commit"
    );
    assert_eq!(sealed["quadrant_counts"]["eliminate"], 1);
    assert_eq!(sealed["quadrant_counts"]["reduce"], 0);

    affi(&dir)
        .args(["errc", "verify", "--receipt", "errc.json"])
        .assert()
        .success();
}

#[test]
fn an_errc_claim_that_contradicts_its_quadrant_is_refused() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    // ELIMINATE requires candidate == 0. This claim declares ELIMINATE with a
    // non-zero candidate, so the directional law must refuse it.
    let mut observation = errc_observation();
    observation["claims"][0]["measure"]["candidate"] = serde_json::json!(3);
    let path = write(
        &dir,
        "bad-errc.json",
        &serde_json::to_string_pretty(&observation).expect("serializes"),
    );

    affi(&dir)
        .args([
            "errc",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &path,
            "--out",
            "nope.json",
        ])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("errc refused"));

    assert!(!dir.path().join("nope.json").exists());
}

#[test]
fn errc_assurance_binds_one_witness_per_claim_and_verifies_against_its_parent() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let parent = seal_errc(&dir, &source);

    let witnesses = serde_json::json!([{
        "claim_id": "eliminate-unreachable-kernel",
        "verifier": "cargo test --test federation_cli",
        "evidence_locator": "tests/federation_cli.rs",
        "observed_result": "8 federation verbs reachable from the affi binary",
        "evidence_commitment": "9".repeat(64),
        "exclusions": ["Does not establish operational utility."],
    }]);
    let path = write(
        &dir,
        "witnesses.json",
        &serde_json::to_string_pretty(&witnesses).expect("serializes"),
    );

    affi(&dir)
        .args([
            "errc",
            "assure",
            "--parent",
            &parent,
            "--witnesses",
            &path,
            "--out",
            "assurance.json",
        ])
        .assert()
        .success();

    let sealed = read_json(&dir, "assurance.json");
    assert_eq!(sealed["profile"], "affidavit/errc-claim-assurance/v1");
    assert_eq!(sealed["claim_ids"][0], "eliminate-unreachable-kernel");

    affi(&dir)
        .args([
            "errc",
            "verify-assurance",
            "--receipt",
            "assurance.json",
            "--parent",
            &parent,
        ])
        .assert()
        .success();
}

#[test]
fn an_assurance_ledger_missing_a_witness_is_refused() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let parent = seal_errc(&dir, &source);

    // The parent has one claim; an empty ledger cannot satisfy the bijection.
    let path = write(&dir, "witnesses.json", "[]");

    affi(&dir)
        .args([
            "errc",
            "assure",
            "--parent",
            &parent,
            "--witnesses",
            &path,
            "--out",
            "nope.json",
        ])
        .assert()
        .code(REJECT)
        .stderr(predicate::str::contains("claim assurance refused"));

    assert!(!dir.path().join("nope.json").exists());
}

// ---------------------------------------------------------------------------
// the shared exit-code contract
// ---------------------------------------------------------------------------

#[test]
fn a_source_receipt_that_is_not_admissible_is_refused_not_certified() {
    let dir = TempDir::new().expect("tempdir");
    let observation = write(&dir, "standing-observation.json", ALIVE_OBSERVATION);

    // A receipt whose stored chain hash does not match its events. It never
    // reaches the certifier: `admit_source` decodes and admits first.
    //
    // The narrower witness — a CHAIN-CONSISTENT but objectless receipt that only
    // the wasm4pm-compat OCEL court can refuse — lives in
    // `src/federation.rs::tests::a_source_receipt_that_fails_admission_is_refused_not_certified`,
    // because `Receipt::sealed` is `pub(crate)` and an integration test cannot
    // construct one. That is the seal working as designed.
    let unadmissible = write(
        &dir,
        "unadmissible.json",
        r#"{"format_version":"core/v1","events":[{"id":"evt-0","seq":0,"event_type":"build","objects":[],"payload_commitment":"9a8cd0e0e0b1e6a4e6cd0f18d8b78cd1f8d6df6a5f2a1e8b1c2d3e4f5a6b7c8d"}],"chain_hash":"0000000000000000000000000000000000000000000000000000000000000000","profile":"core/v1"}"#,
    );

    affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &unadmissible,
            "--observation",
            &observation,
            "--scope",
            "repo:affidavit",
            "--out",
            "nope.json",
        ])
        .assert()
        .code(REJECT);

    assert!(!dir.path().join("nope.json").exists());
}

#[test]
fn a_malformed_observation_is_a_usage_error_and_a_missing_path_is_an_io_error() {
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);

    // An observation is an operator input, not a witness: refusing it is a
    // usage error, never a verdict.
    let malformed = write(&dir, "malformed.json", "{ not json }");
    affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &malformed,
            "--scope",
            "repo:affidavit",
        ])
        .assert()
        .code(USAGE_ERROR);

    let observation = write(&dir, "standing-observation.json", ALIVE_OBSERVATION);
    affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            "no-such-receipt.json",
            "--observation",
            &observation,
            "--scope",
            "repo:affidavit",
        ])
        .assert()
        .code(IO_ERROR);
}

#[test]
fn every_federation_verb_is_reachable_from_the_binary() {
    // The regression witness for the whole release: if any verb loses its
    // projection, registry entry, or handler, its help text disappears and this
    // test fails. Before v26.9.6 all eight of these were unreachable.
    let dir = TempDir::new().expect("tempdir");
    for (noun, verb) in [
        ("standing", "certify"),
        ("standing", "verify"),
        ("ecosystem", "certify"),
        ("ecosystem", "verify"),
        ("errc", "certify"),
        ("errc", "verify"),
        ("errc", "assure"),
        ("errc", "verify-assurance"),
    ] {
        affi(&dir)
            .args([noun, verb, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("affi {noun} {verb}")));
    }
}

#[test]
fn the_registry_advertises_the_federation_courts_to_guide_search() {
    // `affi guide search` reads src/registry.rs. A verb that is implemented but
    // unregistered is undiscoverable, which is how the kernel stayed invisible.
    let dir = TempDir::new().expect("tempdir");
    affi(&dir)
        .args(["guide", "search", "--keyword", "federation"])
        .assert()
        .success()
        .stdout(predicate::str::contains("certify"));
}

#[test]
fn json_format_output_is_a_single_parseable_document_on_every_path() {
    // The `clap-noun-verb` runtime renders each verb's return value to stdout
    // after the handler returns, appending a bare `null` to otherwise valid
    // JSON. The federation courts exit with their own code before that happens,
    // so `--format json` is directly machine-consumable on ACCEPT and REJECT
    // alike. Without that, `affi errc certify --format json | jq` would fail on
    // success and work on failure.
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let observation = write(&dir, "standing-observation.json", ALIVE_OBSERVATION);

    let accepted = affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &observation,
            "--scope",
            "repo:affidavit",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let accepted: serde_json::Value =
        serde_json::from_slice(&accepted).expect("ACCEPT --format json must be one JSON document");
    assert_eq!(accepted["accepted"], true);
    assert_eq!(accepted["court"], "standing/certify");

    let mut refused_observation: serde_json::Value =
        serde_json::from_str(ALIVE_OBSERVATION).expect("observation parses");
    refused_observation["replay"] = serde_json::Value::Null;
    let refused_path = write(
        &dir,
        "no-replay.json",
        &serde_json::to_string_pretty(&refused_observation).expect("serializes"),
    );

    let refused = affi(&dir)
        .args([
            "standing",
            "certify",
            "--receipt",
            &source,
            "--observation",
            &refused_path,
            "--scope",
            "repo:affidavit",
            "--format",
            "json",
        ])
        .assert()
        .code(REJECT)
        .get_output()
        .stdout
        .clone();
    let refused: serde_json::Value =
        serde_json::from_slice(&refused).expect("REJECT --format json must be one JSON document");
    assert_eq!(refused["accepted"], false);
    assert_eq!(refused["receipt"], serde_json::Value::Null);
    assert!(refused["reason"]
        .as_str()
        .expect("reason is a string")
        .contains("alive_missing_replay"));
}

#[test]
fn a_sealed_receipt_written_with_out_is_byte_exact_json() {
    // `--out` exists because the clap-noun-verb runtime appends its own
    // rendering of each verb's return value to stdout; a shell redirect of the
    // certify output is therefore not parseable. This asserts the artifact path
    // stays clean so certify -> verify composes in a real script.
    let dir = TempDir::new().expect("tempdir");
    let source = admitted_source(&dir);
    let sealed = seal_standing(&dir, &source, "standing.json");

    let raw = fs::read_to_string(Path::new(dir.path()).join(&sealed)).expect("artifact readable");
    serde_json::from_str::<serde_json::Value>(&raw)
        .expect("the --out artifact must be parseable JSON with no trailing tokens");
}
