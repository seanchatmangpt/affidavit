//! Lane: CLI value court for the five `receipt sbom-*` verbs (v26.10.16).
//!
//! Owner: tests/cli_value_sbom.rs only. Real built binary, real in-harness
//! fixtures (Chicago: no mocks). Every assertion lands on actual process
//! output or on-disk chain state — a court that cannot fail is not a court.
//!
//! Value assertions under test:
//! 1. `sbom-attest`       — real SLSA provenance over a CycloneDX fixture
//!    with bom-refs: attested dependency edge count equals the fixture's 4
//!    declared edges; primary component and BLAKE3 content address are read
//!    structurally; the event lands in the working chain.
//! 2. `sbom-scan`         — real component/VEX counts: 4 components, 2
//!    advisory matches (purl + cpe coordinates), one suppressed by a VEX
//!    not_affected statement -> exactly 1 exploitable after VEX, max
//!    severity critical.
//! 3. `sbom-blast-radius` — real direct+transitive dependents walk: zlib has
//!    2 direct dependents (left-pad, openssl) and 1 further transitive
//!    dependent (the root app); the impacted list names all three, sorted.
//! 4. `sbom-compliance`   — real per-framework PASS/FAIL with scores; the
//!    complete fixture passes NTIA at score 1.00; stripping the document
//!    author flips NTIA to FAIL with a nonzero failed list.
//! 5. `sbom-ntia`         — CONFORMANT on the complete fixture; stripping
//!    the author yields NON-CONFORMANT whose missing-field list is exactly
//!    ["author"]; adding it back restores CONFORMANT (the value flip).
//!
//! Negatives, typed (never a bare exit 1). SPDX document whose packages
//! carry no SPDXIDs (no bom-refs): blast-radius refuses with the typed
//! unknown-component execution error (exit 208, structured "code":1504
//! diagnostic on stderr). Missing SBOM file: typed IO failure diagnostic,
//! exit 208.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// POSIX exit status is 8-bit: stable code 1504 (CliExecutionError) is
/// encoded mod 256; the stderr JSON diagnostic carries the full value.
const EXIT_CLI_EXECUTION_ERROR: i32 = 1504 % 256; // 208

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "affi-cli-value-sbom-{tag}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&d).expect("create scratch dir");
    d
}

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_affi"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn affi binary");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A real CycloneDX 1.5 document with bom-refs, 4 dependency edges, supplier
/// on every component, purl/cpe identifiers, author and timestamp.
/// `with_author=false` strips the metadata authors block — the NTIA flip lever.
fn cyclonedx_fixture(with_author: bool) -> String {
    let mut doc = serde_json::json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": "urn:uuid:c0ffe000-0000-4000-8000-000000000001",
        "version": 1,
        "metadata": {
            "timestamp": "2026-10-17T00:00:00Z",
            "tools": [{ "vendor": "affidavit", "name": "affi", "version": "26.10.16" }],
            "component": {
                "bom-ref": "pkg:app/court-app@1.0.0",
                "type": "application",
                "name": "court-app",
                "version": "1.0.0",
                "supplier": { "name": "Acme Corp" }
            }
        },
        "components": [
            {
                "bom-ref": "pkg:app/court-app@1.0.0",
                "type": "application",
                "name": "court-app",
                "version": "1.0.0",
                "purl": "pkg:app/court-app@1.0.0",
                "supplier": { "name": "Acme Corp" }
            },
            {
                "bom-ref": "pkg:npm/left-pad@1.3.0",
                "type": "library",
                "name": "left-pad",
                "version": "1.3.0",
                "purl": "pkg:npm/left-pad@1.3.0",
                "supplier": { "name": "Acme Corp" }
            },
            {
                "bom-ref": "pkg:generic/openssl@3.0.0",
                "type": "library",
                "name": "openssl",
                "version": "3.0.0",
                "purl": "pkg:generic/openssl@3.0.0",
                "cpe": "cpe:2.3:a:openssl:openssl:3.0.0:*:*:*:*:*:*:*",
                "supplier": { "name": "Acme Corp" }
            },
            {
                "bom-ref": "pkg:generic/zlib@1.2.13",
                "type": "library",
                "name": "zlib",
                "version": "1.2.13",
                "purl": "pkg:generic/zlib@1.2.13",
                "supplier": { "name": "Acme Corp" }
            }
        ],
        "dependencies": [
            {
                "ref": "pkg:app/court-app@1.0.0",
                "dependsOn": ["pkg:npm/left-pad@1.3.0", "pkg:generic/openssl@3.0.0"]
            },
            { "ref": "pkg:npm/left-pad@1.3.0", "dependsOn": ["pkg:generic/zlib@1.2.13"] },
            { "ref": "pkg:generic/openssl@3.0.0", "dependsOn": ["pkg:generic/zlib@1.2.13"] }
        ]
    });
    if with_author {
        doc["metadata"]["authors"] = serde_json::json!([{ "name": "Court Build Bot" }]);
    }
    serde_json::to_string_pretty(&doc).expect("fixture serializes")
}

fn write_fixture(tag: &str, with_author: bool) -> (PathBuf, PathBuf) {
    let dir = scratch(tag);
    let sbom = dir.join("sbom.json");
    fs::write(&sbom, cyclonedx_fixture(with_author)).expect("write sbom fixture");
    (dir, sbom)
}

fn sbom_arg(sbom: &Path) -> String {
    sbom.to_str().expect("utf8 path").to_string()
}

// ---------------------------------------------------------------------------
// 1. sbom-attest — real provenance over real edges
// ---------------------------------------------------------------------------

#[test]
fn sbom_attest_attests_the_real_dependency_edges() {
    let (dir, sbom) = write_fixture("attest", true);
    let path = sbom_arg(&sbom);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-attest",
            "--sbom-path",
            &path,
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "sbom-attest must succeed on a well-formed fixture; stderr={stderr}"
    );
    assert!(!stdout.trim().is_empty(), "vacuity guard: empty stdout");

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("attest stdout is JSON: {e}"));
    let att = &parsed["attestation"];
    assert_eq!(
        att["dependency_edges"].as_u64(),
        Some(4),
        "attestation must carry the fixture's real edge count; got {att}"
    );
    assert_eq!(
        att["primary_component"].as_str(),
        Some("pkg:app/court-app@1.0.0"),
        "primary component must be read structurally from the fixture"
    );
    assert!(
        att["sbom_address"].as_str().is_some_and(|s| s.len() == 64),
        "sbom_address must be a BLAKE3 hex content address; got {att}"
    );
    assert!(
        parsed["event_seq"].is_u64(),
        "the attestation event must be emitted into the working chain; got {parsed}"
    );

    // The event must actually be on disk in the working chain, not just claimed.
    let chain = fs::read_to_string(dir.join(".affi/working.json"))
        .expect("working chain must exist after sbom-attest");
    assert!(
        chain.contains("sbom:attest"),
        "working chain must carry the sbom:attest event; got {chain}"
    );
}

// ---------------------------------------------------------------------------
// 2. sbom-scan — real component / match / VEX counts
// ---------------------------------------------------------------------------

#[test]
fn sbom_scan_reports_real_component_match_and_vex_counts() {
    let (dir, sbom) = write_fixture("scan", true);
    let advisories = dir.join("advisories.json");
    fs::write(
        &advisories,
        serde_json::json!({
            "vulnerabilities": [
                {
                    "id": "CVE-2026-0001",
                    "cvss": { "base_score": 9.8, "vector": null },
                    "affected_purls": ["pkg:npm/left-pad@1.3.0"],
                    "affected_cpes": [],
                    "fixed_versions": []
                },
                {
                    "id": "CVE-2026-0002",
                    "cvss": { "base_score": 7.5, "vector": null },
                    "affected_purls": [],
                    "affected_cpes": ["cpe:2.3:a:openssl:openssl:3.0.0:*:*:*:*:*:*:*"],
                    "fixed_versions": []
                }
            ],
            "vex": [
                {
                    "vuln_id": "CVE-2026-0001",
                    "component_bom_ref": "pkg:npm/left-pad@1.3.0",
                    "status": "NotAffected",
                    "justification": "vulnerable_code_not_in_execute_path"
                }
            ]
        })
        .to_string(),
    )
    .expect("write advisories fixture");

    let path = sbom_arg(&sbom);
    let adv = sbom_arg(&advisories);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-scan",
            "--sbom-path",
            &path,
            "--advisories-path",
            &adv,
        ],
    );
    assert_eq!(code, 0, "sbom-scan must succeed; stderr={stderr}");
    // Human receipt carries the real counts: 4 components, 2 matches,
    // exactly 1 exploitable after the VEX suppression, max severity critical.
    assert!(
        stdout.contains(
            "sbom-scan: 4 components, 2 matches (1 exploitable after VEX), max severity critical"
        ),
        "scan receipt must carry the real counts; got {stdout:?}"
    );

    // JSON plane: assert the structured fields, not just the rendered line.
    let (code_j, stdout_j, stderr_j) = run(
        &dir,
        &[
            "receipt",
            "sbom-scan",
            "--sbom-path",
            &path,
            "--advisories-path",
            &adv,
            "--format",
            "json",
        ],
    );
    assert_eq!(code_j, 0, "sbom-scan json; stderr={stderr_j}");
    let report: serde_json::Value =
        serde_json::from_str(&stdout_j).unwrap_or_else(|e| panic!("scan stdout is JSON: {e}"));
    assert_eq!(report["total_components"].as_u64(), Some(4));
    assert_eq!(report["total_matches"].as_u64(), Some(2));
    assert_eq!(report["exploitable_after_vex"].as_u64(), Some(1));
    assert_eq!(
        report["by_severity"]["critical"].as_u64(),
        Some(1),
        "exactly one critical match (left-pad purl); got {report}"
    );
    assert_eq!(
        report["by_severity"]["high"].as_u64(),
        Some(1),
        "exactly one high match (openssl cpe); got {report}"
    );
}

// ---------------------------------------------------------------------------
// 3. sbom-blast-radius — real direct+transitive walk
// ---------------------------------------------------------------------------

#[test]
fn sbom_blast_radius_walks_direct_and_transitive_dependents() {
    let (dir, sbom) = write_fixture("blast", true);
    let path = sbom_arg(&sbom);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-blast-radius",
            "--sbom-path",
            &path,
            "--component",
            "pkg:generic/zlib@1.2.13",
            "--format",
            "json",
        ],
    );
    assert_eq!(
        code, 0,
        "blast-radius must succeed on a known bom-ref; stderr={stderr}"
    );
    assert!(!stdout.trim().is_empty(), "vacuity guard: empty stdout");

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("blast stdout is JSON: {e}"));
    assert_eq!(
        parsed["component"].as_str(),
        Some("pkg:generic/zlib@1.2.13")
    );
    assert_eq!(
        parsed["directly_impacted"].as_u64(),
        Some(2),
        "zlib is directly depended on by left-pad and openssl; got {parsed}"
    );
    assert_eq!(
        parsed["transitively_impacted"].as_u64(),
        Some(3),
        "total impacted (direct + transitive) = left-pad, openssl, root app; got {parsed}"
    );
    assert_eq!(
        parsed["impacted"].as_array().map(|a| a.as_slice()),
        Some(
            &[
                serde_json::json!("pkg:app/court-app@1.0.0"),
                serde_json::json!("pkg:generic/openssl@3.0.0"),
                serde_json::json!("pkg:npm/left-pad@1.3.0"),
            ][..]
        ),
        "impacted list must name every dependent, sorted; got {parsed}"
    );
}

// ---------------------------------------------------------------------------
// 4. sbom-compliance — real per-framework PASS/FAIL with scores
// ---------------------------------------------------------------------------

#[test]
fn sbom_compliance_scores_pass_on_complete_fixture() {
    let (dir, sbom) = write_fixture("compliance-good", true);
    let path = sbom_arg(&sbom);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-compliance",
            "--sbom-path",
            &path,
            "--framework",
            "ntia",
        ],
    );
    assert_eq!(code, 0, "compliance must succeed; stderr={stderr}");
    assert!(
        stdout.contains("PASS NTIA Minimum Elements"),
        "complete fixture must PASS NTIA; got {stdout:?}"
    );
    assert!(
        stdout.contains("score 1.00 ("),
        "all NTIA requirements satisfied -> score 1.00; got {stdout:?}"
    );
}

#[test]
fn sbom_compliance_fails_and_rescores_when_author_is_stripped() {
    let (dir, sbom) = write_fixture("compliance-bad", false);
    let path = sbom_arg(&sbom);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-compliance",
            "--sbom-path",
            &path,
            "--framework",
            "ntia",
        ],
    );
    assert_eq!(code, 0, "compliance still assesses; stderr={stderr}");
    assert!(
        stdout.contains("FAIL NTIA Minimum Elements"),
        "author-stripped fixture must FAIL NTIA; got {stdout:?}"
    );
    assert!(
        !stdout.contains("score 1.00"),
        "a failed element must move the score off 1.00; got {stdout:?}"
    );
    assert!(
        stdout.contains("1 failed"),
        "exactly one requirement (author) failed; got {stdout:?}"
    );
}

// ---------------------------------------------------------------------------
// 5. sbom-ntia — CONFORMANT flip on author presence (the value assertion)
// ---------------------------------------------------------------------------

#[test]
fn sbom_ntia_flips_conformant_on_author_presence() {
    // Complete fixture -> CONFORMANT, empty missing list.
    let (dir, sbom) = write_fixture("ntia-good", true);
    let path = sbom_arg(&sbom);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-ntia",
            "--sbom-path",
            &path,
            "--format",
            "json",
        ],
    );
    assert_eq!(code, 0, "ntia on complete fixture; stderr={stderr}");
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("ntia stdout is JSON: {e}"));
    assert_eq!(parsed["conformant"], serde_json::json!(true));
    assert_eq!(
        parsed["missing"].as_array().map(|a| a.as_slice()),
        Some(&[][..]),
        "complete fixture: no missing elements; got {parsed}"
    );
    assert!(
        stdout.contains("\"conformant\": true"),
        "json plane must carry the conformant flag; got {stdout:?}"
    );

    // Strip the author -> NON-CONFORMANT naming exactly the author element.
    let (dir2, sbom2) = write_fixture("ntia-bad", false);
    let path2 = sbom_arg(&sbom2);
    let (code2, stdout2, stderr2) = run(
        &dir2,
        &[
            "receipt",
            "sbom-ntia",
            "--sbom-path",
            &path2,
            "--format",
            "json",
        ],
    );
    assert_eq!(code2, 0, "ntia on stripped fixture; stderr={stderr2}");
    let parsed2: serde_json::Value =
        serde_json::from_str(&stdout2).unwrap_or_else(|e| panic!("ntia stdout is JSON: {e}"));
    assert_eq!(parsed2["conformant"], serde_json::json!(false));
    assert_eq!(
        parsed2["missing"].as_array().map(|a| a.as_slice()),
        Some(&[serde_json::json!("author")][..]),
        "stripping only the author must name exactly 'author'; got {parsed2}"
    );

    // Add the author back -> CONFORMANT again (the value flip closes).
    let (dir3, sbom3) = write_fixture("ntia-restored", true);
    let path3 = sbom_arg(&sbom3);
    let (code3, stdout3, stderr3) = run(
        &dir3,
        &[
            "receipt",
            "sbom-ntia",
            "--sbom-path",
            &path3,
            "--format",
            "json",
        ],
    );
    assert_eq!(code3, 0, "ntia on restored fixture; stderr={stderr3}");
    let parsed3: serde_json::Value =
        serde_json::from_str(&stdout3).unwrap_or_else(|e| panic!("ntia stdout is JSON: {e}"));
    assert_eq!(
        parsed3["conformant"],
        serde_json::json!(true),
        "restoring the author must restore CONFORMANT; got {parsed3}"
    );

    // Human plane names the missing element, not just a boolean.
    let (code4, stdout4, stderr4) = run(&dir2, &["receipt", "sbom-ntia", "--sbom-path", &path2]);
    assert_eq!(code4, 0, "human ntia; stderr={stderr4}");
    assert!(
        stdout4.contains("NON-CONFORMANT") && stdout4.contains("author"),
        "human receipt must name the missing author element; got {stdout4:?}"
    );
}

// ---------------------------------------------------------------------------
// 6-7. Negatives: typed refusals, never a bare 1
// ---------------------------------------------------------------------------

#[test]
fn spdx_without_bom_refs_refused_typed_by_blast_radius() {
    let dir = scratch("spdx-negative");
    let sbom = dir.join("spdx.json");
    fs::write(
        &sbom,
        r#"{
        "spdxVersion": "SPDX-2.3",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "no-refs-doc",
        "documentNamespace": "https://example.invalid/spdx/no-refs",
        "creationInfo": { "created": "2026-10-17T00:00:00Z", "creators": ["Person: Court Bot"] },
        "packages": [
            { "name": "mystery-a", "versionInfo": "1.0.0" },
            { "name": "mystery-b", "versionInfo": "2.0.0" }
        ]
    }"#,
    )
    .expect("write spdx fixture");
    let path = sbom_arg(&sbom);
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-blast-radius",
            "--sbom-path",
            &path,
            "--component",
            "mystery-a",
        ],
    );
    assert_ne!(
        code, 0,
        "blast-radius on an SPDX doc whose packages carry no SPDXIDs must refuse"
    );
    assert_eq!(
        code, EXIT_CLI_EXECUTION_ERROR,
        "refusal must carry the stable CliExecutionError code (1504 mod 256); got {code}"
    );
    let diag = format!("{stdout}{stderr}");
    assert!(
        diag.contains("\"code\":1504"),
        "refusal must be a structured 1504-range diagnostic; got {diag}"
    );
    assert!(
        diag.contains("blast-radius") && diag.contains("unknown component"),
        "typed message must name the verb and the unknown component; got {diag}"
    );
}

#[test]
fn missing_sbom_file_refused_typed() {
    let dir = scratch("missing-file");
    let (code, stdout, stderr) = run(
        &dir,
        &[
            "receipt",
            "sbom-ntia",
            "--sbom-path",
            "/nonexistent/sbom.json",
        ],
    );
    assert_ne!(code, 0, "missing SBOM file must refuse");
    assert_eq!(
        code, EXIT_CLI_EXECUTION_ERROR,
        "refusal must carry the stable CliExecutionError code; got {code}"
    );
    let diag = format!("{stdout}{stderr}");
    assert!(
        diag.contains("\"code\":1504"),
        "refusal must be a structured 1504-range diagnostic; got {diag}"
    );
    assert!(
        diag.contains("IO failure"),
        "typed message must name the IO failure; got {diag}"
    );
}

// Wrap-up lane additions: value-matrix markers for verbs tested here but
// unmarked by the first wave. Verified: this suite passes (exit 0).
// value-matrix: (receipt, sbom-attest)
// value-matrix: (receipt, sbom-blast-radius)
// value-matrix: (receipt, sbom-compliance)
// value-matrix: (receipt, sbom-ntia)
// value-matrix: (receipt, sbom-scan)
