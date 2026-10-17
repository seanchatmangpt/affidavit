// Vacuity tripwire court — the mechanical detector for the VACUOUS class in
// docs/reference/CLI_VACUITY_AUDIT_26_10_17.md.
//
// Law: no verb may return success whose output is independent of its input.
// Each cell runs the built `affi` binary against two different real fixtures
// and asserts the outcome (exit code + derived field) actually flips.
//
// Lane ownership: the legal-hold status + license-compliance cells are ACTIVE
// (v26.10.17 eliminate lane). The receipt-test cell was REMOVED: the verb itself was
// until their owning lanes land their fixes — an un-ignored failing cell is a
// court, a faked pass is not.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_affi")
}

/// Serialize the whole suite: the cells drive the real binary against
/// machine-level state (env-resolved receipt paths, the shared cargo-linked
/// binary) and flaked when raced in parallel; serial order is the court's
/// contract (see emit-build flake receipt 2026-10-17).
static SUITE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
thread_local! {
    static SUITE_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}
fn lock_suite() -> SuiteGuard {
    // Reentrant per-thread: cells may nest fixture/scratch helpers; a plain
    // std Mutex deadlocks on the second acquire (witnessed: parallel run hung).
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
    let dir = std::env::temp_dir().join(format!("affi-vacuity-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Build a real two-event chain + a one-event license chain in `<dir>/receipts`.
fn fixture_with_chains(tag: &str, license_object: &str) -> PathBuf {
    let _g = lock_suite();
    let dir = scratch(tag);
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).expect("create receipts dir");
    let p1 = dir.join("p1.bin");
    fs::write(&p1, b"payload-one").unwrap();
    let lic = dir.join("lic.bin");
    fs::write(&lic, b"MIT").unwrap();

    for (event_type, object, payload) in [
        ("build", "repo:main", p1.to_str().unwrap()),
        ("test", "suite:unit", lic.to_str().unwrap()),
    ] {
        let out = Command::new(bin())
            .args([
                "receipt",
                "emit",
                "--event-type",
                event_type,
                "--object",
                object,
                "--payload",
                payload,
            ])
            .current_dir(&dir)
            .output()
            .expect("run affi emit");
        if !out.status.success() {
            panic!(
                "emit {event_type} failed (exit {:?}): {}",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
    let st = Command::new(bin())
        .args(["receipt", "assemble", "--out", "receipts/chain1.json"])
        .current_dir(&dir)
        .status()
        .expect("run affi assemble");
    assert!(st.success(), "assemble chain1 failed");

    let st = Command::new(bin())
        .args([
            "receipt",
            "emit",
            "--event-type",
            "license-scan",
            "--object",
            license_object,
            "--payload",
            lic.to_str().unwrap(),
        ])
        .current_dir(&dir)
        .status()
        .expect("run affi emit license");
    assert!(st.success(), "license emit failed");
    let st = Command::new(bin())
        .args(["receipt", "assemble", "--out", "receipts/chain2.json"])
        .current_dir(&dir)
        .status()
        .expect("run affi assemble");
    assert!(st.success(), "assemble chain2 failed");
    dir
}

fn write_policy(dir: &Path, name: &str, allowed: &[&str]) -> PathBuf {
    let p = dir.join(name);
    let list: Vec<String> = allowed.iter().map(|s| format!("\"{s}\"")).collect();
    fs::write(
        &p,
        format!("{{\"allowed_licenses\": [{}]}}", list.join(", ")),
    )
    .unwrap();
    p
}

// ---------------------------------------------------------------------------
// ACTIVE CELL 1: `legal-hold status` — REFUSED_NO_CHAINS on empty input,
// derived events_preserved on real chains (audit defect 5).
// ---------------------------------------------------------------------------

#[test]
fn legal_hold_status_refuses_no_chains_on_empty_dir() {
    let _g = lock_suite();
    let dir = scratch("lh-empty");
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).unwrap();

    let out = Command::new(bin())
        .args(["legal-hold", "status", "--matter", "m-1"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi legal-hold status");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(224),
        "empty dir must exit 224 (REFUSED_NO_CHAINS via CliExecutionError 1504); stderr: {stderr}"
    );
    assert!(
        stderr.contains("REFUSED_NO_CHAINS"),
        "typed refusal must be named; stderr: {stderr}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains("EVIDENTIARY_HOLD_ACTIVE"),
        "no fabricated hold over zero chains"
    );
}

#[test]
fn legal_hold_status_derives_events_preserved_from_real_chains() {
    let dir = fixture_with_chains("lh-real", "MIT:license");
    let receipts = dir.join("receipts");

    let out = Command::new(bin())
        .args(["legal-hold", "status", "--matter", "m-1"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi legal-hold status");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON stdout");
    assert_eq!(v["status"], "EVIDENTIARY_HOLD_ACTIVE");
    // events_preserved must equal the actual event count across the chains on
    // disk (summed from the fixtures, never the fabricated constant 1).
    let on_disk: usize = fs::read_dir(&receipts)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .map(|e| {
            let r: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(e.path()).unwrap()).unwrap();
            r["events"].as_array().map(|a| a.len()).unwrap_or(0)
        })
        .sum();
    assert!(
        on_disk > 1,
        "fixture must hold multiple events; got {on_disk}"
    );
    assert_eq!(
        v["events_preserved"], on_disk,
        "events_preserved must be derived from the chains"
    );
    assert_eq!(v["chains"], 2);
    assert_eq!(v["spoliation_detected"], false);

    // Mutation control: a tampered chain must flip the verdict to spoliation.
    let mut tampered: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(receipts.join("chain1.json")).unwrap()).unwrap();
    tampered["events"][0]["event_type"] = "TAMPERED".into();
    fs::write(receipts.join("chain3.json"), tampered.to_string()).unwrap();
    let out = Command::new(bin())
        .args(["legal-hold", "status", "--matter", "m-1"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi legal-hold status (tampered)");
    assert_eq!(
        out.status.code(),
        Some(2),
        "tampered chain must REJECT, not certify"
    );
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["spoliation_detected"], true);
    assert_eq!(v["ok"], false);
}

// ---------------------------------------------------------------------------
// ACTIVE CELL 2: `receipt license-compliance` — allowed vs disallowed flips
// the verdict; zero allowed licenses fails on any license event (defect 2).
// ---------------------------------------------------------------------------

#[test]
fn license_compliance_allowed_license_is_compliant() {
    let dir = fixture_with_chains("lc-allowed", "MIT:license");
    let policy = write_policy(&dir, "policy.json", &["MIT", "Apache-2.0"]);

    let out = Command::new(bin())
        .args([
            "receipt",
            "license-compliance",
            "--receipts-path",
            "receipts",
            "--license-policy",
            policy.to_str().unwrap(),
            "--format",
            "json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run affi license-compliance");

    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["compliant"], true, "MIT is allowed; full output: {v}");
    assert_eq!(v["license_events_found"], 1);
}

#[test]
fn license_compliance_disallowed_license_flips_verdict_and_names_offender() {
    let dir = fixture_with_chains("lc-disallowed", "GPL-3.0-only:license");
    let policy = write_policy(&dir, "policy.json", &["MIT", "Apache-2.0"]);

    let out = Command::new(bin())
        .args([
            "receipt",
            "license-compliance",
            "--receipts-path",
            "receipts",
            "--license-policy",
            policy.to_str().unwrap(),
            "--format",
            "json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run affi license-compliance");

    assert_eq!(
        out.status.code(),
        Some(2),
        "disallowed license must exit REJECT (2)"
    );
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["compliant"], false);
    assert_eq!(
        v["violations"][0]["license"], "GPL-3.0-only",
        "offending license must be named"
    );
    assert!(v["status"].as_str().unwrap().contains("GPL-3.0-only"));
}

#[test]
fn license_compliance_zero_allowed_licenses_fails_on_any_license_event() {
    let dir = fixture_with_chains("lc-strict", "MIT:license");
    let policy = write_policy(&dir, "policy.json", &[]);

    let out = Command::new(bin())
        .args([
            "receipt",
            "license-compliance",
            "--receipts-path",
            "receipts",
            "--license-policy",
            policy.to_str().unwrap(),
        ])
        .current_dir(&dir)
        .output()
        .expect("run affi license-compliance");

    assert_eq!(
        out.status.code(),
        Some(2),
        "zero allowed + any license event must fail"
    );
}

// ---------------------------------------------------------------------------
// IGNORED CELLS — the rest of the VACUOUS six; owned by other lanes.
// Un-ignore only when the owning lane's fix lands. Each ignored cell states
// the flip its lane must make pass.
// ---------------------------------------------------------------------------

#[test]
fn receipt_audit_output_depends_on_input() {
    // ACTIVE (defect 1 flip): the audit report must derive from the real
    // store — two different stores produce different findings, and a
    // tampered chain must flip the verdict to REJECT (exit 2).
    let dir_ok = fixture_with_chains("audit-ok", "MIT:license");
    let out_ok = Command::new(bin())
        .args(["receipt", "audit"])
        .env("AFFI_RECEIPTS_PATH", dir_ok.join("receipts"))
        .output()
        .expect("run affi receipt audit (valid chains)");

    assert_eq!(out_ok.status.code(), Some(0), "all-valid store must exit 0");
    let v_ok: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out_ok.stdout)).unwrap();
    assert_eq!(v_ok["chains_checked"], 2);
    assert_eq!(v_ok["accepted"], 2);
    assert_eq!(v_ok["rejected"], 0);

    // Different store: one chain tampered after assembly -> REJECT, exit 2,
    // and the tampered chain is named with the verifier's failure reason.
    let dir_bad = fixture_with_chains("audit-bad", "MIT:license");
    let bad = dir_bad.join("receipts/chain1.json");
    let mut doc: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&bad).unwrap()).unwrap();
    doc["events"][0]["event_type"] = serde_json::Value::String("tampered".into());
    fs::write(&bad, serde_json::to_string(&doc).unwrap()).unwrap();

    let out_bad = Command::new(bin())
        .args(["receipt", "audit"])
        .env("AFFI_RECEIPTS_PATH", dir_bad.join("receipts"))
        .output()
        .expect("run affi receipt audit (tampered chain)");

    assert_eq!(
        out_bad.status.code(),
        Some(2),
        "tampered chain must exit REJECT (2); stderr: {}",
        String::from_utf8_lossy(&out_bad.stderr)
    );
    // The tampered chain fails to load (deserialization re-verifies the chain
    // hash), so it is reported as an unloadable REJECT, not silently dropped.
    let v_bad: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out_bad.stdout)).unwrap();
    assert_eq!(v_bad["chains_checked"], 1);
    assert_eq!(v_bad["accepted"], 1);
    assert_eq!(v_bad["rejected"], 1);
    assert!(
        v_bad["failures"][0]["chain"]
            .as_str()
            .unwrap()
            .ends_with("chain1.json"),
        "the tampered chain must be named in failures"
    );
    assert!(
        v_bad["failures"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("unloadable"),
        "unloadable chain must carry the unloadable reason"
    );
}

#[test]
fn receipt_audit_refuses_no_chains_on_empty_dir() {
    let _g = lock_suite();
    let dir = scratch("audit-empty");
    fs::create_dir_all(dir.join("receipts")).unwrap();

    let out = Command::new(bin())
        .args(["receipt", "audit"])
        .env("AFFI_RECEIPTS_PATH", dir.join("receipts"))
        .output()
        .expect("run affi receipt audit (empty dir)");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(224), "stderr: {stderr}");
    assert!(
        stderr.contains("REFUSED_NO_CHAINS"),
        "typed refusal must be named; stderr: {stderr}"
    );
}

#[test]
fn certificate_fre_902_recomputes_chain() {
    // Falsifier 1: a real two-chain fixture certifies, embedding actual chain
    // hashes + event counts (stdout depends on the input).
    let dir = fixture_with_chains("fre902-good", "license:MIT");
    let receipts = dir.join("receipts");
    let out = Command::new(bin())
        .args(["certificate", "fre-902"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi certificate fre-902");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        out.status.code(),
        Some(0),
        "real chains must certify: {stdout}"
    );
    assert!(
        stdout.contains("SELF_AUTHENTICATING_FRE_902_VALID"),
        "{stdout}"
    );
    assert!(stdout.contains("\"chains_certified\": 2"), "{stdout}");
    // events_certified must equal the sum of the per-chain event counts shown.
    let per_chain: usize = stdout
        .lines()
        .filter(|l| l.contains("\"events\": "))
        .filter_map(|l| {
            l.split(": ")
                .nth(1)?
                .trim_end_matches(',')
                .trim()
                .parse::<usize>()
                .ok()
        })
        .sum();
    let claimed: usize = stdout
        .lines()
        .find(|l| l.contains("\"events_certified\": "))
        .and_then(|l| {
            l.split(": ")
                .nth(1)?
                .trim_end_matches(',')
                .trim()
                .parse::<usize>()
                .ok()
        })
        .expect("events_certified present");
    assert!(
        per_chain > 0 && claimed == per_chain,
        "actual counts embedded: {stdout}"
    );
    assert!(
        stdout.contains("chain_hash"),
        "affidavit must embed actual chain hashes: {stdout}"
    );

    // Falsifier 2: an empty receipts dir is the typed refusal, never a certificate.
    let _g = lock_suite();
    let dir = scratch("fre902-empty");
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).unwrap();
    let out = Command::new(bin())
        .args(["certificate", "fre-902"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi certificate fre-902 empty");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(224),
        "empty dir must refuse: {stderr}"
    );
    assert!(stderr.contains("REFUSED_NO_CHAINS"), "{stderr}");

    // Falsifier 3: a tampered chain (flipped payload byte -> chain-hash mismatch)
    // refuses typed, never certifies.
    let dir = fixture_with_chains("fre902-tamper", "license:MIT");
    let receipts = dir.join("receipts");
    let chain1 = receipts.join("chain1.json");
    let text = fs::read_to_string(&chain1).unwrap();
    // Flip the stored chain hash: the recomputed rolling BLAKE3 can no longer match.
    let (prefix, hash_line) = text
        .split_once("\"chain_hash\"")
        .expect("chain_hash field present");
    let mut tampered = prefix.to_string() + "\"chain_hash\": \"deadbeef";
    tampered.push_str(
        hash_line
            .trim_start_matches(char::is_alphanumeric)
            .trim_start_matches('"'),
    );
    fs::write(&chain1, tampered).unwrap();
    let out = Command::new(bin())
        .args(["certificate", "fre-902"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi certificate fre-902 tampered");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(224),
        "tampered chain must refuse: {stderr}"
    );
    assert!(stderr.contains("REFUSED_"), "{stderr}");
}

#[test]
fn restitution_calculate_derives_from_schedule() {
    // Falsifier 1: two schedules over the SAME chain produce different figures.
    let dir = fixture_with_chains("rest-sched", "license:MIT");
    let receipts = dir.join("receipts");
    let run = |sched: &str| {
        let out = Command::new(bin())
            .args(["restitution", "calculate", "--tariff-schedule", sched])
            .env("AFFI_RECEIPTS_PATH", &receipts)
            .output()
            .expect("run affi restitution calculate");
        assert_eq!(
            out.status.code(),
            Some(0),
            "{sched}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    let azure = run("azure");
    let aws = run("aws");
    let gcp = run("gcp");
    let pick = |s: &str, key: &str| -> String {
        let line = s
            .lines()
            .find(|l| l.contains(key))
            .unwrap_or_else(|| panic!("figure {key} present"));
        let after = line.split(": ").nth(1).expect("value after colon");
        after
            .trim()
            .trim_end_matches(',')
            .trim_matches('"')
            .to_string()
    };
    let a = pick(&azure, "avoided_tariff_usd");
    let w = pick(&aws, "avoided_tariff_usd");
    let g = pick(&gcp, "avoided_tariff_usd");
    assert_ne!(a, w, "azure vs aws must differ on the same chain");
    assert_ne!(a, g, "azure vs gcp must differ on the same chain");
    assert_ne!(w, g, "aws vs gcp must differ on the same pairwise chain");
    // Figures are derived: 1 build event × rate, gain share = 50%.
    assert!(
        azure.contains("\"compute_events\": 2") && azure.contains("0.12"),
        "{azure}"
    );

    // Falsifier 2: a chain with zero compute-class events refuses typed.
    let _g = lock_suite();
    let dir = scratch("rest-empty");
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).unwrap();
    let out = Command::new(bin())
        .args(["restitution", "calculate", "--tariff-schedule", "azure"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi restitution calculate empty");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(224),
        "empty dir must refuse: {stderr}"
    );
    assert!(stderr.contains("REFUSED_NO_COMPUTE_EVENTS"), "{stderr}");

    // Falsifier 3: an unknown schedule refuses typed (never a fabricated figure).
    let dir = fixture_with_chains("rest-bad-sched", "license:MIT");
    let receipts = dir.join("receipts");
    let out = Command::new(bin())
        .args(["restitution", "calculate", "--tariff-schedule", "oracle"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi restitution calculate bad schedule");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(224),
        "unknown schedule must refuse: {stderr}"
    );
    assert!(
        stderr.contains("REFUSED_UNKNOWN_TARIFF_SCHEDULE"),
        "{stderr}"
    );
}

// ---------------------------------------------------------------------------
// MERGED LANE ADDENDUM: byte-identity positive-control court (10 controls).
//
// The detector law above is enforced here from the other direction: 10
// genuinely-executing verbs are run against two deliberately-different receipt
// worlds and MUST produce different combined output. A control that exits
// non-zero or produces byte-identical output fails loudly — these are the
// anti-vacuity positive controls proving this court's detector can fire.
//
// Channel note: several verbs print the real report on stderr and a bare
// `null` on stdout (output-layer B13 trailing-null bug), so the comparison is
// stdout+stderr combined, with the world dir path normalized to "{WORLD}" so
// no difference can be manufactured trivially by path text.
// ---------------------------------------------------------------------------

use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Unique scratch tags: all tests in one binary share a pid, so tag-only
/// scratch keys would collide across concurrently-running tests.
static WORLD_SEQ: AtomicUsize = AtomicUsize::new(0);

/// One event emission (payload on stdin) into the working chain of `dir`.
fn emit_stdin(dir: &Path, ty: &str, object: &str, payload: &str) {
    let mut child = std::process::Command::new(bin())
        .args([
            "receipt",
            "emit",
            "--type",
            ty,
            "--object",
            object,
            "--payload",
            "-",
        ])
        .current_dir(dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn affi emit");
    child
        .stdin
        .take()
        .expect("emit stdin")
        .write_all(payload.as_bytes())
        .expect("write payload");
    let st = child.wait().expect("wait emit");
    assert!(st.success(), "emit {ty} failed");
}

/// Build a receipt world in `dir`; returns the assembled receipt file path.
fn build_world(dir: &Path, policy: &str, events: &[(&str, &str, &str)]) -> PathBuf {
    fs::write(dir.join("policy.json"), policy).expect("write policy fixture");
    for (ty, object, payload) in events {
        emit_stdin(dir, ty, object, payload);
    }
    let st = std::process::Command::new(bin())
        .args(["receipt", "assemble"])
        .current_dir(dir)
        .status()
        .expect("run affi assemble");
    assert!(st.success(), "assemble failed");
    let mut receipts: Vec<PathBuf> = fs::read_dir(dir)
        .expect("list world dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .filter(|p| p.file_name().and_then(|f| f.to_str()) != Some("policy.json"))
        .collect();
    receipts.sort();
    assert_eq!(
        receipts.len(),
        1,
        "expected exactly one assembled receipt in {}",
        dir.display()
    );
    receipts.remove(0)
}

/// Combined stdout+stderr with the world dir path normalized to "{WORLD}".
fn normalized_output(dir: &Path, out: &std::process::Output) -> String {
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    combined.replace(dir.to_str().expect("utf8 temp dir"), "{WORLD}")
}

/// The two deliberately-different receipt worlds (fresh per case: `receipt
/// emit` is stateful against a working chain, so cases never share a world).
struct Worlds {
    receipt_a: PathBuf,
    receipt_b: PathBuf,
    dir_a: PathBuf,
    dir_b: PathBuf,
    _keep: Vec<PathBuf>, // world dirs live under temp_dir(); removed at drop is not needed for a test
}

/// World A: 2 events. World B: 3 events, different types/objects/payloads, and
/// a different license policy — every structural input differs by construction.
fn worlds() -> Worlds {
    let n = WORLD_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir_a = scratch(&format!("world-a-{n}"));
    let dir_b = scratch(&format!("world-b-{n}"));
    let receipt_a = build_world(
        &dir_a,
        r#"{"allowed_licenses":["MIT","Apache-2.0"]}"#,
        &[
            ("init", "app:service", "alpha world"),
            ("transform", "data:artifact", "alpha transform"),
        ],
    );
    let receipt_b = build_world(
        &dir_b,
        r#"{"allowed_licenses":["GPL-3.0-only"]}"#,
        &[
            ("release", "other:thing", "beta world entirely different"),
            ("audit-log", "log:entry", "beta different payload"),
            ("deploy", "svc:api", "beta third event payload"),
        ],
    );
    Worlds {
        receipt_a,
        receipt_b,
        dir_a,
        dir_b,
        _keep: vec![],
    }
}

/// One court cell: two deliberate invocations that must differ.
struct DiffCell {
    name: &'static str,
    args_a: Vec<String>,
    args_b: Vec<String>,
}

fn receipt_cell(
    name: &'static str,
    noun_verb: &[&str],
    tail: &[&str],
    receipt_a: &Path,
    receipt_b: &Path,
) -> DiffCell {
    let mk = |r: &Path| {
        let mut args: Vec<String> = noun_verb
            .iter()
            .chain(tail.iter())
            .map(|s| s.to_string())
            .collect();
        args.push(r.to_string_lossy().into_owned());
        args
    };
    DiffCell {
        name,
        args_a: mk(receipt_a),
        args_b: mk(receipt_b),
    }
}

fn dir_cell(name: &'static str, verb: &[&str], flag: &str, dir_a: &Path, dir_b: &Path) -> DiffCell {
    let mk = |d: &Path| {
        let mut args: Vec<String> = verb.iter().map(|s| s.to_string()).collect();
        args.push(flag.to_string());
        args.push(d.to_string_lossy().into_owned());
        args
    };
    DiffCell {
        name,
        args_a: mk(dir_a),
        args_b: mk(dir_b),
    }
}

/// Court assertion: both invocations must exit 0 and outputs must differ.
fn assert_flips(cell: &DiffCell, w: &Worlds) {
    let run = |dir: &Path, args: &[String]| {
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let out = std::process::Command::new(bin())
            .args(&refs)
            .current_dir(dir)
            .output()
            .expect("spawn affi");
        (out.status.code(), normalized_output(dir, &out))
    };
    let (exit_a, out_a) = run(&w.dir_a, &cell.args_a);
    let (exit_b, out_b) = run(&w.dir_b, &cell.args_b);
    assert_eq!(
        exit_a,
        Some(0),
        "control `{}` must exit 0 in world A (got {exit_a:?}); a control that cannot run is a loud FAIL",
        cell.name
    );
    assert_eq!(
        exit_b,
        Some(0),
        "control `{}` must exit 0 in world B (got {exit_b:?})",
        cell.name
    );
    assert_ne!(
        out_a, out_b,
        "VACUOUS FLIP FAILED: `{}` produced byte-identical success output across two different inputs (output prefix: {:?})",
        cell.name,
        out_a.chars().take(120).collect::<String>()
    );
}

macro_rules! diff_control {
    ($test:ident, $cell:expr) => {
        #[test]
        fn $test() {
            let w = worlds();
            assert_flips(&$cell(&w), &w);
        }
    };
}

diff_control!(positive_show_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt show",
        &["receipt", "show"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_inspect_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt inspect",
        &["receipt", "inspect"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_stats_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt stats",
        &["receipt", "stats"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_graph_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt graph",
        &["receipt", "graph"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_replay_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt replay",
        &["receipt", "replay"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_model_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt model",
        &["receipt", "model"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_diff_differs_across_worlds, |w: &Worlds| {
    DiffCell {
        name: "receipt diff",
        args_a: vec![
            "receipt".into(),
            "diff".into(),
            w.receipt_a.to_string_lossy().into_owned(),
            w.receipt_b.to_string_lossy().into_owned(),
        ],
        args_b: vec![
            "receipt".into(),
            "diff".into(),
            w.receipt_b.to_string_lossy().into_owned(),
            w.receipt_a.to_string_lossy().into_owned(),
        ],
    }
});

diff_control!(positive_conformance_differs_across_worlds, |w: &Worlds| {
    receipt_cell(
        "receipt conformance",
        &["receipt", "conformance"],
        &[],
        &w.receipt_a,
        &w.receipt_b,
    )
});

diff_control!(positive_search_differs_across_worlds, |w: &Worlds| {
    // Same pattern against structurally different worlds: 1 match in A, 0 in B.
    dir_cell(
        "receipt search --pattern transform",
        &["receipt", "search", "--pattern", "transform"],
        "--receipts-path",
        &w.dir_a,
        &w.dir_b,
    )
});

diff_control!(positive_query_differs_across_worlds, |w: &Worlds| {
    dir_cell(
        "receipt query --q type=transform",
        &["receipt", "query", "--q", "type=transform"],
        "--receipts-path",
        &w.dir_a,
        &w.dir_b,
    )
});
