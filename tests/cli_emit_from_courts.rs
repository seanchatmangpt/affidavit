//! Court: the six `affi receipt emit-from-*` verbs.
//!
//! CONFIRMED DEFECT (CLI audit batch 2-3): github/gitlab/cicd/monitoring/cloud/
//! security handlers synthesized a payload JSON *string* and passed it where
//! `crate::cli::emit` expects a payload file *path* — every invocation failed
//! for any input. Repair: all six route through the `emit_with_payload` temp-file
//! staging pattern (as `emit-from-sbom` already did).
//!
//! Law under test:
//! 1. Each verb, valid minimal flags → exit 0, appends a real event to the
//!    working chain (`.affi/working.json` in the process cwd).
//! 2. Inconsistent flags (empty identity field) → typed refusal (exit 224),
//!    structured error body, never a silent success.
//!
//! Vacuity guard: every test asserts stdout non-empty — a court that cannot
//! fail is not a court.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Unique per-test scratch dir (cwd of the spawned binary).
fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "affi-emit-from-court-{tag}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&d).expect("create scratch dir");
    d
}

fn affi() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_affi"));
    cmd.env_remove("RUST_LOG");
    cmd
}

/// Run one verb in `dir`; return (exit_code, stdout, stderr).
fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = affi()
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn affi binary");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn working_chain(dir: &Path) -> Vec<serde_json::Value> {
    let raw = fs::read_to_string(dir.join(".affi/working.json"))
        .expect("working chain must exist after a successful emit");
    serde_json::from_str(&raw).expect("working chain is a JSON array of events")
}

/// The six verbs with their valid minimal fixtures and their expected
/// event_type prefix in the working chain.
const VERBS: &[(&str, &[&str], &str)] = &[
    (
        "emit-from-github",
        &["--repo", "org/repo", "--event-type", "push"],
        "github.push",
    ),
    (
        "emit-from-gitlab",
        &["--repo", "g/l", "--event-type", "pipeline"],
        "gitlab.pipeline",
    ),
    (
        "emit-from-cicd",
        &["--provider", "github-actions", "--job-status", "success"],
        "cicd.github-actions.success",
    ),
    (
        "emit-from-monitoring",
        &["--provider", "datadog", "--alert-type", "latency"],
        "monitoring.datadog.latency",
    ),
    (
        "emit-from-cloud",
        &["--provider", "aws", "--resource-type", "s3-bucket"],
        "cloud.aws.s3-bucket",
    ),
    (
        "emit-from-security",
        &["--provider", "trivy", "--vuln-type", "cve"],
        "security.trivy.cve",
    ),
];

/// Positive control: the staging helper itself must be reachable through the
/// proven-good verb (emit-from-sbom) — if this fails, the court is vacuous
/// because it would be measuring a broken substrate, not the repair.
#[test]
fn positive_control_sbom_emit_still_appends_events() {
    let dir = scratch("sbom-control");
    let sbom = dir.join("sbom.json");
    fs::write(
        &sbom,
        r#"{"spdxVersion":"SPDX-2.3","name":"court","SPDXID":"SPDXRef-DOCUMENT","packages":[{"name":"zlib","versionInfo":"1.2.13","SPDXID":"SPDXRef-Package-zlib"}]}"#,
    )
    .expect("write sbom fixture");
    let (code, stdout, _stderr) = run(
        &dir,
        &[
            "receipt",
            "emit-from-sbom",
            "--sbom-path",
            sbom.to_str().expect("utf8 path"),
        ],
    );
    assert_eq!(
        code, 0,
        "emit-from-sbom positive control failed; stdout={stdout}"
    );
    assert!(!stdout.trim().is_empty(), "vacuity guard: empty stdout");
    let events = working_chain(&dir);
    assert!(
        events.iter().any(|e| e["event_type"]
            .as_str()
            .expect("event_type is a string")
            .starts_with("sbom")),
        "positive control must append sbom events; got {events:?}"
    );
}

/// For each of the six repaired verbs: valid flags → exit 0 + chain growth +
/// the expected typed event in the chain + non-empty human receipt on stdout.
#[test]
fn each_verb_succeeds_and_appends_to_working_chain() {
    for (verb, flags, expected_type) in VERBS {
        let dir = scratch(verb.trim_start_matches("emit-from-"));
        let args: Vec<&str> = vec!["receipt", verb]
            .into_iter()
            .chain(flags.iter().copied())
            .collect();

        let (code, stdout, stderr) = run(&dir, &args);
        assert_eq!(
            code, 0,
            "{verb}: expected exit 0, got {code}; stderr={stderr}"
        );
        assert!(
            !stdout.trim().is_empty(),
            "{verb}: vacuity guard — empty stdout on success path"
        );
        assert!(
            stdout.contains("emitted"),
            "{verb}: stdout must carry the emission receipt; got {stdout:?}"
        );

        let events = working_chain(&dir);
        assert_eq!(
            events.len(),
            1,
            "{verb}: exactly one event appended to the fresh working chain"
        );
        assert_eq!(
            events[0]["event_type"].as_str().unwrap_or(""),
            *expected_type,
            "{verb}: chain carries the expected event_type"
        );
        assert!(
            events[0]["payload_commitment"].as_str().is_some(),
            "{verb}: event carries a payload commitment"
        );

        // Chain growth: a second invocation must append, not overwrite.
        let (code2, _, _) = run(&dir, &args);
        assert_eq!(code2, 0, "{verb}: second emission must also exit 0");
        assert_eq!(
            working_chain(&dir).len(),
            2,
            "{verb}: working chain must grow from 1 to 2 events"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// Typed-failure law: an empty identity field makes the derived object ref
/// (`":repo"`, `"ci::job"`, …) malformed — the verb must refuse with the typed
/// execution exit (224) and a structured error body, and must NOT append to the
/// working chain.
#[test]
fn corrupt_flags_exit_typed_and_never_append() {
    for (verb, flags, _) in VERBS {
        let dir = scratch(&format!("corrupt-{verb}"));
        // Corrupt the first identity flag to the empty string.
        let mut args: Vec<String> = vec!["receipt".into(), (*verb).into()];
        args.extend(flags.iter().map(|s| (*s).to_string()));
        // Corrupt the first identity *value* (index > 1: noun and verb are
        // structural, never corrupt targets).
        let mut corrupted = false;
        for (i, w) in args.iter_mut().enumerate() {
            if i <= 1 || w.starts_with("--") || corrupted {
                continue;
            }
            *w = String::new();
            corrupted = true;
        }
        assert!(corrupted, "{verb}: fixture must have a corruptible value");
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let (code, stdout, stderr) = run(&dir, &arg_refs);

        assert_ne!(
            code, 0,
            "{verb}: corrupt flags must not exit 0; stdout={stdout}"
        );
        assert_eq!(
            code, 224,
            "{verb}: corrupt flags must exit typed 224; stderr={stderr}"
        );
        let body = format!("{stdout}{stderr}");
        assert!(
            body.contains("\"code\"") || body.contains("not in 'id:type'"),
            "{verb}: typed structured error body expected; got {body:?}"
        );
        // No partial append: the chain must not exist (refusal happens before
        // the working receipt is written).
        assert!(
            !dir.join(".affi/working.json").exists(),
            "{verb}: refused emit must not create a working chain"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// Chain integrity end-to-end: after emissions, the assembled receipt must
/// pass the 7-stage verifier — proves the appended events are not just present
/// but structurally sound.
#[test]
fn assembled_receipt_from_emit_from_events_verifies() {
    let dir = scratch("verify");
    for (verb, flags, _) in VERBS {
        let args: Vec<&str> = vec!["receipt", verb]
            .into_iter()
            .chain(flags.iter().copied())
            .collect();
        let (code, _, stderr) = run(&dir, &args);
        assert_eq!(code, 0, "{verb}: emission failed; stderr={stderr}");
    }
    let (code, stdout, stderr) = run(&dir, &["receipt", "assemble"]);
    assert_eq!(code, 0, "assemble failed; stderr={stderr}");
    assert!(!stdout.trim().is_empty(), "vacuity guard: empty stdout");

    // Verify the assembled receipt file with the 7-stage verifier.
    let receipt_path = stdout
        .lines()
        .find_map(|l| l.strip_prefix("assembled receipt -> "))
        .expect("assemble stdout names the receipt path")
        .trim()
        .to_string();
    let (vcode, vout, verr) = run(&dir, &["receipt", "verify", &receipt_path]);
    assert_eq!(
        vcode, 0,
        "verifier rejected assembled receipt: {vout}{verr}"
    );
    let _ = fs::remove_dir_all(&dir);
}
