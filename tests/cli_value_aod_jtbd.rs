//! Value-matrix court for AOD JTBD close + proof-carrying + standing certify-aod.
//!
//! Covers the eight REGISTRY pairs that landed with the Arrow-on-Disk lanes
//! and were missing from `cli_value_*` markers while `COMPLETE=true`.
//! Chicago: real `affi` binary, real files, real exit codes — no mocks.

// value-matrix: (aod, seal)
// value-matrix: (aod, verify)
// value-matrix: (aod, witness)
// value-matrix: (aod, rotate)
// value-matrix: (aod, fold)
// value-matrix: (standing, certify-aod)
// value-matrix: (receipt, proof-carrying)
// value-matrix: (receipt, proof-carrying-verify)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn affi(cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi builds");
    cmd.current_dir(cwd);
    cmd.env_remove("RUST_LOG");
    cmd
}

fn write_page(dir: &Path, rows: u64, stride: usize) -> (std::path::PathBuf, usize) {
    let page: Vec<u8> = (0..(rows as usize * stride))
        .map(|i| (i % 251) as u8)
        .collect();
    let path = dir.join("page.bin");
    fs::write(&path, &page).unwrap();
    (path, stride)
}

fn mint_ocel_receipt(dir: &Path) -> std::path::PathBuf {
    for (i, (ty, obj)) in [
        ("create", "o1:order"),
        ("pay", "o1:order"),
        ("ship", "o1:order"),
    ]
    .into_iter()
    .enumerate()
    {
        affi(dir)
            .args([
                "receipt",
                "emit",
                "--type",
                ty,
                "--object",
                obj,
                "--payload",
                "-",
            ])
            .write_stdin(format!("pc-{i}"))
            .assert()
            .success();
    }
    let out = dir.join("receipt.json");
    affi(dir)
        .args(["receipt", "assemble", "--out"])
        .arg(&out)
        .assert()
        .success();
    out
}

#[test]
fn aod_seal_witness_live_rotate_fold_value() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let rows = 2u64;
    let (page, stride) = write_page(cwd, rows, 8);
    let ontology = affidavit_core::FIXTURE_ONTOLOGY_HASH_HEX;
    let lease1 = format!("{:0>64}", "bb");
    let out1 = cwd.join("v1.aod");

    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            ontology,
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
    assert_eq!(&bytes[0..4], &[0x9F, 0x1A, 0xC4, 0x01]);

    affi(cwd)
        .args(["aod", "witness", out1.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"region3_mapped\": false"));

    let registry = cwd.join("leases.afal");
    let strides = stride.to_string();
    affi(cwd)
        .args([
            "aod",
            "verify",
            "--live",
            out1.to_str().unwrap(),
            "--ontology",
            ontology,
            "--strides",
            &strides,
            "--registry",
            registry.to_str().unwrap(),
        ])
        .assert()
        .success();

    let lease2 = format!("{:0>64}", "cc");
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
fn standing_certify_aod_seals_partial_alive() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();

    affi(cwd)
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
        .write_stdin("aod federation subject")
        .assert()
        .success();
    affi(cwd)
        .args(["receipt", "assemble", "--out", "source.json"])
        .assert()
        .success();

    let (page, _) = write_page(cwd, 2, 8);
    let ontology = affidavit_core::FIXTURE_ONTOLOGY_HASH_HEX;
    let lease = format!("{:0>64}", "bb");
    affi(cwd)
        .args([
            "aod",
            "seal",
            "--ontology",
            ontology,
            "--lease",
            &lease,
            "--rows",
            "2",
            "--in",
            page.to_str().unwrap(),
            "--out",
            "artifact.aod",
        ])
        .assert()
        .success();

    fs::write(
        cwd.join("subject.json"),
        r#"{
  "subject": "seanchatmangpt/affidavit",
  "base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "candidate": "cccccccccccccccccccccccccccccccccccccccc"
}"#,
    )
    .unwrap();

    affi(cwd)
        .args([
            "standing",
            "certify-aod",
            "--receipt",
            "source.json",
            "--aod",
            "artifact.aod",
            "--subject",
            "subject.json",
            "--scope",
            "repo:seanchatmangpt/affidavit",
            "--out",
            "standing-aod.json",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PARTIAL_ALIVE"));
}

#[test]
fn proof_carrying_mints_subject_and_verify_refuses_bad_pk() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let receipt = mint_ocel_receipt(cwd);
    let model = cwd.join("model.json");
    fs::write(&model, r#"[["create","pay"],["pay","ship"]]"#).unwrap();
    let subject = cwd.join("subject.json");

    affi(cwd)
        .args([
            "receipt",
            "proof-carrying",
            "--receipt",
            receipt.to_str().unwrap(),
            "--object-type",
            "order",
            "--log-ref",
            "log:court",
            "--model",
            model.to_str().unwrap(),
            "--format",
            "json",
            "--out",
            subject.to_str().unwrap(),
        ])
        .assert()
        .success();

    let raw = fs::read_to_string(&subject).unwrap();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(v["profile"], "affidavit/ocel-conformance-subject/v1");
    assert_eq!(v["subject_digest"].as_str().unwrap().len(), 64);

    // Exercise proof-carrying-verify: wrong pk length is a typed refusal.
    let out = affi(cwd)
        .args([
            "receipt",
            "proof-carrying-verify",
            subject.to_str().unwrap(),
            "--pk",
            "00",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_ne!(out.status.code(), Some(0));
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("REFUSED") || text.contains("pk") || text.contains("seal"),
        "typed refusal expected, got: {text}"
    );
}

#[cfg(feature = "crypto-trust")]
#[test]
fn proof_carrying_seal_and_verify_round_trip() {
    let dir = TempDir::new().unwrap();
    let cwd = dir.path();
    let receipt = mint_ocel_receipt(cwd);
    let model = cwd.join("model.json");
    fs::write(&model, r#"[["create","pay"],["pay","ship"]]"#).unwrap();
    let subject = cwd.join("subject.json");
    let seed = "a5".repeat(32);

    let out = affi(cwd)
        .args([
            "receipt",
            "proof-carrying",
            "--receipt",
            receipt.to_str().unwrap(),
            "--object-type",
            "order",
            "--model",
            model.to_str().unwrap(),
            "--seal-seed",
            &seed,
            "--format",
            "json",
            "--out",
            subject.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "seal mint: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let start = stdout.find('{').unwrap_or(0);
    let v: serde_json::Value = serde_json::from_str(&stdout[start..]).unwrap();
    let pk = v["seal"]["pk"].as_str().expect("pk embedded");

    affi(cwd)
        .args([
            "receipt",
            "proof-carrying-verify",
            subject.to_str().unwrap(),
            "--pk",
            pk,
            "--format",
            "json",
        ])
        .assert()
        .success();
}
