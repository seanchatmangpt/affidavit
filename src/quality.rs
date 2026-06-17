//! Code quality measurement and Western Electric statistical process control.
//!
//! This module provides:
//! - `CodeQualityMetrics`: comprehensive code quality measurements
//! - `WesternElectricAnalyzer`: statistical process control with all 7 rules
//! - Violations: detection and reporting of out-of-control conditions
//!
//! Western Electric rules detect when a process deviates from baseline:
//! 1. 1σ rule: single point >3σ from mean (spike detection)
//! 2. 9-in-a-row: 9 consecutive out-of-control points (zombie code)
//! 3. Trend: 6 monotonic points (systematic degradation)
//! 4. Alternating: wild swings (uncertainty/hallucination)
//! 5. 2-of-3 beyond 2σ: early warning
//! 6. 4-of-5 beyond 1σ: sustained deviation
//! 7. 15-in-a-row within 1σ: plateau/stagnation

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Code quality metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    /// Ratio of functions with todo!/unimplemented!/panic! (0.0–1.0)
    pub stub_ratio: f64,

    /// Ratio of function signatures with explicit type annotations (0.0–1.0)
    pub type_coverage: f64,

    /// Lines added + deleted in this commit
    pub churn: usize,

    /// Ratio of comment lines to code lines (0.0–∞)
    pub comment_ratio: f64,

    /// Mean cyclomatic complexity across all functions
    pub cyclomatic_complexity: f64,

    /// Maintainability index (0–100, higher = better)
    pub maintainability_index: f64,

    /// Cognitive complexity score (lower = simpler)
    pub cognitive_complexity: f64,

    /// Clippy warnings count
    pub clippy_warnings: usize,

    /// Rustfmt formatting violations
    pub rustfmt_violations: usize,

    /// Cargo-deny policy violations
    pub cargo_deny_issues: usize,

    /// Cargo-audit CVE detections
    pub cargo_audit_vulnerabilities: usize,

    /// Test coverage percentage (0–100)
    pub test_coverage: f64,

    /// Doc coverage ratio (0.0–1.0)
    pub doc_coverage: f64,

    /// Unix timestamp when measured
    pub timestamp: u64,
}

impl Default for CodeQualityMetrics {
    fn default() -> Self {
        Self {
            stub_ratio: 0.0,
            type_coverage: 1.0,
            churn: 0,
            comment_ratio: 0.2,
            cyclomatic_complexity: 2.0,
            maintainability_index: 100.0,
            cognitive_complexity: 5.0,
            clippy_warnings: 0,
            rustfmt_violations: 0,
            cargo_deny_issues: 0,
            cargo_audit_vulnerabilities: 0,
            test_coverage: 90.0,
            doc_coverage: 0.8,
            timestamp: 0,
        }
    }
}

/// Quality violation detected by Western Electric rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityViolation {
    /// 1σ rule: single point >3σ from baseline mean (spike)
    Rule1Sigma {
        metric: String,
        value: f64,
        threshold: f64,
        z_score: f64,
        severity: String,
    },

    /// 9-in-a-row: 9 consecutive out-of-control points
    Rule9InRow {
        metric: String,
        consecutive: usize,
    },

    /// Trend: 6 monotonic points (increasing or decreasing)
    RuleTrend {
        metric: String,
        direction: String, // "increasing" or "decreasing"
        count: usize,
    },

    /// Alternating: wild swings (up-down-up-down pattern)
    RuleAlternating {
        metric: String,
        oscillations: usize,
    },

    /// 2-of-3 beyond 2σ: early warning
    Rule2of3Beyond2Sigma {
        metric: String,
        count: usize,
        threshold: f64,
    },

    /// 4-of-5 beyond 1σ: sustained deviation
    Rule4of5Beyond1Sigma {
        metric: String,
        count: usize,
        threshold: f64,
    },

    /// 15-in-a-row within 1σ: plateau/stagnation
    Rule15InRowWithin1Sigma {
        metric: String,
        count: usize,
        threshold: f64,
        severity: String,
    },
}

impl QualityViolation {
    pub fn severity(&self) -> &str {
        match self {
            Self::Rule1Sigma { severity, .. } => severity,
            Self::Rule9InRow { .. } => "CRITICAL",
            Self::RuleTrend { .. } => "HIGH",
            Self::RuleAlternating { .. } => "HIGH",
            Self::Rule2of3Beyond2Sigma { .. } => "HIGH",
            Self::Rule4of5Beyond1Sigma { .. } => "MEDIUM",
            Self::Rule15InRowWithin1Sigma { severity, .. } => severity,
        }
    }

    pub fn metric(&self) -> &str {
        match self {
            Self::Rule1Sigma { metric, .. } => metric,
            Self::Rule9InRow { metric, .. } => metric,
            Self::RuleTrend { metric, .. } => metric,
            Self::RuleAlternating { metric, .. } => metric,
            Self::Rule2of3Beyond2Sigma { metric, .. } => metric,
            Self::Rule4of5Beyond1Sigma { metric, .. } => metric,
            Self::Rule15InRowWithin1Sigma { metric, .. } => metric,
        }
    }

    pub fn description(&self) -> String {
        match self {
            Self::Rule1Sigma { metric, value, threshold, z_score, .. } => {
                format!("{}: spike detected (value={:.2}, threshold={:.2}, z-score={:.2})", metric, value, threshold, z_score)
            }
            Self::Rule9InRow { metric, consecutive } => {
                format!("{}: {} consecutive out-of-control points (zombie code)", metric, consecutive)
            }
            Self::RuleTrend { metric, direction, count } => {
                format!("{}: {} monotonic {} (systematic degradation)", metric, count, direction)
            }
            Self::RuleAlternating { metric, oscillations } => {
                format!("{}: {} oscillations detected (uncertainty/hallucination)", metric, oscillations)
            }
            Self::Rule2of3Beyond2Sigma { metric, count, threshold } => {
                format!("{}: {} of 3 points beyond 2σ threshold {:.2}", metric, count, threshold)
            }
            Self::Rule4of5Beyond1Sigma { metric, count, threshold } => {
                format!("{}: {} of 5 points beyond 1σ threshold {:.2}", metric, count, threshold)
            }
            Self::Rule15InRowWithin1Sigma { metric, count, threshold, .. } => {
                format!("{}: {} points in a row within 1σ (plateau/stagnation) threshold {:.2}", metric, count, threshold)
            }
        }
    }
}

/// Western Electric analyzer for a single metric.
pub struct WesternElectricAnalyzer {
    /// Baseline mean
    pub baseline_mean: f64,

    /// Baseline standard deviation
    pub baseline_stddev: f64,

    /// Rolling window of measurements
    pub rolling_window: VecDeque<f64>,

    /// Maximum window size (default: 20)
    pub window_size: usize,

    /// Control limits (lower, upper) at ±3σ
    pub control_limits: (f64, f64),

    /// Detected violations
    pub violations: Vec<QualityViolation>,
}

impl WesternElectricAnalyzer {
    /// Create a new analyzer with baseline.
    pub fn new(baseline_mean: f64, baseline_stddev: f64, window_size: usize) -> Self {
        let lcl = baseline_mean - 3.0 * baseline_stddev;
        let ucl = baseline_mean + 3.0 * baseline_stddev;

        Self {
            baseline_mean,
            baseline_stddev,
            rolling_window: VecDeque::with_capacity(window_size),
            window_size,
            control_limits: (lcl, ucl),
            violations: Vec::new(),
        }
    }

    /// Add a measurement and check all 7 Western Electric rules.
    pub fn add_measurement(&mut self, metric_name: &str, value: f64) {
        self.rolling_window.push_back(value);
        if self.rolling_window.len() > self.window_size {
            self.rolling_window.pop_front();
        }

        // Check all 7 rules
        if self.baseline_stddev > 0.0 {
            self.check_1_sigma_rule(metric_name, value);
            self.check_9_in_a_row_rule(metric_name);
            self.check_trend_rule(metric_name);
            self.check_alternating_rule(metric_name);
            self.check_2_of_3_rule(metric_name);
            self.check_4_of_5_rule(metric_name);
            self.check_15_in_row_rule(metric_name);
        }
    }

    /// Rule 1: Single point >3σ from mean
    fn check_1_sigma_rule(&mut self, metric: &str, value: f64) {
        let z_score = (value - self.baseline_mean).abs() / self.baseline_stddev;
        if z_score > 3.0 {
            self.violations.push(QualityViolation::Rule1Sigma {
                metric: metric.to_string(),
                value,
                threshold: if value > self.baseline_mean {
                    self.baseline_mean + 3.0 * self.baseline_stddev
                } else {
                    self.baseline_mean - 3.0 * self.baseline_stddev
                },
                z_score,
                severity: "CRITICAL".to_string(),
            });
        }
    }

    /// Rule 2: 9 consecutive out-of-control points
    fn check_9_in_a_row_rule(&mut self, metric: &str) {
        if self.rolling_window.len() < 9 {
            return;
        }

        let mut consecutive_out = 0;
        for &value in self.rolling_window.iter().rev().take(9) {
            if value < self.control_limits.0 || value > self.control_limits.1 {
                consecutive_out += 1;
            }
        }

        if consecutive_out >= 9 {
            self.violations.push(QualityViolation::Rule9InRow {
                metric: metric.to_string(),
                consecutive: consecutive_out,
            });
        }
    }

    /// Rule 3: 6 monotonic points (trend)
    fn check_trend_rule(&mut self, metric: &str) {
        if self.rolling_window.len() < 6 {
            return;
        }

        let last_6: Vec<f64> = self.rolling_window.iter().rev().take(6).copied().collect();
        if last_6.is_empty() {
            return;
        }

        let mut increasing = true;
        let mut decreasing = true;

        for i in 1..last_6.len() {
            if last_6[i] <= last_6[i - 1] {
                increasing = false;
            }
            if last_6[i] >= last_6[i - 1] {
                decreasing = false;
            }
        }

        if increasing {
            self.violations.push(QualityViolation::RuleTrend {
                metric: metric.to_string(),
                direction: "increasing".to_string(),
                count: 6,
            });
        } else if decreasing {
            self.violations.push(QualityViolation::RuleTrend {
                metric: metric.to_string(),
                direction: "decreasing".to_string(),
                count: 6,
            });
        }
    }

    /// Rule 4: Alternating pattern (wild swings)
    fn check_alternating_rule(&mut self, metric: &str) {
        if self.rolling_window.len() < 8 {
            return;
        }

        let last_8: Vec<f64> = self.rolling_window.iter().rev().take(8).copied().collect();
        if last_8.is_empty() {
            return;
        }

        let mut alternations = 0;
        for i in 1..last_8.len() {
            if (last_8[i] > self.baseline_mean) != (last_8[i - 1] > self.baseline_mean) {
                alternations += 1;
            }
        }

        if alternations >= 7 {
            self.violations.push(QualityViolation::RuleAlternating {
                metric: metric.to_string(),
                oscillations: alternations,
            });
        }
    }

    /// Rule 5: 2 of 3 points beyond 2σ
    fn check_2_of_3_rule(&mut self, metric: &str) {
        if self.rolling_window.len() < 3 {
            return;
        }

        let last_3: Vec<f64> = self.rolling_window.iter().rev().take(3).copied().collect();
        let beyond_2sigma = last_3.iter().filter(|&&v| {
            let z = (v - self.baseline_mean).abs() / self.baseline_stddev;
            z > 2.0
        }).count();

        if beyond_2sigma >= 2 {
            self.violations.push(QualityViolation::Rule2of3Beyond2Sigma {
                metric: metric.to_string(),
                count: beyond_2sigma,
                threshold: self.baseline_mean + 2.0 * self.baseline_stddev,
            });
        }
    }

    /// Rule 6: 4 of 5 points beyond 1σ
    fn check_4_of_5_rule(&mut self, metric: &str) {
        if self.rolling_window.len() < 5 {
            return;
        }

        let last_5: Vec<f64> = self.rolling_window.iter().rev().take(5).copied().collect();
        let beyond_1sigma = last_5.iter().filter(|&&v| {
            let z = (v - self.baseline_mean).abs() / self.baseline_stddev;
            z > 1.0
        }).count();

        if beyond_1sigma >= 4 {
            self.violations.push(QualityViolation::Rule4of5Beyond1Sigma {
                metric: metric.to_string(),
                count: beyond_1sigma,
                threshold: self.baseline_mean + 1.0 * self.baseline_stddev,
            });
        }
    }

    /// Rule 7: 15 in a row within 1σ (plateau/stagnation)
    fn check_15_in_row_rule(&mut self, metric: &str) {
        if self.rolling_window.len() < 15 {
            return;
        }

        let last_15: Vec<f64> = self.rolling_window.iter().rev().take(15).copied().collect();
        let within_1sigma = last_15.iter().filter(|&&v| {
            let z = (v - self.baseline_mean).abs() / self.baseline_stddev;
            z <= 1.0
        }).count();

        if within_1sigma >= 15 {
            self.violations.push(QualityViolation::Rule15InRowWithin1Sigma {
                metric: metric.to_string(),
                count: within_1sigma,
                threshold: self.baseline_mean + 1.0 * self.baseline_stddev,
                severity: "INFO".to_string(),
            });
        }
    }
}

/// Measure code quality from source path.
/// Returns a CodeQualityMetrics snapshot with measurements at this instant.
pub fn measure_code_quality(src_path: &str) -> anyhow::Result<CodeQualityMetrics> {
    use std::fs;
    use std::path::Path;

    let mut metrics = CodeQualityMetrics::default();
    metrics.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let path = Path::new(src_path);

    if !path.exists() {
        return Ok(metrics);
    }

    // Walk the directory and count metrics
    if path.is_dir() {
        let mut file_count = 0;
        let mut total_lines = 0;
        let mut comment_lines = 0;
        let mut stub_count = 0;
        let mut function_count = 0;
        let mut typed_function_count = 0;
        let mut doc_count = 0;
        let mut total_pub_items = 0;

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                file_count += 1;
                if let Ok(content) = fs::read_to_string(path) {
                    // Count lines
                    let lines = content.lines().collect::<Vec<_>>();
                    total_lines += lines.len();

                    // Count comment lines
                    for line in &lines {
                        let trimmed = line.trim();
                        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                            comment_lines += 1;
                        }
                    }

                    // Detect stubs (todo!, unimplemented!, panic!)
                    let stub_pattern = regex::Regex::new(r"\b(todo|unimplemented|panic)!\s*\(")?;
                    stub_count += stub_pattern.find_iter(&content).count();
                    function_count += content.matches("fn ").count();

                    // Type coverage (simple heuristic: count "->" in signatures)
                    typed_function_count += content.matches("fn ").filter(|_| content.contains("->")).count();

                    // Doc coverage (/// comments)
                    doc_count += content.matches("///").count();
                    total_pub_items += content.matches("pub ").count();
                }
            }
        }

        // Compute ratios
        if function_count > 0 {
            metrics.stub_ratio = stub_count as f64 / function_count as f64;
            metrics.type_coverage = typed_function_count as f64 / function_count as f64;
        }

        if total_lines > 0 {
            metrics.comment_ratio = comment_lines as f64 / total_lines as f64;
        }

        if total_pub_items > 0 {
            metrics.doc_coverage = doc_count as f64 / total_pub_items as f64;
        }

        // Try to get clippy warnings
        if let Ok(output) = std::process::Command::new("cargo")
            .args(&["clippy", "--message-format=short"])
            .current_dir(src_path)
            .output()
        {
            let stderr = String::from_utf8_lossy(&output.stderr);
            metrics.clippy_warnings = stderr.matches("warning:").count();
        }

        // Try to get test coverage estimate (count #[test] and #[cfg(test)])
        let test_count = content_with_test_count(src_path)?;
        if test_count.0 > 0 {
            metrics.test_coverage = (test_count.1 as f64 / test_count.0 as f64) * 100.0;
        }
    }

    Ok(metrics)
}

/// Count total functions and test functions
fn content_with_test_count(src_path: &str) -> anyhow::Result<(usize, usize)> {
    use std::fs;
    use std::path::Path;

    let mut total_fns = 0;
    let mut test_fns = 0;

    let path = Path::new(src_path);
    if path.is_dir() {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let p = entry.path();
            if p.is_file() && p.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(p) {
                    total_fns += content.matches("fn ").count();
                    test_fns += content.matches("#[test]").count();
                    test_fns += content.matches("#[cfg(test)]").count();
                }
            }
        }
    }

    Ok((total_fns, test_fns))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_rule_1_sigma() {
        let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
        analyzer.add_measurement("test_metric", 8.5); // z-score > 3
        assert!(!analyzer.violations.is_empty());
        assert!(matches!(analyzer.violations[0], QualityViolation::Rule1Sigma { .. }));
    }

    #[test]
    fn test_rule_9_in_a_row() {
        let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
        for _ in 0..9 {
            analyzer.add_measurement("test_metric", 10.0); // All beyond ucl
        }
        assert!(!analyzer.violations.is_empty());
        assert!(analyzer.violations.iter().any(|v| matches!(v, QualityViolation::Rule9InRow { .. })));
    }

    #[test]
    fn test_rule_trend() {
        let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
        for i in 0..6 {
            analyzer.add_measurement("test_metric", 5.0 + i as f64);
        }
        assert!(!analyzer.violations.is_empty());
        assert!(analyzer.violations.iter().any(|v| matches!(v, QualityViolation::RuleTrend { .. })));
    }

    #[test]
    fn test_rule_alternating() {
        let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);
        let values = vec![3.0, 7.0, 3.0, 7.0, 3.0, 7.0, 3.0, 7.0];
        for v in values {
            analyzer.add_measurement("test_metric", v);
        }
        assert!(!analyzer.violations.is_empty());
        assert!(analyzer.violations.iter().any(|v| matches!(v, QualityViolation::RuleAlternating { .. })));
    }

    // ========================================================================
    // Extended Unit Tests: Rule 2 (2-of-3 beyond 2σ)
    // ========================================================================

    #[test]
    fn test_rule_2_of_3_beyond_2_sigma() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // 2σ threshold = 10 + 2*1 = 12
        analyzer.add_measurement("metric", 13.0); // z-score = 3.0, beyond 2σ
        analyzer.add_measurement("metric", 13.5); // beyond 2σ
        analyzer.add_measurement("metric", 11.0); // within 2σ

        assert!(!analyzer.violations.is_empty());
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule2of3Beyond2Sigma { count: 2, .. })
        });
        assert!(violation.is_some(), "Rule 2-of-3 beyond 2σ should be detected");
    }

    #[test]
    fn test_rule_2_of_3_all_three_beyond_2_sigma() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // All 3 measurements beyond 2σ
        analyzer.add_measurement("metric", 13.0);
        analyzer.add_measurement("metric", 14.0);
        analyzer.add_measurement("metric", 13.5);

        assert!(!analyzer.violations.is_empty());
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule2of3Beyond2Sigma { count: 3, .. })
        });
        assert!(violation.is_some());
    }

    #[test]
    fn test_rule_2_of_3_only_one_beyond_threshold() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // Only 1 of 3 beyond 2σ should not trigger
        analyzer.add_measurement("metric", 13.0);
        analyzer.add_measurement("metric", 11.0);
        analyzer.add_measurement("metric", 11.5);

        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule2of3Beyond2Sigma { .. })
        });
        assert!(violation.is_none(), "Should not trigger with only 1 of 3");
    }

    // ========================================================================
    // Extended Unit Tests: Rule 4 (4-of-5 beyond 1σ)
    // ========================================================================

    #[test]
    fn test_rule_4_of_5_beyond_1_sigma() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // 1σ threshold = 10 + 1*1 = 11
        analyzer.add_measurement("metric", 11.5);
        analyzer.add_measurement("metric", 12.0);
        analyzer.add_measurement("metric", 11.2);
        analyzer.add_measurement("metric", 11.8);
        analyzer.add_measurement("metric", 10.5); // within 1σ

        assert!(!analyzer.violations.is_empty());
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule4of5Beyond1Sigma { count: 4, .. })
        });
        assert!(violation.is_some(), "Rule 4-of-5 beyond 1σ should be detected");
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
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule4of5Beyond1Sigma { count: 5, .. })
        });
        assert!(violation.is_some());
    }

    #[test]
    fn test_rule_4_of_5_only_three_beyond_threshold() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // Only 3 of 5 beyond 1σ should not trigger
        analyzer.add_measurement("metric", 11.5);
        analyzer.add_measurement("metric", 12.0);
        analyzer.add_measurement("metric", 11.2);
        analyzer.add_measurement("metric", 10.0); // within
        analyzer.add_measurement("metric", 10.5); // within

        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule4of5Beyond1Sigma { .. })
        });
        assert!(violation.is_none(), "Should not trigger with only 3 of 5");
    }

    // ========================================================================
    // Extended Unit Tests: Rule 7 (15-in-a-row within 1σ)
    // ========================================================================

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
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule15InRowWithin1Sigma { count: 15, .. })
        });
        assert!(violation.is_some(), "Rule 15-in-a-row within 1σ should be detected");
    }

    #[test]
    fn test_rule_15_in_row_with_14_measurements() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // Only 14 within 1σ should not trigger
        for i in 0..14 {
            let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.8;
            analyzer.add_measurement("metric", value);
        }

        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule15InRowWithin1Sigma { .. })
        });
        assert!(violation.is_none(), "Should not trigger with only 14 measurements");
    }

    #[test]
    fn test_rule_15_in_row_breaks_with_outlier() {
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, 20);
        // 14 within, then 1 outlier should not trigger 15-in-row
        for i in 0..14 {
            let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.8;
            analyzer.add_measurement("metric", value);
        }
        analyzer.add_measurement("metric", 20.0); // Far outlier

        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule15InRowWithin1Sigma { .. })
        });
        assert!(violation.is_none(), "Sequence broken by outlier should not trigger");
    }

    // ========================================================================
    // Extended Unit Tests: measure_code_quality with stubs
    // ========================================================================

    fn create_test_rs_file(dir: &Path, name: &str, content: &str) -> std::io::Result<()> {
        let path = dir.join(name);
        fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn test_measure_code_quality_with_stubs() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("create src dir");

        // Create a file with known stubs
        create_test_rs_file(
            &src_dir,
            "stub_test.rs",
            r#"
pub fn feature1() {
    todo!("implement feature1")
}

pub fn feature2() {
    unimplemented!("feature2 not ready")
}

pub fn feature3() {
    panic!("unreachable")
}

pub fn good_feature() -> i32 {
    42
}

pub fn another_good() -> String {
    "test".to_string()
}

#[test]
fn test_stubs() {
    assert!(true);
}
"#,
        ).expect("write test file");

        let metrics = measure_code_quality(src_dir.to_str().unwrap())
            .expect("measure code quality");

        // Should detect stubs: 3 stubs / ~7 functions = ~0.43
        assert!(metrics.stub_ratio > 0.0, "stub_ratio should be > 0: {}", metrics.stub_ratio);
    }

    // ========================================================================
    // Extended Unit Tests: Baseline Bootstrap
    // ========================================================================

    #[test]
    fn test_baseline_bootstrap_mean_calculation() {
        let baseline_mean = 25.0;
        let baseline_stddev = 3.0;
        let analyzer = WesternElectricAnalyzer::new(baseline_mean, baseline_stddev, 20);

        assert_eq!(analyzer.baseline_mean, baseline_mean);
    }

    #[test]
    fn test_baseline_bootstrap_stddev_calculation() {
        let baseline_mean = 25.0;
        let baseline_stddev = 3.0;
        let analyzer = WesternElectricAnalyzer::new(baseline_mean, baseline_stddev, 20);

        assert_eq!(analyzer.baseline_stddev, baseline_stddev);
    }

    #[test]
    fn test_baseline_bootstrap_control_limits() {
        let baseline_mean = 50.0;
        let baseline_stddev = 5.0;
        let analyzer = WesternElectricAnalyzer::new(baseline_mean, baseline_stddev, 20);

        let expected_lcl = 50.0 - 3.0 * 5.0; // 35
        let expected_ucl = 50.0 + 3.0 * 5.0; // 65

        assert_eq!(analyzer.control_limits.0, expected_lcl);
        assert_eq!(analyzer.control_limits.1, expected_ucl);
    }

    #[test]
    fn test_baseline_bootstrap_window_size_respected() {
        let window_size = 15;
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, window_size);

        for i in 0..20 {
            analyzer.add_measurement("metric", 10.0 + i as f64);
        }

        // Window should not exceed specified size
        assert!(analyzer.rolling_window.len() <= window_size);
    }

    #[test]
    fn test_baseline_bootstrap_fills_window_gradually() {
        let window_size = 10;
        let mut analyzer = WesternElectricAnalyzer::new(10.0, 1.0, window_size);

        for i in 0..5 {
            analyzer.add_measurement("metric", 10.0 + i as f64);
            assert_eq!(analyzer.rolling_window.len(), i + 1);
        }
    }

    // ========================================================================
    // Extended Unit Tests: Properties and Edge Cases
    // ========================================================================

    #[test]
    fn test_code_quality_metrics_default_values() {
        let metrics = CodeQualityMetrics::default();

        assert_eq!(metrics.stub_ratio, 0.0);
        assert_eq!(metrics.type_coverage, 1.0);
        assert_eq!(metrics.comment_ratio, 0.2);
        assert_eq!(metrics.test_coverage, 90.0);
        assert_eq!(metrics.doc_coverage, 0.8);
    }

    #[test]
    fn test_quality_violation_severity_accessors() {
        let v1 = QualityViolation::Rule1Sigma {
            metric: "m".to_string(),
            value: 1.0,
            threshold: 2.0,
            z_score: 5.0,
            severity: "CRITICAL".to_string(),
        };
        assert_eq!(v1.severity(), "CRITICAL");

        let v2 = QualityViolation::Rule9InRow {
            metric: "m".to_string(),
            consecutive: 9,
        };
        assert_eq!(v2.severity(), "CRITICAL");

        let v3 = QualityViolation::Rule4of5Beyond1Sigma {
            metric: "m".to_string(),
            count: 4,
            threshold: 1.0,
        };
        assert_eq!(v3.severity(), "MEDIUM");
    }

    #[test]
    fn test_analyzer_with_zero_stddev_no_violations() {
        let mut analyzer = WesternElectricAnalyzer::new(5.0, 0.0, 20);

        // With zero stddev, add_measurement checks baseline_stddev > 0.0
        // and skips all rule checks, so no violations should occur
        analyzer.add_measurement("metric", 5.0);
        analyzer.add_measurement("metric", 10.0);

        assert!(analyzer.violations.is_empty(), "Zero stddev should produce no violations");
    }

    #[test]
    fn test_analyzer_with_large_baseline_mean() {
        let mut analyzer = WesternElectricAnalyzer::new(1000000.0, 10000.0, 20);
        // Spike at mean + 4σ
        analyzer.add_measurement("large_metric", 1050000.0);

        assert!(!analyzer.violations.is_empty());
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule1Sigma { .. })
        });
        assert!(violation.is_some(), "Should detect spike even with large values");
    }

    #[test]
    fn test_analyzer_with_negative_mean_and_values() {
        let mut analyzer = WesternElectricAnalyzer::new(-10.0, 2.0, 20);
        // value = -5 => z-score = (-5 - (-10)) / 2.0 = 2.5 (within 3σ)
        // value = 10 => z-score = (10 - (-10)) / 2.0 = 10.0 > 3.0
        analyzer.add_measurement("negative_metric", 10.0);

        assert!(!analyzer.violations.is_empty());
        let violation = analyzer.violations.iter().find(|v| {
            matches!(v, QualityViolation::Rule1Sigma { .. })
        });
        assert!(violation.is_some());
    }

    #[test]
    fn test_multiple_violations_accumulate() {
        let mut analyzer = WesternElectricAnalyzer::new(5.0, 1.0, 20);

        // Trigger Rule1Sigma with first measurement
        analyzer.add_measurement("metric1", 9.5);
        let initial_count = analyzer.violations.len();
        assert!(initial_count > 0);

        // Trigger another violation
        analyzer.add_measurement("metric2", 1.0);
        assert!(analyzer.violations.len() > initial_count);
    }

    #[test]
    fn test_violation_description_contains_metric_name() {
        let violation = QualityViolation::Rule1Sigma {
            metric: "type_coverage".to_string(),
            value: 0.5,
            threshold: 0.8,
            z_score: 3.5,
            severity: "CRITICAL".to_string(),
        };

        let description = violation.description();
        assert!(description.contains("type_coverage"));
    }

    #[test]
    fn test_violation_metric_accessor_all_types() {
        let rules = vec![
            (QualityViolation::Rule1Sigma {
                metric: "stub_ratio".to_string(),
                value: 1.0,
                threshold: 2.0,
                z_score: 4.0,
                severity: "CRITICAL".to_string(),
            }, "stub_ratio"),
            (QualityViolation::Rule9InRow {
                metric: "cyclo".to_string(),
                consecutive: 9,
            }, "cyclo"),
            (QualityViolation::RuleTrend {
                metric: "churn".to_string(),
                direction: "increasing".to_string(),
                count: 6,
            }, "churn"),
            (QualityViolation::RuleAlternating {
                metric: "alt".to_string(),
                oscillations: 8,
            }, "alt"),
        ];

        for (violation, expected_metric) in rules {
            assert_eq!(violation.metric(), expected_metric);
        }
    }
}

/// Continuous file-watching daemon for real-time quality measurement (Phase 2).
///
/// This module provides file-watching capabilities to emit quality-measurement events
/// on source code changes, with debouncing and periodic interval checking.
/// Available only when the `file-watch` feature is enabled.
#[cfg(feature = "file-watch")]
pub mod file_watcher {
    use std::path::{Path, PathBuf};
    use std::time::Duration;
    use std::sync::mpsc::Receiver;
    use anyhow::Result;

    /// Notification event from file watcher.
    #[derive(Debug, Clone)]
    pub enum Notification {
        /// File or directory modified
        FileChanged(PathBuf),
        /// Measurement interval elapsed
        IntervalElapsed,
        /// Watcher error
        Error(String),
    }

    /// File watcher struct for continuous directory monitoring.
    pub struct FileWatcher {
        /// Path being watched
        pub path: PathBuf,
        /// Receiver channel for notifications
        rx: Receiver<Notification>,
        /// Last measurement timestamp (for debouncing)
        last_measure_time: std::time::Instant,
        /// Debounce delay in milliseconds
        debounce_delay_ms: u64,
    }

    impl FileWatcher {
        /// Create a new file watcher for the given directory path.
        ///
        /// # Arguments
        ///
        /// * `path` - Directory path to watch (typically "src/")
        ///
        /// # Errors
        ///
        /// Returns an error if the path doesn't exist or notify watch creation fails.
        pub fn new(path: &str, debounce_delay_ms: u64) -> Result<Self> {
            use notify::{Watcher, RecursiveMode};
            use std::sync::mpsc;

            let path_buf = PathBuf::from(path);
            if !path_buf.exists() {
                anyhow::bail!("Watch path does not exist: {}", path);
            }

            let (tx, rx) = mpsc::channel();

            // Create watcher with simple file-change handler
            let mut watcher = notify::recommended_watcher(move |res| {
                match res {
                    Ok(event) => {
                        // Only report file modifications
                        use notify::EventKind;
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                if let Some(path) = event.paths.first() {
                                    let _ = tx.send(Notification::FileChanged(path.clone()));
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Notification::Error(e.to_string()));
                    }
                }
            })?;

            // Watch the directory recursively
            watcher.watch(&path_buf, RecursiveMode::Recursive)?;

            // Keep watcher alive by leaking it (alternative: store in Arc<Mutex<>>)
            std::mem::forget(watcher);

            Ok(Self {
                path: path_buf,
                rx,
                last_measure_time: std::time::Instant::now(),
                debounce_delay_ms,
            })
        }

        /// Watch directory and emit quality-measurement events on file changes.
        ///
        /// This method blocks and listens for file notifications. On each notification:
        /// - If debounce delay has elapsed, emits a quality-measurement event.
        /// - Otherwise, skips the measurement to avoid rapid-fire events.
        ///
        /// Returns when receiver closes or on error.
        pub fn run_watch_loop(&mut self) -> Result<()> {
            loop {
                match self.rx.recv() {
                    Ok(Notification::FileChanged(path)) => {
                        let elapsed = self.last_measure_time.elapsed();
                        let debounce_duration = Duration::from_millis(self.debounce_delay_ms);

                        if elapsed >= debounce_duration {
                            eprintln!(
                                "[FileWatcher] Detected change: {}",
                                path.display()
                            );

                            // Measure quality
                            match crate::quality::measure_code_quality(self.path.to_str().unwrap_or("src")) {
                                Ok(metrics) => {
                                    eprintln!(
                                        "[Quality] stub_ratio={:.2}, type_coverage={:.2}, clippy_warnings={}",
                                        metrics.stub_ratio,
                                        metrics.type_coverage,
                                        metrics.clippy_warnings
                                    );
                                    self.last_measure_time = std::time::Instant::now();
                                }
                                Err(e) => {
                                    eprintln!("[Quality] Measurement failed: {}", e);
                                }
                            }
                        }
                    }
                    Ok(Notification::IntervalElapsed) => {
                        eprintln!("[FileWatcher] Periodic interval elapsed");
                        match crate::quality::measure_code_quality(self.path.to_str().unwrap_or("src")) {
                            Ok(metrics) => {
                                eprintln!(
                                    "[Quality] stub_ratio={:.2}, type_coverage={:.2}, clippy_warnings={}",
                                    metrics.stub_ratio,
                                    metrics.type_coverage,
                                    metrics.clippy_warnings
                                );
                            }
                            Err(e) => {
                                eprintln!("[Quality] Measurement failed: {}", e);
                            }
                        }
                    }
                    Ok(Notification::Error(msg)) => {
                        eprintln!("[FileWatcher] Error: {}", msg);
                    }
                    Err(_) => {
                        eprintln!("[FileWatcher] Channel closed, exiting watch loop");
                        break;
                    }
                }
            }

            Ok(())
        }
    }

    /// Spawn an async watch loop with optional interval-based measurement.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory path to watch (e.g., "src/")
    /// * `interval_secs` - Periodic measurement interval in seconds (0 = no interval)
    ///
    /// # Returns
    ///
    /// A tokio task handle that runs the watch loop.
    ///
    /// # Errors
    ///
    /// Returns an error if file watcher initialization fails.
    #[cfg(feature = "tokio")]
    pub async fn run_watch_loop_async(
        path: &str,
        interval_secs: u64,
    ) -> Result<tokio::task::JoinHandle<Result<()>>> {
        use std::time::Duration;

        let path_owned = path.to_string();

        let handle = tokio::spawn(async move {
            let mut watcher = FileWatcher::new(&path_owned, 1000)?; // 1-second debounce

            if interval_secs > 0 {
                // Spawn interval task alongside the watch loop
                let path_for_interval = path_owned.clone();
                let interval_handle = tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
                    loop {
                        interval.tick().await;
                        match crate::quality::measure_code_quality(&path_for_interval) {
                            Ok(metrics) => {
                                eprintln!(
                                    "[Quality/Interval] stub_ratio={:.2}, type_coverage={:.2}, clippy_warnings={}",
                                    metrics.stub_ratio,
                                    metrics.type_coverage,
                                    metrics.clippy_warnings
                                );
                            }
                            Err(e) => {
                                eprintln!("[Quality/Interval] Measurement failed: {}", e);
                            }
                        }
                    }
                });

                // Run the main watch loop
                let result = watcher.run_watch_loop();

                // Cancel interval task if watch loop exits
                interval_handle.abort();

                result
            } else {
                // Just run the watch loop without interval
                watcher.run_watch_loop()
            }
        });

        Ok(handle)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_notification_creation() {
            let notif = Notification::FileChanged(PathBuf::from("src/lib.rs"));
            match notif {
                Notification::FileChanged(p) => assert_eq!(p.to_str(), Some("src/lib.rs")),
                _ => panic!("Expected FileChanged"),
            }
        }

        #[test]
        fn test_watcher_path_validation() {
            // Non-existent path should fail
            let result = FileWatcher::new("/nonexistent/path/xyz", 1000);
            assert!(result.is_err());
        }

        #[test]
        fn test_watcher_valid_path() {
            // Watching a real path (e.g., current dir) should succeed
            let result = FileWatcher::new(".", 1000);
            assert!(result.is_ok());
        }
    }
}
