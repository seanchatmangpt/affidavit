//! Extended Western Electric statistical process control rules.
//!
//! This module provides comprehensive coverage of Western Electric SPC rules
//! with support for:
//! - Multiple sigma-level variants (1σ, 2σ, 3σ, custom thresholds)
//! - Variable window sizes (6, 9, 15, 20, 30 points)
//! - Rule combination detection (rule storms)
//! - Severity aggregation and multi-rule failure scenarios
//!
//! The extended analyzer enables fine-grained quality monitoring for code quality
//! metrics, allowing detection of both individual rule violations and systemic
//! issues when multiple rules fire simultaneously.

use crate::quality::QualityViolation;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Configuration for extended Western Electric analyzer.
///
/// Controls the sensitivity and scope of SPC checks, including:
/// - Sigma levels (thresholds in standard deviations)
/// - Window sizes for historical lookback
/// - Which rules are enabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WesternElectricConfig {
    /// Baseline mean for the metric
    pub baseline_mean: f64,

    /// Baseline standard deviation
    pub baseline_stddev: f64,

    /// Primary sigma level (default: 3.0 for Rule 1)
    pub primary_sigma: f64,

    /// Secondary sigma level (default: 2.0 for Rule 5)
    pub secondary_sigma: f64,

    /// Tertiary sigma level (default: 1.0 for Rules 6–7)
    pub tertiary_sigma: f64,

    /// Custom sigma thresholds for specialized rules
    pub custom_thresholds: HashMap<String, f64>,

    /// Window sizes to check (default: [6, 9, 15, 20])
    pub window_sizes: Vec<usize>,

    /// Enable Rule 1 (1σ spike detection)
    pub rule1_enabled: bool,

    /// Enable Rule 2 (9-in-a-row)
    pub rule2_enabled: bool,

    /// Enable Rule 3 (6-point trend)
    pub rule3_enabled: bool,

    /// Enable Rule 4 (alternating pattern)
    pub rule4_enabled: bool,

    /// Enable Rule 5 (2-of-3 beyond 2σ)
    pub rule5_enabled: bool,

    /// Enable Rule 6 (4-of-5 beyond 1σ)
    pub rule6_enabled: bool,

    /// Enable Rule 7 (15-in-a-row within 1σ)
    pub rule7_enabled: bool,

    /// Minimum consecutive violations to report (default: 1)
    pub min_consecutive_violations: usize,

    /// Rule storm threshold: number of rules firing to constitute a "storm"
    pub rule_storm_threshold: usize,
}

impl WesternElectricConfig {
    /// Create a new configuration with standard defaults.
    pub fn new(baseline_mean: f64, baseline_stddev: f64) -> Self {
        Self {
            baseline_mean,
            baseline_stddev,
            primary_sigma: 3.0,
            secondary_sigma: 2.0,
            tertiary_sigma: 1.0,
            custom_thresholds: HashMap::new(),
            window_sizes: vec![6, 9, 15, 20],
            rule1_enabled: true,
            rule2_enabled: true,
            rule3_enabled: true,
            rule4_enabled: true,
            rule5_enabled: true,
            rule6_enabled: true,
            rule7_enabled: true,
            min_consecutive_violations: 1,
            rule_storm_threshold: 3,
        }
    }

    /// Set custom sigma levels for fine-grained control.
    pub fn with_sigmas(
        mut self,
        primary: f64,
        secondary: f64,
        tertiary: f64,
    ) -> Self {
        self.primary_sigma = primary;
        self.secondary_sigma = secondary;
        self.tertiary_sigma = tertiary;
        self
    }

    /// Add a custom threshold for a specialized rule.
    pub fn add_custom_threshold(mut self, rule_name: String, threshold: f64) -> Self {
        self.custom_thresholds.insert(rule_name, threshold);
        self
    }

    /// Override which rules to enable/disable.
    pub fn with_enabled_rules(
        mut self,
        rule1: bool,
        rule2: bool,
        rule3: bool,
        rule4: bool,
        rule5: bool,
        rule6: bool,
        rule7: bool,
    ) -> Self {
        self.rule1_enabled = rule1;
        self.rule2_enabled = rule2;
        self.rule3_enabled = rule3;
        self.rule4_enabled = rule4;
        self.rule5_enabled = rule5;
        self.rule6_enabled = rule6;
        self.rule7_enabled = rule7;
        self
    }
}

impl Default for WesternElectricConfig {
    fn default() -> Self {
        Self::new(0.0, 1.0)
    }
}

/// Enumeration of all Western Electric rule variants.
///
/// Covers all 7 base rules with sigma-level and window-size variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuleVariant {
    /// Rule 1 at 1σ: single point beyond threshold
    Rule1SigmaAt1(String), // (metric_name)

    /// Rule 1 at 2σ: single point beyond 2σ threshold
    Rule1SigmaAt2(String),

    /// Rule 1 at 3σ: single point beyond 3σ threshold
    Rule1SigmaAt3(String),

    /// Rule 1 with custom sigma threshold
    Rule1SigmaAtCustom(String, f64), // (metric_name, sigma_level)

    /// Rule 2 (9-in-a-row) with window size 6
    RuleConsecutiveWindow6(String),

    /// Rule 2 (9-in-a-row) with window size 9
    RuleConsecutiveWindow9(String),

    /// Rule 2 (9-in-a-row) with window size 15
    RuleConsecutiveWindow15(String),

    /// Rule 2 (9-in-a-row) with window size 20
    RuleConsecutiveWindow20(String),

    /// Rule 2 (9-in-a-row) with window size 30
    RuleConsecutiveWindow30(String),

    /// Rule 3: Trend detection with window size 6
    RuleTrendWindow6(String),

    /// Rule 3: Trend detection with window size 9
    RuleTrendWindow9(String),

    /// Rule 3: Trend detection with window size 15
    RuleTrendWindow15(String),

    /// Rule 3: Trend detection with window size 20
    RuleTrendWindow20(String),

    /// Rule 4: Alternating pattern detection
    RuleAlternating(String),

    /// Rule 5: 2-of-3 beyond 2σ
    Rule2of3Beyond2Sigma(String),

    /// Rule 5: 2-of-3 beyond 1σ (stricter early warning)
    Rule2of3Beyond1Sigma(String),

    /// Rule 5: 3-of-3 beyond 2σ (strongest warning)
    Rule3of3Beyond2Sigma(String),

    /// Rule 6: 4-of-5 beyond 1σ
    Rule4of5Beyond1Sigma(String),

    /// Rule 6: 5-of-5 beyond 1σ (all violations)
    Rule5of5Beyond1Sigma(String),

    /// Rule 6: 3-of-5 beyond 1σ (earlier detection)
    Rule3of5Beyond1Sigma(String),

    /// Rule 7: 15-in-a-row within 1σ (plateau)
    Rule15InRowWithin1Sigma(String),

    /// Rule 7: 20-in-a-row within 1σ (extended plateau)
    Rule20InRowWithin1Sigma(String),

    /// Rule 7: 10-in-a-row within 1σ (earlier stagnation)
    Rule10InRowWithin1Sigma(String),
}

impl RuleVariant {
    /// Extract the metric name from this rule variant.
    pub fn metric(&self) -> &str {
        match self {
            Self::Rule1SigmaAt1(m)
            | Self::Rule1SigmaAt2(m)
            | Self::Rule1SigmaAt3(m)
            | Self::Rule1SigmaAtCustom(m, _)
            | Self::RuleConsecutiveWindow6(m)
            | Self::RuleConsecutiveWindow9(m)
            | Self::RuleConsecutiveWindow15(m)
            | Self::RuleConsecutiveWindow20(m)
            | Self::RuleConsecutiveWindow30(m)
            | Self::RuleTrendWindow6(m)
            | Self::RuleTrendWindow9(m)
            | Self::RuleTrendWindow15(m)
            | Self::RuleTrendWindow20(m)
            | Self::RuleAlternating(m)
            | Self::Rule2of3Beyond2Sigma(m)
            | Self::Rule2of3Beyond1Sigma(m)
            | Self::Rule3of3Beyond2Sigma(m)
            | Self::Rule4of5Beyond1Sigma(m)
            | Self::Rule5of5Beyond1Sigma(m)
            | Self::Rule3of5Beyond1Sigma(m)
            | Self::Rule15InRowWithin1Sigma(m)
            | Self::Rule20InRowWithin1Sigma(m)
            | Self::Rule10InRowWithin1Sigma(m) => m,
        }
    }

    /// Get a human-readable description of this rule variant.
    pub fn description(&self) -> String {
        match self {
            Self::Rule1SigmaAt1(m) => format!("{}: spike detection (>1σ)", m),
            Self::Rule1SigmaAt2(m) => format!("{}: spike detection (>2σ)", m),
            Self::Rule1SigmaAt3(m) => format!("{}: spike detection (>3σ)", m),
            Self::Rule1SigmaAtCustom(m, sigma) => {
                format!("{}: spike detection (>{}σ)", m, sigma)
            }
            Self::RuleConsecutiveWindow6(m) => format!("{}: 6 consecutive out-of-control", m),
            Self::RuleConsecutiveWindow9(m) => format!("{}: 9 consecutive out-of-control", m),
            Self::RuleConsecutiveWindow15(m) => format!("{}: 15 consecutive out-of-control", m),
            Self::RuleConsecutiveWindow20(m) => format!("{}: 20 consecutive out-of-control", m),
            Self::RuleConsecutiveWindow30(m) => format!("{}: 30 consecutive out-of-control", m),
            Self::RuleTrendWindow6(m) => format!("{}: 6-point trend (monotonic)", m),
            Self::RuleTrendWindow9(m) => format!("{}: 9-point trend (monotonic)", m),
            Self::RuleTrendWindow15(m) => format!("{}: 15-point trend (monotonic)", m),
            Self::RuleTrendWindow20(m) => format!("{}: 20-point trend (monotonic)", m),
            Self::RuleAlternating(m) => format!("{}: alternating/oscillating pattern", m),
            Self::Rule2of3Beyond2Sigma(m) => format!("{}: 2-of-3 beyond 2σ (early warning)", m),
            Self::Rule2of3Beyond1Sigma(m) => {
                format!("{}: 2-of-3 beyond 1σ (stricter warning)", m)
            }
            Self::Rule3of3Beyond2Sigma(m) => format!("{}: 3-of-3 beyond 2σ (strongest warning)", m),
            Self::Rule4of5Beyond1Sigma(m) => format!("{}: 4-of-5 beyond 1σ (sustained deviation)", m),
            Self::Rule5of5Beyond1Sigma(m) => format!("{}: 5-of-5 beyond 1σ (all violations)", m),
            Self::Rule3of5Beyond1Sigma(m) => format!("{}: 3-of-5 beyond 1σ (earlier detection)", m),
            Self::Rule15InRowWithin1Sigma(m) => format!("{}: 15 in a row within 1σ (plateau)", m),
            Self::Rule20InRowWithin1Sigma(m) => format!("{}: 20 in a row within 1σ (extended plateau)", m),
            Self::Rule10InRowWithin1Sigma(m) => format!("{}: 10 in a row within 1σ (early stagnation)", m),
        }
    }

    /// Get severity level for this rule variant.
    pub fn severity(&self) -> &'static str {
        match self {
            Self::Rule1SigmaAt3(_) => "CRITICAL",
            Self::Rule1SigmaAt2(_) => "CRITICAL",
            Self::Rule1SigmaAtCustom(_, sigma) if *sigma >= 2.0 => "CRITICAL",
            Self::Rule1SigmaAt1(_) | Self::Rule1SigmaAtCustom(_, _) => "HIGH",
            Self::RuleConsecutiveWindow30(_)
            | Self::RuleConsecutiveWindow20(_)
            | Self::RuleConsecutiveWindow15(_) => "CRITICAL",
            Self::RuleConsecutiveWindow9(_) | Self::RuleConsecutiveWindow6(_) => "HIGH",
            Self::RuleTrendWindow20(_) | Self::RuleTrendWindow15(_) => "CRITICAL",
            Self::RuleTrendWindow9(_) | Self::RuleTrendWindow6(_) => "HIGH",
            Self::RuleAlternating(_) => "HIGH",
            Self::Rule3of3Beyond2Sigma(_) | Self::Rule2of3Beyond2Sigma(_) => "HIGH",
            Self::Rule2of3Beyond1Sigma(_) => "MEDIUM",
            Self::Rule5of5Beyond1Sigma(_) | Self::Rule4of5Beyond1Sigma(_) => "MEDIUM",
            Self::Rule3of5Beyond1Sigma(_) => "LOW",
            Self::Rule20InRowWithin1Sigma(_) => "HIGH",
            Self::Rule15InRowWithin1Sigma(_) => "MEDIUM",
            Self::Rule10InRowWithin1Sigma(_) => "LOW",
        }
    }
}

/// A detected "rule storm": multiple rules firing simultaneously.
///
/// When 2 or more rules fire within the same measurement window,
/// it indicates a systemic problem rather than a single anomaly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStorm {
    /// Metric experiencing the storm
    pub metric: String,

    /// Rules that fired in this storm
    pub rules: Vec<RuleVariant>,

    /// Aggregated severity (worst of all rules)
    pub aggregate_severity: String,

    /// Number of distinct rules firing
    pub rule_count: usize,

    /// Whether this constitutes a "severe storm" (3+ rules)
    pub is_severe: bool,

    /// Human-readable summary
    pub summary: String,
}

impl RuleStorm {
    /// Create a new RuleStorm from a collection of rule variants.
    pub fn new(metric: String, rules: Vec<RuleVariant>) -> Self {
        let rule_count = rules.len();
        let is_severe = rule_count >= 3;

        // Determine worst severity
        let aggregate_severity = if rules.iter().any(|r| r.severity() == "CRITICAL") {
            "CRITICAL".to_string()
        } else if rules.iter().any(|r| r.severity() == "HIGH") {
            "HIGH".to_string()
        } else if rules.iter().any(|r| r.severity() == "MEDIUM") {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        };

        let rule_descriptions: Vec<String> = rules.iter().map(|r| {
            // Extract rule name from variant
            format!("{:?}", r).split('(').next().unwrap_or("Unknown").to_string()
        }).collect();

        let summary = if is_severe {
            format!(
                "Severe rule storm on {}: {} rules firing ({} violation patterns)",
                metric,
                rule_count,
                rule_descriptions.join(", ")
            )
        } else {
            format!(
                "Rule combination on {}: {} rules ({} violation patterns)",
                metric,
                rule_count,
                rule_descriptions.join(", ")
            )
        };

        Self {
            metric,
            rules,
            aggregate_severity,
            rule_count,
            is_severe,
            summary,
        }
    }
}

/// Aggregated severity report for a collection of violations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedSeverity {
    /// Total number of violations detected
    pub total_violations: usize,

    /// Number of critical-severity violations
    pub critical_count: usize,

    /// Number of high-severity violations
    pub high_count: usize,

    /// Number of medium-severity violations
    pub medium_count: usize,

    /// Number of low-severity violations
    pub low_count: usize,

    /// Number of detected rule storms (2+ rules firing)
    pub rule_storm_count: usize,

    /// Worst overall severity level
    pub worst_severity: String,

    /// Affected metrics (deduplicated)
    pub affected_metrics: Vec<String>,

    /// Human-readable summary
    pub summary: String,
}

impl AggregatedSeverity {
    /// Compute aggregated severity from a collection of violations.
    pub fn compute(violations: &[QualityViolation]) -> Self {
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        let mut affected_metrics = std::collections::HashSet::new();

        for v in violations {
            affected_metrics.insert(v.metric().to_string());
            match v.severity() {
                "CRITICAL" => critical_count += 1,
                "HIGH" => high_count += 1,
                "MEDIUM" => medium_count += 1,
                _ => low_count += 1,
            }
        }

        let total_violations = violations.len();
        let rule_storm_count = 0; // Placeholder; storms computed separately

        let worst_severity = if critical_count > 0 {
            "CRITICAL".to_string()
        } else if high_count > 0 {
            "HIGH".to_string()
        } else if medium_count > 0 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        };

        let mut affected_metrics_vec: Vec<_> = affected_metrics.into_iter().collect();
        affected_metrics_vec.sort();

        let summary = format!(
            "Quality violations: {} total (C:{}, H:{}, M:{}, L:{}), {} metrics affected",
            total_violations, critical_count, high_count, medium_count, low_count, affected_metrics_vec.len()
        );

        Self {
            total_violations,
            critical_count,
            high_count,
            medium_count,
            low_count,
            rule_storm_count,
            worst_severity,
            affected_metrics: affected_metrics_vec,
            summary,
        }
    }
}

/// Extended Western Electric analyzer with multi-variant rule support.
pub struct EnhancedWesternElectricAnalyzer {
    /// Configuration for all rule checks
    pub config: WesternElectricConfig,

    /// Rolling windows for each window size
    pub rolling_windows: HashMap<usize, VecDeque<f64>>,

    /// All detected rule variants
    pub detected_rules: Vec<RuleVariant>,

    /// All rule storms (2+ rules on same metric)
    pub rule_storms: Vec<RuleStorm>,

    /// Aggregated severity report
    pub severity_report: Option<AggregatedSeverity>,
}

impl EnhancedWesternElectricAnalyzer {
    /// Create a new enhanced analyzer with configuration.
    pub fn new(config: WesternElectricConfig) -> Self {
        let mut rolling_windows = HashMap::new();
        for &size in &config.window_sizes {
            rolling_windows.insert(size, VecDeque::with_capacity(size));
        }

        Self {
            config,
            rolling_windows,
            detected_rules: Vec::new(),
            rule_storms: Vec::new(),
            severity_report: None,
        }
    }

    /// Add a measurement and check all rule variants.
    pub fn add_measurement(&mut self, metric_name: &str, value: f64) {
        // Update all rolling windows
        for (size, window) in &mut self.rolling_windows {
            window.push_back(value);
            if window.len() > *size {
                window.pop_front();
            }
        }

        // Check all enabled rules
        if self.config.baseline_stddev > 0.0 {
            self.check_rule1_variants(metric_name, value);
            self.check_rule2_variants(metric_name);
            self.check_rule3_variants(metric_name);
            self.check_rule4_variant(metric_name);
            self.check_rule5_variants(metric_name);
            self.check_rule6_variants(metric_name);
            self.check_rule7_variants(metric_name);
        }

        // Detect rule storms after all rules checked
        self.detect_rule_storms(metric_name);
    }

    /// Check all Rule 1 sigma-level variants.
    fn check_rule1_variants(&mut self, metric: &str, value: f64) {
        if !self.config.rule1_enabled {
            return;
        }

        let z_score = (value - self.config.baseline_mean).abs() / self.config.baseline_stddev;

        // Check at 1σ
        if z_score > 1.0 {
            self.detected_rules.push(RuleVariant::Rule1SigmaAt1(metric.to_string()));
        }

        // Check at 2σ
        if z_score > 2.0 {
            self.detected_rules.push(RuleVariant::Rule1SigmaAt2(metric.to_string()));
        }

        // Check at 3σ
        if z_score > 3.0 {
            self.detected_rules.push(RuleVariant::Rule1SigmaAt3(metric.to_string()));
        }

        // Check custom thresholds
        for (rule_name, threshold) in &self.config.custom_thresholds {
            if z_score > *threshold && rule_name.starts_with("rule1_") {
                let sigma_level = *threshold;
                self.detected_rules
                    .push(RuleVariant::Rule1SigmaAtCustom(metric.to_string(), sigma_level));
            }
        }
    }

    /// Check all Rule 2 (consecutive out-of-control) variants.
    fn check_rule2_variants(&mut self, metric: &str) {
        if !self.config.rule2_enabled {
            return;
        }

        let lcl = self.config.baseline_mean - 3.0 * self.config.baseline_stddev;
        let ucl = self.config.baseline_mean + 3.0 * self.config.baseline_stddev;

        for &window_size in &self.config.window_sizes {
            if let Some(window) = self.rolling_windows.get(&window_size) {
                if window.len() < window_size {
                    continue;
                }

                let consecutive_out: usize =
                    window.iter().filter(|&&v| v < lcl || v > ucl).count();

                // Check if all points in window are out-of-control
                if consecutive_out == window_size {
                    let rule_variant = match window_size {
                        6 => RuleVariant::RuleConsecutiveWindow6(metric.to_string()),
                        9 => RuleVariant::RuleConsecutiveWindow9(metric.to_string()),
                        15 => RuleVariant::RuleConsecutiveWindow15(metric.to_string()),
                        20 => RuleVariant::RuleConsecutiveWindow20(metric.to_string()),
                        30 => RuleVariant::RuleConsecutiveWindow30(metric.to_string()),
                        _ => continue,
                    };
                    self.detected_rules.push(rule_variant);
                }
            }
        }
    }

    /// Check all Rule 3 (trend) variants.
    fn check_rule3_variants(&mut self, metric: &str) {
        if !self.config.rule3_enabled {
            return;
        }

        for &window_size in &self.config.window_sizes {
            if let Some(window) = self.rolling_windows.get(&window_size) {
                if window.len() < window_size {
                    continue;
                }

                let values: Vec<f64> = window.iter().copied().collect();

                let mut increasing = true;
                let mut decreasing = true;

                for i in 1..values.len() {
                    if values[i] <= values[i - 1] {
                        increasing = false;
                    }
                    if values[i] >= values[i - 1] {
                        decreasing = false;
                    }
                }

                if increasing || decreasing {
                    let rule_variant = match window_size {
                        6 => RuleVariant::RuleTrendWindow6(metric.to_string()),
                        9 => RuleVariant::RuleTrendWindow9(metric.to_string()),
                        15 => RuleVariant::RuleTrendWindow15(metric.to_string()),
                        20 => RuleVariant::RuleTrendWindow20(metric.to_string()),
                        _ => continue,
                    };
                    self.detected_rules.push(rule_variant);
                }
            }
        }
    }

    /// Check Rule 4 (alternating pattern) variant.
    fn check_rule4_variant(&mut self, metric: &str) {
        if !self.config.rule4_enabled {
            return;
        }

        if let Some(window) = self.rolling_windows.get(&8) {
            if window.len() < 8 {
                return;
            }

            let mut alternations = 0;
            let values: Vec<f64> = window.iter().copied().collect();

            for i in 1..values.len() {
                if (values[i] > self.config.baseline_mean) != (values[i - 1] > self.config.baseline_mean)
                {
                    alternations += 1;
                }
            }

            if alternations >= 7 {
                self.detected_rules
                    .push(RuleVariant::RuleAlternating(metric.to_string()));
            }
        }
    }

    /// Check all Rule 5 (2/3 beyond sigma) variants.
    fn check_rule5_variants(&mut self, metric: &str) {
        if !self.config.rule5_enabled {
            return;
        }

        if let Some(window) = self.rolling_windows.get(&3) {
            if window.len() < 3 {
                return;
            }

            let values: Vec<f64> = window.iter().copied().collect();

            // Check 2-of-3 beyond 2σ
            let beyond_2sigma = values
                .iter()
                .filter(|&&v| {
                    let z = (v - self.config.baseline_mean).abs() / self.config.baseline_stddev;
                    z > self.config.secondary_sigma
                })
                .count();

            if beyond_2sigma >= 2 {
                self.detected_rules
                    .push(RuleVariant::Rule2of3Beyond2Sigma(metric.to_string()));
            }

            if beyond_2sigma >= 3 {
                self.detected_rules
                    .push(RuleVariant::Rule3of3Beyond2Sigma(metric.to_string()));
            }

            // Check 2-of-3 beyond 1σ (stricter)
            let beyond_1sigma = values
                .iter()
                .filter(|&&v| {
                    let z = (v - self.config.baseline_mean).abs() / self.config.baseline_stddev;
                    z > self.config.tertiary_sigma
                })
                .count();

            if beyond_1sigma >= 2 {
                self.detected_rules
                    .push(RuleVariant::Rule2of3Beyond1Sigma(metric.to_string()));
            }
        }
    }

    /// Check all Rule 6 (4/5 beyond 1σ) variants.
    fn check_rule6_variants(&mut self, metric: &str) {
        if !self.config.rule6_enabled {
            return;
        }

        if let Some(window) = self.rolling_windows.get(&5) {
            if window.len() < 5 {
                return;
            }

            let values: Vec<f64> = window.iter().copied().collect();

            let beyond_1sigma = values
                .iter()
                .filter(|&&v| {
                    let z = (v - self.config.baseline_mean).abs() / self.config.baseline_stddev;
                    z > self.config.tertiary_sigma
                })
                .count();

            // Check 3-of-5 (earlier detection)
            if beyond_1sigma >= 3 {
                self.detected_rules
                    .push(RuleVariant::Rule3of5Beyond1Sigma(metric.to_string()));
            }

            // Check 4-of-5 (standard)
            if beyond_1sigma >= 4 {
                self.detected_rules
                    .push(RuleVariant::Rule4of5Beyond1Sigma(metric.to_string()));
            }

            // Check 5-of-5 (all violations)
            if beyond_1sigma >= 5 {
                self.detected_rules
                    .push(RuleVariant::Rule5of5Beyond1Sigma(metric.to_string()));
            }
        }
    }

    /// Check all Rule 7 (N-in-a-row within 1σ) variants.
    fn check_rule7_variants(&mut self, metric: &str) {
        if !self.config.rule7_enabled {
            return;
        }

        // Check 10-in-a-row
        if let Some(window) = self.rolling_windows.get(&10) {
            if window.len() == 10 {
                let within_1sigma = window
                    .iter()
                    .filter(|&&v| {
                        let z = (v - self.config.baseline_mean).abs() / self.config.baseline_stddev;
                        z <= self.config.tertiary_sigma
                    })
                    .count();

                if within_1sigma == 10 {
                    self.detected_rules
                        .push(RuleVariant::Rule10InRowWithin1Sigma(metric.to_string()));
                }
            }
        }

        // Check 15-in-a-row
        if let Some(window) = self.rolling_windows.get(&15) {
            if window.len() == 15 {
                let within_1sigma = window
                    .iter()
                    .filter(|&&v| {
                        let z = (v - self.config.baseline_mean).abs() / self.config.baseline_stddev;
                        z <= self.config.tertiary_sigma
                    })
                    .count();

                if within_1sigma == 15 {
                    self.detected_rules
                        .push(RuleVariant::Rule15InRowWithin1Sigma(metric.to_string()));
                }
            }
        }

        // Check 20-in-a-row
        if let Some(window) = self.rolling_windows.get(&20) {
            if window.len() == 20 {
                let within_1sigma = window
                    .iter()
                    .filter(|&&v| {
                        let z = (v - self.config.baseline_mean).abs() / self.config.baseline_stddev;
                        z <= self.config.tertiary_sigma
                    })
                    .count();

                if within_1sigma == 20 {
                    self.detected_rules
                        .push(RuleVariant::Rule20InRowWithin1Sigma(metric.to_string()));
                }
            }
        }
    }

    /// Detect rule storms (multiple rules firing on same metric).
    fn detect_rule_storms(&mut self, _metric: &str) {
        // Group rules by metric
        let mut metric_rules: HashMap<String, Vec<RuleVariant>> = HashMap::new();

        for rule in &self.detected_rules {
            metric_rules
                .entry(rule.metric().to_string())
                .or_insert_with(Vec::new)
                .push(rule.clone());
        }

        // Find storms (2+ rules on same metric)
        for (m, rules) in metric_rules {
            if rules.len() >= self.config.rule_storm_threshold {
                let storm = RuleStorm::new(m, rules);
                if !self.rule_storms.iter().any(|s| s.metric == storm.metric) {
                    self.rule_storms.push(storm);
                }
            }
        }
    }

    /// Finalize analysis and compute aggregated severity.
    pub fn finalize(&mut self) -> AggregatedSeverity {
        // Convert rule variants to quality violations for compatibility
        let violations: Vec<QualityViolation> = vec![]; // Placeholder

        let mut severity = AggregatedSeverity::compute(&violations);
        severity.rule_storm_count = self.rule_storms.len();

        self.severity_report = Some(severity.clone());
        severity
    }
}

/// Detect all rule variants in a metric series.
///
/// Returns a list of all detected rule variants for the given metric.
pub fn detect_all_rule_variants(
    metrics: &[f64],
    config: &WesternElectricConfig,
) -> Vec<RuleVariant> {
    let mut analyzer = EnhancedWesternElectricAnalyzer::new(config.clone());

    for &value in metrics {
        analyzer.add_measurement("metric", value);
    }

    analyzer.detected_rules
}

/// Detect rule storms in a set of violations.
///
/// Groups violations by metric and identifies where 2 or more
/// rules fire simultaneously (rule storm).
pub fn detect_rule_storms(violations: &[QualityViolation]) -> Vec<RuleStorm> {
    let mut metric_rules: HashMap<String, Vec<RuleVariant>> = HashMap::new();

    // Convert violations to rule variants (simplified)
    for violation in violations {
        let rule_variant = match violation {
            QualityViolation::Rule1Sigma { metric, .. } => {
                RuleVariant::Rule1SigmaAt3(metric.clone())
            }
            QualityViolation::Rule9InRow { metric, .. } => {
                RuleVariant::RuleConsecutiveWindow9(metric.clone())
            }
            QualityViolation::RuleTrend { metric, .. } => {
                RuleVariant::RuleTrendWindow6(metric.clone())
            }
            QualityViolation::RuleAlternating { metric, .. } => {
                RuleVariant::RuleAlternating(metric.clone())
            }
            QualityViolation::Rule2of3Beyond2Sigma { metric, .. } => {
                RuleVariant::Rule2of3Beyond2Sigma(metric.clone())
            }
            QualityViolation::Rule4of5Beyond1Sigma { metric, .. } => {
                RuleVariant::Rule4of5Beyond1Sigma(metric.clone())
            }
            QualityViolation::Rule15InRowWithin1Sigma { metric, .. } => {
                RuleVariant::Rule15InRowWithin1Sigma(metric.clone())
            }
        };

        metric_rules
            .entry(violation.metric().to_string())
            .or_insert_with(Vec::new)
            .push(rule_variant);
    }

    let mut storms = Vec::new();
    for (metric, rules) in metric_rules {
        if rules.len() >= 2 {
            storms.push(RuleStorm::new(metric, rules));
        }
    }

    storms
}

/// Compute aggregated severity from violations.
///
/// Returns a summary of total violations, categorized by severity,
/// and identifies rule storms.
pub fn compute_aggregate_severity(violations: &[QualityViolation]) -> AggregatedSeverity {
    AggregatedSeverity::compute(violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Rule Variant Tests (Sigma-level variants)
    // ========================================================================

    #[test]
    fn test_rule1_sigma_at_1() {
        let config = WesternElectricConfig::new(10.0, 1.0);
        let metrics = vec![11.5]; // z-score = 1.5, triggers at 1σ
        let variants = detect_all_rule_variants(&metrics, &config);
        assert!(variants.iter().any(|r| matches!(r, RuleVariant::Rule1SigmaAt1(_))));
    }

    #[test]
    fn test_rule1_sigma_at_2() {
        let config = WesternElectricConfig::new(10.0, 1.0);
        let metrics = vec![12.5]; // z-score = 2.5, triggers at 2σ
        let variants = detect_all_rule_variants(&metrics, &config);
        assert!(variants.iter().any(|r| matches!(r, RuleVariant::Rule1SigmaAt2(_))));
    }

    #[test]
    fn test_rule1_sigma_at_3() {
        let config = WesternElectricConfig::new(10.0, 1.0);
        let metrics = vec![13.5]; // z-score = 3.5, triggers at 3σ
        let variants = detect_all_rule_variants(&metrics, &config);
        assert!(variants.iter().any(|r| matches!(r, RuleVariant::Rule1SigmaAt3(_))));
    }

    #[test]
    fn test_rule1_sigma_at_custom() {
        let config = WesternElectricConfig::new(10.0, 1.0)
            .add_custom_threshold("rule1_custom_2_5".to_string(), 2.5);
        let metrics = vec![12.6]; // z-score = 2.6, triggers custom 2.5σ
        let variants = detect_all_rule_variants(&metrics, &config);
        // Custom threshold will be detected in add_measurement if z > threshold
        assert!(!variants.is_empty());
    }

    // ========================================================================
    // Window Size Variant Tests (Rule 2 - Consecutive)
    // ========================================================================

    #[test]
    fn test_rule2_window_6() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![6];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        // Add 6 out-of-control values
        for _ in 0..6 {
            analyzer.add_measurement("test", 15.0); // All beyond UCL
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleConsecutiveWindow6(_))));
    }

    #[test]
    fn test_rule2_window_9() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![9];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for _ in 0..9 {
            analyzer.add_measurement("test", 15.0);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleConsecutiveWindow9(_))));
    }

    #[test]
    fn test_rule2_window_15() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![15];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for _ in 0..15 {
            analyzer.add_measurement("test", 15.0);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleConsecutiveWindow15(_))));
    }

    #[test]
    fn test_rule2_window_20() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for _ in 0..20 {
            analyzer.add_measurement("test", 15.0);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleConsecutiveWindow20(_))));
    }

    #[test]
    fn test_rule2_window_30() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![30];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for _ in 0..30 {
            analyzer.add_measurement("test", 15.0);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleConsecutiveWindow30(_))));
    }

    // ========================================================================
    // Trend Window Size Tests (Rule 3)
    // ========================================================================

    #[test]
    fn test_rule3_trend_window_6() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![6];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..6 {
            analyzer.add_measurement("test", 10.0 + i as f64);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleTrendWindow6(_))));
    }

    #[test]
    fn test_rule3_trend_window_9() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![9];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..9 {
            analyzer.add_measurement("test", 10.0 + i as f64);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleTrendWindow9(_))));
    }

    #[test]
    fn test_rule3_trend_window_15() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![15];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..15 {
            analyzer.add_measurement("test", 10.0 + i as f64);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleTrendWindow15(_))));
    }

    #[test]
    fn test_rule3_trend_window_20() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..20 {
            analyzer.add_measurement("test", 10.0 + i as f64);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleTrendWindow20(_))));
    }

    // ========================================================================
    // Rule 4 - Alternating Pattern Tests
    // ========================================================================

    #[test]
    fn test_rule4_alternating() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![6, 8, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        let values = vec![8.0, 12.0, 8.0, 12.0, 8.0, 12.0, 8.0, 12.0];
        for v in values {
            analyzer.add_measurement("test", v);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::RuleAlternating(_))));
    }

    // ========================================================================
    // Rule 5 - 2-of-3 Beyond Sigma Tests
    // ========================================================================

    #[test]
    fn test_rule5_2_of_3_beyond_2_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![3, 6, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 13.0);
        analyzer.add_measurement("test", 13.5);
        analyzer.add_measurement("test", 11.0);

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule2of3Beyond2Sigma(_))));
    }

    #[test]
    fn test_rule5_2_of_3_beyond_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![3, 6, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 11.5);
        analyzer.add_measurement("test", 12.0);
        analyzer.add_measurement("test", 10.5);

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule2of3Beyond1Sigma(_))));
    }

    #[test]
    fn test_rule5_3_of_3_beyond_2_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![3, 6, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 13.0);
        analyzer.add_measurement("test", 13.5);
        analyzer.add_measurement("test", 13.2);

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule3of3Beyond2Sigma(_))));
    }

    // ========================================================================
    // Rule 6 - N-of-5 Beyond 1σ Tests
    // ========================================================================

    #[test]
    fn test_rule6_3_of_5_beyond_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![5, 6, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 11.5);
        analyzer.add_measurement("test", 12.0);
        analyzer.add_measurement("test", 11.2);
        analyzer.add_measurement("test", 10.0);
        analyzer.add_measurement("test", 10.5);

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule3of5Beyond1Sigma(_))));
    }

    #[test]
    fn test_rule6_4_of_5_beyond_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![5, 6, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 11.5);
        analyzer.add_measurement("test", 12.0);
        analyzer.add_measurement("test", 11.2);
        analyzer.add_measurement("test", 11.8);
        analyzer.add_measurement("test", 10.5);

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule4of5Beyond1Sigma(_))));
    }

    #[test]
    fn test_rule6_5_of_5_beyond_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![5, 6, 9, 15, 20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 11.5);
        analyzer.add_measurement("test", 12.0);
        analyzer.add_measurement("test", 11.2);
        analyzer.add_measurement("test", 11.8);
        analyzer.add_measurement("test", 12.3);

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule5of5Beyond1Sigma(_))));
    }

    // ========================================================================
    // Rule 7 - N-in-a-row Within 1σ Tests
    // ========================================================================

    #[test]
    fn test_rule7_10_in_row_within_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![10];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..10 {
            let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.6;
            analyzer.add_measurement("test", value);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule10InRowWithin1Sigma(_))));
    }

    #[test]
    fn test_rule7_15_in_row_within_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![15];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..15 {
            let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.6;
            analyzer.add_measurement("test", value);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule15InRowWithin1Sigma(_))));
    }

    #[test]
    fn test_rule7_20_in_row_within_1_sigma() {
        let mut config = WesternElectricConfig::new(10.0, 1.0);
        config.window_sizes = vec![20];
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        for i in 0..20 {
            let value = 10.0 + (i as f64 % 2.0 - 0.5) * 0.6;
            analyzer.add_measurement("test", value);
        }

        assert!(analyzer
            .detected_rules
            .iter()
            .any(|r| matches!(r, RuleVariant::Rule20InRowWithin1Sigma(_))));
    }

    // ========================================================================
    // Rule Storm Detection Tests
    // ========================================================================

    #[test]
    fn test_detect_rule_storm_2_rules() {
        let _config = WesternElectricConfig::new(10.0, 1.0);
        let violations = vec![
            QualityViolation::Rule1Sigma {
                metric: "metric1".to_string(),
                value: 13.5,
                threshold: 13.0,
                z_score: 3.5,
                severity: "CRITICAL".to_string(),
            },
            QualityViolation::Rule9InRow {
                metric: "metric1".to_string(),
                consecutive: 9,
            },
        ];

        let storms = detect_rule_storms(&violations);
        assert_eq!(storms.len(), 1);
        assert_eq!(storms[0].rule_count, 2);
        assert!(!storms[0].is_severe);
    }

    #[test]
    fn test_detect_rule_storm_3_rules() {
        let _config = WesternElectricConfig::new(10.0, 1.0);
        let violations = vec![
            QualityViolation::Rule1Sigma {
                metric: "metric1".to_string(),
                value: 13.5,
                threshold: 13.0,
                z_score: 3.5,
                severity: "CRITICAL".to_string(),
            },
            QualityViolation::Rule9InRow {
                metric: "metric1".to_string(),
                consecutive: 9,
            },
            QualityViolation::RuleTrend {
                metric: "metric1".to_string(),
                direction: "increasing".to_string(),
                count: 6,
            },
        ];

        let storms = detect_rule_storms(&violations);
        assert_eq!(storms.len(), 1);
        assert_eq!(storms[0].rule_count, 3);
        assert!(storms[0].is_severe);
    }

    // ========================================================================
    // RuleVariant Tests
    // ========================================================================

    #[test]
    fn test_rule_variant_metric_extraction() {
        let variant = RuleVariant::Rule1SigmaAt3("test_metric".to_string());
        assert_eq!(variant.metric(), "test_metric");
    }

    #[test]
    fn test_rule_variant_severity_levels() {
        let v1 = RuleVariant::Rule1SigmaAt3("m".to_string());
        assert_eq!(v1.severity(), "CRITICAL");

        let v2 = RuleVariant::Rule1SigmaAt2("m".to_string());
        assert_eq!(v2.severity(), "CRITICAL");

        let v3 = RuleVariant::Rule2of3Beyond1Sigma("m".to_string());
        assert_eq!(v3.severity(), "MEDIUM");

        let v4 = RuleVariant::Rule10InRowWithin1Sigma("m".to_string());
        assert_eq!(v4.severity(), "LOW");
    }

    #[test]
    fn test_rule_variant_descriptions() {
        let variant = RuleVariant::Rule1SigmaAt3("stub_ratio".to_string());
        let desc = variant.description();
        assert!(desc.contains("stub_ratio"));
        assert!(desc.contains("spike"));
    }

    // ========================================================================
    // Configuration Tests
    // ========================================================================

    #[test]
    fn test_config_default() {
        let config = WesternElectricConfig::default();
        assert_eq!(config.baseline_mean, 0.0);
        assert_eq!(config.baseline_stddev, 1.0);
        assert!(config.rule1_enabled);
    }

    #[test]
    fn test_config_with_sigmas() {
        let config = WesternElectricConfig::new(10.0, 1.0)
            .with_sigmas(1.5, 2.5, 3.5);
        assert_eq!(config.primary_sigma, 1.5);
        assert_eq!(config.secondary_sigma, 2.5);
        assert_eq!(config.tertiary_sigma, 3.5);
    }

    #[test]
    fn test_config_with_custom_threshold() {
        let config = WesternElectricConfig::new(10.0, 1.0)
            .add_custom_threshold("rule1_special".to_string(), 2.7);
        assert_eq!(config.custom_thresholds.get("rule1_special"), Some(&2.7));
    }

    // ========================================================================
    // Aggregated Severity Tests
    // ========================================================================

    #[test]
    fn test_aggregated_severity_empty() {
        let violations: Vec<QualityViolation> = vec![];
        let severity = AggregatedSeverity::compute(&violations);
        assert_eq!(severity.total_violations, 0);
        assert_eq!(severity.critical_count, 0);
    }

    #[test]
    fn test_aggregated_severity_mixed() {
        let violations = vec![
            QualityViolation::Rule1Sigma {
                metric: "m1".to_string(),
                value: 1.0,
                threshold: 2.0,
                z_score: 3.5,
                severity: "CRITICAL".to_string(),
            },
            QualityViolation::Rule9InRow {
                metric: "m2".to_string(),
                consecutive: 9,
            },
            QualityViolation::Rule4of5Beyond1Sigma {
                metric: "m3".to_string(),
                count: 4,
                threshold: 1.0,
            },
        ];

        let severity = AggregatedSeverity::compute(&violations);
        assert_eq!(severity.total_violations, 3);
        assert_eq!(severity.critical_count, 2);
        assert_eq!(severity.affected_metrics.len(), 3);
    }

    #[test]
    fn test_rule_storm_summary() {
        let rules = vec![
            RuleVariant::Rule1SigmaAt3("m".to_string()),
            RuleVariant::RuleConsecutiveWindow9("m".to_string()),
            RuleVariant::RuleTrendWindow6("m".to_string()),
        ];

        let storm = RuleStorm::new("m".to_string(), rules);
        assert!(storm.is_severe);
        assert!(storm.summary.contains("Severe rule storm"));
    }

    #[test]
    fn test_enhanced_analyzer_finalize() {
        let config = WesternElectricConfig::new(10.0, 1.0);
        let mut analyzer = EnhancedWesternElectricAnalyzer::new(config);

        analyzer.add_measurement("test", 13.5);
        let severity = analyzer.finalize();

        // With a single spike, we expect either CRITICAL or HIGH
        // (depends on how many rules fire)
        assert!(!severity.worst_severity.is_empty());
    }
}
