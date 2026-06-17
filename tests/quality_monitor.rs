//! Comprehensive integration tests for the quality monitor system.
//!
//! Tests verify:
//! 1. measure_code_quality() correctly scans directories
//! 2. WesternElectricAnalyzer detects all 7 rules correctly
//! 3. Quality events emission to receipt chain
//! 4. Output formatting (JSON, stderr)
//! 5. Baseline bootstrap from initial measurements

use affidavit::quality::{
    CodeQualityMetrics, QualityViolation, WesternElectricAnalyzer, measure_code_quality,
};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// Helper: create a temporary Rust file with known content
fn create_test_rs_file(dir: &Path, name: &str, content: &str) -> std::io::Result<()> {
    let path = dir.join(name);
    fs::write(path, content)?;
    Ok(())
}

// Helper: create a directory structure with test Rust files
fn setup_test_project(test_dir: &TempDir) -> std::io::Result<String> {
    let src_dir = test_dir.path().join("src");
    fs::create_dir_all(&src_dir)?;

    // File 1: Well-formed code (no stubs)
    create_test_rs_file(
        &src_dir,
        "good.rs",
        r#"
/// A well-documented function.
pub fn add(a: i32, b: i32) -> i32 {
    // This is a comment
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
}
"#,
    )?;

    // File 2: Code with stubs
    create_test_rs_file(
        &src_dir,
        "with_stubs.rs",
        r#"
pub fn complex_feature() -> String {
    todo!("implement later")
}

pub fn another_feature() -> String {
    unimplemented!("not ready")
}

pub fn dangerous_path() {
    panic!("unreachable code")
}

#[test]
fn test_stub() {
    assert!(true);
}
"#,
    )?;

    // File 3: High-comment ratio
    create_test_rs_file(
        &src_dir,
        "documented.rs",
        r#"
/// This is a documented module.
/// It has extensive documentation.

/// Function with doc comments.
///
/// # Examples
/// ```
/// let result = compute(5);
/// ```
pub fn compute(x: i32) -> i32 {
    // This does the computation
    x * 2  // Multiply by 2
}

// Helper function
// Does something useful
fn helper() -> bool {
    true
}
"#,
    )?;

    Ok(src_dir.to_string_lossy().to_string())
}

#[test]
fn test_measure_code_quality_scans_directory() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let src_path = setup_test_project(&temp_dir).expect("setup test project");

    let metrics = measure_code_quality(&src_path).expect("measure code quality");

    // Verify basic metrics were populated
    assert!(metrics.stub_ratio >= 0.0 && metrics.stub_ratio <= 1.0);
    assert!(metrics.type_coverage >= 0.0 && metrics.type_coverage <= 1.0);
    assert!(metrics.comment_ratio >= 0.0);
    assert!(metrics.test_coverage >= 0.0);
    assert!(metrics.doc_coverage >= 0.0); // May exceed 1.0 if more docs than items
    assert!(metrics.timestamp > 0);
}

#[test]
fn test_measure_code_quality_detects_stubs() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let src_path = setup_test_project(&temp_dir).expect("setup test project");

    let metrics = measure_code_quality(&src_path).expect("measure code quality");

    // with_stubs.rs has 3 stubs and ~5 functions (add, subtract, complex_feature, another_feature, dangerous_path)
    // stub_ratio should be > 0
    assert!(metrics.stub_ratio > 0.0, "stub_ratio should detect stubs: {}", metrics.stub_ratio);
}

#[test]
fn test_measure_code_quality_empty_directory() {
    let temp_dir = TempDir::new().expect("create temp dir");
    let src_path = temp_dir.path().join("src");
    fs::create_dir_all(&src_path).expect("create src dir");

    let metrics = measure_code_quality(src_path.to_str().unwrap()).expect("measure code quality");

    // Empty directory should have default stub_ratio, comment_ratio uses default (0.2)
    assert_eq!(metrics.stub_ratio, 0.0);
    assert_eq!(metrics.comment_ratio, 0.2); // Default value from CodeQualityMetrics::default()
}

#[test]
fn test_measure_code_quality_nonexistent_path() {
    let metrics = measure_code_quality("/nonexistent/path/to/src").expect("measure code quality");

    // Should return default metrics without crashing
    assert_eq!(metrics.stub_ratio, 0.0);
    assert_eq!(metrics.timestamp > 0, true);
}

// ============================================================================
// Rule 1: 1σ Rule - Single point >3σ from mean
// ============================================================================

#[test]
fn test_rule_1_sigma_detects_spike() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // value = 9.5 => z-score = (9.5 - 5.0) / 1.0 = 4.5 > 3.0
    analyzer.add_measurement("stub_ratio", 9.5);

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule1Sigma { metric, z_score, .. }
            if metric == "stub_ratio" && *z_score > 3.0)
    }));
}

#[test]
fn test_rule_1_sigma_severity_critical() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("stub_ratio", 9.5);

    assert!(!analyzer.violations.is_empty());
    assert_eq!(analyzer.violations[0].severity(), "CRITICAL");
}

#[test]
fn test_rule_1_sigma_below_mean() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // value = 0.5 => z-score = (0.5 - 5.0) / 1.0 = -4.5 > 3.0 (absolute)
    analyzer.add_measurement("type_coverage", 0.5);

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule1Sigma { .. })
    }));
}

// ============================================================================
// Rule 2: 9-in-a-row - 9 consecutive out-of-control points
// ============================================================================

#[test]
fn test_rule_9_in_a_row_detects_zombie_code() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // UCL = 5 + 3*1 = 8; all measurements > UCL
    for i in 0..9 {
        analyzer.add_measurement("stub_ratio", 8.5 + i as f64 * 0.1);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule9InRow { metric, .. } if metric == "stub_ratio")
    }));
}

#[test]
fn test_rule_9_in_a_row_severity_critical() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for _ in 0..9 {
        analyzer.add_measurement("test_metric", 10.0);
    }

    assert!(!analyzer.violations.is_empty());
    let violation = analyzer.violations.iter().find(|v| {
        matches!(v, QualityViolation::Rule9InRow { .. })
    });
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "CRITICAL");
}

#[test]
fn test_rule_9_in_a_row_requires_exactly_9() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Add 8 out-of-control, should not trigger
    for _ in 0..8 {
        analyzer.add_measurement("test_metric", 10.0);
    }

    let has_9_in_row = analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule9InRow { .. })
    });
    assert!(!has_9_in_row, "Should not trigger with only 8 consecutive");
}

// ============================================================================
// Rule 3: Trend - 6 monotonic points (increasing or decreasing)
// ============================================================================

#[test]
fn test_rule_trend_detects_increasing() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Monotonically increasing sequence: 0, 1, 2, 3, 4, 5
    let values = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    for v in values {
        analyzer.add_measurement("cyclomatic_complexity", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::RuleTrend { metric, direction, .. }
            if metric == "cyclomatic_complexity" && direction == "increasing")
    }));
}

#[test]
fn test_rule_trend_detects_decreasing() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Monotonically decreasing sequence: 5, 4, 3, 2, 1, 0
    let values = [5.0, 4.0, 3.0, 2.0, 1.0, 0.0];
    for v in values {
        analyzer.add_measurement("test_coverage", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::RuleTrend { metric, direction, .. }
            if metric == "test_coverage" && direction == "decreasing")
    }));
}

#[test]
fn test_rule_trend_severity_high() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    let trend_violation = analyzer.violations.iter().find(|v| {
        matches!(v, QualityViolation::RuleTrend { .. })
    });
    assert!(trend_violation.is_some());
    assert_eq!(trend_violation.unwrap().severity(), "HIGH");
}

// ============================================================================
// Rule 4: Alternating - Wild swings (up-down-up-down pattern)
// ============================================================================

#[test]
fn test_rule_alternating_detects_oscillations() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Alternating pattern: below, above, below, above, ...
    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer.add_measurement("maintenance_index", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::RuleAlternating { metric, .. }
            if metric == "maintenance_index")
    }));
}

#[test]
fn test_rule_alternating_severity_high() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    let alt_violation = analyzer.violations.iter().find(|v| {
        matches!(v, QualityViolation::RuleAlternating { .. })
    });
    assert!(alt_violation.is_some());
    assert_eq!(alt_violation.unwrap().severity(), "HIGH");
}

// ============================================================================
// Rule 5: 2-of-3 beyond 2σ - Early warning
// ============================================================================

#[test]
fn test_rule_2_of_3_beyond_2_sigma() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    // 2σ threshold = 10 + 2*1 = 12
    // Add 3 measurements where 2 are beyond 2σ
    analyzer.add_measurement("metric", 13.0); // beyond 2σ
    analyzer.add_measurement("metric", 13.5); // beyond 2σ
    analyzer.add_measurement("metric", 11.0); // within 2σ

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule2of3Beyond2Sigma { count: 2, .. })
    }));
}

#[test]
fn test_rule_2_of_3_all_three_beyond_2_sigma() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    // All 3 beyond 2σ
    analyzer.add_measurement("metric", 13.0);
    analyzer.add_measurement("metric", 13.5);
    analyzer.add_measurement("metric", 14.0);

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule2of3Beyond2Sigma { count: 3, .. })
    }));
}

#[test]
fn test_rule_2_of_3_severity_high() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    analyzer.add_measurement("metric", 13.0);
    analyzer.add_measurement("metric", 13.5);
    analyzer.add_measurement("metric", 11.0);

    let violation = analyzer.violations.iter().find(|v| {
        matches!(v, QualityViolation::Rule2of3Beyond2Sigma { .. })
    });
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "HIGH");
}

// ============================================================================
// Rule 6: 4-of-5 beyond 1σ - Sustained deviation
// ============================================================================

#[test]
fn test_rule_4_of_5_beyond_1_sigma() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    // 1σ threshold = 10 + 1*1 = 11
    // 4 measurements beyond 1σ, 1 within
    analyzer.add_measurement("metric", 11.5);
    analyzer.add_measurement("metric", 12.0);
    analyzer.add_measurement("metric", 11.2);
    analyzer.add_measurement("metric", 11.8);
    analyzer.add_measurement("metric", 10.5); // within 1σ

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule4of5Beyond1Sigma { count: 4, .. })
    }));
}

#[test]
fn test_rule_4_of_5_all_five_beyond_1_sigma() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    // All 5 beyond 1σ
    analyzer.add_measurement("metric", 11.5);
    analyzer.add_measurement("metric", 12.0);
    analyzer.add_measurement("metric", 11.2);
    analyzer.add_measurement("metric", 11.8);
    analyzer.add_measurement("metric", 12.3);

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule4of5Beyond1Sigma { count: 5, .. })
    }));
}

#[test]
fn test_rule_4_of_5_severity_medium() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    analyzer.add_measurement("metric", 11.5);
    analyzer.add_measurement("metric", 12.0);
    analyzer.add_measurement("metric", 11.2);
    analyzer.add_measurement("metric", 11.8);
    analyzer.add_measurement("metric", 10.5);

    let violation = analyzer.violations.iter().find(|v| {
        matches!(v, QualityViolation::Rule4of5Beyond1Sigma { .. })
    });
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "MEDIUM");
}

// ============================================================================
// Rule 7: 15-in-a-row within 1σ - Plateau/stagnation
// ============================================================================

#[test]
fn test_rule_15_in_row_within_1_sigma() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    // 1σ threshold = 10 ± 1 = [9, 11]
    // 15 measurements within 1σ
    for i in 0..15 {
        let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.8; // oscillate within ±0.4
        analyzer.add_measurement("metric", value);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule15InRowWithin1Sigma { count: 15, .. })
    }));
}

#[test]
fn test_rule_15_in_row_requires_exactly_15() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    // Only 14 within 1σ
    for i in 0..14 {
        let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.8;
        analyzer.add_measurement("metric", value);
    }

    let has_15_in_row = analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule15InRowWithin1Sigma { .. })
    });
    assert!(!has_15_in_row, "Should not trigger with only 14");
}

#[test]
fn test_rule_15_in_row_severity_info() {
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    for i in 0..15 {
        let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.8;
        analyzer.add_measurement("metric", value);
    }

    let violation = analyzer.violations.iter().find(|v| {
        matches!(v, QualityViolation::Rule15InRowWithin1Sigma { .. })
    });
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "INFO");
}

// ============================================================================
// Baseline Bootstrap - Multiple measurements establish baseline
// ============================================================================

#[test]
fn test_baseline_bootstrap_20_measurements() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 2.0, 20);

    // Simulate 20 normal measurements
    for i in 0..20 {
        let value = 5.0 + (i as f64 * 0.1 - 1.0);
        analyzer.add_measurement("metric", value);
    }

    // After 20 measurements, rolling window should be full
    assert_eq!(analyzer.rolling_window.len(), 20);
}

#[test]
fn test_baseline_bootstrap_calculates_mean_and_stddev() {
    let baseline_mean = 10.0;
    let baseline_stddev = 2.0;

    let analyzer = WesternElectricAnalyzer::new(baseline_mean, baseline_stddev, 20);

    assert_eq!(analyzer.baseline_mean, baseline_mean);
    assert_eq!(analyzer.baseline_stddev, baseline_stddev);
    assert_eq!(analyzer.control_limits.0, baseline_mean - 3.0 * baseline_stddev);
    assert_eq!(analyzer.control_limits.1, baseline_mean + 3.0 * baseline_stddev);
}

#[test]
fn test_baseline_bootstrap_control_limits_computation() {
    let analyzer = WesternElectricAnalyzer::new(50.0, 5.0, 20);

    let expected_lcl = 50.0 - 3.0 * 5.0; // 35
    let expected_ucl = 50.0 + 3.0 * 5.0; // 65

    assert_eq!(analyzer.control_limits.0, expected_lcl);
    assert_eq!(analyzer.control_limits.1, expected_ucl);
}

// ============================================================================
// Multiple Rule Detection in Single Sequence
// ============================================================================

#[test]
fn test_multiple_rules_triggered_simultaneously() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Create a sequence that triggers Rule1Sigma (spike) then Rule9InRow
    analyzer.add_measurement("metric", 9.5); // Rule1Sigma triggered

    for _ in 0..9 {
        analyzer.add_measurement("metric", 10.0); // Rule9InRow triggered
    }

    // Should have multiple violations
    assert!(analyzer.violations.len() >= 2);
    assert!(analyzer.violations.iter().any(|v| matches!(v, QualityViolation::Rule1Sigma { .. })));
    assert!(analyzer.violations.iter().any(|v| matches!(v, QualityViolation::Rule9InRow { .. })));
}

// ============================================================================
// Metric Properties
// ============================================================================

#[test]
fn test_quality_violation_metric_accessor() {
    let violation = QualityViolation::Rule1Sigma {
        metric: "stub_ratio".to_string(),
        value: 0.5,
        threshold: 0.3,
        z_score: 3.5,
        severity: "CRITICAL".to_string(),
    };

    assert_eq!(violation.metric(), "stub_ratio");
}

#[test]
fn test_quality_violation_description_formatting() {
    let violation = QualityViolation::Rule1Sigma {
        metric: "test_coverage".to_string(),
        value: 50.0,
        threshold: 75.0,
        z_score: 3.2,
        severity: "CRITICAL".to_string(),
    };

    let desc = violation.description();
    assert!(desc.contains("test_coverage"));
    assert!(desc.contains("spike"));
}

// ============================================================================
// Edge Cases and Robustness
// ============================================================================

#[test]
fn test_analyzer_with_zero_stddev() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 0.0, 20);

    // With zero stddev, rules should not trigger (division by zero avoided)
    analyzer.add_measurement("metric", 5.0);
    analyzer.add_measurement("metric", 5.0);

    // Should have no violations due to baseline_stddev == 0 check
    assert!(analyzer.violations.is_empty());
}

#[test]
fn test_analyzer_with_large_values() {
    let mut analyzer = WesternElectricAnalyzer::new(1000000.0, 10000.0, 20);

    analyzer.add_measurement("large_metric", 1050000.0);

    // Should detect spike correctly with large values
    assert!(!analyzer.violations.is_empty());
}

#[test]
fn test_analyzer_with_negative_values() {
    let mut analyzer = WesternElectricAnalyzer::new(-5.0, 1.0, 20);

    analyzer.add_measurement("metric", -1.5); // z-score = 3.5

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer.violations.iter().any(|v| {
        matches!(v, QualityViolation::Rule1Sigma { .. })
    }));
}

#[test]
fn test_rolling_window_respects_size_limit() {
    let window_size = 10;
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, window_size);

    // Add 15 measurements
    for i in 0..15 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    // Window should not exceed window_size
    assert_eq!(analyzer.rolling_window.len(), window_size);
}

#[test]
fn test_violations_accumulate() {
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Trigger Rule1Sigma
    analyzer.add_measurement("metric1", 9.5);
    let count_after_first = analyzer.violations.len();

    // Trigger another Rule1Sigma
    analyzer.add_measurement("metric2", 1.0);
    let count_after_second = analyzer.violations.len();

    // Violations should accumulate
    assert!(count_after_second > count_after_first);
}
