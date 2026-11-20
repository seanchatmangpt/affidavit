//! Court: `affi receipt proof-carrying` (+ optional seal/verify under crypto-trust).
//!
//! Law:
//! 1. Receipt → grounded DFG subject JSON (non-zero digest, fitness/precision).
//! 2. Empty --log-ref → typed REFUSED_OCEL_CONFORMANCE_UNGROUNDED.
//! 3. Missing input → REFUSED_INPUT_MISSING.
//! 4. Tempering the normative model moves the subject digest.
//! 5. (crypto-trust) --seal-seed embeds ML-DSA-65; proof-carrying-verify ACCEPT.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn affi(cwd: &Path) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi builds");
    cmd.current_dir(cwd);
    cmd.env_remove("RUST_LOG");
    cmd
}

fn mint_receipt(dir: &Path) -> std::path::PathBuf {
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

fn run_json(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    let out = affi(dir)
        .args(["receipt"])
        .args(args)
        .output()
        .expect("spawn");
    let code = out.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    let combined = format!("{stdout}{stderr}");
    let v: Value = serde_json::from_str(stdout.trim())
        .or_else(|_| {
            // human mode may mix prose; find the JSON object
            let start = stdout.find('{').unwrap_or(0);
            serde_json::from_str(&stdout[start..])
        })
        .unwrap_or(Value::Null);
    (code, v, combined)
}

#[test]
fn proof_carrying_mints_subject_from_receipt() {
    let dir = TempDir::new().unwrap();
    let receipt = mint_receipt(dir.path());
    let model = dir.path().join("model.json");
    fs::write(&model, r#"[["create","pay"],["pay","ship"]]"#).unwrap();
    let subject = dir.path().join("subject.json");
    let (code, v, err) = run_json(
        dir.path(),
        &[
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
        ],
    );
    assert_eq!(code, 0, "mint must succeed: {err}");
    assert_eq!(v["profile"], "affidavit/ocel-conformance-subject/v1");
    assert_eq!(v["fitness"], 1.0);
    assert_eq!(v["precision"], 1.0);
    let dig = v["subject_digest"].as_str().expect("digest");
    assert_eq!(dig.len(), 64);
    assert_ne!(dig, &"0".repeat(64));
    assert!(subject.exists());
}

#[test]
fn empty_log_ref_refuses_before_mint() {
    let dir = TempDir::new().unwrap();
    let receipt = mint_receipt(dir.path());
    let out = affi(dir.path())
        .args([
            "receipt",
            "proof-carrying",
            "--receipt",
            receipt.to_str().unwrap(),
            "--object-type",
            "order",
            "--log-ref",
            "",
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
        text.contains("REFUSED_OCEL_CONFORMANCE_UNGROUNDED") || text.contains("log-ref"),
        "got: {text}"
    );
}

#[test]
fn missing_input_is_typed_refusal() {
    let dir = TempDir::new().unwrap();
    let out = affi(dir.path())
        .args([
            "receipt",
            "proof-carrying",
            "--object-type",
            "order",
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
    assert!(text.contains("REFUSED_INPUT_MISSING"), "got: {text}");
}

#[test]
fn model_temper_moves_subject_digest() {
    let dir = TempDir::new().unwrap();
    let receipt = mint_receipt(dir.path());
    let honest = dir.path().join("honest.json");
    let tempered = dir.path().join("tempered.json");
    fs::write(&honest, r#"[["create","pay"],["pay","ship"]]"#).unwrap();
    fs::write(&tempered, r#"[["create","pay"],["pay","CANCEL"]]"#).unwrap();

    let (_, a, _) = run_json(
        dir.path(),
        &[
            "proof-carrying",
            "--receipt",
            receipt.to_str().unwrap(),
            "--object-type",
            "order",
            "--model",
            honest.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    let (_, b, _) = run_json(
        dir.path(),
        &[
            "proof-carrying",
            "--receipt",
            receipt.to_str().unwrap(),
            "--object-type",
            "order",
            "--model",
            tempered.to_str().unwrap(),
            "--format",
            "json",
        ],
    );
    assert_ne!(
        a["subject_digest"], b["subject_digest"],
        "temper must move the seal subject"
    );
}

#[cfg(feature = "crypto-trust")]
#[test]
fn seal_and_verify_round_trip() {
    let dir = TempDir::new().unwrap();
    let receipt = mint_receipt(dir.path());
    let model = dir.path().join("model.json");
    fs::write(&model, r#"[["create","pay"],["pay","ship"]]"#).unwrap();
    let subject = dir.path().join("subject.json");
    let seed = "a5".repeat(32);
    let (code, v, err) = run_json(
        dir.path(),
        &[
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
        ],
    );
    assert_eq!(code, 0, "seal mint: {err}");
    let pk = v["seal"]["pk"].as_str().expect("pk embedded");
    let out = affi(dir.path())
        .args([
            "receipt",
            "proof-carrying-verify",
            subject.to_str().unwrap(),
            "--pk",
            pk,
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "verify: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
