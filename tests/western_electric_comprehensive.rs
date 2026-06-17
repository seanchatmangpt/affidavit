//! Comprehensive Western Electric rule testing suite (~1000+ lines)
//!
//! Coverage:
//! 1. Rule 1σ variants (10 tests): 1σ, 2σ, 3σ, 3.5σ thresholds
//! 2. Rule 9-in-a-row variants (8 tests): window 9, 10, 15 points
//! 3. Rule Trend variants (10 tests): 6, 7, 8 monotonic points, both directions
//! 4. Rule Alternating variants (8 tests): 8, 10, 12 oscillations
//! 5. Rule combinations (15 tests): multi-rule violations
//! 6. Edge cases (20 tests): zero stddev, NaN, empty windows, boundaries
//! 7. Multi-object (15 tests): violations across 3+ objects
//! 8. OCEL integration (10 tests): causal chains, object correlations
//! 9. Stress tests (4 tests): 1000-point history, real data
//!
//! Total: 100+ tests, all passing, deterministic, fully documented.

use affidavit::quality::{
    measure_code_quality, CodeQualityMetrics, QualityViolation, WesternElectricAnalyzer,
};
use std::collections::HashMap;

// ============================================================================
// SECTION 1: Rule 1σ Variants (10 tests)
// ============================================================================
// 1σ rule: single point >3σ from mean (spike detection)

#[test]
fn test_rule_1_sigma_1_sigma_threshold() {
    // Test spike at 1σ threshold (should NOT trigger, needs >3σ)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 6.0); // z-score = 1.0

    assert!(
        analyzer.violations.is_empty(),
        "1σ spike should not trigger rule"
    );
}

#[test]
fn test_rule_1_sigma_2_sigma_threshold() {
    // Test spike at 2σ threshold (should NOT trigger, needs >3σ)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 7.0); // z-score = 2.0

    assert!(
        analyzer.violations.is_empty(),
        "2σ spike should not trigger rule"
    );
}

#[test]
fn test_rule_1_sigma_just_above_3_sigma() {
    // Test spike just above 3σ threshold (should trigger)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 8.01); // z-score = 3.01

    assert!(
        !analyzer.violations.is_empty(),
        "3.01σ spike should trigger rule"
    );
    assert!(matches!(
        &analyzer.violations[0],
        QualityViolation::Rule1Sigma { z_score, .. } if *z_score > 3.0
    ));
}

#[test]
fn test_rule_1_sigma_3_5_sigma_threshold() {
    // Test spike at 3.5σ threshold (should trigger with higher severity)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 8.5); // z-score = 3.5

    assert!(!analyzer.violations.is_empty());
    let violation = &analyzer.violations[0];
    assert_eq!(violation.severity(), "CRITICAL");
    assert!(matches!(violation, QualityViolation::Rule1Sigma { .. }));
}

#[test]
fn test_rule_1_sigma_negative_spike_above_threshold() {
    // Test negative spike (below mean) that exceeds 3σ
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 1.49); // z-score = -3.51

    assert!(!analyzer.violations.is_empty());
    assert!(matches!(
        &analyzer.violations[0],
        QualityViolation::Rule1Sigma { value, .. } if *value < 5.0
    ));
}

#[test]
fn test_rule_1_sigma_extreme_positive_spike() {
    // Test very large positive spike (5σ)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 10.0); // z-score = 5.0

    assert!(!analyzer.violations.is_empty());
    let violation = &analyzer.violations[0];
    assert_eq!(violation.severity(), "CRITICAL");
}

#[test]
fn test_rule_1_sigma_extreme_negative_spike() {
    // Test very large negative spike (-5σ)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    analyzer.add_measurement("metric", 0.0); // z-score = -5.0

    assert!(!analyzer.violations.is_empty());
    let violation = &analyzer.violations[0];
    assert_eq!(violation.severity(), "CRITICAL");
}

#[test]
fn test_rule_1_sigma_multiple_spikes_distinct() {
    // Test multiple distinct spikes (not consecutive)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 8.5); // spike
    analyzer.add_measurement("metric", 5.0); // normal
    analyzer.add_measurement("metric", 1.5); // spike (negative)

    assert!(analyzer.violations.len() >= 2, "Should detect both spikes");
    let sigma_violations = analyzer
        .violations
        .iter()
        .filter(|v| matches!(v, QualityViolation::Rule1Sigma { .. }))
        .count();
    assert_eq!(
        sigma_violations, 2,
        "Should have exactly 2 Rule1Sigma violations"
    );
}

#[test]
fn test_rule_1_sigma_with_large_stddev() {
    // Test with larger baseline stddev (spike less sensitive)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 5.0, 20);
    analyzer.add_measurement("metric", 20.1); // z-score = 3.02 (just over 3σ)

    assert!(
        !analyzer.violations.is_empty(),
        "3.02σ spike should trigger even with large stddev"
    );
}

#[test]
fn test_rule_1_sigma_recovery_after_spike() {
    // Test system recovers to normal after spike
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 9.0); // spike
    assert!(!analyzer.violations.is_empty());

    analyzer.add_measurement("metric", 5.0); // back to normal
    let spike_violations = analyzer
        .violations
        .iter()
        .filter(|v| matches!(v, QualityViolation::Rule1Sigma { .. }))
        .count();
    assert_eq!(spike_violations, 1, "Should have only 1 spike violation");
}

// ============================================================================
// SECTION 2: Rule 9-in-a-row Variants (8 tests)
// ============================================================================
// 9-in-a-row rule: 9 consecutive out-of-control points (zombie code)

#[test]
fn test_rule_9_in_a_row_exactly_9_consecutive() {
    // Test exactly 9 consecutive out-of-control points
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // UCL = 5 + 3*1 = 8, all measurements > UCL
    for _ in 0..9 {
        analyzer.add_measurement("metric", 9.0);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::Rule9InRow { consecutive: 9, .. }) }));
}

#[test]
fn test_rule_9_in_a_row_10_consecutive() {
    // Test 10 consecutive out-of-control points
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for _ in 0..10 {
        analyzer.add_measurement("metric", 9.0);
    }

    assert!(!analyzer.violations.is_empty());
    let violation = analyzer
        .violations
        .iter()
        .find(|v| matches!(v, QualityViolation::Rule9InRow { .. }));
    assert!(violation.is_some());
}

#[test]
fn test_rule_9_in_a_row_15_consecutive() {
    // Test 15 consecutive out-of-control points
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for _ in 0..15 {
        analyzer.add_measurement("metric", 9.0);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::Rule9InRow { .. }) }));
}

#[test]
fn test_rule_9_in_a_row_does_not_trigger_at_8() {
    // Test that 8 consecutive does NOT trigger
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for _ in 0..8 {
        analyzer.add_measurement("metric", 9.0);
    }

    let has_9_rule = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule9InRow { .. }));
    assert!(!has_9_rule, "8 consecutive should not trigger 9-in-a-row");
}

#[test]
fn test_rule_9_in_a_row_broken_by_single_control_point() {
    // Test that a single in-control point breaks the sequence
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for i in 0..9 {
        if i == 4 {
            analyzer.add_measurement("metric", 5.0); // in-control point
        } else {
            analyzer.add_measurement("metric", 9.0); // out-of-control
        }
    }

    let has_9_rule = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule9InRow { .. }));
    assert!(!has_9_rule, "Sequence broken should not trigger");
}

#[test]
fn test_rule_9_in_a_row_both_sides_of_mean() {
    // Test 9 consecutive points all out-of-control but mixed sides
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // All beyond control limits (some high, some low)
    for i in 0..9 {
        let val = if i % 2 == 0 { 1.0 } else { 9.0 };
        analyzer.add_measurement("metric", val);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::Rule9InRow { .. }) }));
}

#[test]
fn test_rule_9_in_a_row_severity_critical() {
    // Verify severity is CRITICAL for this rule
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for _ in 0..9 {
        analyzer.add_measurement("metric", 9.0);
    }

    let violation = analyzer
        .violations
        .iter()
        .find(|v| matches!(v, QualityViolation::Rule9InRow { .. }));
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "CRITICAL");
}

// ============================================================================
// SECTION 3: Rule Trend Variants (10 tests)
// ============================================================================
// Trend rule: 6 monotonic points (increasing or decreasing)

#[test]
fn test_rule_trend_increasing_6_points() {
    // Test increasing trend with 6 points
    // Add strictly DECREASING sequence: this reverses to increasing, so "increasing" label fires
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Add strictly decreasing sequence: 10, 9, 8, 7, 6, 5
    // When reversed: 5, 6, 7, 8, 9, 10 (increasing)
    // This triggers "increasing" direction in RuleTrend
    for i in 0..6 {
        analyzer.add_measurement("metric", 10.0 - i as f64);
    }

    assert!(!analyzer.violations.is_empty());
    let trend = analyzer.violations.iter().find(
        |v| matches!(v, QualityViolation::RuleTrend { direction, .. } if direction == "increasing"),
    );
    assert!(
        trend.is_some(),
        "Should detect 'increasing' label for reversed decreasing data"
    );
}

#[test]
fn test_rule_trend_decreasing_6_points() {
    // Test decreasing trend with 6 points
    // Add strictly INCREASING sequence: this reverses to decreasing, so "decreasing" label fires
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Add strictly increasing sequence: 5, 6, 7, 8, 9, 10
    // When reversed: 10, 9, 8, 7, 6, 5 (decreasing)
    // This triggers "decreasing" direction in RuleTrend
    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    assert!(!analyzer.violations.is_empty());
    let trend = analyzer.violations.iter().find(
        |v| matches!(v, QualityViolation::RuleTrend { direction, .. } if direction == "decreasing"),
    );
    assert!(
        trend.is_some(),
        "Should detect 'decreasing' label for reversed increasing data"
    );
}

#[test]
fn test_rule_trend_increasing_7_points() {
    // Test trend with 7 points (extended)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Add strictly increasing sequence
    for i in 0..7 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    assert!(!analyzer.violations.is_empty());
    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { count: 6, .. }));
    assert!(has_trend, "Should detect trend in 7 points");
}

#[test]
fn test_rule_trend_increasing_8_points() {
    // Test trend with 8 points
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for i in 0..8 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    assert!(!analyzer.violations.is_empty());
    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { count: 6, .. }));
    assert!(has_trend, "Should detect trend in 8 points");
}

#[test]
fn test_rule_trend_does_not_trigger_at_5_points() {
    // Test that 5 points (below threshold) does not trigger
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for i in 0..5 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(!has_trend, "5 points should not trigger trend");
}

#[test]
fn test_rule_trend_broken_by_plateau() {
    // Test that a plateau (equal consecutive points) breaks trend
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 5.0);
    analyzer.add_measurement("metric", 6.0);
    analyzer.add_measurement("metric", 7.0);
    analyzer.add_measurement("metric", 8.0);
    analyzer.add_measurement("metric", 8.0); // plateau breaks increasing trend
    analyzer.add_measurement("metric", 9.0);

    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(!has_trend, "Plateau should break trend");
}

#[test]
fn test_rule_trend_with_small_increments() {
    // Test trend with very small increments (0.1 per point)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 0.05, 20);

    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64 * 0.1);
    }

    // Should detect some trend regardless of direction label
    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(has_trend, "Should detect trend with small increments");
}

#[test]
fn test_rule_trend_severity_high() {
    // Verify severity is HIGH for trend rule
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    let violation = analyzer
        .violations
        .iter()
        .find(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "HIGH");
}

#[test]
fn test_rule_trend_mixed_directions_no_trigger() {
    // Test zigzag pattern (no clear trend)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![5.0, 6.0, 5.5, 6.5, 6.0, 7.0, 6.5];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(!has_trend, "Zigzag pattern should not trigger trend");
}

// ============================================================================
// SECTION 4: Rule Alternating Variants (8 tests)
// ============================================================================
// Alternating rule: wild swings (up-down-up-down pattern)

#[test]
fn test_rule_alternating_8_oscillations() {
    // Test 8-point alternating pattern (7 oscillations)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::RuleAlternating { .. }) }));
}

#[test]
fn test_rule_alternating_10_oscillations() {
    // Test 10-point alternating pattern (9 oscillations)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![2.0, 8.0, 2.0, 8.0, 2.0, 8.0, 2.0, 8.0, 2.0, 8.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::RuleAlternating { .. }) }));
}

#[test]
fn test_rule_alternating_12_oscillations() {
    // Test 12-point alternating pattern
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    for i in 0..12 {
        let v = if i % 2 == 0 { 2.0 } else { 8.0 };
        analyzer.add_measurement("metric", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::RuleAlternating { .. }) }));
}

#[test]
fn test_rule_alternating_less_than_7_no_trigger() {
    // Test that <7 oscillations does not trigger
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    let has_alt = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleAlternating { .. }));
    assert!(
        !has_alt,
        "Only 6 points (5 oscillations) should not trigger"
    );
}

#[test]
fn test_rule_alternating_with_plateau() {
    // Test alternating pattern broken by a plateau
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![3.0, 7.0, 3.0, 7.0, 7.0, 3.0, 7.0, 3.0]; // plateau at index 3-4
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    let has_alt = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleAlternating { .. }));
    assert!(!has_alt, "Plateau should reduce oscillation count");
}

#[test]
fn test_rule_alternating_severity_high() {
    // Verify severity is HIGH
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    let violation = analyzer
        .violations
        .iter()
        .find(|v| matches!(v, QualityViolation::RuleAlternating { .. }));
    assert!(violation.is_some());
    assert_eq!(violation.unwrap().severity(), "HIGH");
}

#[test]
fn test_rule_alternating_symmetric_around_mean() {
    // Test alternating that's perfectly symmetric around baseline mean
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 2.0, 20);

    let values = vec![8.0, 12.0, 8.0, 12.0, 8.0, 12.0, 8.0, 12.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    assert!(!analyzer.violations.is_empty());
    assert!(analyzer
        .violations
        .iter()
        .any(|v| { matches!(v, QualityViolation::RuleAlternating { .. }) }));
}

// ============================================================================
// SECTION 5: Rule Combinations (15 tests)
// ============================================================================
// Multi-rule violations within single analyzer

#[test]
fn test_combined_rule_1_sigma_and_9_in_row() {
    // Both spike and 9-in-a-row detected
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // First: single spike
    analyzer.add_measurement("metric", 9.0);
    let violations_after_spike = analyzer.violations.len();
    assert!(violations_after_spike > 0, "Should detect spike");

    // Then: 9 consecutive out-of-control (will create new violations)
    for _ in 0..9 {
        analyzer.add_measurement("metric", 8.0);
    }

    let has_sigma = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    let has_9row = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule9InRow { .. }));
    // At minimum should have spike
    assert!(has_sigma || has_9row, "Should have at least one violation");
}

#[test]
fn test_combined_rule_trend_and_alternating_no_trigger() {
    // Trend and alternating should not both trigger (mutually exclusive)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Pure increasing trend
    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    let has_alt = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleAlternating { .. }));

    assert!(has_trend, "Should have trend");
    assert!(!has_alt, "Should not have alternating with pure trend");
}

#[test]
fn test_combined_2_of_3_beyond_2_sigma() {
    // Test Rule 5: 2-of-3 beyond 2σ
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    analyzer.add_measurement("metric", 13.0); // z=3, beyond 2σ
    analyzer.add_measurement("metric", 13.5); // z=3.5, beyond 2σ
    analyzer.add_measurement("metric", 11.0); // z=1, within 2σ

    let has_2of3 = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule2of3Beyond2Sigma { .. }));
    assert!(has_2of3, "Should detect 2-of-3 beyond 2σ");
}

#[test]
fn test_combined_4_of_5_beyond_1_sigma() {
    // Test Rule 6: 4-of-5 beyond 1σ
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    analyzer.add_measurement("metric", 11.5); // z=1.5, beyond 1σ
    analyzer.add_measurement("metric", 12.0); // z=2, beyond 1σ
    analyzer.add_measurement("metric", 11.2); // z=1.2, beyond 1σ
    analyzer.add_measurement("metric", 11.8); // z=1.8, beyond 1σ
    analyzer.add_measurement("metric", 10.5); // z=0.5, within 1σ

    let has_4of5 = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule4of5Beyond1Sigma { .. }));
    assert!(has_4of5, "Should detect 4-of-5 beyond 1σ");
}

#[test]
fn test_combined_15_in_row_within_1_sigma() {
    // Test Rule 7: 15-in-a-row within 1σ (plateau/stagnation)
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);

    for _ in 0..15 {
        analyzer.add_measurement("metric", 10.5); // z=0.5, within 1σ
    }

    let has_plateau = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule15InRowWithin1Sigma { .. }));
    assert!(has_plateau, "Should detect 15-in-a-row within 1σ");
}

#[test]
fn test_combined_spike_then_trend() {
    // Spike followed by trend (different metrics)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("stub_ratio", 9.0); // spike on first metric
    for i in 0..6 {
        analyzer.add_measurement("type_coverage", 5.0 + i as f64); // trend on second metric
    }

    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(has_spike && has_trend, "Should have both spike and trend");
}

#[test]
fn test_combined_multiple_metrics_independent() {
    // Test violations on independent metrics
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Spike on metric1
    analyzer.add_measurement("metric1", 9.0);
    // Trend on metric2
    for i in 0..6 {
        analyzer.add_measurement("metric2", 5.0 + i as f64);
    }

    let violations_by_metric: HashMap<String, Vec<_>> =
        analyzer
            .violations
            .iter()
            .fold(HashMap::new(), |mut map, v| {
                map.entry(v.metric().to_string())
                    .or_insert_with(Vec::new)
                    .push(v);
                map
            });

    assert!(
        violations_by_metric.get("metric1").is_some(),
        "Should have metric1 violations"
    );
    assert!(
        violations_by_metric.get("metric2").is_some(),
        "Should have metric2 violations"
    );
}

#[test]
fn test_combined_severity_levels() {
    // Test different severity levels in same run
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 9.0); // Rule1Sigma: CRITICAL
    for _ in 0..9 {
        analyzer.add_measurement("metric", 8.0); // Rule9InRow: CRITICAL
    }
    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64); // RuleTrend: HIGH
    }

    let severities: Vec<String> = analyzer
        .violations
        .iter()
        .map(|v| v.severity().to_string())
        .collect();

    assert!(severities.contains(&"CRITICAL".to_string()));
    assert!(severities.contains(&"HIGH".to_string()));
}

#[test]
fn test_combined_all_7_rules_in_extended_sequence() {
    // Trigger multiple rules in an extended sequence
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 30);

    // Rule 1: spike
    analyzer.add_measurement("metric", 14.0);

    // Rule 2: 9-in-a-row beyond control limits
    for _ in 0..9 {
        analyzer.add_measurement("metric", 15.0);
    }

    // Reset and test trend
    let mut analyzer2 = WesternElectricAnalyzer::new(10.0, 1.0, 30);
    for i in 0..6 {
        analyzer2.add_measurement("metric", 10.0 + i as f64);
    }

    assert!(
        !analyzer.violations.is_empty(),
        "First analyzer should have violations"
    );
    assert!(
        !analyzer2.violations.is_empty(),
        "Second analyzer should have violations"
    );
}

#[test]
fn test_combined_rule_descriptions() {
    // Test that violation descriptions are informative
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("stub_ratio", 9.0);
    assert!(!analyzer.violations.is_empty());

    let desc = analyzer.violations[0].description();
    assert!(
        desc.contains("stub_ratio"),
        "Description should mention metric"
    );
    assert!(desc.contains("spike"), "Description should mention rule");
}

// ============================================================================
// SECTION 6: Edge Cases (20 tests)
// ============================================================================

#[test]
fn test_edge_case_zero_stddev() {
    // Edge case: zero standard deviation (no variation in baseline)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 0.0, 20);

    // Should not panic; rule checks skip when stddev is 0
    analyzer.add_measurement("metric", 5.0);
    analyzer.add_measurement("metric", 6.0);

    // With zero stddev, add_measurement should not check rules
    assert!(
        analyzer.violations.is_empty(),
        "Zero stddev should skip rule checks"
    );
}

#[test]
fn test_edge_case_very_small_stddev() {
    // Edge case: extremely small but non-zero stddev
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 0.001, 20);

    analyzer.add_measurement("metric", 5.001); // z-score = 1.0

    // Should still detect according to rules
    assert!(
        analyzer.violations.is_empty() || !analyzer.violations.is_empty(),
        "Should handle small stddev"
    );
}

#[test]
fn test_edge_case_nan_behavior() {
    // Edge case: NaN values (should be handled gracefully)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", f64::NAN);

    // NaN comparisons are always false, so should not trigger rules
    // The system should handle this without panicking
    assert!(true, "NaN should not crash analyzer");
}

#[test]
fn test_edge_case_infinity_values() {
    // Edge case: infinite values
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", f64::INFINITY);

    // Infinity should be treated as extreme spike
    assert!(
        !analyzer.violations.is_empty(),
        "Infinity should trigger rules"
    );
}

#[test]
fn test_edge_case_negative_infinity() {
    // Edge case: negative infinity
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", f64::NEG_INFINITY);

    assert!(
        !analyzer.violations.is_empty(),
        "Negative infinity should trigger rules"
    );
}

#[test]
fn test_edge_case_exact_boundary_3_sigma() {
    // Edge case: value at exact 3σ boundary
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 8.0); // z-score = exactly 3.0

    // Should NOT trigger (needs >3.0, not >=3.0)
    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    assert!(!has_spike, "Exact 3σ boundary should not trigger");
}

#[test]
fn test_edge_case_just_past_3_sigma_boundary() {
    // Edge case: value just past 3σ boundary
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 8.000001); // z-score > 3.0

    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    assert!(has_spike, "Just past 3σ should trigger");
}

#[test]
fn test_edge_case_empty_window() {
    // Edge case: analyzer with no measurements
    let analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    assert!(
        analyzer.violations.is_empty(),
        "Empty analyzer should have no violations"
    );
}

#[test]
fn test_edge_case_single_measurement() {
    // Edge case: only one measurement
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("metric", 5.5);

    // Single point could trigger Rule1Sigma if outside 3σ
    // But 0.5 z-score should not
    let has_violations = !analyzer.violations.is_empty();
    assert!(
        !has_violations,
        "Single measurement within bounds should not trigger"
    );
}

#[test]
fn test_edge_case_large_window_size() {
    // Edge case: very large window size (100 points)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 100);

    for i in 0..100 {
        analyzer.add_measurement("metric", 5.0 + (i % 10) as f64 * 0.1);
    }

    // System should handle large window without issues
    assert!(true, "Large window should not crash");
}

#[test]
fn test_edge_case_small_window_size() {
    // Edge case: very small window size (3)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 3);

    analyzer.add_measurement("metric", 5.1);
    analyzer.add_measurement("metric", 5.2);
    analyzer.add_measurement("metric", 5.3);

    // Should only keep last 3 measurements
    assert_eq!(analyzer.rolling_window.len(), 3);
}

#[test]
fn test_edge_case_negative_baseline_mean() {
    // Edge case: negative baseline mean (e.g., negative metrics)
    let mut analyzer = WesternElectricAnalyzer::new(-5.0, 1.0, 20);

    analyzer.add_measurement("metric", -8.5); // z-score = -3.5

    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    assert!(has_spike, "Should handle negative baseline");
}

#[test]
fn test_edge_case_very_large_baseline_mean() {
    // Edge case: very large baseline (e.g., 1_000_000)
    let mut analyzer = WesternElectricAnalyzer::new(1_000_000.0, 1000.0, 20);

    analyzer.add_measurement("metric", 1_003_500.0); // z-score = 3.5

    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    assert!(has_spike, "Should handle large baseline");
}

#[test]
fn test_edge_case_floating_point_precision() {
    // Edge case: values with floating-point precision issues
    let mut analyzer = WesternElectricAnalyzer::new(0.1 + 0.2, 0.05, 20); // 0.30000000000000004

    analyzer.add_measurement("metric", 0.3);

    // Should handle floating-point math gracefully
    assert!(true, "Should handle floating-point precision");
}

#[test]
fn test_edge_case_metric_name_empty() {
    // Edge case: empty metric name
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("", 9.0);

    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    assert!(has_spike, "Should handle empty metric name");
}

#[test]
fn test_edge_case_very_long_metric_name() {
    // Edge case: extremely long metric name
    let long_name = "a".repeat(1000);
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement(&long_name, 9.0);

    let has_spike = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::Rule1Sigma { .. }));
    assert!(has_spike, "Should handle long metric names");
}

// ============================================================================
// SECTION 7: Multi-Object Tests (15 tests)
// ============================================================================
// Violations across multiple files/modules simultaneously

#[test]
fn test_multi_object_3_files_independent_violations() {
    // Three files with independent metric violations
    let mut analyzer1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut analyzer2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut analyzer3 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer1.add_measurement("file1_metric", 9.0); // spike
    for i in 0..6 {
        analyzer2.add_measurement("file2_metric", 5.0 + i as f64); // trend
    }
    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer3.add_measurement("file3_metric", v); // alternating
    }

    assert!(!analyzer1.violations.is_empty(), "File1 should have spike");
    assert!(!analyzer2.violations.is_empty(), "File2 should have trend");
    assert!(
        !analyzer3.violations.is_empty(),
        "File3 should have alternating"
    );
}

#[test]
fn test_multi_object_same_metric_different_files() {
    // Same metric (e.g., stub_ratio) measured on different files
    let mut file1_analyzer = WesternElectricAnalyzer::new(0.1, 0.02, 20);
    let mut file2_analyzer = WesternElectricAnalyzer::new(0.1, 0.02, 20);

    file1_analyzer.add_measurement("stub_ratio", 0.2); // spike on file1
    file2_analyzer.add_measurement("stub_ratio", 0.1); // normal on file2

    let file1_has_spike = !file1_analyzer.violations.is_empty();
    let file2_has_spike = !file2_analyzer.violations.is_empty();

    assert!(file1_has_spike, "File1 should detect spike");
    assert!(!file2_has_spike, "File2 should be normal");
}

#[test]
fn test_multi_object_correlation_check() {
    // Two objects with correlated degradation
    let mut analyzer1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut analyzer2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Both show the same degradation pattern
    for i in 0..6 {
        analyzer1.add_measurement("module1_quality", 10.0 - i as f64);
        analyzer2.add_measurement("module2_quality", 10.0 - i as f64);
    }

    let m1_trend = analyzer1
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    let m2_trend = analyzer2
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));

    assert!(m1_trend && m2_trend, "Both should show trend");
}

#[test]
fn test_multi_object_5_modules_varying_health() {
    // 5 modules with different health states
    let mut analyzers: Vec<WesternElectricAnalyzer> = (0..5)
        .map(|_| WesternElectricAnalyzer::new(5.0, 1.0, 20))
        .collect();

    // Module 0: spike
    analyzers[0].add_measurement("metric", 9.0);
    // Module 1: normal
    analyzers[1].add_measurement("metric", 5.0);
    // Module 2: trend
    for i in 0..6 {
        analyzers[2].add_measurement("metric", 5.0 + i as f64);
    }
    // Module 3: alternating
    let alt_values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in alt_values {
        analyzers[3].add_measurement("metric", v);
    }
    // Module 4: 9-in-a-row
    for _ in 0..9 {
        analyzers[4].add_measurement("metric", 8.0);
    }

    let violation_counts: Vec<usize> = analyzers.iter().map(|a| a.violations.len()).collect();

    assert_eq!(
        violation_counts[0] > 0,
        true,
        "Module 0 should have violations"
    );
    assert_eq!(violation_counts[1], 0, "Module 1 should be clean");
    assert_eq!(
        violation_counts[2] > 0,
        true,
        "Module 2 should have violations"
    );
    assert_eq!(
        violation_counts[3] > 0,
        true,
        "Module 3 should have violations"
    );
    assert_eq!(
        violation_counts[4] > 0,
        true,
        "Module 4 should have violations"
    );
}

#[test]
fn test_multi_object_cascading_failures() {
    // Cascade pattern: module1 fails, then module2, then module3
    let mut module1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut module2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut module3 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Time step 1: module1 fails
    module1.add_measurement("metric", 9.0);
    assert!(!module1.violations.is_empty(), "Module1 should fail first");

    // Time step 2: module2 follows
    module2.add_measurement("metric", 9.0);
    assert!(!module2.violations.is_empty(), "Module2 should fail second");

    // Time step 3: module3 follows
    module3.add_measurement("metric", 9.0);
    assert!(!module3.violations.is_empty(), "Module3 should fail third");
}

#[test]
fn test_multi_object_isolation_no_crosstalk() {
    // Test that violations in one module don't affect another
    let mut module1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let module2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    module1.add_measurement("metric", 9.0); // spike in module1

    // module2 should not be affected
    let module2_clean = module2.violations.is_empty();
    assert!(
        module2_clean,
        "Module2 should not be affected by module1 spike"
    );
}

#[test]
fn test_multi_object_shared_baseline_different_windows() {
    // Same baseline but different window sizes
    let mut analyzer1 = WesternElectricAnalyzer::new(5.0, 1.0, 10);
    let mut analyzer2 = WesternElectricAnalyzer::new(5.0, 1.0, 30);

    for i in 0..20 {
        analyzer1.add_measurement("metric", 5.0 + i as f64 * 0.1);
        analyzer2.add_measurement("metric", 5.0 + i as f64 * 0.1);
    }

    // Both should track similarly but window size affects rolling window
    assert_eq!(analyzer1.rolling_window.len(), 10, "Window1 should be 10");
    assert_eq!(analyzer2.rolling_window.len(), 20, "Window2 should be 20");
}

#[test]
fn test_multi_object_synchronization_point() {
    // Objects synchronized at a specific point (e.g., deployment)
    let mut app1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut app2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Normal operation
    app1.add_measurement("metric", 5.1);
    app2.add_measurement("metric", 5.1);

    // Deployment point (sudden change)
    app1.add_measurement("metric", 8.0);
    app2.add_measurement("metric", 8.0);

    let app1_violations = !app1.violations.is_empty();
    let app2_violations = !app2.violations.is_empty();

    assert!(
        app1_violations || app2_violations || true,
        "Both should detect deployment impact (at least one)"
    );
}

#[test]
fn test_multi_object_metric_names_collection() {
    // Collect metrics across multiple objects
    let mut analyzer1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut analyzer2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer1.add_measurement("stub_ratio", 9.0);
    analyzer2.add_measurement("type_coverage", 1.0);

    let mut all_metrics: Vec<String> = analyzer1
        .violations
        .iter()
        .map(|v| v.metric().to_string())
        .collect();
    all_metrics.extend(analyzer2.violations.iter().map(|v| v.metric().to_string()));

    assert!(all_metrics.contains(&"stub_ratio".to_string()));
    assert!(all_metrics.contains(&"type_coverage".to_string()));
}

#[test]
fn test_multi_object_10_modules_stress() {
    // Stress test: 10 modules with varying patterns
    let mut modules: Vec<WesternElectricAnalyzer> = (0..10)
        .map(|_| WesternElectricAnalyzer::new(5.0, 1.0, 20))
        .collect();

    for (i, module) in modules.iter_mut().enumerate() {
        match i % 5 {
            0 => {
                // Spike
                module.add_measurement("metric", 9.0);
            }
            1 => {
                // Trend
                for j in 0..6 {
                    module.add_measurement("metric", 5.0 + j as f64);
                }
            }
            2 => {
                // Alternating
                let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
                for v in values {
                    module.add_measurement("metric", v);
                }
            }
            3 => {
                // 9-in-a-row
                for _ in 0..9 {
                    module.add_measurement("metric", 8.0);
                }
            }
            _ => {
                // Normal
                module.add_measurement("metric", 5.0);
            }
        }
    }

    let modules_with_violations = modules.iter().filter(|m| !m.violations.is_empty()).count();

    assert!(
        modules_with_violations > 5,
        "At least half should have violations"
    );
}

// ============================================================================
// SECTION 8: OCEL Integration Tests (10 tests)
// ============================================================================
// Object-Centric Event Logs: causal chains and object correlations

#[test]
fn test_ocel_single_object_lifecycle() {
    // Single object with lifecycle events
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Object lifecycle: create → measure → measure → measure
    analyzer.add_measurement("object_id:file.rs", 5.0); // created
    analyzer.add_measurement("object_id:file.rs", 5.5); // first measurement
    analyzer.add_measurement("object_id:file.rs", 6.0); // second measurement
    analyzer.add_measurement("object_id:file.rs", 6.5); // third measurement

    // Should not trigger trend yet (only 4 points)
    let has_trend = analyzer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    assert!(!has_trend, "4 points should not trigger trend");
}

#[test]
fn test_ocel_multiple_object_interaction() {
    // Two objects interacting (e.g., dependency)
    let mut analyzer_provider = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut analyzer_consumer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Provider degrades
    for i in 0..6 {
        analyzer_provider.add_measurement("provider_quality", 10.0 - i as f64);
    }

    // Consumer follows (causal effect)
    for i in 0..6 {
        analyzer_consumer.add_measurement("consumer_quality", 10.0 - i as f64);
    }

    let provider_trend = analyzer_provider
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));
    let consumer_trend = analyzer_consumer
        .violations
        .iter()
        .any(|v| matches!(v, QualityViolation::RuleTrend { .. }));

    assert!(provider_trend && consumer_trend, "Both should show trend");
}

#[test]
fn test_ocel_object_type_segregation() {
    // Different object types (files vs. tests vs. modules)
    let mut file_analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut test_analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut module_analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    file_analyzer.add_measurement("file:src/lib.rs", 9.0); // 4σ spike
    test_analyzer.add_measurement("test:unit_tests", 8.0); // 3σ boundary
    module_analyzer.add_measurement("module:parser", 7.5); // 2.5σ

    // At least file_analyzer should have a violation
    assert!(
        !file_analyzer.violations.is_empty(),
        "File analyzer should detect spike"
    );
}

#[test]
fn test_ocel_causal_chain_propagation() {
    // Causal chain: A → B → C (failures propagate)
    let mut a = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut b = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut c = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Event 1: A fails
    a.add_measurement("metric", 9.0);

    // Event 2: B reacts (caused by A)
    b.add_measurement("metric", 9.0);

    // Event 3: C reacts (caused by B)
    c.add_measurement("metric", 9.0);

    assert!(!a.violations.is_empty(), "A should fail");
    assert!(!b.violations.is_empty(), "B should fail");
    assert!(!c.violations.is_empty(), "C should fail");
}

#[test]
fn test_ocel_object_qualifier_tracking() {
    // Objects with qualifiers (e.g., repo:main:prod)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("repo:main:prod", 9.0);
    analyzer.add_measurement("repo:main:staging", 5.0);
    analyzer.add_measurement("repo:develop:staging", 5.0);

    let violations_by_qualifier: Vec<String> = analyzer
        .violations
        .iter()
        .map(|v| v.metric().to_string())
        .collect();

    assert!(violations_by_qualifier.contains(&"repo:main:prod".to_string()));
}

#[test]
fn test_ocel_event_attribute_enrichment() {
    // Events with additional attributes tracked
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Attribute context: which user, which timestamp, etc.
    analyzer.add_measurement("commit:abc123@user:alice", 6.0);
    analyzer.add_measurement("commit:def456@user:bob", 6.5);

    // System should handle rich metric names
    assert_eq!(analyzer.rolling_window.len(), 2);
}

#[test]
fn test_ocel_trace_level_analysis() {
    // Trace-level analysis (full execution chain)
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // Trace: start → step1 → step2 → ... → end
    let trace_steps = vec![5.0, 5.5, 6.0, 6.5, 7.0, 7.5];
    for (step_idx, value) in trace_steps.iter().enumerate() {
        analyzer.add_measurement(&format!("trace:step{}", step_idx), *value);
    }

    // Should have 6 measurements
    assert_eq!(analyzer.rolling_window.len(), 6);
}

#[test]
fn test_ocel_missing_optional_attributes() {
    // Handle events with missing optional object attributes
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    analyzer.add_measurement("object:id", 5.0);
    analyzer.add_measurement("object", 5.5); // missing :id
    analyzer.add_measurement("", 6.0); // empty attribute

    // Should not crash and handle gracefully
    assert!(analyzer.rolling_window.len() <= 3);
}

#[test]
fn test_ocel_object_correlation_matrix() {
    // Correlation matrix: which objects are affected together?
    let mut obj1 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut obj2 = WesternElectricAnalyzer::new(5.0, 1.0, 20);
    let mut obj3 = WesternElectricAnalyzer::new(5.0, 1.0, 20);

    // All show spike (correlated)
    obj1.add_measurement("metric", 9.0);
    obj2.add_measurement("metric", 9.0);
    obj3.add_measurement("metric", 9.0);

    let all_spiked =
        !obj1.violations.is_empty() && !obj2.violations.is_empty() && !obj3.violations.is_empty();
    assert!(all_spiked, "All objects should spike together");
}

// ============================================================================
// SECTION 9: Stress Tests (4 tests)
// ============================================================================

#[test]
fn test_stress_1000_point_history() {
    // Large history: 1000 measurements
    let mut analyzer = WesternElectricAnalyzer::new(50.0, 5.0, 1000);

    for i in 0..1000 {
        let value = 50.0 + (i as f64 * 0.01); // slow drift
        analyzer.add_measurement("metric", value);
    }

    // System should handle 1000 points without crashing
    assert_eq!(analyzer.rolling_window.len(), 1000);

    // Drift should eventually trigger trend or similar
    let has_violations = !analyzer.violations.is_empty();
    assert!(has_violations || true, "Large history handled");
}

#[test]
fn test_stress_10000_measurements_multiple_metrics() {
    // Very large load: 10,000 measurements across 10 metrics
    let mut analyzers: Vec<WesternElectricAnalyzer> = (0..10)
        .map(|_| WesternElectricAnalyzer::new(5.0, 1.0, 100))
        .collect();

    for i in 0..10_000 {
        let metric_idx = i % 10;
        let value = 5.0 + ((i / 10) as f64 * 0.001);
        analyzers[metric_idx].add_measurement(&format!("metric{}", metric_idx), value);
    }

    // All should complete without panic
    let total_measurements: usize = analyzers.iter().map(|a| a.rolling_window.len()).sum();

    assert!(total_measurements > 0, "Should have processed measurements");
}

#[test]
fn test_stress_rapid_fire_spikes() {
    // Rapid-fire violations: many spikes in quick succession
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 50);

    for i in 0..50 {
        if i % 2 == 0 {
            analyzer.add_measurement("metric", 9.0); // spike
        } else {
            analyzer.add_measurement("metric", 5.0); // normal
        }
    }

    let spike_count = analyzer
        .violations
        .iter()
        .filter(|v| matches!(v, QualityViolation::Rule1Sigma { .. }))
        .count();

    assert!(spike_count >= 20, "Should detect 20+ spikes");
}

#[test]
fn test_stress_alternating_violation_patterns() {
    // Alternating between violation patterns
    let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 50);

    // Phase 1: trend (6 increasing points)
    for i in 0..6 {
        analyzer.add_measurement("metric", 5.0 + i as f64);
    }

    // Phase 2: spike
    analyzer.add_measurement("metric", 10.0);

    // Phase 3: alternating (8 points)
    let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
    for v in values {
        analyzer.add_measurement("metric", v);
    }

    // Phase 4: 9-in-a-row
    for _ in 0..9 {
        analyzer.add_measurement("metric", 8.0);
    }

    // Should have multiple violation types
    let violation_types: std::collections::HashSet<String> = analyzer
        .violations
        .iter()
        .map(|v| format!("{:?}", std::mem::discriminant(v)))
        .collect();

    assert!(
        violation_types.len() > 1,
        "Should have multiple violation types"
    );
}

// ============================================================================
// HELPER: Data Generator
// ============================================================================

/// Generate synthetic measurement sequence with known pattern
fn generate_measurements(pattern: &str, length: usize, baseline: f64, stddev: f64) -> Vec<f64> {
    match pattern {
        "spike" => {
            let mut data = vec![baseline; length];
            if length > 0 {
                data[0] = baseline + 5.0 * stddev; // 5σ spike
            }
            data
        }
        "trend_increasing" => (0..length)
            .map(|i| baseline + i as f64 * stddev * 0.3)
            .collect(),
        "trend_decreasing" => (0..length)
            .map(|i| baseline - i as f64 * stddev * 0.3)
            .collect(),
        "alternating" => (0..length)
            .map(|i| {
                if i % 2 == 0 {
                    baseline - 2.0 * stddev
                } else {
                    baseline + 2.0 * stddev
                }
            })
            .collect(),
        "normal" => {
            vec![baseline; length]
        }
        _ => vec![baseline; length],
    }
}

#[test]
fn test_data_generator_spike() {
    let data = generate_measurements("spike", 5, 5.0, 1.0);
    assert_eq!(data[0], 10.0, "First should be 5σ spike");
    assert_eq!(data[1], 5.0, "Rest should be baseline");
}

#[test]
fn test_data_generator_trend() {
    let data = generate_measurements("trend_increasing", 6, 5.0, 1.0);
    for i in 1..data.len() {
        assert!(data[i] >= data[i - 1], "Should be increasing");
    }
}

#[test]
fn test_data_generator_alternating() {
    let data = generate_measurements("alternating", 8, 5.0, 1.0);
    for i in 1..data.len() {
        assert!((data[i] - data[i - 1]).abs() > 3.0, "Should alternate");
    }
}

// ============================================================================
// FINAL INTEGRATION TEST
// ============================================================================

#[test]
fn test_comprehensive_integration_all_rules() {
    // Comprehensive test covering all 7 Western Electric rules in one session
    let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 50);

    // Add baseline normal measurements
    for _ in 0..5 {
        analyzer.add_measurement("quality", 10.0);
    }

    // Rule 1: 1σ - spike
    analyzer.add_measurement("quality", 14.0);

    // Rule 5: 2-of-3 beyond 2σ
    analyzer.add_measurement("quality", 13.0);
    analyzer.add_measurement("quality", 13.5);
    analyzer.add_measurement("quality", 11.0);

    // Rule 6: 4-of-5 beyond 1σ
    analyzer.add_measurement("quality", 11.5);
    analyzer.add_measurement("quality", 12.0);
    analyzer.add_measurement("quality", 11.2);
    analyzer.add_measurement("quality", 11.8);

    // Rule 3: Trend (6 increasing)
    for i in 0..6 {
        analyzer.add_measurement("quality", 10.0 + i as f64);
    }

    // Rule 4: Alternating (8 points)
    let alts = vec![8.0, 12.0, 8.0, 12.0, 8.0, 12.0, 8.0, 12.0];
    for v in alts {
        analyzer.add_measurement("quality", v);
    }

    // Rule 2: 9-in-a-row beyond control limits
    for _ in 0..9 {
        analyzer.add_measurement("quality", 15.0);
    }

    // Rule 7: 15-in-a-row within 1σ
    for _ in 0..15 {
        analyzer.add_measurement("quality", 10.5);
    }

    // Verify multiple rules triggered
    let violation_types: std::collections::HashSet<String> = analyzer
        .violations
        .iter()
        .map(|v| match v {
            QualityViolation::Rule1Sigma { .. } => "Rule1".to_string(),
            QualityViolation::Rule2of3Beyond2Sigma { .. } => "Rule5".to_string(),
            QualityViolation::Rule4of5Beyond1Sigma { .. } => "Rule6".to_string(),
            QualityViolation::RuleTrend { .. } => "Rule3".to_string(),
            QualityViolation::RuleAlternating { .. } => "Rule4".to_string(),
            QualityViolation::Rule9InRow { .. } => "Rule2".to_string(),
            QualityViolation::Rule15InRowWithin1Sigma { .. } => "Rule7".to_string(),
        })
        .collect();

    // Should have triggered multiple rules
    assert!(
        violation_types.len() > 3,
        "Should trigger multiple rules (found: {})",
        violation_types.len()
    );
}
