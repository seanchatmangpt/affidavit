//! Supply-chain COMPLIANCE gates over a canonical SBOM.
//!
//! This module certifies a canonical [`crate::sbom::Sbom`] against the major
//! regulatory and industry supply-chain frameworks. It is the *compliance* axis
//! of affidavit's supply-chain provenance layer: where [`crate::sbom`] normalizes
//! every source format into one shape, this module evaluates that shape against
//! the structural requirements of each framework.
//!
//! # Doctrine: certify, don't decide
//!
//! Every checker in this module reports **conformance against a standard's
//! structural requirements**. It never decides whether the declared data is
//! *truthful*. When a checker says NTIA is satisfied, it means "every required
//! field is structurally present", not "the supplier names are accurate". This is
//! the same discipline the verifier applies to receipts: we certify a witness
//! against a format standard, we do not adjudicate honesty.
//!
//! # Combinatorial maximalism
//!
//! Fortune-5 supply-chain governance spans many overlapping regimes. This module
//! covers each as its own checker, all returning a uniform [`ComplianceResult`]:
//!
//! - **NTIA** minimum elements (the 2021 baseline).
//! - **Executive Order 14028** (NTIA + SBOM tooling provenance + data completeness).
//! - **SLSA** L1–L4 (structural provenance/attestation proxies).
//! - **in-toto** attestation completeness (functionaries, materials, products).
//! - **CISA** SBOM minimum + recommended fields (identifier/license coverage).
//! - **C-SCRM / NIST SP 800-161** supplier-coverage gates.
//! - **ISO/IEC 27001** and **SOC 2** evidence-presence proxies.
//! - **VEX** readiness (stable identifier coverage to anchor VEX statements).
//!
//! # Structural proxies
//!
//! Several frameworks (SLSA levels, ISO 27001, SOC 2, in-toto) are process and
//! control regimes that an SBOM alone cannot fully certify. Where a checker maps
//! such a regime onto SBOM structure, it documents the mapping as a *structural
//! proxy*: a presence-of-evidence gate that is necessary-but-not-sufficient for
//! the underlying control. Each result records this caveat in its `notes`.
//!
//! # Determinism
//!
//! All checkers are total functions of the SBOM value: same SBOM in, same
//! [`ComplianceResult`] out. No wall-clock, no IO, no randomness. The orchestrator
//! [`assess_all`] runs every framework in a fixed, documented order.

use crate::sbom::Sbom;
use serde::{Deserialize, Serialize};

/// Error raised while running a compliance assessment.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ComplianceError {
    /// The SBOM carries no components, so there is nothing to certify against
    /// any component-scoped framework.
    #[error("empty sbom: no components to certify")]
    EmptySbom,
}

/// A supply-chain compliance framework this module can certify against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    /// NTIA "Minimum Elements For an SBOM" (2021).
    Ntia,
    /// Executive Order 14028 §4 SBOM expectations.
    ExecutiveOrder14028,
    /// Supply-chain Levels for Software Artifacts (SLSA).
    Slsa,
    /// in-toto attestation framework.
    InToto,
    /// CISA SBOM minimum + recommended fields guidance.
    Cisa,
    /// Cyber Supply Chain Risk Management — NIST SP 800-161.
    Cscrm,
    /// ISO/IEC 27001 information-security management (structural proxy).
    Iso27001,
    /// SOC 2 trust-services evidence presence (structural proxy).
    Soc2,
    /// Vulnerability Exploitability eXchange (VEX) readiness.
    Vex,
}

impl Framework {
    /// Stable lowercase tag used in result identifiers and reporting.
    pub fn tag(&self) -> &'static str {
        match self {
            Framework::Ntia => "ntia",
            Framework::ExecutiveOrder14028 => "eo-14028",
            Framework::Slsa => "slsa",
            Framework::InToto => "in-toto",
            Framework::Cisa => "cisa",
            Framework::Cscrm => "cscrm-800-161",
            Framework::Iso27001 => "iso-27001",
            Framework::Soc2 => "soc-2",
            Framework::Vex => "vex",
        }
    }

    /// Human-readable framework name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Framework::Ntia => "NTIA Minimum Elements",
            Framework::ExecutiveOrder14028 => "Executive Order 14028",
            Framework::Slsa => "SLSA",
            Framework::InToto => "in-toto",
            Framework::Cisa => "CISA SBOM",
            Framework::Cscrm => "C-SCRM (NIST SP 800-161)",
            Framework::Iso27001 => "ISO/IEC 27001",
            Framework::Soc2 => "SOC 2",
            Framework::Vex => "VEX Readiness",
        }
    }
}

/// The uniform result of certifying an SBOM against one framework.
///
/// A result lists which structural requirements were `satisfied` and which
/// `failed`, plus free-text `notes` carrying ratios and proxy caveats. The
/// `passed` flag is the framework's overall verdict, and [`ComplianceResult::score`]
/// gives the satisfied fraction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Framework display name (e.g. "NTIA Minimum Elements").
    pub framework: String,
    /// Optional achieved level / tier (e.g. SLSA "L2"), where the framework is tiered.
    pub level: Option<String>,
    /// Overall verdict: whether the SBOM conforms to the framework's gate.
    pub passed: bool,
    /// Names of structural requirements that were satisfied.
    pub satisfied: Vec<String>,
    /// Names of structural requirements that failed.
    pub failed: Vec<String>,
    /// Free-text notes: coverage ratios, structural-proxy caveats, rationale.
    pub notes: Vec<String>,
}

impl ComplianceResult {
    /// Construct a result with a framework name and no checks recorded yet.
    fn new(framework: &Framework) -> Self {
        ComplianceResult {
            framework: framework.display_name().to_string(),
            level: None,
            passed: false,
            satisfied: Vec::new(),
            failed: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Record a single structural requirement outcome under `name`.
    fn record(&mut self, name: impl Into<String>, ok: bool) {
        if ok {
            self.satisfied.push(name.into());
        } else {
            self.failed.push(name.into());
        }
    }

    /// Append a free-text note (ratio, caveat, rationale).
    fn note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// The satisfied fraction: `satisfied / (satisfied + failed)`.
    ///
    /// Returns `0.0` when no requirements were recorded, keeping the function
    /// total (no division by zero).
    pub fn score(&self) -> f64 {
        let total = self.satisfied.len() + self.failed.len();
        if total == 0 {
            0.0
        } else {
            self.satisfied.len() as f64 / total as f64
        }
    }

    /// Total number of recorded requirements (satisfied + failed).
    pub fn requirement_count(&self) -> usize {
        self.satisfied.len() + self.failed.len()
    }
}

/// A SLSA level, from L1 (provenance exists) to L4 (two-party/hermetic).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SlsaLevel {
    /// No level reached: no provenance exists in the document.
    L0,
    /// L1: provenance exists (document metadata + generating tools present).
    L1,
    /// L2: versioned/authenticated document (serial number) + every component hashed.
    L2,
    /// L3: hardened/attested source — all of L2 plus a supplier on every component.
    L3,
    /// L4: two-party review / hermetic completeness — all of L3 plus dependency edges.
    L4,
}

impl SlsaLevel {
    /// Stable tag (e.g. "L2") for reporting.
    pub fn tag(&self) -> &'static str {
        match self {
            SlsaLevel::L0 => "L0",
            SlsaLevel::L1 => "L1",
            SlsaLevel::L2 => "L2",
            SlsaLevel::L3 => "L3",
            SlsaLevel::L4 => "L4",
        }
    }

    /// Numeric rank (0–4) for comparison and ratio reporting.
    pub fn rank(&self) -> u8 {
        match self {
            SlsaLevel::L0 => 0,
            SlsaLevel::L1 => 1,
            SlsaLevel::L2 => 2,
            SlsaLevel::L3 => 3,
            SlsaLevel::L4 => 4,
        }
    }
}

// ---------------------------------------------------------------------------
// Coverage helpers (total functions over the component set).
// ---------------------------------------------------------------------------

/// Fraction of components for which `pred` holds. Returns `0.0` for an empty set.
fn coverage<F>(sbom: &Sbom, pred: F) -> f64
where
    F: Fn(&crate::sbom::Component) -> bool,
{
    let total = sbom.components.len();
    if total == 0 {
        return 0.0;
    }
    let hits = sbom.components.iter().filter(|c| pred(c)).count();
    hits as f64 / total as f64
}

/// Whether every component satisfies `pred` (vacuously false for empty SBOMs,
/// since an empty SBOM certifies nothing).
fn all_components<F>(sbom: &Sbom, pred: F) -> bool
where
    F: Fn(&crate::sbom::Component) -> bool,
{
    !sbom.components.is_empty() && sbom.components.iter().all(pred)
}

/// Whether the document carries a non-empty author string.
fn has_author(sbom: &Sbom) -> bool {
    sbom.metadata
        .author
        .as_ref()
        .is_some_and(|a| !a.trim().is_empty())
}

/// Whether the document carries generating tools (SBOM provenance).
fn has_tools(sbom: &Sbom) -> bool {
    !sbom.metadata.tools.is_empty()
}

/// Whether the document carries a positive timestamp.
fn has_timestamp(sbom: &Sbom) -> bool {
    sbom.metadata.timestamp > 0
}

/// Whether a component carries a stable external coordinate (PURL or CPE).
fn has_stable_id(c: &crate::sbom::Component) -> bool {
    c.purl.as_ref().is_some_and(|p| !p.trim().is_empty())
        || c.cpe.as_ref().is_some_and(|p| !p.trim().is_empty())
}

/// Format a ratio note like `purl coverage: 2/3 (66.7%)`.
fn ratio_note(label: &str, hits: usize, total: usize) -> String {
    let pct = if total == 0 {
        0.0
    } else {
        hits as f64 / total as f64 * 100.0
    };
    format!("{label}: {hits}/{total} ({pct:.1}%)")
}

// ---------------------------------------------------------------------------
// NTIA minimum elements.
// ---------------------------------------------------------------------------

/// Certify the SBOM against the NTIA minimum elements.
///
/// Wraps [`Sbom::ntia_minimum_elements`], unfolding each of the seven elements
/// into a satisfied/failed requirement. Passes iff all seven are present.
pub fn check_ntia(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let ntia = sbom.ntia_minimum_elements();
    let mut result = ComplianceResult::new(&Framework::Ntia);
    result.record("supplier_name", ntia.supplier_name);
    result.record("component_name", ntia.component_name);
    result.record("version", ntia.version);
    result.record("unique_identifiers", ntia.unique_identifiers);
    result.record("dependency_relationship", ntia.dependency_relationship);
    result.record("author", ntia.author);
    result.record("timestamp", ntia.timestamp);
    result.passed = ntia.is_conformant();
    if !result.passed {
        result.note(format!("missing elements: {:?}", ntia.missing()));
    }
    result.note("certifies presence of NTIA fields; does not decide truthfulness");
    Ok(result)
}

// ---------------------------------------------------------------------------
// Executive Order 14028.
// ---------------------------------------------------------------------------

/// Certify the SBOM against Executive Order 14028 §4 SBOM expectations.
///
/// Composes the NTIA baseline with two extra structural gates that EO 14028 (and
/// the implementing OMB / NIST guidance) emphasize: provenance of the SBOM itself
/// (generating tools recorded) and completeness of the document data fields
/// (author + timestamp present, primary component identified). Passes iff NTIA is
/// conformant AND every extra gate holds.
pub fn check_eo_14028(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let ntia = sbom.ntia_minimum_elements();
    let mut result = ComplianceResult::new(&Framework::ExecutiveOrder14028);

    let ntia_ok = ntia.is_conformant();
    result.record("ntia_minimum_elements", ntia_ok);
    if !ntia_ok {
        result.note(format!("NTIA gaps: {:?}", ntia.missing()));
    }

    let tooling = has_tools(sbom);
    result.record("sbom_tooling_provenance", tooling);

    let author = has_author(sbom);
    result.record("document_author", author);

    let timestamp = has_timestamp(sbom);
    result.record("document_timestamp", timestamp);

    let primary = sbom
        .metadata
        .primary_component
        .as_ref()
        .is_some_and(|p| !p.trim().is_empty());
    result.record("primary_component_identified", primary);

    result.passed = ntia_ok && tooling && author && timestamp && primary;
    result.note("EO 14028: NTIA baseline plus SBOM tooling provenance and data completeness");
    Ok(result)
}

// ---------------------------------------------------------------------------
// SLSA.
// ---------------------------------------------------------------------------

/// Assess the highest SLSA level the SBOM's structure supports.
///
/// Returns the achieved [`SlsaLevel`] and a [`ComplianceResult`] recording each
/// tier's requirement set. The structural proxies (documented as such in the
/// result notes) are:
///
/// - **L1** — provenance exists: document metadata carries generating tools.
/// - **L2** — versioned/authenticated doc: a serial number is present AND every
///   component carries at least one content hash.
/// - **L3** — hardened/attested source: all of L2 plus a supplier on every component.
/// - **L4** — two-party review / hermetic completeness: all of L3 plus the document
///   expresses dependency relationships.
///
/// The achieved level is the highest tier whose requirements (and all lower tiers')
/// hold. `passed` is true once at least L1 is reached.
pub fn assess_slsa(sbom: &Sbom) -> Result<(SlsaLevel, ComplianceResult), ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let mut result = ComplianceResult::new(&Framework::Slsa);

    // L1: provenance exists.
    let l1 = has_tools(sbom);
    result.record("L1_provenance_exists", l1);

    // L2: authenticated/versioned document + every component hashed.
    let has_serial = sbom
        .serial_number
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty());
    let all_hashed = all_components(sbom, |c| !c.hashes.is_empty());
    let l2 = l1 && has_serial && all_hashed;
    result.record("L2_authenticated_document", has_serial);
    result.record("L2_all_components_hashed", all_hashed);

    // L3: hardened/attested source — supplier on every component.
    let all_supplied = all_components(sbom, |c| {
        c.supplier
            .as_ref()
            .is_some_and(|s| !s.name.trim().is_empty())
    });
    let l3 = l2 && all_supplied;
    result.record("L3_supplier_on_every_component", all_supplied);

    // L4: two-party review / hermetic completeness — dependency edges present.
    let has_deps = !sbom.dependencies.is_empty();
    let l4 = l3 && has_deps;
    result.record("L4_dependency_relationships", has_deps);

    // Highest contiguous level reached.
    let level = if l4 {
        SlsaLevel::L4
    } else if l3 {
        SlsaLevel::L3
    } else if l2 {
        SlsaLevel::L2
    } else if l1 {
        SlsaLevel::L1
    } else {
        SlsaLevel::L0
    };

    result.level = Some(level.tag().to_string());
    result.passed = level.rank() >= SlsaLevel::L1.rank();
    result.note(format!("achieved SLSA {}", level.tag()));
    result
        .note("structural proxy: SBOM evidence is necessary but not sufficient for SLSA controls");
    Ok((level, result))
}

/// Convenience wrapper returning only the [`ComplianceResult`] for SLSA.
pub fn check_slsa(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    assess_slsa(sbom).map(|(_, result)| result)
}

// ---------------------------------------------------------------------------
// in-toto.
// ---------------------------------------------------------------------------

/// Certify in-toto attestation completeness from SBOM structure.
///
/// Maps the in-toto layout vocabulary onto SBOM structure as a proxy:
///
/// - **functionaries** → generating tools are recorded (who produced the SBOM).
/// - **products** → a primary component is identified (the produced artifact).
/// - **materials/products linkage** → dependency edges are present (inputs→outputs).
///
/// Passes iff all three structural attestations are present.
pub fn check_in_toto(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let mut result = ComplianceResult::new(&Framework::InToto);

    let functionaries = has_tools(sbom);
    result.record("functionaries_present", functionaries);

    let products = sbom
        .metadata
        .primary_component
        .as_ref()
        .is_some_and(|p| !p.trim().is_empty());
    result.record("products_primary_component", products);

    let linkage = !sbom.dependencies.is_empty();
    result.record("materials_products_linkage", linkage);

    result.passed = functionaries && products && linkage;
    result.note("structural proxy: SBOM fields stand in for in-toto layout/link metadata");
    Ok(result)
}

// ---------------------------------------------------------------------------
// CISA SBOM (minimum + recommended).
// ---------------------------------------------------------------------------

/// Certify the SBOM against CISA SBOM minimum + recommended fields.
///
/// The minimum gate reuses NTIA conformance. The recommended gate reports
/// identifier coverage (fraction of components with a PURL or CPE) and license
/// coverage (fraction with at least one license), recording both ratios in the
/// notes. Recommended fields are satisfied when both ratios meet a 0.80 threshold.
/// Passes iff the minimum (NTIA) gate holds.
pub fn check_cisa(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let mut result = ComplianceResult::new(&Framework::Cisa);

    let ntia_ok = sbom.ntia_minimum_elements().is_conformant();
    result.record("minimum_ntia_elements", ntia_ok);

    let total = sbom.components.len();
    let id_hits = sbom.components.iter().filter(|c| has_stable_id(c)).count();
    let lic_hits = sbom
        .components
        .iter()
        .filter(|c| !c.licenses.is_empty())
        .count();
    let id_ratio = coverage(sbom, has_stable_id);
    let lic_ratio = coverage(sbom, |c| !c.licenses.is_empty());

    const RECOMMENDED_THRESHOLD: f64 = 0.80;
    let id_ok = id_ratio >= RECOMMENDED_THRESHOLD;
    let lic_ok = lic_ratio >= RECOMMENDED_THRESHOLD;
    result.record("recommended_identifier_coverage", id_ok);
    result.record("recommended_license_coverage", lic_ok);

    result.note(ratio_note("identifier (purl/cpe) coverage", id_hits, total));
    result.note(ratio_note("license coverage", lic_hits, total));
    result.note(format!(
        "recommended threshold: {:.0}%",
        RECOMMENDED_THRESHOLD * 100.0
    ));

    result.passed = ntia_ok;
    Ok(result)
}

// ---------------------------------------------------------------------------
// C-SCRM / NIST SP 800-161.
// ---------------------------------------------------------------------------

/// Certify the SBOM against C-SCRM (NIST SP 800-161) supplier-coverage gates.
///
/// Supply-chain risk management is supplier-centric, so this checker measures
/// supplier coverage across all components and additionally requires a supplier
/// on the primary component (the artifact under management). Passes iff supplier
/// coverage meets a 0.90 threshold AND the primary component (when identified)
/// carries a supplier.
pub fn check_cscrm(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let mut result = ComplianceResult::new(&Framework::Cscrm);

    let total = sbom.components.len();
    let supplied = |c: &crate::sbom::Component| {
        c.supplier
            .as_ref()
            .is_some_and(|s| !s.name.trim().is_empty())
    };
    let sup_hits = sbom.components.iter().filter(|c| supplied(c)).count();
    let sup_ratio = coverage(sbom, supplied);

    const SUPPLIER_THRESHOLD: f64 = 0.90;
    let coverage_ok = sup_ratio >= SUPPLIER_THRESHOLD;
    result.record("supplier_coverage", coverage_ok);

    // Primary-component supplier: satisfied if a primary is named and it (or the
    // document-level supplier) carries a supplier name. If no primary is declared,
    // this gate is recorded as failed (cannot certify the managed artifact).
    let primary_supplier = match sbom.metadata.primary_component.as_ref() {
        Some(pref) if !pref.trim().is_empty() => {
            sbom.component(pref).map(supplied).unwrap_or(false)
                || sbom
                    .metadata
                    .supplier
                    .as_ref()
                    .is_some_and(|s| !s.name.trim().is_empty())
        }
        _ => false,
    };
    result.record("primary_component_supplier", primary_supplier);

    result.note(ratio_note("supplier coverage", sup_hits, total));
    result.note(format!(
        "supplier threshold: {:.0}%",
        SUPPLIER_THRESHOLD * 100.0
    ));

    result.passed = coverage_ok && primary_supplier;
    Ok(result)
}

// ---------------------------------------------------------------------------
// ISO/IEC 27001 and SOC 2 (evidence-presence proxies).
// ---------------------------------------------------------------------------

/// Minimum fraction of components that must carry integrity hashes for the
/// ISO/SOC integrity-evidence gate.
const INTEGRITY_THRESHOLD: f64 = 0.50;

/// Certify lightweight ISO/IEC 27001 evidence-presence gates.
///
/// ISO 27001 is a management-system standard that an SBOM cannot fully certify.
/// This checker is an explicit **structural proxy**: it gates on the presence of
/// audit-relevant evidence — a named author (accountability), a timestamp
/// (record-keeping), and integrity hashes on at least 50% of components
/// (integrity controls, Annex A.8 lineage). Passes iff all three hold.
pub fn check_iso_27001(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    evidence_presence_gate(sbom, &Framework::Iso27001)
}

/// Certify lightweight SOC 2 evidence-presence gates.
///
/// Like [`check_iso_27001`], this is an explicit **structural proxy** for SOC 2
/// trust-services criteria. It gates on the same audit-evidence triad: author
/// accountability, timestamped records, and integrity hashes on at least 50% of
/// components. Passes iff all three hold.
pub fn check_soc2(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    evidence_presence_gate(sbom, &Framework::Soc2)
}

/// Shared evidence-presence gate used by ISO 27001 and SOC 2 proxies.
fn evidence_presence_gate(
    sbom: &Sbom,
    framework: &Framework,
) -> Result<ComplianceResult, ComplianceError> {
    let mut result = ComplianceResult::new(framework);

    let author = has_author(sbom);
    result.record("author_accountability", author);

    let timestamp = has_timestamp(sbom);
    result.record("timestamped_record", timestamp);

    let total = sbom.components.len();
    let hashed = sbom
        .components
        .iter()
        .filter(|c| !c.hashes.is_empty())
        .count();
    let integrity_ratio = coverage(sbom, |c| !c.hashes.is_empty());
    let integrity_ok = integrity_ratio >= INTEGRITY_THRESHOLD;
    result.record("integrity_hash_evidence", integrity_ok);

    result.note(ratio_note("integrity-hash coverage", hashed, total));
    result.note(format!(
        "integrity threshold: {:.0}%",
        INTEGRITY_THRESHOLD * 100.0
    ));
    result.note("structural proxy: evidence-presence gate, not a full controls audit");

    result.passed = author && timestamp && integrity_ok;
    Ok(result)
}

// ---------------------------------------------------------------------------
// VEX readiness.
// ---------------------------------------------------------------------------

/// Minimum fraction of components that must carry a stable identifier for VEX
/// readiness.
const VEX_THRESHOLD: f64 = 0.80;

/// Certify whether the SBOM is ready to anchor VEX statements.
///
/// VEX (Vulnerability Exploitability eXchange) statements reference product and
/// component identifiers. This checker reports the fraction of components carrying
/// a stable external coordinate (PURL or CPE) — the anchor a VEX statement needs —
/// and passes iff that coverage meets a 0.80 threshold. It does **not** depend on
/// any vulnerability module and makes no claim about exploitability; it only
/// certifies identifier sufficiency.
pub fn vex_readiness(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    let mut result = ComplianceResult::new(&Framework::Vex);

    let total = sbom.components.len();
    let purl_hits = sbom
        .components
        .iter()
        .filter(|c| c.purl.as_ref().is_some_and(|p| !p.trim().is_empty()))
        .count();
    let cpe_hits = sbom
        .components
        .iter()
        .filter(|c| c.cpe.as_ref().is_some_and(|p| !p.trim().is_empty()))
        .count();
    let anchor_hits = sbom.components.iter().filter(|c| has_stable_id(c)).count();
    let anchor_ratio = coverage(sbom, has_stable_id);

    let anchor_ok = anchor_ratio >= VEX_THRESHOLD;
    result.record("stable_identifier_coverage", anchor_ok);

    result.note(ratio_note("purl coverage", purl_hits, total));
    result.note(ratio_note("cpe coverage", cpe_hits, total));
    result.note(ratio_note("vex-anchorable coverage", anchor_hits, total));
    result.note(format!(
        "readiness threshold: {:.0}%",
        VEX_THRESHOLD * 100.0
    ));

    result.passed = anchor_ok;
    Ok(result)
}

// ---------------------------------------------------------------------------
// Orchestration.
// ---------------------------------------------------------------------------

/// Run every framework checker over `sbom` in a fixed, deterministic order.
///
/// The order is: NTIA, EO 14028, SLSA, in-toto, CISA, C-SCRM, ISO 27001, SOC 2,
/// VEX — matching the [`Framework`] enum declaration. Returns an error only when
/// the SBOM is empty (no components to certify).
pub fn assess_all(sbom: &Sbom) -> Result<Vec<ComplianceResult>, ComplianceError> {
    if sbom.components.is_empty() {
        return Err(ComplianceError::EmptySbom);
    }
    Ok(vec![
        check_ntia(sbom)?,
        check_eo_14028(sbom)?,
        check_slsa(sbom)?,
        check_in_toto(sbom)?,
        check_cisa(sbom)?,
        check_cscrm(sbom)?,
        check_iso_27001(sbom)?,
        check_soc2(sbom)?,
        vex_readiness(sbom)?,
    ])
}

/// The stable tags of every framework [`assess_all`] runs, in order.
pub fn supported_frameworks() -> &'static [&'static str] {
    &[
        "ntia",
        "eo-14028",
        "slsa",
        "in-toto",
        "cisa",
        "cscrm-800-161",
        "iso-27001",
        "soc-2",
        "vex",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sbom::{Component, Dependency, License, Sbom, SbomFormat, Supplier, Tool};

    /// A component with the given coordinates and optional enrichments.
    fn component(bom_ref: &str, name: &str, version: &str) -> Component {
        Component::library(bom_ref, name, version)
    }

    /// Attach a supplier name to a component.
    fn with_supplier(mut c: Component, name: &str) -> Component {
        c.supplier = Some(Supplier {
            name: name.to_string(),
            url: None,
            contact: None,
        });
        c
    }

    /// Attach a PURL to a component.
    fn with_purl(mut c: Component, purl: &str) -> Component {
        c.purl = Some(purl.to_string());
        c
    }

    /// Attach a content hash to a component.
    fn with_hash(mut c: Component, value: &str) -> Component {
        c.hashes.push(crate::sbom::Hash::new("SHA-256", value));
        c
    }

    /// Attach a license to a component.
    fn with_license(mut c: Component, id: &str) -> Component {
        c.licenses.push(License::id(id));
        c
    }

    /// A fully NTIA-conformant two-component SBOM with document metadata.
    fn full_sbom() -> Sbom {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.serial_number = Some("urn:uuid:abcd".to_string());
        sbom.metadata.author = Some("Build Bot".to_string());
        sbom.metadata.timestamp = 1_705_314_600;
        sbom.metadata.tools.push(Tool {
            vendor: Some("acme".to_string()),
            name: "cdxgen".to_string(),
            version: Some("9.0".to_string()),
        });
        sbom.metadata.primary_component = Some("app@1.0".to_string());
        sbom.metadata.supplier = Some(Supplier {
            name: "Acme".to_string(),
            url: None,
            contact: None,
        });

        let a = with_license(
            with_hash(
                with_purl(
                    with_supplier(component("a", "alpha", "1.0"), "Acme"),
                    "pkg:cargo/alpha@1.0",
                ),
                "aaaa",
            ),
            "MIT",
        );
        let b = with_license(
            with_hash(
                with_purl(
                    with_supplier(component("b", "beta", "2.0"), "Beta Inc"),
                    "pkg:cargo/beta@2.0",
                ),
                "bbbb",
            ),
            "Apache-2.0",
        );
        sbom.components.push(a);
        sbom.components.push(b);
        sbom.dependencies.push(Dependency {
            dependent: "a".to_string(),
            depends_on: vec!["b".to_string()],
        });
        sbom
    }

    /// An SBOM whose structure tops out at exactly SLSA L2:
    /// tools + serial + all hashed, but NOT every component has a supplier.
    fn slsa_l2_sbom() -> Sbom {
        let mut sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        sbom.serial_number = Some("urn:uuid:l2".to_string());
        sbom.metadata.tools.push(Tool {
            vendor: None,
            name: "gen".to_string(),
            version: None,
        });
        // Both hashed; only one has a supplier → blocks L3.
        let a = with_hash(with_supplier(component("a", "a", "1"), "Acme"), "aa");
        let b = with_hash(component("b", "b", "1"), "bb"); // no supplier
        sbom.components.push(a);
        sbom.components.push(b);
        // No dependency edges (would not matter without L3 anyway).
        sbom
    }

    #[test]
    fn ntia_passes_on_full_sbom() {
        let sbom = full_sbom();
        let result = check_ntia(&sbom).unwrap();
        assert!(result.passed, "failed: {:?}", result.failed);
        assert_eq!(result.requirement_count(), 7);
        assert!(result.failed.is_empty());
    }

    #[test]
    fn ntia_fails_when_versions_missing() {
        let mut sbom = full_sbom();
        sbom.components[0].version = String::new();
        let result = check_ntia(&sbom).unwrap();
        assert!(!result.passed);
        assert!(result.failed.contains(&"version".to_string()));
    }

    #[test]
    fn ntia_errors_on_empty_sbom() {
        let sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        assert_eq!(check_ntia(&sbom), Err(ComplianceError::EmptySbom));
    }

    #[test]
    fn eo_14028_passes_on_full_sbom() {
        let sbom = full_sbom();
        let result = check_eo_14028(&sbom).unwrap();
        assert!(result.passed, "failed: {:?}", result.failed);
        assert!(result
            .satisfied
            .contains(&"sbom_tooling_provenance".to_string()));
    }

    #[test]
    fn eo_14028_fails_without_tools() {
        let mut sbom = full_sbom();
        sbom.metadata.tools.clear();
        let result = check_eo_14028(&sbom).unwrap();
        assert!(!result.passed);
        assert!(result
            .failed
            .contains(&"sbom_tooling_provenance".to_string()));
    }

    #[test]
    fn slsa_reaches_l4_on_full_sbom() {
        let sbom = full_sbom();
        let (level, result) = assess_slsa(&sbom).unwrap();
        assert_eq!(level, SlsaLevel::L4);
        assert_eq!(result.level.as_deref(), Some("L4"));
        assert!(result.passed);
    }

    #[test]
    fn slsa_l2_sbom_is_exactly_l2_not_l3() {
        let sbom = slsa_l2_sbom();
        let (level, _) = assess_slsa(&sbom).unwrap();
        assert_eq!(level, SlsaLevel::L2);
        assert!(level.rank() < SlsaLevel::L3.rank());
    }

    #[test]
    fn slsa_drops_to_l1_without_serial() {
        let mut sbom = slsa_l2_sbom();
        sbom.serial_number = None; // breaks the L2 authenticated-doc gate
        let (level, _) = assess_slsa(&sbom).unwrap();
        assert_eq!(level, SlsaLevel::L1);
    }

    #[test]
    fn slsa_is_l0_without_tools() {
        let mut sbom = slsa_l2_sbom();
        sbom.metadata.tools.clear();
        let (level, result) = assess_slsa(&sbom).unwrap();
        assert_eq!(level, SlsaLevel::L0);
        assert!(!result.passed, "L0 must not pass the SLSA gate");
    }

    #[test]
    fn slsa_l3_without_deps_is_not_l4() {
        // Full SBOM but strip dependencies: every component supplied + hashed +
        // serial + tools → L3, but no edges → not L4.
        let mut sbom = full_sbom();
        sbom.dependencies.clear();
        let (level, _) = assess_slsa(&sbom).unwrap();
        assert_eq!(level, SlsaLevel::L3);
    }

    #[test]
    fn in_toto_passes_with_tools_primary_and_deps() {
        let sbom = full_sbom();
        let result = check_in_toto(&sbom).unwrap();
        assert!(result.passed, "failed: {:?}", result.failed);
    }

    #[test]
    fn in_toto_fails_without_primary_component() {
        let mut sbom = full_sbom();
        sbom.metadata.primary_component = None;
        let result = check_in_toto(&sbom).unwrap();
        assert!(!result.passed);
        assert!(result
            .failed
            .contains(&"products_primary_component".to_string()));
    }

    #[test]
    fn cisa_reports_coverage_ratios() {
        // Two components, one without purl/cpe → 50% identifier coverage.
        let mut sbom = full_sbom();
        sbom.components[1].purl = None;
        sbom.components[1].cpe = None;
        let result = check_cisa(&sbom).unwrap();
        // Minimum NTIA still depends on unique_identifiers; component[1] has a hash
        // so it still counts as a unique identifier → NTIA may still pass.
        assert!(result
            .notes
            .iter()
            .any(|n| n.contains("identifier (purl/cpe) coverage: 1/2")));
        // 50% < 80% recommended threshold.
        assert!(result
            .failed
            .contains(&"recommended_identifier_coverage".to_string()));
    }

    #[test]
    fn cisa_recommended_passes_at_full_coverage() {
        let sbom = full_sbom();
        let result = check_cisa(&sbom).unwrap();
        assert!(result
            .satisfied
            .contains(&"recommended_identifier_coverage".to_string()));
        assert!(result
            .satisfied
            .contains(&"recommended_license_coverage".to_string()));
        assert!(result.passed);
    }

    #[test]
    fn cscrm_passes_with_full_supplier_coverage() {
        let sbom = full_sbom();
        let result = check_cscrm(&sbom).unwrap();
        assert!(result.passed, "failed: {:?}", result.failed);
        assert!(result.satisfied.contains(&"supplier_coverage".to_string()));
    }

    #[test]
    fn cscrm_fails_below_supplier_threshold() {
        // Drop one of two suppliers → 50% < 90%.
        let mut sbom = full_sbom();
        sbom.components[1].supplier = None;
        let result = check_cscrm(&sbom).unwrap();
        assert!(!result.passed);
        assert!(result.failed.contains(&"supplier_coverage".to_string()));
    }

    #[test]
    fn iso_and_soc2_pass_on_full_sbom() {
        let sbom = full_sbom();
        let iso = check_iso_27001(&sbom).unwrap();
        let soc = check_soc2(&sbom).unwrap();
        assert!(iso.passed, "iso failed: {:?}", iso.failed);
        assert!(soc.passed, "soc failed: {:?}", soc.failed);
        assert_eq!(iso.framework, "ISO/IEC 27001");
        assert_eq!(soc.framework, "SOC 2");
    }

    #[test]
    fn iso_fails_without_author() {
        let mut sbom = full_sbom();
        sbom.metadata.author = None;
        let result = check_iso_27001(&sbom).unwrap();
        assert!(!result.passed);
        assert!(result.failed.contains(&"author_accountability".to_string()));
    }

    #[test]
    fn vex_readiness_passes_at_full_identifier_coverage() {
        let sbom = full_sbom();
        let result = vex_readiness(&sbom).unwrap();
        assert!(result.passed);
        assert!(result
            .satisfied
            .contains(&"stable_identifier_coverage".to_string()));
    }

    #[test]
    fn vex_readiness_fails_below_threshold() {
        // Strip identifiers from one of two components → 50% < 80%.
        let mut sbom = full_sbom();
        sbom.components[1].purl = None;
        sbom.components[1].cpe = None;
        let result = vex_readiness(&sbom).unwrap();
        assert!(!result.passed);
        assert!(result
            .notes
            .iter()
            .any(|n| n.contains("vex-anchorable coverage: 1/2")));
    }

    #[test]
    fn score_is_satisfied_fraction() {
        let mut r = ComplianceResult::new(&Framework::Ntia);
        r.record("a", true);
        r.record("b", true);
        r.record("c", false);
        r.record("d", false);
        assert!((r.score() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn score_is_zero_for_no_requirements() {
        let r = ComplianceResult::new(&Framework::Ntia);
        assert_eq!(r.score(), 0.0);
    }

    #[test]
    fn assess_all_runs_every_framework_in_order() {
        let sbom = full_sbom();
        let results = assess_all(&sbom).unwrap();
        assert_eq!(results.len(), supported_frameworks().len());
        assert_eq!(results.len(), 9);
        let names: Vec<&str> = results.iter().map(|r| r.framework.as_str()).collect();
        assert_eq!(names[0], Framework::Ntia.display_name());
        assert_eq!(names[1], Framework::ExecutiveOrder14028.display_name());
        assert_eq!(names[2], Framework::Slsa.display_name());
        assert_eq!(names[8], Framework::Vex.display_name());
    }

    #[test]
    fn assess_all_errors_on_empty_sbom() {
        let sbom = Sbom::new(SbomFormat::CycloneDx16, "1.6");
        assert_eq!(assess_all(&sbom), Err(ComplianceError::EmptySbom));
    }

    #[test]
    fn assess_all_is_deterministic() {
        let sbom = full_sbom();
        let a = assess_all(&sbom).unwrap();
        let b = assess_all(&sbom).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn supported_frameworks_match_enum_tags() {
        let tags = supported_frameworks();
        assert_eq!(tags[0], Framework::Ntia.tag());
        assert_eq!(tags[2], Framework::Slsa.tag());
        assert_eq!(tags[8], Framework::Vex.tag());
        assert_eq!(tags.len(), 9);
    }

    #[test]
    fn framework_tags_are_stable_and_distinct() {
        let all = [
            Framework::Ntia,
            Framework::ExecutiveOrder14028,
            Framework::Slsa,
            Framework::InToto,
            Framework::Cisa,
            Framework::Cscrm,
            Framework::Iso27001,
            Framework::Soc2,
            Framework::Vex,
        ];
        let mut tags: Vec<&str> = all.iter().map(|f| f.tag()).collect();
        let count = tags.len();
        tags.sort_unstable();
        tags.dedup();
        assert_eq!(tags.len(), count, "framework tags must be distinct");
    }
}
