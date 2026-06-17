//! Standalone example demonstrating the quality monitor feature.
//!
//! This example shows how to:
//! 1. Load the affidavit codebase (use "." as path)
//! 2. Measure code quality metrics
//! 3. Set up a Western Electric analyzer for statistical process control
//! 4. Add synthetic measurements to populate the rolling window
//! 5. Detect violations using all 7 Western Electric rules
//! 6. Emit quality measurement and violation events to the receipt chain
//! 7. Display violations with severity levels and descriptions
//!
//! This example deliberately creates violations by varying measurements to trigger
//! different rule violations (trend, spike, alternating, etc.).

use affidavit::chain::ChainAssembler;
use affidavit::ocel::{build_event, object_ref, SeqCounter};
use affidavit::quality::{WesternElectricAnalyzer, QualityViolation};

fn main() -> anyhow::Result<()> {
    eprintln!("=== Quality Monitor Example ===\n");

    // Step 1: Measure code quality from the affidavit repo itself
    eprintln!("Step 1: Measuring code quality from 'src' directory...");
    let metrics = affidavit::quality::measure_code_quality("src")?;
    eprintln!(
        "  stub_ratio:            {:.4}",
        metrics.stub_ratio
    );
    eprintln!(
        "  cyclomatic_complexity: {:.4}",
        metrics.cyclomatic_complexity
    );
    eprintln!(
        "  clippy_warnings:       {}",
        metrics.clippy_warnings
    );
    eprintln!("  churn:                 {}", metrics.churn);
    eprintln!("  test_coverage:         {:.1}%\n", metrics.test_coverage);

    // Step 2: Create a Western Electric analyzer with reasonable baseline
    // For stub_ratio: mean=0.1 (10% of functions are stubs), σ=0.05
    eprintln!("Step 2: Creating Western Electric analyzer...");
    let baseline_mean = 0.1;
    let baseline_stddev = 0.05;
    let window_size = 20;

    let mut analyzer = WesternElectricAnalyzer::new(
        baseline_mean,
        baseline_stddev,
        window_size,
    );
    eprintln!(
        "  baseline_mean:     {:.4}",
        analyzer.baseline_mean
    );
    eprintln!(
        "  baseline_stddev:   {:.4}",
        analyzer.baseline_stddev
    );
    eprintln!(
        "  control_limits:    ({:.4}, {:.4})\n",
        analyzer.control_limits.0, analyzer.control_limits.1
    );

    // Step 3: Add 10-15 synthetic measurements to populate rolling window
    // These are deliberately varied to trigger different rule violations
    eprintln!("Step 3: Adding synthetic measurements to rolling window...");
    let synthetic_measurements = vec![
        0.08,  // 0: normal, within 1σ
        0.09,  // 1: normal
        0.11,  // 2: normal
        0.10,  // 3: normal
        0.10,  // 4: normal
        0.11,  // 5: increasing trend start
        0.13,  // 6: increasing trend
        0.15,  // 7: increasing trend
        0.17,  // 8: increasing trend
        0.19,  // 9: increasing trend (triggers 6-point trend rule)
        0.21,  // 10: high value, beyond 2σ
        0.08,  // 11: alternating swing down
        0.20,  // 12: alternating swing up
        0.09,  // 13: alternating swing down
        0.22,  // 14: spike, beyond 3σ (triggers Rule 1 spike)
    ];

    for (i, &value) in synthetic_measurements.iter().enumerate() {
        analyzer.add_measurement("stub_ratio", value);
        eprintln!("  measurement[{}] = {:.4}", i, value);
    }
    eprintln!();

    // Step 4: Collect violations
    eprintln!("Step 4: Analyzing violations...");
    eprintln!(
        "Total violations detected: {}\n",
        analyzer.violations.len()
    );

    if !analyzer.violations.is_empty() {
        eprintln!("Violations by severity:");
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut info_count = 0;

        for violation in &analyzer.violations {
            let severity = violation.severity();
            match severity {
                "CRITICAL" => critical_count += 1,
                "HIGH" => high_count += 1,
                "MEDIUM" => medium_count += 1,
                "INFO" => info_count += 1,
                _ => {}
            }
        }

        if critical_count > 0 {
            eprintln!("  CRITICAL: {}", critical_count);
        }
        if high_count > 0 {
            eprintln!("  HIGH:     {}", high_count);
        }
        if medium_count > 0 {
            eprintln!("  MEDIUM:   {}", medium_count);
        }
        if info_count > 0 {
            eprintln!("  INFO:     {}", info_count);
        }
        eprintln!();

        eprintln!("Detailed violations:");
        for (i, violation) in analyzer.violations.iter().enumerate() {
            eprintln!(
                "  [{}] [{}] {}: {}",
                i + 1,
                violation.severity(),
                violation.metric(),
                violation.description()
            );
        }
    } else {
        eprintln!("No violations detected (all measurements within control limits)");
    }
    eprintln!();

    // Step 5: Emit quality measurement event to receipt chain
    eprintln!("Step 5: Emitting events to receipt chain...");
    let mut counter = SeqCounter::new();
    let mut assembler = ChainAssembler::new();

    // Event 1: Quality measurement snapshot
    let measurement_payload = serde_json::json!({
        "timestamp": metrics.timestamp,
        "stub_ratio": metrics.stub_ratio,
        "cyclomatic_complexity": metrics.cyclomatic_complexity,
        "clippy_warnings": metrics.clippy_warnings,
        "churn": metrics.churn,
        "test_coverage": metrics.test_coverage,
        "doc_coverage": metrics.doc_coverage,
        "maintainability_index": metrics.maintainability_index,
    }).to_string();

    let measurement_event = build_event(
        "quality.measurement",
        vec![object_ref("codebase", "src")],
        measurement_payload.as_bytes(),
        &mut counter,
    )?;
    assembler.append(measurement_event)?;
    eprintln!("  emitted: quality.measurement");

    // Event 2+: Quality violation events
    for (i, violation) in analyzer.violations.iter().enumerate() {
        let violation_payload = serde_json::json!({
            "violation_id": i,
            "metric": violation.metric(),
            "severity": violation.severity(),
            "description": violation.description(),
            "rule": match violation {
                QualityViolation::Rule1Sigma { .. } => "rule_1_sigma",
                QualityViolation::Rule9InRow { .. } => "rule_9_in_a_row",
                QualityViolation::RuleTrend { .. } => "rule_trend",
                QualityViolation::RuleAlternating { .. } => "rule_alternating",
                QualityViolation::Rule2of3Beyond2Sigma { .. } => "rule_2_of_3_beyond_2sigma",
                QualityViolation::Rule4of5Beyond1Sigma { .. } => "rule_4_of_5_beyond_1sigma",
                QualityViolation::Rule15InRowWithin1Sigma { .. } => "rule_15_in_a_row_within_1sigma",
            },
        }).to_string();

        let violation_event = build_event(
            "quality.violation",
            vec![object_ref("metric", violation.metric())],
            violation_payload.as_bytes(),
            &mut counter,
        )?;
        assembler.append(violation_event)?;
        eprintln!("  emitted: quality.violation ({})", violation.metric());
    }
    eprintln!();

    // Step 6: Finalize and display receipt
    eprintln!("Step 6: Finalizing receipt...");
    let receipt = assembler.finalize();
    eprintln!("  chain_hash: {}", receipt.chain_hash.as_hex());
    eprintln!("  total events: {}\n", receipt.events.len());

    // Step 7: Display JSON output format
    eprintln!("Step 7: JSON output format:");
    let json_output = serde_json::json!({
        "monitor_session": "swarm_example",
        "baseline": {
            "mean": baseline_mean,
            "stddev": baseline_stddev,
        },
        "measurements_count": synthetic_measurements.len(),
        "violations_count": analyzer.violations.len(),
        "violations": analyzer.violations.iter().map(|v| {
            serde_json::json!({
                "metric": v.metric(),
                "severity": v.severity(),
                "description": v.description(),
            })
        }).collect::<Vec<_>>(),
        "receipt": {
            "chain_hash": receipt.chain_hash.as_hex(),
            "events_count": receipt.events.len(),
        }
    });

    eprintln!("{}", serde_json::to_string_pretty(&json_output)?);
    eprintln!();

    eprintln!("=== Example Complete ===");
    eprintln!("To run this example:");
    eprintln!("  cargo run --example swarm_monitor");
    eprintln!();
    eprintln!("To use the monitor in production:");
    eprintln!("  affi receipt monitor --watch . --metrics all --rules all --output stderr,json");

    Ok(())
}
