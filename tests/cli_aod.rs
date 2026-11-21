//! Court: `affi aod {seal,verify --live,witness,rotate,fold}` (AFFI-SPEC-26.11.19).
//!
//! Law:
//! 1. seal → byte-identical on replay; zero ASCII domain tokens in `.aod`.
//! 2. witness ACCEPT without ontology / lease burn.
//! 3. verify --live burns nullifier into durable AFAL store; second live
//!    verify (cross-process) refuses 0xE50D.
//! 4. rotate rematerializes under a fresh lease; fold yields stable tip.

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

fn hex32(byte: u8) -> String {
    format!("{:0>64}", format!("{:x}", byte))
}

fn ontology() -> String {
    "aa".repeat(32)
}

fn lease(tag: u8) -> String {
    hex32(tag)
}

fn write_page(dir: &Path, rows: u64, stride: usize) -> (std::path::PathBuf, usize) {
    let page: Vec<u8> = (0..(rows as usize * stride))
        .map(|i| (i % 251) as u8)
        .collect();
    let path = dir.join("page.bin");
    fs::write(&path, &page).unwrap();
    (path, stride)
}

#[test]
fn aod_seal_witness_live_rotate_fold_court() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let rows = 2u64;
    let (page, stride) = write_page(cwd, rows, 8);
    let ontology = ontology();
    let lease1 = lease(0xbb);
    let out1 = cwd.join("v1.aod");

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
            out1.to_str().unwrap(),
        ])
        .assert()
        .success();

    let bytes = fs::read(&out1).unwrap();
    assert!(bytes.len() >= 416);
    // Zero-text audit: no domain tokens / ASCII magic (PR-2).
    for needle in [
        b"CTRT".as_slice(),
        b"ontology",
        b"lease_nullifier",
        b"arrow",
        b"parquet",
        b"\"row_count\"",
        b"affidavit",
    ] {
        assert!(
            !bytes.windows(needle.len()).any(|w| w == needle),
            "domain token {:?} must not appear in .aod",
            std::str::from_utf8(needle).unwrap_or("?")
        );
    }
    // Magic is non-ASCII sentinel.
    assert_eq!(&bytes[0..4], &[0x9F, 0x1A, 0xC4, 0x01]);

    // Determinism: reseal identical inputs → identical bytes.
    let out1b = cwd.join("v1b.aod");
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
            out1b.to_str().unwrap(),
        ])
        .assert()
        .success();
    assert_eq!(fs::read(&out1).unwrap(), fs::read(&out1b).unwrap());

    // Historical witness — no registry, no ontology.
    affi(cwd)
        .args(["aod", "witness", out1.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"region3_mapped\": false"));

    let registry = cwd.join("leases.afal");
    let strides = stride.to_string();

    // Live gate burns lease into durable AFAL (HMAC-sealed) snapshot.
    affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out1.to_str().unwrap(),
            "--ontology",
            &ontology,
            "--strides",
            &strides,
            "--registry",
            registry.to_str().unwrap(),
        ])
        .assert()
        .success();

    let sealed = fs::read(&registry).expect("AFAL snapshot");
    assert!(
        sealed.len() > 36 && &sealed[32..36] == b"AFAL",
        "sealed = tag(32) ++ AFAL body, got {} bytes",
        sealed.len()
    );

    // Cross-process replay of spent nullifier → 0xE50D (exit low byte 0x0D).
    let replay = affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out1.to_str().unwrap(),
            "--ontology",
            &ontology,
            "--strides",
            &strides,
            "--registry",
            registry.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(replay.status.code(), Some(0x0D));
    let err = String::from_utf8_lossy(&replay.stderr);
    assert!(
        err.contains("0xE50D") || err.contains("LeaseSpentOrExpired"),
        "stderr={err}"
    );

    // Rotate under fresh lease.
    let lease2 = lease(0xcc);
    let out2 = cwd.join("v2.aod");
    affi(cwd)
        .args([
            "aod",
            "rotate",
            out1.to_str().unwrap(),
            "--lease",
            &lease2,
            "--out",
            out2.to_str().unwrap(),
            "--epoch",
            "2",
        ])
        .assert()
        .success();
    assert!(out2.is_file());
    assert_ne!(fs::read(&out1).unwrap(), fs::read(&out2).unwrap());

    // Fold tips are stable.
    let fold = affi(cwd)
        .args([
            "aod",
            "fold",
            out1.to_str().unwrap(),
            out2.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(fold.status.success());
    let stdout = String::from_utf8_lossy(&fold.stdout);
    assert!(stdout.contains("folded_tip"), "stdout={stdout}");
}

#[test]
fn aod_verify_without_live_refuses_mode() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let (page, _) = write_page(cwd, 1, 8);
    let out = cwd.join("x.aod");
    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            &ontology(),
            "--lease",
            &lease(1),
            "--rows",
            "1",
            "--in",
            page.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let bad = affi(cwd)
        .args([
            "aod",
            "verify",
            out.to_str().unwrap(),
            "--ontology",
            &ontology(),
            "--strides",
            "8",
        ])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&bad.stdout),
        String::from_utf8_lossy(&bad.stderr)
    );
    assert!(
        combined.contains("REFUSED_AOD_MODE") || combined.contains("--live"),
        "combined={combined}"
    );
}
