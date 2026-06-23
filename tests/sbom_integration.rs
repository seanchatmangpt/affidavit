//! End-to-end SBOM supply-chain integration tests.
//!
//! Exercises the full vertical slice against the committed fixtures: multi-format
//! ingest (CycloneDX + SPDX), OCEL event generation, NTIA certification,
//! compliance frameworks, vulnerability/VEX correlation, and supply-chain graph
//! analytics. These tests use only the canonical model + modules (no CLI), so
//! they are deterministic and dependency-free.

use affidavit::ocel::SeqCounter;
use affidavit::sbom::{parse_sbom_json, SbomFormat};
use affidavit::{sbom_compliance, sbom_ocel, sbom_supply_chain, sbom_vulnerability};
use std::fs;

fn cyclonedx() -> affidavit::sbom::Sbom {
    let raw = fs::read_to_string("fixtures/sbom/cyclonedx-sample.json").expect("fixture present");
    parse_sbom_json(&raw).expect("cyclonedx parses")
}

fn spdx() -> affidavit::sbom::Sbom {
    let raw = fs::read_to_string("fixtures/sbom/spdx-sample.json").expect("fixture present");
    parse_sbom_json(&raw).expect("spdx parses")
}

#[test]
fn cyclonedx_fixture_ingests_with_expected_shape() {
    let sbom = cyclonedx();
    assert_eq!(sbom.format, SbomFormat::CycloneDx16);
    assert_eq!(sbom.components.len(), 6);
    assert!(!sbom.dependencies.is_empty());
    assert!(sbom.component("pkg:cargo/serde@1.0.197").is_some());
}

#[test]
fn spdx_fixture_ingests_and_resolves_relationships() {
    let sbom = spdx();
    assert_eq!(sbom.format, SbomFormat::Spdx23);
    assert_eq!(sbom.components.len(), 3);
    assert!(!sbom.dependencies.is_empty());
}

#[test]
fn both_formats_are_ntia_conformant() {
    assert!(
        cyclonedx().ntia_minimum_elements().is_conformant(),
        "cyclonedx fixture should be NTIA-conformant"
    );
    assert!(
        spdx().ntia_minimum_elements().is_conformant(),
        "spdx fixture should be NTIA-conformant"
    );
}

#[test]
fn ocel_event_stream_is_deterministic_and_nonempty() {
    let sbom = cyclonedx();
    let mut c1 = SeqCounter::new();
    let mut c2 = SeqCounter::new();
    let a = sbom_ocel::sbom_to_ocel_events(&sbom, &mut c1).expect("events");
    let b = sbom_ocel::sbom_to_ocel_events(&sbom, &mut c2).expect("events");
    assert!(!a.is_empty());
    assert_eq!(a.len(), b.len(), "event count is deterministic");
    // Commitments must match across runs (content-addressed determinism).
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.event.payload_commitment, y.event.payload_commitment);
        assert_eq!(x.sbom_event_type, y.sbom_event_type);
    }
}

#[test]
fn compliance_assessment_runs_all_frameworks() {
    let sbom = cyclonedx();
    let results = sbom_compliance::assess_all(&sbom).expect("assess");
    assert!(results.len() >= 8, "all major frameworks assessed");
    // NTIA must be present and passing for this complete fixture.
    let ntia = results
        .iter()
        .find(|r| r.framework.to_ascii_lowercase().contains("ntia"))
        .expect("ntia result present");
    assert!(ntia.passed, "complete fixture passes NTIA");
}

#[test]
fn vulnerability_scan_matches_and_vex_suppresses() {
    let sbom = cyclonedx();
    let adv = fs::read_to_string("fixtures/sbom/advisories-sample.json").expect("advisories");
    let doc: serde_json::Value = serde_json::from_str(&adv).unwrap();
    let vulns: Vec<sbom_vulnerability::Vulnerability> =
        serde_json::from_value(doc["vulnerabilities"].clone()).unwrap();
    let vex: Vec<sbom_vulnerability::VexStatement> =
        serde_json::from_value(doc["vex"].clone()).unwrap();

    // Without VEX: both advisories match (openssl + log).
    let raw_matches = sbom_vulnerability::match_vulnerabilities(&sbom, &vulns);
    assert!(raw_matches.len() >= 2, "openssl + log advisories match");

    // The RUSTSEC log advisory is marked NotAffected via VEX → suppressed.
    let report = sbom_vulnerability::build_report(&sbom, &vulns, &vex);
    assert!(
        report.exploitable_after_vex < report.total_matches,
        "VEX NotAffected must reduce exploitable count"
    );
    assert_eq!(report.max_severity, sbom_vulnerability::Severity::High);
}

#[test]
fn risk_propagates_along_the_dependency_chain() {
    // openssl has a High CVE; the root application depends on openssl, so the
    // app must inherit the risk through propagation.
    let sbom = cyclonedx();
    let adv = fs::read_to_string("fixtures/sbom/advisories-sample.json").expect("advisories");
    let doc: serde_json::Value = serde_json::from_str(&adv).unwrap();
    let vulns: Vec<sbom_vulnerability::Vulnerability> =
        serde_json::from_value(doc["vulnerabilities"].clone()).unwrap();
    let matches = sbom_vulnerability::match_vulnerabilities(&sbom, &vulns);
    let propagated = sbom_vulnerability::propagate_risk(&sbom, &matches);
    assert!(
        propagated.iter().any(
            |(bom_ref, vuln, _)| bom_ref == "pkg:cargo/affidavit@26.6.22"
                && vuln == "CVE-2023-0464"
        ),
        "root application inherits the openssl CVE through the dependency edge"
    );
}

#[test]
fn blast_radius_identifies_transitive_dependents() {
    let sbom = cyclonedx();
    let graph = sbom_supply_chain::DependencyGraph::from_sbom(&sbom);
    // `log` is depended on by serde and openssl, which the app depends on.
    let radius = sbom_supply_chain::blast_radius(&graph, "pkg:cargo/log@0.4.20").expect("radius");
    assert!(
        radius.transitively_impacted >= 2,
        "log's compromise impacts multiple upstream components"
    );
}

#[test]
fn supplier_concentration_is_reported() {
    let sbom = cyclonedx();
    let conc = sbom_supply_chain::supplier_concentration(&sbom);
    assert!(!conc.is_empty());
    // serde-rs supplies two components (serde + serde_json).
    assert!(
        conc.iter()
            .any(|c| c.supplier == "serde-rs" && c.component_count >= 2),
        "serde-rs supplies multiple components"
    );
}

#[test]
fn provenance_attestation_binds_address_and_receipt() {
    let sbom = cyclonedx();
    let attestation = sbom_supply_chain::attest_provenance(&sbom, Some("receipt://abc"));
    assert!(!attestation.sbom_address.is_empty());
    assert_eq!(
        attestation.generated_from_receipt.as_deref(),
        Some("receipt://abc")
    );
    assert!(attestation.dependency_edges > 0);
}

#[test]
fn content_address_is_format_stable_within_a_format() {
    // Re-parsing the same document yields the same content address.
    let a = cyclonedx().content_address();
    let b = cyclonedx().content_address();
    assert_eq!(a, b);
}
