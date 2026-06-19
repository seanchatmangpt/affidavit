//! OCEL-backed compliance gates for SOC2, GDPR, PCI-DSS, and HIPAA.
//!
//! Each gate runs the receipt chain through the certify pipeline (via
//! `crate::verifier::verify`) and then applies domain-specific checks on top of
//! the verified receipt's OCEL event log.  A gate returns `Ok(())` only when
//! both layers pass; it returns `Err(ComplianceError)` with a machine-readable
//! violation code otherwise.
//!
//! Design principle: compliance is just the certify pipeline with domain-specific
//! stage requirements added on top.  The certify pipeline already proves chain
//! integrity and continuity; the gates add coverage/lineage checks.

use crate::types::Receipt;
use crate::verifier::verify;
use std::collections::BTreeSet;

/// A structured compliance violation with a machine-readable code and human
/// explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplianceViolation {
    /// Short identifier, e.g. "SOC2-COVERAGE", "GDPR-ORPHAN".
    pub code: String,
    /// Human-readable explanation suitable for an audit report.
    pub detail: String,
}

/// Error returned when a compliance gate rejects a receipt (or set of receipts).
#[derive(Debug, Clone)]
pub struct ComplianceError {
    /// Which regulation or standard the gate belongs to.
    pub regulation: String,
    /// All violations found (there may be more than one).
    pub violations: Vec<ComplianceViolation>,
}

impl std::fmt::Display for ComplianceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} compliance gate rejected:", self.regulation)?;
        for v in &self.violations {
            write!(f, " [{}] {}", v.code, v.detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for ComplianceError {}

// ---------------------------------------------------------------------------
// SOC 2 gate
// ---------------------------------------------------------------------------

/// Required certify pipeline stage names for SOC 2 availability control.
///
/// A receipt proves "availability" only when every stage of the certify
/// pipeline ran and passed.  Missing stages mean controls were skipped.
const SOC2_REQUIRED_STAGES: &[&str] = &[
    "decode",
    "check_format",
    "chain_integrity",
    "continuity",
    "verify_commitments",
    "evaluate_profile",
];

/// SOC 2 audit gate.
///
/// Checks:
/// 1. The certify pipeline accepts the receipt (all 6 stages pass).
/// 2. Every required stage is present in the outcome list (stage coverage).
/// 3. Every OCEL event has at least one object reference (no orphaned events).
///
/// Returns `Ok(())` if all checks pass, `Err(ComplianceError)` otherwise.
pub fn soc2_gate(receipt: &Receipt) -> Result<(), ComplianceError> {
    let verdict = verify(receipt);
    let mut violations = Vec::new();

    // Check 1: certify pipeline accepted.
    if !verdict.accepted {
        violations.push(ComplianceViolation {
            code: "SOC2-PIPELINE".to_string(),
            detail: format!("certify pipeline rejected receipt: {}", verdict.reason),
        });
    }

    // Check 2: stage coverage — every required stage is present and passed.
    let outcome_stages: BTreeSet<&str> = verdict
        .outcomes
        .iter()
        .filter(|o| o.passed)
        .map(|o| o.stage.as_str())
        .collect();

    for required in SOC2_REQUIRED_STAGES {
        if !outcome_stages.contains(required) {
            violations.push(ComplianceViolation {
                code: "SOC2-COVERAGE".to_string(),
                detail: format!("required certify stage '{required}' did not pass — control gap"),
            });
        }
    }

    // Check 3: no orphaned events (every event must reference at least one object).
    for event in &receipt.events {
        if event.objects.is_empty() {
            violations.push(ComplianceViolation {
                code: "SOC2-ORPHAN".to_string(),
                detail: format!(
                    "event '{}' (seq {}) has no object references — cannot map to a SOC 2 control object",
                    event.id, event.seq
                ),
            });
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(ComplianceError {
            regulation: "SOC 2".to_string(),
            violations,
        })
    }
}

// ---------------------------------------------------------------------------
// GDPR gate
// ---------------------------------------------------------------------------

/// GDPR proof gate — right to explanation requires complete data lineage.
///
/// Checks:
/// 1. The certify pipeline accepts the receipt (chain integrity + continuity).
/// 2. Every OCEL event has at least one object reference (complete subject
///    coverage — every processing activity has an identified data subject or
///    artifact; orphaned events violate the "right to explanation").
/// 3. No continuity gap in the chain (the pipeline's continuity stage must pass,
///    which is already covered by check 1, but we surface it explicitly).
/// 4. No duplicate event IDs (fork detection — a forked log cannot prove a single
///    lawful processing history).
///
/// Returns `Ok(())` if all checks pass, `Err(ComplianceError)` otherwise.
pub fn gdpr_gate(receipt: &Receipt) -> Result<(), ComplianceError> {
    let verdict = verify(receipt);
    let mut violations = Vec::new();

    // Check 1: certify pipeline accepted (includes continuity + chain integrity).
    if !verdict.accepted {
        violations.push(ComplianceViolation {
            code: "GDPR-PIPELINE".to_string(),
            detail: format!("certify pipeline rejected receipt: {}", verdict.reason),
        });
    }

    // Check 2: every event has a subject (no orphaned processing activities).
    for event in &receipt.events {
        if event.objects.is_empty() {
            violations.push(ComplianceViolation {
                code: "GDPR-ORPHAN".to_string(),
                detail: format!(
                    "event '{}' (seq {}) has no subject — processing activity cannot be attributed (Art. 5 accountability)",
                    event.id, event.seq
                ),
            });
        }
    }

    // Check 3: no duplicate event IDs (fork detection).
    let mut seen_ids: BTreeSet<&str> = BTreeSet::new();
    for event in &receipt.events {
        if !seen_ids.insert(event.id.as_str()) {
            violations.push(ComplianceViolation {
                code: "GDPR-FORK".to_string(),
                detail: format!(
                    "duplicate event id '{}' detected — chain is forked and cannot prove a single lawful processing history",
                    event.id
                ),
            });
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(ComplianceError {
            regulation: "GDPR".to_string(),
            violations,
        })
    }
}

// ---------------------------------------------------------------------------
// PCI-DSS gate
// ---------------------------------------------------------------------------

/// PCI-DSS gate — integrity of payment processing trail.
///
/// Checks:
/// 1. The certify pipeline accepts the receipt.
/// 2. No stage was skipped: the continuity outcome must pass (seq strictly
///    increasing from 0, no gaps).
/// 3. Chain seal is unbroken: the chain_integrity outcome must pass.
///
/// Returns `Ok(())` if all checks pass, `Err(ComplianceError)` otherwise.
pub fn pci_dss_gate(receipt: &Receipt) -> Result<(), ComplianceError> {
    let verdict = verify(receipt);
    let mut violations = Vec::new();

    // Collect per-stage pass/fail for PCI-specific assertions.
    let continuity_passed = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "continuity")
        .map(|o| o.passed)
        .unwrap_or(false);

    let chain_integrity_passed = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "chain_integrity")
        .map(|o| o.passed)
        .unwrap_or(false);

    if !verdict.accepted {
        violations.push(ComplianceViolation {
            code: "PCI-PIPELINE".to_string(),
            detail: format!("certify pipeline rejected receipt: {}", verdict.reason),
        });
    }

    if !continuity_passed {
        let detail = verdict
            .outcomes
            .iter()
            .find(|o| o.stage == "continuity")
            .map(|o| o.detail.clone())
            .unwrap_or_else(|| "continuity stage missing".to_string());
        violations.push(ComplianceViolation {
            code: "PCI-SKIP".to_string(),
            detail: format!("PCI-DSS Req 10.2 — stage skipped or seq gap detected: {detail}"),
        });
    }

    if !chain_integrity_passed {
        let detail = verdict
            .outcomes
            .iter()
            .find(|o| o.stage == "chain_integrity")
            .map(|o| o.detail.clone())
            .unwrap_or_else(|| "chain_integrity stage missing".to_string());
        violations.push(ComplianceViolation {
            code: "PCI-SEAL".to_string(),
            detail: format!("PCI-DSS Req 10.5 — chain seal broken: {detail}"),
        });
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(ComplianceError {
            regulation: "PCI-DSS".to_string(),
            violations,
        })
    }
}

// ---------------------------------------------------------------------------
// HIPAA gate (re-uses SOC2 + GDPR logic: complete stage coverage + lineage)
// ---------------------------------------------------------------------------

/// HIPAA gate — access controls + audit trail integrity.
///
/// Checks:
/// 1. The certify pipeline accepts the receipt.
/// 2. Every event has at least one object reference (access log completeness —
///    every PHI access must be traceable to a subject/resource).
/// 3. Continuity: no seq gaps (audit log cannot have holes per HIPAA §164.312(b)).
///
/// Returns `Ok(())` if all checks pass, `Err(ComplianceError)` otherwise.
pub fn hipaa_gate(receipt: &Receipt) -> Result<(), ComplianceError> {
    let verdict = verify(receipt);
    let mut violations = Vec::new();

    if !verdict.accepted {
        violations.push(ComplianceViolation {
            code: "HIPAA-PIPELINE".to_string(),
            detail: format!("certify pipeline rejected receipt: {}", verdict.reason),
        });
    }

    // Every PHI access event must be traceable.
    for event in &receipt.events {
        if event.objects.is_empty() {
            violations.push(ComplianceViolation {
                code: "HIPAA-LINEAGE".to_string(),
                detail: format!(
                    "event '{}' (seq {}) has no object references — PHI access cannot be attributed (§164.312(b))",
                    event.id, event.seq
                ),
            });
        }
    }

    // No continuity gaps.
    let continuity_passed = verdict
        .outcomes
        .iter()
        .find(|o| o.stage == "continuity")
        .map(|o| o.passed)
        .unwrap_or(false);

    if !continuity_passed {
        let detail = verdict
            .outcomes
            .iter()
            .find(|o| o.stage == "continuity")
            .map(|o| o.detail.clone())
            .unwrap_or_else(|| "continuity stage missing".to_string());
        violations.push(ComplianceViolation {
            code: "HIPAA-GAP".to_string(),
            detail: format!("audit log continuity gap — HIPAA §164.312(b) violation: {detail}"),
        });
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(ComplianceError {
            regulation: "HIPAA".to_string(),
            violations,
        })
    }
}

// ---------------------------------------------------------------------------
// Aggregate result type for reporting
// ---------------------------------------------------------------------------

/// The outcome of running all four compliance gates against a single receipt.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplianceReport {
    /// Number of events in the receipt.
    pub event_count: usize,
    /// Chain hash (hex) of the receipt.
    pub chain_hash: String,
    /// SOC 2 gate result.
    pub soc2: GateResult,
    /// GDPR gate result.
    pub gdpr: GateResult,
    /// PCI-DSS gate result.
    pub pci_dss: GateResult,
    /// HIPAA gate result.
    pub hipaa: GateResult,
    /// `true` when all four gates pass.
    pub all_passed: bool,
}

/// Per-gate pass/fail result with structured violations.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GateResult {
    /// Whether this gate passed.
    pub passed: bool,
    /// Violations (empty when `passed` is `true`).
    pub violations: Vec<GateViolation>,
}

/// Serializable form of a compliance violation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GateViolation {
    pub code: String,
    pub detail: String,
}

impl From<ComplianceError> for GateResult {
    fn from(err: ComplianceError) -> Self {
        GateResult {
            passed: false,
            violations: err
                .violations
                .into_iter()
                .map(|v| GateViolation {
                    code: v.code,
                    detail: v.detail,
                })
                .collect(),
        }
    }
}

impl GateResult {
    fn pass() -> Self {
        GateResult {
            passed: true,
            violations: vec![],
        }
    }
}

/// Run all four compliance gates against `receipt` and return a structured report.
pub fn run_all_gates(receipt: &Receipt) -> ComplianceReport {
    let soc2 = match soc2_gate(receipt) {
        Ok(()) => GateResult::pass(),
        Err(e) => GateResult::from(e),
    };
    let gdpr = match gdpr_gate(receipt) {
        Ok(()) => GateResult::pass(),
        Err(e) => GateResult::from(e),
    };
    let pci_dss = match pci_dss_gate(receipt) {
        Ok(()) => GateResult::pass(),
        Err(e) => GateResult::from(e),
    };
    let hipaa = match hipaa_gate(receipt) {
        Ok(()) => GateResult::pass(),
        Err(e) => GateResult::from(e),
    };

    let all_passed = soc2.passed && gdpr.passed && pci_dss.passed && hipaa.passed;

    ComplianceReport {
        event_count: receipt.events.len(),
        chain_hash: receipt.chain_hash.as_hex().to_string(),
        soc2,
        gdpr,
        pci_dss,
        hipaa,
        all_passed,
    }
}
