//! Integration court: seal a fixture `.aod`, then run the Zero-Text Audit Gate
//! (`scripts/aod_zero_text_gate.py`) — AFFI-SPEC-26.11.19 PR-2 / DoD.
//!
//! Law: a sealed artifact under the admitted CLI must yield exit 0 from the
//! gate (`strings -n 4` + UTF-8 heuristic, domain deny-list). A planted
//! domain token must yield exit 2 (`REFUSED_AOD_TEXT`).

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn affi(cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi builds");
    cmd.current_dir(cwd);
    cmd.env_remove("RUST_LOG");
    cmd
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn gate_script() -> PathBuf {
    repo_root().join("scripts/aod_zero_text_gate.py")
}

fn ontology() -> String {
    "aa".repeat(32)
}

fn lease(tag: u8) -> String {
    format!("{:0>64}", format!("{:x}", tag))
}

fn write_page(dir: &Path, rows: u64, stride: usize) -> PathBuf {
    let page: Vec<u8> = (0..(rows as usize * stride))
        .map(|i| (i % 251) as u8)
        .collect();
    let path = dir.join("page.bin");
    fs::write(&path, &page).unwrap();
    path
}

fn run_gate(aod: &Path) -> std::process::Output {
    StdCommand::new("python3")
        .arg(gate_script())
        .arg(aod)
        .output()
        .expect("python3 scripts/aod_zero_text_gate.py")
}

#[test]
fn sealed_fixture_passes_zero_text_gate() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let rows = 2u64;
    let page = write_page(cwd, rows, 8);
    let out = cwd.join("fixture.aod");

    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            &ontology(),
            "--lease",
            &lease(0x11),
            "--rows",
            &rows.to_string(),
            "--in",
            page.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(out.is_file());
    let bytes = fs::read(&out).unwrap();
    assert!(bytes.len() >= 416);
    assert_eq!(&bytes[0..4], &[0x9F, 0x1A, 0xC4, 0x01]);

    let gate = run_gate(&out);
    let stdout = String::from_utf8_lossy(&gate.stdout);
    let stderr = String::from_utf8_lossy(&gate.stderr);
    assert!(
        gate.status.success(),
        "zero-text gate must PASS on sealed fixture\nstdout={stdout}\nstderr={stderr}"
    );
    assert!(
        stdout.contains("PASS"),
        "gate stdout missing PASS: {stdout}"
    );
}

#[test]
fn planted_domain_token_refuses_zero_text_gate() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let rows = 1u64;
    let page = write_page(cwd, rows, 8);
    let out = cwd.join("clean.aod");

    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            &ontology(),
            "--lease",
            &lease(0x22),
            "--rows",
            &rows.to_string(),
            "--in",
            page.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Plant a deny-list token into a copy (does not mutate the seal path).
    let mut dirty = fs::read(&out).unwrap();
    dirty.extend_from_slice(b"CTRT");
    let dirty_path = cwd.join("dirty.aod");
    fs::write(&dirty_path, &dirty).unwrap();

    let gate = run_gate(&dirty_path);
    let code = gate.status.code();
    let stderr = String::from_utf8_lossy(&gate.stderr);
    assert_eq!(
        code,
        Some(2),
        "planted CTRT must exit 2 (REFUSED_AOD_TEXT)\nstderr={stderr}"
    );
    assert!(
        stderr.contains("REFUSED_AOD_TEXT") || stderr.contains("CTRT"),
        "stderr={stderr}"
    );
}

#[test]
fn gate_script_self_test_passes() {
    let out = StdCommand::new("python3")
        .arg(gate_script())
        .arg("--self-test")
        .output()
        .expect("python3 --self-test");
    assert!(
        out.status.success(),
        "self-test failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
