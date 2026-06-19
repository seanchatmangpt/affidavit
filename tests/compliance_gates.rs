//! Integration tests for the OCEL-backed compliance gates.
//!
//! Each gate is tested with:
//! (a) a conforming receipt that must PASS
//! (b) a non-conforming receipt that must FAIL with the expected violation code
//!
//! Teeth verification: the conforming receipt passes, so a false-negative in the
//! conforming test would show as a test failure; the non-conforming test is only
//! reachable when the gate correctly detects the defect.

use affidavit::chain::ChainAssembler;
use affidavit::compliance::{gdpr_gate, hipaa_gate, pci_dss_gate, run_all_gates, soc2_gate};
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::types::Receipt;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a well-formed receipt with `n` events, each referencing one object.
fn conforming_receipt(n: usize) -> Receipt {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();
    for i in 0..n {
        let ev = build_event(
            &format!("certify-stage-{i}"),
            vec![object_ref(format!("artifact-{i}"), "artifact")],
            format!("payload-{i}").as_bytes(),
            &mut counter,
        )
        .expect("build event");
        asm.append(ev).expect("append");
    }
    asm.finalize()
}

/// Build a receipt where one event has no object references (orphaned event).
fn orphaned_event_receipt() -> Receipt {
    let mut counter = SeqCounter::new();
    let mut asm = ChainAssembler::new();

    // First event is fine.
    let ev0 = build_event(
        "certify-stage-0",
        vec![object_ref("artifact-0", "artifact")],
        b"payload-0",
        &mut counter,
    )
    .expect("build event");
    asm.append(ev0).expect("append");

    // Second event has NO objects — this is the orphan.
    let ev1 = build_event("certify-stage-1", vec![], b"payload-1", &mut counter)
        .expect("build event");
    asm.append(ev1).expect("append");

    asm.finalize()
}

// ---------------------------------------------------------------------------
// SOC 2 gate tests
// ---------------------------------------------------------------------------

#[test]
fn soc2_conforming_receipt_passes() {
    let receipt = conforming_receipt(3);
    let result = soc2_gate(&receipt);
    assert!(
        result.is_ok(),
        "conforming receipt must pass SOC 2 gate: {:?}",
        result
    );
}

#[test]
fn soc2_orphaned_event_fails_with_coverage_code() {
    let receipt = orphaned_event_receipt();
    let err = soc2_gate(&receipt).expect_err("orphaned event must fail SOC 2 gate");
    assert!(
        err.violations
            .iter()
            .any(|v| v.code == "SOC2-ORPHAN"),
        "expected SOC2-ORPHAN violation, got: {:?}",
        err.violations
    );
}

// ---------------------------------------------------------------------------
// GDPR gate tests
// ---------------------------------------------------------------------------

#[test]
fn gdpr_conforming_receipt_passes() {
    let receipt = conforming_receipt(3);
    let result = gdpr_gate(&receipt);
    assert!(
        result.is_ok(),
        "conforming receipt must pass GDPR gate: {:?}",
        result
    );
}

#[test]
fn gdpr_orphaned_event_fails_with_orphan_code() {
    let receipt = orphaned_event_receipt();
    let err = gdpr_gate(&receipt).expect_err("orphaned event must fail GDPR gate");
    assert!(
        err.violations.iter().any(|v| v.code == "GDPR-ORPHAN"),
        "expected GDPR-ORPHAN violation, got: {:?}",
        err.violations
    );
}

/// Build a receipt that has a seq gap (continuity violation) — simulates a
/// skipped stage or missing event.  We construct events manually with a bad seq
/// to bypass the assembler's sequential append.
#[test]
fn gdpr_continuity_gap_fails_pipeline() {
    use affidavit::chain::recompute_chain;
    use affidavit::types::{Blake3Hash, OperationEvent, ObjectRef};

    // Build two events with a seq gap: seq 0 then seq 2 (skipping 1).
    let ev0 = OperationEvent {
        id: "evt-0".to_string(),
        seq: 0,
        event_type: "certify-stage-0".to_string(),
        objects: vec![ObjectRef {
            id: "artifact-0".to_string(),
            obj_type: "artifact".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(b"payload-0"),
    };
    let ev2 = OperationEvent {
        id: "evt-2".to_string(),
        seq: 2, // GAP: seq 1 is missing
        event_type: "certify-stage-2".to_string(),
        objects: vec![ObjectRef {
            id: "artifact-2".to_string(),
            obj_type: "artifact".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(b"payload-2"),
    };

    let events = vec![ev0, ev2];
    // We need to produce a Receipt with correct chain_hash for these events
    // (so deserialization passes), but the continuity check will still fail
    // because seq jumps from 0 to 2.
    let chain_hash = recompute_chain(&events).expect("recompute");

    // Serialize + deserialize to get a Receipt (bypasses the private constructor).
    // We build the JSON manually since Receipt::sealed is pub(crate).
    let json = serde_json::json!({
        "format_version": "core/v1",
        "events": events,
        "chain_hash": chain_hash,
    });
    let receipt: Receipt = serde_json::from_value(json).expect("deserialize receipt with gap");

    let err = gdpr_gate(&receipt).expect_err("receipt with seq gap must fail GDPR gate");
    assert!(
        err.violations.iter().any(|v| v.code == "GDPR-PIPELINE"),
        "expected GDPR-PIPELINE violation for continuity gap, got: {:?}",
        err.violations
    );
}

// ---------------------------------------------------------------------------
// PCI-DSS gate tests
// ---------------------------------------------------------------------------

#[test]
fn pci_dss_conforming_receipt_passes() {
    let receipt = conforming_receipt(4);
    let result = pci_dss_gate(&receipt);
    assert!(
        result.is_ok(),
        "conforming receipt must pass PCI-DSS gate: {:?}",
        result
    );
}

#[test]
fn pci_dss_continuity_gap_fails_with_skip_code() {
    use affidavit::chain::recompute_chain;
    use affidavit::types::{Blake3Hash, OperationEvent, ObjectRef};

    let ev0 = OperationEvent {
        id: "evt-0".to_string(),
        seq: 0,
        event_type: "deploy-0".to_string(),
        objects: vec![ObjectRef {
            id: "artifact-0".to_string(),
            obj_type: "artifact".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(b"payload-0"),
    };
    let ev2 = OperationEvent {
        id: "evt-2".to_string(),
        seq: 2, // GAP
        event_type: "deploy-2".to_string(),
        objects: vec![ObjectRef {
            id: "artifact-2".to_string(),
            obj_type: "artifact".to_string(),
            qualifier: None,
        }],
        payload_commitment: Blake3Hash::from_bytes(b"payload-2"),
    };

    let events = vec![ev0, ev2];
    let chain_hash = recompute_chain(&events).expect("recompute");

    let json = serde_json::json!({
        "format_version": "core/v1",
        "events": events,
        "chain_hash": chain_hash,
    });
    let receipt: Receipt = serde_json::from_value(json).expect("deserialize");

    let err = pci_dss_gate(&receipt).expect_err("seq gap must fail PCI-DSS gate");
    let codes: Vec<&str> = err.violations.iter().map(|v| v.code.as_str()).collect();
    assert!(
        codes.contains(&"PCI-SKIP") || codes.contains(&"PCI-PIPELINE"),
        "expected PCI-SKIP or PCI-PIPELINE violation, got: {:?}",
        codes
    );
}

// ---------------------------------------------------------------------------
// HIPAA gate tests
// ---------------------------------------------------------------------------

#[test]
fn hipaa_conforming_receipt_passes() {
    let receipt = conforming_receipt(3);
    let result = hipaa_gate(&receipt);
    assert!(
        result.is_ok(),
        "conforming receipt must pass HIPAA gate: {:?}",
        result
    );
}

#[test]
fn hipaa_orphaned_event_fails_with_lineage_code() {
    let receipt = orphaned_event_receipt();
    let err = hipaa_gate(&receipt).expect_err("orphaned event must fail HIPAA gate");
    assert!(
        err.violations.iter().any(|v| v.code == "HIPAA-LINEAGE"),
        "expected HIPAA-LINEAGE violation, got: {:?}",
        err.violations
    );
}

// ---------------------------------------------------------------------------
// run_all_gates aggregate test
// ---------------------------------------------------------------------------

#[test]
fn run_all_gates_conforming_all_pass() {
    let receipt = conforming_receipt(3);
    let report = run_all_gates(&receipt);
    assert!(
        report.all_passed,
        "conforming receipt must pass all four gates: {report:?}"
    );
    assert!(report.soc2.passed);
    assert!(report.gdpr.passed);
    assert!(report.pci_dss.passed);
    assert!(report.hipaa.passed);
}

#[test]
fn run_all_gates_orphaned_event_fails_multiple() {
    let receipt = orphaned_event_receipt();
    let report = run_all_gates(&receipt);
    assert!(
        !report.all_passed,
        "orphaned event receipt must not pass all gates"
    );
    // SOC2, GDPR, and HIPAA all check for orphaned events.
    assert!(!report.soc2.passed, "SOC2 should fail");
    assert!(!report.gdpr.passed, "GDPR should fail");
    assert!(!report.hipaa.passed, "HIPAA should fail");
}
