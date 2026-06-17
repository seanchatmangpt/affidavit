//! End-to-end SBOM supply-chain provenance demo (Fortune-5 architecture).
//!
//! Loads a CycloneDX SBOM + an advisories/VEX file and runs the full pipeline:
//! ingest → OCEL events → NTIA certification → multi-framework compliance →
//! vulnerability + VEX correlation → blast-radius → provenance attestation.
//!
//! Run: `cargo run --example sbom_supply_chain_demo`

use affidavit::ocel::SeqCounter;
use affidavit::sbom::{parse_sbom_json, Sbom};
use affidavit::{sbom_compliance, sbom_ocel, sbom_supply_chain, sbom_vulnerability};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 SBOM Supply-Chain Provenance Demo");
    println!("════════════════════════════════════\n");

    // ---- 1. Ingest -------------------------------------------------------
    let cdx = fs::read_to_string("fixtures/sbom/cyclonedx-sample.json")?;
    let sbom: Sbom = parse_sbom_json(&cdx)?;
    println!(
        "📦 Ingested {} ({} components, {} dependency edges)",
        sbom.format.tag(),
        sbom.components.len(),
        sbom.dependencies.len()
    );
    println!("   content address: {}\n", &sbom.content_address().0[..16]);

    // ---- 2. OCEL event stream -------------------------------------------
    let mut counter = SeqCounter::new();
    let events = sbom_ocel::sbom_to_ocel_events(&sbom, &mut counter)?;
    println!("🧾 Generated {} OCEL events:", events.len());
    for ev in events.iter().take(4) {
        println!("   seq {} · {}", ev.event.seq, ev.sbom_event_type);
    }
    if events.len() > 4 {
        println!("   … and {} more", events.len() - 4);
    }
    println!();

    // ---- 3. NTIA minimum elements ---------------------------------------
    let ntia = sbom.ntia_minimum_elements();
    println!(
        "✅ NTIA minimum elements: {}",
        if ntia.is_conformant() {
            "CONFORMANT".to_string()
        } else {
            format!("missing {:?}", ntia.missing())
        }
    );

    // ---- 4. Compliance frameworks ---------------------------------------
    println!("\n🏛  Compliance assessment:");
    for r in sbom_compliance::assess_all(&sbom)? {
        let level = r
            .level
            .as_deref()
            .map(|l| format!(" [{l}]"))
            .unwrap_or_default();
        println!(
            "   {} {}{} — score {:.2}",
            if r.passed { "PASS" } else { "FAIL" },
            r.framework,
            level,
            r.score()
        );
    }

    // ---- 5. Vulnerability + VEX correlation -----------------------------
    let adv = fs::read_to_string("fixtures/sbom/advisories-sample.json")?;
    let doc: serde_json::Value = serde_json::from_str(&adv)?;
    let vulns: Vec<sbom_vulnerability::Vulnerability> =
        serde_json::from_value(doc["vulnerabilities"].clone())?;
    let vex: Vec<sbom_vulnerability::VexStatement> = serde_json::from_value(doc["vex"].clone())?;
    let report = sbom_vulnerability::build_report(&sbom, &vulns, &vex);
    println!(
        "\n🛡  Vulnerability scan: {} matches, {} exploitable after VEX, max severity {}",
        report.total_matches,
        report.exploitable_after_vex,
        report.max_severity.tag()
    );

    // ---- 6. Blast radius -------------------------------------------------
    let graph = sbom_supply_chain::DependencyGraph::from_sbom(&sbom);
    let target = "pkg:cargo/log@0.4.20";
    let radius = sbom_supply_chain::blast_radius(&graph, target)?;
    println!(
        "\n💥 Blast radius of {target}: {} transitively impacted -> {:?}",
        radius.transitively_impacted, radius.impacted
    );

    // ---- 7. Provenance attestation --------------------------------------
    let attestation = sbom_supply_chain::attest_provenance(&sbom, Some("receipt://demo"));
    println!(
        "\n📜 Provenance attestation: builder={:?}, supplier={:?}, edges={}",
        attestation.builder, attestation.attested_supplier, attestation.dependency_edges
    );

    println!("\n✅ Demo complete.");
    Ok(())
}
