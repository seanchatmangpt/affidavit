//! Extended Western Electric Statistical Process Control Example
//!
//! Demonstrates the comprehensive Western Electric analyzer with:
//! - Multiple sigma-level variants (1σ, 2σ, 3σ, custom)
//! - Variable window sizes (6, 9, 15, 20, 30 points)
//! - Rule combination detection (rule storms)
//! - Severity aggregation and multi-rule failure analysis

use affidavit::quality::QualityViolation;
use affidavit::quality_extended::{
    compute_aggregate_severity, detect_all_rule_variants, detect_rule_storms,
    EnhancedWesternElectricAnalyzer, RuleVariant, WesternElectricConfig,
};

fn main() {
    println!("=== Extended Western Electric SPC Example ===\n");

    // ========================================================================
    // Scenario 1: Detecting Sigma-Level Variants
    // ========================================================================
    println!("Scenario 1: Sigma-Level Variants (1σ, 2σ, 3σ)");
    println!("{}", "-".repeat(50));

    let config = WesternElectricConfig::new(10.0, 1.0);

    // Metric that crosses different sigma thresholds
    let metrics = vec![
        10.0, 10.2, 10.1, 10.5, 10.3, 11.2, // Within ±1σ
        11.8, 12.0, 12.1, 12.5, 10.0, 10.1, // Approaching 2σ
        13.0, 13.2, 13.5, 9.5, 9.0, // Crossing 3σ
    ];

    let variants = detect_all_rule_variants(&metrics, &config);
    println!("Detected {} rule variants:\n", variants.len());

    for variant in &variants {
        println!(
            "  [{}] {} (metric: {})",
            variant.severity(),
            variant.description(),
            variant.metric()
        );
    }

    // ========================================================================
    // Scenario 2: Window Size Variants
    // ========================================================================
    println!("\n\nScenario 2: Window Size Variants");
    println!("{}", "-".repeat(50));

    let mut config = WesternElectricConfig::new(10.0, 1.0);
    config.window_sizes = vec![6, 9, 15, 20, 30];

    let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

    // Sustained degradation: trend across 20 points
    println!("\nAdding 20 monotonically increasing measurements...");
    for i in 0..20 {
        let value = 10.0 + (i as f64 * 0.3); // Steady increase
        analyzer.add_measurement("complexity", value);
    }

    println!("\nDetected {} variants:", analyzer.detected_rules.len());
    for variant in &analyzer.detected_rules {
        println!("  - {}", variant.description());
    }

    // ========================================================================
    // Scenario 3: Rule Storms (Multiple Rules Firing)
    // ========================================================================
    println!("\n\nScenario 3: Rule Storms (2+ Rules Firing Simultaneously)");
    println!("{}", "-".repeat(50));

    let violations = vec![
        QualityViolation::Rule1Sigma {
            metric: "stub_ratio".to_string(),
            value: 0.85,
            threshold: 0.75,
            z_score: 3.2,
            severity: "CRITICAL".to_string(),
        },
        QualityViolation::Rule9InRow {
            metric: "stub_ratio".to_string(),
            consecutive: 9,
        },
        QualityViolation::RuleTrend {
            metric: "stub_ratio".to_string(),
            direction: "increasing".to_string(),
            count: 6,
        },
    ];

    let storms = detect_rule_storms(&violations);
    println!("\nDetected {} rule storm(s):\n", storms.len());
    for storm in &storms {
        println!(
            "Metric: {}\n\
             Rules Firing: {}\n\
             Aggregate Severity: {}\n\
             Is Severe (3+ rules): {}\n\
             Summary: {}\n",
            storm.metric,
            storm.rule_count,
            storm.aggregate_severity,
            storm.is_severe,
            storm.summary
        );
    }

    // ========================================================================
    // Scenario 4: Aggregated Severity Report
    // ========================================================================
    println!("\n\nScenario 4: Aggregated Severity Report");
    println!("{}", "-".repeat(50));

    let violations = vec![
        QualityViolation::Rule1Sigma {
            metric: "clippy_warnings".to_string(),
            value: 42.0,
            threshold: 30.0,
            z_score: 4.0,
            severity: "CRITICAL".to_string(),
        },
        QualityViolation::Rule9InRow {
            metric: "maintainability".to_string(),
            consecutive: 10,
        },
        QualityViolation::RuleTrend {
            metric: "type_coverage".to_string(),
            direction: "decreasing".to_string(),
            count: 8,
        },
        QualityViolation::Rule4of5Beyond1Sigma {
            metric: "doc_coverage".to_string(),
            count: 4,
            threshold: 0.7,
        },
    ];

    let severity = compute_aggregate_severity(&violations);
    println!(
        "Total Violations: {}\n\
         Critical: {}\n\
         High: {}\n\
         Medium: {}\n\
         Low: {}\n\
         Affected Metrics: {}\n\
         Worst Severity: {}\n\
         Summary: {}",
        severity.total_violations,
        severity.critical_count,
        severity.high_count,
        severity.medium_count,
        severity.low_count,
        severity.affected_metrics.join(", "),
        severity.worst_severity,
        severity.summary
    );

    // ========================================================================
    // Scenario 5: Custom Configuration with Fine-Grained Control
    // ========================================================================
    println!("\n\nScenario 5: Custom Configuration");
    println!("{}", "-".repeat(50));

    let config = WesternElectricConfig::new(50.0, 5.0)
        .with_sigmas(1.5, 2.5, 3.5)
        .add_custom_threshold("rule1_aggressive".to_string(), 1.2)
        .with_enabled_rules(true, true, true, false, true, true, false);

    println!(
        "Custom Config:\n\
         - Baseline Mean: {}\n\
         - Baseline StdDev: {}\n\
         - Primary Sigma: {} (1σ spike detection)\n\
         - Secondary Sigma: {} (2-of-3 detection)\n\
         - Tertiary Sigma: {} (sustained deviation)\n\
         - Rule Storm Threshold: {}\n\
         - Rule 1 Enabled: {}\n\
         - Rule 4 (Alternating) Disabled: {}",
        config.baseline_mean,
        config.baseline_stddev,
        config.primary_sigma,
        config.secondary_sigma,
        config.tertiary_sigma,
        config.rule_storm_threshold,
        config.rule1_enabled,
        !config.rule4_enabled
    );

    // ========================================================================
    // Scenario 6: Real-World Code Quality Monitoring
    // ========================================================================
    println!("\n\nScenario 6: Real-World Code Quality Monitoring");
    println!("{}", "-".repeat(50));

    // Simulating a metric like cyclomatic complexity over commits
    let mut config = WesternElectricConfig::new(8.0, 2.0);
    config.window_sizes = vec![6, 9, 15, 20];
    let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

    let complexity_history = vec![
        7.5, 8.1, 7.8, 9.2, 8.5, 8.3, 9.1, 10.2, 11.5, 12.3, 13.1, 14.0, 14.2, 13.9,
    ];

    println!("\nCyclomatic Complexity History (baseline=8.0, σ=2.0):");
    for (i, &value) in complexity_history.iter().enumerate() {
        let z_score = (value - 8.0) / 2.0;
        println!(
            "  Commit {}: value={:.1}, z-score={:.2}",
            i + 1,
            value,
            z_score
        );
        analyzer.add_measurement("cyclomatic_complexity", value);
    }

    println!("\nDetected Issues:");
    if analyzer.detected_rules.is_empty() {
        println!("  No issues detected");
    } else {
        for variant in &analyzer.detected_rules {
            println!("  [{}] {}", variant.severity(), variant.description());
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n\n=== Summary ===");
    println!("{}", "-".repeat(50));
    println!(
        "Extended Western Electric analyzer provides:\n\
         ✓ Sigma-level variants (1σ, 2σ, 3σ, custom)\n\
         ✓ Window size variants (6, 9, 15, 20, 30 points)\n\
         ✓ Rule combination detection (rule storms)\n\
         ✓ Severity aggregation and multi-rule failure analysis\n\
         ✓ Fine-grained configuration options\n\
         ✓ Production-ready quality monitoring"
    );
}
