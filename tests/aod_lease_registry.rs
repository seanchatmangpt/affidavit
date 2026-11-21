//! Court: durable AFAL AOD lease registry — cross-process monoatomic anti-replay
//! (AFFI-SPEC-26.11.19 PR-5).
//!
//! Law:
//! 1. Live verify burns into a sealed AFAL snapshot (magic + HMAC tag).
//! 2. A second process reopening the same registry refuses replay with `0xE50D`.
//! 3. Forged / truncated snapshots fail closed (no empty re-admit).

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn affi(cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi builds");
    cmd.current_dir(cwd);
    cmd.env_remove("RUST_LOG");
    cmd
}

fn ontology() -> String {
    "aa".repeat(32)
}

fn lease(tag: u8) -> String {
    format!("{:0>64}", format!("{:x}", tag))
}

fn write_page(dir: &Path, rows: u64, stride: usize) -> std::path::PathBuf {
    let page: Vec<u8> = (0..(rows as usize * stride))
        .map(|i| (i % 251) as u8)
        .collect();
    let path = dir.join("page.bin");
    fs::write(&path, &page).unwrap();
    path
}

#[test]
fn cross_process_live_replay_refuses_0xe50d() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let rows = 2u64;
    let stride = 8usize;
    let page = write_page(cwd, rows, stride);
    let ontology = ontology();
    let lease1 = lease(0xbb);
    let out = cwd.join("v1.aod");
    let registry = cwd.join(".affi").join("aod-leases");

    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            &ontology,
            "--lease",
            &lease1,
            "--rows",
            &rows.to_string(),
            "--in",
            page.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Process A: live gate burns the monoatomic lease into AFAL.
    affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out.to_str().unwrap(),
            "--ontology",
            &ontology,
            "--strides",
            &stride.to_string(),
            "--registry",
            registry.to_str().unwrap(),
        ])
        .assert()
        .success();

    let sealed = fs::read(&registry).expect("AFAL snapshot written");
    assert!(
        sealed.len() > 36 && &sealed[32..36] == b"AFAL",
        "sealed snapshot = tag(32) ++ AFAL body, got {} bytes",
        sealed.len()
    );
    let key_path = Path::new(&format!("{}.key", registry.display())).to_path_buf();
    assert!(
        key_path.is_file(),
        "expected MAC key sibling at {}",
        key_path.display()
    );

    // Process B (fresh affi invocation): reopen durable store → 0xE50D.
    let replay = affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out.to_str().unwrap(),
            "--ontology",
            &ontology,
            "--strides",
            &stride.to_string(),
            "--registry",
            registry.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        replay.status.code(),
        Some(0x0D),
        "stderr={}",
        String::from_utf8_lossy(&replay.stderr)
    );
    let err = String::from_utf8_lossy(&replay.stderr);
    assert!(
        err.contains("0xE50D") || err.contains("LeaseSpentOrExpired"),
        "stderr={err}"
    );
}

#[test]
fn forged_afal_snapshot_refuses_open() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let rows = 1u64;
    let stride = 8usize;
    let page = write_page(cwd, rows, stride);
    let ontology = ontology();
    let out = cwd.join("x.aod");
    let registry = cwd.join("leases.afal");

    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            &ontology,
            "--lease",
            &lease(0x42),
            "--rows",
            "1",
            "--in",
            page.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out.to_str().unwrap(),
            "--ontology",
            &ontology,
            "--strides",
            &stride.to_string(),
            "--registry",
            registry.to_str().unwrap(),
        ])
        .assert()
        .success();

    let mut sealed = fs::read(&registry).unwrap();
    sealed[0] ^= 0xff;
    fs::write(&registry, &sealed).unwrap();

    // Fresh seal under a new lease so envelope/ontology are fine; only the
    // registry is forged — open must fail closed before any burn.
    let out2 = cwd.join("y.aod");
    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            &ontology,
            "--lease",
            &lease(0x43),
            "--rows",
            "1",
            "--in",
            page.to_str().unwrap(),
            "--out",
            out2.to_str().unwrap(),
        ])
        .assert()
        .success();

    let forged = affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out2.to_str().unwrap(),
            "--ontology",
            &ontology,
            "--strides",
            &stride.to_string(),
            "--registry",
            registry.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!forged.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&forged.stdout),
        String::from_utf8_lossy(&forged.stderr)
    );
    assert!(
        combined.contains("REFUSED_LEASE_REGISTRY") || combined.contains("CorruptLedger"),
        "combined={combined}"
    );
}
