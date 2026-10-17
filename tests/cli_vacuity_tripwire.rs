// Vacuity tripwire court — the mechanical detector for the VACUOUS class in
// docs/reference/CLI_VACUITY_AUDIT_26_10_17.md.
//
// Law: no verb may return success whose output is independent of its input.
// Each cell runs the built `affi` binary against two different real fixtures
// and asserts the outcome (exit code + derived field) actually flips.
//
// Lane ownership: the legal-hold status + license-compliance cells are ACTIVE
// (v26.10.17 eliminate lane). The remaining VACUOUS-six cells stay `#[ignore]`d
// until their owning lanes land their fixes — an un-ignored failing cell is a
// court, a faked pass is not.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_affi")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("affi-vacuity-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Build a real two-event chain + a one-event license chain in `<dir>/receipts`.
fn fixture_with_chains(tag: &str, license_object: &str) -> PathBuf {
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
        let st = Command::new(bin())
            .args(["receipt", "emit", "--event-type", event_type, "--object", object, "--payload", payload])
            .current_dir(&dir)
            .status()
            .expect("run affi emit");
        assert!(st.success(), "emit {event_type} failed");
    }
    let st = Command::new(bin())
        .args(["receipt", "assemble", "--out", "receipts/chain1.json"])
        .current_dir(&dir)
        .status()
        .expect("run affi assemble");
    assert!(st.success(), "assemble chain1 failed");

    let st = Command::new(bin())
        .args(["receipt", "emit", "--event-type", "license-scan", "--object", license_object, "--payload", lic.to_str().unwrap()])
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
    fs::write(&p, format!("{{\"allowed_licenses\": [{}]}}", list.join(", "))).unwrap();
    p
}

// ---------------------------------------------------------------------------
// ACTIVE CELL 1: `legal-hold status` — REFUSED_NO_CHAINS on empty input,
// derived events_preserved on real chains (audit defect 5).
// ---------------------------------------------------------------------------

#[test]
fn legal_hold_status_refuses_no_chains_on_empty_dir() {
    let dir = scratch("lh-empty");
    let receipts = dir.join("receipts");
    fs::create_dir_all(&receipts).unwrap();

    let out = Command::new(bin())
        .args(["legal-hold", "status", "--matter", "m-1"])
        .env("AFFI_RECEIPTS_PATH", &receipts)
        .output()
        .expect("run affi legal-hold status");

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(224), "empty dir must exit 224 (REFUSED_NO_CHAINS via CliExecutionError 1504); stderr: {stderr}");
    assert!(stderr.contains("REFUSED_NO_CHAINS"), "typed refusal must be named; stderr: {stderr}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("EVIDENTIARY_HOLD_ACTIVE"),
        "no fabricated hold over zero chains");
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

    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("JSON stdout");
    assert_eq!(v["status"], "EVIDENTIARY_HOLD_ACTIVE");
    // events_preserved must equal the actual event count across the chains on
    // disk (summed from the fixtures, never the fabricated constant 1).
    let on_disk: usize = fs::read_dir(&receipts).unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .map(|e| {
            let r: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(e.path()).unwrap()).unwrap();
            r["events"].as_array().map(|a| a.len()).unwrap_or(0)
        })
        .sum();
    assert!(on_disk > 1, "fixture must hold multiple events; got {on_disk}");
    assert_eq!(v["events_preserved"], on_disk, "events_preserved must be derived from the chains");
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
    assert_eq!(out.status.code(), Some(2), "tampered chain must REJECT, not certify");
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
        .args(["receipt", "license-compliance", "--receipts-path", "receipts", "--license-policy", policy.to_str().unwrap(), "--format", "json"])
        .current_dir(&dir)
        .output()
        .expect("run affi license-compliance");

    assert_eq!(out.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["compliant"], true, "MIT is allowed; full output: {v}");
    assert_eq!(v["license_events_found"], 1);
}

#[test]
fn license_compliance_disallowed_license_flips_verdict_and_names_offender() {
    let dir = fixture_with_chains("lc-disallowed", "GPL-3.0-only:license");
    let policy = write_policy(&dir, "policy.json", &["MIT", "Apache-2.0"]);

    let out = Command::new(bin())
        .args(["receipt", "license-compliance", "--receipts-path", "receipts", "--license-policy", policy.to_str().unwrap(), "--format", "json"])
        .current_dir(&dir)
        .output()
        .expect("run affi license-compliance");

    assert_eq!(out.status.code(), Some(2), "disallowed license must exit REJECT (2)");
    let v: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    assert_eq!(v["compliant"], false);
    assert_eq!(v["violations"][0]["license"], "GPL-3.0-only", "offending license must be named");
    assert!(v["status"].as_str().unwrap().contains("GPL-3.0-only"));
}

#[test]
fn license_compliance_zero_allowed_licenses_fails_on_any_license_event() {
    let dir = fixture_with_chains("lc-strict", "MIT:license");
    let policy = write_policy(&dir, "policy.json", &[]);

    let out = Command::new(bin())
        .args(["receipt", "license-compliance", "--receipts-path", "receipts", "--license-policy", policy.to_str().unwrap()])
        .current_dir(&dir)
        .output()
        .expect("run affi license-compliance");

    assert_eq!(out.status.code(), Some(2), "zero allowed + any license event must fail");
}

// ---------------------------------------------------------------------------
// IGNORED CELLS — the rest of the VACUOUS six; owned by other lanes.
// Un-ignore only when the owning lane's fix lands. Each ignored cell states
// the flip its lane must make pass.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "VACUOUS defect 1 (receipt audit: stdout null) — owned by another lane; flip when the scan-and-report fix lands"]
fn receipt_audit_output_depends_on_input() {
    let _ = scratch("audit");
    // Fix must make: two different receipt stores produce different stdout.
    todo!("other lane")
}

#[test]
#[ignore = "VACUOUS defect 3 (receipt test: dummy verb) — owned by another lane; flip when the verb executes or is deleted"]
fn receipt_test_is_falsifiable() {
    todo!("other lane")
}

#[test]
#[ignore = "VACUOUS defect 4 (certificate fre-902: canned affidavit) — owned by another lane; flip when the CTRT chain recompute gates certification"]
fn certificate_fre_902_recomputes_chain() {
    todo!("other lane")
}

#[test]
#[ignore = "VACUOUS defect 6 (restitution calculate: constant tariffs) — owned by another lane; flip when figures derive from a supplied schedule or refuse"]
fn restitution_calculate_derives_from_schedule() {
    todo!("other lane")
}
