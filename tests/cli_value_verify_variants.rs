//! Value court: `receipt verify-compliance`, `receipt verify-sla`,
//! `receipt verify-family`, `receipt policy-enforce`,
//! `receipt license-compliance`.
//!
//! Law under test: the exit code is verdict-determined and format-independent
//! — ACCEPT/PASS/COMPLIANT exits 0, REJECT/FAIL/VIOLATIONS exits 2 — in BOTH
//! `--format human` (default) and `--format json`. Negative cells are typed
//! degradations: a malformed policy fails closed, an unenforceable SLA key
//! refuses as UNSUPPORTED_ENFORCEMENT, an unknown compliance framework
//! degrades to an honest generic check (never a silent invented pass).
//!
//! Chicago discipline: real built binary, real emitted+assembled chains, no
//! mocks; every assertion is on actual process exit codes, parsed JSON, or
//! output text derived from the fixture.
//!
// value-matrix: (receipt, verify-compliance)
// value-matrix: (receipt, verify-sla)
// value-matrix: (receipt, verify-family)
// value-matrix: (receipt, policy-enforce)
// value-matrix: (receipt, license-compliance)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_affi")
}

/// Serialize the suite: the cells drive the real binary against
/// machine-level state (cwd-resolved receipt stores) and flaked when raced
/// in parallel; serial order is the court's contract (see emit-build flake
/// receipt 2026-10-17).
static SUITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
thread_local! {
    static SUITE_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}
fn lock_suite() -> SuiteGuard {
    // Reentrant per-thread: fixture helpers nest, a plain std Mutex would
    // deadlock on the second acquire.
    if SUITE_DEPTH.with(|d| d.get()) == 0 {
        let g = SUITE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        SUITE_DEPTH.with(|d| d.set(1));
        return SuiteGuard(Some(g));
    }
    SuiteGuard(None)
}
struct SuiteGuard(Option<std::sync::MutexGuard<'static, ()>>);
impl Drop for SuiteGuard {
    fn drop(&mut self) {
        if self.0.is_some() {
            SUITE_DEPTH.with(|d| d.set(0));
        }
    }
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "affi-verify-variants-{}-{}",
        tag,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Emit `events` via the real binary and assemble them into
/// `<dir>/receipts/<name>`. Returns the receipt path.
fn assemble_chain(dir: &Path, name: &str, events: &[(&str, &str)]) -> PathBuf {
    fs::create_dir_all(dir.join("receipts")).expect("create receipts dir");
    let payload = dir.join("payload.bin");
    fs::write(&payload, b"payload").expect("write payload");
    for (event_type, object) in events {
        let out = Command::new(bin())
            .args([
                "receipt",
                "emit",
                "--event-type",
                event_type,
                "--object",
                object,
                "--payload",
                payload.to_str().unwrap(),
            ])
            .current_dir(dir)
            .output()
            .expect("spawn affi emit");
        assert!(
            out.status.success(),
            "emit {event_type} failed (exit {:?}): {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let out_path = dir.join("receipts").join(name);
    let out = Command::new(bin())
        .args(["receipt", "assemble", "--out", out_path.to_str().unwrap()])
        .current_dir(dir)
        .output()
        .expect("spawn affi assemble");
    assert!(
        out.status.success(),
        "assemble {name} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out_path.exists(), "assemble must produce {name}");
    out_path
}

/// Tamper an assembled receipt on disk (flip an event type) so the 7-stage
/// verifier has a real REJECT to report.
fn tamper(src: &Path, dst: &Path) {
    let mut v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(src).expect("read receipt"))
            .expect("parse receipt");
    v["events"][0]["event_type"] = "TAMPERED".into();
    fs::write(dst, v.to_string()).expect("write tampered receipt");
}

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
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

fn parse_json(stdout: &str) -> serde_json::Value {
    serde_json::from_str(stdout).unwrap_or_else(|e| panic!("stdout must be JSON ({e}): {stdout}"))
}

fn write_json(dir: &Path, name: &str, v: &serde_json::Value) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, v.to_string()).expect("write json fixture");
    p
}

// ===========================================================================
// verify-compliance — SOC2 on a real chain, both formats, exit follows verdict
// ===========================================================================

#[test]
fn verify_compliance_soc2_accept_human() {
    let _g = lock_suite();
    let dir = scratch("vc-human");
    let receipt = assemble_chain(
        &dir,
        "chain.json",
        &[("build", "repo:main"), ("test", "suite:unit")],
    );

    let (code, stdout, _stderr) = run(
        &dir,
        &[
            "receipt",
            "verify-compliance",
            receipt.to_str().unwrap(),
            "--framework",
            "soc2",
        ],
    );
    assert_eq!(code, 0, "accept verdict must exit 0; stdout: {stdout}");
    assert!(stdout.contains("soc2"), "names the framework: {stdout}");
    assert!(
        stdout.contains("EVIDENCE_PRESENT"),
        "accept verdict must print EVIDENCE_PRESENT: {stdout}"
    );
    assert!(
        stdout.contains("note: legal compliance determination requires human auditor review"),
        "certify-don't-decide note required: {stdout}"
    );
    assert!(
        stdout.contains("evidence present for control"),
        "per-control evidence lines required: {stdout}"
    );
}

#[test]
fn verify_compliance_soc2_accept_json() {
    let _g = lock_suite();
    let dir = scratch("vc-json");
    let receipt = assemble_chain(
        &dir,
        "chain.json",
        &[("build", "repo:main"), ("test", "suite:unit")],
    );

    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "verify-compliance",
            receipt.to_str().unwrap(),
            "--framework",
            "soc2",
            "--format",
            "json",
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(v["framework"], "soc2");
    assert_eq!(v["compliant"], true, "valid chain must be compliant: {v}");
    let checks = v["checks"].as_array().expect("checks array");
    assert_eq!(checks.len(), 2, "SOC2 exposes two controls: {v}");
    assert_eq!(checks[0]["check"], "access-control");
    assert_eq!(checks[0]["passed"], true);
    assert_eq!(checks[1]["check"], "availability");
    assert_eq!(checks[1]["passed"], true);
}

#[test]
fn verify_compliance_reject_follows_verdict_in_both_formats() {
    let _g = lock_suite();
    // Tampered receipt (broken chain hash) => verdict REJECT => exit 2 in
    // both human and json, proving the exit is verdict-determined and
    // format-independent.
    for format in ["human", "json"] {
        let dir = scratch(&format!("vc-reject-{format}"));
        let good = assemble_chain(&dir, "good.json", &[("build", "repo:main")]);
        let bad = dir.join("bad.json");
        tamper(&good, &bad);

        let mut args = vec![
            "receipt",
            "verify-compliance",
            bad.to_str().unwrap(),
            "--framework",
            "soc2",
        ];
        if format == "json" {
            args.extend(["--format", "json"]);
        }
        let (code, stdout, stderr) = run(&dir, &args);
        assert_eq!(
            code, 2,
            "tampered receipt must exit REJECT (2) in {format} format; stderr: {stderr}"
        );
        if format == "json" {
            let v = parse_json(&stdout);
            assert_eq!(v["compliant"], false, "{v}");
            assert_eq!(v["checks"][0]["passed"], false, "{v}");
        } else {
            assert!(
                stdout.contains("EVIDENCE_ABSENT"),
                "human reject must print EVIDENCE_ABSENT: {stdout}"
            );
        }
    }
}

#[test]
fn verify_compliance_unknown_framework_degrades_honestly() {
    let _g = lock_suite();
    let dir = scratch("vc-unknown");
    let receipt = assemble_chain(&dir, "chain.json", &[("build", "repo:main")]);

    // Unknown framework: never a silently invented framework verdict — it
    // degrades to the generic check and says so, on a valid chain.
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "verify-compliance",
            receipt.to_str().unwrap(),
            "--framework",
            "soc2-typo",
            "--format",
            "json",
        ],
    );
    assert_eq!(code, 0, "valid chain still exits 0; stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(v["framework"], "soc2-typo", "echoes the unknown name: {v}");
    let checks = v["checks"].as_array().expect("checks array");
    assert_eq!(checks.len(), 1, "degrades to one generic control: {v}");
    assert_eq!(checks[0]["check"], "generic-check", "{v}");
    assert_eq!(checks[0]["passed"], true, "{v}");

    // Human format names the honest-degradation note.
    let (code, stdout, _) = run(
        &dir,
        &[
            "receipt",
            "verify-compliance",
            receipt.to_str().unwrap(),
            "--framework",
            "soc2-typo",
        ],
    );
    assert_eq!(code, 0);
    assert!(
        stdout.contains("generic-check") || stdout.contains("basic chain verification"),
        "human output must name the generic degradation: {stdout}"
    );
    assert!(
        stdout.contains("note: legal compliance determination requires human auditor review"),
        "degradation stays honest about certification vs decision: {stdout}"
    );
}

// ===========================================================================
// verify-sla — min_events flip (0 -> PASS, 5 -> FAIL exit 2) + typed refusal
// ===========================================================================

fn sla_file(dir: &Path, name: &str, min_events: u64, ttl_ms: Option<u64>) -> PathBuf {
    let mut v = serde_json::json!({ "min_events": min_events });
    if let Some(ms) = ttl_ms {
        v["max_chain_ttl_ms"] = serde_json::json!(ms);
    }
    write_json(dir, name, &v)
}

#[test]
fn verify_sla_min_events_zero_passes_in_both_formats() {
    let _g = lock_suite();
    for format in ["human", "json"] {
        let dir = scratch(&format!("sla-pass-{format}"));
        let receipt = assemble_chain(&dir, "chain.json", &[("build", "repo:main")]);
        let sla = sla_file(&dir, "sla.json", 0, None);
        let mut args = vec![
            "receipt",
            "verify-sla",
            receipt.to_str().unwrap(),
            "--sla-file",
            sla.to_str().unwrap(),
        ];
        if format == "json" {
            args.extend(["--format", "json"]);
        }
        let (code, stdout, stderr) = run(&dir, &args);
        assert_eq!(code, 0, "min_events=0 must PASS; stderr: {stderr}");
        if format == "json" {
            let v = parse_json(&stdout);
            assert_eq!(v["sla_met"], true, "{v}");
            assert_eq!(v["event_count"], 1, "{v}");
            assert_eq!(v["min_events_required"], 0, "{v}");
        } else {
            assert!(
                stdout.contains("PASS"),
                "human PASS marker required: {stdout}"
            );
            assert!(stdout.contains("events=1"), "{stdout}");
        }
    }
}

#[test]
fn verify_sla_min_events_five_fails_exit2_in_both_formats() {
    let _g = lock_suite();
    // The chain has 2 events; a min of 5 must FAIL and exit 2 in every format.
    for format in ["human", "json"] {
        let dir = scratch(&format!("sla-fail-{format}"));
        let receipt = assemble_chain(
            &dir,
            "chain.json",
            &[("build", "repo:main"), ("test", "suite:unit")],
        );
        let sla = sla_file(&dir, "sla.json", 5, None);
        let mut args = vec![
            "receipt",
            "verify-sla",
            receipt.to_str().unwrap(),
            "--sla-file",
            sla.to_str().unwrap(),
        ];
        if format == "json" {
            args.extend(["--format", "json"]);
        }
        let (code, stdout, stderr) = run(&dir, &args);
        assert_eq!(
            code, 2,
            "min_events=5 over a 2-event chain must exit REJECT (2) in {format}; stderr: {stderr}"
        );
        if format == "json" {
            let v = parse_json(&stdout);
            assert_eq!(v["sla_met"], false, "{v}");
            assert_eq!(v["min_events_required"], 5, "{v}");
        } else {
            assert!(
                stdout.contains("FAIL"),
                "human FAIL marker required: {stdout}"
            );
        }
    }
}

#[test]
fn verify_sla_ttl_key_is_typed_unsupported_enforcement() {
    let _g = lock_suite();
    let dir = scratch("sla-ttl");
    let receipt = assemble_chain(&dir, "chain.json", &[("build", "repo:main")]);
    let sla = sla_file(&dir, "sla.json", 0, Some(60_000));

    let (code, _stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "verify-sla",
            receipt.to_str().unwrap(),
            "--sla-file",
            sla.to_str().unwrap(),
        ],
    );
    assert_ne!(
        code, 0,
        "an unenforceable TTL key must fail closed, never pass silently"
    );
    assert_ne!(
        code, 2,
        "this is a typed refusal about the SLA file, not a receipt REJECT verdict"
    );
    assert!(
        stderr.contains("UNSUPPORTED_ENFORCEMENT"),
        "typed marker required: {stderr}"
    );
    assert!(
        stderr.contains("max_chain_ttl_ms"),
        "names the unenforceable key: {stderr}"
    );
}

// ===========================================================================
// verify-family — real directory scan with per-file results
// ===========================================================================

#[test]
fn verify_family_clean_dir_all_accepted_exit0() {
    let _g = lock_suite();
    let dir = scratch("fam-clean");
    assemble_chain(
        &dir,
        "a.json",
        &[("build", "repo:main"), ("test", "suite:unit")],
    );
    assemble_chain(&dir, "b.json", &[("build", "repo:lib")]);

    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "verify-family",
            "--receipts-dir",
            "receipts",
            "--format",
            "json",
        ],
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let v = parse_json(&stdout);
    assert_eq!(v["total"], 2, "{v}");
    assert_eq!(v["accepted"], 2, "{v}");
    assert_eq!(v["rejected"], 0, "{v}");
    for r in v["results"].as_array().expect("results array") {
        assert_eq!(r["accepted"], true, "every receipt accepted: {v}");
    }

    // Human format carries the per-file lines and exits 0 too.
    let (code, stdout, _) = run(
        &dir,
        &["receipt", "verify-family", "--receipts-dir", "receipts"],
    );
    assert_eq!(code, 0);
    assert!(
        stdout.contains("2/2 receipts accepted"),
        "human summary required: {stdout}"
    );
    assert_eq!(stdout.matches("[ACCEPT]").count(), 2, "{stdout}");
}

#[test]
fn verify_family_tampered_receipt_flips_to_reject_exit2() {
    let _g = lock_suite();
    let dir = scratch("fam-reject");
    assemble_chain(
        &dir,
        "good.json",
        &[("build", "repo:main"), ("test", "suite:unit")],
    );
    let bad = dir.join("receipts").join("tampered.json");
    tamper(&dir.join("receipts").join("good.json"), &bad);

    let (code, stdout, _) = run(
        &dir,
        &[
            "receipt",
            "verify-family",
            "--receipts-dir",
            "receipts",
            "--format",
            "json",
        ],
    );
    assert_eq!(code, 2, "any reject must exit 2; stdout: {stdout}");
    let v = parse_json(&stdout);
    assert_eq!(v["total"], 2, "{v}");
    assert_eq!(v["rejected"], 1, "{v}");
    let rejected: Vec<&serde_json::Value> = v["results"]
        .as_array()
        .expect("results")
        .iter()
        .filter(|r| r["accepted"] == false)
        .collect();
    assert_eq!(rejected.len(), 1);
    assert_eq!(
        rejected[0]["path"].as_str().unwrap().contains("tampered"),
        true,
        "per-file result names the offending file: {v}"
    );
    assert!(
        rejected[0]["reject_reason"]
            .as_str()
            .unwrap_or("")
            .contains("parse error"),
        "reject carries a reason: {v}"
    );

    // Human format: REJECT line names the file, exit stays 2.
    let (code, stdout, _) = run(
        &dir,
        &["receipt", "verify-family", "--receipts-dir", "receipts"],
    );
    assert_eq!(code, 2);
    assert!(
        stdout.contains("tampered"),
        "human output names the offending file: {stdout}"
    );
    assert!(
        stdout.contains("[REJECT]"),
        "human per-file REJECT marker: {stdout}"
    );
}

// ===========================================================================
// policy-enforce — min_approvals flip
// ===========================================================================

#[test]
fn policy_enforce_min_approvals_flips_verdict_human() {
    let _g = lock_suite();
    // Chain holds exactly one approval event: min_approvals=1 passes,
    // min_approvals=2 violates (human format).
    for min_approvals in [1u64, 2u64] {
        let dir = scratch(&format!("pe-human-{min_approvals}"));
        assemble_chain(
            &dir,
            "chain.json",
            &[("build", "repo:main"), ("approve", "repo:main")],
        );
        let policy = write_json(
            &dir,
            "policy.json",
            &serde_json::json!({ "min_approvals": min_approvals }),
        );
        let (code, stdout, stderr) = run(
            &dir,
            &[
                "receipt",
                "policy-enforce",
                "--receipts-path",
                "receipts",
                "--policy-file",
                policy.to_str().unwrap(),
            ],
        );
        let expected = if min_approvals == 1 { 0 } else { 2 };
        assert_eq!(
            code, expected,
            "min_approvals={min_approvals} must exit {expected}; stderr: {stderr}"
        );
        if min_approvals == 2 {
            assert!(
                stdout.contains("VIOLATIONS FOUND"),
                "human verdict marker required: {stdout}"
            );
        }
    }
}

/// DEFECT COURT (ignored until the owning lane fixes src/handlers.rs):
/// `policy-enforce --format json` exits 0 even when the policy is violated —
/// the json branch prints the report and returns Ok(()) without the
/// exit_codes::REJECT (2) transition the human path performs (witnessed
/// 2026-10-17: min_approvals=2 over a 1-approval chain exited 0 in json while
/// the identical run in human format exited 2). Un-ignore when the json exit
/// is verdict-determined like every other consumer on this surface.
#[test]
// DEFECT CLOSED (verdict-determined exits in both formats — policy_enforce restructured).
fn policy_enforce_json_exit_is_verdict_determined() {
    let _g = lock_suite();
    let dir = scratch("pe-json-defect");
    assemble_chain(
        &dir,
        "chain.json",
        &[("build", "repo:main"), ("approve", "repo:main")],
    );
    let policy = write_json(
        &dir,
        "policy.json",
        &serde_json::json!({ "min_approvals": 2 }),
    );
    let (code, stdout, _) = run(
        &dir,
        &[
            "receipt",
            "policy-enforce",
            "--receipts-path",
            "receipts",
            "--policy-file",
            policy.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 2,
        "violations in json format must still exit 2: {stdout}"
    );
    let v = parse_json(&stdout);
    assert_eq!(v["compliant"], false, "{v}");
    assert_eq!(v["violations"], 1, "{v}");
}

#[test]
fn policy_enforce_missing_security_scan_flag_flips_human() {
    let _g = lock_suite();
    let dir = scratch("pe-scan");
    assemble_chain(&dir, "chain.json", &[("approve", "repo:main")]);
    let strict = write_json(
        &dir,
        "strict.json",
        &serde_json::json!({ "min_approvals": 0, "require_security_scan": true }),
    );
    let (code, stdout, _) = run(
        &dir,
        &[
            "receipt",
            "policy-enforce",
            "--receipts-path",
            "receipts",
            "--policy-file",
            strict.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 2, "no security-scan event in the chain: {stdout}");
    assert!(
        stdout.contains("VIOLATIONS FOUND"),
        "human verdict marker: {stdout}"
    );
}

// ===========================================================================
// license-compliance — allowed-list flip with the offender named
// ===========================================================================

#[test]
fn license_compliance_allowed_list_flip_names_offender() {
    let _g = lock_suite();
    for allowed in [vec!["MIT"], vec!["Apache-2.0"]] {
        let tag = allowed.join("_");
        for format in ["human", "json"] {
            let dir = scratch(&format!("lic-{}-{}", tag.replace('.', ""), format));
            assemble_chain(&dir, "chain.json", &[("license-scan", "MIT:license")]);
            let policy = write_json(
                &dir,
                "policy.json",
                &serde_json::json!({ "allowed_licenses": allowed }),
            );
            let mut args = vec![
                "receipt",
                "license-compliance",
                "--receipts-path",
                "receipts",
                "--license-policy",
                policy.to_str().unwrap(),
            ];
            if format == "json" {
                args.extend(["--format", "json"]);
            }
            let (code, stdout, stderr) = run(&dir, &args);
            if allowed[0] == "MIT" {
                assert_eq!(
                    code, 0,
                    "MIT allowed must exit 0 ({format}); stderr: {stderr}"
                );
            } else {
                assert_eq!(
                    code, 2,
                    "MIT not on the list must exit 2 ({format}); stderr: {stderr}"
                );
            }
            if format == "json" {
                let v = parse_json(&stdout);
                assert_eq!(v["compliant"], allowed[0] == "MIT", "{v}");
                if allowed[0] != "MIT" {
                    assert_eq!(
                        v["violations"][0]["license"], "MIT",
                        "offender must be named: {v}"
                    );
                    assert_eq!(
                        v["violations"][0]["violation"],
                        "license-not-in-allowed-list"
                    );
                }
            } else if allowed[0] != "MIT" {
                assert!(
                    stdout.contains("\"MIT\"") && stdout.contains("not in the allowed list"),
                    "human output names the offender: {stdout}"
                );
            }
        }
    }
}

// ===========================================================================
// Negatives: malformed policy fails closed (typed, non-zero) on every consumer
// ===========================================================================

#[test]
fn malformed_policy_fails_closed_on_all_three_consumers() {
    let _g = lock_suite();
    let dir = scratch("policy-malformed");
    assemble_chain(&dir, "chain.json", &[("build", "repo:main")]);
    let bad_policy = dir.join("bad.json");
    fs::write(&bad_policy, "{ not valid json !!").expect("write malformed policy");

    // verify-sla
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "verify-sla",
            "receipts/chain.json",
            "--sla-file",
            bad_policy.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code, 1504 % 256,
        "verify-sla must fail closed on malformed SLA with the typed 1504 diagnostic: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("\"code\":1504"),
        "structured diagnostic required: {stdout} {stderr}"
    );

    // policy-enforce
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "policy-enforce",
            "--receipts-path",
            "receipts",
            "--policy-file",
            bad_policy.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code, 1504 % 256,
        "policy-enforce must fail closed on malformed policy with the typed 1504 diagnostic: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("\"code\":1504"),
        "structured diagnostic required: {stdout} {stderr}"
    );

    // license-compliance
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "license-compliance",
            "--receipts-path",
            "receipts",
            "--license-policy",
            bad_policy.to_str().unwrap(),
        ],
    );
    assert_eq!(
        code, 1504 % 256,
        "license-compliance must fail closed on malformed policy with the typed 1504 diagnostic: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("\"code\":1504"),
        "structured diagnostic required: {stdout} {stderr}"
    );
}

#[test]
fn missing_policy_file_fails_closed_named() {
    let _g = lock_suite();
    let dir = scratch("policy-missing");
    assemble_chain(&dir, "chain.json", &[("build", "repo:main")]);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "policy-enforce",
            "--receipts-path",
            "receipts",
            "--policy-file",
            dir.join("nope.json").to_str().unwrap(),
        ],
    );
    assert_eq!(
        code,
        1504 % 256,
        "missing policy must fail closed with the typed 1504 diagnostic: {stdout} {stderr}"
    );
    assert!(
        format!("{stdout}{stderr}").contains("No such file or directory"),
        "IO diagnostic required: {stdout} {stderr}"
    );
}
