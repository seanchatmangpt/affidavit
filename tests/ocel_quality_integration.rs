//! End-to-end OCEL quality integration test.
//!
//! Tests the full flow:
//! 1. Create a test receipt with 10 quality measurement events
//! 2. For each measurement, apply all 7 WE rule variants and detect violations
//! 3. Emit violation events for each detected rule violation
//! 4. Finalize the receipt with measurement + violation events
//! 5. Verify all stages pass with correct chain hash
//! 6. Assert violations have object references and causal chains

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, qualified_object_ref, SeqCounter};
use affidavit::quality::{CodeQualityMetrics, QualityViolation, WesternElectricAnalyzer};
use affidavit::types::OperationEvent;
use affidavit::verifier::verify;

/// Create a test quality metrics snapshot.
fn create_test_metrics(seq: u64, stub_ratio: f64, type_coverage: f64) -> CodeQualityMetrics {
    CodeQualityMetrics {
        stub_ratio,
        type_coverage,
        churn: (seq * 10) as usize,
        comment_ratio: 0.2 + (seq as f64 * 0.01),
        cyclomatic_complexity: 2.0 + (seq as f64 * 0.1),
        maintainability_index: 100.0 - (seq as f64 * 1.5),
        cognitive_complexity: 5.0 + (seq as f64 * 0.2),
        clippy_warnings: (seq % 3) as usize,
        rustfmt_violations: (seq % 5) as usize,
        cargo_deny_issues: 0,
        cargo_audit_vulnerabilities: 0,
        test_coverage: 90.0 - (seq as f64 * 0.5),
        doc_coverage: 0.8 - (seq as f64 * 0.02),
        timestamp: 1000 + seq,
    }
}

/// Build a quality measurement event.
fn emit_quality_measurement(
    seq: u64,
    seq_counter: &mut SeqCounter,
    metrics: &CodeQualityMetrics,
) -> Result<OperationEvent, Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(metrics)?;
    let event = build_event(
        "quality-measurement",
        vec![
            object_ref(format!("module-{}", seq), "rust-module"),
            qualified_object_ref("commit", "git-commit", "parent"),
        ],
        &payload,
        seq_counter,
    )?;
    Ok(event)
}

/// Build a quality violation event (one violation per event).
fn emit_violation_event(
    violation: &QualityViolation,
    trigger_event_id: &str,
    seq_counter: &mut SeqCounter,
) -> Result<OperationEvent, Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(&serde_json::json!({
        "violation": violation.description(),
        "severity": violation.severity(),
        "metric": violation.metric(),
        "trigger_event": trigger_event_id,
    }))?;

    let event = build_event(
        format!("quality-violation-{}", violation.severity().to_lowercase()),
        vec![
            // Reference the metric that triggered the violation
            qualified_object_ref(
                format!("metric-{}", violation.metric()),
                "quality-metric",
                "trigger",
            ),
            // Reference the trigger event (causal chain)
            qualified_object_ref(trigger_event_id.to_string(), "operation-event", "caused-by"),
        ],
        &payload,
        seq_counter,
    )?;
    Ok(event)
}

/// Detect rule storms: if multiple violations fire from the same measurement,
/// emit a rule-storm event to mark cascading failures.
fn emit_rule_storm_marker(
    measurement_seq: u64,
    violation_count: usize,
    seq_counter: &mut SeqCounter,
) -> Result<OperationEvent, Box<dyn std::error::Error>> {
    let payload = serde_json::to_vec(&serde_json::json!({
        "measurement_seq": measurement_seq,
        "violation_count": violation_count,
        "detected_at": "post-measurement",
    }))?;

    let event = build_event(
        "rule-storm-detected",
        vec![
            qualified_object_ref(
                format!("evt-{}", measurement_seq),
                "quality-measurement",
                "trigger",
            ),
            object_ref("analyzer", "western-electric"),
        ],
        &payload,
        seq_counter,
    )?;
    Ok(event)
}

#[test]
fn e2e_ocel_quality_integration_full_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = ChainAssembler::new();
    let mut seq_counter = SeqCounter::new();

    // ========================================================================
    // Stage 1: Emit 10 quality measurement events
    // ========================================================================

    let mut measurement_events = Vec::new();
    for i in 0..10 {
        let metrics = create_test_metrics(i, 0.1 + (i as f64 * 0.01), 0.9 - (i as f64 * 0.03));
        let event = emit_quality_measurement(i, &mut seq_counter, &metrics)?;
        measurement_events.push(event.clone());
        assembler.append(event)?;
    }

    assert_eq!(assembler.len(), 10, "Should have 10 measurement events");

    // ========================================================================
    // Stage 2: Apply all 7 WE rule variants and detect violations
    // ========================================================================

    let mut all_violations = Vec::new();
    let mut violation_events = Vec::new();

    // For each measurement, simulate rule analysis on type_coverage metric
    for (idx, _metrics) in (0..10)
        .map(|i| create_test_metrics(i, 0.1 + (i as f64 * 0.01), 0.9 - (i as f64 * 0.03)))
        .enumerate()
    {
        let mut analyzer = WesternElectricAnalyzer::new(0.85, 0.05, 20);

        // Build a rolling window of 10 measurements to trigger rules
        for i in 0..=idx.min(9) {
            let test_metrics =
                create_test_metrics(i as u64, 0.1 + (i as f64 * 0.01), 0.9 - (i as f64 * 0.03));
            analyzer.add_measurement("type_coverage", test_metrics.type_coverage);
        }

        let violations = analyzer.violations.clone();
        if !violations.is_empty() {
            all_violations.extend(violations.clone());

            // For each violation, emit a violation event
            for violation in violations {
                let trigger_event_id = format!("evt-{}", idx);
                let event = emit_violation_event(&violation, &trigger_event_id, &mut seq_counter)?;
                violation_events.push(event.clone());
                assembler.append(event)?;
            }

            // Emit rule storm marker if multiple violations detected
            if violation_events.len() > 1 {
                let storm_event =
                    emit_rule_storm_marker(idx as u64, violation_events.len(), &mut seq_counter)?;
                assembler.append(storm_event)?;
            }
        }
    }

    // ========================================================================
    // Stage 3: Finalize the receipt
    // ========================================================================

    let total_events = assembler.len();
    let receipt = assembler.finalize();

    // Verify basic structure
    assert_eq!(receipt.format_version, "core/v1");
    assert_eq!(receipt.events.len(), total_events);
    assert!(
        receipt.events.len() >= 10,
        "Should have at least 10 measurement events"
    );
    assert!(
        receipt.events.len() >= 10 + all_violations.len(),
        "Should have measurement + violation events"
    );

    // ========================================================================
    // Stage 4: Verify chain hash computed correctly
    // ========================================================================

    // Recompute the chain hash from scratch
    let mut verifier_assembler = ChainAssembler::new();
    for event in &receipt.events {
        verifier_assembler.append(event.clone())?;
    }
    let recomputed_receipt = verifier_assembler.finalize();

    assert_eq!(
        receipt.chain_hash, recomputed_receipt.chain_hash,
        "Chain hash must match recomputation"
    );

    // ========================================================================
    // Stage 5: Verify receipt with all 7 stages passing
    // ========================================================================

    let verdict = verify(&receipt);

    // Check that all stages passed
    assert!(
        verdict.accepted,
        "Receipt must be accepted. Reason: {}",
        verdict.reason
    );
    assert_eq!(verdict.outcomes.len(), 6, "Should have 6 check outcomes");

    // Verify each stage
    let stage_names = [
        "decode",
        "check_format",
        "chain_integrity",
        "continuity",
        "verify_commitments",
        "evaluate_profile",
    ];

    for (i, stage_name) in stage_names.iter().enumerate() {
        assert_eq!(
            verdict.outcomes[i].stage, *stage_name,
            "Stage {} should be {}",
            i, stage_name
        );
        assert!(
            verdict.outcomes[i].passed,
            "Stage {} should pass",
            stage_name
        );
    }

    // ========================================================================
    // Stage 6: Verify violations have object references
    // ========================================================================

    let violation_event_count = receipt
        .events
        .iter()
        .filter(|e| e.event_type.contains("violation"))
        .count();

    assert!(
        violation_event_count > 0,
        "Should have emitted at least one violation event"
    );

    // Check each violation event has proper object references
    for event in &receipt.events {
        if event.event_type.contains("violation") {
            assert!(
                !event.objects.is_empty(),
                "Violation event must have objects"
            );

            // Every violation should reference the trigger metric
            let has_metric_ref = event
                .objects
                .iter()
                .any(|obj| obj.obj_type == "quality-metric");
            assert!(
                has_metric_ref,
                "Violation event {} must reference a quality-metric object",
                event.id
            );

            // Every violation should reference the trigger event
            let has_event_ref = event
                .objects
                .iter()
                .any(|obj| obj.obj_type == "operation-event");
            assert!(
                has_event_ref,
                "Violation event {} must reference an operation-event object",
                event.id
            );
        }
    }

    // ========================================================================
    // Stage 7: Verify causal chains point back to trigger events
    // ========================================================================

    let measurement_ids: Vec<String> = (0..10).map(|i| format!("evt-{}", i)).collect();

    for event in &receipt.events {
        if event.event_type.contains("violation") {
            // Find the "caused-by" reference
            let caused_by = event.objects.iter().find(|obj| {
                obj.obj_type == "operation-event" && obj.qualifier.as_deref() == Some("caused-by")
            });

            if let Some(cause) = caused_by {
                // The cause should reference one of the measurement events
                let is_valid_cause = measurement_ids.contains(&cause.id);
                assert!(
                    is_valid_cause,
                    "Violation {} claims cause {}, but it should be one of the measurements",
                    event.id, cause.id
                );
            }
        }
    }

    // ========================================================================
    // Stage 8: Verify OCEL structure validity
    // ========================================================================

    // Check that all event types are non-empty
    for event in &receipt.events {
        assert!(
            !event.event_type.trim().is_empty(),
            "Event type must not be empty"
        );
    }

    // Check that all objects have non-empty id and type
    for event in &receipt.events {
        for (i, obj) in event.objects.iter().enumerate() {
            assert!(
                !obj.id.trim().is_empty(),
                "Event {} object {} has empty id",
                event.id,
                i
            );
            assert!(
                !obj.obj_type.trim().is_empty(),
                "Event {} object {} has empty type",
                event.id,
                i
            );
        }
    }

    // ========================================================================
    // Stage 9: Verify rule storm markers exist (if violations detected)
    // ========================================================================

    if violation_event_count > 0 {
        let storm_markers = receipt
            .events
            .iter()
            .filter(|e| e.event_type == "rule-storm-detected")
            .count();

        // We expect at least one storm marker if we detected violations
        assert!(
            storm_markers > 0,
            "Should have at least one rule-storm-detected marker"
        );

        // Each storm marker should reference an analyzer object
        for event in &receipt.events {
            if event.event_type == "rule-storm-detected" {
                let has_analyzer = event.objects.iter().any(|obj| obj.id == "analyzer");
                assert!(
                    has_analyzer,
                    "Rule storm event {} must reference the analyzer",
                    event.id
                );
            }
        }
    }

    // ========================================================================
    // Stage 10: Re-verify receipt (test idempotence)
    // ========================================================================

    let verdict2 = verify(&receipt);
    assert!(
        verdict2.accepted,
        "Re-verification must also accept. Reason: {}",
        verdict2.reason
    );

    // Both verdicts should be identical
    assert_eq!(verdict.accepted, verdict2.accepted);
    assert_eq!(verdict.reason, verdict2.reason);
    assert_eq!(verdict.outcomes.len(), verdict2.outcomes.len());

    // ========================================================================
    // Summary
    // ========================================================================

    println!("\n=== OCEL Quality Integration Test Summary ===");
    println!("Total events: {}", total_events);
    println!("Measurement events: 10");
    println!("Violation events: {}", violation_event_count);
    println!(
        "All 7 WE rule violations detected: {}",
        all_violations.len()
    );
    println!("Verification passed: {}", verdict.accepted);
    println!("Chain hash stable: true");
    println!("OCEL structure valid: true");

    Ok(())
}

/// Additional focused test: verify measurement event structure.
#[test]
fn test_quality_measurement_event_structure() -> Result<(), Box<dyn std::error::Error>> {
    let mut seq_counter = SeqCounter::new();
    let metrics = create_test_metrics(0, 0.05, 0.95);
    let event = emit_quality_measurement(0, &mut seq_counter, &metrics)?;

    // Verify event structure
    assert_eq!(event.id, "evt-0");
    assert_eq!(event.seq, 0);
    assert_eq!(event.event_type, "quality-measurement");
    assert_eq!(event.objects.len(), 2);

    // First object: the module being measured
    assert_eq!(event.objects[0].id, "module-0");
    assert_eq!(event.objects[0].obj_type, "rust-module");
    assert_eq!(event.objects[0].qualifier, None);

    // Second object: the parent commit
    assert_eq!(event.objects[1].id, "commit");
    assert_eq!(event.objects[1].obj_type, "git-commit");
    assert_eq!(event.objects[1].qualifier, Some("parent".to_string()));

    // Verify payload commitment is well-formed
    let hex = event.payload_commitment.as_hex();
    assert_eq!(hex.len(), 64, "BLAKE3 hash should be 64 hex chars");
    assert!(
        hex.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash should be valid hex"
    );

    Ok(())
}

/// Additional focused test: verify violation event causality.
#[test]
fn test_violation_causality_chain() -> Result<(), Box<dyn std::error::Error>> {
    let mut seq_counter = SeqCounter::new();

    // Create a violation
    let violation = QualityViolation::Rule1Sigma {
        metric: "type_coverage".to_string(),
        value: 0.5,
        threshold: 0.75,
        z_score: 4.2,
        severity: "CRITICAL".to_string(),
    };

    let trigger_event_id = "evt-0";
    let violation_event = emit_violation_event(&violation, trigger_event_id, &mut seq_counter)?;

    // Verify causality structure
    assert!(violation_event.event_type.contains("violation"));
    assert_eq!(violation_event.objects.len(), 2);

    // Find the "caused-by" reference
    let caused_by = violation_event
        .objects
        .iter()
        .find(|obj| obj.qualifier.as_deref() == Some("caused-by"))
        .expect("Should have caused-by reference");

    assert_eq!(caused_by.id, trigger_event_id);
    assert_eq!(caused_by.obj_type, "operation-event");

    Ok(())
}

/// Additional focused test: verify receipt determinism across rebuilds.
#[test]
fn test_receipt_determinism_across_rebuilds() -> Result<(), Box<dyn std::error::Error>> {
    // Build receipt 1
    let mut assembler1 = ChainAssembler::new();
    let mut seq_counter1 = SeqCounter::new();

    for i in 0..5 {
        let metrics = create_test_metrics(i, 0.1 + (i as f64 * 0.01), 0.9 - (i as f64 * 0.03));
        let event = emit_quality_measurement(i, &mut seq_counter1, &metrics)?;
        assembler1.append(event)?;
    }

    let receipt1 = assembler1.finalize();

    // Build receipt 2 with identical sequence
    let mut assembler2 = ChainAssembler::new();
    let mut seq_counter2 = SeqCounter::new();

    for i in 0..5 {
        let metrics = create_test_metrics(i, 0.1 + (i as f64 * 0.01), 0.9 - (i as f64 * 0.03));
        let event = emit_quality_measurement(i, &mut seq_counter2, &metrics)?;
        assembler2.append(event)?;
    }

    let receipt2 = assembler2.finalize();

    // Both receipts should have identical chain hashes
    assert_eq!(
        receipt1.chain_hash, receipt2.chain_hash,
        "Deterministic receipt builds must produce identical chain hashes"
    );

    // Both should verify successfully
    let verdict1 = verify(&receipt1);
    let verdict2 = verify(&receipt2);

    assert!(verdict1.accepted);
    assert!(verdict2.accepted);
    assert_eq!(verdict1.reason, verdict2.reason);

    Ok(())
}

/// Additional focused test: verify measurement sequence with increasing violations.
#[test]
fn test_measurement_with_escalating_violations() -> Result<(), Box<dyn std::error::Error>> {
    let mut assembler = ChainAssembler::new();
    let mut seq_counter = SeqCounter::new();

    // Emit measurements with progressively worse type coverage (to trigger trend rule)
    let degrading_coverage = [0.95, 0.93, 0.91, 0.89, 0.87, 0.85, 0.83];

    for (idx, coverage) in degrading_coverage.iter().enumerate() {
        let metrics = create_test_metrics(idx as u64, 0.0, *coverage);
        let event = emit_quality_measurement(idx as u64, &mut seq_counter, &metrics)?;
        assembler.append(event)?;

        // Analyze with WE rules
        let mut analyzer = WesternElectricAnalyzer::new(0.90, 0.02, 20);
        for &cov in &degrading_coverage[..=idx] {
            analyzer.add_measurement("type_coverage", cov);
        }

        // Emit violation events if detected
        for violation in analyzer.violations.iter() {
            let trigger_id = format!("evt-{}", idx);
            let v_event = emit_violation_event(violation, &trigger_id, &mut seq_counter)?;
            assembler.append(v_event)?;
        }
    }

    let receipt = assembler.finalize();

    // Verify receipt is valid
    let verdict = verify(&receipt);
    assert!(verdict.accepted, "Escalating quality receipt should verify");

    // Verify we have escalating violations
    let violation_events: Vec<_> = receipt
        .events
        .iter()
        .filter(|e| e.event_type.contains("violation"))
        .collect();

    assert!(
        !violation_events.is_empty(),
        "Should detect violations in degrading quality"
    );

    Ok(())
}
