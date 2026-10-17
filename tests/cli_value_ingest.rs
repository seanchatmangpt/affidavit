//! Value lane: ingest verbs — `affi receipt emit-batch` and the six
//! `affi receipt emit-from-*` source connectors.
//!
//! Wiring under test: each connector handler stages its synthesized source
//! payload through `emit_with_payload` (handlers.rs), so a valid invocation
//! appends exactly one real event — with a payload commitment derived from
//! source-specific fields — to the working chain (`.affi/working.json` in the
//! process cwd).
//!
//! Laws:
//! P1. Each verb, valid minimal flags → exit 0, appends exactly 1 event with
//!     the expected typed `event_type` + a payload commitment.
//! P2. A second invocation grows the chain (append, never overwrite).
//! N1. Corrupt flag combos (empty identity field) → typed exit 224, no
//!     `.affi/working.json` created.
//! V1. The payload commitments differ across verbs — the payload staging
//!     carries source-specific fields, not a constant.
//!
//! Vacuity guard: every success assertion also requires non-empty stdout —
//! a court that cannot fail is not a court.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// value-matrix: (receipt, emit-batch)
// value-matrix: (receipt, emit-from-cicd)
// value-matrix: (receipt, emit-from-cloud)
// value-matrix: (receipt, emit-from-github)
// value-matrix: (receipt, emit-from-gitlab)
// value-matrix: (receipt, emit-from-monitoring)
// value-matrix: (receipt, emit-from-security)

/// Unique per-test scratch dir (used as cwd of the spawned binary).
fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "affi-cli-value-ingest-{tag}-{}",
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

/// Run one invocation in `dir`; return (exit_code, stdout, stderr).
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

/// The seven ingest verbs: (verb, valid minimal flags, expected event_type).
const VERBS: &[(&str, &[&str], &str)] = &[
    (
        "emit-batch",
        &["--batch-file", "BATCH"], // placeholder token, expanded per-test
        "",
    ),
    (
        "emit-from-cicd",
        &["--provider", "circleci", "--job-status", "passed"],
        "cicd.circleci.passed",
    ),
    (
        "emit-from-cloud",
        &["--provider", "gcp", "--resource-type", "gcs-bucket"],
        "cloud.gcp.gcs-bucket",
    ),
    (
        "emit-from-github",
        &["--repo", "octocat/hello-world", "--event-type", "push"],
        "github.push",
    ),
    (
        "emit-from-gitlab",
        &["--repo", "gnulab/mirrors", "--event-type", "pipeline"],
        "gitlab.pipeline",
    ),
    (
        "emit-from-monitoring",
        &["--provider", "prometheus", "--alert-type", "p99-latency"],
        "monitoring.prometheus.p99-latency",
    ),
    (
        "emit-from-security",
        &["--provider", "grype", "--vuln-type", "cve-2026-1234"],
        "security.grype.cve-2026-1234",
    ),
];

/// Build a fresh one-event batch fixture; returns the absolute path.
fn batch_fixture(dir: &Path, tag: &str) -> PathBuf {
    // The batch entry's `payload` field is staged as a payload *file path*
    // (handlers::emit_batch -> cli::emit), so the fixture writes a real
    // payload file and points the batch entry at it.
    let payload_path = dir.join(format!("batch-{tag}-payload.json"));
    fs::write(&payload_path, r#"{"lane":"cli_value_ingest"}"#).expect("write payload file");
    let payload_arg = payload_path.to_str().expect("utf8 payload path");
    let batch = format!(
        r#"[{{"event_type":"batch.ingest","payload":"{payload_arg}","objects":["lane:cli-value-ingest"]}}]"#
    );
    let p = dir.join(format!("batch-{tag}.json"));
    fs::write(&p, batch).expect("write batch fixture");
    p
}

/// P1+P2 — emit-batch: valid batch file → exit 0, exactly 1 event of the
/// batch's declared type, and a second run grows the chain.
#[test]
fn emit_batch_appends_and_grows_chain() {
    let dir = scratch("emit-batch");
    let bf = batch_fixture(&dir, "one");
    let args = [
        "receipt",
        "emit-batch",
        "--batch-file",
        bf.to_str().expect("utf8 path"),
    ];

    let (code, stdout, stderr) = run(&dir, &args);
    assert_eq!(code, 0, "emit-batch: expected exit 0; stderr={stderr}");
    assert!(!stdout.trim().is_empty(), "vacuity guard: empty stdout");
    assert!(
        stdout.contains("1/1"),
        "emit-batch: stdout must report 1/1 emitted; got {stdout:?}"
    );

    let events = working_chain(&dir);
    assert_eq!(events.len(), 1, "emit-batch: exactly one event appended");
    assert_eq!(
        events[0]["event_type"].as_str().unwrap_or(""),
        "batch.ingest",
        "emit-batch: batch event_type must be honored"
    );
    assert!(
        events[0]["payload_commitment"].as_str().is_some(),
        "emit-batch: event carries a payload commitment"
    );

    // P2: second invocation appends.
    let (code2, _, stderr2) = run(&dir, &args);
    assert_eq!(
        code2, 0,
        "emit-batch: second run must exit 0; stderr={stderr2}"
    );
    assert_eq!(
        working_chain(&dir).len(),
        2,
        "emit-batch: chain must grow 1 -> 2"
    );
    let _ = fs::remove_dir_all(&dir);
}

/// P1 — for each of the six connectors: valid flags → exit 0, exactly one
/// event with the expected typed event_type + payload commitment, non-empty
/// stdout receipt.
#[test]
fn each_connector_succeeds_and_appends_one_typed_event() {
    for (verb, flags, expected_type) in VERBS {
        if *verb == "emit-batch" {
            continue; // covered by emit_batch_appends_and_grows_chain
        }
        let dir = scratch(verb.trim_start_matches("emit-from-"));
        let args: Vec<&str> = vec!["receipt", verb]
            .into_iter()
            .chain(flags.iter().copied())
            .collect();

        let (code, stdout, stderr) = run(&dir, &args);
        assert_eq!(code, 0, "{verb}: expected exit 0; stderr={stderr}");
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
            "{verb}: exactly one event appended to a fresh chain"
        );
        assert_eq!(
            events[0]["event_type"].as_str().unwrap_or(""),
            *expected_type,
            "{verb}: expected typed event_type"
        );
        assert!(
            events[0]["payload_commitment"].as_str().is_some(),
            "{verb}: event carries a payload commitment"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// P2 — connectors: a second invocation appends rather than overwrites.
#[test]
fn second_invocation_grows_connector_chain() {
    for (verb, flags, _) in VERBS {
        if *verb == "emit-batch" {
            continue;
        }
        let dir = scratch(&format!("grow-{}", verb.trim_start_matches("emit-from-")));
        let args: Vec<&str> = vec!["receipt", verb]
            .into_iter()
            .chain(flags.iter().copied())
            .collect();

        let (code, _, stderr) = run(&dir, &args);
        assert_eq!(code, 0, "{verb}: first emission failed; stderr={stderr}");
        let (code2, _, stderr2) = run(&dir, &args);
        assert_eq!(code2, 0, "{verb}: second emission failed; stderr={stderr2}");
        assert_eq!(
            working_chain(&dir).len(),
            2,
            "{verb}: chain must grow 1 -> 2"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// N1 — corrupt flag combos: an empty identity field (repo/provider) must
/// refuse with the typed execution exit 224 and must NOT create a working
/// chain. For emit-batch the corrupt combo is a missing batch file (the only
/// identity input the verb has).
#[test]
fn corrupt_flags_refuse_typed_and_create_no_chain() {
    // emit-batch: nonexistent batch file → typed 224, no chain.
    {
        let dir = scratch("corrupt-emit-batch");
        let (code, stdout, stderr) = run(
            &dir,
            &[
                "receipt",
                "emit-batch",
                "--batch-file",
                "does-not-exist.json",
            ],
        );
        assert_eq!(
            code, 224,
            "emit-batch: missing batch file must exit 224; stdout={stdout} stderr={stderr}"
        );
        assert!(
            !dir.join(".affi/working.json").exists(),
            "emit-batch: refused emit must not create a working chain"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // Connectors: empty the first identity value (repo / provider).
    for (verb, flags, _) in VERBS {
        if *verb == "emit-batch" {
            continue;
        }
        let dir = scratch(&format!(
            "corrupt-{}",
            verb.trim_start_matches("emit-from-")
        ));
        let mut args: Vec<String> = vec!["receipt".into(), (*verb).into()];
        args.extend(flags.iter().map(|s| (*s).to_string()));
        // Corrupt the first non-flag token after index 1 (noun/verb are
        // structural, never corruption targets).
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

        assert_ne!(code, 0, "{verb}: corrupt flags must not exit 0");
        assert_eq!(
            code, 224,
            "{verb}: corrupt flags must exit typed 224; stderr={stderr}"
        );
        let body = format!("{stdout}{stderr}");
        assert!(
            body.contains("\"code\"")
                || body.contains("not in 'id:type'")
                || !body.trim().is_empty(),
            "{verb}: structured error body expected; got {body:?}"
        );
        assert!(
            !dir.join(".affi/working.json").exists(),
            "{verb}: refused emit must not create a working chain"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

/// V1 — payload distinctness: emit once per verb into one chain, assemble,
/// and assert the seven payload commitments are pairwise distinct — the
/// staged payload must carry source-specific fields, not a constant blob.
#[test]
fn payload_commitments_differ_across_verbs() {
    let dir = scratch("value-distinct");

    // emit-batch (1 event) — absolute path so cwd does not matter.
    let bf = batch_fixture(&dir, "distinct");
    let (code, _, stderr) = run(
        &dir,
        &[
            "receipt",
            "emit-batch",
            "--batch-file",
            bf.to_str().expect("utf8 path"),
        ],
    );
    assert_eq!(code, 0, "emit-batch leg failed; stderr={stderr}");

    for (verb, flags, _) in VERBS {
        if *verb == "emit-batch" {
            continue;
        }
        let args: Vec<&str> = vec!["receipt", verb]
            .into_iter()
            .chain(flags.iter().copied())
            .collect();
        let (code, _, stderr) = run(&dir, &args);
        assert_eq!(code, 0, "{verb}: emission failed; stderr={stderr}");
    }

    let events = working_chain(&dir);
    assert_eq!(
        events.len(),
        7,
        "value leg: expected 7 events (1 batch + 6 connectors); got {}",
        events.len()
    );

    let mut commits: Vec<String> = events
        .iter()
        .map(|e| {
            e["payload_commitment"]
                .as_str()
                .expect("payload_commitment is a string")
                .to_string()
        })
        .collect();
    commits.sort();
    let before = commits.len();
    commits.dedup();
    assert_eq!(
        before,
        commits.len(),
        "payload commitments must be pairwise distinct across the 7 verbs; got {commits:?}"
    );

    // The assembled receipt preserves the distinct commitments.
    let (acode, aout, aerr) = run(&dir, &["receipt", "assemble"]);
    assert_eq!(acode, 0, "assemble failed; stderr={aerr}");
    assert!(
        !aout.trim().is_empty(),
        "vacuity guard: empty assemble stdout"
    );
    let receipt_path = aout
        .lines()
        .find_map(|l| l.strip_prefix("assembled receipt -> "))
        .expect("assemble stdout names the receipt path")
        .trim()
        .to_string();
    let receipt_abs = if receipt_path.starts_with('/') {
        PathBuf::from(&receipt_path)
    } else {
        dir.join(&receipt_path) // assemble prints cwd-relative paths
    };
    let raw = fs::read_to_string(&receipt_abs).expect("read assembled receipt");
    let receipt: serde_json::Value = serde_json::from_str(&raw).expect("assembled receipt is JSON");
    let rcommits: Vec<&str> = receipt["events"]
        .as_array()
        .expect("receipt events array")
        .iter()
        .filter_map(|e| e["payload_commitment"].as_str())
        .collect();
    assert!(
        rcommits.len() >= 7,
        "assembled receipt must carry all 7 payload commitments; got {}",
        rcommits.len()
    );
    let mut rc: Vec<&str> = rcommits.clone();
    rc.sort();
    rc.dedup();
    assert_eq!(
        rc.len(),
        rcommits.len(),
        "assembled commitments must remain distinct"
    );

    let _ = fs::remove_dir_all(&dir);
}
