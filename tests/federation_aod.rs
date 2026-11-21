//! Court: `affi standing certify-aod` — AOD Live Gate format → PARTIAL_ALIVE.
//!
//! Law (AFFI-SPEC-26.11.19 federation adapter):
//! 1. Real `map_live` path admits a fixture-sealed `.aod` (in-memory lease).
//! 2. Sealed standing is PARTIAL_ALIVE — never ALIVE from this court.
//! 3. Ontology-hash witness is carried; durable AFAL burn does not occur.
//! 4. Tampered / non-AOD bytes refuse with typed REFUSED_AOD_FEDERATION_*.
//! 5. `standing verify` re-accepts the sealed PARTIAL_ALIVE receipt.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn affi(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("affi").expect("affi binary builds");
    cmd.current_dir(dir.path());
    cmd
}

fn write(dir: &TempDir, name: &str, contents: &str) {
    fs::write(dir.path().join(name), contents).expect("fixture written");
}

fn admitted_source(dir: &TempDir) -> String {
    affi(dir)
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
    affi(dir)
        .args(["receipt", "assemble", "--out", "source.json"])
        .assert()
        .success();
    "source.json".to_string()
}

fn seal_fixture_aod(dir: &TempDir) -> String {
    let page: Vec<u8> = (0..16).map(|i| (i + 1) as u8).collect();
    let page_path = dir.path().join("page.bin");
    fs::write(&page_path, &page).unwrap();

    let ontology = affidavit_core::FIXTURE_ONTOLOGY_HASH_HEX;
    let lease = format!("{:0>64}", "bb");
    affi(dir)
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
            page_path.to_str().unwrap(),
            "--out",
            "artifact.aod",
        ])
        .assert()
        .success();
    "artifact.aod".to_string()
}

fn subject_json(dir: &TempDir) -> String {
    write(
        dir,
        "subject.json",
        r#"{
  "subject": "seanchatmangpt/affidavit",
  "base": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "tree": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "candidate": "cccccccccccccccccccccccccccccccccccccccc"
}"#,
    );
    "subject.json".to_string()
}

#[test]
fn certify_aod_seals_partial_alive_and_verify_reaccepts() {
    let dir = TempDir::new().unwrap();
    let source = admitted_source(&dir);
    let aod = seal_fixture_aod(&dir);
    let subject = subject_json(&dir);

    affi(&dir)
        .args([
            "standing",
            "certify-aod",
            "--receipt",
            &source,
            "--aod",
            &aod,
            "--subject",
            &subject,
            "--scope",
            "repo:seanchatmangpt/affidavit",
            "--out",
            "standing-aod.json",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("PARTIAL_ALIVE"))
        .stdout(predicate::str::contains(
            "affidavit/aod-federation-adapter/v1",
        ))
        .stdout(predicate::str::contains("durable_lease_burn"));

    let sealed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join("standing-aod.json")).unwrap())
            .unwrap();
    assert_eq!(sealed["profile"], "affidavit/standing/v2");
    assert_eq!(sealed["standing"], "PARTIAL_ALIVE");
    assert!(
        sealed.get("aod_federation").is_none(),
        "sealed artifact stays profile-pure"
    );

    // Durable AFAL must not have been created by federation certify.
    assert!(
        !dir.path().join(".affi/aod-leases").exists(),
        "certify-aod must not durable-burn leases"
    );

    affi(&dir)
        .args(["standing", "verify", "--receipt", "standing-aod.json"])
        .assert()
        .success()
        .stderr(predicate::str::contains("ACCEPT"));
}

#[test]
fn certify_aod_refuses_non_aod_bytes() {
    let dir = TempDir::new().unwrap();
    let source = admitted_source(&dir);
    let subject = subject_json(&dir);
    write(&dir, "bogus.aod", "not-an-aod");

    affi(&dir)
        .args([
            "standing",
            "certify-aod",
            "--receipt",
            &source,
            "--aod",
            "bogus.aod",
            "--subject",
            &subject,
            "--scope",
            "repo:affidavit",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("REFUSED_AOD_FEDERATION_NOT_AOD"));
}

#[test]
fn certify_aod_refuses_missing_aod_path() {
    let dir = TempDir::new().unwrap();
    let source = admitted_source(&dir);
    let subject = subject_json(&dir);

    affi(&dir)
        .args([
            "standing",
            "certify-aod",
            "--receipt",
            &source,
            "--aod",
            "missing.aod",
            "--subject",
            &subject,
            "--scope",
            "repo:affidavit",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .code(4)
        .stdout(predicate::str::contains("REFUSED_AOD_FEDERATION_IO"));
}

#[test]
fn library_alive_ceiling_is_typed() {
    use affidavit::federation_aod::{refuse_alive_ceiling, AodFederationRefusal};
    use affidavit::standing::Standing;

    let err = refuse_alive_ceiling(Standing::Alive).unwrap_err();
    assert_eq!(err, AodFederationRefusal::AliveCeiling);
    assert!(err
        .to_string()
        .contains("REFUSED_AOD_FEDERATION_ALIVE_CEILING"));
}

#[test]
fn library_live_format_uses_real_map_live() {
    use affidavit::federation_aod::{
        certify_aod_live_format, fixture_compiled, fixture_ontology_hex,
    };
    use affidavit_core::{seal_aod, AodBufferSpec, SealParams, FIXTURE_ONTOLOGY_HASH};

    let page = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let raw = seal_aod(
        &[AodBufferSpec { bytes: &page }],
        &SealParams {
            ontology_hash: &FIXTURE_ONTOLOGY_HASH,
            row_count: 1,
            lease_epoch: 1,
            lease_nullifier: &[0x22; 32],
            folded_receipt: &[0u8; 32],
        },
    )
    .unwrap();

    let witness = certify_aod_live_format(&raw, &fixture_compiled()).unwrap();
    assert_eq!(witness.ontology_hash_hex, fixture_ontology_hex());
    assert_eq!(
        witness.standing_ceiling,
        affidavit::standing::Standing::PartialAlive
    );
}

#[test]
fn wiring_removal_breaks_this_court() {
    // Anti-vacuity: the CLI surface must exist.
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/federation.rs"));
    assert!(
        src.contains("pub fn aod_standing_certify"),
        "federation adapter entrypoint must remain"
    );
    let verb = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/verbs/standing_certify_aod.rs");
    assert!(
        verb.exists(),
        "standing certify-aod verb projection must exist"
    );
}
