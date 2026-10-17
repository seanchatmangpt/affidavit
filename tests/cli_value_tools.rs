//! CLI value-tools court (v26.10.17 matrix, lane: cli_value_tools).
//!
//! Real-subprocess courts over the `affi` binary for the value-tool verbs:
//! `dossier emit` / `dossier inspect` (default-binary negative layer),
//! `guide search`, `affi doctor`, `receipt catalog`, `receipt search`,
//! `receipt profile`, `receipt visualize`.
//!
//! Chicago-style: real binary, real fixtures on disk, assert on real process
//! state (exit codes, parsed stdout). Every court has a positive control and,
//! where refusal is the contract, a typed refusal that must fire.
//!
//! Receipt: `cargo test --test cli_value_tools` — see run log for counts.

// value-matrix: (dossier, emit)
// value-matrix: (dossier, inspect)
// value-matrix: (guide, search)
// value-matrix: (affi, doctor)
// value-matrix: (receipt, catalog)
// value-matrix: (receipt, search)
// value-matrix: (receipt, profile)
// value-matrix: (receipt, visualize)

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static SCRATCH_SEQ: AtomicUsize = AtomicUsize::new(0);

fn affi() -> Command {
    Command::new(env!("CARGO_BIN_EXE_affi"))
}

/// Unique scratch dir, self-cleaning (lane law: no orphan build/test roots).
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cli_value_tools_{}_{tag}_{}_{}",
        std::process::id(),
        SCRATCH_SEQ.fetch_add(1, Ordering::SeqCst),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

/// Emit two events and assemble a real receipt in `dir`; returns the receipt
/// path. Real lifecycle, no fixture shims.
fn make_receipt(dir: &Path, name: &str) -> PathBuf {
    let out = affi()
        .current_dir(dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "emit",
            "--object",
            "src:artifact",
            "--payload",
            "-",
        ])
        .output()
        .expect("spawn affi receipt emit");
    assert!(
        out.status.success(),
        "receipt emit failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::write(dir.join("payload2.bin"), b"world");
    let out = affi()
        .current_dir(dir)
        .args([
            "receipt",
            "emit",
            "--type",
            "transform",
            "--object",
            "dst:artifact",
            "--payload",
            "-",
        ])
        .output()
        .expect("spawn affi receipt emit");
    assert!(
        out.status.success(),
        "receipt emit #2 failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = affi()
        .current_dir(dir)
        .args(["receipt", "assemble", "--out", name])
        .output()
        .expect("spawn affi receipt assemble");
    assert!(
        out.status.success(),
        "receipt assemble failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let p = dir.join(name);
    assert!(p.exists(), "assemble must produce {name}");
    p
}

// ---------------------------------------------------------------------------
// dossier emit / dossier inspect — default-binary negative layer
// (crypto-trust plane not compiled: typed REFUSED_UNSUPPORTED, never a fiction)
// ---------------------------------------------------------------------------

#[test]
fn dossier_emit_default_binary_is_typed_refusal() {
    let dir = scratch("dossier_emit");
    // A real, existing receipts dir (the refusal is about the missing feature
    // plane, not a missing directory — keep the precondition honest).
    std::fs::create_dir_all(dir.join("receipts")).unwrap();
    let out = affi()
        .current_dir(&dir)
        .args([
            "dossier",
            "emit",
            "--receipts-dir",
            "receipts",
            "--out",
            "dossier.json",
        ])
        .output()
        .expect("spawn affi dossier emit");
    assert!(
        !out.status.success(),
        "dossier emit must refuse in the default binary"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("REFUSED_UNSUPPORTED"),
        "typed refusal expected, got: {combined}"
    );
    assert!(
        !dir.join("dossier.json").exists(),
        "a refused dossier emit must not write an artifact"
    );
    cleanup(&dir);
}

#[test]
fn dossier_inspect_default_binary_is_typed_refusal() {
    let dir = scratch("dossier_inspect");
    let dossier = dir.join("dossier.json");
    std::fs::write(&dossier, b"{\"format\":\"CTT-DOSSIER-v1\"}").unwrap();
    let out = affi()
        .current_dir(&dir)
        .args(["dossier", "inspect", "dossier.json"])
        .output()
        .expect("spawn affi dossier inspect");
    assert!(
        !out.status.success(),
        "dossier inspect must refuse in the default binary"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("REFUSED_UNSUPPORTED"),
        "typed refusal expected, got: {combined}"
    );
    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// guide search — real registry search
// ---------------------------------------------------------------------------

#[test]
fn guide_search_known_term_hit_counts() {
    let dir = scratch("guide_hit");
    // Real search over the compile-time registry: "search" must hit the
    // guide/search verb itself, and the JSON count must equal the human
    // header count for the same query (two renderings, one truth).
    for query in ["search", "verify", "receipt"] {
        let out = affi()
            .current_dir(&dir)
            .args(["guide", "search", "--keyword", query, "--format", "json"])
            .output()
            .expect("spawn affi guide search");
        assert!(out.status.success(), "guide search must exit 0");
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let doc: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("guide search --format json must emit JSON: {e}\n{stdout}"));
        let count = doc["count"]
            .as_u64()
            .unwrap_or_else(|| panic!("json output must carry a count field:\n{stdout}"));
        assert!(count > 0, "known term '{query}' must hit at least one verb");
        let results = doc["results"].as_array().expect("results array");
        assert_eq!(
            results.len() as u64,
            count,
            "count field must match the results array length"
        );
        // Human rendering of the same query must state the same count.
        let human = affi()
            .current_dir(&dir)
            .args(["guide", "search", "--keyword", query])
            .output()
            .expect("spawn affi guide search");
        assert!(human.status.success());
        let human_out = String::from_utf8_lossy(&human.stdout).to_string();
        let expected = format!("({} match", count);
        assert!(
            human_out.contains(&expected),
            "human header must carry the same hit count {count}:\n{human_out}"
        );
    }
    cleanup(&dir);
}

#[test]
fn guide_search_exact_pair_hit() {
    let dir = scratch("guide_pair");
    let out = affi()
        .current_dir(&dir)
        .args([
            "guide",
            "search",
            "--keyword",
            "federation",
            "--format",
            "json",
        ])
        .output()
        .expect("spawn affi guide search");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let doc: serde_json::Value = serde_json::from_str(&stdout).expect("json output");
    let results = doc["results"].as_array().expect("results array");
    let pairs: Vec<(String, String)> = results
        .iter()
        .map(|r| {
            (
                r["noun"].as_str().unwrap_or_default().to_string(),
                r["verb"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect();
    assert!(
        pairs.contains(&("standing".to_string(), "certify".to_string())),
        "federation search must surface the standing certify verb: {pairs:?}"
    );
    cleanup(&dir);
}

#[test]
fn guide_search_zero_match_is_honest_exit_0() {
    let dir = scratch("guide_zero");
    let query = "zzqqxynonexistentverb";
    let human = affi()
        .current_dir(&dir)
        .args(["guide", "search", "--keyword", query])
        .output()
        .expect("spawn affi guide search");
    assert!(
        human.status.success(),
        "a zero-match search is a valid answer, not an error"
    );
    let human_out = String::from_utf8_lossy(&human.stdout).to_string();
    assert!(
        human_out.contains(&format!("No verbs matched '{query}'")),
        "honest zero-match message expected:\n{human_out}"
    );
    let json = affi()
        .current_dir(&dir)
        .args(["guide", "search", "--keyword", query, "--format", "json"])
        .output()
        .expect("spawn affi guide search");
    assert!(json.status.success());
    let doc: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&json.stdout)).expect("json output");
    assert_eq!(
        doc["query"].as_str(),
        Some(query),
        "json output must echo the query"
    );
    assert_eq!(
        doc["results"].as_array().map(|a| a.len()),
        Some(0),
        "json results must be empty"
    );
    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// affi doctor — real checks, findings sorted by id, one per registered check
// ---------------------------------------------------------------------------

/// Parse doctor stdout into (status, id) pairs.
fn doctor_findings(stdout: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix('[') {
            if let Some((status, tail)) = rest.split_once(']') {
                let status = status.trim();
                if !matches!(status, "ok" | "warn" | "FAIL") {
                    continue;
                }
                if let Some((id, _)) = tail.trim().split_once(": ") {
                    found.push((status.to_string(), id.to_string()));
                }
            }
        }
    }
    found
}

#[test]
fn doctor_findings_sorted_one_per_registered_check() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Real workspace fixture: `.affi/` exists but no working.json (warn path),
    // and real completions copied from the repo (ok path). Genesis is
    // compile-time ok.
    let ws = scratch("doctor_ws");
    std::fs::create_dir_all(ws.join(".affi")).unwrap();
    std::fs::create_dir_all(ws.join("completions")).unwrap();
    for ext in ["bash", "zsh", "fish"] {
        std::fs::copy(
            repo.join(format!("completions/affi.{ext}")),
            ws.join(format!("completions/affi.{ext}")),
        )
        .expect("copy completions fixture");
    }
    let out = affi()
        .current_dir(&ws)
        .args(["affi", "doctor"])
        .output()
        .expect("spawn affi doctor");
    assert!(
        out.status.success(),
        "doctor must exit 0 with no FAIL findings (warns are legal): {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let findings = doctor_findings(&stdout);
    assert!(
        findings.len() >= 3,
        "default doctor run must surface the built-in checks:\n{stdout}"
    );
    // Value law: findings sorted by check id.
    let mut ids: Vec<&str> = findings.iter().map(|(_, id)| id.as_str()).collect();
    let sorted = ids.clone();
    ids.sort_unstable();
    assert_eq!(sorted, ids, "doctor findings must be sorted by check id");
    // Exactly one finding per registered check (no duplicate ids).
    let unique: std::collections::BTreeSet<&str> = sorted.iter().copied().collect();
    assert_eq!(
        unique.len(),
        sorted.len(),
        "exactly one finding per registered check; got {findings:?}"
    );
    // The three built-ins with their lawful statuses in the repo root.
    let status_of = |id: &str| {
        findings
            .iter()
            .find(|(_, i)| i == id)
            .map(|(s, _)| s.clone())
    };
    assert_eq!(
        status_of("genesis-seed-version").as_deref(),
        Some("ok"),
        "genesis seed must be ok"
    );
    assert_eq!(
        status_of("shell-completions").as_deref(),
        Some("ok"),
        "completions must be ok when completions/ is present"
    );
    assert_eq!(
        status_of("affi-working-dir").as_deref(),
        Some("warn"),
        ".affi/ without working.json -> warn, not ok, not FAIL"
    );
    cleanup(&ws);
}

#[test]
fn doctor_receipt_store_check_is_real() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = scratch("doctor_store");
    // Empty receipts dir -> the count check warns with the real path in the
    // message (a real check over a real directory, not a canned row).
    let empty = dir.join("empty-store");
    std::fs::create_dir_all(&empty).unwrap();
    let out = affi()
        .current_dir(&repo)
        .args(["affi", "doctor", "--receipts", empty.to_str().unwrap()])
        .output()
        .expect("spawn affi doctor");
    assert!(out.status.success(), "warn-only run must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let findings = doctor_findings(&stdout);
    let count = findings
        .iter()
        .find(|(_, id)| id == "receipt-store-count")
        .expect("receipt-store-count finding must appear for a --receipts dir");
    assert_eq!(count.0, "warn", "empty store is a warn, not a FAIL");
    // Still sorted, still lossless: one finding per check id.
    let mut ids: Vec<&str> = findings.iter().map(|(_, id)| id.as_str()).collect();
    let sorted = ids.clone();
    ids.sort_unstable();
    assert_eq!(sorted, ids);
    cleanup(&dir);
}

#[test]
fn doctor_missing_receipts_path_is_fail_exit_4() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = scratch("doctor_missing");
    let out = affi()
        .current_dir(repo)
        .args([
            "affi",
            "doctor",
            "--receipts",
            dir.join("nope").to_str().unwrap(),
        ])
        .output()
        .expect("spawn affi doctor");
    assert_eq!(
        out.status.code(),
        Some(4),
        "missing receipt path is IO_ERROR exit 4"
    );
    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// receipt catalog — real fixtures table
// ---------------------------------------------------------------------------

#[test]
fn catalog_lists_builtin_fixtures() {
    let dir = scratch("catalog");
    let out = affi()
        .current_dir(&dir)
        .args(["receipt", "catalog"])
        .output()
        .expect("spawn affi receipt catalog");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    for (name, events) in [
        ("linear-3", 3),
        ("linear-5", 5),
        ("branch-4", 4),
        ("parallel-6", 6),
        ("loop-example", 3),
        ("minimal-1", 1),
        ("audit-trail-10", 10),
    ] {
        assert!(
            stdout.contains(name),
            "builtin fixture {name} must be listed:\n{stdout}"
        );
        assert!(
            stdout.contains(&events.to_string()),
            "fixture event count {events} must appear"
        );
    }
    assert!(
        stdout.contains("(source: fixtures.json)"),
        "catalog must disclose its source\n{stdout}"
    );
    cleanup(&dir);
}

#[test]
fn catalog_filter_narrows_rows() {
    let dir = scratch("catalog_filter");
    // --filter-events 10 must keep exactly audit-trail-10.
    let out = affi()
        .current_dir(&dir)
        .args(["receipt", "catalog", "--filter-events", "10"])
        .output()
        .expect("spawn affi receipt catalog");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains("audit-trail-10"),
        "10-event filter must keep audit-trail-10:\n{stdout}"
    );
    assert!(
        !stdout.contains("linear-3"),
        "10-event filter must drop linear-3:\n{stdout}"
    );
    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// receipt search — real hit/miss over a real receipt
// ---------------------------------------------------------------------------

#[test]
fn receipt_search_hit_and_miss() {
    let dir = scratch("search");
    make_receipt(&dir, "receipt.json");
    // HIT: 'transform' matches event 1's type.
    let hit = affi()
        .current_dir(&dir)
        .args([
            "receipt",
            "search",
            "--pattern",
            "transform",
            "--receipts-path",
            "receipt.json",
        ])
        .output()
        .expect("spawn affi receipt search");
    assert!(hit.status.success());
    let hit_out = String::from_utf8_lossy(&hit.stdout).to_string();
    assert!(
        hit_out.contains("search 'transform': 1 match(es)"),
        "expected exactly one transform hit:\n{hit_out}"
    );
    assert!(hit_out.contains("seq=1"), "hit must carry seq=1");
    // MISS: honest zero.
    let miss = affi()
        .current_dir(&dir)
        .args([
            "receipt",
            "search",
            "--pattern",
            "zzz-no-such-event",
            "--receipts-path",
            "receipt.json",
        ])
        .output()
        .expect("spawn affi receipt search");
    assert!(miss.status.success(), "a miss is a valid answer, exit 0");
    let miss_out = String::from_utf8_lossy(&miss.stdout).to_string();
    assert!(
        miss_out.contains("search 'zzz-no-such-event': 0 match(es)"),
        "honest zero-match search:\n{miss_out}"
    );
    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// receipt profile — real sustained-workload numbers, bounded run
// ---------------------------------------------------------------------------

#[test]
fn receipt_profile_bounded_run_reports_real_cycles() {
    let dir = scratch("profile");
    let out = affi()
        .current_dir(&dir)
        .args(["receipt", "profile", "--duration", "1"])
        .output()
        .expect("spawn affi receipt profile");
    assert!(out.status.success(), "profile must exit 0");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("Profile workload complete."),
        "profile must report completion:\n{stderr}"
    );
    // The cycle count is real: a 1s run must execute at least one cycle and
    // the reported number must parse as a positive integer.
    let cycles_line = stderr
        .lines()
        .find(|l| l.contains("Executed") && l.contains("cycles"))
        .unwrap_or_else(|| panic!("cycle-count line missing:\n{stderr}"));
    let cycles: u64 = cycles_line
        .split_whitespace()
        .find_map(|tok| tok.parse::<u64>().ok())
        .unwrap_or_else(|| panic!("no parseable cycle count in: {cycles_line}"));
    assert!(cycles >= 1, "1s sustained workload must run >=1 cycle");
    cleanup(&dir);
}

// ---------------------------------------------------------------------------
// receipt visualize — real DOT/JSON graph, bad format enum -> typed error
// ---------------------------------------------------------------------------

#[test]
fn visualize_dot_graph_is_real() {
    let dir = scratch("viz_dot");
    make_receipt(&dir, "receipt.json");
    let out = affi()
        .current_dir(&dir)
        .args(["receipt", "visualize", "--format", "dot", "receipt.json"])
        .output()
        .expect("spawn affi receipt visualize");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        stdout.contains("digraph"),
        "DOT output must contain a digraph:\n{stdout}"
    );
    // Two emitted events -> two graph nodes named after their types.
    assert!(stdout.contains("emit"), "emit node expected in DOT");
    assert!(stdout.contains("transform"), "transform node expected");
    cleanup(&dir);
}

#[test]
fn visualize_json_graph_is_real() {
    let dir = scratch("viz_json");
    make_receipt(&dir, "receipt.json");
    let out = affi()
        .current_dir(&dir)
        .args(["receipt", "visualize", "--format", "json", "receipt.json"])
        .output()
        .expect("spawn affi receipt visualize");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let doc: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("visualize --format json must emit JSON: {e}\n{stdout}"));
    let nodes = doc["nodes"].as_array().expect("nodes array");
    assert_eq!(nodes.len(), 2, "two emitted events -> two nodes");
    cleanup(&dir);
}

#[test]
fn visualize_bad_format_is_typed_refusal() {
    let dir = scratch("viz_bad");
    make_receipt(&dir, "receipt.json");
    let out = affi()
        .current_dir(&dir)
        .args(["receipt", "visualize", "--format=xml", "receipt.json"])
        .output()
        .expect("spawn affi receipt visualize");
    assert!(
        !out.status.success(),
        "an unsupported format must be refused"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("invalid value 'xml'"),
        "typed enum refusal expected:\n{combined}"
    );
    assert!(
        combined.contains("dot, json"),
        "refusal must name the supported formats:\n{combined}"
    );
    cleanup(&dir);
}
